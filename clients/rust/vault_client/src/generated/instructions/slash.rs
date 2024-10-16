//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>

use borsh::{BorshDeserialize, BorshSerialize};

/// Accounts.
pub struct Slash {
    pub config: solana_program::pubkey::Pubkey,

    pub vault: solana_program::pubkey::Pubkey,

    pub ncn: solana_program::pubkey::Pubkey,

    pub operator: solana_program::pubkey::Pubkey,

    pub slasher: solana_program::pubkey::Pubkey,

    pub ncn_operator_state: solana_program::pubkey::Pubkey,

    pub ncn_vault_ticket: solana_program::pubkey::Pubkey,

    pub operator_vault_ticket: solana_program::pubkey::Pubkey,

    pub vault_ncn_ticket: solana_program::pubkey::Pubkey,

    pub vault_operator_delegation: solana_program::pubkey::Pubkey,

    pub ncn_vault_slasher_ticket: solana_program::pubkey::Pubkey,

    pub vault_ncn_slasher_ticket: solana_program::pubkey::Pubkey,

    pub vault_ncn_slasher_operator_ticket: solana_program::pubkey::Pubkey,

    pub vault_token_account: solana_program::pubkey::Pubkey,

    pub slasher_token_account: solana_program::pubkey::Pubkey,

    pub token_program: solana_program::pubkey::Pubkey,
}

impl Slash {
    pub fn instruction(
        &self,
        args: SlashInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: SlashInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(16 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.config,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.vault, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.ncn, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.operator,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.slasher,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.ncn_operator_state,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.ncn_vault_ticket,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.operator_vault_ticket,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.vault_ncn_ticket,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.vault_operator_delegation,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.ncn_vault_slasher_ticket,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.vault_ncn_slasher_ticket,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.vault_ncn_slasher_operator_ticket,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.vault_token_account,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.slasher_token_account,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.token_program,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = SlashInstructionData::new().try_to_vec().unwrap();
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
pub struct SlashInstructionData {
    discriminator: u8,
}

impl SlashInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 32 }
    }
}

impl Default for SlashInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SlashInstructionArgs {
    pub amount: u64,
}

/// Instruction builder for `Slash`.
///
/// ### Accounts:
///
///   0. `[]` config
///   1. `[writable]` vault
///   2. `[]` ncn
///   3. `[]` operator
///   4. `[signer]` slasher
///   5. `[]` ncn_operator_state
///   6. `[]` ncn_vault_ticket
///   7. `[]` operator_vault_ticket
///   8. `[]` vault_ncn_ticket
///   9. `[writable]` vault_operator_delegation
///   10. `[]` ncn_vault_slasher_ticket
///   11. `[]` vault_ncn_slasher_ticket
///   12. `[writable]` vault_ncn_slasher_operator_ticket
///   13. `[writable]` vault_token_account
///   14. `[]` slasher_token_account
///   15. `[optional]` token_program (default to `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`)
#[derive(Clone, Debug, Default)]
pub struct SlashBuilder {
    config: Option<solana_program::pubkey::Pubkey>,
    vault: Option<solana_program::pubkey::Pubkey>,
    ncn: Option<solana_program::pubkey::Pubkey>,
    operator: Option<solana_program::pubkey::Pubkey>,
    slasher: Option<solana_program::pubkey::Pubkey>,
    ncn_operator_state: Option<solana_program::pubkey::Pubkey>,
    ncn_vault_ticket: Option<solana_program::pubkey::Pubkey>,
    operator_vault_ticket: Option<solana_program::pubkey::Pubkey>,
    vault_ncn_ticket: Option<solana_program::pubkey::Pubkey>,
    vault_operator_delegation: Option<solana_program::pubkey::Pubkey>,
    ncn_vault_slasher_ticket: Option<solana_program::pubkey::Pubkey>,
    vault_ncn_slasher_ticket: Option<solana_program::pubkey::Pubkey>,
    vault_ncn_slasher_operator_ticket: Option<solana_program::pubkey::Pubkey>,
    vault_token_account: Option<solana_program::pubkey::Pubkey>,
    slasher_token_account: Option<solana_program::pubkey::Pubkey>,
    token_program: Option<solana_program::pubkey::Pubkey>,
    amount: Option<u64>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl SlashBuilder {
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
    pub fn ncn(&mut self, ncn: solana_program::pubkey::Pubkey) -> &mut Self {
        self.ncn = Some(ncn);
        self
    }
    #[inline(always)]
    pub fn operator(&mut self, operator: solana_program::pubkey::Pubkey) -> &mut Self {
        self.operator = Some(operator);
        self
    }
    #[inline(always)]
    pub fn slasher(&mut self, slasher: solana_program::pubkey::Pubkey) -> &mut Self {
        self.slasher = Some(slasher);
        self
    }
    #[inline(always)]
    pub fn ncn_operator_state(
        &mut self,
        ncn_operator_state: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.ncn_operator_state = Some(ncn_operator_state);
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
    pub fn operator_vault_ticket(
        &mut self,
        operator_vault_ticket: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.operator_vault_ticket = Some(operator_vault_ticket);
        self
    }
    #[inline(always)]
    pub fn vault_ncn_ticket(
        &mut self,
        vault_ncn_ticket: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.vault_ncn_ticket = Some(vault_ncn_ticket);
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
    pub fn ncn_vault_slasher_ticket(
        &mut self,
        ncn_vault_slasher_ticket: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.ncn_vault_slasher_ticket = Some(ncn_vault_slasher_ticket);
        self
    }
    #[inline(always)]
    pub fn vault_ncn_slasher_ticket(
        &mut self,
        vault_ncn_slasher_ticket: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.vault_ncn_slasher_ticket = Some(vault_ncn_slasher_ticket);
        self
    }
    #[inline(always)]
    pub fn vault_ncn_slasher_operator_ticket(
        &mut self,
        vault_ncn_slasher_operator_ticket: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.vault_ncn_slasher_operator_ticket = Some(vault_ncn_slasher_operator_ticket);
        self
    }
    #[inline(always)]
    pub fn vault_token_account(
        &mut self,
        vault_token_account: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.vault_token_account = Some(vault_token_account);
        self
    }
    #[inline(always)]
    pub fn slasher_token_account(
        &mut self,
        slasher_token_account: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.slasher_token_account = Some(slasher_token_account);
        self
    }
    /// `[optional account, default to 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA']`
    #[inline(always)]
    pub fn token_program(&mut self, token_program: solana_program::pubkey::Pubkey) -> &mut Self {
        self.token_program = Some(token_program);
        self
    }
    #[inline(always)]
    pub fn amount(&mut self, amount: u64) -> &mut Self {
        self.amount = Some(amount);
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
        let accounts = Slash {
            config: self.config.expect("config is not set"),
            vault: self.vault.expect("vault is not set"),
            ncn: self.ncn.expect("ncn is not set"),
            operator: self.operator.expect("operator is not set"),
            slasher: self.slasher.expect("slasher is not set"),
            ncn_operator_state: self
                .ncn_operator_state
                .expect("ncn_operator_state is not set"),
            ncn_vault_ticket: self.ncn_vault_ticket.expect("ncn_vault_ticket is not set"),
            operator_vault_ticket: self
                .operator_vault_ticket
                .expect("operator_vault_ticket is not set"),
            vault_ncn_ticket: self.vault_ncn_ticket.expect("vault_ncn_ticket is not set"),
            vault_operator_delegation: self
                .vault_operator_delegation
                .expect("vault_operator_delegation is not set"),
            ncn_vault_slasher_ticket: self
                .ncn_vault_slasher_ticket
                .expect("ncn_vault_slasher_ticket is not set"),
            vault_ncn_slasher_ticket: self
                .vault_ncn_slasher_ticket
                .expect("vault_ncn_slasher_ticket is not set"),
            vault_ncn_slasher_operator_ticket: self
                .vault_ncn_slasher_operator_ticket
                .expect("vault_ncn_slasher_operator_ticket is not set"),
            vault_token_account: self
                .vault_token_account
                .expect("vault_token_account is not set"),
            slasher_token_account: self
                .slasher_token_account
                .expect("slasher_token_account is not set"),
            token_program: self.token_program.unwrap_or(solana_program::pubkey!(
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            )),
        };
        let args = SlashInstructionArgs {
            amount: self.amount.clone().expect("amount is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `slash` CPI accounts.
pub struct SlashCpiAccounts<'a, 'b> {
    pub config: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn: &'b solana_program::account_info::AccountInfo<'a>,

    pub operator: &'b solana_program::account_info::AccountInfo<'a>,

    pub slasher: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn_operator_state: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn_vault_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub operator_vault_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault_ncn_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault_operator_delegation: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn_vault_slasher_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault_ncn_slasher_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault_ncn_slasher_operator_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault_token_account: &'b solana_program::account_info::AccountInfo<'a>,

    pub slasher_token_account: &'b solana_program::account_info::AccountInfo<'a>,

    pub token_program: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `slash` CPI instruction.
pub struct SlashCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,

    pub config: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn: &'b solana_program::account_info::AccountInfo<'a>,

    pub operator: &'b solana_program::account_info::AccountInfo<'a>,

    pub slasher: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn_operator_state: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn_vault_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub operator_vault_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault_ncn_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault_operator_delegation: &'b solana_program::account_info::AccountInfo<'a>,

    pub ncn_vault_slasher_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault_ncn_slasher_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault_ncn_slasher_operator_ticket: &'b solana_program::account_info::AccountInfo<'a>,

    pub vault_token_account: &'b solana_program::account_info::AccountInfo<'a>,

    pub slasher_token_account: &'b solana_program::account_info::AccountInfo<'a>,

    pub token_program: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: SlashInstructionArgs,
}

impl<'a, 'b> SlashCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: SlashCpiAccounts<'a, 'b>,
        args: SlashInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            config: accounts.config,
            vault: accounts.vault,
            ncn: accounts.ncn,
            operator: accounts.operator,
            slasher: accounts.slasher,
            ncn_operator_state: accounts.ncn_operator_state,
            ncn_vault_ticket: accounts.ncn_vault_ticket,
            operator_vault_ticket: accounts.operator_vault_ticket,
            vault_ncn_ticket: accounts.vault_ncn_ticket,
            vault_operator_delegation: accounts.vault_operator_delegation,
            ncn_vault_slasher_ticket: accounts.ncn_vault_slasher_ticket,
            vault_ncn_slasher_ticket: accounts.vault_ncn_slasher_ticket,
            vault_ncn_slasher_operator_ticket: accounts.vault_ncn_slasher_operator_ticket,
            vault_token_account: accounts.vault_token_account,
            slasher_token_account: accounts.slasher_token_account,
            token_program: accounts.token_program,
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
        let mut accounts = Vec::with_capacity(16 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.config.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.vault.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.ncn.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.operator.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.slasher.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.ncn_operator_state.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.ncn_vault_ticket.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.operator_vault_ticket.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.vault_ncn_ticket.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.vault_operator_delegation.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.ncn_vault_slasher_ticket.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.vault_ncn_slasher_ticket.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.vault_ncn_slasher_operator_ticket.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.vault_token_account.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.slasher_token_account.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.token_program.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = SlashInstructionData::new().try_to_vec().unwrap();
        let mut args = self.__args.try_to_vec().unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::JITO_VAULT_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(16 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.config.clone());
        account_infos.push(self.vault.clone());
        account_infos.push(self.ncn.clone());
        account_infos.push(self.operator.clone());
        account_infos.push(self.slasher.clone());
        account_infos.push(self.ncn_operator_state.clone());
        account_infos.push(self.ncn_vault_ticket.clone());
        account_infos.push(self.operator_vault_ticket.clone());
        account_infos.push(self.vault_ncn_ticket.clone());
        account_infos.push(self.vault_operator_delegation.clone());
        account_infos.push(self.ncn_vault_slasher_ticket.clone());
        account_infos.push(self.vault_ncn_slasher_ticket.clone());
        account_infos.push(self.vault_ncn_slasher_operator_ticket.clone());
        account_infos.push(self.vault_token_account.clone());
        account_infos.push(self.slasher_token_account.clone());
        account_infos.push(self.token_program.clone());
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

/// Instruction builder for `Slash` via CPI.
///
/// ### Accounts:
///
///   0. `[]` config
///   1. `[writable]` vault
///   2. `[]` ncn
///   3. `[]` operator
///   4. `[signer]` slasher
///   5. `[]` ncn_operator_state
///   6. `[]` ncn_vault_ticket
///   7. `[]` operator_vault_ticket
///   8. `[]` vault_ncn_ticket
///   9. `[writable]` vault_operator_delegation
///   10. `[]` ncn_vault_slasher_ticket
///   11. `[]` vault_ncn_slasher_ticket
///   12. `[writable]` vault_ncn_slasher_operator_ticket
///   13. `[writable]` vault_token_account
///   14. `[]` slasher_token_account
///   15. `[]` token_program
#[derive(Clone, Debug)]
pub struct SlashCpiBuilder<'a, 'b> {
    instruction: Box<SlashCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> SlashCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(SlashCpiBuilderInstruction {
            __program: program,
            config: None,
            vault: None,
            ncn: None,
            operator: None,
            slasher: None,
            ncn_operator_state: None,
            ncn_vault_ticket: None,
            operator_vault_ticket: None,
            vault_ncn_ticket: None,
            vault_operator_delegation: None,
            ncn_vault_slasher_ticket: None,
            vault_ncn_slasher_ticket: None,
            vault_ncn_slasher_operator_ticket: None,
            vault_token_account: None,
            slasher_token_account: None,
            token_program: None,
            amount: None,
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
    pub fn ncn(&mut self, ncn: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.ncn = Some(ncn);
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
    pub fn slasher(
        &mut self,
        slasher: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.slasher = Some(slasher);
        self
    }
    #[inline(always)]
    pub fn ncn_operator_state(
        &mut self,
        ncn_operator_state: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.ncn_operator_state = Some(ncn_operator_state);
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
    pub fn operator_vault_ticket(
        &mut self,
        operator_vault_ticket: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.operator_vault_ticket = Some(operator_vault_ticket);
        self
    }
    #[inline(always)]
    pub fn vault_ncn_ticket(
        &mut self,
        vault_ncn_ticket: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.vault_ncn_ticket = Some(vault_ncn_ticket);
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
    pub fn ncn_vault_slasher_ticket(
        &mut self,
        ncn_vault_slasher_ticket: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.ncn_vault_slasher_ticket = Some(ncn_vault_slasher_ticket);
        self
    }
    #[inline(always)]
    pub fn vault_ncn_slasher_ticket(
        &mut self,
        vault_ncn_slasher_ticket: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.vault_ncn_slasher_ticket = Some(vault_ncn_slasher_ticket);
        self
    }
    #[inline(always)]
    pub fn vault_ncn_slasher_operator_ticket(
        &mut self,
        vault_ncn_slasher_operator_ticket: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.vault_ncn_slasher_operator_ticket =
            Some(vault_ncn_slasher_operator_ticket);
        self
    }
    #[inline(always)]
    pub fn vault_token_account(
        &mut self,
        vault_token_account: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.vault_token_account = Some(vault_token_account);
        self
    }
    #[inline(always)]
    pub fn slasher_token_account(
        &mut self,
        slasher_token_account: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.slasher_token_account = Some(slasher_token_account);
        self
    }
    #[inline(always)]
    pub fn token_program(
        &mut self,
        token_program: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.token_program = Some(token_program);
        self
    }
    #[inline(always)]
    pub fn amount(&mut self, amount: u64) -> &mut Self {
        self.instruction.amount = Some(amount);
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
        let args = SlashInstructionArgs {
            amount: self.instruction.amount.clone().expect("amount is not set"),
        };
        let instruction = SlashCpi {
            __program: self.instruction.__program,

            config: self.instruction.config.expect("config is not set"),

            vault: self.instruction.vault.expect("vault is not set"),

            ncn: self.instruction.ncn.expect("ncn is not set"),

            operator: self.instruction.operator.expect("operator is not set"),

            slasher: self.instruction.slasher.expect("slasher is not set"),

            ncn_operator_state: self
                .instruction
                .ncn_operator_state
                .expect("ncn_operator_state is not set"),

            ncn_vault_ticket: self
                .instruction
                .ncn_vault_ticket
                .expect("ncn_vault_ticket is not set"),

            operator_vault_ticket: self
                .instruction
                .operator_vault_ticket
                .expect("operator_vault_ticket is not set"),

            vault_ncn_ticket: self
                .instruction
                .vault_ncn_ticket
                .expect("vault_ncn_ticket is not set"),

            vault_operator_delegation: self
                .instruction
                .vault_operator_delegation
                .expect("vault_operator_delegation is not set"),

            ncn_vault_slasher_ticket: self
                .instruction
                .ncn_vault_slasher_ticket
                .expect("ncn_vault_slasher_ticket is not set"),

            vault_ncn_slasher_ticket: self
                .instruction
                .vault_ncn_slasher_ticket
                .expect("vault_ncn_slasher_ticket is not set"),

            vault_ncn_slasher_operator_ticket: self
                .instruction
                .vault_ncn_slasher_operator_ticket
                .expect("vault_ncn_slasher_operator_ticket is not set"),

            vault_token_account: self
                .instruction
                .vault_token_account
                .expect("vault_token_account is not set"),

            slasher_token_account: self
                .instruction
                .slasher_token_account
                .expect("slasher_token_account is not set"),

            token_program: self
                .instruction
                .token_program
                .expect("token_program is not set"),
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct SlashCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    ncn: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    operator: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    slasher: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    ncn_operator_state: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    ncn_vault_ticket: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    operator_vault_ticket: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault_ncn_ticket: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault_operator_delegation: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    ncn_vault_slasher_ticket: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault_ncn_slasher_ticket: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault_ncn_slasher_operator_ticket: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    vault_token_account: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    slasher_token_account: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    token_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    amount: Option<u64>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
