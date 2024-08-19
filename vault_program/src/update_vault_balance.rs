use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::{load_associated_token_account, load_token_mint, load_token_program};
use jito_vault_core::{config::Config, vault::Vault};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
};
use spl_token::{instruction::mint_to, state::Account};

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

    let new_balance = Account::unpack(&vault_token_account.data.borrow())?.amount;

    {
        let vault_data = vault_info.data.borrow();
        let vault = Vault::try_from_slice_unchecked(&vault_data)?;
        load_associated_token_account(vault_token_account, vault_info.key, &vault.supported_mint)?;

        load_token_mint(vrt_mint)?;
        load_associated_token_account(vault_token_account, vault_info.key, &vault.supported_mint)?;
        load_associated_token_account(vault_fee_token_account, &vault.fee_wallet, vrt_mint.key)?;
        load_token_program(token_program)?;
        vault.check_vrt_mint(vrt_mint.key)?;

        // Calculate rewards

        let reward_fee = vault.calculate_rewards_fee(new_balance)?;

        // Mint rewards
        if reward_fee > 0 {
            let (_, vault_bump, mut vault_seeds) =
                Vault::find_program_address(program_id, &vault.base);
            vault_seeds.push(vec![vault_bump]);
            let seed_slices: Vec<&[u8]> = vault_seeds.iter().map(|seed| seed.as_slice()).collect();

            msg!("Minting {} VRT rewards to the fee wallet", reward_fee);

            invoke_signed(
                &mint_to(
                    &spl_token::id(),
                    vrt_mint.key,
                    vault_fee_token_account.key,
                    vault_info.key,
                    &[],
                    reward_fee,
                )?,
                &[
                    vrt_mint.clone(),
                    vault_fee_token_account.clone(),
                    vault_info.clone(),
                ],
                &[&seed_slices],
            )?;
        }
    }

    // Update state
    {
        // need to drop the reference to vault_data before we can borrow_mut again
        let mut vault_data = vault_info.data.borrow_mut();
        let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;

        vault.tokens_deposited = new_balance;
    }

    Ok(())
}
