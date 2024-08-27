use jito_jsm_core::loader::load_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn process_log(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _message: Vec<u8>,
) -> ProgramResult {
    let [log_signer] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys.into());
    };

    load_signer(log_signer, false)?;
    let (expected_pda, _) = Pubkey::find_program_address(&[b"log"], program_id);
    if log_signer.key != &expected_pda {
        msg!("Invalid log signer");
        return Err(ProgramError::InvalidAccountData.into());
    }

    Ok(())
}
