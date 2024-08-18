use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::{
    load_signer, load_token_2022_program, load_token_account, load_token_mint, load_token_program,
};
use jito_vault_core::vault::Vault;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey,
};

/// Processes the delegate token account instruction: [`crate::VaultInstruction::DelegateTokenAccount`]
///
/// Admin might call the instruction when the vault is airdropped or transferred tokens
pub fn process_delegate_token_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let [vault_info, admin, token_mint, token_account, delegate, token_program_info] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Vault::load(program_id, vault_info, false)?;
    load_signer(admin, false)?;
    load_token_mint(token_mint)?;

    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;

    vault.check_admin(admin)?;
    if vault.supported_mint.eq(token_mint.key) {
        msg!("Invalid Token mint");
        return Err(ProgramError::InvalidAccountData);
    }

    load_token_account(token_account, token_program_info)?;

    match (*token_mint.owner, *token_account.owner) {
        (spl_token::ID, spl_token::ID) => {
            load_token_program(token_program_info)?;
        }
        (spl_token_2022::ID, spl_token_2022::ID) => {
            load_token_2022_program(token_program_info)?;
        }
        _ => {
            msg!("token_mint and token_account owner does not match");
            return Err(ProgramError::InvalidAccountData);
        }
    }

    let (vault_pubkey, vault_bump, mut vault_seeds) =
        Vault::find_program_address(program_id, &vault.base);
    vault_seeds.push(vec![vault_bump]);

    drop(vault_data);

    let ix = if token_program_info.key.eq(&spl_token::id()) {
        spl_token::instruction::approve(
            token_program_info.key,
            token_account.key,
            delegate.key,
            &vault_pubkey,
            &[],
            amount,
        )?
    } else {
        spl_token_2022::instruction::approve(
            token_program_info.key,
            token_account.key,
            delegate.key,
            &vault_pubkey,
            &[],
            amount,
        )?
    };

    invoke_signed(
        &ix,
        &[
            token_program_info.clone(),
            token_account.clone(),
            delegate.clone(),
            vault_info.clone(),
        ],
        &[vault_seeds
            .iter()
            .map(|seed| seed.as_slice())
            .collect::<Vec<&[u8]>>()
            .as_slice()],
    )?;

    Ok(())
}
