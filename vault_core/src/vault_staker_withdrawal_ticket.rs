//! The [`VaultStakerWithdrawalTicket`] account is used to represent a pending withdrawal from a vault by a staker.
//! For every withdraw ticket, there's an associated token account owned by the withdrawal ticket with the staker's VRT.
use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use solana_program::account_info::AccountInfo;
use solana_program::{msg, program_error::ProgramError, pubkey::Pubkey};

impl Discriminator for VaultStakerWithdrawalTicket {
    const DISCRIMINATOR: u8 = 7;
}

/// The [`VaultStakerWithdrawalTicket`] account is used to represent a pending withdrawal from a vault by a staker.
/// For every withdraw ticket, there's an associated token account owned by the withdrawal ticket with the staker's VRT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct VaultStakerWithdrawalTicket {
    /// The vault being withdrawn from
    pub vault: Pubkey,

    /// The staker withdrawing from the vault
    pub staker: Pubkey,

    /// The base account used as a PDA seed
    pub base: Pubkey,

    /// The amount of VRT held in the VaultStakerWithdrawalTicket token account at the time of creation.
    /// This is used to ensure the amount redeemed is the same as the amount allocated.
    pub vrt_amount: u64,

    /// The slot the withdrawal was enqueued
    pub slot_unstaked: u64,

    /// The bump seed used to create the PDA
    pub bump: u8,

    reserved: [u8; 7],
}

impl VaultStakerWithdrawalTicket {
    pub const fn new(
        vault: Pubkey,
        staker: Pubkey,
        base: Pubkey,
        vrt_amount: u64,
        slot_unstaked: u64,
        bump: u8,
    ) -> Self {
        Self {
            vault,
            staker,
            base,
            vrt_amount,
            slot_unstaked,
            bump,
            reserved: [0; 7],
        }
    }

    /// In order for the ticket to be withdrawable, it needs to be more than one **full** epoch
    /// since unstaking
    pub fn is_withdrawable(&self, slot: u64, epoch_length: u64) -> Result<bool, ProgramError> {
        let current_epoch = slot.checked_div(epoch_length).unwrap();
        let epoch_unstaked = self.slot_unstaked.checked_div(epoch_length).unwrap();
        if current_epoch
            <= epoch_unstaked
                .checked_add(1)
                .ok_or(ProgramError::ArithmeticOverflow)?
        {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    /// Returns the seeds for the PDA
    ///
    /// # Arguments
    /// * `vault` - The vault
    /// * `staker` - The staker
    /// * `base` - The base account used as a PDA seed
    pub fn seeds(vault: &Pubkey, staker: &Pubkey, base: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"vault_staker_withdrawal_ticket".to_vec(),
            vault.to_bytes().to_vec(),
            staker.to_bytes().to_vec(),
            base.to_bytes().to_vec(),
        ])
    }

    /// Find the program address for the PDA
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `vault` - The vault
    /// * `staker` - The staker
    /// * `base` - The base account used as a PDA seed
    ///
    /// # Returns
    /// * [`Pubkey`] - The program address
    /// * `u8` - The bump seed
    /// * `Vec<Vec<u8>` - The seeds used to generate the PDA
    pub fn find_program_address(
        program_id: &Pubkey,
        vault: &Pubkey,
        staker: &Pubkey,
        base: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(vault, staker, base);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    /// Loads the [`VaultStakerWithdrawalTicket`] account
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `vault_staker_withdrawal_ticket` - The [`VaultStakerWithdrawalTicket`] account
    /// * `vault` - The [`Vault`] account
    /// * `staker` - The staker account
    /// * `expect_writable` - Whether the account should be writable
    ///
    /// # Returns
    /// * `Result<(), ProgramError>` - The result of the operation
    pub fn load(
        program_id: &Pubkey,
        vault_staker_withdrawal_ticket: &AccountInfo,
        vault: &AccountInfo,
        staker: &AccountInfo,
        expect_writable: bool,
    ) -> Result<(), ProgramError> {
        if vault_staker_withdrawal_ticket.owner.ne(program_id) {
            msg!("Vault staker withdraw ticket has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if vault_staker_withdrawal_ticket.data_is_empty() {
            msg!("Vault staker withdraw ticket data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !vault_staker_withdrawal_ticket.is_writable {
            msg!("Vault staker withdraw ticket is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if vault_staker_withdrawal_ticket.data.borrow()[0]
            .ne(&VaultStakerWithdrawalTicket::DISCRIMINATOR)
        {
            msg!("Vault staker withdraw ticket discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        let vault_staker_withdraw_ticket_data = vault_staker_withdrawal_ticket.data.borrow();
        let base = VaultStakerWithdrawalTicket::try_from_slice_unchecked(
            &vault_staker_withdraw_ticket_data,
        )?
        .base;
        let expected_pubkey = VaultStakerWithdrawalTicket::find_program_address(
            program_id, vault.key, staker.key, &base,
        )
        .0;
        if vault_staker_withdrawal_ticket.key.ne(&expected_pubkey) {
            msg!("Vault staker withdraw ticket is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}
