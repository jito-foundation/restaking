use borsh::BorshSerialize;
use jito_restaking_core::{avs::SanitizedAvs, avs_slasher_list::SanitizedAvsSlasherList};
use jito_restaking_sanitization::assert_with_msg;
use jito_restaking_sdk::{GetMaxSlashablePerEpochRequest, GetMaxSlashablePerEpochResponse};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program::set_return_data,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

pub fn process_get_max_slashable_per_epoch(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    request: GetMaxSlashablePerEpochRequest,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();

    let avs = SanitizedAvs::sanitize(program_id, next_account_info(account_iter)?, false)?;
    let slasher_list = SanitizedAvsSlasherList::sanitize(
        program_id,
        next_account_info(account_iter)?,
        false,
        avs.account().key,
    )?;

    let slot = Clock::get()?.slot;

    let slasher =
        slasher_list
            .avs_slasher_list()
            .get_slasher_info(request.vault, request.slasher, slot);
    assert_with_msg(
        slasher.is_some(),
        ProgramError::InvalidArgument,
        "Slasher not found",
    )?;

    let max_slashable_per_epoch = GetMaxSlashablePerEpochResponse {
        max_slashable_per_epoch: slasher.unwrap().max_slashable_per_epoch(),
    };

    set_return_data(max_slashable_per_epoch.try_to_vec()?.as_slice());

    Ok(())
}
