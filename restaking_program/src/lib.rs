mod processor;

use borsh::BorshDeserialize;
use jito_restaking_sdk::RestakingInstruction;
use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey,
};
#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

use crate::processor::RestakingProcessor;

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
            RestakingProcessor::initialize_config(program_id, accounts)
        }
        RestakingInstruction::InitializeAvs => {
            msg!("Instruction: InitializeAvs");
            RestakingProcessor::initialize_avs(program_id, accounts)
        }
        RestakingInstruction::AvsAddVault => {
            msg!("Instruction: AvsAddVault");
            RestakingProcessor::avs_add_vault(program_id, accounts)
        }
        RestakingInstruction::AvsRemoveVault => {
            msg!("Instruction: AvsRemoveVault");
            RestakingProcessor::avs_remove_vault(program_id, accounts)
        }
        RestakingInstruction::AvsAddNodeOperator => {
            msg!("Instruction: AvsAddNodeOperator");
            RestakingProcessor::avs_add_node_operator(program_id, accounts)
        }
        RestakingInstruction::AvsRemoveNodeOperator => {
            msg!("Instruction: AvsRemoveNodeOperator");
            RestakingProcessor::avs_remove_node_operator(program_id, accounts)
        }
        RestakingInstruction::AvsAddVaultSlasher(max_slashable_per_epoch) => {
            msg!("Instruction: AvsAddVaultSlasher");
            RestakingProcessor::avs_add_vault_slasher(program_id, accounts, max_slashable_per_epoch)
        }
        RestakingInstruction::AvsDeprecateVaultSlasher => {
            msg!("Instruction: AvsDeprecateVaultSlasher");
            RestakingProcessor::avs_deprecate_slasher(program_id, accounts)
        }
        RestakingInstruction::InitializeOperator => {
            msg!("Instruction: InitializeNodeOperator");
            RestakingProcessor::initialize_node_operator(program_id, accounts)
        }
        RestakingInstruction::OperatorSetAdmin => {
            msg!("Instruction: OperatorSetAdmin");
            RestakingProcessor::set_node_operator_admin(program_id, accounts)
        }
        RestakingInstruction::OperatorSetVoter => {
            msg!("Instruction: OperatorSetVoter");
            RestakingProcessor::set_node_operator_voter(program_id, accounts)
        }
        RestakingInstruction::OperatorAddVault => {
            msg!("Instruction: NodeOperatorAddVault");
            RestakingProcessor::node_operator_add_vault(program_id, accounts)
        }
        RestakingInstruction::OperatorRemoveVault => {
            msg!("Instruction: NodeOperatorRemoveVault");
            RestakingProcessor::node_operator_remove_vault(program_id, accounts)
        }
        RestakingInstruction::OperatorAddAvs => {
            msg!("Instruction: OperatorAddAvs");
            RestakingProcessor::node_operator_add_avs(program_id, accounts)
        }
        RestakingInstruction::OperatorRemoveAvs => {
            msg!("Instruction: OperatorRemoveAvs");
            RestakingProcessor::node_operator_remove_avs(program_id, accounts)
        }
    }
}
