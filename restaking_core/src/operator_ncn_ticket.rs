use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    AccountType,
};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct OperatorNcnTicket {
    account_type: AccountType,

    /// The operator account
    operator: Pubkey,

    /// The NCN account
    ncn: Pubkey,

    index: u64,

    state: SlotToggle,

    /// Reserved space
    reserved: [u8; 128],

    bump: u8,
}

impl OperatorNcnTicket {
    pub const fn new(operator: Pubkey, ncn: Pubkey, index: u64, slot_added: u64, bump: u8) -> Self {
        Self {
            account_type: AccountType::OperatorNcnTicket,
            operator,
            ncn,
            index,
            state: SlotToggle::new(slot_added),
            reserved: [0; 128],
            bump,
        }
    }

    pub const fn operator(&self) -> Pubkey {
        self.operator
    }

    pub const fn ncn(&self) -> Pubkey {
        self.ncn
    }

    pub const fn state(&self) -> &SlotToggle {
        &self.state
    }

    pub fn deactivate(&mut self, slot: u64, epoch_length: u64) -> RestakingCoreResult<()> {
        if self.state.deactivate(slot, epoch_length) {
            Ok(())
        } else {
            Err(RestakingCoreError::OperatorNcnTicketAlreadyInactive)
        }
    }

    pub const fn index(&self) -> u64 {
        self.index
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
            msg!("OperatorNcnTicket is not active or in cooldown");
            Err(RestakingCoreError::OperatorNcnTicketNotActive)
        }
    }

    pub fn seeds(operator: &Pubkey, ncn: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"operator_ncn_ticket".to_vec(),
            operator.to_bytes().to_vec(),
            ncn.to_bytes().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        operator: &Pubkey,
        ncn: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(operator, ncn);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        operator: &Pubkey,
        ncn: &Pubkey,
    ) -> RestakingCoreResult<Self> {
        if account.data_is_empty() {
            return Err(RestakingCoreError::OperatorNcnTicketEmpty);
        }
        if account.owner != program_id {
            return Err(RestakingCoreError::OperatorNcnTicketInvalidOwner);
        }

        let ncn_vault_ticket = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| RestakingCoreError::OperatorNcnTicketInvalidData(e.to_string()))?;
        if ncn_vault_ticket.account_type != AccountType::OperatorNcnTicket {
            return Err(RestakingCoreError::OperatorNcnTicketInvalidAccountType);
        }

        let mut seeds = Self::seeds(operator, ncn);
        seeds.push(vec![ncn_vault_ticket.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| RestakingCoreError::OperatorNcnTicketInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(RestakingCoreError::OperatorNcnTicketInvalidPda);
        }

        Ok(ncn_vault_ticket)
    }
}

pub struct SanitizedOperatorNcnTicket<'a, 'info> {
    account: &'a AccountInfo<'info>,
    operator_ncn_ticket: Box<OperatorNcnTicket>,
}

impl<'a, 'info> SanitizedOperatorNcnTicket<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        operator: &Pubkey,
        ncn: &Pubkey,
    ) -> RestakingCoreResult<Self> {
        if expect_writable && !account.is_writable {
            return Err(RestakingCoreError::OperatorNcnTicketNotWritable);
        }

        let operator_ncn_ticket = Box::new(OperatorNcnTicket::deserialize_checked(
            program_id, account, operator, ncn,
        )?);

        Ok(Self {
            account,
            operator_ncn_ticket,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn operator_ncn_ticket(&self) -> &OperatorNcnTicket {
        &self.operator_ncn_ticket
    }

    pub fn operator_ncn_ticket_mut(&mut self) -> &mut OperatorNcnTicket {
        &mut self.operator_ncn_ticket
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(
            &mut self.account.data.borrow_mut()[..],
            &self.operator_ncn_ticket,
        )?;
        Ok(())
    }
}
