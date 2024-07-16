use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    result::{VaultCoreError, VaultCoreResult},
    AccountType,
};

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct VaultOperatorTicket {
    /// The account type
    account_type: AccountType,

    /// The vault account
    vault: Pubkey,

    /// The operator account
    operator: Pubkey,

    /// The index
    index: u64,

    /// The slot toggle
    state: SlotToggle,

    /// Reserved space
    reserved: [u8; 128],

    bump: u8,
}

impl VaultOperatorTicket {
    pub const fn new(
        vault: Pubkey,
        operator: Pubkey,
        index: u64,
        slot_added: u64,
        bump: u8,
    ) -> Self {
        Self {
            account_type: AccountType::VaultOperatorTicket,
            vault,
            operator,
            index,
            state: SlotToggle::new(slot_added),
            reserved: [0; 128],
            bump,
        }
    }

    pub const fn vault(&self) -> Pubkey {
        self.vault
    }

    pub const fn operator(&self) -> Pubkey {
        self.operator
    }

    pub const fn index(&self) -> u64 {
        self.index
    }

    pub const fn state(&self) -> &SlotToggle {
        &self.state
    }

    pub const fn check_active(&self, slot: u64) -> VaultCoreResult<()> {
        if self.state.is_active(slot) {
            Ok(())
        } else {
            Err(VaultCoreError::VaultOperatorTicketInactive)
        }
    }

    pub fn deactivate(&mut self, slot: u64) -> VaultCoreResult<()> {
        if self.state.deactivate(slot) {
            Ok(())
        } else {
            Err(VaultCoreError::VaultOperatorTicketAlreadyDeactivated)
        }
    }

    pub fn seeds(vault: &Pubkey, operator: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"vault_operator_ticket".to_vec(),
            vault.as_ref().to_vec(),
            operator.as_ref().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        vault: &Pubkey,
        operator: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(vault, operator);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        vault: &Pubkey,
        operator: &Pubkey,
    ) -> VaultCoreResult<Self> {
        if account.data_is_empty() {
            return Err(VaultCoreError::VaultOperatorTicketEmpty);
        }
        if account.owner != program_id {
            return Err(VaultCoreError::VaultOperatorTicketInvalidOwner);
        }

        let ticket = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| VaultCoreError::VaultOperatorTicketInvalidData(e.to_string()))?;
        if ticket.account_type != AccountType::VaultOperatorTicket {
            return Err(VaultCoreError::VaultOperatorTicketInvalidAccountType);
        }

        let mut seeds = Self::seeds(vault, operator);
        seeds.push(vec![ticket.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| VaultCoreError::VaultOperatorTicketInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(VaultCoreError::VaultOperatorTicketInvalidPda);
        }
        Ok(ticket)
    }
}

pub struct SanitizedVaultOperatorTicket<'a, 'info> {
    account: &'a AccountInfo<'info>,
    vault_operator_ticket: Box<VaultOperatorTicket>,
}

impl<'a, 'info> SanitizedVaultOperatorTicket<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        vault: &Pubkey,
        avs: &Pubkey,
    ) -> VaultCoreResult<SanitizedVaultOperatorTicket<'a, 'info>> {
        if expect_writable && !account.is_writable {
            return Err(VaultCoreError::VaultOperatorTicketNotWritable);
        }
        let vault_operator_ticket = Box::new(VaultOperatorTicket::deserialize_checked(
            program_id, account, vault, avs,
        )?);

        Ok(SanitizedVaultOperatorTicket {
            account,
            vault_operator_ticket,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn vault_operator_ticket(&self) -> &VaultOperatorTicket {
        &self.vault_operator_ticket
    }

    pub fn vault_operator_ticket_mut(&mut self) -> &mut VaultOperatorTicket {
        &mut self.vault_operator_ticket
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(
            &mut self.account.data.borrow_mut()[..],
            &self.vault_operator_ticket,
        )?;
        Ok(())
    }
}
