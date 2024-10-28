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
        /// The old admin's pubkey
        old_admin: Pubkey,
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
    /// List all NCNs
    List,
}

#[derive(Subcommand)]
pub enum OperatorActions {
    /// Initialize Operator
    Initialize,
    /// Get operator
    Get { pubkey: String },
    /// List all operators
    List,
}
