use jito_restaking_sanitization::{
    assert_with_msg, associated_token_account::SanitizedAssociatedTokenAccount,
    signer::SanitizedSignerAccount, token_mint::SanitizedTokenMint,
    token_program::SanitizedTokenProgram,
};
use jito_vault_core::vault::{SanitizedVault, Vault};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::instruction::{mint_to, transfer};

/// Processes the mint instruction: [`crate::VaultInstruction::MintTo`]
pub fn process_mint(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let SanitizedAccounts {
        mut vault,
        lrt_mint,
        depositor,
        depositor_token_account,
        vault_token_account,
        depositor_lrt_token_account,
        vault_fee_token_account,
        token_program,
        mint_signer,
    } = SanitizedAccounts::sanitize(program_id, accounts)?;

    // The LRT mint provided shall be equal to the one the vault supports
    assert_with_msg(
        *lrt_mint.account().key == vault.vault().lrt_mint(),
        ProgramError::InvalidAccountData,
        "Mint account does not match LRT mint",
    )?;
    // If a mint_signer is set, the signer shall be authorized by the vault to make deposits
    if let Some(mint_signer) = mint_signer {
        assert_with_msg(
            *mint_signer.account().key == vault.vault().mint_burn_authority().unwrap(),
            ProgramError::InvalidAccountData,
            "Mint signer does not match vault mint signer",
        )?;
    }

    // refresh the amount in the vault in-case out-of-band token account increases
    vault
        .vault_mut()
        .set_tokens_deposited(vault_token_account.token_account().amount);

    _transfer_to_vault(
        &token_program,
        &depositor_token_account,
        &vault_token_account,
        &depositor,
        amount,
    )?;

    let lrt_to_mint = vault
        .vault_mut()
        .deposit_and_mint_with_capacity_check(amount)?;
    let lrt_to_fee_account = vault.vault().calculate_deposit_fee(lrt_to_mint)?;
    let lrt_to_user = lrt_to_mint.checked_sub(lrt_to_fee_account).unwrap();

    // mint LRT to user and fee wallet
    _mint_lrt(
        program_id,
        &token_program,
        &vault,
        &lrt_mint,
        &depositor_lrt_token_account,
        lrt_to_user,
    )?;
    _mint_lrt(
        program_id,
        &token_program,
        &vault,
        &lrt_mint,
        &vault_fee_token_account,
        lrt_to_fee_account,
    )?;

    vault.save()?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    vault: SanitizedVault<'a, 'info>,
    lrt_mint: SanitizedTokenMint<'a, 'info>,
    depositor: SanitizedSignerAccount<'a, 'info>,
    depositor_token_account: SanitizedAssociatedTokenAccount<'a, 'info>,
    vault_token_account: SanitizedAssociatedTokenAccount<'a, 'info>,
    depositor_lrt_token_account: SanitizedAssociatedTokenAccount<'a, 'info>,
    vault_fee_token_account: SanitizedAssociatedTokenAccount<'a, 'info>,
    token_program: SanitizedTokenProgram<'a, 'info>,
    mint_signer: Option<SanitizedSignerAccount<'a, 'info>>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// Loads accounts for [`crate::VaultInstruction::MintTo`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        msg!("0");
        let vault = SanitizedVault::sanitize(program_id, next_account_info(accounts_iter)?, true)?;
        msg!("1");
        let lrt_mint = SanitizedTokenMint::sanitize(next_account_info(accounts_iter)?, true)?;
        msg!("2");
        let depositor = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, true)?;
        msg!("3");
        let depositor_token_account = SanitizedAssociatedTokenAccount::sanitize(
            next_account_info(accounts_iter)?,
            &vault.vault().supported_mint(),
            depositor.account().key,
        )?;
        msg!("4");
        let vault_token_account = SanitizedAssociatedTokenAccount::sanitize(
            next_account_info(accounts_iter)?,
            &vault.vault().supported_mint(),
            vault.account().key,
        )?;
        msg!("5");
        let depositor_lrt_token_account = SanitizedAssociatedTokenAccount::sanitize(
            next_account_info(accounts_iter)?,
            &vault.vault().lrt_mint(),
            depositor.account().key,
        )?;
        msg!("6");
        let vault_fee_token_account = SanitizedAssociatedTokenAccount::sanitize(
            next_account_info(accounts_iter)?,
            &vault.vault().lrt_mint(),
            &vault.vault().fee_owner(),
        )?;
        msg!("7");
        let token_program = SanitizedTokenProgram::sanitize(next_account_info(accounts_iter)?)?;
        msg!("8");
        let mint_signer = if vault.vault().mint_burn_authority().is_some() {
            Some(SanitizedSignerAccount::sanitize(
                next_account_info(accounts_iter)?,
                false,
            )?)
        } else {
            None
        };
        msg!("9");

        Ok(SanitizedAccounts {
            vault,
            lrt_mint,
            depositor,
            depositor_token_account,
            vault_token_account,
            depositor_lrt_token_account,
            vault_fee_token_account,
            token_program,
            mint_signer,
        })
    }
}

/// Transfers tokens from the `depositor_token_account` owned by the `owner` to the `vault_token_account`
/// using a CPI.
///
/// # Arguments
/// * `depositor_token_account` - The source token account to transfer from
/// * `vault_token_account` - The destination token account to transfer to
/// * `owner` - The owner of the source token account
/// * `amount` - The amount of tokens to transfer
fn _transfer_to_vault<'a, 'info>(
    token_program: &SanitizedTokenProgram,
    depositor_token_account: &SanitizedAssociatedTokenAccount<'a, 'info>,
    vault_token_account: &SanitizedAssociatedTokenAccount<'a, 'info>,
    owner: &SanitizedSignerAccount<'a, 'info>,
    amount: u64,
) -> ProgramResult {
    invoke(
        &transfer(
            token_program.account().key,
            depositor_token_account.account().key,
            vault_token_account.account().key,
            owner.account().key,
            &[],
            amount,
        )?,
        &[
            depositor_token_account.account().clone(),
            vault_token_account.account().clone(),
            owner.account().clone(),
        ],
    )
}

fn _mint_lrt<'a, 'info>(
    program_id: &Pubkey,
    token_program: &SanitizedTokenProgram,
    vault: &SanitizedVault<'a, 'info>,
    lrt_mint: &SanitizedTokenMint<'a, 'info>,
    depositor_lrt_token_account: &SanitizedAssociatedTokenAccount<'a, 'info>,
    mint_amount: u64,
) -> ProgramResult {
    let (_, bump, mut seeds) = Vault::find_program_address(program_id, &vault.vault().base());
    seeds.push(vec![bump]);
    let seed_slices: Vec<&[u8]> = seeds.iter().map(|seed| seed.as_slice()).collect();

    invoke_signed(
        &mint_to(
            token_program.account().key,
            lrt_mint.account().key,
            depositor_lrt_token_account.account().key,
            vault.account().key,
            &[],
            mint_amount,
        )?,
        &[
            lrt_mint.account().clone(),
            depositor_lrt_token_account.account().clone(),
            vault.account().clone(),
        ],
        &[&seed_slices],
    )?;

    Ok(())
}
