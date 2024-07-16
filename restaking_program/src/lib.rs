mod avs_add_operator;
mod avs_add_vault;
mod avs_add_vault_slasher;
mod avs_remove_operator;
mod avs_remove_vault;
mod avs_remove_vault_slasher;
mod avs_set_admin;
mod avs_set_secondary_admin;
mod avs_withdraw_asset;
mod initialize_avs;
mod initialize_config;
mod initialize_operator;
mod operator_add_avs;
mod operator_add_vault;
mod operator_remove_avs;
mod operator_remove_vault;
mod operator_set_admin;
mod operator_set_voter;
mod operator_withdrawal_asset;

use borsh::BorshDeserialize;
use jito_restaking_sdk::RestakingInstruction;
use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey,
};
#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

use crate::{
    avs_add_operator::process_avs_add_operator, avs_add_vault::process_avs_add_vault,
    avs_add_vault_slasher::process_avs_add_vault_slasher,
    avs_remove_operator::process_avs_remove_operator, avs_remove_vault::process_avs_remove_vault,
    avs_remove_vault_slasher::process_avs_remove_slasher, avs_set_admin::process_avs_set_admin,
    avs_set_secondary_admin::process_avs_set_secondary_admin,
    avs_withdraw_asset::process_avs_withdraw_asset, initialize_avs::process_initialize_avs,
    initialize_config::process_initialize_config, initialize_operator::process_initialize_operator,
    operator_add_avs::process_operator_add_avs, operator_add_vault::process_operator_add_vault,
    operator_remove_avs::process_operator_remove_avs,
    operator_remove_vault::process_operator_remove_vault,
    operator_set_admin::process_set_node_operator_admin,
    operator_set_voter::process_set_node_operator_voter,
    operator_withdrawal_asset::process_operator_withdrawal_asset,
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
            msg!("Instruction: AvsAddOperator");
            process_avs_add_operator(program_id, accounts)
        }
        RestakingInstruction::AvsRemoveOperator => {
            msg!("Instruction: AvsRemoveOperator");
            process_avs_remove_operator(program_id, accounts)
        }
        RestakingInstruction::AvsAddVaultSlasher(max_slashable_per_epoch) => {
            msg!("Instruction: AvsAddVaultSlasher");
            process_avs_add_vault_slasher(program_id, accounts, max_slashable_per_epoch)
        }
        RestakingInstruction::AvsRemoveVaultSlasher => {
            msg!("Instruction: AvsRemoveVaultSlasher");
            process_avs_remove_slasher(program_id, accounts)
        }
        RestakingInstruction::AvsSetAdmin => {
            msg!("Instruction: AvsSetAdmin");
            process_avs_set_admin(program_id, accounts)
        }
        RestakingInstruction::AvsSetSecondaryAdmin(role) => {
            msg!("Instruction: AvsSetSecondaryAdmin");
            process_avs_set_secondary_admin(program_id, accounts, role)
        }
        RestakingInstruction::InitializeOperator => {
            msg!("Instruction: InitializeNodeOperator");
            process_initialize_operator(program_id, accounts)
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
            msg!("Instruction: OperatorAddVault");
            process_operator_add_vault(program_id, accounts)
        }
        RestakingInstruction::OperatorRemoveVault => {
            msg!("Instruction: OperatorRemoveVault");
            process_operator_remove_vault(program_id, accounts)
        }
        RestakingInstruction::OperatorAddAvs => {
            msg!("Instruction: OperatorAddAvs");
            process_operator_add_avs(program_id, accounts)
        }
        RestakingInstruction::OperatorRemoveAvs => {
            msg!("Instruction: OperatorRemoveAvs");
            process_operator_remove_avs(program_id, accounts)
        }
        RestakingInstruction::AvsWithdrawalAsset { token_mint, amount } => {
            msg!("Instruction: AvsWithdrawalAsset");
            process_avs_withdraw_asset(program_id, accounts, token_mint, amount)
        }
        RestakingInstruction::OperatorWithdrawalAsset { token_mint, amount } => {
            msg!("Instruction: OperatorWithdrawalAsset");
            process_operator_withdrawal_asset(program_id, accounts, token_mint, amount)
        }
    }
}
