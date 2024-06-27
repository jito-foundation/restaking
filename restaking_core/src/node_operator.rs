use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use jito_restaking_sanitization::{assert_with_msg, realloc};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent,
};

use crate::{vault::RestakingVault, AccountType};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct NodeOperator {
    /// The account type
    account_type: AccountType,

    /// The base pubkey used as a seed for the PDA
    base: Pubkey,

    /// The admin pubkey
    admin: Pubkey,

    /// The voter pubkey
    voter: Pubkey,

    /// The node operator index
    index: u64,

    /// Reserved space
    reserved_space: [u8; 1024],

    /// The bump seed for the PDA
    bump: u8,
}

impl NodeOperator {
    pub const fn new(base: Pubkey, admin: Pubkey, voter: Pubkey, index: u64, bump: u8) -> Self {
        Self {
            account_type: AccountType::NodeOperator,
            base,
            admin,
            voter,
            index,
            reserved_space: [0; 1024],
            bump,
        }
    }

    pub const fn index(&self) -> u64 {
        self.index
    }

    pub const fn base(&self) -> Pubkey {
        self.base
    }

    pub const fn bump(&self) -> u8 {
        self.bump
    }

    pub const fn admin(&self) -> Pubkey {
        self.admin
    }

    pub fn set_admin(&mut self, admin: Pubkey) {
        self.admin = admin;
    }

    pub const fn voter(&self) -> Pubkey {
        self.voter
    }

    pub fn set_voter(&mut self, voter: Pubkey) {
        self.voter = voter;
    }

    pub fn seeds(base: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([b"node_operator".to_vec(), base.as_ref().to_vec()])
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
            "Node Operator account is not initialized",
        )?;
        assert_with_msg(
            account.owner == program_id,
            ProgramError::IllegalOwner,
            "Node Operator account is not owned by the program",
        )?;

        // The AvsState shall be properly deserialized and valid struct
        let node_operator = Self::deserialize(&mut account.data.borrow_mut().as_ref())?;
        assert_with_msg(
            node_operator.account_type == AccountType::NodeOperator,
            ProgramError::InvalidAccountData,
            "Node Operator account is not valid",
        )?;

        // The AvsState shall be at the correct PDA as defined by the seeds and bump
        let mut seeds = Self::seeds(&node_operator.base);
        seeds.push(vec![node_operator.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)?;

        assert_with_msg(
            expected_pubkey == *account.key,
            ProgramError::InvalidAccountData,
            "Node Operator account is not at the correct PDA",
        )?;

        Ok(node_operator)
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct NodeOperatorAvs {
    /// The AVS account
    avs: Pubkey,

    state: SlotToggle,

    /// Reserved space
    reserved: [u8; 256],
}

impl NodeOperatorAvs {
    pub const fn new(avs: Pubkey, slot_added: u64) -> Self {
        Self {
            avs,
            state: SlotToggle::new(slot_added),
            reserved: [0; 256],
        }
    }

    pub const fn avs(&self) -> Pubkey {
        self.avs
    }

    pub const fn state(&self) -> &SlotToggle {
        &self.state
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct NodeOperatorAvsList {
    account_type: AccountType,

    operator: Pubkey,

    bump: u8,

    avs: Vec<NodeOperatorAvs>,
}

impl NodeOperatorAvsList {
    pub const fn new(operator: Pubkey, bump: u8) -> Self {
        Self {
            account_type: AccountType::NodeOperatorAvsList,
            operator,
            bump,
            avs: vec![],
        }
    }

    pub const fn operator(&self) -> Pubkey {
        self.operator
    }

    pub fn avs_list(&self) -> &[NodeOperatorAvs] {
        &self.avs
    }

    pub fn add_avs(&mut self, avs: Pubkey, slot: u64) -> bool {
        let maybe_avs = self.avs.iter_mut().find(|a| a.avs() == avs);
        if let Some(avs) = maybe_avs {
            avs.state.activate(slot)
        } else {
            self.avs.push(NodeOperatorAvs::new(avs, slot));
            true
        }
    }

    pub fn remove_avs(&mut self, avs: Pubkey, slot: u64) -> bool {
        let maybe_avs = self.avs.iter_mut().find(|a| a.avs() == avs);
        maybe_avs.map_or(false, |avs| avs.state.deactivate(slot))
    }

    pub fn contains_active_avs(&self, avs: &Pubkey, slot: u64) -> bool {
        self.avs
            .iter()
            .any(|a| a.avs() == *avs && a.state.is_active(slot))
    }

    pub fn seeds(node_operator: &Pubkey) -> Vec<Vec<u8>> {
        vec![
            b"node_operator_avs_list".to_vec(),
            node_operator.to_bytes().to_vec(),
        ]
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        node_operator: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(node_operator);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        node_operator: &Pubkey,
    ) -> Result<Self, ProgramError> {
        assert_with_msg(
            !account.data_is_empty(),
            ProgramError::UninitializedAccount,
            "Node Operator AVS List account is not initialized",
        )?;
        assert_with_msg(
            account.owner == program_id,
            ProgramError::IllegalOwner,
            "Node Operator AVS List account is not owned by the program",
        )?;

        // The AvsState shall be properly deserialized and valid struct
        let node_operator_avs_list = Self::deserialize(&mut account.data.borrow_mut().as_ref())?;
        assert_with_msg(
            node_operator_avs_list.account_type == AccountType::NodeOperatorAvsList,
            ProgramError::InvalidAccountData,
            "Node Operator AVS List account is not valid",
        )?;
        assert_with_msg(
            node_operator_avs_list.operator == *node_operator,
            ProgramError::InvalidAccountData,
            "Node Operator AVS List account is not for the correct node operator",
        )?;

        // The AvsState shall be at the correct PDA as defined by the seeds and bump
        let mut seeds = Self::seeds(node_operator);
        seeds.push(vec![node_operator_avs_list.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)?;

        assert_with_msg(
            expected_pubkey == *account.key,
            ProgramError::InvalidAccountData,
            "Node Operator AVS List account is not at the correct PDA",
        )?;

        Ok(node_operator_avs_list)
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct OperatorVaultList {
    account_type: AccountType,

    operator: Pubkey,

    bump: u8,

    vaults: Vec<RestakingVault>,
}

impl OperatorVaultList {
    pub const fn new(operator: Pubkey, bump: u8) -> Self {
        Self {
            account_type: AccountType::NodeOperatorVaultList,
            operator,
            bump,
            vaults: vec![],
        }
    }

    pub const fn operator(&self) -> Pubkey {
        self.operator
    }

    pub fn vault_list(&self) -> &[RestakingVault] {
        &self.vaults
    }

    pub fn add_vault(&mut self, vault: Pubkey, slot: u64) -> bool {
        let maybe_vault = self.vaults.iter_mut().find(|v| v.vault() == vault);
        if let Some(vault) = maybe_vault {
            vault.state_mut().activate(slot)
        } else {
            self.vaults.push(RestakingVault::new(vault, slot));
            true
        }
    }

    pub fn remove_vault(&mut self, vault: Pubkey, slot: u64) -> bool {
        let maybe_vault = self.vaults.iter_mut().find(|v| v.vault() == vault);
        maybe_vault.map_or(false, |vault| vault.state_mut().deactivate(slot))
    }

    pub fn seeds(operator: &Pubkey) -> Vec<Vec<u8>> {
        vec![
            b"node_operator_vault_list".to_vec(),
            operator.to_bytes().to_vec(),
        ]
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        operator: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(operator);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        node_operator: &Pubkey,
    ) -> Result<Self, ProgramError> {
        assert_with_msg(
            !account.data_is_empty(),
            ProgramError::UninitializedAccount,
            "Node Operator Vault List account is not initialized",
        )?;
        assert_with_msg(
            account.owner == program_id,
            ProgramError::IllegalOwner,
            "Node Operator Vault List account is not owned by the program",
        )?;

        // The AvsState shall be properly deserialized and valid struct
        let node_operator_vault_list = Self::deserialize(&mut account.data.borrow_mut().as_ref())?;
        assert_with_msg(
            node_operator_vault_list.account_type == AccountType::NodeOperatorVaultList,
            ProgramError::InvalidAccountData,
            "Node Operator Vault List account is not valid",
        )?;
        assert_with_msg(
            node_operator_vault_list.operator == *node_operator,
            ProgramError::InvalidAccountData,
            "Node Operator Vault List account is not for the correct node operator",
        )?;

        // The AvsState shall be at the correct PDA as defined by the seeds and bump
        let mut seeds = Self::seeds(node_operator);
        seeds.push(vec![node_operator_vault_list.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)?;

        assert_with_msg(
            expected_pubkey == *account.key,
            ProgramError::InvalidAccountData,
            "Node Operator Vault List account is not at the correct PDA",
        )?;

        Ok(node_operator_vault_list)
    }
}

pub struct SanitizedNodeOperator<'a, 'info> {
    account: &'a AccountInfo<'info>,
    node_operator: NodeOperator,
}

impl<'a, 'info> SanitizedNodeOperator<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
    ) -> Result<Self, ProgramError> {
        if expect_writable {
            assert_with_msg(
                account.is_writable,
                ProgramError::InvalidAccountData,
                "Node Operator account is not writable",
            )?;
        }

        let node_operator = NodeOperator::deserialize_checked(program_id, account)?;

        Ok(Self {
            account,
            node_operator,
        })
    }

    pub const fn account(&self) -> &'a AccountInfo<'info> {
        self.account
    }

    pub const fn node_operator(&self) -> &NodeOperator {
        &self.node_operator
    }

    pub fn node_operator_mut(&mut self) -> &mut NodeOperator {
        &mut self.node_operator
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(&mut self.account.data.borrow_mut()[..], &self.node_operator)?;
        Ok(())
    }
}

pub struct SanitizedNodeOperatorAvsList<'a, 'info> {
    account: &'a AccountInfo<'info>,
    node_operator_avs_list: NodeOperatorAvsList,
}

impl<'a, 'info> SanitizedNodeOperatorAvsList<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        node_operator: &Pubkey,
    ) -> Result<Self, ProgramError> {
        if expect_writable {
            assert_with_msg(
                account.is_writable,
                ProgramError::InvalidAccountData,
                "Node Operator AVS List account is not writable",
            )?;
        }

        let node_operator_avs_list =
            NodeOperatorAvsList::deserialize_checked(program_id, account, node_operator)?;

        Ok(Self {
            account,
            node_operator_avs_list,
        })
    }

    pub const fn account(&self) -> &'a AccountInfo<'info> {
        self.account
    }

    pub const fn node_operator_avs_list(&self) -> &NodeOperatorAvsList {
        &self.node_operator_avs_list
    }

    pub fn node_operator_avs_list_mut(&mut self) -> &mut NodeOperatorAvsList {
        &mut self.node_operator_avs_list
    }

    pub fn save_with_realloc(&self, rent: &Rent, payer: &'a AccountInfo<'info>) -> ProgramResult {
        let serialized = self.node_operator_avs_list.try_to_vec()?;

        if serialized.len() > self.account.data.borrow().len() {
            realloc(self.account, serialized.len(), payer, rent)?;
        }

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }

    pub fn save(&self) -> ProgramResult {
        let serialized = self.node_operator_avs_list.try_to_vec()?;

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }
}

pub struct SanitizedNodeOperatorVaultList<'a, 'info> {
    account: &'a AccountInfo<'info>,
    node_operator_vault_list: OperatorVaultList,
}

impl<'a, 'info> SanitizedNodeOperatorVaultList<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        node_operator: &Pubkey,
    ) -> Result<Self, ProgramError> {
        if expect_writable {
            assert_with_msg(
                account.is_writable,
                ProgramError::InvalidAccountData,
                "Node Operator Vault List account is not writable",
            )?;
        }

        let node_operator_vault_list =
            OperatorVaultList::deserialize_checked(program_id, account, node_operator)?;

        Ok(Self {
            account,
            node_operator_vault_list,
        })
    }

    pub const fn account(&self) -> &'a AccountInfo<'info> {
        self.account
    }

    pub const fn node_operator_vault_list(&self) -> &OperatorVaultList {
        &self.node_operator_vault_list
    }

    pub fn node_operator_vault_list_mut(&mut self) -> &mut OperatorVaultList {
        &mut self.node_operator_vault_list
    }

    pub fn save(&self, rent: &Rent, payer: &'a AccountInfo<'info>) -> ProgramResult {
        let serialized = self.node_operator_vault_list.try_to_vec()?;

        if serialized.len() > self.account.data.borrow().len() {
            realloc(self.account, serialized.len(), payer, rent)?;
        }

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }
}
