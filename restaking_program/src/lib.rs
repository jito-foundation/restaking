mod avs_add_operator;
mod avs_add_vault;
mod avs_add_vault_slasher;
mod avs_deprecate_vault_slasher;
mod avs_remove_operator;
mod avs_remove_vault;
mod get_max_slashable_per_epoch;
mod initialize_avs;
mod initialize_config;
mod initialize_operator;
mod operator_add_avs;
mod operator_add_vault;
mod operator_remove_avs;
mod operator_remove_vault;
mod operator_set_admin;
mod operator_set_voter;

use borsh::BorshDeserialize;
use jito_restaking_sdk::RestakingInstruction;
use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey,
};
#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

use crate::{
    avs_add_operator::process_avs_add_node_operator, avs_add_vault::process_avs_add_vault,
    avs_add_vault_slasher::process_avs_add_vault_slasher,
    avs_deprecate_vault_slasher::process_avs_deprecate_slasher,
    avs_remove_operator::process_avs_remove_node_operator,
    avs_remove_vault::process_avs_remove_vault,
    get_max_slashable_per_epoch::process_get_max_slashable_per_epoch,
    initialize_avs::process_initialize_avs, initialize_config::process_initialize_config,
    initialize_operator::process_initialize_node_operator,
    operator_add_avs::process_operator_add_avs, operator_add_vault::process_operator_add_vault,
    operator_remove_avs::process_operator_remove_avs,
    operator_remove_vault::process_node_operator_remove_vault,
    operator_set_admin::process_set_node_operator_admin,
    operator_set_voter::process_set_node_operator_voter,
};

declare_id!("E5YF9Um1mwQWHffqaUEUwtwnhQKsbMEt33qtvjto3NDZ");

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    // Required fields
    name: "Jito's Restaking Program",
    project_url: "https://jito.wtf/",
    contacts: "email:team@jito.wtf",
    policy: "https://github.com/jito-labs/jsm",
    // Optional Fields
    preferred_languages: "en",
    source_code: "https://github.com/jito-labs/jsm"
}

#[cfg(not(feature = "no-entrypoint"))]
solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if *program_id != id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    let instruction = RestakingInstruction::try_from_slice(instruction_data)?;

    match instruction {
        RestakingInstruction::InitializeConfig => {
            msg!("Instruction: InitializeConfig");
            process_initialize_config(program_id, accounts)
        }
        RestakingInstruction::InitializeAvs => {
            msg!("Instruction: InitializeAvs");
            process_initialize_avs(program_id, accounts)
        }
        RestakingInstruction::AvsAddVault => {
            msg!("Instruction: AvsAddVault");
            process_avs_add_vault(program_id, accounts)
        }
        RestakingInstruction::AvsRemoveVault => {
            msg!("Instruction: AvsRemoveVault");
            process_avs_remove_vault(program_id, accounts)
        }
        RestakingInstruction::AvsAddOperator => {
            msg!("Instruction: AvsAddNodeOperator");
            process_avs_add_node_operator(program_id, accounts)
        }
        RestakingInstruction::AvsRemoveOperator => {
            msg!("Instruction: AvsRemoveNodeOperator");
            process_avs_remove_node_operator(program_id, accounts)
        }
        RestakingInstruction::AvsAddVaultSlasher(max_slashable_per_epoch) => {
            msg!("Instruction: AvsAddVaultSlasher");
            process_avs_add_vault_slasher(program_id, accounts, max_slashable_per_epoch)
        }
        RestakingInstruction::AvsDeprecateVaultSlasher => {
            msg!("Instruction: AvsDeprecateVaultSlasher");
            process_avs_deprecate_slasher(program_id, accounts)
        }
        RestakingInstruction::InitializeOperator => {
            msg!("Instruction: InitializeNodeOperator");
            process_initialize_node_operator(program_id, accounts)
        }
        RestakingInstruction::OperatorSetAdmin => {
            msg!("Instruction: OperatorSetAdmin");
            process_set_node_operator_admin(program_id, accounts)
        }
        RestakingInstruction::OperatorSetVoter => {
            msg!("Instruction: OperatorSetVoter");
            process_set_node_operator_voter(program_id, accounts)
        }
        RestakingInstruction::OperatorAddVault => {
            msg!("Instruction: NodeOperatorAddVault");
            process_operator_add_vault(program_id, accounts)
        }
        RestakingInstruction::OperatorRemoveVault => {
            msg!("Instruction: NodeOperatorRemoveVault");
            process_node_operator_remove_vault(program_id, accounts)
        }
        RestakingInstruction::OperatorAddAvs => {
            msg!("Instruction: OperatorAddAvs");
            process_operator_add_avs(program_id, accounts)
        }
        RestakingInstruction::OperatorRemoveAvs => {
            msg!("Instruction: OperatorRemoveAvs");
            process_operator_remove_avs(program_id, accounts)
        }
        RestakingInstruction::GetMaxSlashablePerEpoch(request) => {
            msg!("Instruction: GetMaxSlashablePerEpoch");
            process_get_max_slashable_per_epoch(program_id, accounts, request)
        }
    }
}
