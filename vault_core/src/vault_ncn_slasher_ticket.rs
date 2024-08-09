use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::{
    result::{VaultCoreError, VaultCoreResult},
    AccountType,
};

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct VaultNcnSlasherTicket {
    /// The account type
    account_type: AccountType,

    vault: Pubkey,

    ncn: Pubkey,

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

impl VaultNcnSlasherTicket {
    pub const fn new(
        vault: Pubkey,
        ncn: Pubkey,
        slasher: Pubkey,
        max_slashable_per_epoch: u64,
        index: u64,
        slot_added: u64,
        bump: u8,
    ) -> Self {
        Self {
            account_type: AccountType::VaultNcnSlasherTicket,
            vault,
            ncn,
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

    pub const fn ncn(&self) -> Pubkey {
        self.ncn
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
            msg!("VaultNcnSlasherTicket is not active or in cooldown");
            Err(VaultCoreError::VaultNcnSlasherTicketInactive)
        }
    }

    pub fn seeds(vault: &Pubkey, ncn: &Pubkey, slasher: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"vault_slasher_ticket".to_vec(),
            vault.as_ref().to_vec(),
            ncn.as_ref().to_vec(),
            slasher.as_ref().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        vault: &Pubkey,
        ncn: &Pubkey,
        slasher: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(vault, ncn, slasher);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        vault: &Pubkey,
        ncn: &Pubkey,
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
        if ticket.account_type != AccountType::VaultNcnSlasherTicket {
            return Err(VaultCoreError::VaultSlasherTicketInvalidAccountType);
        }

        let mut seeds = Self::seeds(vault, ncn, slasher);
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

pub struct SanitizedVaultNcnSlasherTicket<'a, 'info> {
    account: &'a AccountInfo<'info>,
    vault_slasher_ticket: Box<VaultNcnSlasherTicket>,
}

impl<'a, 'info> SanitizedVaultNcnSlasherTicket<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        vault: &Pubkey,
        ncn: &Pubkey,
        slasher: &Pubkey,
    ) -> VaultCoreResult<SanitizedVaultNcnSlasherTicket<'a, 'info>> {
        if expect_writable && !account.is_writable {
            return Err(VaultCoreError::VaultSlasherTicketNotWritable);
        }
        let vault_slasher_ticket = Box::new(VaultNcnSlasherTicket::deserialize_checked(
            program_id, account, vault, ncn, slasher,
        )?);

        Ok(SanitizedVaultNcnSlasherTicket {
            account,
            vault_slasher_ticket,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn vault_ncn_slasher_ticket(&self) -> &VaultNcnSlasherTicket {
        &self.vault_slasher_ticket
    }

    pub fn vault_slasher_ticket_mut(&mut self) -> &mut VaultNcnSlasherTicket {
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
