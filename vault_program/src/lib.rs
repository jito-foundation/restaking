mod add_avs;
mod add_delegation;
mod add_operator;
mod add_slasher;
mod burn;
mod burn_withdraw_ticket;
mod create_token_metadata;
mod enqueue_withdraw;
mod initialize_config;
mod initialize_vault;
mod initialize_vault_avs_slasher_operator_ticket;
mod initialize_vault_with_mint;
mod mint_to;
mod remove_avs;
mod remove_delegation;
mod remove_operator;
mod set_admin;
mod set_capacity;
mod set_secondary_admin;
mod slash;
mod update_token_metadata;
mod update_vault;
mod withdrawal_asset;

use borsh::BorshDeserialize;
use jito_vault_sdk::VaultInstruction;
use solana_program::{
    account_info::AccountInfo, declare_id, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey,
};
#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

use crate::{
    add_avs::process_vault_add_avs, add_delegation::process_add_delegation,
    add_operator::process_vault_add_operator, add_slasher::process_add_slasher, burn::process_burn,
    burn_withdraw_ticket::process_burn_withdraw_ticket,
    create_token_metadata::process_create_token_metadata,
    enqueue_withdraw::process_enqueue_withdraw, initialize_config::process_initialize_config,
    initialize_vault::process_initialize_vault,
    initialize_vault_avs_slasher_operator_ticket::process_initialize_vault_avs_slasher_operator_ticket,
    initialize_vault_with_mint::process_initialize_vault_with_mint, mint_to::process_mint,
    remove_avs::process_vault_remove_avs, remove_delegation::process_remove_delegation,
    remove_operator::process_vault_remove_operator, set_admin::process_set_admin,
    set_capacity::process_set_capacity, set_secondary_admin::process_set_secondary_admin,
    slash::process_slash, update_token_metadata::process_update_token_metadata,
    update_vault::process_update_vault, withdrawal_asset::process_withdrawal_asset,
};

declare_id!("DVoKuzt4i8EAakix852XwSAYmXnECdhegB6EDtabp4dg");

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    // Required fields
    name: "Jito's Liquid Restaking Program",
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

    let instruction = VaultInstruction::try_from_slice(instruction_data)?;

    match instruction {
        // ------------------------------------------
        // Initialization
        // ------------------------------------------
        VaultInstruction::InitializeConfig => {
            msg!("Instruction: InitializeConfig");
            process_initialize_config(program_id, accounts)
        }
        VaultInstruction::InitializeVault {
            deposit_fee_bps,
            withdrawal_fee_bps,
        } => {
            msg!("Instruction: InitializeVault");
            process_initialize_vault(program_id, accounts, deposit_fee_bps, withdrawal_fee_bps)
        }
        VaultInstruction::InitializeVaultWithMint => {
            msg!("Instruction: InitializeVaultWithMint");
            process_initialize_vault_with_mint(program_id, accounts)
        }
        // ------------------------------------------
        // Vault administration
        // ------------------------------------------
        VaultInstruction::SetSecondaryAdmin(role) => {
            msg!("Instruction: SetDelegationAdmin");
            process_set_secondary_admin(program_id, accounts, role)
        }
        VaultInstruction::SetAdmin => {
            msg!("Instruction: SetAdmin");
            process_set_admin(program_id, accounts)
        }
        VaultInstruction::SetDepositCapacity { amount } => {
            msg!("Instruction: SetCapacity");
            process_set_capacity(program_id, accounts, amount)
        }
        VaultInstruction::AdminWithdraw { amount } => {
            msg!("Instruction: WithdrawalAsset");
            process_withdrawal_asset(program_id, accounts, amount)
        }
        // ------------------------------------------
        // Vault minting and burning
        // ------------------------------------------
        VaultInstruction::MintTo { amount } => {
            msg!("Instruction: MintTo");
            process_mint(program_id, accounts, amount)
        }
        VaultInstruction::Burn { amount } => {
            msg!("Instruction: Burn");
            process_burn(program_id, accounts, amount)
        }
        VaultInstruction::EnqueueWithdraw { amount } => {
            msg!("Instruction: EnqueueWithdraw");
            process_enqueue_withdraw(program_id, accounts, amount)
        }
        VaultInstruction::BurnWithdrawTicket => {
            msg!("Instruction: BurnWithdrawTicket");
            process_burn_withdraw_ticket(program_id, accounts)
        }
        // ------------------------------------------
        // Vault-AVS operations
        // ------------------------------------------
        VaultInstruction::AddAvs => {
            msg!("Instruction: AddAvs");
            process_vault_add_avs(program_id, accounts)
        }
        VaultInstruction::RemoveAvs => {
            msg!("Instruction: RemoveAvs");
            process_vault_remove_avs(program_id, accounts)
        }
        // ------------------------------------------
        // Vault-operator operations
        // ------------------------------------------
        VaultInstruction::AddOperator => {
            msg!("Instruction: AddOperator");
            process_vault_add_operator(program_id, accounts)
        }
        VaultInstruction::RemoveOperator => {
            msg!("Instruction: RemoveOperator");
            process_vault_remove_operator(program_id, accounts)
        }
        // ------------------------------------------
        // Vault delegation
        // ------------------------------------------
        VaultInstruction::AddDelegation { amount } => {
            msg!("Instruction: AddDelegation");
            process_add_delegation(program_id, accounts, amount)
        }
        VaultInstruction::RemoveDelegation { amount } => {
            msg!("Instruction: RemoveDelegation");
            process_remove_delegation(program_id, accounts, amount)
        }
        VaultInstruction::UpdateVault => {
            msg!("Instruction: UpdateDelegations");
            process_update_vault(program_id, accounts)
        }
        // ------------------------------------------
        // Vault slashing
        // ------------------------------------------
        VaultInstruction::AddSlasher => {
            msg!("Instruction: RegisterSlasher");
            process_add_slasher(program_id, accounts)
        }
        VaultInstruction::InitializeVaultAvsSlasherOperatorTicket => {
            msg!("Instruction: InitializeVaultAvsSlasherOperatorTicket");
            process_initialize_vault_avs_slasher_operator_ticket(program_id, accounts)
        }
        VaultInstruction::Slash { amount } => {
            msg!("Instruction: Slash");
            process_slash(program_id, accounts, amount)
        }
        // ------------------------------------------
        // LRT metadata
        // ------------------------------------------
        VaultInstruction::CreateTokenMetadata { name, symbol, uri } => {
            msg!("Instruction: CreateTokenMetadata");
            process_create_token_metadata(program_id, accounts, name, symbol, uri)
        }
        VaultInstruction::UpdateTokenMetadata { name, symbol, uri } => {
            msg!("Instruction: UpdateTokenMetadata");
            process_update_token_metadata(program_id, accounts, name, symbol, uri)
        }
    }
}
