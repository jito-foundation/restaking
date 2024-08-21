use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::{
    load_associated_token_account, load_signer, load_system_program, load_token_mint,
    load_token_program,
};
use jito_vault_core::{
    config::Config,
    vault::{BurnSummary, Vault},
};
use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token::instruction::{burn, transfer};

pub fn process_burn(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount_in: u64,
    min_amount_out: u64,
) -> ProgramResult {
    let (required_accounts, optional_accounts) = accounts.split_at(10);

    let [config, vault_info, vault_token_account, vrt_mint, staker, staker_token_account, staker_vrt_token_account, vault_fee_token_account, token_program, system_program] =
        required_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let clock = Clock::get()?;

    Config::load(program_id, config, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;
    Vault::load(program_id, vault_info, true)?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    load_associated_token_account(vault_token_account, vault_info.key, &vault.supported_mint)?;
    load_token_mint(vrt_mint)?;
    load_signer(staker, false)?;
    load_associated_token_account(staker_token_account, staker.key, &vault.supported_mint)?;
    load_associated_token_account(staker_vrt_token_account, staker.key, &vault.vrt_mint)?;
    load_associated_token_account(vault_fee_token_account, &vault.fee_wallet, &vault.vrt_mint)?;
    load_token_program(token_program)?;
    load_system_program(system_program)?;

    // The vault VRT mint shall be correct
    // The vault shall not need an update
    vault.check_vrt_mint(vrt_mint.key)?;
    vault.check_update_state_ok(clock.slot, config.epoch_length())?;
    vault.check_mint_burn_admin(optional_accounts.first())?;

    let BurnSummary {
        fee_amount,
        burn_amount,
        out_amount,
    } = vault.burn_with_fee(amount_in, min_amount_out)?;

    // Burn the VRT tokens from the staker's account
    invoke(
        &burn(
            &spl_token::id(),
            staker_vrt_token_account.key,
            vrt_mint.key,
            staker.key,
            &[],
            burn_amount,
        )?,
        &[
            staker_vrt_token_account.clone(),
            vrt_mint.clone(),
            staker.clone(),
        ],
    )?;
    // Transfer the assets from the staker to the vault fee account
    invoke(
        &transfer(
            &spl_token::id(),
            staker_vrt_token_account.key,
            vault_fee_token_account.key,
            staker.key,
            &[],
            fee_amount,
        )?,
        &[
            staker_vrt_token_account.clone(),
            vault_fee_token_account.clone(),
            staker.clone(),
        ],
    )?;

    // Transfer the assets from the vault to the staker's account
    let (_, vault_bump, mut vault_seeds) = Vault::find_program_address(program_id, &vault.base);
    vault_seeds.push(vec![vault_bump]);
    let seed_slices: Vec<&[u8]> = vault_seeds.iter().map(|seed| seed.as_slice()).collect();
    drop(vault_data);
    invoke_signed(
        &transfer(
            &spl_token::id(),
            vault_token_account.key,
            staker_token_account.key,
            vault_info.key,
            &[],
            out_amount,
        )?,
        &[
            vault_token_account.clone(),
            staker_token_account.clone(),
            vault_info.clone(),
        ],
        &[seed_slices.as_slice()],
    )?;

    Ok(())
}
