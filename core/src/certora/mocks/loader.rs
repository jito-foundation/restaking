use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, 
};

pub fn check_mint_account(_info : &AccountInfo) -> Result<(), ProgramError> {
    if cvlr::cvlr_nondet::<bool>() {
        Ok(())
    } else {
        Err(ProgramError::InvalidAccountData)
    }  
}