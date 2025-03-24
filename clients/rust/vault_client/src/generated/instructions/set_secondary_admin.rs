//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>

use borsh::{BorshDeserialize, BorshSerialize};

use crate::generated::types::VaultAdminRole;

/// Accounts.
pub struct SetSecondaryAdmin {
    pub config: solana_program::pubkey::Pubkey,

    pub vault: solana_program::pubkey::Pubkey,

    pub admin: solana_program::pubkey::Pubkey,

    pub new_admin: solana_program::pubkey::Pubkey,
}

impl SetSecondaryAdmin {
    pub fn instruction(
        &self,
        args: SetSecondaryAdminInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: SetSecondaryAdminInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(4 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.config,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.vault, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.admin, true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.new_admin,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = SetSecondaryAdminInstructionData::new()
            .try_to_vec()
            .unwrap();
        let mut args = args.try_to_vec().unwrap();
        data.append(&mut args);

        solana_program::instruction::Instruction {
            program_id: crate::JITO_VAULT_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SetSecondaryAdminInstructionData {
    discriminator: u8,
}

impl SetSecondaryAdminInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 22 }
    }
}

impl Default for SetSecondaryAdminInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SetSecondaryAdminInstructionArgs {
    pub vault_admin_role: VaultAdminRole,
}

/// Instruction builder for `SetSecondaryAdmin`.
///
/// ### Accounts:
///
///   0. `[]` config
///   1. `[writable]` vault
///   2. `[signer]` admin
///   3. `[]` new_admin
#[derive(Clone, Debug, Default)]
pub struct SetSecondaryAdminBuilder {
    config: Option<solana_program::pubkey::Pubkey>,
    vault: Option<solana_program::pubkey::Pubkey>,
    admin: Option<solana_program::pubkey::Pubkey>,
    new_admin: Option<solana_program::pubkey::Pubkey>,
    vault_admin_role: Option<VaultAdminRole>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl SetSecondaryAdminBuilder {
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
    pub fn admin(&mut self, admin: solana_program::pubkey::Pubkey) -> &mut Self {
        self.admin = Some(admin);
        self
    }
    #[inline(always)]
    pub fn new_admin(&mut self, new_admin: solana_program::pubkey::Pubkey) -> &mut Self {
        self.new_admin = Some(new_admin);
        self
    }
    #[inline(always)]
    pub fn vault_admin_role(&mut self, vault_admin_role: VaultAdminRole) -> &mut Self {
        self.vault_admin_role = Some(vault_admin_role);
        self
    }
    /// Add an additional account to the instruction.
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
        let accounts = SetSecondaryAdmin {
            config: self.config.expect("config is not set"),
            vault: self.vault.expect("vault is not set"),
            admin: self.admin.expect("admin is not set"),
            new_admin: self.new_admin.expect("new_admin is not set"),
        };
        let args = SetSecondaryAdminInstructionArgs {
            vault_admin_role: self
                .vault_admin_role
                .clone()
                .expect("vault_admin_role is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `set_secondary_admin` CPI accounts.
pub struct SetSecondaryAdminCpiAccounts<'a, 'b> {
    pub config: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault: &'b solana_program::account_info::AccountInfo<'a>,

    pub admin: &'b solana_program::account_info::AccountInfo<'a>,

    pub new_admin: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `set_secondary_admin` CPI instruction.
pub struct SetSecondaryAdminCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,

    pub config: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault: &'b solana_program::account_info::AccountInfo<'a>,

    pub admin: &'b solana_program::account_info::AccountInfo<'a>,

    pub new_admin: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: SetSecondaryAdminInstructionArgs,
}

impl<'a, 'b> SetSecondaryAdminCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: SetSecondaryAdminCpiAccounts<'a, 'b>,
        args: SetSecondaryAdminInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            config: accounts.config,
            vault: accounts.vault,
            admin: accounts.admin,
            new_admin: accounts.new_admin,
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
        let mut accounts = Vec::with_capacity(4 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.config.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.vault.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.admin.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.new_admin.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = SetSecondaryAdminInstructionData::new()
            .try_to_vec()
            .unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::JITO_VAULT_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(4 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.config.clone());
        account_infos.push(self.vault.clone());
        account_infos.push(self.admin.clone());
        account_infos.push(self.new_admin.clone());
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

/// Instruction builder for `SetSecondaryAdmin` via CPI.
///
/// ### Accounts:
///
///   0. `[]` config
///   1. `[writable]` vault
///   2. `[signer]` admin
///   3. `[]` new_admin
#[derive(Clone, Debug)]
pub struct SetSecondaryAdminCpiBuilder<'a, 'b> {
    instruction: Box<SetSecondaryAdminCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> SetSecondaryAdminCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(SetSecondaryAdminCpiBuilderInstruction {
            __program: program,
            config: None,
            vault: None,
            admin: None,
            new_admin: None,
            vault_admin_role: None,
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
    pub fn admin(&mut self, admin: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.admin = Some(admin);
        self
    }
    #[inline(always)]
    pub fn new_admin(
        &mut self,
        new_admin: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.new_admin = Some(new_admin);
        self
    }
    #[inline(always)]
    pub fn vault_admin_role(&mut self, vault_admin_role: VaultAdminRole) -> &mut Self {
        self.instruction.vault_admin_role = Some(vault_admin_role);
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
        let args = SetSecondaryAdminInstructionArgs {
            vault_admin_role: self
                .instruction
                .vault_admin_role
                .clone()
                .expect("vault_admin_role is not set"),
        };
        let instruction = SetSecondaryAdminCpi {
            __program: self.instruction.__program,

            config: self.instruction.config.expect("config is not set"),

            vault: self.instruction.vault.expect("vault is not set"),

            admin: self.instruction.admin.expect("admin is not set"),

            new_admin: self.instruction.new_admin.expect("new_admin is not set"),
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct SetSecondaryAdminCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    admin: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    new_admin: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault_admin_role: Option<VaultAdminRole>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
