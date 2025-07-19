use std::{path::PathBuf, str::FromStr};

use solana_cli_config::Config;
use solana_sdk::commitment_config::CommitmentConfig;

use crate::{cli_args::Cli, cli_signer::CliSigner};

pub struct CliConfig {
    /// The RPC endpoint URL
    pub rpc_url: String,

    /// The Commitment level
    pub commitment: CommitmentConfig,

    /// Optional signer
    pub signer: Option<CliSigner>,
}

impl CliConfig {
    /// Creates a new [`CliConfig`] from CLI arguments
    pub fn new(args: &Cli) -> Result<Self, anyhow::Error> {
        match &args.config_file {
            Some(config_file) => Self::from_config_file(config_file, args),
            None => Self::from_default_config_or_args(args),
        }
    }

    /// Creates configuration from a specific config file
    fn from_config_file(config_file: &PathBuf, args: &Cli) -> Result<Self, anyhow::Error> {
        let config_path = config_file
            .as_os_str()
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid config file path"))?;

        let config = Config::load(config_path)?;
        let signer = Self::create_signer(args.signer.as_ref(), Some(&config.keypair_path))?;

        Ok(Self {
            rpc_url: config.json_rpc_url,
            commitment: CommitmentConfig::from_str(&config.commitment)?,
            signer: Some(signer),
        })
    }

    /// Attempts to load from default config file
    fn from_default_config_or_args(args: &Cli) -> Result<Self, anyhow::Error> {
        let default_config_file = solana_cli_config::CONFIG_FILE
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Unable to get config file path"))?;

        match Config::load(default_config_file) {
            Ok(config) => Self::from_loaded_config(config, args),
            Err(_) => Self::from_args_only(args),
        }
    }

    /// Creates configuration by merging loaded config file with CLI arguments
    fn from_loaded_config(config: Config, args: &Cli) -> Result<Self, anyhow::Error> {
        let signer = Self::create_signer(args.signer.as_ref(), Some(&config.keypair_path))?;
        let rpc_url = args
            .rpc_url
            .as_ref()
            .map(|url| url.to_string())
            .unwrap_or(config.json_rpc_url);

        Ok(Self {
            rpc_url,
            commitment: CommitmentConfig::from_str(&config.commitment)?,
            signer: Some(signer),
        })
    }

    /// Creates configuration using only CLI arguments (no config file)
    fn from_args_only(args: &Cli) -> Result<Self, anyhow::Error> {
        let signer = Self::try_create_signer(args.signer.as_ref())?;
        let rpc_url = args
            .rpc_url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("RPC URL not provided"))?
            .to_string();

        let commitment = args
            .commitment
            .as_ref()
            .map(|c| CommitmentConfig::from_str(c))
            .transpose()?
            .unwrap_or_else(CommitmentConfig::confirmed);

        Ok(Self {
            rpc_url,
            commitment,
            signer,
        })
    }

    /// Creates a signer from keypair arguments
    fn create_signer(
        keypair_arg: Option<&String>,
        default_keypair_path: Option<&String>,
    ) -> Result<CliSigner, anyhow::Error> {
        let keypair_path = keypair_arg
            .or(default_keypair_path)
            .ok_or_else(|| anyhow::anyhow!("No keypair path provided"))?;

        if keypair_path.starts_with("usb://") {
            Ok(CliSigner::new_ledger(keypair_path))
        } else {
            CliSigner::new_keypair_from_path(keypair_path)
        }
    }

    /// Attempts to create an optional signer from CLI argument only
    fn try_create_signer(keypair_arg: Option<&String>) -> Result<Option<CliSigner>, anyhow::Error> {
        match keypair_arg {
            Some(keypair_path) => {
                let signer = if keypair_path.starts_with("usb://") {
                    CliSigner::new_ledger(keypair_path)
                } else {
                    CliSigner::new_keypair_from_path(keypair_path)?
                };
                Ok(Some(signer))
            }
            None => Ok(None),
        }
    }
}
