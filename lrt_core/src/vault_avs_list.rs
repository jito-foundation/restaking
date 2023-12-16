use borsh::{BorshDeserialize, BorshSerialize};
use jito_restaking_sanitization::{assert_with_msg, realloc};
use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, rent::Rent,
};

use crate::AccountType;

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct AvsInfo {
    avs: Pubkey,

    slot_added: u64,

    slot_removed: u64,
}

impl AvsInfo {
    pub const fn new(avs: Pubkey, slot_added: u64) -> Self {
        Self {
            avs,
            slot_added,
            slot_removed: 0,
        }
    }

    pub const fn avs(&self) -> Pubkey {
        self.avs
    }

    pub const fn slot_added(&self) -> u64 {
        self.slot_added
    }

    pub const fn slot_removed(&self) -> u64 {
        self.slot_removed
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct VaultAvsList {
    account_type: AccountType,

    vault: Pubkey,

    supported_avs: Vec<AvsInfo>,

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

    pub fn supported_avs(&self) -> &[AvsInfo] {
        &self.supported_avs
    }

    pub fn add_avs(&mut self, avs: Pubkey, slot: u64) -> bool {
        if let Some(avs_info) = self.supported_avs.iter_mut().find(|x| x.avs == avs) {
            avs_info.slot_added = slot;
        } else {
            self.supported_avs.push(AvsInfo::new(avs, slot));
        }
        true
    }

    pub fn remove_avs(&mut self, avs: Pubkey, slot: u64) -> bool {
        if let Some(avs_info) = self.supported_avs.iter_mut().find(|x| x.avs == avs) {
            avs_info.slot_removed = slot;
            true
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
                "Invalid writable flag for LRT",
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

    pub fn save(
        &self,
        rent: &Rent,
        payer: &'a AccountInfo<'info>,
    ) -> solana_program::entrypoint_deprecated::ProgramResult {
        let serialized = self.vault_avs_list.try_to_vec()?;

        if serialized.len() > self.account.data.borrow().len() {
            realloc(self.account, serialized.len(), payer, rent)?;
        }

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }
}
