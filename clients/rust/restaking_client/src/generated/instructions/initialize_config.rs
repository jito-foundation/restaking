//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use borsh::BorshDeserialize;
use borsh::BorshSerialize;

/// Accounts.
pub struct InitializeConfig {
      
              
          pub config: solana_program::pubkey::Pubkey,
          
              
          pub admin: solana_program::pubkey::Pubkey,
          
              
          pub vault_program: solana_program::pubkey::Pubkey,
          
              
          pub system_program: solana_program::pubkey::Pubkey,
      }

impl InitializeConfig {
  pub fn instruction(&self) -> solana_program::instruction::Instruction {
    self.instruction_with_remaining_accounts(&[])
  }
  #[allow(clippy::vec_init_then_push)]
  pub fn instruction_with_remaining_accounts(&self, remaining_accounts: &[solana_program::instruction::AccountMeta]) -> solana_program::instruction::Instruction {
    let mut accounts = Vec::with_capacity(4 + remaining_accounts.len());
                            accounts.push(solana_program::instruction::AccountMeta::new(
            self.config,
            false
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new(
            self.admin,
            true
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.vault_program,
            false
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.system_program,
            false
          ));
                      accounts.extend_from_slice(remaining_accounts);
    let data = InitializeConfigInstructionData::new().try_to_vec().unwrap();
    
    solana_program::instruction::Instruction {
      program_id: crate::JITO_RESTAKING_ID,
      accounts,
      data,
    }
  }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct InitializeConfigInstructionData {
            discriminator: u8,
      }

impl InitializeConfigInstructionData {
  pub fn new() -> Self {
    Self {
                        discriminator: 0,
                  }
  }
}

impl Default for InitializeConfigInstructionData {
  fn default() -> Self {
    Self::new()
  }
}



/// Instruction builder for `InitializeConfig`.
///
/// ### Accounts:
///
                ///   0. `[writable]` config
                      ///   1. `[writable, signer]` admin
          ///   2. `[]` vault_program
                ///   3. `[optional]` system_program (default to `11111111111111111111111111111111`)
#[derive(Clone, Debug, Default)]
pub struct InitializeConfigBuilder {
            config: Option<solana_program::pubkey::Pubkey>,
                admin: Option<solana_program::pubkey::Pubkey>,
                vault_program: Option<solana_program::pubkey::Pubkey>,
                system_program: Option<solana_program::pubkey::Pubkey>,
                __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl InitializeConfigBuilder {
  pub fn new() -> Self {
    Self::default()
  }
            #[inline(always)]
    pub fn config(&mut self, config: solana_program::pubkey::Pubkey) -> &mut Self {
                        self.config = Some(config);
                    self
    }
            #[inline(always)]
    pub fn admin(&mut self, admin: solana_program::pubkey::Pubkey) -> &mut Self {
                        self.admin = Some(admin);
                    self
    }
            #[inline(always)]
    pub fn vault_program(&mut self, vault_program: solana_program::pubkey::Pubkey) -> &mut Self {
                        self.vault_program = Some(vault_program);
                    self
    }
            /// `[optional account, default to '11111111111111111111111111111111']`
#[inline(always)]
    pub fn system_program(&mut self, system_program: solana_program::pubkey::Pubkey) -> &mut Self {
                        self.system_program = Some(system_program);
                    self
    }
            /// Add an additional account to the instruction.
  #[inline(always)]
  pub fn add_remaining_account(&mut self, account: solana_program::instruction::AccountMeta) -> &mut Self {
    self.__remaining_accounts.push(account);
    self
  }
  /// Add additional accounts to the instruction.
  #[inline(always)]
  pub fn add_remaining_accounts(&mut self, accounts: &[solana_program::instruction::AccountMeta]) -> &mut Self {
    self.__remaining_accounts.extend_from_slice(accounts);
    self
  }
  #[allow(clippy::clone_on_copy)]
  pub fn instruction(&self) -> solana_program::instruction::Instruction {
    let accounts = InitializeConfig {
                              config: self.config.expect("config is not set"),
                                        admin: self.admin.expect("admin is not set"),
                                        vault_program: self.vault_program.expect("vault_program is not set"),
                                        system_program: self.system_program.unwrap_or(solana_program::pubkey!("11111111111111111111111111111111")),
                      };
    
    accounts.instruction_with_remaining_accounts(&self.__remaining_accounts)
  }
}

  /// `initialize_config` CPI accounts.
  pub struct InitializeConfigCpiAccounts<'a, 'b> {
          
                    
              pub config: &'b solana_program::account_info::AccountInfo<'a>,
                
                    
              pub admin: &'b solana_program::account_info::AccountInfo<'a>,
                
                    
              pub vault_program: &'b solana_program::account_info::AccountInfo<'a>,
                
                    
              pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
            }

/// `initialize_config` CPI instruction.
pub struct InitializeConfigCpi<'a, 'b> {
  /// The program to invoke.
  pub __program: &'b solana_program::account_info::AccountInfo<'a>,
      
              
          pub config: &'b solana_program::account_info::AccountInfo<'a>,
          
              
          pub admin: &'b solana_program::account_info::AccountInfo<'a>,
          
              
          pub vault_program: &'b solana_program::account_info::AccountInfo<'a>,
          
              
          pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
        }

impl<'a, 'b> InitializeConfigCpi<'a, 'b> {
  pub fn new(
    program: &'b solana_program::account_info::AccountInfo<'a>,
          accounts: InitializeConfigCpiAccounts<'a, 'b>,
          ) -> Self {
    Self {
      __program: program,
              config: accounts.config,
              admin: accounts.admin,
              vault_program: accounts.vault_program,
              system_program: accounts.system_program,
                }
  }
  #[inline(always)]
  pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
    self.invoke_signed_with_remaining_accounts(&[], &[])
  }
  #[inline(always)]
  pub fn invoke_with_remaining_accounts(&self, remaining_accounts: &[(&'b solana_program::account_info::AccountInfo<'a>, bool, bool)]) -> solana_program::entrypoint::ProgramResult {
    self.invoke_signed_with_remaining_accounts(&[], remaining_accounts)
  }
  #[inline(always)]
  pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> solana_program::entrypoint::ProgramResult {
    self.invoke_signed_with_remaining_accounts(signers_seeds, &[])
  }
  #[allow(clippy::clone_on_copy)]
  #[allow(clippy::vec_init_then_push)]
  pub fn invoke_signed_with_remaining_accounts(
    &self,
    signers_seeds: &[&[&[u8]]],
    remaining_accounts: &[(&'b solana_program::account_info::AccountInfo<'a>, bool, bool)]
  ) -> solana_program::entrypoint::ProgramResult {
    let mut accounts = Vec::with_capacity(4 + remaining_accounts.len());
                            accounts.push(solana_program::instruction::AccountMeta::new(
            *self.config.key,
            false
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new(
            *self.admin.key,
            true
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.vault_program.key,
            false
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.system_program.key,
            false
          ));
                      remaining_accounts.iter().for_each(|remaining_account| {
      accounts.push(solana_program::instruction::AccountMeta {
          pubkey: *remaining_account.0.key,
          is_signer: remaining_account.1,
          is_writable: remaining_account.2,
      })
    });
    let data = InitializeConfigInstructionData::new().try_to_vec().unwrap();
    
    let instruction = solana_program::instruction::Instruction {
      program_id: crate::JITO_RESTAKING_ID,
      accounts,
      data,
    };
    let mut account_infos = Vec::with_capacity(4 + 1 + remaining_accounts.len());
    account_infos.push(self.__program.clone());
                  account_infos.push(self.config.clone());
                        account_infos.push(self.admin.clone());
                        account_infos.push(self.vault_program.clone());
                        account_infos.push(self.system_program.clone());
              remaining_accounts.iter().for_each(|remaining_account| account_infos.push(remaining_account.0.clone()));

    if signers_seeds.is_empty() {
      solana_program::program::invoke(&instruction, &account_infos)
    } else {
      solana_program::program::invoke_signed(&instruction, &account_infos, signers_seeds)
    }
  }
}

/// Instruction builder for `InitializeConfig` via CPI.
///
/// ### Accounts:
///
                ///   0. `[writable]` config
                      ///   1. `[writable, signer]` admin
          ///   2. `[]` vault_program
          ///   3. `[]` system_program
#[derive(Clone, Debug)]
pub struct InitializeConfigCpiBuilder<'a, 'b> {
  instruction: Box<InitializeConfigCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> InitializeConfigCpiBuilder<'a, 'b> {
  pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
    let instruction = Box::new(InitializeConfigCpiBuilderInstruction {
      __program: program,
              config: None,
              admin: None,
              vault_program: None,
              system_program: None,
                                __remaining_accounts: Vec::new(),
    });
    Self { instruction }
  }
      #[inline(always)]
    pub fn config(&mut self, config: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
                        self.instruction.config = Some(config);
                    self
    }
      #[inline(always)]
    pub fn admin(&mut self, admin: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
                        self.instruction.admin = Some(admin);
                    self
    }
      #[inline(always)]
    pub fn vault_program(&mut self, vault_program: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
                        self.instruction.vault_program = Some(vault_program);
                    self
    }
      #[inline(always)]
    pub fn system_program(&mut self, system_program: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
                        self.instruction.system_program = Some(system_program);
                    self
    }
            /// Add an additional account to the instruction.
  #[inline(always)]
  pub fn add_remaining_account(&mut self, account: &'b solana_program::account_info::AccountInfo<'a>, is_writable: bool, is_signer: bool) -> &mut Self {
    self.instruction.__remaining_accounts.push((account, is_writable, is_signer));
    self
  }
  /// Add additional accounts to the instruction.
  ///
  /// Each account is represented by a tuple of the `AccountInfo`, a `bool` indicating whether the account is writable or not,
  /// and a `bool` indicating whether the account is a signer or not.
  #[inline(always)]
  pub fn add_remaining_accounts(&mut self, accounts: &[(&'b solana_program::account_info::AccountInfo<'a>, bool, bool)]) -> &mut Self {
    self.instruction.__remaining_accounts.extend_from_slice(accounts);
    self
  }
  #[inline(always)]
  pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
    self.invoke_signed(&[])
  }
  #[allow(clippy::clone_on_copy)]
  #[allow(clippy::vec_init_then_push)]
  pub fn invoke_signed(&self, signers_seeds: &[&[&[u8]]]) -> solana_program::entrypoint::ProgramResult {
        let instruction = InitializeConfigCpi {
        __program: self.instruction.__program,
                  
          config: self.instruction.config.expect("config is not set"),
                  
          admin: self.instruction.admin.expect("admin is not set"),
                  
          vault_program: self.instruction.vault_program.expect("vault_program is not set"),
                  
          system_program: self.instruction.system_program.expect("system_program is not set"),
                    };
    instruction.invoke_signed_with_remaining_accounts(signers_seeds, &self.instruction.__remaining_accounts)
  }
}

#[derive(Clone, Debug)]
struct InitializeConfigCpiBuilderInstruction<'a, 'b> {
  __program: &'b solana_program::account_info::AccountInfo<'a>,
            config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
                admin: Option<&'b solana_program::account_info::AccountInfo<'a>>,
                vault_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
                system_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
                /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
  __remaining_accounts: Vec<(&'b solana_program::account_info::AccountInfo<'a>, bool, bool)>,
}

