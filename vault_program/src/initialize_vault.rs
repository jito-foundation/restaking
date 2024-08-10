use std::mem::size_of;

use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{
        load_signer, load_system_account, load_system_program, load_token_mint, load_token_program,
    },
};
use jito_vault_core::{
    config::Config,
    loader::load_config,
    operator_delegation::OperatorDelegation,
    vault::Vault,
    vault_delegation_list::{VaultDelegationList, MAX_DELEGATIONS},
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke,
    program_error::ProgramError, program_pack::Pack, pubkey::Pubkey, rent::Rent,
    system_instruction, sysvar::Sysvar,
};
use spl_token::state::Mint;

/// Processes the create instruction: [`crate::VaultInstruction::InitializeVault`]
pub fn process_initialize_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    deposit_fee_bps: u16,
    withdrawal_fee_bps: u16,
) -> ProgramResult {
    let [config, vault, vault_delegation_list, lrt_mint, mint, admin, base, system_program, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_config(program_id, config, true)?;
    load_system_account(vault, true)?;
    load_system_account(vault_delegation_list, true)?;
    load_system_account(lrt_mint, true)?;
    load_signer(lrt_mint, true)?;
    load_token_mint(mint)?;
    load_signer(admin, true)?;
    load_signer(base, false)?;
    load_system_program(system_program)?;
    load_token_program(token_program)?;

    let (vault_pubkey, vault_bump, mut vault_seeds) =
        Vault::find_program_address(program_id, base.key);
    vault_seeds.push(vec![vault_bump]);
    if vault.key.ne(&vault_pubkey) {
        msg!("Vault account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    let (vault_delegation_list_pubkey, vault_delegation_list_bump, mut vault_delegation_list_seeds) =
        VaultDelegationList::find_program_address(program_id, vault.key);
    vault_delegation_list_seeds.push(vec![vault_delegation_list_bump]);
    if vault_delegation_list.key.ne(&vault_delegation_list_pubkey) {
        msg!("Vault delegation list account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut config_data = config.data.borrow_mut();
    let config = Config::try_from_slice_mut(&mut config_data)?;

    let rent = Rent::get()?;

    // Initialize LRT mint
    {
        msg!("Initializing mint @ address {}", lrt_mint.key);
        invoke(
            &system_instruction::create_account(
                admin.key,
                lrt_mint.key,
                rent.minimum_balance(Mint::get_packed_len()),
                Mint::get_packed_len() as u64,
                token_program.key,
            ),
            &[admin.clone(), lrt_mint.clone(), system_program.clone()],
        )?;

        invoke(
            &spl_token::instruction::initialize_mint2(
                &spl_token::id(),
                lrt_mint.key,
                vault.key,
                None,
                9,
            )?,
            &[lrt_mint.clone()],
        )?;
    }

    // Initialize vault
    {
        msg!("Initializing vault at address {}", vault.key);
        create_account(
            admin,
            vault,
            system_program,
            program_id,
            &Rent::get()?,
            (8 + size_of::<Vault>()) as u64,
            &vault_seeds,
        )?;

        let mut vault_data = vault.try_borrow_mut_data()?;
        vault_data[0] = Vault::DISCRIMINATOR;
        let vault = Vault::try_from_slice_mut(&mut vault_data)?;
        vault.base = *base.key;
        vault.lrt_mint = *lrt_mint.key;
        vault.supported_mint = *mint.key;
        vault.admin = *admin.key;
        vault.delegation_admin = *admin.key;
        vault.operator_admin = *admin.key;
        vault.ncn_admin = *admin.key;
        vault.slasher_admin = *admin.key;
        vault.fee_wallet = *admin.key;
        vault.mint_burn_authority = Pubkey::default();
        vault.capacity = u64::MAX;
        vault.vault_index = config.num_vaults;
        vault.lrt_supply = 0;
        vault.tokens_deposited = 0;
        vault.withdrawable_reserve_amount = 0;
        vault.ncn_count = 0;
        vault.operator_count = 0;
        vault.slasher_count = 0;
        vault.deposit_fee_bps = deposit_fee_bps;
        vault.withdrawal_fee_bps = withdrawal_fee_bps;
        vault.bump = vault_bump;
    }

    // Initialize vault delegation list
    {
        msg!(
            "Initializing vault delegation list at address {}",
            vault_delegation_list.key
        );
        create_account(
            admin,
            vault_delegation_list,
            system_program,
            program_id,
            &Rent::get()?,
            (8 + size_of::<VaultDelegationList>()) as u64,
            &vault_delegation_list_seeds,
        )?;

        let mut vault_delegation_list_data = vault_delegation_list.try_borrow_mut_data()?;
        vault_delegation_list_data[0] = VaultDelegationList::DISCRIMINATOR;
        let vault_delegation_list =
            VaultDelegationList::try_from_slice_mut(&mut vault_delegation_list_data)?;
        vault_delegation_list.vault = *vault.key;
        vault_delegation_list.delegations = [OperatorDelegation::default(); MAX_DELEGATIONS];
        vault_delegation_list.last_slot_updated = 0; // force an update as soon as possible
        vault_delegation_list.bump = vault_delegation_list_bump;
    }

    config.num_vaults = config
        .num_vaults
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;

    Ok(())
}
