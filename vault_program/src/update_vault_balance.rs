use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::{load_associated_token_account, load_token_mint, load_token_program};
use jito_vault_core::{config::Config, vault::Vault};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program::invoke_signed, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token_interface::{instruction::mint_to, state::Account};

pub fn process_update_vault_balance(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let [config, vault_info, vault_token_account, vrt_mint, vault_fee_token_account, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Config::load(program_id, config, false)?;
    Vault::load(program_id, vault_info, true)?;

    let config_data = config.data.borrow();
    let config = Config::try_from_slice_unchecked(&config_data)?;

    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;

    load_token_mint(vrt_mint)?;
    load_associated_token_account(vault_fee_token_account, &vault.fee_wallet, vrt_mint.key)?;
    load_associated_token_account(vault_token_account, vault_info.key, &vault.supported_mint)?;
    load_token_program(token_program)?;

    vault.check_update_state_ok(Clock::get()?.slot, config.epoch_length())?;
    vault.check_vrt_mint(vrt_mint.key)?;
    vault.check_is_paused()?;

    // Calculate rewards
    // - We take our fee in st
    // - We add the reward ( total reward - fee in st )
    // - We virtually call mint_to on the reward fee ob behalf of the vault
    let new_st_balance = Account::unpack(&vault_token_account.data.borrow())?.amount;

    // 1. Calculate reward fee in ST
    let st_rewards = new_st_balance.saturating_sub(vault.tokens_deposited());
    let st_reward_fee = vault.calculate_st_reward_fee(new_st_balance)?;

    // 2. Increment ST less the reward fee
    let st_balance_after_fees = new_st_balance
        .checked_sub(st_reward_fee)
        .ok_or(VaultError::ArithmeticUnderflow)?;
    vault.set_tokens_deposited(st_balance_after_fees);

    // 3. Calculate the reward fee in VRT
    let vrt_reward_fee = vault.calculate_vrt_mint_amount(st_reward_fee)?;

    // 4. Update State, with the vrt fee and the new st balance
    vault.set_tokens_deposited(new_st_balance);
    vault.increment_vrt_supply(vrt_reward_fee)?;

    // 5. Check for rewards not substantial enough
    vault.check_reward_fee_effective_rate(
        st_rewards,
        vrt_reward_fee,
        Vault::MAX_REWARD_DELTA_BPS,
    )?;

    // Mint rewards
    if vrt_reward_fee > 0 {
        let vault_seeds = vault.signing_seeds();
        let seed_slices: Vec<&[u8]> = vault_seeds.iter().map(|seed| seed.as_slice()).collect();

        drop(vault_data);

        msg!("Minting {} VRT rewards to the fee wallet", vrt_reward_fee);

        invoke_signed(
            &mint_to(
                &spl_token_interface::id(),
                vrt_mint.key,
                vault_fee_token_account.key,
                vault_info.key,
                &[],
                vrt_reward_fee,
            )?,
            &[
                vrt_mint.clone(),
                vault_fee_token_account.clone(),
                vault_info.clone(),
            ],
            &[&seed_slices],
        )?;
    }

    Ok(())
}
