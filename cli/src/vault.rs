use std::path::PathBuf;

use clap::{command, Subcommand};
use solana_program::pubkey::Pubkey;

#[derive(Subcommand)]
pub enum VaultCommands {
    Config {
        #[command(subcommand)]
        action: ConfigActions,
    },
    Vault {
        #[command(subcommand)]
        action: VaultActions,
    },
}

#[derive(Subcommand)]
pub enum ConfigActions {
    /// Creates global config (can only be done once)
    Initialize {
        /// The program fee in basis points
        program_fee_bps: u16,
        /// The program fee wallet pubkey
        program_fee_wallet: Pubkey,
    },
    /// Fetches global config
    Get,
    /// Set the config admin
    SetAdmin {
        /// The new admin's pubkey
        new_admin: Pubkey,
    },
    /// Set the program fee
    SetProgramFee {
        /// The program fee
        new_fee_bps: u16,
    },
    /// Set the program fee wallet
    SetProgramFeeWallet {
        /// The program fee wallet
        program_fee_wallet: Pubkey,
    },
}

/// Vault commands
#[derive(Subcommand)]
pub enum VaultActions {
    /// Creates a new vault
    Initialize {
        /// The token which is allowed to be deposited into the vault
        token_mint: String,
        /// The deposit fee in bips
        deposit_fee_bps: u16,
        /// The withdrawal fee in bips
        withdrawal_fee_bps: u16,
        /// The reward fee in bips
        reward_fee_bps: u16,
        /// The decimals of the token
        decimals: u8,
        /// The amount of tokens to initialize the vault with ( in the smallest unit )
        initialize_token_amount: u64,
        /// The file path of VRT mint address
        vrt_mint_address_file_path: Option<PathBuf>,
    },
    /// Creates token metadata for the vault's LRT token
    CreateTokenMetadata {
        /// The vault pubkey
        vault: String,
        /// The name of the token
        name: String,
        /// The symbol of the token
        symbol: String,
        /// The URI for the token metadata
        uri: String,
    },
    UpdateTokenMetadata {
        /// The vault pubkey
        vault: String,
        /// The name of the token
        name: String,
        /// The symbol of the token
        symbol: String,
        /// The URI for the token metadata
        uri: String,
    },
    /// Starts the vault update cycle
    InitializeVaultUpdateStateTracker {
        /// Vault account
        vault: String,
    },
    /// Cranks the vault update state tracker, needs to be run per operator
    CrankVaultUpdateStateTracker {
        /// Vault account
        vault: String,
        /// Operator account
        operator: String,
    },
    /// Ends the vault update cycle
    CloseVaultUpdateStateTracker {
        /// Vault account
        vault: String,
        /// Optional NCN epoch to close
        ncn_epoch: Option<u64>,
    },
    /// Mints VRT tokens
    MintVRT {
        /// Vault account
        vault: String,
        /// Amount to deposit
        amount_in: u64,
        /// Minimum amount of VRT to mint
        min_amount_out: u64,
    },
    /// Sets up the delegations for an operator
    InitializeOperatorDelegation {
        /// Vault account
        vault: String,
        /// Operator account
        operator: String,
    },
    /// Delegates tokens to an operator
    DelegateToOperator {
        /// Vault account
        vault: String,
        /// Operator account
        operator: String,
        /// Amount to delegate
        amount: u64,
    },
    /// Cooldown delegation for an operator
    CooldownOperatorDelegation {
        /// Vault account
        vault: String,
        /// Operator account
        operator: String,
        /// Amount to cooldown
        amount: u64,
    },
    /// Initialize Vault NCN Ticket
    InitializeVaultNcnTicket {
        /// Vault account
        vault: String,
        /// NCN account
        ncn: String,
    },
    /// Warmup Vault NCN Ticket
    WarmupVaultNcnTicket {
        /// Vault account
        vault: String,
        /// NCN account
        ncn: String,
    },
    /// Cooldown Vault NCN Ticket
    CooldownVaultNcnTicket {
        /// Vault account
        vault: String,
        /// NCN account
        ncn: String,
    },
    /// Starts the withdrawal process
    EnqueueWithdrawal {
        /// Vault account
        vault: String,
        /// Amount to withdraw
        amount: u64,
    },
    /// Burns the withdrawal ticket, ending the withdrawal process
    BurnWithdrawalTicket {
        /// Vault account
        vault: String,
    },
    /// Gets the update state tracker for a vault
    GetVaultUpdateStateTracker {
        /// Vault account
        vault: String,
    },
    /// Gets the operator delegations for a vault
    GetOperatorDelegations {
        /// Vault account
        vault: String,
    },
    /// Gets the operator delegation for a vault
    GetOperatorDelegation {
        /// Vault account
        vault: String,
        /// Operator account
        operator: String,
    },
    GetWithdrawalTicket {
        /// Vault account
        vault: String,
        /// Staker account
        staker: Option<String>,
    },
    /// Gets a vault
    Get {
        /// The vault pubkey
        pubkey: String,
    },
    /// List all vaults
    List,
    /// Set Admin
    SetAdmin {
        /// The Vault pubkey
        vault: Pubkey,

        /// Path to the old admin keypair file
        #[arg(long)]
        old_admin_keypair: String,
    },
    /// Sets the deposit capacity in the vault
    SetCapacity {
        /// The vault pubkey
        vault: String,
        /// The new capacity
        amount: u64,
    },
    /// Sets the fees in the vault
    SetFees {
        /// The vault pubkey
        vault: Pubkey,

        /// The deposit fee BPS
        #[arg(long)]
        deposit_fee_bps: Option<u16>,

        /// The withdrawal fee BPS
        #[arg(long)]
        withdrawal_fee_bps: Option<u16>,

        /// The reward fee BPS
        #[arg(long)]
        reward_fee_bps: Option<u16>,
    },
    /// Sets the paused the vault
    SetIsPaused {
        /// The vault pubkey
        vault: Pubkey,

        /// Set pause
        #[arg(long)]
        set_pause: bool,
    },
    /// Set Secondary Admin
    SetSecondaryAdmin {
        /// The vault pubkey
        vault: Pubkey,

        /// The new admin pubkey
        new_admin: Pubkey,

        /// Set delegation_admin
        #[arg(long)]
        set_delegation_admin: bool,

        /// Set operator_admin
        #[arg(long)]
        set_operator_admin: bool,

        /// Set ncn_admin
        #[arg(long)]
        set_ncn_admin: bool,

        /// Set slasher_admin
        #[arg(long)]
        set_slasher_admin: bool,

        /// Set capacity_admin
        #[arg(long)]
        set_capacity_admin: bool,

        /// Set fee_wallet
        #[arg(long)]
        set_fee_wallet: bool,

        /// Set mint_burn_admin
        #[arg(long)]
        set_mint_burn_admin: bool,

        /// Set delegate_asset_admin
        #[arg(long)]
        set_delegate_asset_admin: bool,

        /// Set fee_admin
        #[arg(long)]
        set_fee_admin: bool,

        /// Set metadata_admin
        #[arg(long)]
        set_metadata_admin: bool,
    },
    /// Update Vault Balance
    UpdateVaultBalance {
        /// The vault pubkey
        vault: Pubkey,
    },
    /// Delegate a token account
    DelegateTokenAccount {
        /// The vault pubkey
        vault: String,
        /// The delegate account
        delegate: String,
        /// The token mint
        token_mint: String,
        /// The token account
        token_account: String,
    },
    /// Transfer a token account
    DelegatedTokenTransfer {
        /// The token account
        token_account: String,
        /// The recipient pubkey
        recipient_pubkey: String,
        /// The amount to transfer
        amount: u64,
    },
}
