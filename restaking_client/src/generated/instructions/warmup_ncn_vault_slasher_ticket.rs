//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use borsh::BorshDeserialize;
use borsh::BorshSerialize;

/// Accounts.
pub struct WarmupNcnVaultSlasherTicket {
    pub config: solana_program::pubkey::Pubkey,

    pub ncn: solana_program::pubkey::Pubkey,

    pub vault: solana_program::pubkey::Pubkey,

    pub slasher: solana_program::pubkey::Pubkey,

    pub ncn_vault_ticket: solana_program::pubkey::Pubkey,

    pub ncn_vault_slasher_ticket: solana_program::pubkey::Pubkey,

    pub admin: solana_program::pubkey::Pubkey,
}

impl WarmupNcnVaultSlasherTicket {
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(&[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(7 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.config,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.ncn, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.vault, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.slasher,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.ncn_vault_ticket,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.ncn_vault_slasher_ticket,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.admin, true,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let data = WarmupNcnVaultSlasherTicketInstructionData::new()
            .try_to_vec()
            .unwrap();

        solana_program::instruction::Instruction {
            program_id: crate::JITO_RESTAKING_SDK_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct WarmupNcnVaultSlasherTicketInstructionData {
    discriminator: u8,
}

impl WarmupNcnVaultSlasherTicketInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 13 }
    }
}

impl Default for WarmupNcnVaultSlasherTicketInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

/// Instruction builder for `WarmupNcnVaultSlasherTicket`.
///
/// ### Accounts:
///
///   0. `[]` config
///   1. `[]` ncn
///   2. `[]` vault
///   3. `[]` slasher
///   4. `[]` ncn_vault_ticket
///   5. `[writable]` ncn_vault_slasher_ticket
///   6. `[signer]` admin
#[derive(Clone, Debug, Default)]
pub struct WarmupNcnVaultSlasherTicketBuilder {
    config: Option<solana_program::pubkey::Pubkey>,
    ncn: Option<solana_program::pubkey::Pubkey>,
    vault: Option<solana_program::pubkey::Pubkey>,
    slasher: Option<solana_program::pubkey::Pubkey>,
    ncn_vault_ticket: Option<solana_program::pubkey::Pubkey>,
    ncn_vault_slasher_ticket: Option<solana_program::pubkey::Pubkey>,
    admin: Option<solana_program::pubkey::Pubkey>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl WarmupNcnVaultSlasherTicketBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    #[inline(always)]
    pub fn config(&mut self, config: solana_program::pubkey::Pubkey) -> &mut Self {
        self.config = Some(config);
        self
    }
    #[inline(always)]
    pub fn ncn(&mut self, ncn: solana_program::pubkey::Pubkey) -> &mut Self {
        self.ncn = Some(ncn);
        self
    }
    #[inline(always)]
    pub fn vault(&mut self, vault: solana_program::pubkey::Pubkey) -> &mut Self {
        self.vault = Some(vault);
        self
    }
    #[inline(always)]
    pub fn slasher(&mut self, slasher: solana_program::pubkey::Pubkey) -> &mut Self {
        self.slasher = Some(slasher);
        self
    }
    #[inline(always)]
    pub fn ncn_vault_ticket(
        &mut self,
        ncn_vault_ticket: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.ncn_vault_ticket = Some(ncn_vault_ticket);
        self
    }
    #[inline(always)]
    pub fn ncn_vault_slasher_ticket(
        &mut self,
        ncn_vault_slasher_ticket: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.ncn_vault_slasher_ticket = Some(ncn_vault_slasher_ticket);
        self
    }
    #[inline(always)]
    pub fn admin(&mut self, admin: solana_program::pubkey::Pubkey) -> &mut Self {
        self.admin = Some(admin);
        self
    }
    /// Add an aditional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: solana_program::instruction::AccountMeta,
    ) -> &mut Self {
        self.__remaining_accounts.push(account);
        self
    }
    /// Add additional accounts to the instruction.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[solana_program::instruction::AccountMeta],
    ) -> &mut Self {
        self.__remaining_accounts.extend_from_slice(accounts);
        self
    }
    #[allow(clippy::clone_on_copy)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let accounts = WarmupNcnVaultSlasherTicket {
            config: self.config.expect("config is not set"),
            ncn: self.ncn.expect("ncn is not set"),
            vault: self.vault.expect("vault is not set"),
            slasher: self.slasher.expect("slasher is not set"),
            ncn_vault_ticket: self.ncn_vault_ticket.expect("ncn_vault_ticket is not set"),
            ncn_vault_slasher_ticket: self
                .ncn_vault_slasher_ticket
                .expect("ncn_vault_slasher_ticket is not set"),
            admin: self.admin.expect("admin is not set"),
        };

        accounts.instruction_with_remaining_accounts(&self.__remaining_accounts)
    }
}

/// `warmup_ncn_vault_slasher_ticket` CPI accounts.
pub struct WarmupNcnVaultSlasherTicketCpiAccounts<'a, 'b> {
    pub config: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault: &'b solana_program::account_info::AccountInfo<'a>,

    pub slasher: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn_vault_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn_vault_slasher_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub admin: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `warmup_ncn_vault_slasher_ticket` CPI instruction.
pub struct WarmupNcnVaultSlasherTicketCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,

    pub config: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault: &'b solana_program::account_info::AccountInfo<'a>,

    pub slasher: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn_vault_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn_vault_slasher_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub admin: &'b solana_program::account_info::AccountInfo<'a>,
}

impl<'a, 'b> WarmupNcnVaultSlasherTicketCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: WarmupNcnVaultSlasherTicketCpiAccounts<'a, 'b>,
    ) -> Self {
        Self {
            __program: program,
            config: accounts.config,
            ncn: accounts.ncn,
            vault: accounts.vault,
            slasher: accounts.slasher,
            ncn_vault_ticket: accounts.ncn_vault_ticket,
            ncn_vault_slasher_ticket: accounts.ncn_vault_slasher_ticket,
            admin: accounts.admin,
        }
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], &[])
    }
    #[inline(always)]
    pub fn invoke_with_remaining_accounts(
        &self,
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], remaining_accounts)
    }
    #[inline(always)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(signers_seeds, &[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed_with_remaining_accounts(
        &self,
        signers_seeds: &[&[&[u8]]],
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        let mut accounts = Vec::with_capacity(7 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.config.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.ncn.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.vault.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.slasher.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.ncn_vault_ticket.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.ncn_vault_slasher_ticket.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.admin.key,
            true,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let data = WarmupNcnVaultSlasherTicketInstructionData::new()
            .try_to_vec()
            .unwrap();

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::JITO_RESTAKING_SDK_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(7 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.config.clone());
        account_infos.push(self.ncn.clone());
        account_infos.push(self.vault.clone());
        account_infos.push(self.slasher.clone());
        account_infos.push(self.ncn_vault_ticket.clone());
        account_infos.push(self.ncn_vault_slasher_ticket.clone());
        account_infos.push(self.admin.clone());
        remaining_accounts
            .iter()
            .for_each(|remaining_account| account_infos.push(remaining_account.0.clone()));

        if signers_seeds.is_empty() {
            solana_program::program::invoke(&instruction, &account_infos)
        } else {
            solana_program::program::invoke_signed(&instruction, &account_infos, signers_seeds)
        }
    }
}

/// Instruction builder for `WarmupNcnVaultSlasherTicket` via CPI.
///
/// ### Accounts:
///
///   0. `[]` config
///   1. `[]` ncn
///   2. `[]` vault
///   3. `[]` slasher
///   4. `[]` ncn_vault_ticket
///   5. `[writable]` ncn_vault_slasher_ticket
///   6. `[signer]` admin
#[derive(Clone, Debug)]
pub struct WarmupNcnVaultSlasherTicketCpiBuilder<'a, 'b> {
    instruction: Box<WarmupNcnVaultSlasherTicketCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> WarmupNcnVaultSlasherTicketCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(WarmupNcnVaultSlasherTicketCpiBuilderInstruction {
            __program: program,
            config: None,
            ncn: None,
            vault: None,
            slasher: None,
            ncn_vault_ticket: None,
            ncn_vault_slasher_ticket: None,
            admin: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }
    #[inline(always)]
    pub fn config(
        &mut self,
        config: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.config = Some(config);
        self
    }
    #[inline(always)]
    pub fn ncn(&mut self, ncn: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.ncn = Some(ncn);
        self
    }
    #[inline(always)]
    pub fn vault(&mut self, vault: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.vault = Some(vault);
        self
    }
    #[inline(always)]
    pub fn slasher(
        &mut self,
        slasher: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.slasher = Some(slasher);
        self
    }
    #[inline(always)]
    pub fn ncn_vault_ticket(
        &mut self,
        ncn_vault_ticket: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.ncn_vault_ticket = Some(ncn_vault_ticket);
        self
    }
    #[inline(always)]
    pub fn ncn_vault_slasher_ticket(
        &mut self,
        ncn_vault_slasher_ticket: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.ncn_vault_slasher_ticket = Some(ncn_vault_slasher_ticket);
        self
    }
    #[inline(always)]
    pub fn admin(&mut self, admin: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.admin = Some(admin);
        self
    }
    /// Add an additional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: &'b solana_program::account_info::AccountInfo<'a>,
        is_writable: bool,
        is_signer: bool,
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .push((account, is_writable, is_signer));
        self
    }
    /// Add additional accounts to the instruction.
    ///
    /// Each account is represented by a tuple of the `AccountInfo`, a `bool` indicating whether the account is writable or not,
    /// and a `bool` indicating whether the account is a signer or not.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .extend_from_slice(accounts);
        self
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed(&[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint::ProgramResult {
        let instruction = WarmupNcnVaultSlasherTicketCpi {
            __program: self.instruction.__program,

            config: self.instruction.config.expect("config is not set"),

            ncn: self.instruction.ncn.expect("ncn is not set"),

            vault: self.instruction.vault.expect("vault is not set"),

            slasher: self.instruction.slasher.expect("slasher is not set"),

            ncn_vault_ticket: self
                .instruction
                .ncn_vault_ticket
                .expect("ncn_vault_ticket is not set"),

            ncn_vault_slasher_ticket: self
                .instruction
                .ncn_vault_slasher_ticket
                .expect("ncn_vault_slasher_ticket is not set"),

            admin: self.instruction.admin.expect("admin is not set"),
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct WarmupNcnVaultSlasherTicketCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    ncn: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    slasher: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    ncn_vault_ticket: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    ncn_vault_slasher_ticket: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    admin: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
