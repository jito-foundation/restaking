//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>

use borsh::{BorshDeserialize, BorshSerialize};

/// Accounts.
pub struct InitializeNcnVaultSlasherTicket {
    pub config: solana_program::pubkey::Pubkey,

    pub ncn: solana_program::pubkey::Pubkey,

    pub vault: solana_program::pubkey::Pubkey,

    pub slasher: solana_program::pubkey::Pubkey,

    pub ncn_vault_ticket: solana_program::pubkey::Pubkey,

    pub ncn_vault_slasher_ticket: solana_program::pubkey::Pubkey,

    pub admin: solana_program::pubkey::Pubkey,

    pub payer: solana_program::pubkey::Pubkey,

    pub system_program: solana_program::pubkey::Pubkey,
}

impl InitializeNcnVaultSlasherTicket {
    pub fn instruction(
        &self,
        args: InitializeNcnVaultSlasherTicketInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: InitializeNcnVaultSlasherTicketInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(9 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.config,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
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
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.payer, true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.system_program,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = InitializeNcnVaultSlasherTicketInstructionData::new()
            .try_to_vec()
            .unwrap();
        let mut args = args.try_to_vec().unwrap();
        data.append(&mut args);

        solana_program::instruction::Instruction {
            program_id: crate::JITO_RESTAKING_PROGRAM_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct InitializeNcnVaultSlasherTicketInstructionData {
    discriminator: u8,
}

impl InitializeNcnVaultSlasherTicketInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 3 }
    }
}

impl Default for InitializeNcnVaultSlasherTicketInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InitializeNcnVaultSlasherTicketInstructionArgs {
    pub args: u64,
}

/// Instruction builder for `InitializeNcnVaultSlasherTicket`.
///
/// ### Accounts:
///
///   0. `[]` config
///   1. `[writable]` ncn
///   2. `[]` vault
///   3. `[]` slasher
///   4. `[]` ncn_vault_ticket
///   5. `[writable]` ncn_vault_slasher_ticket
///   6. `[signer]` admin
///   7. `[writable, signer]` payer
///   8. `[optional]` system_program (default to `11111111111111111111111111111111`)
#[derive(Clone, Debug, Default)]
pub struct InitializeNcnVaultSlasherTicketBuilder {
    config: Option<solana_program::pubkey::Pubkey>,
    ncn: Option<solana_program::pubkey::Pubkey>,
    vault: Option<solana_program::pubkey::Pubkey>,
    slasher: Option<solana_program::pubkey::Pubkey>,
    ncn_vault_ticket: Option<solana_program::pubkey::Pubkey>,
    ncn_vault_slasher_ticket: Option<solana_program::pubkey::Pubkey>,
    admin: Option<solana_program::pubkey::Pubkey>,
    payer: Option<solana_program::pubkey::Pubkey>,
    system_program: Option<solana_program::pubkey::Pubkey>,
    args: Option<u64>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl InitializeNcnVaultSlasherTicketBuilder {
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
    #[inline(always)]
    pub fn payer(&mut self, payer: solana_program::pubkey::Pubkey) -> &mut Self {
        self.payer = Some(payer);
        self
    }
    /// `[optional account, default to '11111111111111111111111111111111']`
    #[inline(always)]
    pub fn system_program(&mut self, system_program: solana_program::pubkey::Pubkey) -> &mut Self {
        self.system_program = Some(system_program);
        self
    }
    #[inline(always)]
    pub fn args(&mut self, args: u64) -> &mut Self {
        self.args = Some(args);
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
        let accounts = InitializeNcnVaultSlasherTicket {
            config: self.config.expect("config is not set"),
            ncn: self.ncn.expect("ncn is not set"),
            vault: self.vault.expect("vault is not set"),
            slasher: self.slasher.expect("slasher is not set"),
            ncn_vault_ticket: self.ncn_vault_ticket.expect("ncn_vault_ticket is not set"),
            ncn_vault_slasher_ticket: self
                .ncn_vault_slasher_ticket
                .expect("ncn_vault_slasher_ticket is not set"),
            admin: self.admin.expect("admin is not set"),
            payer: self.payer.expect("payer is not set"),
            system_program: self
                .system_program
                .unwrap_or(solana_program::pubkey!("11111111111111111111111111111111")),
        };
        let args = InitializeNcnVaultSlasherTicketInstructionArgs {
            args: self.args.clone().expect("args is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `initialize_ncn_vault_slasher_ticket` CPI accounts.
pub struct InitializeNcnVaultSlasherTicketCpiAccounts<'a, 'b> {
    pub config: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault: &'b solana_program::account_info::AccountInfo<'a>,

    pub slasher: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn_vault_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn_vault_slasher_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub admin: &'b solana_program::account_info::AccountInfo<'a>,

    pub payer: &'b solana_program::account_info::AccountInfo<'a>,

    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `initialize_ncn_vault_slasher_ticket` CPI instruction.
pub struct InitializeNcnVaultSlasherTicketCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,

    pub config: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault: &'b solana_program::account_info::AccountInfo<'a>,

    pub slasher: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn_vault_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn_vault_slasher_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub admin: &'b solana_program::account_info::AccountInfo<'a>,

    pub payer: &'b solana_program::account_info::AccountInfo<'a>,

    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: InitializeNcnVaultSlasherTicketInstructionArgs,
}

impl<'a, 'b> InitializeNcnVaultSlasherTicketCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: InitializeNcnVaultSlasherTicketCpiAccounts<'a, 'b>,
        args: InitializeNcnVaultSlasherTicketInstructionArgs,
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
            payer: accounts.payer,
            system_program: accounts.system_program,
            __args: args,
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
        let mut accounts = Vec::with_capacity(9 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.config.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
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
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.payer.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.system_program.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = InitializeNcnVaultSlasherTicketInstructionData::new()
            .try_to_vec()
            .unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::JITO_RESTAKING_PROGRAM_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(9 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.config.clone());
        account_infos.push(self.ncn.clone());
        account_infos.push(self.vault.clone());
        account_infos.push(self.slasher.clone());
        account_infos.push(self.ncn_vault_ticket.clone());
        account_infos.push(self.ncn_vault_slasher_ticket.clone());
        account_infos.push(self.admin.clone());
        account_infos.push(self.payer.clone());
        account_infos.push(self.system_program.clone());
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

/// Instruction builder for `InitializeNcnVaultSlasherTicket` via CPI.
///
/// ### Accounts:
///
///   0. `[]` config
///   1. `[writable]` ncn
///   2. `[]` vault
///   3. `[]` slasher
///   4. `[]` ncn_vault_ticket
///   5. `[writable]` ncn_vault_slasher_ticket
///   6. `[signer]` admin
///   7. `[writable, signer]` payer
///   8. `[]` system_program
#[derive(Clone, Debug)]
pub struct InitializeNcnVaultSlasherTicketCpiBuilder<'a, 'b> {
    instruction: Box<InitializeNcnVaultSlasherTicketCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> InitializeNcnVaultSlasherTicketCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(InitializeNcnVaultSlasherTicketCpiBuilderInstruction {
            __program: program,
            config: None,
            ncn: None,
            vault: None,
            slasher: None,
            ncn_vault_ticket: None,
            ncn_vault_slasher_ticket: None,
            admin: None,
            payer: None,
            system_program: None,
            args: None,
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
    #[inline(always)]
    pub fn payer(&mut self, payer: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.payer = Some(payer);
        self
    }
    #[inline(always)]
    pub fn system_program(
        &mut self,
        system_program: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.system_program = Some(system_program);
        self
    }
    #[inline(always)]
    pub fn args(&mut self, args: u64) -> &mut Self {
        self.instruction.args = Some(args);
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
        let args = InitializeNcnVaultSlasherTicketInstructionArgs {
            args: self.instruction.args.clone().expect("args is not set"),
        };
        let instruction = InitializeNcnVaultSlasherTicketCpi {
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

            payer: self.instruction.payer.expect("payer is not set"),

            system_program: self
                .instruction
                .system_program
                .expect("system_program is not set"),
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct InitializeNcnVaultSlasherTicketCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    ncn: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    slasher: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    ncn_vault_ticket: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    ncn_vault_slasher_ticket: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    admin: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    payer: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    system_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    args: Option<u64>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
