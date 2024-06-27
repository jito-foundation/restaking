use jito_restaking_sanitization::{
    assert_with_msg, associated_token_account::SanitizedAssociatedTokenAccount,
    signer::SanitizedSignerAccount, token_mint::SanitizedTokenMint,
    token_program::SanitizedTokenProgram,
};
use jito_vault_core::vault::{SanitizedVault, Vault};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Processes the mint instruction: [`crate::VaultInstruction::MintTo`]
pub fn process_mint(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let mut accounts_iter = &mut accounts.iter();

    let mut vault =
        SanitizedVault::sanitize(program_id, next_account_info(&mut accounts_iter)?, true)?;
    let mut lrt_mint = SanitizedTokenMint::sanitize(next_account_info(&mut accounts_iter)?)?;
    assert_with_msg(
        lrt_mint.account().is_writable,
        ProgramError::InvalidAccountData,
        "Mint account is not writable",
    )?;
    assert_with_msg(
        *lrt_mint.account().key == vault.vault().lrt_mint(),
        ProgramError::InvalidAccountData,
        "Mint account does not match LRT mint",
    )?;
    let source_owner =
        SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, true)?;
    let source_token_account = SanitizedAssociatedTokenAccount::sanitize(
        next_account_info(&mut accounts_iter)?,
        &vault.vault().supported_mint(),
        source_owner.account().key,
    )?;
    let mut dest_token_account = SanitizedAssociatedTokenAccount::sanitize(
        next_account_info(&mut accounts_iter)?,
        &vault.vault().supported_mint(),
        vault.account().key,
    )?;
    let lrt_receiver = SanitizedAssociatedTokenAccount::sanitize(
        next_account_info(&mut accounts_iter)?,
        &vault.vault().lrt_mint(),
        source_owner.account().key,
    )?;
    let _token_program = SanitizedTokenProgram::sanitize(next_account_info(&mut accounts_iter)?)?;
    if let Some(vault_mint_signer) = vault.vault().mint_burn_authority() {
        let mint_signer =
            SanitizedSignerAccount::sanitize(next_account_info(&mut accounts_iter)?, false)?;
        assert_with_msg(
            *mint_signer.account().key == vault_mint_signer,
            ProgramError::InvalidAccountData,
            "Mint signer does not match vault mint signer",
        )?;
    }

    // check capacity
    let amount_after_deposit = amount.checked_add(dest_token_account.token_account().amount);
    assert_with_msg(
        amount_after_deposit.is_some(),
        ProgramError::InvalidArgument,
        "Overflow when adding amount to destination token account",
    )?;
    let amount_after_deposit = amount_after_deposit.unwrap();
    assert_with_msg(
        vault.vault().capacity() <= amount_after_deposit,
        ProgramError::InvalidArgument,
        "Amount exceeds vault capacity",
    )?;

    // transfer the amount from the source token account to the destination token account
    invoke(
        &spl_token::instruction::transfer(
            &spl_token::id(),
            source_token_account.account().key,
            dest_token_account.account().key,
            source_owner.account().key,
            &[],
            amount,
        )?,
        &[
            source_token_account.account().clone(),
            dest_token_account.account().clone(),
            source_owner.account().clone(),
        ],
    )?;

    let (_, bump, mut seeds) = Vault::find_program_address(program_id, &vault.vault().base());
    seeds.push(vec![bump]);
    let seed_slices: Vec<&[u8]> = seeds.iter().map(|seed| seed.as_slice()).collect();

    // mint the amount to the LRT receiver in a 1:1 ratio
    invoke_signed(
        &spl_token::instruction::mint_to(
            &spl_token::id(),
            lrt_mint.account().key,
            lrt_receiver.account().key,
            vault.account().key,
            &[],
            amount,
        )?,
        &[
            lrt_mint.account().clone(),
            lrt_receiver.account().clone(),
            vault.account().clone(),
        ],
        &[&seed_slices],
    )?;

    // need to reload after CPI
    lrt_mint.reload()?;
    dest_token_account.reload()?;

    // TODO (LB): should do this incrementally or refresh based?
    vault
        .vault_mut()
        .set_tokens_deposited(dest_token_account.token_account().amount);
    vault.vault_mut().set_lrt_supply(lrt_mint.mint().supply);

    vault.save()?;

    Ok(())
}
