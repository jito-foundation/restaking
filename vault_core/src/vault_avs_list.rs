use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use jito_restaking_sanitization::{assert_with_msg, realloc};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent,
};

use crate::AccountType;

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct VaultAvsInfo {
    avs: Pubkey,

    state: SlotToggle,

    reserved: [u8; 256],
}

impl VaultAvsInfo {
    pub const fn new(avs: Pubkey, slot: u64) -> Self {
        Self {
            avs,
            state: SlotToggle::new(slot),
            reserved: [0; 256],
        }
    }

    pub const fn avs(&self) -> &Pubkey {
        &self.avs
    }

    pub const fn state(&self) -> &SlotToggle {
        &self.state
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct VaultAvsList {
    account_type: AccountType,

    vault: Pubkey,

    supported_avs: Vec<VaultAvsInfo>,

    reserved_space: [u8; 256],

    bump: u8,
}

impl VaultAvsList {
    pub const fn new(vault: Pubkey, bump: u8) -> Self {
        Self {
            account_type: AccountType::VaultAvsList,
            vault,
            supported_avs: vec![],
            reserved_space: [0; 256],
            bump,
        }
    }

    pub const fn vault(&self) -> Pubkey {
        self.vault
    }

    pub fn supported_avs(&self) -> &[VaultAvsInfo] {
        &self.supported_avs
    }

    pub fn add_avs(&mut self, avs: Pubkey, slot: u64) -> bool {
        if let Some(avs_info) = self.supported_avs.iter_mut().find(|x| *x.avs() == avs) {
            avs_info.state.activate(slot)
        } else {
            self.supported_avs.push(VaultAvsInfo::new(avs, slot));
            true
        }
    }

    pub fn remove_avs(&mut self, avs: Pubkey, slot: u64) -> bool {
        if let Some(avs_info) = self.supported_avs.iter_mut().find(|x| *x.avs() == avs) {
            avs_info.state.deactivate(slot)
        } else {
            false
        }
    }

    pub fn seeds(vault: &Pubkey) -> Vec<Vec<u8>> {
        vec![b"vault_supported_avs".to_vec(), vault.to_bytes().to_vec()]
    }

    pub fn find_program_address(program_id: &Pubkey, vault: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(vault);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        vault: &Pubkey,
    ) -> Result<Self, ProgramError> {
        assert_with_msg(
            !account.data_is_empty(),
            ProgramError::UninitializedAccount,
            "VaultAvsList account is not initialized",
        )?;
        assert_with_msg(
            account.owner == program_id,
            ProgramError::IllegalOwner,
            "VaultAvsList account not owned by the correct program",
        )?;

        let state = Self::deserialize(&mut account.data.borrow_mut().as_ref())?;
        assert_with_msg(
            state.account_type == AccountType::VaultAvsList,
            ProgramError::InvalidAccountData,
            "VaultAvsList account is invalid",
        )?;

        // The AvsState shall be at the correct PDA as defined by the seeds and bump
        let mut seeds = Self::seeds(vault);
        seeds.push(vec![state.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)?;

        assert_with_msg(
            expected_pubkey == *account.key,
            ProgramError::InvalidAccountData,
            "VaultAvsList account is not at the correct PDA",
        )?;

        Ok(state)
    }
}

pub struct SanitizedVaultAvsList<'a, 'info> {
    account: &'a AccountInfo<'info>,
    vault_avs_list: VaultAvsList,
}

impl<'a, 'info> SanitizedVaultAvsList<'a, 'info> {
    /// Sanitizes the AvsAccount so it can be used in a safe context
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        vault: &Pubkey,
    ) -> Result<SanitizedVaultAvsList<'a, 'info>, ProgramError> {
        if expect_writable {
            assert_with_msg(
                account.is_writable,
                ProgramError::InvalidAccountData,
                "VaultAvsList account is not writable",
            )?;
        }
        let vault_avs_list = VaultAvsList::deserialize_checked(program_id, account, vault)?;

        Ok(SanitizedVaultAvsList {
            account,
            vault_avs_list,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn vault_avs_list(&self) -> &VaultAvsList {
        &self.vault_avs_list
    }

    pub fn vault_avs_list_mut(&mut self) -> &mut VaultAvsList {
        &mut self.vault_avs_list
    }

    pub fn save_with_realloc(&self, rent: &Rent, payer: &'a AccountInfo<'info>) -> ProgramResult {
        let serialized = self.vault_avs_list.try_to_vec()?;

        if serialized.len() > self.account.data.borrow().len() {
            realloc(self.account, serialized.len(), payer, rent)?;
        }

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }

    pub fn save(&self) -> ProgramResult {
        let serialized = self.vault_avs_list.try_to_vec()?;

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }
}
