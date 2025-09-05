use crate::instruction::{NcnAdminRole, OperatorAdminRole, RestakingInstruction};
use borsh::BorshSerialize;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

pub fn initialize_config(
    program_id: &Pubkey,
    config: &Pubkey,
    admin: &Pubkey,
    vault_program: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*config, false),
        AccountMeta::new(*admin, true),
        AccountMeta::new_readonly(*vault_program, false),
        AccountMeta::new_readonly(solana_system_interface::program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::InitializeConfig).unwrap(),
    }
}

pub fn initialize_ncn(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    admin: &Pubkey,
    base: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*config, false),
        AccountMeta::new(*ncn, false),
        AccountMeta::new(*admin, true),
        AccountMeta::new_readonly(*base, true),
        AccountMeta::new_readonly(solana_system_interface::program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::InitializeNcn).unwrap(),
    }
}

pub fn initialize_ncn_vault_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    vault: &Pubkey,
    ncn_vault_ticket: &Pubkey,
    admin: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*ncn, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new(*ncn_vault_ticket, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(solana_system_interface::program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::InitializeNcnVaultTicket).unwrap(),
    }
}

pub fn cooldown_ncn_vault_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    vault: &Pubkey,
    ncn_vault_ticket: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new(*ncn_vault_ticket, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::CooldownNcnVaultTicket).unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn initialize_ncn_operator_state(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    operator: &Pubkey,
    ncn_operator_state: &Pubkey,
    admin: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*ncn, false),
        AccountMeta::new(*operator, false),
        AccountMeta::new(*ncn_operator_state, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(solana_system_interface::program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::InitializeNcnOperatorState).unwrap(),
    }
}

pub fn cooldown_ncn_operator_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    operator: &Pubkey,
    ncn_operator_ticket: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new(*ncn_operator_ticket, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::NcnCooldownOperator).unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn initialize_ncn_vault_slasher_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    vault: &Pubkey,
    slasher: &Pubkey,
    ncn_vault_ticket: &Pubkey,
    ncn_vault_slasher_ticket: &Pubkey,
    admin: &Pubkey,
    payer: &Pubkey,
    max_slashable_per_epoch: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*ncn, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new_readonly(*slasher, false),
        AccountMeta::new_readonly(*ncn_vault_ticket, false),
        AccountMeta::new(*ncn_vault_slasher_ticket, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(solana_system_interface::program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::InitializeNcnVaultSlasherTicket {
            max_slashable_per_epoch,
        })
        .unwrap(),
    }
}

pub fn cooldown_ncn_vault_slasher_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    vault: &Pubkey,
    slasher: &Pubkey,
    ncn_vault_slasher_ticket: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new_readonly(*slasher, false),
        AccountMeta::new(*ncn_vault_slasher_ticket, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::CooldownNcnVaultSlasherTicket).unwrap(),
    }
}

pub fn ncn_set_admin(
    program_id: &Pubkey,
    ncn: &Pubkey,
    old_admin: &Pubkey,
    new_admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*ncn, false),
        AccountMeta::new_readonly(*old_admin, true),
        AccountMeta::new_readonly(*new_admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::NcnSetAdmin).unwrap(),
    }
}

pub fn ncn_set_secondary_admin(
    program_id: &Pubkey,
    ncn: &Pubkey,
    admin: &Pubkey,
    new_admin: &Pubkey,
    role: NcnAdminRole,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*ncn, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new_readonly(*new_admin, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::NcnSetSecondaryAdmin(role)).unwrap(),
    }
}

pub fn initialize_operator(
    program_id: &Pubkey,
    config: &Pubkey,
    operator: &Pubkey,
    admin: &Pubkey,
    base: &Pubkey,
    operator_fee_bps: u16,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*config, false),
        AccountMeta::new(*operator, false),
        AccountMeta::new(*admin, true),
        AccountMeta::new_readonly(*base, true),
        AccountMeta::new_readonly(solana_system_interface::program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::InitializeOperator { operator_fee_bps })
            .unwrap(),
    }
}

pub fn operator_set_admin(
    program_id: &Pubkey,
    operator: &Pubkey,
    old_admin: &Pubkey,
    new_admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*operator, false),
        AccountMeta::new_readonly(*old_admin, true),
        AccountMeta::new_readonly(*new_admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::OperatorSetAdmin).unwrap(),
    }
}

pub fn operator_set_secondary_admin(
    program_id: &Pubkey,
    operator: &Pubkey,
    admin: &Pubkey,
    new_admin: &Pubkey,
    operator_admin_role: OperatorAdminRole,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*operator, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new_readonly(*new_admin, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::OperatorSetSecondaryAdmin(
            operator_admin_role,
        ))
        .unwrap(),
    }
}

pub fn operator_set_fee(
    program_id: &Pubkey,
    config: &Pubkey,
    operator: &Pubkey,
    admin: &Pubkey,
    new_fee_bps: u16,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*operator, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::OperatorSetFee { new_fee_bps }).unwrap(),
    }
}

pub fn initialize_operator_vault_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    operator: &Pubkey,
    vault: &Pubkey,
    operator_vault_ticket: &Pubkey,
    admin: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*operator, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new(*operator_vault_ticket, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(solana_system_interface::program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::InitializeOperatorVaultTicket).unwrap(),
    }
}

pub fn cooldown_operator_vault_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    operator: &Pubkey,
    vault: &Pubkey,
    operator_vault_ticket: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new(*operator_vault_ticket, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::CooldownOperatorVaultTicket).unwrap(),
    }
}

pub fn ncn_delegate_token_account(
    program_id: &Pubkey,
    ncn: &Pubkey,
    delegate_admin: &Pubkey,
    token_mint: &Pubkey,
    token_account: &Pubkey,
    delegate: &Pubkey,
    token_program: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*delegate_admin, true),
        AccountMeta::new(*token_mint, false),
        AccountMeta::new(*token_account, false),
        AccountMeta::new_readonly(*delegate, false),
        AccountMeta::new_readonly(*token_program, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::NcnDelegateTokenAccount).unwrap(),
    }
}

pub fn operator_delegate_token_account(
    program_id: &Pubkey,
    operator: &Pubkey,
    delegate_admin: &Pubkey,
    token_mint: &Pubkey,
    token_account: &Pubkey,
    delegate: &Pubkey,
    token_program: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new_readonly(*delegate_admin, true),
        AccountMeta::new(*token_mint, false),
        AccountMeta::new(*token_account, false),
        AccountMeta::new_readonly(*delegate, false),
        AccountMeta::new_readonly(*token_program, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::OperatorDelegateTokenAccount).unwrap(),
    }
}

pub fn operator_warmup_ncn(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    operator: &Pubkey,
    ncn_operator_state: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new(*ncn_operator_state, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::OperatorWarmupNcn).unwrap(),
    }
}

pub fn operator_cooldown_ncn(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    operator: &Pubkey,
    ncn_operator_state: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new(*ncn_operator_state, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::OperatorCooldownNcn).unwrap(),
    }
}

pub fn ncn_warmup_operator(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    operator: &Pubkey,
    ncn_operator_state: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new(*ncn_operator_state, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::NcnWarmupOperator).unwrap(),
    }
}

pub fn ncn_cooldown_operator(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    operator: &Pubkey,
    ncn_operator_state: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new(*ncn_operator_state, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::NcnCooldownOperator).unwrap(),
    }
}

pub fn warmup_ncn_vault_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    vault: &Pubkey,
    ncn_vault_ticket: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new(*ncn_vault_ticket, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::WarmupNcnVaultTicket).unwrap(),
    }
}

pub fn warmup_operator_vault_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    operator: &Pubkey,
    vault: &Pubkey,
    operator_vault_ticket: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new(*operator_vault_ticket, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::WarmupOperatorVaultTicket).unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn warmup_ncn_vault_slasher_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    vault: &Pubkey,
    slasher: &Pubkey,
    ncn_vault_ticket: &Pubkey,
    ncn_vault_slasher_ticket: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new_readonly(*slasher, false),
        AccountMeta::new_readonly(*ncn_vault_ticket, false),
        AccountMeta::new(*ncn_vault_slasher_ticket, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::WarmupNcnVaultSlasherTicket).unwrap(),
    }
}

pub fn set_config_admin(
    program_id: &Pubkey,
    config: &Pubkey,
    old_admin: &Pubkey,
    new_admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*config, false),
        AccountMeta::new_readonly(*old_admin, true),
        AccountMeta::new_readonly(*new_admin, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&RestakingInstruction::SetConfigAdmin).unwrap(),
    }
}
