mod processor;

use borsh::BorshDeserialize;
use jito_lrt_sdk::LrtInstruction;
use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey,
};
#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

use crate::processor::LrtProcessor;

declare_id!("DVoKuzt4i8EAakix852XwSAYmXnECdhegB6EDtabp4dg");

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    // Required fields
    name: "Jito's Liquid Restaking Program",
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

    let instruction = LrtInstruction::try_from_slice(instruction_data)?;

    match instruction {
        LrtInstruction::InitializeConfig => {
            msg!("Instruction: InitializeConfig");
            LrtProcessor::initialize_config(program_id, accounts)
        }
        LrtInstruction::InitializeVault => {
            msg!("Instruction: InitializeVault");
            LrtProcessor::initialize_vault(program_id, accounts)
        }
        LrtInstruction::SetCapacity(amount) => {
            msg!("Instruction: SetCapacity");
            LrtProcessor::set_capacity(program_id, accounts, amount)
        }
        LrtInstruction::MintTo(amount) => {
            msg!("Instruction: MintTo");
            LrtProcessor::mint(program_id, accounts, amount)
        }
        LrtInstruction::AddAvs => {
            msg!("Instruction: AddAvs");
            LrtProcessor::vault_add_avs(program_id, accounts)
        }
        LrtInstruction::RemoveAvs => {
            msg!("Instruction: RemoveAvs");
            LrtProcessor::vault_remove_avs(program_id, accounts)
        }
        LrtInstruction::AddOperator => {
            msg!("Instruction: AddOperator");
            LrtProcessor::vault_add_node_operator(program_id, accounts)
        }
        LrtInstruction::RemoveOperator => {
            msg!("Instruction: RemoveOperator");
            LrtProcessor::vault_remove_node_operator(program_id, accounts)
        }
        LrtInstruction::AddDelegation(amount) => {
            msg!("Instruction: AddDelegation");
            LrtProcessor::add_delegation(program_id, accounts, amount)
        }
        LrtInstruction::RemoveDelegation(amount) => {
            msg!("Instruction: RemoveDelegation");
            LrtProcessor::remove_delegation(program_id, accounts, amount)
        }
        LrtInstruction::SetDelegationAdmin => {
            msg!("Instruction: SetDelegationAdmin");
            LrtProcessor::set_delegation_admin(program_id, accounts)
        }
        LrtInstruction::SetAdmin => {
            msg!("Instruction: SetAdmin");
            LrtProcessor::set_admin(program_id, accounts)
        }
    }
}
