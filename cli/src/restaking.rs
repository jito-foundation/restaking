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
}
