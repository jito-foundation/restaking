//! The [`VaultStakerWithdrawalTicket`] account is used to represent a pending withdrawal from a vault by a staker.
//! For every withdraw ticket, there's an associated token account owned by the withdrawal ticket with the staker's VRT.
use bytemuck::{Pod, Zeroable};
use jito_bytemuck::{types::PodU64, AccountDeserialize, Discriminator};
use jito_vault_sdk::error::VaultError;
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

impl Discriminator for VaultStakerWithdrawalTicket {
    const DISCRIMINATOR: u8 = 7;
}

/// The [`VaultStakerWithdrawalTicket`] account is used to represent a pending withdrawal from a vault by a staker.
/// For every withdrawal ticket, there's an associated token account owned by the withdrawal ticket with the staker's VRT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize, ShankAccount)]
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
    vrt_amount: PodU64,

    /// The slot the withdrawal was enqueued
    slot_unstaked: PodU64,

    /// The bump seed used to create the PDA
    pub bump: u8,

    reserved: [u8; 263],
}

impl VaultStakerWithdrawalTicket {
    pub fn new(
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
            vrt_amount: PodU64::from(vrt_amount),
            slot_unstaked: PodU64::from(slot_unstaked),
            bump,
            reserved: [0; 263],
        }
    }

    pub fn vrt_amount(&self) -> u64 {
        self.vrt_amount.into()
    }

    pub fn slot_unstaked(&self) -> u64 {
        self.slot_unstaked.into()
    }

    pub fn check_staker(&self, staker: &Pubkey) -> Result<(), VaultError> {
        if self.staker.ne(staker) {
            msg!("Staker is not the owner of the withdrawal ticket");
            return Err(VaultError::VaultStakerWithdrawalTicketInvalidStaker);
        }
        Ok(())
    }

    /// In order for the ticket to be withdrawable, it needs to be more than one **full** epoch
    /// since unstaking
    pub fn is_withdrawable(&self, slot: u64, epoch_length: u64) -> Result<bool, ProgramError> {
        let current_epoch = slot
            .checked_div(epoch_length)
            .ok_or(VaultError::DivisionByZero)?;
        let epoch_unstaked = self
            .slot_unstaked()
            .checked_div(epoch_length)
            .ok_or(VaultError::DivisionByZero)?;
        if current_epoch
            <= epoch_unstaked
                .checked_add(1)
                .ok_or(VaultError::ArithmeticOverflow)?
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
    pub fn seeds(vault: &Pubkey, base: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"vault_staker_withdrawal_ticket".to_vec(),
            vault.to_bytes().to_vec(),
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
        base: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(vault, base);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    /// Loads the [`VaultStakerWithdrawalTicket`] account
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `vault_staker_withdrawal_ticket` - The [`VaultStakerWithdrawalTicket`] account
    /// * `vault` - The [`crate::vault::Vault`] account
    /// * `staker` - The staker account
    /// * `expect_writable` - Whether the account should be writable
    ///
    /// # Returns
    /// * `Result<(), ProgramError>` - The result of the operation
    pub fn load(
        program_id: &Pubkey,
        vault_staker_withdrawal_ticket: &AccountInfo,
        vault: &AccountInfo,
        expect_writable: bool,
    ) -> Result<(), ProgramError> {
        if vault_staker_withdrawal_ticket.owner.ne(program_id) {
            msg!("Vault staker withdrawal ticket has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if vault_staker_withdrawal_ticket.data_is_empty() {
            msg!("Vault staker withdrawal ticket data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !vault_staker_withdrawal_ticket.is_writable {
            msg!("Vault staker withdrawal ticket is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if vault_staker_withdrawal_ticket.data.borrow()[0].ne(&Self::DISCRIMINATOR) {
            msg!("Vault staker withdrawal ticket discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        let vault_staker_withdrawal_ticket_data = vault_staker_withdrawal_ticket.data.borrow();
        let base = Self::try_from_slice_unchecked(&vault_staker_withdrawal_ticket_data)?.base;
        let expected_pubkey = Self::find_program_address(program_id, vault.key, &base).0;
        if vault_staker_withdrawal_ticket.key.ne(&expected_pubkey) {
            msg!("Vault staker withdrawal ticket is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_staker_withdrawal_ticket_no_padding() {
        let vault_staker_withdrawal_ticket_size =
            std::mem::size_of::<VaultStakerWithdrawalTicket>();
        let sum_of_fields = size_of::<Pubkey>() + // vault
            size_of::<Pubkey>() + // staker
            size_of::<Pubkey>() + // base
            size_of::<PodU64>() + // vrt_amount
            size_of::<PodU64>() + // slot_unstaked
            size_of::<u8>() + // bump
            263; // reserved
        assert_eq!(vault_staker_withdrawal_ticket_size, sum_of_fields);
    }
}
