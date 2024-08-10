use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    AccountType,
};

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
#[repr(C)]
pub struct NcnVaultTicket {
    /// The account type
    account_type: AccountType,

    /// The NCN
    ncn: Pubkey,

    /// The vault account
    vault: Pubkey,

    index: u64,

    state: SlotToggle,

    /// Reserved space
    reserved: [u8; 128],

    bump: u8,
}

impl NcnVaultTicket {
    pub const fn new(ncn: Pubkey, vault: Pubkey, index: u64, slot_added: u64, bump: u8) -> Self {
        Self {
            account_type: AccountType::NcnVaultTicket,
            ncn,
            vault,
            index,
            state: SlotToggle::new(slot_added),
            reserved: [0; 128],
            bump,
        }
    }

    pub const fn ncn(&self) -> Pubkey {
        self.ncn
    }

    pub const fn vault(&self) -> Pubkey {
        self.vault
    }

    pub const fn state(&self) -> &SlotToggle {
        &self.state
    }

    pub fn check_active_or_cooldown(
        &self,
        slot: u64,
        epoch_length: u64,
    ) -> RestakingCoreResult<()> {
        if self.state.is_active_or_cooldown(slot, epoch_length) {
            Ok(())
        } else {
            msg!("NcnVaultTicket is not active or in cooldown");
            Err(RestakingCoreError::NcnVaultTicketInactive)
        }
    }

    pub fn deactivate(&mut self, slot: u64, epoch_length: u64) -> RestakingCoreResult<()> {
        if self.state.deactivate(slot, epoch_length) {
            Ok(())
        } else {
            Err(RestakingCoreError::NcnVaultTicketInactive)
        }
    }

    pub const fn index(&self) -> u64 {
        self.index
    }

    pub const fn bump(&self) -> u8 {
        self.bump
    }

    pub fn seeds(ncn: &Pubkey, vault: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"ncn_vault_ticket".to_vec(),
            ncn.to_bytes().to_vec(),
            vault.to_bytes().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        ncn: &Pubkey,
        vault: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(ncn, vault);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        ncn: &Pubkey,
        vault: &Pubkey,
    ) -> RestakingCoreResult<Self> {
        if account.data_is_empty() {
            return Err(RestakingCoreError::NcnVaultTicketEmpty);
        }
        if account.owner != program_id {
            return Err(RestakingCoreError::NcnVaultTicketInvalidOwner);
        }

        let ncn_vault_ticket = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| RestakingCoreError::NcnVaultTicketInvalidData(e.to_string()))?;
        if ncn_vault_ticket.account_type != AccountType::NcnVaultTicket {
            return Err(RestakingCoreError::NcnVaultTicketInvalidAccountType);
        }

        let mut seeds = Self::seeds(ncn, vault);
        seeds.push(vec![ncn_vault_ticket.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| RestakingCoreError::NcnVaultTicketInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(RestakingCoreError::NcnVaultTicketInvalidPda);
        }

        Ok(ncn_vault_ticket)
    }
}

pub struct SanitizedNcnVaultTicket<'a, 'info> {
    account: &'a AccountInfo<'info>,
    ncn_vault_ticket: Box<NcnVaultTicket>,
}

impl<'a, 'info> SanitizedNcnVaultTicket<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        ncn: &Pubkey,
        vault: &Pubkey,
    ) -> RestakingCoreResult<Self> {
        if expect_writable && !account.is_writable {
            return Err(RestakingCoreError::NcnVaultTicketNotWritable);
        }

        let ncn_vault_ticket = Box::new(NcnVaultTicket::deserialize_checked(
            program_id, account, ncn, vault,
        )?);

        Ok(Self {
            account,
            ncn_vault_ticket,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn ncn_vault_ticket(&self) -> &NcnVaultTicket {
        &self.ncn_vault_ticket
    }

    pub fn ncn_vault_ticket_mut(&mut self) -> &mut NcnVaultTicket {
        &mut self.ncn_vault_ticket
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(
            &mut self.account.data.borrow_mut()[..],
            &self.ncn_vault_ticket,
        )?;
        Ok(())
    }
}
