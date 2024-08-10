use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    AccountType,
};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct NcnOperatorTicket {
    /// The account type
    account_type: AccountType,

    /// The NCN
    ncn: Pubkey,

    /// The operator
    operator: Pubkey,

    /// The index
    index: u64,

    /// The state
    state: SlotToggle,

    /// Reserved space
    reserved: [u8; 128],

    /// The bump seed for the PDA
    bump: u8,
}

impl NcnOperatorTicket {
    pub const fn new(ncn: Pubkey, operator: Pubkey, index: u64, slot_added: u64, bump: u8) -> Self {
        Self {
            account_type: AccountType::NcnOperatorTicket,
            ncn,
            operator,
            index,
            state: SlotToggle::new(slot_added),
            reserved: [0; 128],
            bump,
        }
    }

    pub const fn ncn(&self) -> Pubkey {
        self.ncn
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

    pub fn check_active_or_cooldown(
        &self,
        slot: u64,
        epoch_length: u64,
    ) -> RestakingCoreResult<()> {
        if self.state.is_active_or_cooldown(slot, epoch_length) {
            Ok(())
        } else {
            msg!("NcnOperatorTicket is not active or in cooldown");
            Err(RestakingCoreError::NcnOperatorTicketInactive)
        }
    }

    pub fn deactivate(&mut self, slot: u64, epoch_length: u64) -> RestakingCoreResult<()> {
        if self.state.deactivate(slot, epoch_length) {
            Ok(())
        } else {
            Err(RestakingCoreError::NcnOperatorTicketInactive)
        }
    }

    pub const fn bump(&self) -> u8 {
        self.bump
    }

    pub fn seeds(ncn: &Pubkey, operator: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"ncn_operator_ticket".to_vec(),
            ncn.as_ref().to_vec(),
            operator.as_ref().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        ncn: &Pubkey,
        operator: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(ncn, operator);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        ncn: &Pubkey,
        operator: &Pubkey,
    ) -> RestakingCoreResult<Self> {
        if account.data_is_empty() {
            return Err(RestakingCoreError::NcnOperatorTicketEmpty);
        }
        if account.owner != program_id {
            return Err(RestakingCoreError::NcnOperatorTicketInvalidOwner);
        }

        let ncn_operator_ticket = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| RestakingCoreError::NcnOperatorTicketInvalidData(e.to_string()))?;
        if ncn_operator_ticket.account_type != AccountType::NcnOperatorTicket {
            return Err(RestakingCoreError::NcnOperatorTicketInvalidAccountType);
        }

        let mut seeds = Self::seeds(ncn, operator);
        seeds.push(vec![ncn_operator_ticket.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| RestakingCoreError::NcnOperatorTicketInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(RestakingCoreError::NcnOperatorTicketInvalidPda);
        }

        Ok(ncn_operator_ticket)
    }
}

pub struct SanitizedNcnOperatorTicket<'a, 'info> {
    account: &'a AccountInfo<'info>,
    ncn_operator_ticket: Box<NcnOperatorTicket>,
}

impl<'a, 'info> SanitizedNcnOperatorTicket<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        ncn: &Pubkey,
        operator: &Pubkey,
    ) -> RestakingCoreResult<SanitizedNcnOperatorTicket<'a, 'info>> {
        if expect_writable && !account.is_writable {
            return Err(RestakingCoreError::NcnOperatorTicketExpectedWritable);
        }
        let ncn_operator_ticket = Box::new(NcnOperatorTicket::deserialize_checked(
            program_id, account, ncn, operator,
        )?);

        Ok(SanitizedNcnOperatorTicket {
            account,
            ncn_operator_ticket,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn ncn_operator_ticket(&self) -> &NcnOperatorTicket {
        &self.ncn_operator_ticket
    }

    pub fn ncn_operator_ticket_mut(&mut self) -> &mut NcnOperatorTicket {
        &mut self.ncn_operator_ticket
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(
            &mut self.account.data.borrow_mut()[..],
            &self.ncn_operator_ticket,
        )?;
        Ok(())
    }
}
