use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::slot_toggle::SlotToggle;
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

impl Discriminator for OperatorVaultTicket {
    const DISCRIMINATOR: u8 = 5;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct OperatorVaultTicket {
    /// The operator account
    pub operator: Pubkey,

    /// The vault account
    pub vault: Pubkey,

    /// The index
    pub index: u64,

    /// The slot toggle
    pub state: SlotToggle,

    pub bump: u8,

    /// Reserved space
    reserved: [u8; 7],
}

impl OperatorVaultTicket {
    pub const fn new(operator: Pubkey, vault: Pubkey, index: u64, bump: u8) -> Self {
        Self {
            operator,
            vault,
            index,
            state: SlotToggle::new(0),
            bump,
            reserved: [0; 7],
        }
    }

    pub fn seeds(operator: &Pubkey, vault: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"operator_vault_ticket".to_vec(),
            operator.to_bytes().to_vec(),
            vault.to_bytes().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        operator: &Pubkey,
        vault: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(operator, vault);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    /// Loads the account as an [`OperatorVaultTicket`] account, returning an error if it is not.
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `operator_vault_ticket` - The account to load the operator vault ticket from
    /// * `operator` - The operator account
    /// * `vault` - The vault account
    /// * `expect_writable` - Whether the account should be writable
    ///
    /// # Returns
    /// * `Result<(), ProgramError>` - The result of the operation
    pub fn load(
        program_id: &Pubkey,
        operator_vault_ticket: &AccountInfo,
        operator: &AccountInfo,
        vault: &AccountInfo,
        expect_writable: bool,
    ) -> Result<(), ProgramError> {
        if operator_vault_ticket.owner.ne(program_id) {
            msg!("Operator vault ticket account has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if operator_vault_ticket.data_is_empty() {
            msg!("Operator vault ticket account data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !operator_vault_ticket.is_writable {
            msg!("Operator vault ticket account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if operator_vault_ticket.data.borrow()[0].ne(&Self::DISCRIMINATOR) {
            msg!("Operator vault ticket account discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        let expected_pubkey = Self::find_program_address(program_id, operator.key, vault.key).0;
        if operator_vault_ticket.key.ne(&expected_pubkey) {
            msg!("Operator vault ticket account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}
