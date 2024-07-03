use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use jito_restaking_sanitization::{assert_with_msg, realloc};
use solana_program::{
    account_info::AccountInfo, entrypoint_deprecated::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent,
};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    AccountType,
};

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct AvsSlasher {
    /// The vault account this slasher can slash
    vault: Pubkey,

    /// The slasher signer
    slasher: Pubkey,

    /// The max slashable funds per epoch
    max_slashable_per_epoch: u64,

    /// State of the AVS slasher
    state: SlotToggle,

    /// Reserved space
    reserved: [u8; 64],
}

impl AvsSlasher {
    pub const fn new(
        vault: Pubkey,
        slasher: Pubkey,
        max_slashable_per_epoch: u64,
        slot: u64,
    ) -> Self {
        Self {
            vault,
            slasher,
            max_slashable_per_epoch,
            state: SlotToggle::new(slot),
            reserved: [0; 64],
        }
    }

    pub const fn vault(&self) -> Pubkey {
        self.vault
    }

    pub const fn slasher(&self) -> Pubkey {
        self.slasher
    }

    pub const fn state(&self) -> &SlotToggle {
        &self.state
    }

    pub const fn max_slashable_per_epoch(&self) -> u64 {
        self.max_slashable_per_epoch
    }
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
    ) -> RestakingCoreResult<()> {
        let maybe_slasher = self
            .slashers
            .iter_mut()
            .find(|s| s.vault == vault && s.slasher == slasher);
        if maybe_slasher.is_some() {
            Err(RestakingCoreError::VaultSlasherAlreadyExists)
        } else {
            self.slashers.push(AvsSlasher {
                vault,
                slasher,
                max_slashable_per_epoch,
                state: SlotToggle::new(slot),
                reserved: [0; 64],
            });
            Ok(())
        }
    }

    pub fn deprecate_slasher(
        &mut self,
        vault: Pubkey,
        slasher: Pubkey,
        slot: u64,
    ) -> RestakingCoreResult<()> {
        let maybe_slasher = self
            .slashers
            .iter_mut()
            .find(|s| s.vault == vault && s.slasher == slasher);
        if let Some(slasher) = maybe_slasher {
            let deactivated = slasher.state.deactivate(slot);
            if deactivated {
                Ok(())
            } else {
                Err(RestakingCoreError::VaultSlasherNotActive)
            }
        } else {
            Err(RestakingCoreError::VaultSlasherNotFound)
        }
    }

    pub fn get_slasher_info(
        &self,
        vault: Pubkey,
        slasher: Pubkey,
        slot: u64,
    ) -> Option<&AvsSlasher> {
        self.slashers
            .iter()
            .find(|s| s.vault == vault && s.slasher == slasher && s.state.is_active(slot))
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

    pub fn save_with_realloc(&self, rent: &Rent, payer: &'a AccountInfo<'info>) -> ProgramResult {
        let serialized = self.avs_slasher_list.try_to_vec()?;

        if serialized.len() > self.account.data.borrow().len() {
            realloc(self.account, serialized.len(), payer, rent)?;
        }

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }

    pub fn save(&self) -> ProgramResult {
        let serialized = self.avs_slasher_list.try_to_vec()?;

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }
}
