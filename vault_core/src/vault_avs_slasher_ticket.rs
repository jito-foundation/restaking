use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    result::{VaultCoreError, VaultCoreResult},
    AccountType,
};

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct VaultAvsSlasherTicket {
    /// The account type
    account_type: AccountType,

    vault: Pubkey,

    avs: Pubkey,

    slasher: Pubkey,

    max_slashable_per_epoch: u64,

    /// The index
    index: u64,

    /// The slot toggle
    state: SlotToggle,

    /// Reserved space
    reserved: [u8; 128],

    bump: u8,
}

impl VaultAvsSlasherTicket {
    pub const fn new(
        vault: Pubkey,
        avs: Pubkey,
        slasher: Pubkey,
        max_slashable_per_epoch: u64,
        index: u64,
        slot_added: u64,
        bump: u8,
    ) -> Self {
        Self {
            account_type: AccountType::VaultAvsSlasherTicket,
            vault,
            avs,
            slasher,
            max_slashable_per_epoch,
            index,
            state: SlotToggle::new(slot_added),
            reserved: [0; 128],
            bump,
        }
    }

    pub const fn vault(&self) -> Pubkey {
        self.vault
    }

    pub const fn avs(&self) -> Pubkey {
        self.avs
    }

    pub const fn slasher(&self) -> Pubkey {
        self.slasher
    }

    pub const fn index(&self) -> u64 {
        self.index
    }

    pub const fn max_slashable_per_epoch(&self) -> u64 {
        self.max_slashable_per_epoch
    }

    pub const fn state(&self) -> &SlotToggle {
        &self.state
    }

    pub fn check_active_or_cooldown(&self, slot: u64, epoch_length: u64) -> VaultCoreResult<()> {
        if self.state.is_active_or_cooldown(slot, epoch_length) {
            Ok(())
        } else {
            Err(VaultCoreError::VaultAvsSlasherTicketInactive)
        }
    }

    pub fn seeds(vault: &Pubkey, avs: &Pubkey, slasher: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"vault_slasher_ticket".to_vec(),
            vault.as_ref().to_vec(),
            avs.as_ref().to_vec(),
            slasher.as_ref().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        vault: &Pubkey,
        avs: &Pubkey,
        slasher: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(vault, avs, slasher);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        vault: &Pubkey,
        avs: &Pubkey,
        slasher: &Pubkey,
    ) -> VaultCoreResult<Self> {
        if account.data_is_empty() {
            return Err(VaultCoreError::VaultSlasherTicketEmpty);
        }
        if account.owner != program_id {
            return Err(VaultCoreError::VaultSlasherTicketInvalidOwner);
        }

        let ticket = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| VaultCoreError::VaultSlasherTicketInvalidData(e.to_string()))?;
        if ticket.account_type != AccountType::VaultAvsSlasherTicket {
            return Err(VaultCoreError::VaultSlasherTicketInvalidAccountType);
        }

        let mut seeds = Self::seeds(vault, avs, slasher);
        seeds.push(vec![ticket.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| VaultCoreError::VaultSlasherTicketInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(VaultCoreError::VaultSlasherTicketInvalidPda);
        }
        Ok(ticket)
    }
}

pub struct SanitizedVaultAvsSlasherTicket<'a, 'info> {
    account: &'a AccountInfo<'info>,
    vault_slasher_ticket: Box<VaultAvsSlasherTicket>,
}

impl<'a, 'info> SanitizedVaultAvsSlasherTicket<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        vault: &Pubkey,
        avs: &Pubkey,
        slasher: &Pubkey,
    ) -> VaultCoreResult<SanitizedVaultAvsSlasherTicket<'a, 'info>> {
        if expect_writable && !account.is_writable {
            return Err(VaultCoreError::VaultSlasherTicketNotWritable);
        }
        let vault_slasher_ticket = Box::new(VaultAvsSlasherTicket::deserialize_checked(
            program_id, account, vault, avs, slasher,
        )?);

        Ok(SanitizedVaultAvsSlasherTicket {
            account,
            vault_slasher_ticket,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn vault_avs_slasher_ticket(&self) -> &VaultAvsSlasherTicket {
        &self.vault_slasher_ticket
    }

    pub fn vault_slasher_ticket_mut(&mut self) -> &mut VaultAvsSlasherTicket {
        &mut self.vault_slasher_ticket
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(
            &mut self.account.data.borrow_mut()[..],
            &self.vault_slasher_ticket,
        )?;
        Ok(())
    }
}
