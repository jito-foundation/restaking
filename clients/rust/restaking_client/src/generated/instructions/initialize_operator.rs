//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use borsh::BorshDeserialize;
use borsh::BorshSerialize;

/// Accounts.
pub struct InitializeOperator {
      
              
          pub config: solana_program::pubkey::Pubkey,
          
              
          pub operator: solana_program::pubkey::Pubkey,
          
              
          pub admin: solana_program::pubkey::Pubkey,
          
              
          pub base: solana_program::pubkey::Pubkey,
          
              
          pub system_program: solana_program::pubkey::Pubkey,
      }

impl InitializeOperator {
  pub fn instruction(&self, args: InitializeOperatorInstructionArgs) -> solana_program::instruction::Instruction {
    self.instruction_with_remaining_accounts(args, &[])
  }
  #[allow(clippy::vec_init_then_push)]
  pub fn instruction_with_remaining_accounts(&self, args: InitializeOperatorInstructionArgs, remaining_accounts: &[solana_program::instruction::AccountMeta]) -> solana_program::instruction::Instruction {
    let mut accounts = Vec::with_capacity(5 + remaining_accounts.len());
                            accounts.push(solana_program::instruction::AccountMeta::new(
            self.config,
            false
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new(
            self.operator,
            false
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new(
            self.admin,
            true
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.base,
            true
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.system_program,
            false
          ));
                      accounts.extend_from_slice(remaining_accounts);
    let mut data = InitializeOperatorInstructionData::new().try_to_vec().unwrap();
          let mut args = args.try_to_vec().unwrap();
      data.append(&mut args);
    
    solana_program::instruction::Instruction {
      program_id: crate::JITO_RESTAKING_ID,
      accounts,
      data,
    }
  }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct InitializeOperatorInstructionData {
            discriminator: u8,
            }

impl InitializeOperatorInstructionData {
  pub fn new() -> Self {
    Self {
                        discriminator: 2,
                                }
  }
}

impl Default for InitializeOperatorInstructionData {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InitializeOperatorInstructionArgs {
                  pub operator_fee_bps: u16,
      }


/// Instruction builder for `InitializeOperator`.
///
/// ### Accounts:
///
                ///   0. `[writable]` config
                ///   1. `[writable]` operator
                      ///   2. `[writable, signer]` admin
                ///   3. `[signer]` base
                ///   4. `[optional]` system_program (default to `11111111111111111111111111111111`)
#[derive(Clone, Debug, Default)]
pub struct InitializeOperatorBuilder {
            config: Option<solana_program::pubkey::Pubkey>,
                operator: Option<solana_program::pubkey::Pubkey>,
                admin: Option<solana_program::pubkey::Pubkey>,
                base: Option<solana_program::pubkey::Pubkey>,
                system_program: Option<solana_program::pubkey::Pubkey>,
                        operator_fee_bps: Option<u16>,
        __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl InitializeOperatorBuilder {
  pub fn new() -> Self {
    Self::default()
  }
            #[inline(always)]
    pub fn config(&mut self, config: solana_program::pubkey::Pubkey) -> &mut Self {
                        self.config = Some(config);
                    self
    }
            #[inline(always)]
    pub fn operator(&mut self, operator: solana_program::pubkey::Pubkey) -> &mut Self {
                        self.operator = Some(operator);
                    self
    }
            #[inline(always)]
    pub fn admin(&mut self, admin: solana_program::pubkey::Pubkey) -> &mut Self {
                        self.admin = Some(admin);
                    self
    }
            #[inline(always)]
    pub fn base(&mut self, base: solana_program::pubkey::Pubkey) -> &mut Self {
                        self.base = Some(base);
                    self
    }
            /// `[optional account, default to '11111111111111111111111111111111']`
#[inline(always)]
    pub fn system_program(&mut self, system_program: solana_program::pubkey::Pubkey) -> &mut Self {
                        self.system_program = Some(system_program);
                    self
    }
                    #[inline(always)]
      pub fn operator_fee_bps(&mut self, operator_fee_bps: u16) -> &mut Self {
        self.operator_fee_bps = Some(operator_fee_bps);
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
    let accounts = InitializeOperator {
                              config: self.config.expect("config is not set"),
                                        operator: self.operator.expect("operator is not set"),
                                        admin: self.admin.expect("admin is not set"),
                                        base: self.base.expect("base is not set"),
                                        system_program: self.system_program.unwrap_or(solana_program::pubkey!("11111111111111111111111111111111")),
                      };
          let args = InitializeOperatorInstructionArgs {
                                                              operator_fee_bps: self.operator_fee_bps.clone().expect("operator_fee_bps is not set"),
                                    };
    
    accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
  }
}

  /// `initialize_operator` CPI accounts.
  pub struct InitializeOperatorCpiAccounts<'a, 'b> {
          
                    
              pub config: &'b solana_program::account_info::AccountInfo<'a>,
                
                    
              pub operator: &'b solana_program::account_info::AccountInfo<'a>,
                
                    
              pub admin: &'b solana_program::account_info::AccountInfo<'a>,
                
                    
              pub base: &'b solana_program::account_info::AccountInfo<'a>,
                
                    
              pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
            }

/// `initialize_operator` CPI instruction.
pub struct InitializeOperatorCpi<'a, 'b> {
  /// The program to invoke.
  pub __program: &'b solana_program::account_info::AccountInfo<'a>,
      
              
          pub config: &'b solana_program::account_info::AccountInfo<'a>,
          
              
          pub operator: &'b solana_program::account_info::AccountInfo<'a>,
          
              
          pub admin: &'b solana_program::account_info::AccountInfo<'a>,
          
              
          pub base: &'b solana_program::account_info::AccountInfo<'a>,
          
              
          pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
            /// The arguments for the instruction.
    pub __args: InitializeOperatorInstructionArgs,
  }

impl<'a, 'b> InitializeOperatorCpi<'a, 'b> {
  pub fn new(
    program: &'b solana_program::account_info::AccountInfo<'a>,
          accounts: InitializeOperatorCpiAccounts<'a, 'b>,
              args: InitializeOperatorInstructionArgs,
      ) -> Self {
    Self {
      __program: program,
              config: accounts.config,
              operator: accounts.operator,
              admin: accounts.admin,
              base: accounts.base,
              system_program: accounts.system_program,
                    __args: args,
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
    let mut accounts = Vec::with_capacity(5 + remaining_accounts.len());
                            accounts.push(solana_program::instruction::AccountMeta::new(
            *self.config.key,
            false
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new(
            *self.operator.key,
            false
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new(
            *self.admin.key,
            true
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.base.key,
            true
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
    let mut data = InitializeOperatorInstructionData::new().try_to_vec().unwrap();
          let mut args = self.__args.try_to_vec().unwrap();
      data.append(&mut args);
    
    let instruction = solana_program::instruction::Instruction {
      program_id: crate::JITO_RESTAKING_ID,
      accounts,
      data,
    };
    let mut account_infos = Vec::with_capacity(5 + 1 + remaining_accounts.len());
    account_infos.push(self.__program.clone());
                  account_infos.push(self.config.clone());
                        account_infos.push(self.operator.clone());
                        account_infos.push(self.admin.clone());
                        account_infos.push(self.base.clone());
                        account_infos.push(self.system_program.clone());
              remaining_accounts.iter().for_each(|remaining_account| account_infos.push(remaining_account.0.clone()));

    if signers_seeds.is_empty() {
      solana_program::program::invoke(&instruction, &account_infos)
    } else {
      solana_program::program::invoke_signed(&instruction, &account_infos, signers_seeds)
    }
  }
}

/// Instruction builder for `InitializeOperator` via CPI.
///
/// ### Accounts:
///
                ///   0. `[writable]` config
                ///   1. `[writable]` operator
                      ///   2. `[writable, signer]` admin
                ///   3. `[signer]` base
          ///   4. `[]` system_program
#[derive(Clone, Debug)]
pub struct InitializeOperatorCpiBuilder<'a, 'b> {
  instruction: Box<InitializeOperatorCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> InitializeOperatorCpiBuilder<'a, 'b> {
  pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
    let instruction = Box::new(InitializeOperatorCpiBuilderInstruction {
      __program: program,
              config: None,
              operator: None,
              admin: None,
              base: None,
              system_program: None,
                                            operator_fee_bps: None,
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
    pub fn operator(&mut self, operator: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
                        self.instruction.operator = Some(operator);
                    self
    }
      #[inline(always)]
    pub fn admin(&mut self, admin: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
                        self.instruction.admin = Some(admin);
                    self
    }
      #[inline(always)]
    pub fn base(&mut self, base: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
                        self.instruction.base = Some(base);
                    self
    }
      #[inline(always)]
    pub fn system_program(&mut self, system_program: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
                        self.instruction.system_program = Some(system_program);
                    self
    }
                    #[inline(always)]
      pub fn operator_fee_bps(&mut self, operator_fee_bps: u16) -> &mut Self {
        self.instruction.operator_fee_bps = Some(operator_fee_bps);
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
          let args = InitializeOperatorInstructionArgs {
                                                              operator_fee_bps: self.instruction.operator_fee_bps.clone().expect("operator_fee_bps is not set"),
                                    };
        let instruction = InitializeOperatorCpi {
        __program: self.instruction.__program,
                  
          config: self.instruction.config.expect("config is not set"),
                  
          operator: self.instruction.operator.expect("operator is not set"),
                  
          admin: self.instruction.admin.expect("admin is not set"),
                  
          base: self.instruction.base.expect("base is not set"),
                  
          system_program: self.instruction.system_program.expect("system_program is not set"),
                          __args: args,
            };
    instruction.invoke_signed_with_remaining_accounts(signers_seeds, &self.instruction.__remaining_accounts)
  }
}

#[derive(Clone, Debug)]
struct InitializeOperatorCpiBuilderInstruction<'a, 'b> {
  __program: &'b solana_program::account_info::AccountInfo<'a>,
            config: Option<&'b solana_program::account_info::AccountInfo<'a>>,
                operator: Option<&'b solana_program::account_info::AccountInfo<'a>>,
                admin: Option<&'b solana_program::account_info::AccountInfo<'a>>,
                base: Option<&'b solana_program::account_info::AccountInfo<'a>>,
                system_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
                        operator_fee_bps: Option<u16>,
        /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
  __remaining_accounts: Vec<(&'b solana_program::account_info::AccountInfo<'a>, bool, bool)>,
}

