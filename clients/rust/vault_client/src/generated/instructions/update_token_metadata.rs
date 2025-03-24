//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>
//!

use borsh::BorshDeserialize;
use borsh::BorshSerialize;

/// Accounts.
pub struct UpdateTokenMetadata {
      
              
          pub vault: solana_program::pubkey::Pubkey,
          
              
          pub admin: solana_program::pubkey::Pubkey,
          
              
          pub vrt_mint: solana_program::pubkey::Pubkey,
          
              
          pub metadata: solana_program::pubkey::Pubkey,
          
              
          pub mpl_token_metadata_program: solana_program::pubkey::Pubkey,
      }

impl UpdateTokenMetadata {
  pub fn instruction(&self, args: UpdateTokenMetadataInstructionArgs) -> solana_program::instruction::Instruction {
    self.instruction_with_remaining_accounts(args, &[])
  }
  #[allow(clippy::vec_init_then_push)]
  pub fn instruction_with_remaining_accounts(&self, args: UpdateTokenMetadataInstructionArgs, remaining_accounts: &[solana_program::instruction::AccountMeta]) -> solana_program::instruction::Instruction {
    let mut accounts = Vec::with_capacity(5 + remaining_accounts.len());
                            accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.vault,
            false
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.admin,
            true
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.vrt_mint,
            false
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new(
            self.metadata,
            false
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.mpl_token_metadata_program,
            false
          ));
                      accounts.extend_from_slice(remaining_accounts);
    let mut data = UpdateTokenMetadataInstructionData::new().try_to_vec().unwrap();
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
pub struct UpdateTokenMetadataInstructionData {
            discriminator: u8,
                        }

impl UpdateTokenMetadataInstructionData {
  pub fn new() -> Self {
    Self {
                        discriminator: 30,
                                                            }
  }
}

impl Default for UpdateTokenMetadataInstructionData {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UpdateTokenMetadataInstructionArgs {
                  pub name: String,
                pub symbol: String,
                pub uri: String,
      }


/// Instruction builder for `UpdateTokenMetadata`.
///
/// ### Accounts:
///
          ///   0. `[]` vault
                ///   1. `[signer]` admin
          ///   2. `[]` vrt_mint
                ///   3. `[writable]` metadata
                ///   4. `[optional]` mpl_token_metadata_program (default to `metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s`)
#[derive(Clone, Debug, Default)]
pub struct UpdateTokenMetadataBuilder {
            vault: Option<solana_program::pubkey::Pubkey>,
                admin: Option<solana_program::pubkey::Pubkey>,
                vrt_mint: Option<solana_program::pubkey::Pubkey>,
                metadata: Option<solana_program::pubkey::Pubkey>,
                mpl_token_metadata_program: Option<solana_program::pubkey::Pubkey>,
                        name: Option<String>,
                symbol: Option<String>,
                uri: Option<String>,
        __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl UpdateTokenMetadataBuilder {
  pub fn new() -> Self {
    Self::default()
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
    pub fn vrt_mint(&mut self, vrt_mint: solana_program::pubkey::Pubkey) -> &mut Self {
                        self.vrt_mint = Some(vrt_mint);
                    self
    }
            #[inline(always)]
    pub fn metadata(&mut self, metadata: solana_program::pubkey::Pubkey) -> &mut Self {
                        self.metadata = Some(metadata);
                    self
    }
            /// `[optional account, default to 'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s']`
#[inline(always)]
    pub fn mpl_token_metadata_program(&mut self, mpl_token_metadata_program: solana_program::pubkey::Pubkey) -> &mut Self {
                        self.mpl_token_metadata_program = Some(mpl_token_metadata_program);
                    self
    }
                    #[inline(always)]
      pub fn name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
      }
                #[inline(always)]
      pub fn symbol(&mut self, symbol: String) -> &mut Self {
        self.symbol = Some(symbol);
        self
      }
                #[inline(always)]
      pub fn uri(&mut self, uri: String) -> &mut Self {
        self.uri = Some(uri);
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
    let accounts = UpdateTokenMetadata {
                              vault: self.vault.expect("vault is not set"),
                                        admin: self.admin.expect("admin is not set"),
                                        vrt_mint: self.vrt_mint.expect("vrt_mint is not set"),
                                        metadata: self.metadata.expect("metadata is not set"),
                                        mpl_token_metadata_program: self.mpl_token_metadata_program.unwrap_or(solana_program::pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")),
                      };
          let args = UpdateTokenMetadataInstructionArgs {
                                                              name: self.name.clone().expect("name is not set"),
                                                                  symbol: self.symbol.clone().expect("symbol is not set"),
                                                                  uri: self.uri.clone().expect("uri is not set"),
                                    };
    
    accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
  }
}

  /// `update_token_metadata` CPI accounts.
  pub struct UpdateTokenMetadataCpiAccounts<'a, 'b> {
          
                    
              pub vault: &'b solana_program::account_info::AccountInfo<'a>,
                
                    
              pub admin: &'b solana_program::account_info::AccountInfo<'a>,
                
                    
              pub vrt_mint: &'b solana_program::account_info::AccountInfo<'a>,
                
                    
              pub metadata: &'b solana_program::account_info::AccountInfo<'a>,
                
                    
              pub mpl_token_metadata_program: &'b solana_program::account_info::AccountInfo<'a>,
            }

/// `update_token_metadata` CPI instruction.
pub struct UpdateTokenMetadataCpi<'a, 'b> {
  /// The program to invoke.
  pub __program: &'b solana_program::account_info::AccountInfo<'a>,
      
              
          pub vault: &'b solana_program::account_info::AccountInfo<'a>,
          
              
          pub admin: &'b solana_program::account_info::AccountInfo<'a>,
          
              
          pub vrt_mint: &'b solana_program::account_info::AccountInfo<'a>,
          
              
          pub metadata: &'b solana_program::account_info::AccountInfo<'a>,
          
              
          pub mpl_token_metadata_program: &'b solana_program::account_info::AccountInfo<'a>,
            /// The arguments for the instruction.
    pub __args: UpdateTokenMetadataInstructionArgs,
  }

impl<'a, 'b> UpdateTokenMetadataCpi<'a, 'b> {
  pub fn new(
    program: &'b solana_program::account_info::AccountInfo<'a>,
          accounts: UpdateTokenMetadataCpiAccounts<'a, 'b>,
              args: UpdateTokenMetadataInstructionArgs,
      ) -> Self {
    Self {
      __program: program,
              vault: accounts.vault,
              admin: accounts.admin,
              vrt_mint: accounts.vrt_mint,
              metadata: accounts.metadata,
              mpl_token_metadata_program: accounts.mpl_token_metadata_program,
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
                            accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.vault.key,
            false
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.admin.key,
            true
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.vrt_mint.key,
            false
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new(
            *self.metadata.key,
            false
          ));
                                          accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.mpl_token_metadata_program.key,
            false
          ));
                      remaining_accounts.iter().for_each(|remaining_account| {
      accounts.push(solana_program::instruction::AccountMeta {
          pubkey: *remaining_account.0.key,
          is_signer: remaining_account.1,
          is_writable: remaining_account.2,
      })
    });
    let mut data = UpdateTokenMetadataInstructionData::new().try_to_vec().unwrap();
          let mut args = self.__args.try_to_vec().unwrap();
      data.append(&mut args);
    
    let instruction = solana_program::instruction::Instruction {
      program_id: crate::JITO_VAULT_ID,
      accounts,
      data,
    };
    let mut account_infos = Vec::with_capacity(5 + 1 + remaining_accounts.len());
    account_infos.push(self.__program.clone());
                  account_infos.push(self.vault.clone());
                        account_infos.push(self.admin.clone());
                        account_infos.push(self.vrt_mint.clone());
                        account_infos.push(self.metadata.clone());
                        account_infos.push(self.mpl_token_metadata_program.clone());
              remaining_accounts.iter().for_each(|remaining_account| account_infos.push(remaining_account.0.clone()));

    if signers_seeds.is_empty() {
      solana_program::program::invoke(&instruction, &account_infos)
    } else {
      solana_program::program::invoke_signed(&instruction, &account_infos, signers_seeds)
    }
  }
}

/// Instruction builder for `UpdateTokenMetadata` via CPI.
///
/// ### Accounts:
///
          ///   0. `[]` vault
                ///   1. `[signer]` admin
          ///   2. `[]` vrt_mint
                ///   3. `[writable]` metadata
          ///   4. `[]` mpl_token_metadata_program
#[derive(Clone, Debug)]
pub struct UpdateTokenMetadataCpiBuilder<'a, 'b> {
  instruction: Box<UpdateTokenMetadataCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> UpdateTokenMetadataCpiBuilder<'a, 'b> {
  pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
    let instruction = Box::new(UpdateTokenMetadataCpiBuilderInstruction {
      __program: program,
              vault: None,
              admin: None,
              vrt_mint: None,
              metadata: None,
              mpl_token_metadata_program: None,
                                            name: None,
                                symbol: None,
                                uri: None,
                    __remaining_accounts: Vec::new(),
    });
    Self { instruction }
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
    pub fn vrt_mint(&mut self, vrt_mint: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
                        self.instruction.vrt_mint = Some(vrt_mint);
                    self
    }
      #[inline(always)]
    pub fn metadata(&mut self, metadata: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
                        self.instruction.metadata = Some(metadata);
                    self
    }
      #[inline(always)]
    pub fn mpl_token_metadata_program(&mut self, mpl_token_metadata_program: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
                        self.instruction.mpl_token_metadata_program = Some(mpl_token_metadata_program);
                    self
    }
                    #[inline(always)]
      pub fn name(&mut self, name: String) -> &mut Self {
        self.instruction.name = Some(name);
        self
      }
                #[inline(always)]
      pub fn symbol(&mut self, symbol: String) -> &mut Self {
        self.instruction.symbol = Some(symbol);
        self
      }
                #[inline(always)]
      pub fn uri(&mut self, uri: String) -> &mut Self {
        self.instruction.uri = Some(uri);
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
          let args = UpdateTokenMetadataInstructionArgs {
                                                              name: self.instruction.name.clone().expect("name is not set"),
                                                                  symbol: self.instruction.symbol.clone().expect("symbol is not set"),
                                                                  uri: self.instruction.uri.clone().expect("uri is not set"),
                                    };
        let instruction = UpdateTokenMetadataCpi {
        __program: self.instruction.__program,
                  
          vault: self.instruction.vault.expect("vault is not set"),
                  
          admin: self.instruction.admin.expect("admin is not set"),
                  
          vrt_mint: self.instruction.vrt_mint.expect("vrt_mint is not set"),
                  
          metadata: self.instruction.metadata.expect("metadata is not set"),
                  
          mpl_token_metadata_program: self.instruction.mpl_token_metadata_program.expect("mpl_token_metadata_program is not set"),
                          __args: args,
            };
    instruction.invoke_signed_with_remaining_accounts(signers_seeds, &self.instruction.__remaining_accounts)
  }
}

#[derive(Clone, Debug)]
struct UpdateTokenMetadataCpiBuilderInstruction<'a, 'b> {
  __program: &'b solana_program::account_info::AccountInfo<'a>,
            vault: Option<&'b solana_program::account_info::AccountInfo<'a>>,
                admin: Option<&'b solana_program::account_info::AccountInfo<'a>>,
                vrt_mint: Option<&'b solana_program::account_info::AccountInfo<'a>>,
                metadata: Option<&'b solana_program::account_info::AccountInfo<'a>>,
                mpl_token_metadata_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
                        name: Option<String>,
                symbol: Option<String>,
                uri: Option<String>,
        /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
  __remaining_accounts: Vec<(&'b solana_program::account_info::AccountInfo<'a>, bool, bool)>,
}

