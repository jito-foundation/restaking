use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    result::{VaultCoreError, VaultCoreResult},
    AccountType,
};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct VaultAvsTicket {
    /// The account type
    account_type: AccountType,

    /// The vault account
    vault: Pubkey,

    /// The avs account
    avs: Pubkey,

    /// The index
    index: u64,

    /// The slot toggle
    state: SlotToggle,

    /// Reserved space
    reserved: [u8; 128],

    bump: u8,
}

impl VaultAvsTicket {
    pub const fn new(vault: Pubkey, avs: Pubkey, index: u64, slot_added: u64, bump: u8) -> Self {
        Self {
            account_type: AccountType::VaultAvsTicket,
            vault,
            avs,
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
            Err(VaultCoreError::VaultAvsTicketInactive)
        }
    }

    pub fn check_active(&self, slot: u64, epoch_length: u64) -> VaultCoreResult<()> {
        if self.state.is_active(slot, epoch_length) {
            Ok(())
        } else {
            Err(VaultCoreError::VaultAvsTicketInactive)
        }
    }

    pub fn seeds(vault: &Pubkey, avs: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"vault_avs_ticket".to_vec(),
            vault.as_ref().to_vec(),
            avs.as_ref().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        vault: &Pubkey,
        avs: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(vault, avs);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        vault: &Pubkey,
        avs: &Pubkey,
    ) -> VaultCoreResult<Self> {
        if account.data_is_empty() {
            return Err(VaultCoreError::VaultAvsTicketEmpty);
        }
        if account.owner != program_id {
            return Err(VaultCoreError::VaultAvsTicketInvalidOwner);
        }

        let ticket = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| VaultCoreError::VaultAvsTicketInvalidData(e.to_string()))?;
        if ticket.account_type != AccountType::VaultAvsTicket {
            return Err(VaultCoreError::VaultAvsTicketInvalidAccountType);
        }

        let mut seeds = Self::seeds(vault, avs);
        seeds.push(vec![ticket.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| VaultCoreError::VaultAvsTicketInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(VaultCoreError::VaultAvsTicketInvalidPda);
        }
        Ok(ticket)
    }
}

pub struct SanitizedVaultAvsTicket<'a, 'info> {
    account: &'a AccountInfo<'info>,
    vault_avs_ticket: Box<VaultAvsTicket>,
}

impl<'a, 'info> SanitizedVaultAvsTicket<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        vault: &Pubkey,
        avs: &Pubkey,
    ) -> VaultCoreResult<SanitizedVaultAvsTicket<'a, 'info>> {
        if expect_writable && !account.is_writable {
            return Err(VaultCoreError::VaultAvsTicektNotWritable);
        }
        let vault_avs_ticket = Box::new(VaultAvsTicket::deserialize_checked(
            program_id, account, vault, avs,
        )?);

        Ok(SanitizedVaultAvsTicket {
            account,
            vault_avs_ticket,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn vault_avs_ticket(&self) -> &VaultAvsTicket {
        &self.vault_avs_ticket
    }

    pub fn vault_avs_ticket_mut(&mut self) -> &mut VaultAvsTicket {
        &mut self.vault_avs_ticket
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(
            &mut self.account.data.borrow_mut()[..],
            &self.vault_avs_ticket,
        )?;
        Ok(())
    }
}
