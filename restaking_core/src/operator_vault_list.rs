use borsh::{BorshDeserialize, BorshSerialize};
use jito_restaking_sanitization::{assert_with_msg, realloc};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent,
};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    vault::RestakingVault,
    AccountType,
};

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
            account_type: AccountType::OperatorVaultList,
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

    pub fn check_vault_active(&self, vault: &Pubkey, slot: u64) -> RestakingCoreResult<()> {
        let maybe_vault = self.vaults.iter().find(|v| v.vault() == *vault);
        maybe_vault.map_or(Err(RestakingCoreError::VaultNotFound), |vault| {
            if vault.state().is_active(slot) {
                Ok(())
            } else {
                Err(RestakingCoreError::VaultNotActive)
            }
        })
    }

    pub fn add_vault(&mut self, vault: Pubkey, slot: u64) -> RestakingCoreResult<()> {
        let maybe_vault = self.vaults.iter_mut().find(|v| v.vault() == vault);
        if let Some(vault) = maybe_vault {
            let activated = vault.state_mut().activate(slot);
            if activated {
                Ok(())
            } else {
                Err(RestakingCoreError::VaultFailedToActivate)
            }
        } else {
            self.vaults.push(RestakingVault::new(vault, slot));
            Ok(())
        }
    }

    pub fn remove_vault(&mut self, vault: Pubkey, slot: u64) -> RestakingCoreResult<()> {
        let maybe_vault = self.vaults.iter_mut().find(|v| v.vault() == vault);
        maybe_vault.map_or(Err(RestakingCoreError::VaultNotFound), |vault| {
            let deactivated = vault.state_mut().deactivate(slot);
            if deactivated {
                Ok(())
            } else {
                Err(RestakingCoreError::VaultFailedToDeactivate)
            }
        })
    }

    pub fn seeds(operator: &Pubkey) -> Vec<Vec<u8>> {
        vec![
            b"operator_vault_list".to_vec(),
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
        operator: &Pubkey,
    ) -> Result<Self, ProgramError> {
        assert_with_msg(
            !account.data_is_empty(),
            ProgramError::UninitializedAccount,
            "Operator Vault List account is not initialized",
        )?;
        assert_with_msg(
            account.owner == program_id,
            ProgramError::IllegalOwner,
            "Operator Vault List account is not owned by the program",
        )?;

        // The AvsState shall be properly deserialized and valid struct
        let operator_vault_list = Self::deserialize(&mut account.data.borrow_mut().as_ref())?;
        assert_with_msg(
            operator_vault_list.account_type == AccountType::OperatorVaultList,
            ProgramError::InvalidAccountData,
            "Operator Vault List account is not valid",
        )?;
        assert_with_msg(
            operator_vault_list.operator == *operator,
            ProgramError::InvalidAccountData,
            "Operator Vault List account is not for the correct operator",
        )?;

        // The AvsState shall be at the correct PDA as defined by the seeds and bump
        let mut seeds = Self::seeds(operator);
        seeds.push(vec![operator_vault_list.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)?;

        assert_with_msg(
            expected_pubkey == *account.key,
            ProgramError::InvalidAccountData,
            "Operator Vault List account is not at the correct PDA",
        )?;

        Ok(operator_vault_list)
    }
}

pub struct SanitizedOperatorVaultList<'a, 'info> {
    account: &'a AccountInfo<'info>,
    operator_vault_list: OperatorVaultList,
}

impl<'a, 'info> SanitizedOperatorVaultList<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        operator: &Pubkey,
    ) -> Result<Self, ProgramError> {
        if expect_writable {
            assert_with_msg(
                account.is_writable,
                ProgramError::InvalidAccountData,
                "Operator Vault List account is not writable",
            )?;
        }

        let operator_vault_list =
            OperatorVaultList::deserialize_checked(program_id, account, operator)?;

        Ok(Self {
            account,
            operator_vault_list,
        })
    }

    pub const fn account(&self) -> &'a AccountInfo<'info> {
        self.account
    }

    pub const fn operator_vault_list(&self) -> &OperatorVaultList {
        &self.operator_vault_list
    }

    pub fn operator_vault_list_mut(&mut self) -> &mut OperatorVaultList {
        &mut self.operator_vault_list
    }

    pub fn save_with_realloc(&self, rent: &Rent, payer: &'a AccountInfo<'info>) -> ProgramResult {
        let serialized = self.operator_vault_list.try_to_vec()?;

        if serialized.len() > self.account.data.borrow().len() {
            realloc(self.account, serialized.len(), payer, rent)?;
        }

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }

    pub fn save(&self) -> ProgramResult {
        let serialized = self.operator_vault_list.try_to_vec()?;

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }
}
