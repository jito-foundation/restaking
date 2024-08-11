mod initialize_config;
mod initialize_ncn;
mod initialize_operator;
mod ncn_add_operator;
mod ncn_add_vault;
mod ncn_add_vault_slasher;
mod ncn_cooldown_operator;
mod ncn_cooldown_vault;
mod ncn_cooldown_vault_slasher;
mod ncn_set_admin;
mod ncn_set_secondary_admin;
mod ncn_withdraw_asset;
mod operator_add_ncn;
mod operator_add_vault;
mod operator_cooldown_ncn;
mod operator_cooldown_vault;
mod operator_set_admin;
mod operator_set_secondary_admin;
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
    initialize_config::process_initialize_config, initialize_ncn::process_initialize_ncn,
    initialize_operator::process_initialize_operator, ncn_add_operator::process_ncn_add_operator,
    ncn_add_vault::process_ncn_add_vault, ncn_add_vault_slasher::process_ncn_add_vault_slasher,
    ncn_cooldown_operator::process_ncn_cooldown_operator,
    ncn_cooldown_vault::process_ncn_cooldown_vault,
    ncn_cooldown_vault_slasher::process_ncn_remove_slasher, ncn_set_admin::process_ncn_set_admin,
    ncn_set_secondary_admin::process_ncn_set_secondary_admin,
    ncn_withdraw_asset::process_ncn_withdraw_asset, operator_add_ncn::process_operator_add_ncn,
    operator_add_vault::process_operator_add_vault,
    operator_cooldown_ncn::process_operator_cooldown_ncn,
    operator_cooldown_vault::process_operator_cooldown_vault,
    operator_set_admin::process_set_node_operator_admin,
    operator_set_secondary_admin::process_set_operator_secondary_admin,
    operator_withdrawal_asset::process_operator_withdrawal_asset,
};

declare_id!("E5YF9Um1mwQWHffqaUEUwtwnhQKsbMEt33qtvjto3NDZ");

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    // Required fields
    name: "Jito's Restaking Program",
    project_url: "https://jito.network/",
    contacts: "email:team@jito.network",
    policy: "https://github.com/jito-foundation/restaking",
    // Optional Fields
    preferred_languages: "en",
    source_code: "https://github.com/jito-foundation/restaking"
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
        RestakingInstruction::InitializeNcn => {
            msg!("Instruction: InitializeNcn");
            process_initialize_ncn(program_id, accounts)
        }
        RestakingInstruction::NcnAddVault => {
            msg!("Instruction: NcnAddVault");
            process_ncn_add_vault(program_id, accounts)
        }
        RestakingInstruction::NcnCooldownVault => {
            msg!("Instruction: NcnCooldownVault");
            process_ncn_cooldown_vault(program_id, accounts)
        }
        RestakingInstruction::NcnAddOperator => {
            msg!("Instruction: NcnAddOperator");
            process_ncn_add_operator(program_id, accounts)
        }
        RestakingInstruction::NcnCooldownOperator => {
            msg!("Instruction: NcnCooldownOperator");
            process_ncn_cooldown_operator(program_id, accounts)
        }
        RestakingInstruction::NcnAddVaultSlasher(max_slashable_per_epoch) => {
            msg!("Instruction: NcnAddVaultSlasher");
            process_ncn_add_vault_slasher(program_id, accounts, max_slashable_per_epoch)
        }
        RestakingInstruction::NcnCooldownVaultSlasher => {
            msg!("Instruction: NcnCooldownVaultSlasher");
            process_ncn_remove_slasher(program_id, accounts)
        }
        RestakingInstruction::NcnSetAdmin => {
            msg!("Instruction: NcnSetAdmin");
            process_ncn_set_admin(program_id, accounts)
        }
        RestakingInstruction::NcnSetSecondaryAdmin(role) => {
            msg!("Instruction: NcnSetSecondaryAdmin");
            process_ncn_set_secondary_admin(program_id, accounts, role)
        }
        RestakingInstruction::InitializeOperator => {
            msg!("Instruction: InitializeNodeOperator");
            process_initialize_operator(program_id, accounts)
        }
        RestakingInstruction::OperatorSetAdmin => {
            msg!("Instruction: OperatorSetAdmin");
            process_set_node_operator_admin(program_id, accounts)
        }
        RestakingInstruction::OperatorSetSecondaryAdmin(role) => {
            msg!("Instruction: OperatorSetSecondaryAdmin");
            process_set_operator_secondary_admin(program_id, accounts, role)
        }
        RestakingInstruction::OperatorAddVault => {
            msg!("Instruction: OperatorAddVault");
            process_operator_add_vault(program_id, accounts)
        }
        RestakingInstruction::OperatorCooldownVault => {
            msg!("Instruction: OperatorCooldownVault");
            process_operator_cooldown_vault(program_id, accounts)
        }
        RestakingInstruction::OperatorAddNcn => {
            msg!("Instruction: OperatorAddNcn");
            process_operator_add_ncn(program_id, accounts)
        }
        RestakingInstruction::OperatorCooldownNcn => {
            msg!("Instruction: OperatorCooldownNcn");
            process_operator_cooldown_ncn(program_id, accounts)
        }
        RestakingInstruction::NcnWithdrawalAsset { token_mint, amount } => {
            msg!("Instruction: NcnWithdrawalAsset");
            process_ncn_withdraw_asset(program_id, accounts, token_mint, amount)
        }
        RestakingInstruction::OperatorWithdrawalAsset { token_mint, amount } => {
            msg!("Instruction: OperatorWithdrawalAsset");
            process_operator_withdrawal_asset(program_id, accounts, token_mint, amount)
        }
    }
}
