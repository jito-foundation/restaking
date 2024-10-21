use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::{load_associated_token_account, load_token_mint, load_token_program};
use jito_vault_core::{config::Config, vault::Vault};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program::invoke_signed, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
    sysvar::Sysvar,
};
use spl_token_2022::{
    instruction::mint_to,
    state::{Account, Mint},
};

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

    vault.check_is_paused()?;

    load_token_mint(vrt_mint)?;
    load_associated_token_account(vault_fee_token_account, &vault.fee_wallet, vrt_mint.key)?;
    load_associated_token_account(vault_token_account, vault_info.key, &vault.supported_mint)?;
    load_token_program(token_program)?;

    vault.check_update_state_ok(Clock::get()?.slot, config.epoch_length())?;
    vault.check_vrt_mint(vrt_mint.key)?;

    // Calculate rewards
    let new_balance = Account::unpack(&vault_token_account.data.borrow())?.amount;

    msg!("Before fee {} {}", new_balance, vault.tokens_deposited());

    let reward_fee_in_vrt = vault.calculate_rewards_fee_in_vrt(new_balance)?;
    msg!("Reward fee in VRT: {}", reward_fee_in_vrt);

    vault.check_reward_fee_effective_rate(new_balance, reward_fee_in_vrt, 50)?;
    msg!("Checked");

    let (_, vault_bump, mut vault_seeds) = Vault::find_program_address(program_id, &vault.base);
    vault_seeds.push(vec![vault_bump]);
    let seed_slices: Vec<&[u8]> = vault_seeds.iter().map(|seed| seed.as_slice()).collect();

    drop(vault_data);

    // Mint rewards
    if reward_fee_in_vrt > 0 {
        msg!(
            "Minting {} VRT rewards to the fee wallet",
            reward_fee_in_vrt
        );

        invoke_signed(
            &mint_to(
                &spl_token::id(),
                vrt_mint.key,
                vault_fee_token_account.key,
                vault_info.key,
                &[],
                reward_fee_in_vrt,
            )?,
            &[
                vrt_mint.clone(),
                vault_fee_token_account.clone(),
                vault_info.clone(),
            ],
            &[&seed_slices],
        )?;
    }

    {
        // Update state
        let mut vault_data = vault_info.data.borrow_mut();
        let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
        let vrt_mint_account = Mint::unpack(&vrt_mint.data.borrow())?;
        vault.set_tokens_deposited(new_balance);
        vault.set_vrt_supply(vrt_mint_account.supply);
    }

    Ok(())
}
