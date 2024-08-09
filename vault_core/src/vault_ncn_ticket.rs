use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::{
    result::{VaultCoreError, VaultCoreResult},
    AccountType,
};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct VaultNcnTicket {
    /// The account type
    account_type: AccountType,

    /// The vault account
    vault: Pubkey,

    /// The ncn account
    ncn: Pubkey,

    /// The index
    index: u64,

    /// The slot toggle
    state: SlotToggle,

    /// Reserved space
    reserved: [u8; 128],

    bump: u8,
}

impl VaultNcnTicket {
    pub const fn new(vault: Pubkey, ncn: Pubkey, index: u64, slot_added: u64, bump: u8) -> Self {
        Self {
            account_type: AccountType::VaultNcnTicket,
            vault,
            ncn,
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

    pub const fn index(&self) -> u64 {
        self.index
    }

    pub const fn state(&self) -> &SlotToggle {
        &self.state
    }

    pub fn deactivate(&mut self, slot: u64, epoch_length: u64) -> VaultCoreResult<()> {
        if self.state.deactivate(slot, epoch_length) {
            Ok(())
        } else {
            Err(VaultCoreError::VaultNcnTicketInactive)
        }
    }

    pub fn check_active_or_cooldown(&self, slot: u64, epoch_length: u64) -> VaultCoreResult<()> {
        if self.state.is_active_or_cooldown(slot, epoch_length) {
            Ok(())
        } else {
            msg!("VaultNcnTicket is not active or in cooldown");
            Err(VaultCoreError::VaultNcnTicketInactive)
        }
    }

    pub fn seeds(vault: &Pubkey, ncn: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"vault_ncn_ticket".to_vec(),
            vault.as_ref().to_vec(),
            ncn.as_ref().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        vault: &Pubkey,
        ncn: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(vault, ncn);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        vault: &Pubkey,
        ncn: &Pubkey,
    ) -> VaultCoreResult<Self> {
        if account.data_is_empty() {
            return Err(VaultCoreError::VaultNcnTicketEmpty);
        }
        if account.owner != program_id {
            return Err(VaultCoreError::VaultNcnTicketInvalidOwner);
        }

        let ticket = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| VaultCoreError::VaultNcnTicketInvalidData(e.to_string()))?;
        if ticket.account_type != AccountType::VaultNcnTicket {
            return Err(VaultCoreError::VaultNcnTicketInvalidAccountType);
        }

        let mut seeds = Self::seeds(vault, ncn);
        seeds.push(vec![ticket.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| VaultCoreError::VaultNcnTicketInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(VaultCoreError::VaultNcnTicketInvalidPda);
        }
        Ok(ticket)
    }
}

pub struct SanitizedVaultNcnTicket<'a, 'info> {
    account: &'a AccountInfo<'info>,
    vault_ncn_ticket: Box<VaultNcnTicket>,
}

impl<'a, 'info> SanitizedVaultNcnTicket<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        vault: &Pubkey,
        ncn: &Pubkey,
    ) -> VaultCoreResult<SanitizedVaultNcnTicket<'a, 'info>> {
        if expect_writable && !account.is_writable {
            return Err(VaultCoreError::VaultNcnTicektNotWritable);
        }
        let vault_ncn_ticket = Box::new(VaultNcnTicket::deserialize_checked(
            program_id, account, vault, ncn,
        )?);

        Ok(SanitizedVaultNcnTicket {
            account,
            vault_ncn_ticket,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn vault_ncn_ticket(&self) -> &VaultNcnTicket {
        &self.vault_ncn_ticket
    }

    pub fn vault_ncn_ticket_mut(&mut self) -> &mut VaultNcnTicket {
        &mut self.vault_ncn_ticket
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(
            &mut self.account.data.borrow_mut()[..],
            &self.vault_ncn_ticket,
        )?;
        Ok(())
    }
}
