use jito_restaking_core::avs::{Avs, SanitizedAvs};
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

pub fn process_avs_withdraw_asset(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    token_mint: Pubkey,
    amount: u64,
) -> ProgramResult {
    let SanitizedAccounts {
        avs,
        avs_token_account,
        receiver_token_account,
        admin,
    } = SanitizedAccounts::sanitize(program_id, accounts, &token_mint)?;

    avs.avs().check_withdraw_admin(admin.account().key)?;

    assert_with_msg(
        avs_token_account.token_account().amount >= amount,
        ProgramError::InsufficientFunds,
        "Not enough funds in AVS token account",
    )?;

    _withdraw_avs_asset(&avs, &avs_token_account, receiver_token_account, amount)?;

    Ok(())
}

fn _withdraw_avs_asset<'a, 'info>(
    avs: &SanitizedAvs<'a, 'info>,
    avs_token_account: &SanitizedTokenAccount<'a, 'info>,
    receiver_token_account: &'a AccountInfo<'info>,
    amount: u64,
) -> ProgramResult {
    let mut avs_seeds = Avs::seeds(&avs.avs().base());
    avs_seeds.push(vec![avs.avs().bump()]);

    let avs_seeds_slice = avs_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect::<Vec<&[u8]>>();

    invoke_signed(
        &transfer(
            &spl_token::id(),
            avs_token_account.account().key,
            receiver_token_account.key,
            avs.account().key,
            &[],
            amount,
        )?,
        &[
            avs_token_account.account().clone(),
            receiver_token_account.clone(),
            avs.account().clone(),
        ],
        &[avs_seeds_slice.as_slice()],
    )?;

    Ok(())
}

struct SanitizedAccounts<'a, 'info> {
    avs: SanitizedAvs<'a, 'info>,
    avs_token_account: SanitizedTokenAccount<'a, 'info>,
    receiver_token_account: &'a AccountInfo<'info>,
    admin: SanitizedSignerAccount<'a, 'info>,
}

impl<'a, 'info> SanitizedAccounts<'a, 'info> {
    /// [`jito_restaking_sdk::RestakingInstruction::AvsWithdrawalAsset`]
    fn sanitize(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'info>],
        token_mint: &Pubkey,
    ) -> Result<SanitizedAccounts<'a, 'info>, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let avs = SanitizedAvs::sanitize(program_id, next_account_info(accounts_iter)?, false)?;
        let avs_token_account = SanitizedTokenAccount::sanitize(
            next_account_info(accounts_iter)?,
            token_mint,
            avs.account().key,
        )?;
        let receiver_token_account = next_account_info(accounts_iter)?; // let token program handle this
        let admin = SanitizedSignerAccount::sanitize(next_account_info(accounts_iter)?, false)?;
        let _token_program = SanitizedTokenProgram::sanitize(next_account_info(accounts_iter)?)?;

        Ok(SanitizedAccounts {
            avs,
            avs_token_account,
            receiver_token_account,
            admin,
        })
    }
}
