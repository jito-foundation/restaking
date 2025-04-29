use std::{path::PathBuf, str::FromStr};

use anyhow::{anyhow, Result};
use borsh::BorshDeserialize;
use jito_restaking_client::{
    instructions::{
        CooldownNcnVaultTicketBuilder, CooldownOperatorVaultTicketBuilder, InitializeConfigBuilder,
        InitializeNcnBuilder, InitializeNcnOperatorStateBuilder, InitializeNcnVaultTicketBuilder,
        InitializeOperatorBuilder, InitializeOperatorVaultTicketBuilder,
        NcnCooldownOperatorBuilder, NcnDelegateTokenAccountBuilder, NcnSetAdminBuilder,
        NcnSetSecondaryAdminBuilder, NcnWarmupOperatorBuilder, OperatorCooldownNcnBuilder,
        OperatorDelegateTokenAccountBuilder, OperatorSetAdminBuilder, OperatorSetFeeBuilder,
        OperatorSetSecondaryAdminBuilder, OperatorWarmupNcnBuilder, SetConfigAdminBuilder,
        WarmupNcnVaultTicketBuilder, WarmupOperatorVaultTicketBuilder,
    },
    types::{NcnAdminRole, OperatorAdminRole},
};
use jito_restaking_client_common::log::PrettyDisplay;
use jito_restaking_core::{
    config::Config, ncn::Ncn, ncn_operator_state::NcnOperatorState,
    ncn_vault_ticket::NcnVaultTicket, operator::Operator,
    operator_vault_ticket::OperatorVaultTicket,
};
use log::info;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Keypair, Signer};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account_idempotent,
};

use crate::{
    cli_config::CliConfig,
    cli_signer::CliSigner,
    restaking::{ConfigActions, NcnActions, OperatorActions, RestakingCommands},
    CliHandler,
};

pub struct RestakingCliHandler {
    /// The configuration of CLI
    cli_config: CliConfig,

    /// The Pubkey of Jito Restaking Program ID
    restaking_program_id: Pubkey,

    /// The Pubkey of Jito Vault Program ID
    vault_program_id: Pubkey,

    /// This will print out the raw TX instead of running it
    print_tx: bool,

    /// This will print out the account information in JSON format
    print_json: bool,

    /// This will print out the account information in JSON format without reserved space
    print_json_without_reserves: bool,
}

impl CliHandler for RestakingCliHandler {
    fn cli_config(&self) -> &CliConfig {
        &self.cli_config
    }

    fn print_tx(&self) -> bool {
        self.print_tx
    }

    fn print_json(&self) -> bool {
        self.print_json
    }

    fn print_json_without_reserves(&self) -> bool {
        self.print_json_without_reserves
    }
}

impl RestakingCliHandler {
    pub const fn new(
        cli_config: CliConfig,
        restaking_program_id: Pubkey,
        vault_program_id: Pubkey,
        print_tx: bool,
        print_json: bool,
        print_json_without_reserves: bool,
    ) -> Self {
        Self {
            cli_config,
            restaking_program_id,
            vault_program_id,
            print_tx,
            print_json,
            print_json_without_reserves,
        }
    }

    #[allow(clippy::future_not_send)]
    pub async fn handle(&self, action: RestakingCommands) -> Result<()> {
        match action {
            RestakingCommands::Config {
                action: ConfigActions::Initialize,
            } => self.initialize_config().await,
            RestakingCommands::Config {
                action: ConfigActions::Get,
            } => self.get_config().await,
            RestakingCommands::Config {
                action: ConfigActions::SetAdmin { new_admin },
            } => self.set_config_admin(new_admin).await,
            RestakingCommands::Ncn {
                action:
                    NcnActions::Initialize {
                        path_to_base_keypair,
                    },
            } => self.initialize_ncn(path_to_base_keypair).await,
            RestakingCommands::Ncn {
                action: NcnActions::InitializeNcnOperatorState { ncn, operator },
            } => self.initialize_ncn_operator_state(ncn, operator).await,
            RestakingCommands::Ncn {
                action: NcnActions::NcnWarmupOperator { ncn, operator },
            } => self.ncn_warmup_operator(ncn, operator).await,
            RestakingCommands::Ncn {
                action: NcnActions::NcnCooldownOperator { ncn, operator },
            } => self.ncn_cooldown_operator(ncn, operator).await,
            RestakingCommands::Ncn {
                action: NcnActions::InitializeNcnVaultTicket { ncn, vault },
            } => self.initialize_ncn_vault_ticket(ncn, vault).await,
            RestakingCommands::Ncn {
                action: NcnActions::WarmupNcnVaultTicket { ncn, vault },
            } => self.warmup_ncn_vault_ticket(ncn, vault).await,
            RestakingCommands::Ncn {
                action: NcnActions::CooldownNcnVaultTicket { ncn, vault },
            } => self.cooldown_ncn_vault_ticket(ncn, vault).await,
            RestakingCommands::Ncn {
                action:
                    NcnActions::NcnDelegateTokenAccount {
                        ncn,
                        delegate,
                        token_mint,
                        should_create_token_account,
                    },
            } => {
                self.ncn_delegate_token_account(
                    ncn,
                    delegate,
                    token_mint,
                    should_create_token_account,
                )
                .await
            }
            RestakingCommands::Ncn {
                action: NcnActions::Get { pubkey },
            } => self.get_ncn(pubkey).await,
            RestakingCommands::Ncn {
                action: NcnActions::List,
            } => self.list_ncn().await,
            RestakingCommands::Ncn {
                action: NcnActions::ListNcnOperatorState { ncn },
            } => self.list_ncn_operator_state(Some(&ncn), None).await,
            RestakingCommands::Ncn {
                action: NcnActions::ListNcnVaultTicket { ncn },
            } => self.list_ncn_vault_ticket(ncn).await,
            RestakingCommands::Ncn {
                action:
                    NcnActions::NcnSetAdmin {
                        ncn,
                        old_admin_keypair,
                        new_admin_keypair,
                    },
            } => {
                self.ncn_set_admin(&ncn, &old_admin_keypair, &new_admin_keypair)
                    .await
            }
            RestakingCommands::Ncn {
                action:
                    NcnActions::NcnSetSecondaryAdmin {
                        ncn,
                        new_admin,
                        set_operator_admin,
                        set_vault_admin,
                        set_slasher_admin,
                        set_delegate_admin,
                        set_metadata_admin,
                        set_weight_table_admin,
                        set_ncn_program_admin,
                    },
            } => {
                self.ncn_set_secondary_admin(
                    &ncn,
                    &new_admin,
                    set_operator_admin,
                    set_vault_admin,
                    set_slasher_admin,
                    set_delegate_admin,
                    set_metadata_admin,
                    set_weight_table_admin,
                    set_ncn_program_admin,
                )
                .await
            }
            RestakingCommands::Operator {
                action: OperatorActions::Initialize { operator_fee_bps },
            } => self.initialize_operator(operator_fee_bps).await,
            RestakingCommands::Operator {
                action: OperatorActions::InitializeOperatorVaultTicket { operator, vault },
            } => self.initialize_operator_vault_ticket(operator, vault).await,
            RestakingCommands::Operator {
                action: OperatorActions::WarmupOperatorVaultTicket { operator, vault },
            } => self.warmup_operator_vault_ticket(operator, vault).await,
            RestakingCommands::Operator {
                action: OperatorActions::CooldownOperatorVaultTicket { operator, vault },
            } => self.cooldown_operator_vault_ticket(operator, vault).await,
            RestakingCommands::Operator {
                action: OperatorActions::OperatorWarmupNcn { operator, ncn },
            } => self.operator_warmup_ncn(operator, ncn).await,
            RestakingCommands::Operator {
                action: OperatorActions::OperatorCooldownNcn { operator, ncn },
            } => self.operator_cooldown_ncn(operator, ncn).await,
            RestakingCommands::Operator {
                action:
                    OperatorActions::OperatorSetAdmin {
                        operator,
                        old_admin_keypair,
                        new_admin_keypair,
                    },
            } => {
                self.operator_set_admin(&operator, &old_admin_keypair, &new_admin_keypair)
                    .await
            }
            RestakingCommands::Operator {
                action:
                    OperatorActions::OperatorSetSecondaryAdmin {
                        operator,
                        new_admin,
                        set_ncn_admin,
                        set_vault_admin,
                        set_voter_admin,
                        set_delegate_admin,
                        set_metadata_admin,
                    },
            } => {
                self.operator_set_secondary_admin(
                    operator,
                    new_admin,
                    set_ncn_admin,
                    set_vault_admin,
                    set_voter_admin,
                    set_delegate_admin,
                    set_metadata_admin,
                )
                .await
            }
            RestakingCommands::Operator {
                action:
                    OperatorActions::OperatorSetFees {
                        operator,
                        operator_fee_bps,
                    },
            } => self.operator_set_fee(operator, operator_fee_bps).await,
            RestakingCommands::Operator {
                action:
                    OperatorActions::OperatorDelegateTokenAccount {
                        operator,
                        delegate,
                        token_mint,
                        should_create_token_account,
                    },
            } => {
                self.operator_delegate_token_account(
                    operator,
                    delegate,
                    token_mint,
                    should_create_token_account,
                )
                .await
            }
            RestakingCommands::Operator {
                action: OperatorActions::Get { pubkey },
            } => self.get_operator(pubkey).await,
            RestakingCommands::Operator {
                action: OperatorActions::List,
            } => self.list_operator().await,
            RestakingCommands::Operator {
                action: OperatorActions::ListOperatorVaultTicket { operator },
            } => self.list_operator_vault_ticket(&operator).await,
            RestakingCommands::Operator {
                action: OperatorActions::ListNcnOperatorState { operator },
            } => self.list_ncn_operator_state(None, Some(&operator)).await,
        }
    }

    /// Sets the primary admin for NCN
    ///
    /// This function transfers the primary administrative control of an NCN from an existing admin
    /// to a new admin.
    #[allow(clippy::future_not_send)]
    async fn ncn_set_admin(
        &self,
        ncn: &str,
        old_admin_keypair: &PathBuf,
        new_admin_keypair: &PathBuf,
    ) -> Result<()> {
        let ncn = Pubkey::from_str(ncn)?;

        let old_admin = read_keypair_file(old_admin_keypair)
            .map_err(|e| anyhow!("Failed to read old admin keypair: {}", e))?;
        let old_admin_signer = CliSigner::new(Some(old_admin), None);

        let new_admin = read_keypair_file(new_admin_keypair)
            .map_err(|e| anyhow!("Failed to read new admin keypair: {}", e))?;
        let new_admin_signer = CliSigner::new(Some(new_admin), None);

        let mut ix_builder = NcnSetAdminBuilder::new();
        ix_builder
            .ncn(ncn)
            .old_admin(old_admin_signer.pubkey())
            .new_admin(new_admin_signer.pubkey());
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        info!("Setting NCN admin to {}", new_admin_signer.pubkey());

        self.process_transaction(
            &[ix],
            &new_admin_signer.pubkey(),
            &[new_admin_signer, old_admin_signer],
        )
        .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_restaking_client::accounts::Ncn>(&ncn)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    /// Sets secondary admin roles for NCN
    ///
    /// This function allows assigning a new administrator to various administrative roles
    /// for a specific NCN. Multiple roles can be assigned in a single call by enabling the
    /// corresponding boolean flags.
    #[allow(clippy::too_many_arguments, clippy::future_not_send)]
    async fn ncn_set_secondary_admin(
        &self,
        ncn: &str,
        new_admin: &str,
        set_operator_admin: bool,
        set_vault_admin: bool,
        set_slasher_admin: bool,
        set_delegate_admin: bool,
        set_metadata_admin: bool,
        set_weight_table_admin: bool,
        set_ncn_program_admin: bool,
    ) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No signer"))?;

        let ncn = Pubkey::from_str(ncn)?;
        let new_admin = Pubkey::from_str(new_admin)?;

        let mut roles: Vec<NcnAdminRole> = vec![];
        if set_operator_admin {
            roles.push(NcnAdminRole::OperatorAdmin);
        }
        if set_vault_admin {
            roles.push(NcnAdminRole::VaultAdmin);
        }
        if set_slasher_admin {
            roles.push(NcnAdminRole::SlasherAdmin);
        }
        if set_delegate_admin {
            roles.push(NcnAdminRole::DelegateAdmin);
        }
        if set_metadata_admin {
            roles.push(NcnAdminRole::MetadataAdmin);
        }
        if set_weight_table_admin {
            roles.push(NcnAdminRole::WeightTableAdmin);
        }
        if set_ncn_program_admin {
            roles.push(NcnAdminRole::NcnProgramAdmin);
        }

        for role in roles.iter() {
            let mut ix_builder = NcnSetSecondaryAdminBuilder::new();
            ix_builder
                .new_admin(new_admin)
                .ncn(ncn)
                .admin(signer.pubkey())
                .ncn_admin_role(*role)
                .instruction();
            let mut ix = ix_builder.instruction();
            ix.program_id = self.restaking_program_id;

            info!("Setting {:?} Admin to {} for NCN {}", role, new_admin, ncn);

            self.process_transaction(&[ix], &signer.pubkey(), &[signer])
                .await?;
        }

        if !self.print_tx {
            let account = self
                .get_account::<jito_restaking_client::accounts::Ncn>(&ncn)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    /// Sets the primary admin for Operator
    ///
    /// This function transfers the primary administrative control of an Operator from an existing admin
    /// to a new admin.
    #[allow(clippy::future_not_send)]
    async fn operator_set_admin(
        &self,
        operator: &str,
        old_admin_keypair: &PathBuf,
        new_admin_keypair: &PathBuf,
    ) -> Result<()> {
        let operator = Pubkey::from_str(operator)?;

        let old_admin = read_keypair_file(old_admin_keypair)
            .map_err(|e| anyhow!("Failed to read old admin keypair: {}", e))?;
        let old_admin_signer = CliSigner::new(Some(old_admin), None);

        let new_admin = read_keypair_file(new_admin_keypair)
            .map_err(|e| anyhow!("Failed to read new admin keypair: {}", e))?;
        let new_admin_signer = CliSigner::new(Some(new_admin), None);

        let mut ix_builder = OperatorSetAdminBuilder::new();
        ix_builder
            .operator(operator)
            .old_admin(old_admin_signer.pubkey())
            .new_admin(new_admin_signer.pubkey());
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        info!("Setting Operator admin to {}", new_admin_signer.pubkey());

        self.process_transaction(
            &[ix],
            &new_admin_signer.pubkey(),
            &[new_admin_signer, old_admin_signer],
        )
        .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_restaking_client::accounts::Operator>(&operator)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn operator_set_fee(&self, operator: String, operator_fee_bps: u16) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No signer"))?;

        let (restaking_vault_config, _, _) =
            Config::find_program_address(&self.restaking_program_id);

        let operator = Pubkey::from_str(&operator)?;

        let mut ix_builder = OperatorSetFeeBuilder::new();
        ix_builder
            .operator(operator)
            .new_fee_bps(operator_fee_bps)
            .admin(signer.pubkey())
            .config(restaking_vault_config)
            .instruction();
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        info!(
            "Setting fees to {:?} to Operator {}",
            operator_fee_bps, operator,
        );

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_restaking_client::accounts::Operator>(&operator)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments, clippy::future_not_send)]
    pub async fn operator_set_secondary_admin(
        &self,
        operator: String,
        new_admin: String,
        set_ncn_admin: bool,
        set_vault_admin: bool,
        set_voter_admin: bool,
        set_delegate_admin: bool,
        set_metadata_admin: bool,
    ) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No signer"))?;

        let operator = Pubkey::from_str(&operator)?;
        let new_admin = Pubkey::from_str(&new_admin)?;

        let mut roles: Vec<OperatorAdminRole> = vec![];
        if set_ncn_admin {
            roles.push(OperatorAdminRole::NcnAdmin);
        }
        if set_vault_admin {
            roles.push(OperatorAdminRole::VaultAdmin);
        }
        if set_voter_admin {
            roles.push(OperatorAdminRole::VoterAdmin);
        }
        if set_delegate_admin {
            roles.push(OperatorAdminRole::DelegateAdmin);
        }
        if set_metadata_admin {
            roles.push(OperatorAdminRole::MetadataAdmin);
        }

        for role in roles.iter() {
            let mut ix_builder = OperatorSetSecondaryAdminBuilder::new();
            ix_builder
                .new_admin(new_admin)
                .operator(operator)
                .admin(signer.pubkey())
                .operator_admin_role(*role)
                .instruction();
            let mut ix = ix_builder.instruction();
            ix.program_id = self.restaking_program_id;

            info!(
                "Setting {:?} Admin to {} for Operator {}",
                role, new_admin, operator
            );

            self.process_transaction(&[ix], &signer.pubkey(), &[signer])
                .await?;
        }

        if !self.print_tx {
            let account = self
                .get_account::<jito_restaking_client::accounts::Operator>(&operator)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn operator_delegate_token_account(
        &self,
        operator: String,
        delegate: String,
        token_mint: String,
        should_create_token_account: bool,
    ) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No signer"))?;

        let operator = Pubkey::from_str(&operator)?;
        let delegate = Pubkey::from_str(&delegate)?;
        let token_mint = Pubkey::from_str(&token_mint)?;

        let token_account = get_associated_token_address(&operator, &token_mint);

        let mut ixs = vec![];

        if should_create_token_account {
            let ix = create_associated_token_account_idempotent(
                &signer.pubkey(),
                &operator,
                &token_mint,
                &spl_token::id(),
            );
            ixs.push(ix);
        }

        let mut ix_builder = OperatorDelegateTokenAccountBuilder::new();
        ix_builder
            .operator(operator)
            .delegate(delegate)
            .delegate_admin(signer.pubkey())
            .token_account(token_account)
            .token_mint(token_mint);
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        ixs.push(ix_builder.instruction());

        info!("Setting delegate for mint: {} to {}", token_mint, delegate,);

        self.process_transaction(&ixs, &signer.pubkey(), &[signer])
            .await?;

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn ncn_delegate_token_account(
        &self,
        ncn: String,
        delegate: String,
        token_mint: String,
        should_create_token_account: bool,
    ) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No signer"))?;

        let ncn = Pubkey::from_str(&ncn)?;
        let delegate = Pubkey::from_str(&delegate)?;
        let token_mint = Pubkey::from_str(&token_mint)?;

        let token_account = get_associated_token_address(&ncn, &token_mint);

        let mut ixs = vec![];

        if should_create_token_account {
            let ix = create_associated_token_account_idempotent(
                &signer.pubkey(),
                &ncn,
                &token_mint,
                &spl_token::id(),
            );
            ixs.push(ix);
        }

        let mut ix_builder = NcnDelegateTokenAccountBuilder::new();
        ix_builder
            .ncn(ncn)
            .delegate(delegate)
            .delegate_admin(signer.pubkey())
            .token_account(token_account)
            .token_mint(token_mint);
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        ixs.push(ix_builder.instruction());

        info!("Setting delegate for mint: {} to {}", token_mint, delegate,);

        self.process_transaction(&ixs, &signer.pubkey(), &[signer])
            .await?;

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn initialize_ncn_vault_ticket(&self, ncn: String, vault: String) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No signer"))?;

        let ncn = Pubkey::from_str(&ncn)?;
        let vault = Pubkey::from_str(&vault)?;

        let (ncn_vault_ticket, _, _) =
            NcnVaultTicket::find_program_address(&self.restaking_program_id, &ncn, &vault);

        let mut ix_builder = InitializeNcnVaultTicketBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .ncn(ncn)
            .vault(vault)
            .ncn_vault_ticket(ncn_vault_ticket)
            .admin(signer.pubkey())
            .payer(signer.pubkey())
            .instruction();
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        info!("Initializing NCN Vault Ticket");

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_restaking_client::accounts::NcnVaultTicket>(&ncn_vault_ticket)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn warmup_ncn_vault_ticket(&self, ncn: String, vault: String) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No signer"))?;

        let ncn = Pubkey::from_str(&ncn)?;
        let vault = Pubkey::from_str(&vault)?;

        let (ncn_vault_ticket, _, _) =
            NcnVaultTicket::find_program_address(&self.restaking_program_id, &ncn, &vault);

        let mut ix_builder = WarmupNcnVaultTicketBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .ncn(ncn)
            .vault(vault)
            .ncn_vault_ticket(ncn_vault_ticket)
            .admin(signer.pubkey())
            .instruction();
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        info!("Warmup NCN Vault Ticket");

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_restaking_client::accounts::NcnVaultTicket>(&ncn_vault_ticket)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn cooldown_ncn_vault_ticket(&self, ncn: String, vault: String) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No signer"))?;

        let ncn = Pubkey::from_str(&ncn)?;
        let vault = Pubkey::from_str(&vault)?;

        let (ncn_vault_ticket, _, _) =
            NcnVaultTicket::find_program_address(&self.restaking_program_id, &ncn, &vault);

        let mut ix_builder = CooldownNcnVaultTicketBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .ncn(ncn)
            .vault(vault)
            .ncn_vault_ticket(ncn_vault_ticket)
            .admin(signer.pubkey())
            .instruction();
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        info!("Cooldown NCN Vault Ticket");

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn initialize_ncn_operator_state(&self, ncn: String, operator: String) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No signer"))?;

        let ncn = Pubkey::from_str(&ncn)?;
        let operator = Pubkey::from_str(&operator)?;

        let (ncn_operator_state, _, _) =
            NcnOperatorState::find_program_address(&self.restaking_program_id, &ncn, &operator);

        let mut ix_builder = InitializeNcnOperatorStateBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .ncn(ncn)
            .operator(operator)
            .ncn_operator_state(ncn_operator_state)
            .admin(signer.pubkey())
            .payer(signer.pubkey())
            .instruction();
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        info!("Initializing NCN Operator State");

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_restaking_client::accounts::NcnOperatorState>(
                    &ncn_operator_state,
                )
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn ncn_warmup_operator(&self, ncn: String, operator: String) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No signer"))?;

        let ncn = Pubkey::from_str(&ncn)?;
        let operator = Pubkey::from_str(&operator)?;

        let (ncn_operator_state, _, _) =
            NcnOperatorState::find_program_address(&self.restaking_program_id, &ncn, &operator);

        let mut ix_builder = NcnWarmupOperatorBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .ncn(ncn)
            .operator(operator)
            .ncn_operator_state(ncn_operator_state)
            .admin(signer.pubkey())
            .instruction();
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        info!("NCN Warmup Operator");

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_restaking_client::accounts::NcnOperatorState>(
                    &ncn_operator_state,
                )
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn ncn_cooldown_operator(&self, ncn: String, operator: String) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No signer"))?;

        let ncn = Pubkey::from_str(&ncn)?;
        let operator = Pubkey::from_str(&operator)?;

        let (ncn_operator_state, _, _) =
            NcnOperatorState::find_program_address(&self.restaking_program_id, &ncn, &operator);

        let mut ix_builder = NcnCooldownOperatorBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .ncn(ncn)
            .operator(operator)
            .ncn_operator_state(ncn_operator_state)
            .admin(signer.pubkey())
            .instruction();
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        info!("NCN Cooldown Operator");

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_restaking_client::accounts::NcnOperatorState>(
                    &ncn_operator_state,
                )
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn operator_warmup_ncn(&self, operator: String, ncn: String) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No signer"))?;

        let operator = Pubkey::from_str(&operator)?;
        let ncn = Pubkey::from_str(&ncn)?;

        let (ncn_operator_state, _, _) =
            NcnOperatorState::find_program_address(&self.restaking_program_id, &ncn, &operator);

        let mut ix_builder = OperatorWarmupNcnBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .ncn(ncn)
            .operator(operator)
            .ncn_operator_state(ncn_operator_state)
            .admin(signer.pubkey())
            .instruction();
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        info!("Operator Warmup NCN");

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_restaking_client::accounts::NcnOperatorState>(
                    &ncn_operator_state,
                )
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn operator_cooldown_ncn(&self, operator: String, ncn: String) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No signer"))?;

        let operator = Pubkey::from_str(&operator)?;
        let ncn = Pubkey::from_str(&ncn)?;

        let (ncn_operator_state, _, _) =
            NcnOperatorState::find_program_address(&self.restaking_program_id, &ncn, &operator);

        let mut ix_builder = OperatorCooldownNcnBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .ncn(ncn)
            .operator(operator)
            .ncn_operator_state(ncn_operator_state)
            .admin(signer.pubkey())
            .instruction();
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        info!("Operator Cooldown NCN");

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_restaking_client::accounts::NcnOperatorState>(
                    &ncn_operator_state,
                )
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    async fn initialize_config(&self) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No signer"))?;

        let config_address = Config::find_program_address(&self.restaking_program_id).0;
        let mut ix_builder = InitializeConfigBuilder::new();
        ix_builder
            .config(config_address)
            .admin(signer.pubkey())
            .vault_program(self.vault_program_id);
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        info!("Initializing restaking config parameters: {:?}", ix_builder);

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_restaking_client::accounts::Config>(&config_address)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn initialize_ncn(&self, path_to_base_keypair: Option<String>) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No signer"))?;

        let base =
            path_to_base_keypair.map_or_else(Keypair::new, |path| read_keypair_file(path).unwrap());
        let base_signer = CliSigner::new(Some(base), None);
        let ncn = Ncn::find_program_address(&self.restaking_program_id, &base_signer.pubkey()).0;

        let mut ix_builder = InitializeNcnBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .ncn(ncn)
            .admin(signer.pubkey())
            .base(base_signer.pubkey())
            .instruction();
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        info!("Initializing NCN: {:?}", ncn);

        self.process_transaction(&[ix], &signer.pubkey(), &[signer, &base_signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_restaking_client::accounts::Ncn>(&ncn)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn initialize_operator(&self, operator_fee_bps: u16) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No signer"))?;

        let base_signer = CliSigner::new(Some(Keypair::new()), None);
        let operator =
            Operator::find_program_address(&self.restaking_program_id, &base_signer.pubkey()).0;

        let mut ix_builder = InitializeOperatorBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .operator(operator)
            .admin(signer.pubkey())
            .base(base_signer.pubkey())
            .operator_fee_bps(operator_fee_bps)
            .instruction();
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        info!("Initializing Operator: {:?}", operator);

        self.process_transaction(&[ix], &signer.pubkey(), &[signer, &base_signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_restaking_client::accounts::Operator>(&operator)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn initialize_operator_vault_ticket(
        &self,
        operator: String,
        vault: String,
    ) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;

        let operator = Pubkey::from_str(&operator)?;
        let vault = Pubkey::from_str(&vault)?;

        let operator_vault_ticket = OperatorVaultTicket::find_program_address(
            &self.restaking_program_id,
            &operator,
            &vault,
        )
        .0;

        let mut ix_builder = InitializeOperatorVaultTicketBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .operator(operator)
            .vault(vault)
            .admin(signer.pubkey())
            .operator_vault_ticket(operator_vault_ticket)
            .payer(signer.pubkey());
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        info!("Operator Vault Ticket address: {}", operator_vault_ticket);
        info!("Operator address: {}", operator);
        info!("Vault address: {}", vault);

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_restaking_client::accounts::OperatorVaultTicket>(
                    &operator_vault_ticket,
                )
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn warmup_operator_vault_ticket(
        &self,
        operator: String,
        vault: String,
    ) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;

        let operator = Pubkey::from_str(&operator)?;
        let vault = Pubkey::from_str(&vault)?;

        let operator_vault_ticket = OperatorVaultTicket::find_program_address(
            &self.restaking_program_id,
            &operator,
            &vault,
        )
        .0;

        let mut ix_builder = WarmupOperatorVaultTicketBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .operator(operator)
            .vault(vault)
            .operator_vault_ticket(operator_vault_ticket)
            .admin(signer.pubkey());
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        info!("Warming up operator vault ticket transaction");
        info!("Operator address: {}", operator);
        info!("Vault address: {}", vault);

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_restaking_client::accounts::OperatorVaultTicket>(
                    &operator_vault_ticket,
                )
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn cooldown_operator_vault_ticket(
        &self,
        operator: String,
        vault: String,
    ) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("Keypair not provided"))?;

        let operator = Pubkey::from_str(&operator)?;
        let vault = Pubkey::from_str(&vault)?;

        let operator_vault_ticket = OperatorVaultTicket::find_program_address(
            &self.restaking_program_id,
            &operator,
            &vault,
        )
        .0;

        let mut ix_builder = CooldownOperatorVaultTicketBuilder::new();
        ix_builder
            .config(Config::find_program_address(&self.restaking_program_id).0)
            .operator(operator)
            .vault(vault)
            .operator_vault_ticket(operator_vault_ticket)
            .admin(signer.pubkey());
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        info!("Cooldown Operator Vault Ticket");
        info!("Operator address: {}", operator);
        info!("Vault address: {}", vault);

        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_restaking_client::accounts::OperatorVaultTicket>(
                    &operator_vault_ticket,
                )
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn get_config(&self) -> Result<()> {
        let rpc_client = self.get_rpc_client();

        let config_address = Config::find_program_address(&self.restaking_program_id).0;

        let account = rpc_client.get_account(&config_address).await?;
        let config =
            jito_restaking_client::accounts::Config::deserialize(&mut account.data.as_slice())?;

        self.print_out(None, Some(&config_address), &config)?;

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn get_ncn(&self, pubkey: String) -> Result<()> {
        let pubkey = Pubkey::from_str(&pubkey)?;
        let account = self.get_rpc_client().get_account(&pubkey).await?;
        let ncn = jito_restaking_client::accounts::Ncn::deserialize(&mut account.data.as_slice())?;

        self.print_out(None, Some(&pubkey), &ncn)?;

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn list_ncn(&self) -> Result<()> {
        let rpc_client = self.get_rpc_client();
        let config = self.get_rpc_program_accounts_config::<Ncn>(None)?;

        let accounts = rpc_client
            .get_program_accounts_with_config(&self.restaking_program_id, config)
            .await?;
        for (index, (ncn_pubkey, ncn)) in accounts.iter().enumerate() {
            let ncn = jito_restaking_client::accounts::Ncn::deserialize(&mut ncn.data.as_slice())?;

            self.print_out(Some(index), Some(ncn_pubkey), &ncn)?;
        }
        Ok(())
    }

    /// Lists NCN operator state accounts filtered by either NCN or Operator public key.
    #[allow(clippy::future_not_send)]
    pub async fn list_ncn_operator_state(
        &self,
        ncn: Option<&Pubkey>,
        operator: Option<&Pubkey>,
    ) -> Result<()> {
        let rpc_client = self.get_rpc_client();

        let (pubkey, offset) = match (ncn, operator) {
            (Some(ncn_pubkey), None) => (ncn_pubkey, 8),
            (None, Some(operator_pubkey)) => (operator_pubkey, 8 + 32),
            _ => return Err(anyhow!("Choose Operator or NCN")),
        };

        let config =
            self.get_rpc_program_accounts_config::<NcnOperatorState>(Some((pubkey, offset)))?;

        let accounts = rpc_client
            .get_program_accounts_with_config(&self.restaking_program_id, config)
            .await?;
        for (index, (ncn_operator_state_pubkey, ncn_operator_state)) in accounts.iter().enumerate()
        {
            let ncn_operator_state =
                jito_restaking_client::accounts::NcnOperatorState::deserialize(
                    &mut ncn_operator_state.data.as_slice(),
                )?;
            self.print_out(
                Some(index),
                Some(ncn_operator_state_pubkey),
                &ncn_operator_state,
            )?;
        }
        Ok(())
    }

    /// Lists NCN operator state accounts filtered by NCN public key.
    #[allow(clippy::future_not_send)]
    pub async fn list_ncn_vault_ticket(&self, ncn: Pubkey) -> Result<()> {
        let rpc_client = self.get_rpc_client();
        let config = self.get_rpc_program_accounts_config::<NcnVaultTicket>(Some((&ncn, 8)))?;

        let accounts = rpc_client
            .get_program_accounts_with_config(&self.restaking_program_id, config)
            .await?;
        for (index, (ticket_pubkey, ticket)) in accounts.iter().enumerate() {
            let ticket = jito_restaking_client::accounts::NcnVaultTicket::deserialize(
                &mut ticket.data.as_slice(),
            )?;
            self.print_out(Some(index), Some(ticket_pubkey), &ticket)?;
        }
        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn get_operator(&self, pubkey: String) -> Result<()> {
        let pubkey = Pubkey::from_str(&pubkey)?;
        let account = self.get_rpc_client().get_account(&pubkey).await?;
        let operator =
            jito_restaking_client::accounts::Operator::deserialize(&mut account.data.as_slice())?;
        self.print_out(None, Some(&pubkey), &operator)?;

        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn list_operator(&self) -> Result<()> {
        let rpc_client = self.get_rpc_client();
        let config = self.get_rpc_program_accounts_config::<Operator>(None)?;
        let accounts = rpc_client
            .get_program_accounts_with_config(&self.restaking_program_id, config)
            .await?;
        for (index, (operator_pubkey, operator)) in accounts.iter().enumerate() {
            let operator = jito_restaking_client::accounts::Operator::deserialize(
                &mut operator.data.as_slice(),
            )?;
            self.print_out(Some(index), Some(operator_pubkey), &operator)?;
        }
        Ok(())
    }

    #[allow(clippy::future_not_send)]
    pub async fn list_operator_vault_ticket(&self, operator: &Pubkey) -> Result<()> {
        let rpc_client = self.get_rpc_client();
        let config =
            self.get_rpc_program_accounts_config::<OperatorVaultTicket>(Some((operator, 8)))?;
        let accounts = rpc_client
            .get_program_accounts_with_config(&self.restaking_program_id, config)
            .await?;
        for (index, (ticket_pubkey, ticket)) in accounts.iter().enumerate() {
            let ticket = jito_restaking_client::accounts::OperatorVaultTicket::deserialize(
                &mut ticket.data.as_slice(),
            )?;
            self.print_out(Some(index), Some(ticket_pubkey), &ticket)?;
        }
        Ok(())
    }

    #[allow(clippy::future_not_send)]
    async fn set_config_admin(&self, new_admin: Pubkey) -> Result<()> {
        let signer = self
            .cli_config
            .signer
            .as_ref()
            .ok_or_else(|| anyhow!("No signer"))?;

        let config_address = Config::find_program_address(&self.restaking_program_id).0;
        let mut ix_builder = SetConfigAdminBuilder::new();
        ix_builder
            .config(config_address)
            .old_admin(signer.pubkey())
            .new_admin(new_admin);
        let mut ix = ix_builder.instruction();
        ix.program_id = self.restaking_program_id;

        info!(
            "Setting restaking config admin parameters: {:?}",
            ix_builder
        );
        self.process_transaction(&[ix], &signer.pubkey(), &[signer])
            .await?;

        if !self.print_tx {
            let account = self
                .get_account::<jito_restaking_client::accounts::Config>(&config_address)
                .await?;
            info!("{}", account.pretty_display());
        }

        Ok(())
    }
}
