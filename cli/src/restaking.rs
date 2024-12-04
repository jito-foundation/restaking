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
    InitializeNcnOperatorState { ncn: Pubkey, operator: Pubkey },
    /// NCN warmpup Operator
    NcnWarmupOperator { ncn: Pubkey, operator: Pubkey },
    /// NCN cooldown Operator
    NcnCooldownOperator { ncn: Pubkey, operator: Pubkey },
    /// Operator warmup NCN
    OperatorWarmupNcn { ncn: Pubkey, operator: Pubkey },
    /// Operator cooldown NCN
    OperatorCooldownNcn { ncn: Pubkey, operator: Pubkey },
    /// Initialize NCN Vault Ticket
    InitializeNcnVaultTicket { ncn: Pubkey, vault: Pubkey },
    /// Warmup NCN Vault Ticket
    WarmupNcnVaultTicket { ncn: Pubkey, vault: Pubkey },
    /// Cooldown NCN Vault Ticket
    CooldownNcnVaultTicket { ncn: Pubkey, vault: Pubkey },
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
    /// Get operator
    Get { pubkey: String },
    /// List all operators
    List,
}
