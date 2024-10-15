use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_vault_core::{config::Config, vault::Vault};
use jito_vault_sdk::instruction::VaultAdminRole;
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

    Config::load(program_id, config, false)?;
    Vault::load(program_id, vault, true)?;
    let mut vault_data = vault.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    load_signer(admin, false)?;

    vault.check_admin(admin.key)?;

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
        VaultAdminRole::DelegateAssetAdmin => {
            vault.delegate_asset_admin = *new_admin.key;
            msg!("Delegate asset admin set to {:?}", new_admin.key);
        }
        VaultAdminRole::FeeAdmin => {
            vault.fee_admin = *new_admin.key;
            msg!("Fee admin set to {:?}", new_admin.key);
        }
    }

    Ok(())
}
