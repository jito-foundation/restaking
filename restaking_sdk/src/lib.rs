use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankInstruction;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

#[derive(Debug, BorshSerialize, BorshDeserialize, ShankInstruction)]
pub enum RestakingInstruction {
    /// Initializes the global configuration
    #[account(0, writable, name = "config")]
    #[account(1, writable, signer, name = "admin")]
    #[account(2, name = "vault_program")]
    #[account(3, name = "system_program")]
    InitializeConfig,

    /// Initializes the AVS
    #[account(0, writable, name = "config")]
    #[account(1, writable, name = "avs")]
    #[account(2, writable, signer, name = "admin")]
    #[account(3, signer, name = "base")]
    #[account(4, name = "system_program")]
    InitializeAvs,

    /// AVS adds support for receiving delegation from a vault
    #[account(0, name = "config")]
    #[account(1, writable, name = "avs")]
    #[account(2, name = "vault")]
    #[account(3, writable, name = "avs_vault_ticket")]
    #[account(4, signer, name = "admin")]
    #[account(5, writable, signer, name = "payer")]
    #[account(6, name = "system_program")]
    AvsAddVault,

    /// AVS removes support for receiving delegation from a vault
    #[account(0, name = "config")]
    #[account(1, name = "avs")]
    #[account(2, name = "vault")]
    #[account(3, writable, name = "avs_vault_ticket")]
    #[account(4, signer, name = "admin")]
    AvsRemoveVault,

    /// After the operator has signaled they are ready to join the network,
    /// the AVS admin can add the operator to the AVS
    #[account(0, name = "config")]
    #[account(1, writable, name = "avs")]
    #[account(2, name = "operator")]
    #[account(3, writable, name = "avs_operator_ticket")]
    #[account(4, name = "operator_avs_ticket")]
    #[account(5, signer, name = "admin")]
    #[account(6, writable, signer, name = "payer")]
    #[account(7, name = "system_program")]
    AvsAddOperator,

    #[account(0, name = "config")]
    #[account(1, name = "avs")]
    #[account(2, name = "operator")]
    #[account(3, writable, name = "avs_operator_ticket")]
    #[account(4, signer, name = "admin")]
    AvsRemoveOperator,

    /// The AVS adds support for a vault slasher
    ///
    /// # Arguments
    /// * `u64` - The maximum amount that can be slashed from the vault per epoch
    #[account(0, name = "config")]
    #[account(1, writable, name = "avs")]
    #[account(2, name = "vault")]
    #[account(3, name = "slasher")]
    #[account(4, name = "avs_vault_ticket")]
    #[account(5, writable, name = "avs_slasher_ticket")]
    #[account(6, signer, name = "admin")]
    #[account(7, writable, signer, name = "payer")]
    #[account(8, name = "system_program")]
    AvsAddVaultSlasher(u64),

    /// AVS removes support for a slasher
    #[account(0, name = "config")]
    #[account(1, name = "avs")]
    #[account(2, name = "vault")]
    #[account(3, name = "slasher")]
    #[account(4, writable, name = "avs_slasher_ticket")]
    #[account(5, signer, name = "admin")]
    AvsRemoveVaultSlasher,

    #[account(0, writable, name = "avs")]
    #[account(1, signer, name = "old_admin")]
    #[account(2, signer, name = "new_admin")]
    AvsSetAdmin,

    #[account(0, writable, name = "avs")]
    #[account(1, signer, name = "admin")]
    #[account(2, name = "new_admin")]
    AvsSetSecondaryAdmin(AvsAdminRole),

    /// Initializes a operator
    #[account(0, writable, name = "config")]
    #[account(1, writable, name = "operator")]
    #[account(2, writable, signer, name = "admin")]
    #[account(3, signer, name = "base")]
    #[account(4, name = "system_program")]
    InitializeOperator,

    /// Sets the admin for a node operator
    #[account(0, writable, name = "node_operator")]
    #[account(1, signer, name = "old_admin")]
    #[account(2, signer, name = "new_admin")]
    OperatorSetAdmin,

    /// Sets the voter for a node operator
    #[account(0, writable, name = "node_operator")]
    #[account(1, signer, name = "admin")]
    #[account(2, name = "voter")]
    OperatorSetVoter,

    /// Operator adds support for receiving delegation from a vault
    #[account(0, name = "config")]
    #[account(1, writable, name = "operator")]
    #[account(2, name = "vault")]
    #[account(3, writable, name = "operator_vault_ticket")]
    #[account(4, signer, name = "admin")]
    #[account(5, writable, signer, name = "payer")]
    #[account(6, name = "system_program")]
    OperatorAddVault,

    /// Node operator removes support for receiving delegation from a vault
    #[account(0, name = "config")]
    #[account(1, name = "operator")]
    #[account(2, name = "vault")]
    #[account(3, writable, name = "operator_vault_ticket")]
    #[account(4, signer, name = "admin")]
    OperatorRemoveVault,

    /// Node operator adds support for running an AVS
    #[account(0, name = "config")]
    #[account(1, writable, name = "operator")]
    #[account(2, name = "avs")]
    #[account(3, writable, name = "operator_avs_ticket")]
    #[account(4, signer, name = "admin")]
    #[account(5, writable, signer, name = "payer")]
    #[account(6, name = "system_program")]
    OperatorAddAvs,

    /// Node operator removes support for running an AVS
    #[account(0, name = "config")]
    #[account(1, name = "operator")]
    #[account(2, name = "avs")]
    #[account(3, writable, name = "operator_avs_ticket")]
    #[account(4, signer, name = "admin")]
    OperatorRemoveAvs,

    #[account(0, name = "avs")]
    #[account(1, writable, name = "avs_token_account")]
    #[account(2, writable, name = "receiver_token_account")]
    #[account(3, signer, name = "admin")]
    #[account(4, name = "token_program")]
    AvsWithdrawalAsset { token_mint: Pubkey, amount: u64 },

    #[account(0, name = "operator")]
    #[account(1, signer, name = "admin")]
    #[account(2, writable, name = "operator_token_account")]
    #[account(3, writable, name = "receiver_token_account")]
    #[account(4, name = "token_program")]
    OperatorWithdrawalAsset { token_mint: Pubkey, amount: u64 },
}

#[derive(Debug, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub enum AvsAdminRole {
    Operator,
    Vault,
    Slasher,
    Withdraw,
}

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

pub fn initialize_avs(
    program_id: &Pubkey,
    config: &Pubkey,
    avs: &Pubkey,
    admin: &Pubkey,
    base: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*config, false),
        AccountMeta::new(*avs, false),
        AccountMeta::new(*admin, true),
        AccountMeta::new_readonly(*base, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::InitializeAvs.try_to_vec().unwrap(),
    }
}

pub fn avs_add_vault(
    program_id: &Pubkey,
    config: &Pubkey,
    avs: &Pubkey,
    vault: &Pubkey,
    avs_vault_ticket: &Pubkey,
    admin: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*avs, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new(*avs_vault_ticket, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::AvsAddVault.try_to_vec().unwrap(),
    }
}

pub fn avs_remove_vault(
    program_id: &Pubkey,
    config: &Pubkey,
    avs: &Pubkey,
    vault: &Pubkey,
    avs_vault_ticket: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*avs, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new(*avs_vault_ticket, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::AvsRemoveVault.try_to_vec().unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn avs_add_operator(
    program_id: &Pubkey,
    config: &Pubkey,
    avs: &Pubkey,
    operator: &Pubkey,
    avs_operator_ticket: &Pubkey,
    operator_avs_ticket: &Pubkey,
    admin: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*avs, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new(*avs_operator_ticket, false),
        AccountMeta::new_readonly(*operator_avs_ticket, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::AvsAddOperator.try_to_vec().unwrap(),
    }
}

pub fn avs_remove_operator(
    program_id: &Pubkey,
    config: &Pubkey,
    avs: &Pubkey,
    operator: &Pubkey,
    avs_operator_ticket: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*avs, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new(*avs_operator_ticket, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::AvsRemoveOperator
            .try_to_vec()
            .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn avs_add_vault_slasher(
    program_id: &Pubkey,
    config: &Pubkey,
    avs: &Pubkey,
    vault: &Pubkey,
    slasher: &Pubkey,
    avs_vault_ticket: &Pubkey,
    avs_slasher_ticket: &Pubkey,
    admin: &Pubkey,
    payer: &Pubkey,

    max_slash_amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*avs, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new_readonly(*slasher, false),
        AccountMeta::new_readonly(*avs_vault_ticket, false),
        AccountMeta::new(*avs_slasher_ticket, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::AvsAddVaultSlasher(max_slash_amount)
            .try_to_vec()
            .unwrap(),
    }
}

pub fn avs_remove_vault_slasher(
    program_id: &Pubkey,
    config: &Pubkey,
    avs: &Pubkey,
    vault: &Pubkey,
    slasher: &Pubkey,
    avs_slasher_ticket: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*avs, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new_readonly(*slasher, false),
        AccountMeta::new(*avs_slasher_ticket, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::AvsRemoveVaultSlasher
            .try_to_vec()
            .unwrap(),
    }
}

pub fn avs_set_admin(
    program_id: &Pubkey,
    avs: &Pubkey,
    old_admin: &Pubkey,
    new_admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*avs, false),
        AccountMeta::new_readonly(*old_admin, true),
        AccountMeta::new_readonly(*new_admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::AvsSetAdmin.try_to_vec().unwrap(),
    }
}

pub fn avs_set_secondary_admin(
    program_id: &Pubkey,
    avs: &Pubkey,
    admin: &Pubkey,
    new_admin: &Pubkey,
    role: AvsAdminRole,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*avs, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new_readonly(*new_admin, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::AvsSetSecondaryAdmin(role)
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
    node_operator: &Pubkey,
    old_admin: &Pubkey,
    new_admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*node_operator, false),
        AccountMeta::new_readonly(*old_admin, true),
        AccountMeta::new_readonly(*new_admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::OperatorSetAdmin.try_to_vec().unwrap(),
    }
}

pub fn operator_set_voter(
    program_id: &Pubkey,
    node_operator: &Pubkey,
    admin: &Pubkey,
    voter: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*node_operator, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new_readonly(*voter, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::OperatorSetVoter.try_to_vec().unwrap(),
    }
}

pub fn operator_add_vault(
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
        data: RestakingInstruction::OperatorAddVault.try_to_vec().unwrap(),
    }
}

pub fn operator_remove_vault(
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
        data: RestakingInstruction::OperatorRemoveVault
            .try_to_vec()
            .unwrap(),
    }
}

pub fn operator_add_avs(
    program_id: &Pubkey,
    config: &Pubkey,
    operator: &Pubkey,
    avs: &Pubkey,
    operator_avs_ticket: &Pubkey,
    admin: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*operator, false),
        AccountMeta::new_readonly(*avs, false),
        AccountMeta::new(*operator_avs_ticket, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::OperatorAddAvs.try_to_vec().unwrap(),
    }
}

pub fn operator_remove_avs(
    program_id: &Pubkey,
    config: &Pubkey,
    operator: &Pubkey,
    avs: &Pubkey,
    operator_avs_ticket: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new_readonly(*avs, false),
        AccountMeta::new(*operator_avs_ticket, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::OperatorRemoveAvs
            .try_to_vec()
            .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn avs_withdrawal_asset(
    program_id: &Pubkey,
    avs: &Pubkey,
    avs_token_account: &Pubkey,
    receiver_token_account: &Pubkey,
    admin: &Pubkey,
    token_program: &Pubkey,
    token_mint: Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*avs, false),
        AccountMeta::new(*avs_token_account, false),
        AccountMeta::new(*receiver_token_account, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new_readonly(*token_program, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::AvsWithdrawalAsset { token_mint, amount }
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
    token_program: &Pubkey,
    token_mint: Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new(*operator_token_account, false),
        AccountMeta::new(*receiver_token_account, false),
        AccountMeta::new_readonly(*token_program, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::OperatorWithdrawalAsset { token_mint, amount }
            .try_to_vec()
            .unwrap(),
    }
}
