use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    AccountType,
};

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
#[repr(C)]
pub struct NcnVaultSlasherTicket {
    account_type: AccountType,

    /// The NCN
    ncn: Pubkey,

    /// The vault account this slasher can slash
    vault: Pubkey,

    /// The slasher signer
    slasher: Pubkey,

    /// The max slashable funds per epoch
    max_slashable_per_epoch: u64,

    /// The index
    index: u64,

    /// State of the NCN slasher
    state: SlotToggle,

    /// Reserved space
    reserved: [u8; 128],

    /// The bump seed for the PDA
    bump: u8,
}

impl NcnVaultSlasherTicket {
    pub const fn new(
        ncn: Pubkey,
        vault: Pubkey,
        slasher: Pubkey,
        max_slashable_per_epoch: u64,
        index: u64,
        slot_added: u64,
        bump: u8,
    ) -> Self {
        Self {
            account_type: AccountType::NcnVaultSlasherTicket,
            ncn,
            vault,
            slasher,
            max_slashable_per_epoch,
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

    pub const fn bump(&self) -> u8 {
        self.bump
    }

    pub fn check_active_or_cooldown(
        &self,
        slot: u64,
        epoch_length: u64,
    ) -> RestakingCoreResult<()> {
        if self.state.is_active_or_cooldown(slot, epoch_length) {
            Ok(())
        } else {
            msg!("NcnVaultSlasherTicket is not active or in cooldown");
            Err(RestakingCoreError::NcnVaultSlasherTicketInactive)
        }
    }

    pub fn deactivate(&mut self, slot: u64, epoch_length: u64) -> RestakingCoreResult<()> {
        if self.state.deactivate(slot, epoch_length) {
            Ok(())
        } else {
            Err(RestakingCoreError::NcnVaultSlasherTicketInactive)
        }
    }

    pub fn seeds(ncn: &Pubkey, vault: &Pubkey, slasher: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"ncn_slasher_ticket".to_vec(),
            ncn.as_ref().to_vec(),
            vault.as_ref().to_vec(),
            slasher.as_ref().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        ncn: &Pubkey,
        vault: &Pubkey,
        slasher: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(ncn, vault, slasher);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        ncn: &Pubkey,
        vault: &Pubkey,
        slasher: &Pubkey,
    ) -> RestakingCoreResult<Self> {
        if account.data_is_empty() {
            return Err(RestakingCoreError::NcnSlasherTicketEmpty);
        }
        if account.owner != program_id {
            return Err(RestakingCoreError::NcnSlasherTicketInvalidOwner);
        }

        let ncn_slasher_ticket = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| RestakingCoreError::NcnSlasherTicketInvalidData(e.to_string()))?;
        if ncn_slasher_ticket.account_type != AccountType::NcnVaultSlasherTicket {
            return Err(RestakingCoreError::NcnSlasherTicketInvalidAccountType);
        }

        let mut seeds = Self::seeds(ncn, vault, slasher);
        seeds.push(vec![ncn_slasher_ticket.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| RestakingCoreError::NcnSlasherTicketInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(RestakingCoreError::NcnSlasherTicketInvalidPda);
        }

        Ok(ncn_slasher_ticket)
    }
}

pub struct SanitizedNcnVaultSlasherTicket<'a, 'info> {
    account: &'a AccountInfo<'info>,
    ncn_slasher_ticket: Box<NcnVaultSlasherTicket>,
}

impl<'a, 'info> SanitizedNcnVaultSlasherTicket<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        ncn: &Pubkey,
        vault: &Pubkey,
        slasher: &Pubkey,
    ) -> RestakingCoreResult<Self> {
        if expect_writable && !account.is_writable {
            return Err(RestakingCoreError::NcnSlasherTicketNotWritable);
        }

        let ncn_slasher_ticket = Box::new(NcnVaultSlasherTicket::deserialize_checked(
            program_id, account, ncn, vault, slasher,
        )?);

        Ok(Self {
            account,
            ncn_slasher_ticket,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn ncn_vault_slasher_ticket(&self) -> &NcnVaultSlasherTicket {
        &self.ncn_slasher_ticket
    }

    pub fn ncn_vault_slasher_ticket_mut(&mut self) -> &mut NcnVaultSlasherTicket {
        &mut self.ncn_slasher_ticket
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(
            &mut self.account.data.borrow_mut()[..],
            &self.ncn_slasher_ticket,
        )?;
        Ok(())
    }
}
