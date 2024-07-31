use borsh::BorshDeserialize;
use jito_restaking_core::avs_operator_ticket::AvsOperatorTicket;
use jito_restaking_core::avs_vault_slasher_ticket::AvsVaultSlasherTicket;
use jito_restaking_core::avs_vault_ticket::AvsVaultTicket;
use jito_restaking_core::operator_avs_ticket::OperatorAvsTicket;
use jito_restaking_core::operator_vault_ticket::OperatorVaultTicket;
use jito_vault_core::{
    config::Config, vault::Vault, vault_avs_slasher_operator_ticket::VaultAvsSlasherOperatorTicket,
    vault_avs_slasher_ticket::VaultAvsSlasherTicket, vault_avs_ticket::VaultAvsTicket,
    vault_delegation_list::VaultDelegationList, vault_operator_ticket::VaultOperatorTicket,
};
use jito_vault_sdk::{add_delegation, initialize_config, initialize_vault};
use solana_program::{
    native_token::sol_to_lamports,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::{create_account, transfer},
};
use solana_program_test::{BanksClient, BanksClientError};
use solana_sdk::{
    commitment_config::CommitmentLevel,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account_idempotent,
};
use spl_token::{
    instruction::initialize_mint2,
    state::{Account, Mint},
};
//
// struct EnqueueWithdrawAccountData {
//     config_pubkey: Pubkey,
//     config: Config,
//
//     vault_pubkey: Pubkey,
//     vault: Vault,
//
//     vault_delegation_list_pubkey: Pubkey,
//     vault_delegation_list: VaultDelegationList,
//
//     vault_staker_withdraw_ticket: Pubkey,
//
//     vault_staker_withdraw_ticket_token_account: Pubkey,
//
//     staker: Keypair,
//
//     staker_lrt_token_account: Pubkey,
//
//     base: Keypair,
// }

// fn prepare_accounts(
//     vault_index: u64,
//     deposit_fee_bps: u16,
//     withdrawal_fee_bps: u16,
// ) -> EnqueueWithdrawAccountData {
//     let (config_pubkey, config_bump, _) = Config::find_program_address(&jito_vault_program::id());
//     let mut config = Config::new(
//         Pubkey::new_unique(),
//         jito_restaking_program::id(),
//         config_bump,
//     );
//     // assume the vault is created
//     config.increment_vaults().unwrap();
//
//     let base = Pubkey::new_unique();
//     let (vault_pubkey, bump, _) = Vault::find_program_address(&jito_vault_program::id(), &base);
//     let vault = Vault::new(
//         Pubkey::new_unique(),
//         Pubkey::new_unique(),
//         Pubkey::new_unique(),
//         vault_index,
//         base,
//         deposit_fee_bps,
//         withdrawal_fee_bps,
//         bump,
//     );
//
//     let (vault_delegation_list_pubkey, vault_delegation_list_bump, _) =
//         VaultDelegationList::find_program_address(&jito_vault_program::id(), &vault_pubkey);
//     let vault_delegation_list = VaultDelegationList::new(vault_pubkey, vault_delegation_list_bump);
//
//     let staker = Keypair::new();
//
//     let staker_lrt_token_account =
//         get_associated_token_address(&staker.pubkey(), &vault.lrt_mint());
//
//     let base = Keypair::new();
//
//     let vault_staker_withdraw_ticket = VaultStakerWithdrawTicket::find_program_address(
//         &jito_vault_program::id(),
//         &vault_pubkey,
//         &staker.pubkey(),
//         &base.pubkey(),
//     )
//     .0;
//     let vault_staker_withdraw_ticket_token_account =
//         get_associated_token_address(&vault_staker_withdraw_ticket, &vault.lrt_mint());
//
//     EnqueueWithdrawAccountData {
//         config_pubkey,
//         config,
//         vault_pubkey,
//         vault,
//         vault_delegation_list_pubkey,
//         vault_delegation_list,
//         vault_staker_withdraw_ticket,
//         vault_staker_withdraw_ticket_token_account,
//         staker,
//         staker_lrt_token_account,
//         base,
//     }
// }
//
// async fn write_enqueue_withdraw_accounts(
//     accounts: &EnqueueWithdrawAccountData,
//     fixture: &mut TestBuilder,
// ) {
//     let EnqueueWithdrawAccountData {
//         config_pubkey,
//         config,
//         vault_pubkey,
//         vault,
//         vault_delegation_list_pubkey,
//         vault_delegation_list,
//         vault_staker_withdraw_ticket: _, // created in the function
//         vault_staker_withdraw_ticket_token_account,
//         staker,
//         staker_lrt_token_account,
//         base: _,
//     } = accounts;
//
//     fixture
//         .store_borsh_account(&config_pubkey, &jito_vault_program::id(), &config)
//         .await
//         .unwrap();
//
//     fixture
//         .store_borsh_account(vault_pubkey, &jito_vault_program::id(), &vault)
//         .await
//         .unwrap();
//     let mut mint_buf = [0; Mint::LEN];
//     Mint {
//         mint_authority: COption::Some(*vault_pubkey),
//         supply: 0,
//         decimals: 9,
//         is_initialized: true,
//         freeze_authority: COption::None,
//     }
//     .pack_into_slice(&mut mint_buf);
//     fixture
//         .store_account(&vault.lrt_mint(), &spl_token::id(), &mint_buf)
//         .await
//         .unwrap();
//     fixture
//         .store_account(&vault.supported_mint(), &spl_token::id(), &mint_buf)
//         .await
//         .unwrap();
//
//     fixture
//         .store_borsh_account(
//             &VaultDelegationList::find_program_address(&jito_vault_program::id(), &vault_pubkey).0,
//             &jito_vault_program::id(),
//             &vault_delegation_list,
//         )
//         .await
//         .unwrap();
//
//     let mut token_account_buf = [0; Account::LEN];
//     // setup the LRT token account owned by the vault_staker_withdraw_ticket
//     Account {
//         mint: vault.lrt_mint(),
//         owner: *vault_pubkey,
//         amount: 0,
//         delegate: COption::None,
//         state: spl_token::state::AccountState::Initialized,
//         is_native: COption::None,
//         delegated_amount: 0,
//         close_authority: COption::None,
//     }
//     .pack_into_slice(&mut token_account_buf);
//     fixture
//         .store_account(
//             vault_staker_withdraw_ticket_token_account,
//             &spl_token::id(),
//             &token_account_buf,
//         )
//         .await
//         .unwrap();
//
//     // setup the LRT token account owned by the staker
//     Account {
//         mint: vault.lrt_mint(),
//         owner: staker.pubkey(),
//         amount: 0,
//         delegate: COption::None,
//         state: spl_token::state::AccountState::Initialized,
//         is_native: COption::None,
//         delegated_amount: 0,
//         close_authority: COption::None,
//     }
//     .pack_into_slice(&mut token_account_buf);
//     fixture
//         .store_account(
//             staker_lrt_token_account,
//             &spl_token::id(),
//             &token_account_buf,
//         )
//         .await
//         .unwrap();
// }

pub struct VaultRoot {
    pub vault_pubkey: Pubkey,
    pub vault_admin: Keypair,
}

pub struct VaultProgramClient {
    banks_client: BanksClient,
    payer: Keypair,
}

impl VaultProgramClient {
    pub const fn new(banks_client: BanksClient, payer: Keypair) -> Self {
        Self {
            banks_client,
            payer,
        }
    }

    pub async fn get_config(&mut self, account: &Pubkey) -> Result<Config, BanksClientError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(Config::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn get_vault(&mut self, account: &Pubkey) -> Result<Vault, BanksClientError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(Vault::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn get_vault_avs_ticket(
        &mut self,
        vault: &Pubkey,
        avs: &Pubkey,
    ) -> Result<VaultAvsTicket, BanksClientError> {
        let account = VaultAvsTicket::find_program_address(&jito_vault_program::id(), vault, avs).0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(VaultAvsTicket::deserialize(&mut account.data.as_slice())?)
    }

    pub async fn get_vault_operator_ticket(
        &mut self,
        vault: &Pubkey,
        operator: &Pubkey,
    ) -> Result<VaultOperatorTicket, BanksClientError> {
        let account =
            VaultOperatorTicket::find_program_address(&jito_vault_program::id(), vault, operator).0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(VaultOperatorTicket::deserialize(
            &mut account.data.as_slice(),
        )?)
    }

    pub async fn get_vault_delegation_list(
        &mut self,
        account: &Pubkey,
    ) -> Result<VaultDelegationList, BanksClientError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(VaultDelegationList::deserialize(
            &mut account.data.as_slice(),
        )?)
    }

    pub async fn get_vault_avs_slasher_ticket(
        &mut self,
        vault: &Pubkey,
        avs: &Pubkey,
        slasher: &Pubkey,
    ) -> Result<VaultAvsSlasherTicket, BanksClientError> {
        let account = VaultAvsSlasherTicket::find_program_address(
            &jito_vault_program::id(),
            vault,
            avs,
            slasher,
        )
        .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(VaultAvsSlasherTicket::deserialize(
            &mut account.data.as_slice(),
        )?)
    }

    pub async fn get_vault_avs_slasher_operator_ticket(
        &mut self,
        vault: &Pubkey,
        avs: &Pubkey,
        slasher: &Pubkey,
        operator: &Pubkey,
        epoch: u64,
    ) -> Result<VaultAvsSlasherOperatorTicket, BanksClientError> {
        let account = VaultAvsSlasherOperatorTicket::find_program_address(
            &jito_vault_program::id(),
            vault,
            avs,
            slasher,
            operator,
            epoch,
        )
        .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(VaultAvsSlasherOperatorTicket::deserialize(
            &mut account.data.as_slice(),
        )?)
    }

    pub async fn setup_config(&mut self) -> Result<Keypair, BanksClientError> {
        let config_admin = Keypair::new();

        self._airdrop(&config_admin.pubkey(), 1.0).await?;

        let config_pubkey = Config::find_program_address(&jito_vault_program::id()).0;
        self.initialize_config(&config_pubkey, &config_admin)
            .await?;

        Ok(config_admin)
    }

    pub async fn initialize_config(
        &mut self,
        config: &Pubkey,
        config_admin: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_config(
                &jito_vault_program::id(),
                &config,
                &config_admin.pubkey(),
                &jito_restaking_program::id(),
            )],
            Some(&config_admin.pubkey()),
            &[config_admin],
            blockhash,
        ))
        .await
    }

    pub async fn setup_vault(
        &mut self,
        deposit_fee_bps: u16,
        withdraw_fee_bps: u16,
    ) -> Result<(Keypair, VaultRoot), BanksClientError> {
        let config_admin = self.setup_config().await?;

        let vault_base = Keypair::new();

        let vault_pubkey =
            Vault::find_program_address(&jito_vault_program::id(), &vault_base.pubkey()).0;
        let vault_delegation_list =
            VaultDelegationList::find_program_address(&jito_vault_program::id(), &vault_pubkey).0;

        let lrt_mint = Keypair::new();
        let vault_admin = Keypair::new();
        let token_mint = Keypair::new();

        self._airdrop(&vault_admin.pubkey(), 1.0).await?;
        self._create_token_mint(&token_mint).await?;

        self.initialize_vault(
            &Config::find_program_address(&jito_vault_program::id()).0,
            &vault_pubkey,
            &vault_delegation_list,
            &lrt_mint,
            &token_mint,
            &vault_admin,
            &vault_base,
            deposit_fee_bps,
            withdraw_fee_bps,
        )
        .await?;

        // for holding the backed asset in the vault
        self.create_ata(&token_mint.pubkey(), &vault_pubkey).await?;
        // for holding fees
        self.create_ata(&lrt_mint.pubkey(), &vault_admin.pubkey())
            .await?;

        Ok((
            config_admin,
            VaultRoot {
                vault_admin,
                vault_pubkey,
            },
        ))
    }

    pub async fn vault_avs_opt_in(
        &mut self,
        vault_root: &VaultRoot,
        avs: &Pubkey,
    ) -> Result<(), BanksClientError> {
        let vault_avs_ticket = VaultAvsTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            &avs,
        )
        .0;
        let avs_vault_ticket = AvsVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            &avs,
            &vault_root.vault_pubkey,
        )
        .0;
        self.add_avs(
            &Config::find_program_address(&jito_vault_program::id()).0,
            &vault_root.vault_pubkey,
            &avs,
            &avs_vault_ticket,
            &vault_avs_ticket,
            &vault_root.vault_admin,
            &self.payer.insecure_clone(),
        )
        .await?;

        Ok(())
    }

    pub async fn setup_vault_avs_slasher_operator_ticket(
        &mut self,
        vault_root: &VaultRoot,
        avs_pubkey: &Pubkey,
        slasher: &Pubkey,
        operator_pubkey: &Pubkey,
    ) -> Result<(), BanksClientError> {
        let vault_avs_slasher_ticket = VaultAvsSlasherTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            avs_pubkey,
            slasher,
        )
        .0;
        let vault_avs_slasher_operator_ticket =
            VaultAvsSlasherOperatorTicket::find_program_address(
                &jito_vault_program::id(),
                &vault_root.vault_pubkey,
                avs_pubkey,
                slasher,
                operator_pubkey,
                0, // TODO (LB): fix this
            )
            .0;
        self.initialize_vault_avs_slasher_operator_ticket(
            &Config::find_program_address(&jito_vault_program::id()).0,
            &vault_root.vault_pubkey,
            &avs_pubkey,
            &slasher,
            &operator_pubkey,
            &vault_avs_slasher_ticket,
            &vault_avs_slasher_operator_ticket,
            &self.payer.insecure_clone(),
        )
        .await
        .unwrap();

        Ok(())
    }

    pub async fn do_slash(
        &mut self,
        vault_root: &VaultRoot,
        avs_pubkey: &Pubkey,
        slasher: &Keypair,
        operator_pubkey: &Pubkey,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        let avs_operator_ticket_pubkey = AvsOperatorTicket::find_program_address(
            &jito_restaking_program::id(),
            avs_pubkey,
            operator_pubkey,
        )
        .0;
        let operator_avs_ticket_pubkey = OperatorAvsTicket::find_program_address(
            &jito_restaking_program::id(),
            operator_pubkey,
            &avs_pubkey,
        )
        .0;
        let avs_vault_ticket_pubkey = AvsVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            avs_pubkey,
            &vault_root.vault_pubkey,
        )
        .0;
        let operator_vault_ticket_pubkey = OperatorVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            operator_pubkey,
            &vault_root.vault_pubkey,
        )
        .0;
        let vault_avs_ticket_pubkey = VaultAvsTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            avs_pubkey,
        )
        .0;
        let vault_operator_ticket = VaultOperatorTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            operator_pubkey,
        )
        .0;
        let avs_slasher_ticket_pubkey = AvsVaultSlasherTicket::find_program_address(
            &jito_restaking_program::id(),
            avs_pubkey,
            &vault_root.vault_pubkey,
            &slasher.pubkey(),
        )
        .0;
        let vault_slasher_ticket_pubkey = VaultAvsSlasherTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            avs_pubkey,
            &slasher.pubkey(),
        )
        .0;
        let vault_delegate_list_pubkey = VaultDelegationList::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
        )
        .0;
        let vault_avs_slasher_operator_ticket =
            VaultAvsSlasherOperatorTicket::find_program_address(
                &jito_vault_program::id(),
                &vault_root.vault_pubkey,
                avs_pubkey,
                &slasher.pubkey(),
                operator_pubkey,
                0, // TODO (LB): fix this
            )
            .0;

        let vault = self.get_vault(&vault_root.vault_pubkey).await.unwrap();
        let vault_token_account =
            get_associated_token_address(&vault_root.vault_pubkey, &vault.supported_mint());
        let slasher_token_account =
            get_associated_token_address(&slasher.pubkey(), &vault.supported_mint());

        self.slash(
            &Config::find_program_address(&jito_vault_program::id()).0,
            &vault_root.vault_pubkey,
            &avs_pubkey,
            &operator_pubkey,
            slasher,
            &avs_operator_ticket_pubkey,
            &operator_avs_ticket_pubkey,
            &avs_vault_ticket_pubkey,
            &operator_vault_ticket_pubkey,
            &vault_avs_ticket_pubkey,
            &vault_operator_ticket,
            &avs_slasher_ticket_pubkey,
            &vault_slasher_ticket_pubkey,
            &vault_delegate_list_pubkey,
            &vault_avs_slasher_operator_ticket,
            &vault_token_account,
            &slasher_token_account,
            amount,
        )
        .await
        .unwrap();

        Ok(())
    }

    pub async fn vault_operator_opt_in(
        &mut self,
        vault_root: &VaultRoot,
        operator_pubkey: &Pubkey,
    ) -> Result<(), BanksClientError> {
        let vault_operator_ticket = VaultOperatorTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            &operator_pubkey,
        )
        .0;
        let operator_vault_ticket = OperatorVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_pubkey,
            &vault_root.vault_pubkey,
        )
        .0;
        self.add_operator(
            &Config::find_program_address(&jito_vault_program::id()).0,
            &vault_root.vault_pubkey,
            &operator_pubkey,
            &operator_vault_ticket,
            &vault_operator_ticket,
            &vault_root.vault_admin,
            &vault_root.vault_admin,
        )
        .await?;

        Ok(())
    }

    pub async fn vault_avs_vault_slasher_opt_in(
        &mut self,
        vault_root: &VaultRoot,
        avs_pubkey: &Pubkey,
        slasher: &Pubkey,
    ) -> Result<(), BanksClientError> {
        let vault_slasher_ticket_pubkey = VaultAvsSlasherTicket::find_program_address(
            &jito_vault_program::id(),
            &vault_root.vault_pubkey,
            &avs_pubkey,
            slasher,
        )
        .0;
        let avs_slasher_ticket_pubkey = AvsVaultSlasherTicket::find_program_address(
            &jito_restaking_program::id(),
            &avs_pubkey,
            &vault_root.vault_pubkey,
            slasher,
        )
        .0;

        self.add_slasher(
            &Config::find_program_address(&jito_vault_program::id()).0,
            &vault_root.vault_pubkey,
            &avs_pubkey,
            slasher,
            &avs_slasher_ticket_pubkey,
            &vault_slasher_ticket_pubkey,
            &vault_root.vault_admin,
            &vault_root.vault_admin,
        )
        .await
        .unwrap();

        Ok(())
    }

    pub async fn delegate(
        &mut self,
        vault_root: &VaultRoot,
        operator: &Pubkey,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        self.add_delegation(
            &Config::find_program_address(&jito_vault_program::id()).0,
            &vault_root.vault_pubkey,
            operator,
            &VaultOperatorTicket::find_program_address(
                &jito_vault_program::id(),
                &vault_root.vault_pubkey,
                operator,
            )
            .0,
            &VaultDelegationList::find_program_address(
                &jito_vault_program::id(),
                &vault_root.vault_pubkey,
            )
            .0,
            &vault_root.vault_admin,
            &vault_root.vault_admin,
            amount,
        )
        .await?;

        Ok(())
    }

    pub async fn initialize_vault(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        vault_delegation_list: &Pubkey,
        lrt_mint: &Keypair,
        token_mint: &Keypair,
        vault_admin: &Keypair,
        vault_base: &Keypair,
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self._process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_vault(
                &jito_vault_program::id(),
                &config,
                &vault,
                &vault_delegation_list,
                &lrt_mint.pubkey(),
                &token_mint.pubkey(),
                &vault_admin.pubkey(),
                &vault_base.pubkey(),
                deposit_fee_bps,
                withdrawal_fee_bps,
            )],
            Some(&vault_admin.pubkey()),
            &[&vault_admin, &lrt_mint, &vault_base],
            blockhash,
        ))
        .await
    }

    pub async fn add_avs(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        avs: &Pubkey,
        avs_vault_ticket: &Pubkey,
        vault_avs_ticket: &Pubkey,
        admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::add_avs(
                &jito_vault_program::id(),
                config,
                vault,
                avs,
                avs_vault_ticket,
                vault_avs_ticket,
                &admin.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[admin, payer],
            blockhash,
        ))
        .await
    }

    // pub async fn remove_avs(
    //     &mut self,
    //     config: &Pubkey,
    //     vault: &Pubkey,
    //     avs: &Pubkey,
    //     vault_avs_ticket: &Pubkey,
    //     admin: &Keypair,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[jito_vault_sdk::remove_avs(
    //             &jito_vault_program::id(),
    //             config,
    //             vault,
    //             avs,
    //             vault_avs_ticket,
    //             &admin.pubkey(),
    //         )],
    //         Some(&admin.pubkey()),
    //         &[admin],
    //         blockhash,
    //     ))
    //     .await
    // }

    pub async fn add_operator(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        operator: &Pubkey,
        operator_vault_ticket: &Pubkey,
        vault_operator_ticket: &Pubkey,
        admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::add_operator(
                &jito_vault_program::id(),
                config,
                vault,
                operator,
                operator_vault_ticket,
                vault_operator_ticket,
                &admin.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[admin, payer],
            blockhash,
        ))
        .await
    }

    pub async fn enqueue_withdraw(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        vault_delegation_list: &Pubkey,
        vault_staker_withdraw_ticket: &Pubkey,
        vault_staker_withdraw_ticket_token_account: &Pubkey,
        staker: &Keypair,
        staker_lrt_token_account: &Pubkey,
        base: &Keypair,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::enqueue_withdraw(
                &jito_vault_program::id(),
                config,
                vault,
                vault_delegation_list,
                vault_staker_withdraw_ticket,
                vault_staker_withdraw_ticket_token_account,
                &staker.pubkey(),
                staker_lrt_token_account,
                &base.pubkey(),
                amount,
            )],
            Some(&staker.pubkey()),
            &[staker, base],
            blockhash,
        ))
        .await
    }

    pub async fn add_delegation(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        operator: &Pubkey,
        vault_operator_ticket: &Pubkey,
        vault_delegation_list: &Pubkey,
        admin: &Keypair,
        payer: &Keypair,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[add_delegation(
                &jito_vault_program::id(),
                config,
                vault,
                operator,
                vault_operator_ticket,
                vault_delegation_list,
                &admin.pubkey(),
                &payer.pubkey(),
                amount,
            )],
            Some(&payer.pubkey()),
            &[admin, payer],
            blockhash,
        ))
        .await
    }

    // pub async fn remove_delegation(
    //     &mut self,
    //     config: &Pubkey,
    //     vault: &Pubkey,
    //     operator: &Pubkey,
    //     vault_delegation_list: &Pubkey,
    //     admin: &Keypair,
    //     amount: u64,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[remove_delegation(
    //             &jito_vault_program::id(),
    //             config,
    //             vault,
    //             operator,
    //             vault_delegation_list,
    //             &admin.pubkey(),
    //             amount,
    //         )],
    //         Some(&admin.pubkey()),
    //         &[admin],
    //         blockhash,
    //     ))
    //     .await
    // }

    pub async fn mint_to(
        &mut self,
        vault: &Pubkey,
        lrt_mint: &Pubkey,
        depositor: &Keypair,
        depositor_token_account: &Pubkey,
        vault_token_account: &Pubkey,
        depositor_lrt_token_account: &Pubkey,
        vault_fee_token_account: &Pubkey,
        mint_signer: Option<&Keypair>,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        let mut signers = vec![depositor];
        if let Some(signer) = mint_signer {
            signers.push(signer);
        }
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::mint_to(
                &jito_vault_program::id(),
                vault,
                lrt_mint,
                &depositor.pubkey(),
                depositor_token_account,
                vault_token_account,
                depositor_lrt_token_account,
                vault_fee_token_account,
                mint_signer.map(|s| s.pubkey()).as_ref(),
                amount,
            )],
            Some(&depositor.pubkey()),
            &signers,
            blockhash,
        ))
        .await
    }

    // pub async fn set_deposit_capacity(
    //     &mut self,
    //     vault: &Pubkey,
    //     admin: &Keypair,
    //     amount: u64,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[jito_vault_sdk::set_deposit_capacity(
    //             &jito_vault_program::id(),
    //             vault,
    //             &admin.pubkey(),
    //             amount,
    //         )],
    //         Some(&admin.pubkey()),
    //         &[admin],
    //         blockhash,
    //     ))
    //     .await
    // }

    // pub async fn set_admin(
    //     &mut self,
    //     vault: &Pubkey,
    //     old_admin: &Keypair,
    //     new_admin: &Pubkey,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[jito_vault_sdk::set_admin(
    //             &jito_vault_program::id(),
    //             vault,
    //             &old_admin.pubkey(),
    //             new_admin,
    //         )],
    //         Some(&old_admin.pubkey()),
    //         &[old_admin],
    //         blockhash,
    //     ))
    //     .await
    // }

    // pub async fn set_secondary_admin(
    //     &mut self,
    //     vault: &Pubkey,
    //     admin: &Keypair,
    //     new_admin: &Pubkey,
    //     role: VaultAdminRole,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[jito_vault_sdk::set_secondary_admin(
    //             &jito_vault_program::id(),
    //             vault,
    //             &admin.pubkey(),
    //             new_admin,
    //             role,
    //         )],
    //         Some(&admin.pubkey()),
    //         &[admin],
    //         blockhash,
    //     ))
    //     .await
    // }

    // pub async fn update_delegations(
    //     &mut self,
    //     config: &Pubkey,
    //     vault: &Pubkey,
    //     vault_delegation_list: &Pubkey,
    //     payer: &Keypair,
    // ) -> Result<(), BanksClientError> {
    //     let blockhash = self.banks_client.get_latest_blockhash().await?;
    //     self.process_transaction(&Transaction::new_signed_with_payer(
    //         &[jito_vault_sdk::update_delegations(
    //             &jito_vault_program::id(),
    //             config,
    //             vault,
    //             vault_delegation_list,
    //             &payer.pubkey(),
    //         )],
    //         Some(&payer.pubkey()),
    //         &[payer],
    //         blockhash,
    //     ))
    //     .await
    // }

    pub async fn add_slasher(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        avs: &Pubkey,
        slasher: &Pubkey,
        avs_slasher_ticket: &Pubkey,
        vault_slasher_ticket: &Pubkey,
        admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::add_slasher(
                &jito_vault_program::id(),
                config,
                vault,
                avs,
                slasher,
                avs_slasher_ticket,
                vault_slasher_ticket,
                &admin.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[admin, payer],
            blockhash,
        ))
        .await
    }

    pub async fn initialize_vault_avs_slasher_operator_ticket(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        avs: &Pubkey,
        slasher: &Pubkey,
        operator: &Pubkey,
        vault_avs_slasher_ticket: &Pubkey,
        vault_avs_slasher_operator_ticket: &Pubkey,
        payer: &Keypair,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[
                jito_vault_sdk::initialize_vault_avs_slasher_operator_ticket(
                    &jito_vault_program::id(),
                    config,
                    vault,
                    avs,
                    slasher,
                    operator,
                    vault_avs_slasher_ticket,
                    vault_avs_slasher_operator_ticket,
                    &payer.pubkey(),
                ),
            ],
            Some(&payer.pubkey()),
            &[payer],
            blockhash,
        ))
        .await
    }

    pub async fn slash(
        &mut self,
        config: &Pubkey,
        vault: &Pubkey,
        avs: &Pubkey,
        operator: &Pubkey,
        slasher: &Keypair,
        avs_operator_ticket: &Pubkey,
        operator_avs_ticket: &Pubkey,
        avs_vault_ticket: &Pubkey,
        operator_vault_ticket: &Pubkey,
        vault_avs_ticket: &Pubkey,
        vault_operator_ticket: &Pubkey,
        avs_vault_slasher_ticket: &Pubkey,
        vault_avs_slasher_ticket: &Pubkey,
        vault_delegation_list: &Pubkey,
        vault_avs_slasher_operator_ticket: &Pubkey,
        vault_token_account: &Pubkey,
        slasher_token_account: &Pubkey,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self._process_transaction(&Transaction::new_signed_with_payer(
            &[jito_vault_sdk::slash(
                &jito_vault_program::id(),
                config,
                vault,
                avs,
                operator,
                &slasher.pubkey(),
                avs_operator_ticket,
                operator_avs_ticket,
                avs_vault_ticket,
                operator_vault_ticket,
                vault_avs_ticket,
                vault_operator_ticket,
                avs_vault_slasher_ticket,
                vault_avs_slasher_ticket,
                vault_delegation_list,
                vault_avs_slasher_operator_ticket,
                vault_token_account,
                slasher_token_account,
                amount,
            )],
            Some(&slasher.pubkey()),
            &[slasher],
            blockhash,
        ))
        .await
    }

    async fn _process_transaction(&mut self, tx: &Transaction) -> Result<(), BanksClientError> {
        self.banks_client
            .process_transaction_with_preflight_and_commitment(
                tx.clone(),
                CommitmentLevel::Processed,
            )
            .await
    }

    pub async fn _airdrop(&mut self, to: &Pubkey, sol: f64) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.banks_client
            .process_transaction_with_preflight_and_commitment(
                Transaction::new_signed_with_payer(
                    &[transfer(&self.payer.pubkey(), to, sol_to_lamports(sol))],
                    Some(&self.payer.pubkey()),
                    &[&self.payer],
                    blockhash,
                ),
                CommitmentLevel::Processed,
            )
            .await
    }

    pub async fn get_token_account(
        &mut self,
        token_account: &Pubkey,
    ) -> Result<Account, BanksClientError> {
        let account = self
            .banks_client
            .get_account(*token_account)
            .await?
            .unwrap();
        Ok(Account::unpack(&account.data).unwrap())
    }

    /// Mints tokens to an ATA owned by the `to` address
    pub async fn mint_spl_to(
        &mut self,
        mint: &Pubkey,
        to: &Pubkey,
        amount: u64,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.banks_client
            .process_transaction_with_preflight_and_commitment(
                Transaction::new_signed_with_payer(
                    &[
                        create_associated_token_account_idempotent(
                            &self.payer.pubkey(),
                            to,
                            mint,
                            &spl_token::id(),
                        ),
                        spl_token::instruction::mint_to(
                            &spl_token::id(),
                            mint,
                            &get_associated_token_address(to, mint),
                            &self.payer.pubkey(),
                            &[],
                            amount,
                        )
                        .unwrap(),
                    ],
                    Some(&self.payer.pubkey()),
                    &[&self.payer],
                    blockhash,
                ),
                CommitmentLevel::Processed,
            )
            .await
    }

    async fn _create_token_mint(&mut self, mint: &Keypair) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        let rent: Rent = self.banks_client.get_sysvar().await?;
        self.banks_client
            .process_transaction_with_preflight_and_commitment(
                Transaction::new_signed_with_payer(
                    &[
                        create_account(
                            &self.payer.pubkey(),
                            &mint.pubkey(),
                            rent.minimum_balance(Mint::LEN),
                            Mint::LEN as u64,
                            &spl_token::id(),
                        ),
                        initialize_mint2(
                            &spl_token::id(),
                            &mint.pubkey(),
                            &self.payer.pubkey(),
                            None,
                            9,
                        )
                        .unwrap(),
                    ],
                    Some(&self.payer.pubkey()),
                    &[&self.payer, mint],
                    blockhash,
                ),
                CommitmentLevel::Processed,
            )
            .await
    }

    pub async fn create_ata(
        &mut self,
        mint: &Pubkey,
        owner: &Pubkey,
    ) -> Result<(), BanksClientError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.banks_client
            .process_transaction_with_preflight_and_commitment(
                Transaction::new_signed_with_payer(
                    &[create_associated_token_account_idempotent(
                        &self.payer.pubkey(),
                        owner,
                        mint,
                        &spl_token::id(),
                    )],
                    Some(&self.payer.pubkey()),
                    &[&self.payer],
                    blockhash,
                ),
                CommitmentLevel::Processed,
            )
            .await
    }
}
