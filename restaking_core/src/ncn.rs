//! The NCN (Node Consensus Network) account is a program-owned account that
//! represents a network of nodes that participate in consensus. The NCN
//! account is used to manage the operators, vaults, and slashers that are
//! associated with the network.
use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use solana_program::account_info::AccountInfo;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

/// The NCN manages the operators, vaults, and slashers associated with a network
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct Ncn {
    /// The base account used as a PDA seed
    pub base: Pubkey,

    /// The admin of the NCN
    pub admin: Pubkey,

    /// The operator admin of the NCN
    pub operator_admin: Pubkey,

    /// The vault admin of the NCN
    pub vault_admin: Pubkey,

    /// The slasher admin of the NCN
    pub slasher_admin: Pubkey,

    /// The withdraw admin of the NCN
    pub withdraw_admin: Pubkey,

    /// The withdraw fee wallet of the NCN
    pub withdraw_fee_wallet: Pubkey,

    /// The index of the NCN
    pub index: u64,

    /// Number of operator accounts associated with the NCN
    pub operator_count: u64,

    /// Number of vault accounts associated with the NCN
    pub vault_count: u64,

    /// Number of slasher accounts associated with the NCN
    pub slasher_count: u64,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 7],
}

impl Discriminator for Ncn {
    const DISCRIMINATOR: u8 = 2;
}

impl Ncn {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(base: Pubkey, admin: Pubkey, ncn_index: u64, bump: u8) -> Self {
        Self {
            base,
            admin,
            operator_admin: admin,
            vault_admin: admin,
            slasher_admin: admin,
            withdraw_admin: admin,
            withdraw_fee_wallet: admin,
            index: ncn_index,
            operator_count: 0,
            vault_count: 0,
            slasher_count: 0,
            bump,
            reserved: [0; 7],
        }
    }

    /// Returns the seeds for the PDA
    pub fn seeds(base: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([b"ncn".to_vec(), base.as_ref().to_vec()])
    }

    /// Find the program address for the NCN account
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `base` - The base account used as a PDA seed
    ///
    /// # Returns
    /// * [`Pubkey`] - The program address
    pub fn find_program_address(program_id: &Pubkey, base: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(base);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    /// Attempts to load the account as [`Ncn`], returning an error if it's not valid.
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `account` - The account to load the NCN from
    /// * `expect_writable` - Whether the account should be writable
    ///
    /// # Returns
    /// * `Result<(), ProgramError>` - The result of the operation
    pub fn load(
        program_id: &Pubkey,
        account: &AccountInfo,
        expect_writable: bool,
    ) -> Result<(), ProgramError> {
        if account.owner.ne(program_id) {
            msg!("NCN account has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if account.data_is_empty() {
            msg!("NCN account data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !account.is_writable {
            msg!("NCN account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if account.data.borrow()[0].ne(&Ncn::DISCRIMINATOR) {
            msg!("NCN account discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        let base = Ncn::try_from_slice_unchecked(&account.data.borrow())?.base;
        if account
            .key
            .ne(&Ncn::find_program_address(program_id, &base).0)
        {
            msg!("NCN account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}
