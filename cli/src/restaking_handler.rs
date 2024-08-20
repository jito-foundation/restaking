use crate::cli_args::{
    CliConfig, OperatorActions, ProgramCommand, RestakingCommands, RestakingConfigActions,
    VaultCommands, VaultConfigActions,
};
use jito_account_traits::AccountDeserialize;
use jito_restaking_core::config::Config;
use solana_program::pubkey::Pubkey;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;

pub struct CliHandler {
    cli_config: CliConfig,
    restaking_program_id: Pubkey,
    vault_program_id: Pubkey,
}

impl CliHandler {
    pub fn new(
        cli_config: CliConfig,
        restaking_program_id: Pubkey,
        vault_program_id: Pubkey,
    ) -> Self {
        Self {
            cli_config,
            restaking_program_id,
            vault_program_id,
        }
    }

    pub async fn handle(&self, args: ProgramCommand) -> Result<(), anyhow::Error> {
        match args {
            ProgramCommand::Restaking { action } => self.handle_restaking(action).await,
            ProgramCommand::Vault { action } => self.handle_vault(action).await,
        }
    }

    async fn handle_restaking(&self, args: RestakingCommands) -> Result<(), anyhow::Error> {
        match args {
            RestakingCommands::Config { action } => self.handle_restaking_config(action).await,
            RestakingCommands::Operator { action } => self.handle_restaking_operator(action).await,
        }
    }

    async fn handle_vault(&self, args: VaultCommands) -> Result<(), anyhow::Error> {
        match args {
            VaultCommands::Config { action } => self.handle_vault_config(action).await,
        }
    }

    async fn handle_vault_config(&self, args: VaultConfigActions) -> Result<(), anyhow::Error> {
        match args {
            VaultConfigActions::Initialize => {}
            VaultConfigActions::Set => {}
            VaultConfigActions::Get => {}
        }
        Ok(())
    }

    async fn handle_restaking_config(
        &self,
        args: RestakingConfigActions,
    ) -> Result<(), anyhow::Error> {
        match args {
            RestakingConfigActions::Initialize => {}
            RestakingConfigActions::Set => {}
            RestakingConfigActions::Get => {
                let rpc_client = self.get_rpc_client();
                let config_address = Config::find_program_address(&self.restaking_program_id).0;
                let account = rpc_client.get_account(&config_address).await?;
                let config = Config::try_from_slice_unchecked(&account.data)?;
                println!("Restaking config");
                println!("Address: {:?}", config_address);
                println!("Config: {:?}", config);
            }
        }
        Ok(())
    }

    async fn handle_restaking_operator(&self, args: OperatorActions) -> Result<(), anyhow::Error> {
        match args {
            OperatorActions::Set => {}
        }
        Ok(())
    }

    fn get_rpc_client(&self) -> RpcClient {
        RpcClient::new_with_commitment(self.cli_config.rpc_url.clone(), self.cli_config.commitment)
    }
}
