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

    /// Initializes the NCN
    #[account(0, writable, name = "config")]
    #[account(1, writable, name = "ncn")]
    #[account(2, writable, signer, name = "admin")]
    #[account(3, signer, name = "base")]
    #[account(4, name = "system_program")]
    InitializeNcn,

    /// NCN adds support for receiving delegation from a vault
    #[account(0, name = "config")]
    #[account(1, writable, name = "ncn")]
    #[account(2, name = "vault")]
    #[account(3, writable, name = "ncn_vault_ticket")]
    #[account(4, signer, name = "admin")]
    #[account(5, writable, signer, name = "payer")]
    #[account(6, name = "system_program")]
    NcnAddVault,

    /// NCN removes support for receiving delegation from a vault
    #[account(0, name = "config")]
    #[account(1, name = "ncn")]
    #[account(2, name = "vault")]
    #[account(3, writable, name = "ncn_vault_ticket")]
    #[account(4, signer, name = "admin")]
    NcnRemoveVault,

    /// After the operator has signaled they are ready to join the network,
    /// the NCN admin can add the operator to the NCN
    #[account(0, name = "config")]
    #[account(1, writable, name = "ncn")]
    #[account(2, name = "operator")]
    #[account(3, writable, name = "ncn_operator_ticket")]
    #[account(4, name = "operator_ncn_ticket")]
    #[account(5, signer, name = "admin")]
    #[account(6, writable, signer, name = "payer")]
    #[account(7, name = "system_program")]
    NcnAddOperator,

    #[account(0, name = "config")]
    #[account(1, name = "ncn")]
    #[account(2, name = "operator")]
    #[account(3, writable, name = "ncn_operator_ticket")]
    #[account(4, signer, name = "admin")]
    NcnRemoveOperator,

    /// The NCN adds support for a vault slasher
    ///
    /// # Arguments
    /// * `u64` - The maximum amount that can be slashed from the vault per epoch
    #[account(0, name = "config")]
    #[account(1, writable, name = "ncn")]
    #[account(2, name = "vault")]
    #[account(3, name = "slasher")]
    #[account(4, name = "ncn_vault_ticket")]
    #[account(5, writable, name = "ncn_slasher_ticket")]
    #[account(6, signer, name = "admin")]
    #[account(7, writable, signer, name = "payer")]
    #[account(8, name = "system_program")]
    NcnAddVaultSlasher(u64),

    /// NCN removes support for a slasher
    #[account(0, name = "config")]
    #[account(1, name = "ncn")]
    #[account(2, name = "vault")]
    #[account(3, name = "slasher")]
    #[account(4, writable, name = "ncn_slasher_ticket")]
    #[account(5, signer, name = "admin")]
    NcnRemoveVaultSlasher,

    #[account(0, writable, name = "ncn")]
    #[account(1, signer, name = "old_admin")]
    #[account(2, signer, name = "new_admin")]
    NcnSetAdmin,

    #[account(0, writable, name = "ncn")]
    #[account(1, signer, name = "admin")]
    #[account(2, name = "new_admin")]
    NcnSetSecondaryAdmin(NcnAdminRole),

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

    /// Node operator adds support for running an NCN
    #[account(0, name = "config")]
    #[account(1, writable, name = "operator")]
    #[account(2, name = "ncn")]
    #[account(3, writable, name = "operator_ncn_ticket")]
    #[account(4, signer, name = "admin")]
    #[account(5, writable, signer, name = "payer")]
    #[account(6, name = "system_program")]
    OperatorAddNcn,

    /// Node operator removes support for running an NCN
    #[account(0, name = "config")]
    #[account(1, name = "operator")]
    #[account(2, name = "ncn")]
    #[account(3, writable, name = "operator_ncn_ticket")]
    #[account(4, signer, name = "admin")]
    OperatorRemoveNcn,

    #[account(0, name = "ncn")]
    #[account(1, writable, name = "ncn_token_account")]
    #[account(2, writable, name = "receiver_token_account")]
    #[account(3, signer, name = "admin")]
    #[account(4, name = "token_program")]
    NcnWithdrawalAsset { token_mint: Pubkey, amount: u64 },

    #[account(0, name = "operator")]
    #[account(1, signer, name = "admin")]
    #[account(2, writable, name = "operator_token_account")]
    #[account(3, writable, name = "receiver_token_account")]
    #[account(4, name = "token_program")]
    OperatorWithdrawalAsset { token_mint: Pubkey, amount: u64 },
}

#[derive(Debug, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub enum NcnAdminRole {
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

pub fn ncn_add_vault(
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
        data: RestakingInstruction::NcnAddVault.try_to_vec().unwrap(),
    }
}

pub fn ncn_remove_vault(
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
        data: RestakingInstruction::NcnRemoveVault.try_to_vec().unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ncn_add_operator(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    operator: &Pubkey,
    ncn_operator_ticket: &Pubkey,
    operator_ncn_ticket: &Pubkey,
    admin: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*ncn, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new(*ncn_operator_ticket, false),
        AccountMeta::new_readonly(*operator_ncn_ticket, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::NcnAddOperator.try_to_vec().unwrap(),
    }
}

pub fn ncn_remove_operator(
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
        data: RestakingInstruction::NcnRemoveOperator
            .try_to_vec()
            .unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ncn_add_vault_slasher(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    vault: &Pubkey,
    slasher: &Pubkey,
    ncn_vault_ticket: &Pubkey,
    ncn_slasher_ticket: &Pubkey,
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
        AccountMeta::new(*ncn_slasher_ticket, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::NcnAddVaultSlasher(max_slash_amount)
            .try_to_vec()
            .unwrap(),
    }
}

pub fn ncn_remove_vault_slasher(
    program_id: &Pubkey,
    config: &Pubkey,
    ncn: &Pubkey,
    vault: &Pubkey,
    slasher: &Pubkey,
    ncn_slasher_ticket: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new_readonly(*vault, false),
        AccountMeta::new_readonly(*slasher, false),
        AccountMeta::new(*ncn_slasher_ticket, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::NcnRemoveVaultSlasher
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

pub fn operator_add_ncn(
    program_id: &Pubkey,
    config: &Pubkey,
    operator: &Pubkey,
    ncn: &Pubkey,
    operator_ncn_ticket: &Pubkey,
    admin: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new(*operator, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new(*operator_ncn_ticket, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::OperatorAddNcn.try_to_vec().unwrap(),
    }
}

pub fn operator_remove_ncn(
    program_id: &Pubkey,
    config: &Pubkey,
    operator: &Pubkey,
    ncn: &Pubkey,
    operator_ncn_ticket: &Pubkey,
    admin: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*config, false),
        AccountMeta::new_readonly(*operator, false),
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new(*operator_ncn_ticket, false),
        AccountMeta::new_readonly(*admin, true),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::OperatorRemoveNcn
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
    token_program: &Pubkey,
    token_mint: Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*ncn, false),
        AccountMeta::new(*ncn_token_account, false),
        AccountMeta::new(*receiver_token_account, false),
        AccountMeta::new_readonly(*admin, true),
        AccountMeta::new_readonly(*token_program, false),
    ];
    Instruction {
        program_id: *program_id,
        accounts,
        data: RestakingInstruction::NcnWithdrawalAsset { token_mint, amount }
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
