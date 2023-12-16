use borsh::{BorshDeserialize, BorshSerialize};
use jito_restaking_sanitization::{assert_with_msg, realloc};
use solana_program::{
    account_info::AccountInfo, entrypoint_deprecated::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent,
};

use crate::AccountType;

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct Avs {
    /// The account type
    account_type: AccountType,

    /// The base account used as a PDA seed
    base: Pubkey,

    /// The admin of the AVS
    admin: Pubkey,

    /// The index of the AVS
    avs_index: u64,

    /// Reserved space
    reserved: [u8; 1024],

    /// The bump seed for the PDA
    bump: u8,
}

impl Avs {
    pub const fn new(base: Pubkey, admin: Pubkey, avs_index: u64, bump: u8) -> Self {
        Self {
            account_type: AccountType::Avs,
            base,
            admin,
            avs_index,
            reserved: [0; 1024],
            bump,
        }
    }

    pub const fn base(&self) -> Pubkey {
        self.base
    }

    pub const fn admin(&self) -> Pubkey {
        self.admin
    }

    pub const fn avs_index(&self) -> u64 {
        self.avs_index
    }

    pub const fn bump(&self) -> u8 {
        self.bump
    }

    pub fn seeds(base: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([b"avs".to_vec(), base.as_ref().to_vec()])
    }

    pub fn find_program_address(program_id: &Pubkey, base: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(base);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
    ) -> Result<Self, ProgramError> {
        assert_with_msg(
            !account.data_is_empty(),
            ProgramError::UninitializedAccount,
            "AVS account is not initialized",
        )?;
        assert_with_msg(
            account.owner == program_id,
            ProgramError::IllegalOwner,
            "AVS account not owned by the correct program",
        )?;

        // The AvsState shall be properly deserialized and valid struct
        let avs_state = Self::deserialize(&mut account.data.borrow_mut().as_ref())?;
        assert_with_msg(
            avs_state.account_type == AccountType::Avs,
            ProgramError::InvalidAccountData,
            "AVS account is invalid",
        )?;

        // The AvsState shall be at the correct PDA as defined by the seeds and bump
        let mut seeds = Self::seeds(&avs_state.base());
        seeds.push(vec![avs_state.bump()]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)?;

        assert_with_msg(
            expected_pubkey == *account.key,
            ProgramError::InvalidAccountData,
            "AVS account is not at the correct PDA",
        )?;

        Ok(avs_state)
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct AvsOperator {
    /// operator in the AVS
    operator: Pubkey,
    /// slot the operator was last added
    slot_added: u64,
    /// slot the operator was last removed
    slot_removed: u64,
    /// reserved space
    reserved: [u8; 256],
}

pub enum AvsOperatorState {
    Inactive,
    WarmingUp,
    Active,
    CoolingDown,
}

impl AvsOperator {
    pub const fn new(operator: Pubkey, slot_added: u64) -> Self {
        Self {
            operator,
            slot_added,
            slot_removed: 0,
            reserved: [0; 256],
        }
    }

    pub const fn operator(&self) -> Pubkey {
        self.operator
    }

    pub const fn slot_added(&self) -> u64 {
        self.slot_added
    }

    pub const fn slot_removed(&self) -> u64 {
        self.slot_removed
    }

    pub fn set_slot_added(&mut self, slot: u64, num_slots_per_epoch: u64) -> bool {
        match self.state(slot, num_slots_per_epoch) {
            AvsOperatorState::Inactive => {
                self.slot_added = slot;
                true
            }
            _ => false,
        }
    }

    pub fn set_slot_removed(&mut self, slot: u64, num_slots_per_epoch: u64) -> bool {
        match self.state(slot, num_slots_per_epoch) {
            AvsOperatorState::Active => {
                self.slot_removed = slot;
                true
            }
            _ => false,
        }
    }

    pub fn state(&self, slot: u64, num_slots_per_epoch: u64) -> AvsOperatorState {
        if self.slot_removed > self.slot_added {
            // either cooling down or inactive
            let epoch = self.slot_removed.checked_div(num_slots_per_epoch).unwrap();
            let inactive_epoch = epoch.checked_add(2).unwrap();
            if slot < inactive_epoch.checked_mul(num_slots_per_epoch).unwrap() {
                AvsOperatorState::CoolingDown
            } else {
                AvsOperatorState::Inactive
            }
        } else {
            let epoch = self.slot_added.checked_div(num_slots_per_epoch).unwrap();
            let active_epoch = epoch.checked_add(2).unwrap();
            if slot < active_epoch.checked_mul(num_slots_per_epoch).unwrap() {
                AvsOperatorState::WarmingUp
            } else {
                AvsOperatorState::Active
            }
        }
    }
}

/// The AVS operator list stores a list of validators in the AVS validator set
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct AvsOperatorList {
    /// The account type
    account_type: AccountType,

    /// The AVS
    avs: Pubkey,

    /// The list of operators in the AVS
    operators: Vec<AvsOperator>,

    /// Reserved space
    reserved: [u8; 1024],

    /// The bump seed for the PDA
    bump: u8,
}

impl AvsOperatorList {
    pub const fn new(avs: Pubkey, bump: u8) -> Self {
        Self {
            account_type: AccountType::AvsOperatorList,
            avs,
            bump,
            operators: vec![],
            reserved: [0; 1024],
        }
    }

    pub fn add_operator(&mut self, operator: Pubkey, slot: u64, num_slots_per_epoch: u64) -> bool {
        let maybe_operator = self.operators.iter_mut().find(|a| a.operator() == operator);
        if let Some(operator) = maybe_operator {
            operator.set_slot_added(slot, num_slots_per_epoch)
        } else {
            self.operators.push(AvsOperator::new(operator, slot));
            true
        }
    }

    pub fn remove_operator(
        &mut self,
        operator: Pubkey,
        slot: u64,
        num_slots_per_epoch: u64,
    ) -> bool {
        let maybe_operator = self.operators.iter_mut().find(|a| a.operator() == operator);
        maybe_operator.map_or(false, |operator| {
            operator.set_slot_removed(slot, num_slots_per_epoch)
        })
    }

    pub const fn avs(&self) -> Pubkey {
        self.avs
    }

    pub fn operators(&self) -> &[AvsOperator] {
        &self.operators
    }

    pub const fn bump(&self) -> u8 {
        self.bump
    }

    pub fn seeds(avs: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([b"avs_operator_list".to_vec(), avs.as_ref().to_vec()])
    }

    pub fn find_program_address(program_id: &Pubkey, avs: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(avs);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        avs: &Pubkey,
    ) -> Result<Self, ProgramError> {
        assert_with_msg(
            !account.data_is_empty(),
            ProgramError::UninitializedAccount,
            "AVS Operator List account is not initialized",
        )?;
        assert_with_msg(
            account.owner == program_id,
            ProgramError::IllegalOwner,
            "AVS Operator List account not owned by the correct program",
        )?;

        // The AvsState shall be properly deserialized and valid struct
        let avs_operator_list = Self::deserialize(&mut account.data.borrow_mut().as_ref())?;
        assert_with_msg(
            avs_operator_list.account_type == AccountType::AvsOperatorList,
            ProgramError::InvalidAccountData,
            "AVS Operator List account is invalid",
        )?;
        assert_with_msg(
            avs_operator_list.avs == *avs,
            ProgramError::InvalidAccountData,
            "AVS Operator List account is not for the correct AVS",
        )?;

        // The AvsState shall be at the correct PDA as defined by the seeds and bump
        let mut seeds = Self::seeds(avs);
        seeds.push(vec![avs_operator_list.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)?;

        assert_with_msg(
            expected_pubkey == *account.key,
            ProgramError::InvalidAccountData,
            "AVS Operator List account is not at the correct PDA",
        )?;

        Ok(avs_operator_list)
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct AvsVaultInfo {
    /// The vault account
    vault: Pubkey,

    /// The slot when the vault was last added
    slot_added: u64,

    /// The slot when the vault was last removed
    slot_removed: u64,

    /// Reserved space
    reserved: [u8; 256],
}

pub enum AvsVaultState {
    Inactive,
    WarmingUp,
    Active,
    CoolingDown,
}

impl AvsVaultInfo {
    pub const fn new(vault: Pubkey, slot_added: u64) -> Self {
        Self {
            vault,
            slot_added,
            slot_removed: 0,
            reserved: [0; 256],
        }
    }

    pub const fn vault(&self) -> Pubkey {
        self.vault
    }

    pub const fn slot_added(&self) -> u64 {
        self.slot_added
    }

    pub const fn slot_removed(&self) -> u64 {
        self.slot_removed
    }

    pub fn set_slot_added(&mut self, slot: u64, _num_slots_per_epoch: u64) -> bool {
        // match self.state(slot, num_slots_per_epoch) {
        //     AvsVaultState::Inactive => {
        //         self.slot_added = slot;
        //         true
        //     }
        //     _ => false,
        // }
        self.slot_added = slot;
        true
    }

    pub fn set_slot_removed(&mut self, slot: u64, _num_slots_per_epoch: u64) -> bool {
        // match self.state(slot, num_slots_per_epoch) {
        //     AvsVaultState::Active => {
        //         self.slot_removed = slot;
        //         true
        //     }
        //     _ => false,
        // }
        self.slot_removed = slot;
        true
    }

    pub fn state(&self, slot: u64, num_slots_per_epoch: u64) -> AvsVaultState {
        if self.slot_removed > self.slot_added {
            // either cooling down or inactive
            let epoch = self.slot_removed.checked_div(num_slots_per_epoch).unwrap();
            let inactive_epoch = epoch.checked_add(2).unwrap();
            if slot < inactive_epoch.checked_mul(num_slots_per_epoch).unwrap() {
                AvsVaultState::CoolingDown
            } else {
                AvsVaultState::Inactive
            }
        } else {
            let epoch = self.slot_added.checked_div(num_slots_per_epoch).unwrap();
            let active_epoch = epoch.checked_add(2).unwrap();
            if slot < active_epoch.checked_mul(num_slots_per_epoch).unwrap() {
                AvsVaultState::WarmingUp
            } else {
                AvsVaultState::Active
            }
        }
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct AvsVaultList {
    /// The account type
    account_type: AccountType,

    /// The AVS
    avs: Pubkey,

    /// The list of supported vaults by the AVS
    /// Doesn't necessarily mean they're delegated to the AVS
    vault_list: Vec<AvsVaultInfo>,

    /// Reserved space
    reserved: [u8; 1024],

    /// The bump seed for the PDA
    bump: u8,
}

impl AvsVaultList {
    pub const fn new(avs: Pubkey, bump: u8) -> Self {
        Self {
            account_type: AccountType::AvsVaultList,
            avs,
            bump,
            vault_list: Vec::new(),
            reserved: [0; 1024],
        }
    }

    pub const fn avs(&self) -> Pubkey {
        self.avs
    }

    pub fn vault_list(&self) -> &[AvsVaultInfo] {
        &self.vault_list
    }

    pub fn contains_vault(&self, vault: Pubkey) -> bool {
        self.vault_list.iter().any(|v| v.vault() == vault)
    }

    pub fn add_vault(&mut self, vault: Pubkey, slot: u64, num_slots_per_epoch: u64) -> bool {
        let maybe_vault = self.vault_list.iter_mut().find(|v| v.vault() == vault);
        if let Some(vault) = maybe_vault {
            vault.set_slot_added(slot, num_slots_per_epoch)
        } else {
            self.vault_list.push(AvsVaultInfo::new(vault, slot));
            true
        }
    }

    pub fn remove_vault(&mut self, vault: Pubkey, slot: u64, num_slots_per_epoch: u64) -> bool {
        let maybe_vault = self.vault_list.iter_mut().find(|v| v.vault() == vault);
        maybe_vault.map_or(false, |vault| {
            vault.set_slot_removed(slot, num_slots_per_epoch)
        })
    }

    pub fn seeds(avs: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([b"avs_vault_list".to_vec(), avs.as_ref().to_vec()])
    }

    pub fn find_program_address(program_id: &Pubkey, avs: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(avs);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        avs: &Pubkey,
    ) -> Result<Self, ProgramError> {
        assert_with_msg(
            !account.data_is_empty(),
            ProgramError::UninitializedAccount,
            "AVS Vault List account is not initialized",
        )?;
        assert_with_msg(
            account.owner == program_id,
            ProgramError::IllegalOwner,
            "AVS Vault List account not owned by the correct program",
        )?;

        // The AvsState shall be properly deserialized and valid struct
        let avs_vault_list = Self::deserialize(&mut account.data.borrow_mut().as_ref())?;
        assert_with_msg(
            avs_vault_list.account_type == AccountType::AvsVaultList,
            ProgramError::InvalidAccountData,
            "AVS Vault List account is invalid",
        )?;
        assert_with_msg(
            avs_vault_list.avs == *avs,
            ProgramError::InvalidAccountData,
            "AVS Vault List account is not for the correct AVS",
        )?;

        // The AvsState shall be at the correct PDA as defined by the seeds and bump
        let mut seeds = Self::seeds(avs);
        seeds.push(vec![avs_vault_list.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)?;

        assert_with_msg(
            expected_pubkey == *account.key,
            ProgramError::InvalidAccountData,
            "AVS Vault List account is not at the correct PDA",
        )?;

        Ok(avs_vault_list)
    }
}

pub struct SanitizedAvs<'a, 'info> {
    account: &'a AccountInfo<'info>,
    avs: Avs,
}

impl<'a, 'info> SanitizedAvs<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
    ) -> Result<SanitizedAvs<'a, 'info>, ProgramError> {
        if expect_writable {
            assert_with_msg(
                account.is_writable,
                ProgramError::InvalidAccountData,
                "Invalid writable flag for AVS",
            )?;
        }
        let avs = Avs::deserialize_checked(program_id, account)?;

        Ok(SanitizedAvs { account, avs })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn avs(&self) -> &Avs {
        &self.avs
    }

    pub fn avs_mut(&mut self) -> &mut Avs {
        &mut self.avs
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(&mut self.account.data.borrow_mut()[..], &self.avs)?;
        Ok(())
    }
}

pub struct SanitizedAvsOperatorList<'a, 'info> {
    account: &'a AccountInfo<'info>,
    avs_operator_list: AvsOperatorList,
}

impl<'a, 'info> SanitizedAvsOperatorList<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        avs: &Pubkey,
    ) -> Result<SanitizedAvsOperatorList<'a, 'info>, ProgramError> {
        if expect_writable {
            assert_with_msg(
                account.is_writable,
                ProgramError::InvalidAccountData,
                "Invalid writable flag for AVS Operator List",
            )?;
        }
        let avs_operator_list = AvsOperatorList::deserialize_checked(program_id, account, avs)?;

        Ok(SanitizedAvsOperatorList {
            account,
            avs_operator_list,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn avs_operator_list(&self) -> &AvsOperatorList {
        &self.avs_operator_list
    }

    pub fn avs_operator_list_mut(&mut self) -> &mut AvsOperatorList {
        &mut self.avs_operator_list
    }

    pub fn save(&self, rent: &Rent, payer: &'a AccountInfo<'info>) -> ProgramResult {
        let serialized = self.avs_operator_list.try_to_vec()?;

        if serialized.len() > self.account.data.borrow().len() {
            realloc(self.account, serialized.len(), payer, rent)?;
        }

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }
}

pub struct SanitizedAvsVaultList<'a, 'info> {
    account: &'a AccountInfo<'info>,
    avs_vault_list: AvsVaultList,
}

impl<'a, 'info> SanitizedAvsVaultList<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        avs: &Pubkey,
    ) -> Result<SanitizedAvsVaultList<'a, 'info>, ProgramError> {
        if expect_writable {
            assert_with_msg(
                account.is_writable,
                ProgramError::InvalidAccountData,
                "Invalid writable flag for AVS Vault List",
            )?;
        }
        let avs_vault_list = AvsVaultList::deserialize_checked(program_id, account, avs)?;

        Ok(SanitizedAvsVaultList {
            account,
            avs_vault_list,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn avs_vault_list(&self) -> &AvsVaultList {
        &self.avs_vault_list
    }

    pub fn avs_vault_list_mut(&mut self) -> &mut AvsVaultList {
        &mut self.avs_vault_list
    }

    pub fn save(&self, rent: &Rent, payer: &'a AccountInfo<'info>) -> ProgramResult {
        let serialized = self.avs_vault_list.try_to_vec()?;

        if serialized.len() > self.account.data.borrow().len() {
            realloc(self.account, serialized.len(), payer, rent)?;
        }

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);
        Ok(())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct AvsSlasher {
    /// The vault account this slasher can slash
    vault: Pubkey,

    /// The slasher signer
    slasher: Pubkey,

    /// The max slashable funds per epoch
    max_slashable_per_epoch: u64,

    /// Slot the slasher was added
    slot_added: u64,

    /// Slot the slasher was deprecated
    slot_deprecated: u64,

    /// true if the slasher is enabled, false if not
    enabled: bool,

    /// Reserved space
    reserved: [u8; 64],
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct AvsSlasherList {
    /// The account type
    account_type: AccountType,

    /// The AVS
    avs: Pubkey,

    /// The slashers approved for the AVS
    slashers: Vec<AvsSlasher>,

    /// Reserved space
    reserved: [u8; 256],

    /// The bump seed for the PDA
    bump: u8,
}

impl AvsSlasherList {
    pub const fn new(avs: Pubkey, avs_slasher_list_bump: u8) -> Self {
        Self {
            account_type: AccountType::AvsSlasherList,
            avs,
            slashers: vec![],
            reserved: [0; 256],
            bump: avs_slasher_list_bump,
        }
    }

    /// Adds the slasher for a given vault.
    /// A (slasher, vault) pair can only be added once. Once added, it can't be modified.
    ///
    /// # Arguments
    /// * `vault` - The vault account
    /// * `slasher` - The slasher account
    /// * `slot` - The current slot
    /// * `max_slashable_per_epoch` - The max slashable funds that can be slashed for a given node operator
    /// per epoch.
    pub fn add_slasher(
        &mut self,
        vault: Pubkey,
        slasher: Pubkey,
        slot: u64,
        max_slashable_per_epoch: u64,
    ) -> bool {
        let maybe_slasher = self
            .slashers
            .iter_mut()
            .find(|s| s.vault == vault && s.slasher == slasher);
        if maybe_slasher.is_some() {
            false
        } else {
            self.slashers.push(AvsSlasher {
                vault,
                slasher,
                max_slashable_per_epoch,
                slot_added: slot,
                slot_deprecated: u64::MAX,
                enabled: true,
                reserved: [0; 64],
            });
            true
        }
    }

    pub fn deprecate_slasher(&mut self, vault: Pubkey, slasher: Pubkey, slot: u64) -> bool {
        let maybe_slasher = self
            .slashers
            .iter_mut()
            .find(|s| s.vault == vault && s.slasher == slasher && s.enabled);
        maybe_slasher.map_or(false, |slasher| {
            slasher.enabled = false;
            slasher.slot_deprecated = slot;
            true
        })
    }

    pub fn seeds(avs: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([b"avs_slasher_list".to_vec(), avs.as_ref().to_vec()])
    }

    pub fn find_program_address(program_id: &Pubkey, avs: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(avs);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        avs: &Pubkey,
    ) -> Result<Self, ProgramError> {
        assert_with_msg(
            !account.data_is_empty(),
            ProgramError::UninitializedAccount,
            "AVS Slasher List account is not initialized",
        )?;
        assert_with_msg(
            account.owner == program_id,
            ProgramError::IllegalOwner,
            "AVS Slasher List account not owned by the correct program",
        )?;

        // The AvsState shall be properly deserialized and valid struct
        let avs_slasher_list = Self::deserialize(&mut account.data.borrow_mut().as_ref())?;
        assert_with_msg(
            avs_slasher_list.account_type == AccountType::AvsSlasherList,
            ProgramError::InvalidAccountData,
            "AVS Slasher List account is invalid",
        )?;
        assert_with_msg(
            avs_slasher_list.avs == *avs,
            ProgramError::InvalidAccountData,
            "AVS Slasher List account is not for the correct AVS",
        )?;

        // The AvsState shall be at the correct PDA as defined by the seeds and bump
        let mut seeds = Self::seeds(avs);
        seeds.push(vec![avs_slasher_list.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)?;

        assert_with_msg(
            expected_pubkey == *account.key,
            ProgramError::InvalidAccountData,
            "AVS Slasher List account is not at the correct PDA",
        )?;

        Ok(avs_slasher_list)
    }
}

pub struct SanitizedAvsSlasherList<'a, 'info> {
    account: &'a AccountInfo<'info>,
    avs_slasher_list: AvsSlasherList,
}

impl<'a, 'info> SanitizedAvsSlasherList<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        avs: &Pubkey,
    ) -> Result<SanitizedAvsSlasherList<'a, 'info>, ProgramError> {
        if expect_writable {
            assert_with_msg(
                account.is_writable,
                ProgramError::InvalidAccountData,
                "Invalid writable flag for AVS Slasher List",
            )?;
        }
        let avs_slasher_list = AvsSlasherList::deserialize_checked(program_id, account, avs)?;

        Ok(SanitizedAvsSlasherList {
            account,
            avs_slasher_list,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn avs_slasher_list(&self) -> &AvsSlasherList {
        &self.avs_slasher_list
    }

    pub fn avs_slasher_list_mut(&mut self) -> &mut AvsSlasherList {
        &mut self.avs_slasher_list
    }

    pub fn save(&self, rent: &Rent, payer: &'a AccountInfo<'info>) -> ProgramResult {
        let serialized = self.avs_slasher_list.try_to_vec()?;

        if serialized.len() > self.account.data.borrow().len() {
            realloc(self.account, serialized.len(), payer, rent)?;
        }

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);
        Ok(())
    }
}
