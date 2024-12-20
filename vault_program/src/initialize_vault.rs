use std::mem::size_of;

use jito_bytemuck::{AccountDeserialize, Discriminator};
use jito_jsm_core::{
    create_account,
    loader::{
        load_signer, load_system_account, load_system_program, load_token_account, load_token_mint,
        load_token_program,
    },
};
use jito_vault_core::{burn_vault::BurnVault, config::Config, vault::Vault, MAX_FEE_BPS};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::{
    instruction::{mint_to, transfer},
    state::Mint,
};

/// Processes the create instruction: [`crate::VaultInstruction::InitializeVault`]
pub fn process_initialize_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    deposit_fee_bps: u16,
    withdrawal_fee_bps: u16,
    reward_fee_bps: u16,
    decimals: u8,
    initialize_token_amount: u64,
) -> ProgramResult {
    let [config, vault, vrt_mint, st_mint, admin_st_token_account, vault_st_token_account, burn_vault, burn_vault_vrt_token_account, admin, base, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Config::load(program_id, config, true)?;
    let mut config_data = config.data.borrow_mut();
    let config = Config::try_from_slice_unchecked_mut(&mut config_data)?;

    load_system_account(vault, true)?;
    load_system_account(vrt_mint, true)?;
    load_signer(vrt_mint, true)?;
    load_token_mint(st_mint)?;
    load_signer(admin, true)?;
    load_signer(base, false)?;
    load_system_program(system_program)?;

    // Only the original spl token program is allowed
    load_token_program(token_program)?;
    load_token_account(
        admin_st_token_account,
        admin.key,
        st_mint.key,
        token_program,
    )?;

    load_system_account(burn_vault, true)?;
    load_system_account(burn_vault_vrt_token_account, true)?;

    if initialize_token_amount == 0 {
        msg!("Initialize token amount must be greater than zero");
        return Err(ProgramError::InvalidArgument);
    }

    // The vault account shall be at the canonical PDA
    let (vault_pubkey, vault_bump, mut vault_seeds) =
        Vault::find_program_address(program_id, base.key);
    vault_seeds.push(vec![vault_bump]);
    if vault.key.ne(&vault_pubkey) {
        msg!("Vault account is not at the correct PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    let (_, burn_vault_bump, mut burn_vault_seeds) =
        BurnVault::find_program_address(program_id, base.key);
    burn_vault_seeds.push(vec![burn_vault_bump]);

    if deposit_fee_bps > config.deposit_withdrawal_fee_cap_bps()
        || withdrawal_fee_bps > config.deposit_withdrawal_fee_cap_bps()
        || reward_fee_bps > MAX_FEE_BPS
    {
        msg!(
            "Fee cap exceeds maximum allowed of {}",
            config.deposit_withdrawal_fee_cap_bps()
        );
        return Err(VaultError::VaultFeeCapExceeded.into());
    }

    let rent = Rent::get()?;

    // Initialize VRT mint
    {
        msg!("Initializing mint @ address {}", vrt_mint.key);
        invoke(
            &system_instruction::create_account(
                admin.key,
                vrt_mint.key,
                rent.minimum_balance(Mint::get_packed_len()),
                Mint::get_packed_len() as u64,
                token_program.key,
            ),
            &[admin.clone(), vrt_mint.clone(), system_program.clone()],
        )?;

        invoke(
            &spl_token::instruction::initialize_mint2(
                &spl_token::id(),
                vrt_mint.key,
                vault.key,
                None,
                decimals,
            )?,
            &[vrt_mint.clone()],
        )?;
    }

    let slot = Clock::get()?.slot;

    // Initialize vault
    {
        msg!("Initializing vault at address {}", vault.key);
        create_account(
            admin,
            vault,
            system_program,
            program_id,
            &Rent::get()?,
            8_u64
                .checked_add(size_of::<Vault>() as u64)
                .ok_or(VaultError::ArithmeticOverflow)?,
            &vault_seeds,
        )?;

        let mut vault_data = vault.try_borrow_mut_data()?;
        vault_data[0] = Vault::DISCRIMINATOR;
        let vault_account = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;

        *vault_account = Vault::new(
            *vrt_mint.key,
            *st_mint.key,
            *admin.key,
            config.num_vaults(),
            *base.key,
            deposit_fee_bps,
            withdrawal_fee_bps,
            reward_fee_bps,
            config.program_fee_bps(),
            vault_bump,
            slot,
        )?;

        {
            // "Mint" the initial VRT supply
            vault_account.initialize_vault_override_deposit_fee_bps(0, base)?;

            let mint_summary =
                vault_account.mint_with_fee(initialize_token_amount, initialize_token_amount)?;
            if mint_summary.vrt_to_depositor != initialize_token_amount
                || mint_summary.vrt_to_fee_wallet != 0
            {
                msg!("Minted VRT to depositor does not match expected amount");
                return Err(VaultError::VaultInitialAmountFailed.into());
            }

            vault_account.initialize_vault_override_deposit_fee_bps(deposit_fee_bps, base)?;
        }

        // Deposit min ST
        {
            invoke(
                &transfer(
                    token_program.key,
                    admin_st_token_account.key,
                    vault_st_token_account.key,
                    admin.key,
                    &[],
                    initialize_token_amount,
                )?,
                &[
                    vault_st_token_account.clone(),
                    admin_st_token_account.clone(),
                    admin.clone(),
                ],
            )?;
        }

        // Create ATA
        {
            invoke(
                &create_associated_token_account(
                    admin.key,        // funding account
                    burn_vault.key,   // wallet address (ATA owner)
                    vrt_mint.key,     // mint address
                    &spl_token::id(), // token program
                ),
                &[
                    admin.clone(),
                    burn_vault_vrt_token_account.clone(), // The ATA address itself
                    burn_vault.clone(),
                    vrt_mint.clone(),
                    system_program.clone(), // Don't forget system program
                    token_program.clone(),
                    associated_token_program.clone(),
                ],
            )?;
        }

        // Mint VRT to burn vault
        {
            let signing_seeds = vault_account.signing_seeds();
            let seed_slices: Vec<&[u8]> =
                signing_seeds.iter().map(|seed| seed.as_slice()).collect();
            drop(vault_data);

            invoke_signed(
                &mint_to(
                    token_program.key,
                    vrt_mint.key,
                    burn_vault_vrt_token_account.key,
                    vault.key,
                    &[],
                    initialize_token_amount,
                )?,
                &[
                    vrt_mint.clone(),
                    burn_vault_vrt_token_account.clone(),
                    vault.clone(),
                ],
                &[&seed_slices],
            )?;
        }
    }

    config.increment_num_vaults()?;

    Ok(())
}
