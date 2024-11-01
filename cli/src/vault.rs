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
    /// Set the config program fee wallet
    SetProgramFeeWallet,
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
        /// NCN epoch
        ncn_epoch: u64,
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
    /// Sets the deposit capacity in the vault
    SetCapacity {
        /// The vault pubkey
        vault: String,
        /// The new capacity
        amount: u64,
    },
}
