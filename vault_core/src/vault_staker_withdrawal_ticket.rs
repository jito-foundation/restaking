use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

use crate::{
    result::{VaultCoreError, VaultCoreResult},
    AccountType,
};

/// Represents a pending withdrawal from a vault by a staker.
/// For every withdraw ticket, there's an associated token account owned by the withdraw ticket
/// with the staker's LRT.
#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
#[repr(C)]
pub struct VaultStakerWithdrawalTicket {
    /// The account type
    account_type: AccountType,

    /// The vault being withdrawn from
    vault: Pubkey,

    /// The staker withdrawing from the vault
    staker: Pubkey,

    /// The base account used as a PDA seed
    base: Pubkey,

    /// The amount of assets allocated for this staker's withdraw
    withdraw_allocation_amount: u64,

    /// The amount of LRT held in the VaultStakerWithdrawalTicket token account at the time of creation
    /// At first glance, this seems redundant, but it's necessary to prevent someone from depositing
    /// more LRT into the token account and skipping the withdraw queue.
    lrt_amount: u64,

    /// The slot the withdrawal was enqueued
    slot_unstaked: u64,

    /// The bump seed used to create the PDA
    bump: u8,
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
            account_type: AccountType::VaultStakerWithdrawalTicket,
            vault,
            staker,
            base,
            withdraw_allocation_amount,
            lrt_amount,
            slot_unstaked,
            bump,
        }
    }

    pub const fn vault(&self) -> Pubkey {
        self.vault
    }

    pub const fn staker(&self) -> Pubkey {
        self.staker
    }

    pub const fn base(&self) -> Pubkey {
        self.base
    }

    pub const fn slot_unstaked(&self) -> u64 {
        self.slot_unstaked
    }

    pub const fn withdraw_allocation_amount(&self) -> u64 {
        self.withdraw_allocation_amount
    }

    pub const fn lrt_amount(&self) -> u64 {
        self.lrt_amount
    }

    /// In order for the ticket to be withdrawable, it needs to be more than one **full** epoch
    /// since unstaking
    #[inline(always)]
    pub fn check_withdrawable(&self, slot: u64, epoch_length: u64) -> VaultCoreResult<()> {
        let current_epoch = slot.checked_div(epoch_length).unwrap(); // epoch_length is always > 0
        let epoch_unstaked = self.slot_unstaked.checked_div(epoch_length).unwrap();
        if epoch_unstaked
            .checked_add(1)
            .ok_or(VaultCoreError::VaultStakerWithdrawalTicketOverflow)?
            < current_epoch
        {
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

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        vault: &Pubkey,
        staker: &Pubkey,
    ) -> VaultCoreResult<Self> {
        if account.data_is_empty() {
            return Err(VaultCoreError::VaultStakerWithdrawalTicketEmpty);
        }
        if account.owner != program_id {
            return Err(VaultCoreError::VaultStakerWithdrawalTicketEmptyInvalidOwner);
        }

        let vault_staker_withdrawal_ticket =
            Self::deserialize(&mut account.data.borrow_mut().as_ref()).map_err(|e| {
                VaultCoreError::VaultStakerWithdrawalTicketEmptyInvalidData(e.to_string())
            })?;
        if vault_staker_withdrawal_ticket.account_type
            != AccountType::VaultStakerWithdrawalTicketEmpty
        {
            return Err(VaultCoreError::VaultStakerWithdrawalTicketEmptyInvalidAccountType);
        }

        let mut seeds = Self::seeds(vault, staker, &vault_staker_withdrawal_ticket.base());
        seeds.push(vec![vault_staker_withdrawal_ticket.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| VaultCoreError::VaultStakerWithdrawalTicketEmptyInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(VaultCoreError::VaultStakerWithdrawalTicketEmptyInvalidPda);
        }

        Ok(vault_staker_withdrawal_ticket)
    }
}

pub struct SanitizedVaultStakerWithdrawalTicket<'a, 'info> {
    account: &'a AccountInfo<'info>,
    vault_staker_withdrawal_ticket: VaultStakerWithdrawalTicket,
}

impl<'a, 'info> SanitizedVaultStakerWithdrawalTicket<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        vault: &Pubkey,
        staker: &Pubkey,
        expected_writable: bool,
    ) -> VaultCoreResult<Self> {
        if expected_writable && !account.is_writable {
            return Err(VaultCoreError::VaultStakerWithdrawalTicketNotWritable);
        }

        let vault_staker_withdrawal_ticket =
            VaultStakerWithdrawalTicket::deserialize_checked(program_id, account, vault, staker)?;

        Ok(SanitizedVaultStakerWithdrawalTicket {
            account,
            vault_staker_withdrawal_ticket,
        })
    }

    pub const fn vault_staker_withdrawal_ticket(&self) -> &VaultStakerWithdrawalTicket {
        &self.vault_staker_withdrawal_ticket
    }

    pub fn vault_staker_withdrawal_ticket_mut(&mut self) -> &mut VaultStakerWithdrawalTicket {
        &mut self.vault_staker_withdrawal_ticket
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }
}
