use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    AccountType,
};

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
#[repr(C)]
pub struct AvsVaultSlasherTicket {
    account_type: AccountType,

    /// The AVS
    avs: Pubkey,

    /// The vault account this slasher can slash
    vault: Pubkey,

    /// The slasher signer
    slasher: Pubkey,

    /// The max slashable funds per epoch
    max_slashable_per_epoch: u64,

    /// The index
    index: u64,

    /// State of the AVS slasher
    state: SlotToggle,

    /// Reserved space
    reserved: [u8; 128],

    /// The bump seed for the PDA
    bump: u8,
}

impl AvsVaultSlasherTicket {
    pub const fn new(
        avs: Pubkey,
        vault: Pubkey,
        slasher: Pubkey,
        max_slashable_per_epoch: u64,
        index: u64,
        slot_added: u64,
        bump: u8,
    ) -> Self {
        Self {
            account_type: AccountType::AvsVaultSlasherTicket,
            avs,
            vault,
            slasher,
            max_slashable_per_epoch,
            index,
            state: SlotToggle::new(slot_added),
            reserved: [0; 128],
            bump,
        }
    }

    pub const fn avs(&self) -> Pubkey {
        self.avs
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

    pub const fn check_active(&self, slot: u64) -> RestakingCoreResult<()> {
        if self.state.is_active(slot) {
            Ok(())
        } else {
            Err(RestakingCoreError::AvsVaultSlasherTicketInactive)
        }
    }

    pub fn deactivate(&mut self, slot: u64) -> RestakingCoreResult<()> {
        if self.state.deactivate(slot) {
            Ok(())
        } else {
            Err(RestakingCoreError::AvsVaultSlasherTicketInactive)
        }
    }

    pub fn seeds(avs: &Pubkey, vault: &Pubkey, slasher: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"avs_slasher_ticket".to_vec(),
            avs.as_ref().to_vec(),
            vault.as_ref().to_vec(),
            slasher.as_ref().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        avs: &Pubkey,
        vault: &Pubkey,
        slasher: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(avs, vault, slasher);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        avs: &Pubkey,
        vault: &Pubkey,
        slasher: &Pubkey,
    ) -> RestakingCoreResult<Self> {
        if account.data_is_empty() {
            return Err(RestakingCoreError::AvsSlasherTicketEmpty);
        }
        if account.owner != program_id {
            return Err(RestakingCoreError::AvsSlasherTicketInvalidOwner);
        }

        let avs_slasher_ticket = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| RestakingCoreError::AvsSlasherTicketInvalidData(e.to_string()))?;
        if avs_slasher_ticket.account_type != AccountType::AvsVaultSlasherTicket {
            return Err(RestakingCoreError::AvsSlasherTicketInvalidAccountType);
        }

        let mut seeds = Self::seeds(avs, vault, slasher);
        seeds.push(vec![avs_slasher_ticket.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| RestakingCoreError::AvsSlasherTicketInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(RestakingCoreError::AvsSlasherTicketInvalidPda);
        }

        Ok(avs_slasher_ticket)
    }
}

pub struct SanitizedAvsVaultSlasherTicket<'a, 'info> {
    account: &'a AccountInfo<'info>,
    avs_slasher_ticket: Box<AvsVaultSlasherTicket>,
}

impl<'a, 'info> SanitizedAvsVaultSlasherTicket<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        avs: &Pubkey,
        vault: &Pubkey,
        slasher: &Pubkey,
    ) -> RestakingCoreResult<Self> {
        if expect_writable && !account.is_writable {
            return Err(RestakingCoreError::AvsSlasherTicketNotWritable);
        }

        let avs_slasher_ticket = Box::new(AvsVaultSlasherTicket::deserialize_checked(
            program_id, account, avs, vault, slasher,
        )?);

        Ok(Self {
            account,
            avs_slasher_ticket,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn avs_vault_slasher_ticket(&self) -> &AvsVaultSlasherTicket {
        &self.avs_slasher_ticket
    }

    pub fn avs_vault_slasher_ticket_mut(&mut self) -> &mut AvsVaultSlasherTicket {
        &mut self.avs_slasher_ticket
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(
            &mut self.account.data.borrow_mut()[..],
            &self.avs_slasher_ticket,
        )?;
        Ok(())
    }
}
