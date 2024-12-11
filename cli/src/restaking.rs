use clap::{command, Subcommand};
use jito_restaking_client::types::{NcnAdminRole, OperatorAdminRole};
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
    Initialize,
    /// Get NCN
    Get { pubkey: String },
    /// Set Secondary Admin
    SetSecondaryAdmin {
        /// The NCN pubkey
        ncn: Pubkey,
        /// New admin pubkey
        new_admin: Pubkey,
        /// NCN Admin role
        role: NcnAdminRole,
    },
    /// List all NCNs
    List,
}

#[derive(Subcommand)]
pub enum OperatorActions {
    /// Initialize Operator
    Initialize { operator_fee_bps: u16 },
    /// Initialize Operator Vault Ticket
    InitializeOperatorVaultTicket { operator: String, vault: String },
    /// Warmup Operator Vault Ticket
    WarmupOperatorVaultTicket { operator: String, vault: String },
    /// Set Secondary Admin
    SetSecondaryAdmin {
        /// The operator pubkey
        operator: Pubkey,
        /// New admin pubkey
        new_admin: Pubkey,
        /// Operator Admin role
        role: OperatorAdminRole,
    },
    /// Get operator
    Get { pubkey: String },
    /// List all operators
    List,
}
