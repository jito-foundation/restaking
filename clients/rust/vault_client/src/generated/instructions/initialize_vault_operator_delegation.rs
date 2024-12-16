//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>

use borsh::{BorshDeserialize, BorshSerialize};

/// Accounts.
pub struct InitializeVaultOperatorDelegation {
    pub config: solana_program::pubkey::Pubkey,

    pub vault: solana_program::pubkey::Pubkey,

    pub operator: solana_program::pubkey::Pubkey,

    pub operator_vault_ticket: solana_program::pubkey::Pubkey,

    pub vault_operator_delegation: solana_program::pubkey::Pubkey,

    pub admin: solana_program::pubkey::Pubkey,

    pub payer: solana_program::pubkey::Pubkey,

    pub system_program: solana_program::pubkey::Pubkey,
}

impl InitializeVaultOperatorDelegation {
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(&[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(8 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.config,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.vault, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.operator,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.operator_vault_ticket,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.vault_operator_delegation,
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
        let data = InitializeVaultOperatorDelegationInstructionData::new()
            .try_to_vec()
            .unwrap();

        solana_program::instruction::Instruction {
            program_id: crate::JITO_VAULT_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct InitializeVaultOperatorDelegationInstructionData {
    discriminator: u8,
}

impl InitializeVaultOperatorDelegationInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 4 }
    }
}

impl Default for InitializeVaultOperatorDelegationInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

/// Instruction builder for `InitializeVaultOperatorDelegation`.
///
/// ### Accounts:
///
///   0. `[]` config
///   1. `[writable]` vault
///   2. `[writable]` operator
///   3. `[]` operator_vault_ticket
///   4. `[writable]` vault_operator_delegation
///   5. `[signer]` admin
///   6. `[writable, signer]` payer
///   7. `[optional]` system_program (default to `11111111111111111111111111111111`)
#[derive(Clone, Debug, Default)]
pub struct InitializeVaultOperatorDelegationBuilder {
    config: Option<solana_program::pubkey::Pubkey>,
    vault: Option<solana_program::pubkey::Pubkey>,
    operator: Option<solana_program::pubkey::Pubkey>,
    operator_vault_ticket: Option<solana_program::pubkey::Pubkey>,
    vault_operator_delegation: Option<solana_program::pubkey::Pubkey>,
    admin: Option<solana_program::pubkey::Pubkey>,
    payer: Option<solana_program::pubkey::Pubkey>,
    system_program: Option<solana_program::pubkey::Pubkey>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl InitializeVaultOperatorDelegationBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    #[inline(always)]
    pub fn config(&mut self, config: solana_program::pubkey::Pubkey) -> &mut Self {
        self.config = Some(config);
        self
    }
    #[inline(always)]
    pub fn vault(&mut self, vault: solana_program::pubkey::Pubkey) -> &mut Self {
        self.vault = Some(vault);
        self
    }
    #[inline(always)]
    pub fn operator(&mut self, operator: solana_program::pubkey::Pubkey) -> &mut Self {
        self.operator = Some(operator);
        self
    }
    #[inline(always)]
    pub fn operator_vault_ticket(
        &mut self,
        operator_vault_ticket: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.operator_vault_ticket = Some(operator_vault_ticket);
        self
    }
    #[inline(always)]
    pub fn vault_operator_delegation(
        &mut self,
        vault_operator_delegation: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.vault_operator_delegation = Some(vault_operator_delegation);
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
        let accounts = InitializeVaultOperatorDelegation {
            config: self.config.expect("config is not set"),
            vault: self.vault.expect("vault is not set"),
            operator: self.operator.expect("operator is not set"),
            operator_vault_ticket: self
                .operator_vault_ticket
                .expect("operator_vault_ticket is not set"),
            vault_operator_delegation: self
                .vault_operator_delegation
                .expect("vault_operator_delegation is not set"),
            admin: self.admin.expect("admin is not set"),
            payer: self.payer.expect("payer is not set"),
            system_program: self
                .system_program
                .unwrap_or(solana_program::pubkey!("11111111111111111111111111111111")),
        };

        accounts.instruction_with_remaining_accounts(&self.__remaining_accounts)
    }
}

/// `initialize_vault_operator_delegation` CPI accounts.
pub struct InitializeVaultOperatorDelegationCpiAccounts<'a, 'b> {
    pub config: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault: &'b solana_program::account_info::AccountInfo<'a>,

    pub operator: &'b solana_program::account_info::AccountInfo<'a>,

    pub operator_vault_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault_operator_delegation: &'b solana_program::account_info::AccountInfo<'a>,

    pub admin: &'b solana_program::account_info::AccountInfo<'a>,

    pub payer: &'b solana_program::account_info::AccountInfo<'a>,

    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `initialize_vault_operator_delegation` CPI instruction.
pub struct InitializeVaultOperatorDelegationCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,

    pub config: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault: &'b solana_program::account_info::AccountInfo<'a>,

    pub operator: &'b solana_program::account_info::AccountInfo<'a>,

    pub operator_vault_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault_operator_delegation: &'b solana_program::account_info::AccountInfo<'a>,

    pub admin: &'b solana_program::account_info::AccountInfo<'a>,

    pub payer: &'b solana_program::account_info::AccountInfo<'a>,

    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
}

impl<'a, 'b> InitializeVaultOperatorDelegationCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: InitializeVaultOperatorDelegationCpiAccounts<'a, 'b>,
    ) -> Self {
        Self {
            __program: program,
            config: accounts.config,
            vault: accounts.vault,
            operator: accounts.operator,
            operator_vault_ticket: accounts.operator_vault_ticket,
            vault_operator_delegation: accounts.vault_operator_delegation,
            admin: accounts.admin,
            payer: accounts.payer,
            system_program: accounts.system_program,
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
        let mut accounts = Vec::with_capacity(8 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.config.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.vault.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.operator.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.operator_vault_ticket.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.vault_operator_delegation.key,
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
        let data = InitializeVaultOperatorDelegationInstructionData::new()
            .try_to_vec()
            .unwrap();

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::JITO_VAULT_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(8 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.config.clone());
        account_infos.push(self.vault.clone());
        account_infos.push(self.operator.clone());
        account_infos.push(self.operator_vault_ticket.clone());
        account_infos.push(self.vault_operator_delegation.clone());
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

/// Instruction builder for `InitializeVaultOperatorDelegation` via CPI.
///
/// ### Accounts:
///
///   0. `[]` config
///   1. `[writable]` vault
///   2. `[writable]` operator
///   3. `[]` operator_vault_ticket
///   4. `[writable]` vault_operator_delegation
///   5. `[signer]` admin
///   6. `[writable, signer]` payer
///   7. `[]` system_program
#[derive(Clone, Debug)]
pub struct InitializeVaultOperatorDelegationCpiBuilder<'a, 'b> {
    instruction: Box<InitializeVaultOperatorDelegationCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> InitializeVaultOperatorDelegationCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(InitializeVaultOperatorDelegationCpiBuilderInstruction {
            __program: program,
            config: None,
            vault: None,
            operator: None,
            operator_vault_ticket: None,
            vault_operator_delegation: None,
            admin: None,
            payer: None,
            system_program: None,
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
    pub fn vault(&mut self, vault: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.vault = Some(vault);
        self
    }
    #[inline(always)]
    pub fn operator(
        &mut self,
        operator: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.operator = Some(operator);
        self
    }
    #[inline(always)]
    pub fn operator_vault_ticket(
        &mut self,
        operator_vault_ticket: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.operator_vault_ticket = Some(operator_vault_ticket);
        self
    }
    #[inline(always)]
    pub fn vault_operator_delegation(
        &mut self,
        vault_operator_delegation: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.vault_operator_delegation = Some(vault_operator_delegation);
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
        let instruction = InitializeVaultOperatorDelegationCpi {
            __program: self.instruction.__program,

            config: self.instruction.config.expect("config is not set"),

            vault: self.instruction.vault.expect("vault is not set"),

            operator: self.instruction.operator.expect("operator is not set"),

            operator_vault_ticket: self
                .instruction
                .operator_vault_ticket
                .expect("operator_vault_ticket is not set"),

            vault_operator_delegation: self
                .instruction
                .vault_operator_delegation
                .expect("vault_operator_delegation is not set"),

            admin: self.instruction.admin.expect("admin is not set"),

            payer: self.instruction.payer.expect("payer is not set"),

            system_program: self
                .instruction
                .system_program
                .expect("system_program is not set"),
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct InitializeVaultOperatorDelegationCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    operator: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    operator_vault_ticket: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault_operator_delegation: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    admin: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    payer: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    system_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
