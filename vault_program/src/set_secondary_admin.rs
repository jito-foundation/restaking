use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_vault_core::{
    loader::{load_config, load_vault},
    vault::Vault,
};
use jito_vault_sdk::{error::VaultError, instruction::VaultAdminRole};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Processes the set delegation admin instruction: [`crate::VaultInstruction::SetSecondaryAdmin`]
pub fn process_set_secondary_admin(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    role: VaultAdminRole,
) -> ProgramResult {
    let [config, vault, admin, new_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_vault(program_id, vault, true)?;
    load_signer(admin, false)?;

    // The vault admin shall be the signer of the transaction
    let mut vault_data = vault.data.borrow_mut();
    let vault = Vault::try_from_slice_mut(&mut vault_data)?;
    if vault.admin.ne(admin.key) {
        msg!("Invalid admin for vault");
        return Err(VaultError::VaultAdminInvalid.into());
    }

    match role {
        VaultAdminRole::DelegationAdmin => {
            vault.delegation_admin = *new_admin.key;
            msg!("Delegation admin set to {:?}", new_admin.key);
        }
        VaultAdminRole::OperatorAdmin => {
            vault.operator_admin = *new_admin.key;
            msg!("Operator admin set to {:?}", new_admin.key);
        }
        VaultAdminRole::NcnAdmin => {
            vault.ncn_admin = *new_admin.key;
            msg!("Ncn admin set to {:?}", new_admin.key);
        }
        VaultAdminRole::SlasherAdmin => {
            vault.slasher_admin = *new_admin.key;
            msg!("Slasher admin set to {:?}", new_admin.key);
        }
        VaultAdminRole::CapacityAdmin => {
            vault.capacity_admin = *new_admin.key;
            msg!("Capacity admin set to {:?}", new_admin.key);
        }
        VaultAdminRole::FeeWallet => {
            vault.fee_wallet = *new_admin.key;
            msg!("Fee wallet set to {:?}", new_admin.key);
        }
        VaultAdminRole::MintBurnAdmin => {
            vault.mint_burn_admin = *new_admin.key;
            msg!("Mint burn admin set to {:?}", new_admin.key);
        }
        VaultAdminRole::WithdrawAdmin => {
            vault.withdraw_admin = *new_admin.key;
            msg!("Withdraw admin set to {:?}", new_admin.key);
        }
    }

    Ok(())
}
