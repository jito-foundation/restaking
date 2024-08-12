use borsh::BorshSerialize;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use crate::instruction::{VaultAdminRole, VaultInstruction};

pub fn initialize_config(
    program_id: &Pubkey,
    config: &Pubkey,
    admin: &Pubkey,
    restaking_program: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*config, false),
        AccountMeta::new(*admin, true),
        AccountMeta::new_readonly(*restaking_program, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::InitializeConfig.try_to_vec().unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn initialize_vault(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    lrt_mint: &Pubkey,
    token_mint: &Pubkey,
    admin: &Pubkey,
    base: &Pubkey,
    deposit_fee_bps: u16,
    withdrawal_fee_bps: u16,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*lrt_mint, true),
        AccountMeta::new_readonly(*token_mint, false),
        AccountMeta::new(*admin, true),
        AccountMeta::new_readonly(*base, true),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::InitializeVault {
            deposit_fee_bps,
            withdrawal_fee_bps,
        }
        .try_to_vec()
        .unwrap(),
    }
}

pub fn initialize_vault_delegation_list(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    vault_delegation_list: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new_readonly(*config, false),
            AccountMeta::new_readonly(*vault, false),
            AccountMeta::new(*vault_delegation_list, false),
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: VaultInstruction::InitializeVaultDelegationList
            .try_to_vec()
            .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn initialize_vault_ncn_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    ncn: &Pubkey,
    ncn_vault_ticket: &Pubkey,
    vault_ncn_ticket: &Pubkey,
    admin: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*ncn_vault_ticket, false),
        AccountMeta::new(*vault_ncn_ticket, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::InitializeVaultNcnTicket
            .try_to_vec()
            .unwrap(),
    }
}

pub fn cooldown_vault_ncn_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    ncn: &Pubkey,
    vault_ncn_ticket: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new(*vault_ncn_ticket, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::CooldownVaultNcnTicket
            .try_to_vec()
            .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn initialize_vault_operator_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    operator: &Pubkey,
    operator_vault_ticket: &Pubkey,
    vault_operator_ticket: &Pubkey,
    admin: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*operator, false),
        AccountMeta::new_readonly(*operator_vault_ticket, false),
        AccountMeta::new(*vault_operator_ticket, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::InitializeVaultOperatorTicket
            .try_to_vec()
            .unwrap(),
    }
}

pub fn cooldown_vault_operator_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    operator: &Pubkey,
    vault_operator_ticket: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new(*vault_operator_ticket, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::CooldownVaultOperatorTicket
            .try_to_vec()
            .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn mint_to(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    lrt_mint: &Pubkey,
    depositor: &Pubkey,
    depositor_token_account: &Pubkey,
    vault_token_account: &Pubkey,
    depositor_lrt_token_account: &Pubkey,
    vault_fee_token_account: &Pubkey,
    mint_signer: Option<&Pubkey>,
    amount: u64,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*lrt_mint, false),
        AccountMeta::new(*depositor, true),
        AccountMeta::new(*depositor_token_account, false),
        AccountMeta::new(*vault_token_account, false),
        AccountMeta::new(*depositor_lrt_token_account, false),
        AccountMeta::new(*vault_fee_token_account, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    if let Some(signer) = mint_signer {
        accounts.push(AccountMeta::new_readonly(*signer, true));
    }
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::MintTo { amount }.try_to_vec().unwrap(),
    }
}

pub fn burn(program_id: &Pubkey, amount: u64) -> Instruction {
    Instruction {
        program_id: *program_id,
        accounts: vec![],
        data: VaultInstruction::Burn { amount }.try_to_vec().unwrap(),
    }
}

pub fn set_deposit_capacity(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    admin: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::SetDepositCapacity { amount }
            .try_to_vec()
            .unwrap(),
    }
}

pub fn set_fees(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    admin: &Pubkey,
    deposit_fee_bps: u16,
    withdrawal_fee_bps: u16,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::SetFees {
            deposit_fee_bps,
            withdrawal_fee_bps,
        }
        .try_to_vec()
        .unwrap(),
    }
}

pub fn withdrawal_asset(program_id: &Pubkey, amount: u64) -> Instruction {
    Instruction {
        program_id: *program_id,
        accounts: vec![],
        data: VaultInstruction::AdminWithdraw { amount }
            .try_to_vec()
            .unwrap(),
    }
}

pub fn set_admin(
    program_id: &Pubkey,
    vault: &Pubkey,
    old_admin: &Pubkey,
    new_admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*vault, false),
        AccountMeta::new_readonly(*old_admin, true),
        AccountMeta::new_readonly(*new_admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::SetAdmin.try_to_vec().unwrap(),
    }
}

pub fn set_secondary_admin(
    program_id: &Pubkey,
    vault: &Pubkey,
    admin: &Pubkey,
    new_admin: &Pubkey,
    role: VaultAdminRole,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*vault, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new_readonly(*new_admin, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::SetSecondaryAdmin(role)
            .try_to_vec()
            .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn add_delegation(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    operator: &Pubkey,
    vault_operator_ticket: &Pubkey,
    vault_delegation_list: &Pubkey,
    admin: &Pubkey,
    payer: &Pubkey,

    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new_readonly(*vault_operator_ticket, false),
        AccountMeta::new(*vault_delegation_list, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::AddDelegation { amount }
            .try_to_vec()
            .unwrap(),
    }
}

pub fn cooldown_delegation(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    operator: &Pubkey,
    vault_delegation_list: &Pubkey,
    admin: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new(*vault_delegation_list, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::CooldownDelegation { amount }
            .try_to_vec()
            .unwrap(),
    }
}

pub fn update_vault(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    vault_delegation_list: &Pubkey,
    vault_token_account: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*vault_delegation_list, false),
        AccountMeta::new(*vault_token_account, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::UpdateVault.try_to_vec().unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn initialize_vault_ncn_slasher_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    ncn: &Pubkey,
    slasher: &Pubkey,
    ncn_slasher_ticket: &Pubkey,
    vault_slasher_ticket: &Pubkey,
    admin: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*slasher, false),
        AccountMeta::new_readonly(*ncn_slasher_ticket, false),
        AccountMeta::new(*vault_slasher_ticket, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::InitializeVaultNcnSlasherTicket
            .try_to_vec()
            .unwrap(),
    }
}

pub fn create_token_metadata(
    program_id: &Pubkey,
    name: String,
    symbol: String,
    uri: String,
) -> Instruction {
    Instruction {
        program_id: *program_id,
        accounts: vec![],
        data: VaultInstruction::CreateTokenMetadata { name, symbol, uri }
            .try_to_vec()
            .unwrap(),
    }
}

pub fn update_token_metadata(
    program_id: &Pubkey,
    name: String,
    symbol: String,
    uri: String,
) -> Instruction {
    Instruction {
        program_id: *program_id,
        accounts: vec![],
        data: VaultInstruction::UpdateTokenMetadata { name, symbol, uri }
            .try_to_vec()
            .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn initialize_vault_ncn_slasher_operator_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    ncn: &Pubkey,
    slasher: &Pubkey,
    operator: &Pubkey,
    vault_ncn_slasher_ticket: &Pubkey,
    vault_ncn_slasher_operator_ticket: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*slasher, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new_readonly(*vault_ncn_slasher_ticket, false),
        AccountMeta::new(*vault_ncn_slasher_operator_ticket, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::InitializeVaultNcnSlasherOperatorTicket
            .try_to_vec()
            .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn slash(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    ncn: &Pubkey,
    operator: &Pubkey,
    slasher: &Pubkey,
    ncn_operator_ticket: &Pubkey,
    operator_ncn_ticket: &Pubkey,
    ncn_vault_ticket: &Pubkey,
    operator_vault_ticket: &Pubkey,
    vault_ncn_ticket: &Pubkey,
    vault_operator_ticket: &Pubkey,
    ncn_vault_slasher_ticket: &Pubkey,
    vault_ncn_slasher_ticket: &Pubkey,
    vault_delegation_list: &Pubkey,
    vault_ncn_slasher_operator_ticket: &Pubkey,
    vault_token_account: &Pubkey,
    slasher_token_account: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new_readonly(*slasher, false),
        AccountMeta::new_readonly(*ncn_operator_ticket, false),
        AccountMeta::new_readonly(*operator_ncn_ticket, false),
        AccountMeta::new_readonly(*ncn_vault_ticket, false),
        AccountMeta::new_readonly(*operator_vault_ticket, false),
        AccountMeta::new_readonly(*vault_ncn_ticket, false),
        AccountMeta::new_readonly(*vault_operator_ticket, false),
        AccountMeta::new_readonly(*ncn_vault_slasher_ticket, false),
        AccountMeta::new_readonly(*vault_ncn_slasher_ticket, false),
        AccountMeta::new(*vault_delegation_list, false),
        AccountMeta::new(*vault_ncn_slasher_operator_ticket, false),
        AccountMeta::new(*vault_token_account, false),
        AccountMeta::new(*slasher_token_account, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::Slash { amount }.try_to_vec().unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn enqueue_withdraw(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    vault_delegation_list: &Pubkey,
    vault_staker_withdrawal_ticket: &Pubkey,
    vault_staker_withdrawal_ticket_token_account: &Pubkey,
    vault_fee_token_account: &Pubkey,
    staker: &Pubkey,
    staker_lrt_token_account: &Pubkey,
    base: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*vault_delegation_list, false),
        AccountMeta::new(*vault_staker_withdrawal_ticket, false),
        AccountMeta::new(*vault_staker_withdrawal_ticket_token_account, false),
        AccountMeta::new(*vault_fee_token_account, false),
        AccountMeta::new(*staker, true),
        AccountMeta::new(*staker_lrt_token_account, false),
        AccountMeta::new_readonly(*base, true),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::EnqueueWithdrawal { amount }
            .try_to_vec()
            .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn burn_withdrawal_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    vault_delegation_list: &Pubkey,
    vault_token_account: &Pubkey,
    lrt_mint: &Pubkey,
    staker: &Pubkey,
    staker_token_account: &Pubkey,
    staker_lrt_token_account: &Pubkey,
    vault_staker_withdrawal_ticket: &Pubkey,
    vault_staker_withdrawal_ticket_token_account: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*vault_delegation_list, false),
        AccountMeta::new(*vault_token_account, false),
        AccountMeta::new(*lrt_mint, false),
        AccountMeta::new(*staker, true),
        AccountMeta::new(*staker_token_account, false),
        AccountMeta::new(*staker_lrt_token_account, false),
        AccountMeta::new(*vault_staker_withdrawal_ticket, false),
        AccountMeta::new(*vault_staker_withdrawal_ticket_token_account, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::BurnWithdrawTicket.try_to_vec().unwrap(),
    }
}
