use borsh::BorshSerialize;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use crate::instruction::{NcnAdminRole, OperatorAdminRole, RestakingInstruction};

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
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::InitializeConfig.try_to_vec().unwrap(),
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
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::InitializeNcn.try_to_vec().unwrap(),
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
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::InitializeNcnVaultTicket
            .try_to_vec()
            .unwrap(),
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
        data: RestakingInstruction::CooldownNcnVaultTicket
            .try_to_vec()
            .unwrap(),
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
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::InitializeNcnOperatorState
            .try_to_vec()
            .unwrap(),
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
        data: RestakingInstruction::NcnCooldownOperator
            .try_to_vec()
            .unwrap(),
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

    max_slash_amount: u64,
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
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::InitializeNcnVaultSlasherTicket(max_slash_amount)
            .try_to_vec()
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
        data: RestakingInstruction::CooldownNcnVaultSlasherTicket
            .try_to_vec()
            .unwrap(),
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
        data: RestakingInstruction::NcnSetAdmin.try_to_vec().unwrap(),
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
        data: RestakingInstruction::NcnSetSecondaryAdmin(role)
            .try_to_vec()
            .unwrap(),
    }
}

pub fn initialize_operator(
    program_id: &Pubkey,
    config: &Pubkey,
    operator: &Pubkey,
    admin: &Pubkey,
    base: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*config, false),
        AccountMeta::new(*operator, false),
        AccountMeta::new(*admin, true),
        AccountMeta::new_readonly(*base, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::InitializeOperator
            .try_to_vec()
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
        data: RestakingInstruction::OperatorSetAdmin.try_to_vec().unwrap(),
    }
}

pub fn operator_set_secondary_admin(
    program_id: &Pubkey,
    operator: &Pubkey,
    admin: &Pubkey,
    voter: &Pubkey,
    operator_admin_role: OperatorAdminRole,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*operator, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new_readonly(*voter, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::OperatorSetSecondaryAdmin(operator_admin_role)
            .try_to_vec()
            .unwrap(),
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
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::InitializeOperatorVaultTicket
            .try_to_vec()
            .unwrap(),
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
        data: RestakingInstruction::CooldownOperatorVaultTicket
            .try_to_vec()
            .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ncn_withdrawal_asset(
    program_id: &Pubkey,
    ncn: &Pubkey,
    ncn_token_account: &Pubkey,
    receiver_token_account: &Pubkey,
    admin: &Pubkey,
    token_mint: &Pubkey,
    token_program: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new(*ncn_token_account, false),
        AccountMeta::new(*receiver_token_account, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new_readonly(*token_mint, false),
        AccountMeta::new_readonly(*token_program, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::NcnWithdrawalAsset { amount }
            .try_to_vec()
            .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn operator_withdrawal_asset(
    program_id: &Pubkey,
    operator: &Pubkey,
    admin: &Pubkey,
    operator_token_account: &Pubkey,
    receiver_token_account: &Pubkey,
    token_mint: &Pubkey,
    token_program: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new(*operator_token_account, false),
        AccountMeta::new(*receiver_token_account, false),
        AccountMeta::new_readonly(*token_mint, false),
        AccountMeta::new_readonly(*token_program, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::OperatorWithdrawalAsset { amount }
            .try_to_vec()
            .unwrap(),
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
        data: RestakingInstruction::OperatorWarmupNcn
            .try_to_vec()
            .unwrap(),
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
        data: RestakingInstruction::OperatorCooldownNcn
            .try_to_vec()
            .unwrap(),
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
        data: RestakingInstruction::NcnWarmupOperator
            .try_to_vec()
            .unwrap(),
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
        data: RestakingInstruction::NcnCooldownOperator
            .try_to_vec()
            .unwrap(),
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
        data: RestakingInstruction::WarmupNcnVaultTicket
            .try_to_vec()
            .unwrap(),
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
        data: RestakingInstruction::WarmupOperatorVaultTicket
            .try_to_vec()
            .unwrap(),
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
        data: RestakingInstruction::WarmupNcnVaultSlasherTicket
            .try_to_vec()
            .unwrap(),
    }
}
