use borsh::{BorshDeserialize, BorshSerialize};
use jito_restaking_sanitization::{assert_with_msg, realloc};
use solana_program::{
    account_info::AccountInfo, entrypoint_deprecated::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent,
};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    vault::RestakingVault,
    AccountType,
};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct AvsVaultList {
    /// The account type
    account_type: AccountType,

    /// The AVS
    avs: Pubkey,

    /// The list of supported vaults by the AVS
    /// Doesn't necessarily mean they're delegated to the AVS
    vault_list: Vec<RestakingVault>,

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

    pub fn vault_list(&self) -> &[RestakingVault] {
        &self.vault_list
    }

    pub fn contains_vault(&self, vault: Pubkey) -> bool {
        self.vault_list.iter().any(|v| v.vault() == vault)
    }

    pub fn check_active_vault(&self, vault: &Pubkey, slot: u64) -> RestakingCoreResult<()> {
        let maybe_vault = self.vault_list.iter().find(|v| v.vault() == *vault);
        maybe_vault.map_or(Err(RestakingCoreError::VaultNotFound), |vault| {
            if vault.state().is_active(slot) {
                Ok(())
            } else {
                Err(RestakingCoreError::VaultNotActive)
            }
        })
    }

    pub fn add_vault(&mut self, vault: Pubkey, slot: u64) -> RestakingCoreResult<()> {
        let maybe_vault = self.vault_list.iter_mut().find(|v| v.vault() == vault);
        if let Some(vault) = maybe_vault {
            let activated = vault.state_mut().activate(slot);
            if activated {
                Ok(())
            } else {
                Err(RestakingCoreError::VaultFailedToActivate)
            }
        } else {
            self.vault_list.push(RestakingVault::new(vault, slot));
            Ok(())
        }
    }

    pub fn remove_vault(&mut self, vault: Pubkey, slot: u64) -> RestakingCoreResult<()> {
        let maybe_vault = self.vault_list.iter_mut().find(|v| v.vault() == vault);
        maybe_vault.map_or(Err(RestakingCoreError::VaultNotFound), |vault| {
            let deactivated = vault.state_mut().deactivate(slot);
            if deactivated {
                Ok(())
            } else {
                Err(RestakingCoreError::VaultFailedToDeactivate)
            }
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

    pub fn save_with_realloc(&self, rent: &Rent, payer: &'a AccountInfo<'info>) -> ProgramResult {
        let serialized = self.avs_vault_list.try_to_vec()?;

        if serialized.len() > self.account.data.borrow().len() {
            realloc(self.account, serialized.len(), payer, rent)?;
        }

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }

    pub fn save(&self) -> ProgramResult {
        let serialized = self.avs_vault_list.try_to_vec()?;

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }
}
