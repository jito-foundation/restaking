use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use solana_program::{msg, pubkey::Pubkey};

use crate::result::{VaultCoreError, VaultCoreResult};

impl Discriminator for VaultStakerWithdrawalTicket {
    const DISCRIMINATOR: u8 = 7;
}

/// Represents a pending withdrawal from a vault by a staker.
/// For every withdraw ticket, there's an associated token account owned by the withdraw ticket
/// with the staker's LRT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct VaultStakerWithdrawalTicket {
    /// The vault being withdrawn from
    pub vault: Pubkey,

    /// The staker withdrawing from the vault
    pub staker: Pubkey,

    /// The base account used as a PDA seed
    pub base: Pubkey,

    /// The amount of assets allocated for this staker's withdraw
    pub withdraw_allocation_amount: u64,

    /// The amount of LRT held in the VaultStakerWithdrawalTicket token account at the time of creation
    /// At first glance, this seems redundant, but it's necessary to prevent someone from depositing
    /// more LRT into the token account and skipping the withdraw queue.
    pub lrt_amount: u64,

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
        withdraw_allocation_amount: u64,
        lrt_amount: u64,
        slot_unstaked: u64,
        bump: u8,
    ) -> Self {
        Self {
            vault,
            staker,
            base,
            withdraw_allocation_amount,
            lrt_amount,
            slot_unstaked,
            bump,
            reserved: [0; 7],
        }
    }

    /// In order for the ticket to be withdrawable, it needs to be more than one **full** epoch
    /// since unstaking
    #[inline(always)]
    pub fn check_withdrawable(&self, slot: u64, epoch_length: u64) -> VaultCoreResult<()> {
        let current_epoch = slot.checked_div(epoch_length).unwrap(); // epoch_length is always > 0
        let epoch_unstaked = self.slot_unstaked.checked_div(epoch_length).unwrap();
        if current_epoch
            <= epoch_unstaked
                .checked_add(1)
                .ok_or(VaultCoreError::VaultStakerWithdrawalTicketOverflow)?
        {
            msg!(
                "current_epoch: {:?}, epoch_unstaked: {:?}",
                current_epoch,
                epoch_unstaked
            );
            return Err(VaultCoreError::VaultStakerWithdrawalTicketNotWithdrawable);
        }
        Ok(())
    }

    pub fn seeds(vault: &Pubkey, staker: &Pubkey, base: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"vault_staker_withdrawal_ticket".to_vec(),
            vault.to_bytes().to_vec(),
            staker.to_bytes().to_vec(),
            base.to_bytes().to_vec(),
        ])
    }

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
}
