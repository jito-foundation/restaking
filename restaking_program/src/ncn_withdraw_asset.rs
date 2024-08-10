use jito_restaking_core::ncn::{Ncn, SanitizedNcn};
use jito_restaking_sanitization::{
    assert_with_msg, signer::SanitizedSignerAccount, token_account::SanitizedTokenAccount,
    token_program::SanitizedTokenProgram,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::instruction::transfer;

pub fn process_ncn_withdraw_asset(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    token_mint: Pubkey,
    amount: u64,
) -> ProgramResult {
    let SanitizedAccounts {
        ncn,
        ncn_token_account,
        receiver_token_account,
        admin,
    } = SanitizedAccounts::sanitize(program_id, accounts, &token_mint)?;

    ncn.ncn().check_withdraw_admin(admin.account().key)?;

    assert_with_msg(
        ncn_token_account.token_account().amount >= amount,
        ProgramError::InsufficientFunds,
        "Not enough funds in NCN token account",
    )?;

    _withdraw_ncn_asset(&ncn, &ncn_token_account, receiver_token_account, amount)?;

    Ok(())
}

fn _withdraw_ncn_asset<'a, 'info>(
    ncn: &SanitizedNcn<'a, 'info>,
    ncn_token_account: &SanitizedTokenAccount<'a, 'info>,
    receiver_token_account: &'a AccountInfo<'info>,
    amount: u64,
) -> ProgramResult {
    let mut ncn_seeds = Ncn::seeds(&ncn.ncn().base());
    ncn_seeds.push(vec![ncn.ncn().bump()]);

    let ncn_seeds_slice = ncn_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect::<Vec<&[u8]>>();

    invoke_signed(
        &transfer(
            &spl_token::id(),
            ncn_token_account.account().key,
            receiver_token_account.key,
            ncn.account().key,
            &[],
            amount,
        )?,
        &[
            ncn_token_account.account().clone(),
            receiver_token_account.clone(),
            ncn.account().clone(),
        ],
        &[ncn_seeds_slice.as_slice()],
    )?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    ncn: SanitizedNcn<'a, 'info>,
    ncn_token_account: SanitizedTokenAccount<'a, 'info>,
    receiver_token_account: &'a AccountInfo<'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// [`jito_restaking_sdk::RestakingInstruction::NcnWithdrawalAsset`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
        token_mint: &Pubkey,
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let ncn = SanitizedNcn::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let ncn_token_account = SanitizedTokenAccount::sanitize(
            next_account_info(accounts_iter)?,
            token_mint,
            ncn.account().key,
        )?;
        let receiver_token_account = next_account_info(accounts_iter)?; // let token program handle this
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        let _token_program = SanitizedTokenProgram::sanitize(next_account_info(accounts_iter)?)?;

        Ok(SanitizedAccounts {
            ncn,
            ncn_token_account,
            receiver_token_account,
            admin,
        })
    }
}
