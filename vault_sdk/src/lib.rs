use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankInstruction;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

#[rustfmt::skip]
#[derive(Debug, BorshSerialize, BorshDeserialize, ShankInstruction)]
pub enum VaultInstruction {
    /// Initializes global configuration
    #[account(0, writable, name = "config")]
    #[account(1, writable, signer, name = "admin")]
    #[account(2, name = "restaking_program")]
    #[account(3, name = "system_program")]
    InitializeConfig,

    /// Initializes the vault
    #[account(0, writable, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, writable, signer, name = "lrt_mint")]
    #[account(3, name = "token_mint")]
    #[account(4, writable, signer, name = "admin")]
    #[account(5, signer, name = "base")]
    #[account(6, name = "system_program")]
    #[account(7, name = "token_program")]
    InitializeVault {
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
    },

    /// Initializes a vault with an already-created LRT mint
    InitializeVaultWithMint,

    /// The vault_delegation_list account is too big for a single instruction, so it needs to be
    /// called until the discriminator is set
    #[account(0, name = "config")]
    #[account(1, name = "vault")]
    #[account(2, writable, name = "vault_delegation_list")]
    #[account(3, writable, signer, name = "payer")]
    #[account(4, name = "system_program")]
    InitializeVaultDelegationList,

    /// Vault adds support for the NCN
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, name = "ncn")]
    #[account(3, name = "ncn_vault_ticket")]
    #[account(4, writable, name = "vault_ncn_ticket")]
    #[account(5, signer, name = "admin")]
    #[account(6, writable, signer, name = "payer")]
    #[account(7, name = "system_program")]
    AddNcn,

    /// Vault removes support for an NCN
    #[account(0, name = "config")]
    #[account(1, name = "vault")]
    #[account(2, name = "ncn")]
    #[account(3, writable, name = "vault_ncn_ticket")]
    #[account(4, signer, name = "admin")]
    CooldownNcn,

    /// Vault adds support for an operator
    #[account(0, name = "config")]
    #[account(1, name = "vault")]
    #[account(2, writable, name = "operator")]
    #[account(3, name = "operator_vault_ticket")]
    #[account(4, writable, name = "vault_operator_ticket")]
    #[account(5, signer, name = "admin")]
    #[account(6, writable, signer, name = "payer")]
    #[account(7, name = "system_program")]
    AddOperator,

    /// Vault removes support for an operator
    #[account(0, name = "config")]
    #[account(1, name = "vault")]
    #[account(2, name = "operator")]
    #[account(3, writable, name = "vault_operator_ticket")]
    #[account(4, signer, name = "admin")]
    CooldownOperator,

    /// Mints LRT by depositing tokens into the vault
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, writable, name = "lrt_mint")]
    #[account(3, writable, signer, name = "depositor")]
    #[account(4, writable, name = "depositor_token_account")]
    #[account(5, writable, name = "vault_token_account")]
    #[account(6, writable, name = "depositor_lrt_token_account")]
    #[account(7, writable, name = "vault_fee_token_account")]
    #[account(8, name = "token_program")]
    #[account(9, signer, optional, name = "mint_signer", description = "Signer for minting")]
    MintTo {
        amount: u64
    },

    /// Burns LRT by withdrawing tokens from the vault
    Burn {
        amount: u64
    },

    /// Enqueues a withdrawal of LRT tokens
    /// Used when there aren't enough idle assets in the vault to cover a withdrawal
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, writable, name = "vault_delegation_list")]
    #[account(3, writable, name = "vault_staker_withdrawal_ticket")]
    #[account(4, writable, name = "vault_staker_withdrawal_ticket_token_account")]
    #[account(5, writable, name = "vault_fee_token_account")]
    #[account(6, writable, signer, name = "staker")]
    #[account(7, writable, name = "staker_lrt_token_account")]
    #[account(8, signer, name = "base")]
    #[account(9, name = "token_program")]
    #[account(10, name = "system_program")]
    #[account(11, signer, optional, name = "burn_signer", description = "Signer for burning")]
    EnqueueWithdrawal {
        amount: u64
    },

    /// Burns the withdraw ticket, returning funds to the staker. Withdraw tickets can be burned
    /// after one full epoch of being enqueued.
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, writable, name = "vault_delegation_list")]
    #[account(3, writable, name = "vault_token_account")]
    #[account(4, writable, name = "lrt_mint")]
    #[account(5, writable, signer, name = "staker")]
    #[account(6, writable, name = "staker_token_account")]
    #[account(7, writable, name = "staker_lrt_token_account")]
    #[account(8, writable, name = "vault_staker_withdrawal_ticket")]
    #[account(9, writable, name = "vault_staker_withdrawal_ticket_token_account")]
    #[account(10, name = "token_program")]
    #[account(11, name = "system_program")]
    BurnWithdrawTicket,

    /// Sets the max tokens that can be deposited into the LRT
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, signer, name = "admin")]
    SetDepositCapacity {
        amount: u64
    },

    /// Withdraws any non-backing tokens from the vault
    AdminWithdraw {
        amount: u64
    },

    /// Changes the signer for vault admin
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, signer, name = "old_admin")]
    #[account(3, signer, name = "new_admin")]
    SetAdmin,

    /// Changes the signer for vault delegation
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, signer, name = "admin")]
    #[account(3, name = "new_admin")]
    SetSecondaryAdmin(VaultAdminRole),

    /// Delegates a token amount to a specific node operator
    #[account(0, name = "config")]
    #[account(1, name = "vault")]
    #[account(2, name = "operator")]
    #[account(3, name = "vault_operator_ticket")]
    #[account(4, writable, name = "vault_delegation_list")]
    #[account(5, signer, name = "admin")]
    #[account(6, writable, signer, name = "payer")]
    #[account(7, name = "system_program")]
    AddDelegation {
        amount: u64,
    },

    #[account(0, name = "config")]
    #[account(1, name = "vault")]
    #[account(2, name = "operator")]
    #[account(3, writable, name = "vault_delegation_list")]
    #[account(4, signer, name = "admin")]
    CooldownDelegation {
        amount: u64,
    },

    /// Updates the vault
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, writable, name = "vault_delegation_list")]
    #[account(3, writable, name = "vault_token_account")]
    UpdateVault,

    /// Registers a slasher with the vault
    #[account(0, name = "config")]
    #[account(1, name = "vault")]
    #[account(2, name = "ncn")]
    #[account(3, name = "slasher")]
    #[account(4, name = "ncn_slasher_ticket")]
    #[account(5, writable, name = "vault_slasher_ticket")]
    #[account(6, signer, name = "admin")]
    #[account(7, signer, writable, name = "payer")]
    #[account(8, name = "system_program")]
    AddSlasher,

    /// Creates token metadata for the vault LRT
    #[account(0, writable, name = "metadata")]
    #[account(1, name = "update_authority_info")]
    #[account(2, name = "mint_info")]
    #[account(3, name = "mint_authority_info")]
    #[account(4, signer, name = "vault_admin")]
    #[account(5, name = "system_program")]
    #[account(6, name = "token_program")]
    CreateTokenMetadata {
        name: String,
        symbol: String,
        uri: String,
    },

    /// Updates token metadata for the vault LRT
    UpdateTokenMetadata {
        name: String,
        symbol: String,
        uri: String,
    },

    /// Initializes the account which keeps track of how much an operator has been slashed
    /// by a slasher for a given NCN and vault for a given epoch.
    #[account(0, name = "config")]
    #[account(1, name = "vault")]
    #[account(2, name = "ncn")]
    #[account(3, name = "slasher")]
    #[account(4, name = "operator")]
    #[account(5, name = "vault_ncn_slasher_ticket")]
    #[account(6, writable, name = "vault_ncn_slasher_operator_ticket")]
    #[account(7, writable, signer, name = "payer")]
    #[account(8, name = "system_program")]
    InitializeVaultNcnSlasherOperatorTicket,

    /// Slashes an amount of tokens from the vault
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, name = "ncn")]
    #[account(3, name = "operator")]
    #[account(4, name = "slasher")]
    #[account(5, name = "ncn_operator_ticket")]
    #[account(6, name = "operator_ncn_ticket")]
    #[account(7, name = "ncn_vault_ticket")]
    #[account(8, name = "operator_vault_ticket")]
    #[account(9, name = "vault_ncn_ticket")]
    #[account(10, name = "vault_operator_ticket")]
    #[account(11, name = "ncn_vault_slasher_ticket")]
    #[account(12, name = "vault_ncn_slasher_ticket")]
    #[account(13, writable, name = "vault_delegation_list")]
    #[account(14, writable, name = "vault_ncn_slasher_operator_ticket")]
    #[account(15, writable, name = "vault_token_account")]
    #[account(16, name = "slasher_token_account")]
    #[account(17, name = "token_program")]
    Slash {
        amount: u64
    },
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum VaultAdminRole {
    DelegationAdmin,
    OperatorAdmin,
    NcnAdmin,
    SlasherAdmin,
    CapacityAdmin,
    FeeWallet,
    MintBurnAdmin,
    WithdrawAdmin,
}

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
pub fn add_ncn(
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
        data: VaultInstruction::AddNcn.try_to_vec().unwrap(),
    }
}

pub fn remove_ncn(
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
        data: VaultInstruction::CooldownNcn.try_to_vec().unwrap(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn add_operator(
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
        data: VaultInstruction::AddOperator.try_to_vec().unwrap(),
    }
}

pub fn remove_operator(
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
        data: VaultInstruction::CooldownOperator.try_to_vec().unwrap(),
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
    vault: &Pubkey,
    admin: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
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

pub fn remove_delegation(
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
pub fn add_slasher(
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
        data: VaultInstruction::AddSlasher.try_to_vec().unwrap(),
    }
}

pub fn create_token_metadata(
    program_id: &Pubkey,
    metadata: &Pubkey,
    vault: &Pubkey,
    vault_admin: &Pubkey,
    name: String,
    symbol: String,
    uri: String,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*metadata, false),
        AccountMeta::new(*vault, false),
        AccountMeta::new_readonly(*vault_admin, true),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
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
