use std::path::PathBuf;

use clap::{command, Subcommand};
use solana_program::pubkey::Pubkey;

/// The CLI handler for the restaking program
#[derive(Subcommand)]
pub enum RestakingCommands {
    /// Initialize, get, and set the config struct
    Config {
        #[command(subcommand)]
        action: ConfigActions,
    },
    Ncn {
        #[command(subcommand)]
        action: NcnActions,
    },
    Operator {
        #[command(subcommand)]
        action: OperatorActions,
    },
}

/// The actions that can be performed on the restaking config
#[derive(Subcommand)]
pub enum ConfigActions {
    /// Initialize the config
    Initialize,
    /// Get the config
    Get,
    /// Set the config admin
    SetAdmin {
        /// The new admin's pubkey
        new_admin: Pubkey,
    },
}

#[derive(Subcommand)]
pub enum NcnActions {
    /// Initialize NCN
    Initialize {
        #[arg(long)]
        path_to_base_keypair: Option<String>,
    },
    /// Initialize NCN Operator State
    InitializeNcnOperatorState { ncn: String, operator: String },
    /// Warmup NCN Operator State
    NcnWarmupOperator { ncn: String, operator: String },
    /// NCN Cooldown Operator State
    NcnCooldownOperator { ncn: String, operator: String },
    /// Initialize NCN Vault Ticket
    InitializeNcnVaultTicket { ncn: String, vault: String },
    /// Warmup NCN Vault Ticket
    WarmupNcnVaultTicket { ncn: String, vault: String },
    /// Cooldown NCN Vault Ticket
    CooldownNcnVaultTicket { ncn: String, vault: String },
    /// NCN Delegate Token Account
    NcnDelegateTokenAccount {
        ncn: String,
        delegate: String,
        token_mint: String,
        #[arg(long)]
        should_create_token_account: bool,
    },
    /// Get NCN
    Get { pubkey: String },
    /// List all NCNs
    List,
    /// List All Ncn Operator State for a NCN
    ListNcnOperatorState { ncn: Pubkey },
    /// List All Ncn Vault Ticket for a NCN
    ListNcnVaultTicket { ncn: Pubkey },
    /// Set NCN Admin
    NcnSetAdmin {
        /// The NCN pubkey
        ncn: String,

        /// Path to the old admin keypair file
        #[arg(long)]
        old_admin_keypair: PathBuf,

        /// Path to the new admin keypair file
        #[arg(long)]
        new_admin_keypair: PathBuf,
    },
    /// Set NCN Secondary Admin
    NcnSetSecondaryAdmin {
        /// The NCN pubkey
        ncn: String,

        /// The new admin pubkey
        new_admin: String,

        /// Set operator_admin
        #[arg(long)]
        set_operator_admin: bool,

        /// Set vault_admin
        #[arg(long)]
        set_vault_admin: bool,

        /// Set slasher_admin
        #[arg(long)]
        set_slasher_admin: bool,

        /// Set delegate_admin
        #[arg(long)]
        set_delegate_admin: bool,

        ///Set metadata_admin
        #[arg(long)]
        set_metadata_admin: bool,

        ///Set weight_table_admin
        #[arg(long)]
        set_weight_table_admin: bool,

        ///Set ncn_program_admin
        #[arg(long)]
        set_ncn_program_admin: bool,
    },
}

#[derive(Subcommand)]
pub enum OperatorActions {
    /// Initialize Operator
    Initialize { operator_fee_bps: u16 },
    /// Initialize Operator Vault Ticket
    InitializeOperatorVaultTicket { operator: String, vault: String },
    /// Warmup Operator Vault Ticket
    WarmupOperatorVaultTicket { operator: String, vault: String },
    /// Cooldown Operator Vault Ticket
    CooldownOperatorVaultTicket { operator: String, vault: String },
    /// Operator Warmup NCN
    OperatorWarmupNcn { operator: String, ncn: String },
    /// Operator Cooldown NCN
    OperatorCooldownNcn { operator: String, ncn: String },
    /// Operator Set Admin
    OperatorSetAdmin {
        /// The Operator pubkey
        operator: String,

        /// Path to the old admin keypair file
        #[arg(long)]
        old_admin_keypair: PathBuf,

        /// Path to the new admin keypair file
        #[arg(long)]
        new_admin_keypair: PathBuf,
    },
    /// Operator Set Secondary Admin
    OperatorSetSecondaryAdmin {
        operator: String,
        new_admin: String,
        #[arg(long)]
        set_ncn_admin: bool,
        #[arg(long)]
        set_vault_admin: bool,
        #[arg(long)]
        set_voter_admin: bool,
        #[arg(long)]
        set_delegate_admin: bool,
        #[arg(long)]
        set_metadata_admin: bool,
    },
    /// Sets the operator fee
    OperatorSetFees {
        operator: String,
        operator_fee_bps: u16,
    },
    /// Operator Delegate Token Account
    OperatorDelegateTokenAccount {
        operator: String,
        delegate: String,
        token_mint: String,
        #[arg(long)]
        should_create_token_account: bool,
    },
    /// Get operator
    Get { pubkey: String },
    /// List all operators
    List,
    /// List Operator Vault Ticket for an Operator
    ListOperatorVaultTicket { operator: Pubkey },
    /// List All Ncn Operator State for a Operator
    ListNcnOperatorState { operator: Pubkey },
}
