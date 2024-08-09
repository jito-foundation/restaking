use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    AccountType,
};

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
#[repr(C)]
pub struct AvsVaultTicket {
    /// The account type
    account_type: AccountType,

    /// The AVS
    avs: Pubkey,

    /// The vault account
    vault: Pubkey,

    index: u64,

    state: SlotToggle,

    /// Reserved space
    reserved: [u8; 128],

    bump: u8,
}

impl AvsVaultTicket {
    pub const fn new(avs: Pubkey, vault: Pubkey, index: u64, slot_added: u64, bump: u8) -> Self {
        Self {
            account_type: AccountType::AvsVaultTicket,
            avs,
            vault,
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
            msg!("AvsVaultTicket is not active or in cooldown");
            Err(RestakingCoreError::AvsVaultTicketInactive)
        }
    }

    pub fn deactivate(&mut self, slot: u64, epoch_length: u64) -> RestakingCoreResult<()> {
        if self.state.deactivate(slot, epoch_length) {
            Ok(())
        } else {
            Err(RestakingCoreError::AvsVaultTicketInactive)
        }
    }

    pub const fn index(&self) -> u64 {
        self.index
    }

    pub const fn bump(&self) -> u8 {
        self.bump
    }

    pub fn seeds(avs: &Pubkey, vault: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"avs_vault_ticket".to_vec(),
            avs.to_bytes().to_vec(),
            vault.to_bytes().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        avs: &Pubkey,
        vault: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(avs, vault);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        avs: &Pubkey,
        vault: &Pubkey,
    ) -> RestakingCoreResult<Self> {
        if account.data_is_empty() {
            return Err(RestakingCoreError::AvsVaultTicketEmpty);
        }
        if account.owner != program_id {
            return Err(RestakingCoreError::AvsVaultTicketInvalidOwner);
        }

        let avs_vault_ticket = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| RestakingCoreError::AvsVaultTicketInvalidData(e.to_string()))?;
        if avs_vault_ticket.account_type != AccountType::AvsVaultTicket {
            return Err(RestakingCoreError::AvsVaultTicketInvalidAccountType);
        }

        let mut seeds = Self::seeds(avs, vault);
        seeds.push(vec![avs_vault_ticket.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| RestakingCoreError::AvsVaultTicketInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(RestakingCoreError::AvsVaultTicketInvalidPda);
        }

        Ok(avs_vault_ticket)
    }
}

pub struct SanitizedAvsVaultTicket<'a, 'info> {
    account: &'a AccountInfo<'info>,
    avs_vault_ticket: Box<AvsVaultTicket>,
}

impl<'a, 'info> SanitizedAvsVaultTicket<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        avs: &Pubkey,
        vault: &Pubkey,
    ) -> RestakingCoreResult<Self> {
        if expect_writable && !account.is_writable {
            return Err(RestakingCoreError::AvsVaultTicketNotWritable);
        }

        let avs_vault_ticket = Box::new(AvsVaultTicket::deserialize_checked(
            program_id, account, avs, vault,
        )?);

        Ok(Self {
            account,
            avs_vault_ticket,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn avs_vault_ticket(&self) -> &AvsVaultTicket {
        &self.avs_vault_ticket
    }

    pub fn avs_vault_ticket_mut(&mut self) -> &mut AvsVaultTicket {
        &mut self.avs_vault_ticket
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(
            &mut self.account.data.borrow_mut()[..],
            &self.avs_vault_ticket,
        )?;
        Ok(())
    }
}
