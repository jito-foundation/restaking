use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    AccountType,
};

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
#[repr(C)]
pub struct OperatorVaultTicket {
    /// The account type
    account_type: AccountType,

    /// The operator account
    operator: Pubkey,

    /// The vault account
    vault: Pubkey,

    /// The index
    index: u64,

    /// The slot toggle
    state: SlotToggle,

    /// Reserved space
    reserved: [u8; 128],

    bump: u8,
}

impl OperatorVaultTicket {
    pub const fn new(
        operator: Pubkey,
        vault: Pubkey,
        index: u64,
        slot_added: u64,
        bump: u8,
    ) -> Self {
        Self {
            account_type: AccountType::OperatorVaultTicket,
            operator,
            vault,
            index,
            state: SlotToggle::new(slot_added),
            reserved: [0; 128],
            bump,
        }
    }

    pub const fn operator(&self) -> Pubkey {
        self.operator
    }

    pub const fn vault(&self) -> Pubkey {
        self.vault
    }

    pub const fn state(&self) -> &SlotToggle {
        &self.state
    }

    pub fn deactivate(&mut self, slot: u64, epoch_length: u64) -> RestakingCoreResult<()> {
        if self.state.deactivate(slot, epoch_length) {
            Ok(())
        } else {
            Err(RestakingCoreError::OperatorVaultTicketAlreadyDeactivated)
        }
    }

    pub fn check_active_or_cooldown(
        &self,
        slot: u64,
        epoch_length: u64,
    ) -> RestakingCoreResult<()> {
        if self.state.is_active_or_cooldown(slot, epoch_length) {
            Ok(())
        } else {
            msg!("OperatorVaultTicket is not active or in cooldown");
            Err(RestakingCoreError::OperatorVaultTicketInactive)
        }
    }

    pub const fn index(&self) -> u64 {
        self.index
    }

    pub const fn bump(&self) -> u8 {
        self.bump
    }

    pub fn seeds(operator: &Pubkey, vault: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"operator_vault_ticket".to_vec(),
            operator.to_bytes().to_vec(),
            vault.to_bytes().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        operator: &Pubkey,
        vault: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(operator, vault);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        operator: &Pubkey,
        vault: &Pubkey,
    ) -> RestakingCoreResult<Self> {
        if account.data_is_empty() {
            return Err(RestakingCoreError::OperatorVaultTicketEmpty);
        }
        if account.owner != program_id {
            return Err(RestakingCoreError::OperatorVaultTicketInvalidOwner);
        }

        let ncn_vault_ticket = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| RestakingCoreError::OperatorVaultTicketInvalidData(e.to_string()))?;
        if ncn_vault_ticket.account_type != AccountType::OperatorVaultTicket {
            return Err(RestakingCoreError::OperatorVaultTicketInvalidAccountType);
        }

        let mut seeds = Self::seeds(operator, vault);
        seeds.push(vec![ncn_vault_ticket.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| RestakingCoreError::OperatorVaultTicketInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(RestakingCoreError::OperatorVaultTicketInvalidPda);
        }

        Ok(ncn_vault_ticket)
    }
}

pub struct SanitizedOperatorVaultTicket<'a, 'info> {
    account: &'a AccountInfo<'info>,
    operator_vault_ticket: Box<OperatorVaultTicket>,
}

impl<'a, 'info> SanitizedOperatorVaultTicket<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        operator: &Pubkey,
        ncn: &Pubkey,
    ) -> RestakingCoreResult<Self> {
        if expect_writable && !account.is_writable {
            return Err(RestakingCoreError::OperatorVaultTicketNotWritable);
        }

        let operator_vault_ticket = Box::new(OperatorVaultTicket::deserialize_checked(
            program_id, account, operator, ncn,
        )?);

        Ok(Self {
            account,
            operator_vault_ticket,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn operator_vault_ticket(&self) -> &OperatorVaultTicket {
        &self.operator_vault_ticket
    }

    pub fn operator_vault_ticket_mut(&mut self) -> &mut OperatorVaultTicket {
        &mut self.operator_vault_ticket
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(
            &mut &mut self.account.data.borrow_mut()[..],
            &self.operator_vault_ticket,
        )?;
        Ok(())
    }
}
