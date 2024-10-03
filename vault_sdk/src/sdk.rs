use borsh::BorshSerialize;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use crate::{
    inline_mpl_token_metadata::{self},
    instruction::{VaultAdminRole, VaultInstruction, WithdrawalAllocationMethod},
};

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
    vrt_mint: &Pubkey,
    token_mint: &Pubkey,
    admin: &Pubkey,
    base: &Pubkey,
    deposit_fee_bps: u16,
    withdrawal_fee_bps: u16,
    reward_fee_bps: u16,
    decimals: u8,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*vrt_mint, true),
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
            reward_fee_bps,
            decimals,
        }
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
pub fn initialize_vault_operator_delegation(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    operator: &Pubkey,
    operator_vault_ticket: &Pubkey,
    vault_operator_delegation: &Pubkey,
    admin: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*operator, false),
        AccountMeta::new_readonly(*operator_vault_ticket, false),
        AccountMeta::new(*vault_operator_delegation, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::InitializeVaultOperatorDelegation
            .try_to_vec()
            .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn mint_to(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    vrt_mint: &Pubkey,
    depositor: &Pubkey,
    depositor_token_account: &Pubkey,
    vault_token_account: &Pubkey,
    depositor_vrt_token_account: &Pubkey,
    vault_fee_token_account: &Pubkey,
    mint_signer: Option<&Pubkey>,
    amount_in: u64,
    min_amount_out: u64,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*vrt_mint, false),
        AccountMeta::new(*depositor, true),
        AccountMeta::new(*depositor_token_account, false),
        AccountMeta::new(*vault_token_account, false),
        AccountMeta::new(*depositor_vrt_token_account, false),
        AccountMeta::new(*vault_fee_token_account, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    if let Some(signer) = mint_signer {
        accounts.push(AccountMeta::new_readonly(*signer, true));
    }
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::MintTo {
            amount_in,
            min_amount_out,
        }
        .try_to_vec()
        .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn burn(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    vault_token_account: &Pubkey,
    vrt_mint: &Pubkey,
    staker: &Pubkey,
    staker_token_account: &Pubkey,
    staker_vrt_token_account: &Pubkey,
    vault_fee_token_account: &Pubkey,
    burn_signer: Option<&Pubkey>,
    amount_in: u64,
    min_amount_out: u64,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*vault_token_account, false),
        AccountMeta::new(*vrt_mint, false),
        AccountMeta::new(*staker, true),
        AccountMeta::new(*staker_token_account, false),
        AccountMeta::new(*staker_vrt_token_account, false),
        AccountMeta::new(*vault_fee_token_account, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    if let Some(signer) = burn_signer {
        accounts.push(AccountMeta::new_readonly(*signer, true));
    }
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::Burn {
            amount_in,
            min_amount_out,
        }
        .try_to_vec()
        .unwrap(),
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
    deposit_fee_bps: Option<u16>,
    withdrawal_fee_bps: Option<u16>,
    reward_fee_bps: Option<u16>,
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
            reward_fee_bps,
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
    config: &Pubkey,
    vault: &Pubkey,
    old_admin: &Pubkey,
    new_admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
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
    config: &Pubkey,
    vault: &Pubkey,
    admin: &Pubkey,
    new_admin: &Pubkey,
    role: VaultAdminRole,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
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

pub fn add_delegation(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    operator: &Pubkey,
    vault_operator_delegation: &Pubkey,
    admin: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new(*vault_operator_delegation, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::AddDelegation { amount }
            .try_to_vec()
            .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn cooldown_delegation(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    operator: &Pubkey,
    vault_operator_delegation: &Pubkey,
    admin: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new(*vault_operator_delegation, false),
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

pub fn crank_vault_update_state_tracker(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    operator: &Pubkey,
    vault_operator_delegation: &Pubkey,
    vault_update_state_tracker: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new(*vault_operator_delegation, false),
        AccountMeta::new(*vault_update_state_tracker, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::CrankVaultUpdateStateTracker
            .try_to_vec()
            .unwrap(),
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

#[allow(clippy::too_many_arguments)]
pub fn create_token_metadata(
    program_id: &Pubkey,
    vault: &Pubkey,
    admin: &Pubkey,
    vrt_mint: &Pubkey,
    payer: &Pubkey,
    metadata: &Pubkey,
    name: String,
    symbol: String,
    uri: String,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new_readonly(*vrt_mint, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new(*metadata, false),
        AccountMeta::new_readonly(inline_mpl_token_metadata::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::CreateTokenMetadata { name, symbol, uri }
            .try_to_vec()
            .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_token_metadata(
    program_id: &Pubkey,
    vault: &Pubkey,
    admin: &Pubkey,
    vrt_mint: &Pubkey,
    metadata: &Pubkey,
    name: String,
    symbol: String,
    uri: String,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new(*vrt_mint, false),
        AccountMeta::new(*metadata, false),
        AccountMeta::new_readonly(inline_mpl_token_metadata::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
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
    ncn_operator_state: &Pubkey,
    ncn_vault_ticket: &Pubkey,
    operator_vault_ticket: &Pubkey,
    vault_ncn_ticket: &Pubkey,
    vault_operator_delegation: &Pubkey,
    ncn_vault_slasher_ticket: &Pubkey,
    vault_ncn_slasher_ticket: &Pubkey,
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
        AccountMeta::new_readonly(*ncn_operator_state, false),
        AccountMeta::new_readonly(*ncn_vault_ticket, false),
        AccountMeta::new_readonly(*operator_vault_ticket, false),
        AccountMeta::new_readonly(*vault_ncn_ticket, false),
        AccountMeta::new(*vault_operator_delegation, false),
        AccountMeta::new_readonly(*ncn_vault_slasher_ticket, false),
        AccountMeta::new_readonly(*vault_ncn_slasher_ticket, false),
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
    vault_staker_withdrawal_ticket: &Pubkey,
    vault_staker_withdrawal_ticket_token_account: &Pubkey,
    staker: &Pubkey,
    staker_vrt_token_account: &Pubkey,
    base: &Pubkey,
    amount: u64,
    min_amount_out: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*vault_staker_withdrawal_ticket, false),
        AccountMeta::new(*vault_staker_withdrawal_ticket_token_account, false),
        AccountMeta::new(*staker, true),
        AccountMeta::new(*staker_vrt_token_account, false),
        AccountMeta::new_readonly(*base, true),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::EnqueueWithdrawal {
            amount,
            min_amount_out,
        }
        .try_to_vec()
        .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn burn_withdrawal_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    vault_token_account: &Pubkey,
    vrt_mint: &Pubkey,
    staker: &Pubkey,
    staker_token_account: &Pubkey,
    vault_staker_withdrawal_ticket: &Pubkey,
    vault_staker_withdrawal_ticket_token_account: &Pubkey,
    vault_fee_token_account: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*vault_token_account, false),
        AccountMeta::new(*vrt_mint, false),
        AccountMeta::new(*staker, false),
        AccountMeta::new(*staker_token_account, false),
        AccountMeta::new(*vault_staker_withdrawal_ticket, false),
        AccountMeta::new(*vault_staker_withdrawal_ticket_token_account, false),
        AccountMeta::new(*vault_fee_token_account, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::BurnWithdrawTicket.try_to_vec().unwrap(),
    }
}

pub fn update_vault_balance(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    vault_token_account: &Pubkey,
    vrt_mint: &Pubkey,
    vault_fee_token_account: &Pubkey,
    token_program: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new_readonly(*vault_token_account, false),
        AccountMeta::new(*vrt_mint, false),
        AccountMeta::new(*vault_fee_token_account, false),
        AccountMeta::new_readonly(*token_program, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::UpdateVaultBalance.try_to_vec().unwrap(),
    }
}
pub fn initialize_vault_update_state_tracker(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    vault_update_state_tracker: &Pubkey,
    payer: &Pubkey,
    withdrawal_allocation_method: WithdrawalAllocationMethod,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*vault_update_state_tracker, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::InitializeVaultUpdateStateTracker {
            withdrawal_allocation_method,
        }
        .try_to_vec()
        .unwrap(),
    }
}

pub fn close_vault_update_state_tracker(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    vault_update_state_tracker: &Pubkey,
    payer: &Pubkey,
    ncn_epoch: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new(*vault_update_state_tracker, false),
        AccountMeta::new(*payer, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::CloseVaultUpdateStateTracker { ncn_epoch }
            .try_to_vec()
            .unwrap(),
    }
}

pub fn warmup_vault_ncn_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    ncn: &Pubkey,
    vault_ncn_ticket: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new(*vault_ncn_ticket, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::WarmupVaultNcnTicket.try_to_vec().unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn warmup_vault_ncn_slasher_ticket(
    program_id: &Pubkey,
    config: &Pubkey,
    vault: &Pubkey,
    ncn: &Pubkey,
    slasher: &Pubkey,
    vault_slasher_ticket: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*slasher, false),
        AccountMeta::new(*vault_slasher_ticket, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: VaultInstruction::WarmupVaultNcnSlasherTicket
            .try_to_vec()
            .unwrap(),
    }
}
