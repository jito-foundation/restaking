//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>

use borsh::{BorshDeserialize, BorshSerialize};

/// Accounts.
pub struct CrankVaultUpdateStateTracker {
    pub config: solana_program::pubkey::Pubkey,

    pub vault: solana_program::pubkey::Pubkey,

    pub operator: solana_program::pubkey::Pubkey,

    pub vault_operator_delegation: solana_program::pubkey::Pubkey,

    pub vault_update_state_tracker: solana_program::pubkey::Pubkey,
}

impl CrankVaultUpdateStateTracker {
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(&[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(5 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.config,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.vault, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.operator,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.vault_operator_delegation,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.vault_update_state_tracker,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let data = CrankVaultUpdateStateTrackerInstructionData::new()
            .try_to_vec()
            .unwrap();

        solana_program::instruction::Instruction {
            program_id: crate::JITO_VAULT_SDK_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct CrankVaultUpdateStateTrackerInstructionData {
    discriminator: u8,
}

impl CrankVaultUpdateStateTrackerInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 25 }
    }
}

impl Default for CrankVaultUpdateStateTrackerInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

/// Instruction builder for `CrankVaultUpdateStateTracker`.
///
/// ### Accounts:
///
///   0. `[]` config
///   1. `[]` vault
///   2. `[]` operator
///   3. `[writable]` vault_operator_delegation
///   4. `[writable]` vault_update_state_tracker
#[derive(Clone, Debug, Default)]
pub struct CrankVaultUpdateStateTrackerBuilder {
    config: Option<solana_program::pubkey::Pubkey>,
    vault: Option<solana_program::pubkey::Pubkey>,
    operator: Option<solana_program::pubkey::Pubkey>,
    vault_operator_delegation: Option<solana_program::pubkey::Pubkey>,
    vault_update_state_tracker: Option<solana_program::pubkey::Pubkey>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl CrankVaultUpdateStateTrackerBuilder {
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
    pub fn vault_operator_delegation(
        &mut self,
        vault_operator_delegation: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.vault_operator_delegation = Some(vault_operator_delegation);
        self
    }
    #[inline(always)]
    pub fn vault_update_state_tracker(
        &mut self,
        vault_update_state_tracker: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.vault_update_state_tracker = Some(vault_update_state_tracker);
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
        let accounts = CrankVaultUpdateStateTracker {
            config: self.config.expect("config is not set"),
            vault: self.vault.expect("vault is not set"),
            operator: self.operator.expect("operator is not set"),
            vault_operator_delegation: self
                .vault_operator_delegation
                .expect("vault_operator_delegation is not set"),
            vault_update_state_tracker: self
                .vault_update_state_tracker
                .expect("vault_update_state_tracker is not set"),
        };

        accounts.instruction_with_remaining_accounts(&self.__remaining_accounts)
    }
}

/// `crank_vault_update_state_tracker` CPI accounts.
pub struct CrankVaultUpdateStateTrackerCpiAccounts<'a, 'b> {
    pub config: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault: &'b solana_program::account_info::AccountInfo<'a>,

    pub operator: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault_operator_delegation: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault_update_state_tracker: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `crank_vault_update_state_tracker` CPI instruction.
pub struct CrankVaultUpdateStateTrackerCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,

    pub config: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault: &'b solana_program::account_info::AccountInfo<'a>,

    pub operator: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault_operator_delegation: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault_update_state_tracker: &'b solana_program::account_info::AccountInfo<'a>,
}

impl<'a, 'b> CrankVaultUpdateStateTrackerCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: CrankVaultUpdateStateTrackerCpiAccounts<'a, 'b>,
    ) -> Self {
        Self {
            __program: program,
            config: accounts.config,
            vault: accounts.vault,
            operator: accounts.operator,
            vault_operator_delegation: accounts.vault_operator_delegation,
            vault_update_state_tracker: accounts.vault_update_state_tracker,
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
        let mut accounts = Vec::with_capacity(5 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.config.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.vault.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.operator.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.vault_operator_delegation.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.vault_update_state_tracker.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let data = CrankVaultUpdateStateTrackerInstructionData::new()
            .try_to_vec()
            .unwrap();

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::JITO_VAULT_SDK_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(5 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.config.clone());
        account_infos.push(self.vault.clone());
        account_infos.push(self.operator.clone());
        account_infos.push(self.vault_operator_delegation.clone());
        account_infos.push(self.vault_update_state_tracker.clone());
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

/// Instruction builder for `CrankVaultUpdateStateTracker` via CPI.
///
/// ### Accounts:
///
///   0. `[]` config
///   1. `[]` vault
///   2. `[]` operator
///   3. `[writable]` vault_operator_delegation
///   4. `[writable]` vault_update_state_tracker
#[derive(Clone, Debug)]
pub struct CrankVaultUpdateStateTrackerCpiBuilder<'a, 'b> {
    instruction: Box<CrankVaultUpdateStateTrackerCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> CrankVaultUpdateStateTrackerCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(CrankVaultUpdateStateTrackerCpiBuilderInstruction {
            __program: program,
            config: None,
            vault: None,
            operator: None,
            vault_operator_delegation: None,
            vault_update_state_tracker: None,
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
    pub fn vault_operator_delegation(
        &mut self,
        vault_operator_delegation: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.vault_operator_delegation = Some(vault_operator_delegation);
        self
    }
    #[inline(always)]
    pub fn vault_update_state_tracker(
        &mut self,
        vault_update_state_tracker: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.vault_update_state_tracker = Some(vault_update_state_tracker);
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
        let instruction = CrankVaultUpdateStateTrackerCpi {
            __program: self.instruction.__program,

            config: self.instruction.config.expect("config is not set"),

            vault: self.instruction.vault.expect("vault is not set"),

            operator: self.instruction.operator.expect("operator is not set"),

            vault_operator_delegation: self
                .instruction
                .vault_operator_delegation
                .expect("vault_operator_delegation is not set"),

            vault_update_state_tracker: self
                .instruction
                .vault_update_state_tracker
                .expect("vault_update_state_tracker is not set"),
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct CrankVaultUpdateStateTrackerCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    operator: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault_operator_delegation: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault_update_state_tracker: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
