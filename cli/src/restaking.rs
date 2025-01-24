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
    Initialize,
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
    /// Get NCN
    Get { pubkey: String },
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
    /// Cooldown Operator Vault Ticket
    CooldownOperatorVaultTicket { operator: String, vault: String },
    /// Operator Warmup NCN
    OperatorWarmupNcn { operator: String, ncn: String },
    /// Operator Cooldown NCN
    OperatorCooldownNcn { operator: String, ncn: String },
    /// Get operator
    Get { pubkey: String },
    /// List all operators
    List,
}
