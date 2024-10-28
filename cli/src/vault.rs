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
}

/// Vault commands
#[derive(Subcommand)]
pub enum VaultActions {
    /// Initializes the vault
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
    /// Gets a vault
    Get {
        /// The vault pubkey
        pubkey: String,
    },
    /// List all vaults
    List,
}
