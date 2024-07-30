use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    AccountType,
};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct AvsOperatorTicket {
    /// The account type
    account_type: AccountType,

    /// The AVS
    avs: Pubkey,

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

impl AvsOperatorTicket {
    pub const fn new(avs: Pubkey, operator: Pubkey, index: u64, slot_added: u64, bump: u8) -> Self {
        Self {
            account_type: AccountType::AvsOperatorTicket,
            avs,
            operator,
            index,
            state: SlotToggle::new(slot_added),
            reserved: [0; 128],
            bump,
        }
    }

    pub const fn avs(&self) -> Pubkey {
        self.avs
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

    pub const fn check_active(&self, slot: u64) -> RestakingCoreResult<()> {
        if self.state.is_active(slot) {
            Ok(())
        } else {
            Err(RestakingCoreError::AvsOperatorTicketInactive)
        }
    }

    pub fn deactivate(&mut self, slot: u64) -> RestakingCoreResult<()> {
        if self.state.deactivate(slot) {
            Ok(())
        } else {
            Err(RestakingCoreError::AvsOperatorTicketInactive)
        }
    }

    pub const fn bump(&self) -> u8 {
        self.bump
    }

    pub fn seeds(avs: &Pubkey, operator: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"avs_operator_ticket".to_vec(),
            avs.as_ref().to_vec(),
            operator.as_ref().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        avs: &Pubkey,
        operator: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(avs, operator);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        avs: &Pubkey,
        operator: &Pubkey,
    ) -> RestakingCoreResult<Self> {
        if account.data_is_empty() {
            return Err(RestakingCoreError::AvsOperatorTicketEmpty);
        }
        if account.owner != program_id {
            return Err(RestakingCoreError::AvsOperatorTicketInvalidOwner);
        }

        let avs_operator_ticket = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| RestakingCoreError::AvsOperatorTicketInvalidData(e.to_string()))?;
        if avs_operator_ticket.account_type != AccountType::AvsOperatorTicket {
            return Err(RestakingCoreError::AvsOperatorTicketInvalidAccountType);
        }

        let mut seeds = Self::seeds(avs, operator);
        seeds.push(vec![avs_operator_ticket.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| RestakingCoreError::AvsOperatorTicketInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(RestakingCoreError::AvsOperatorTicketInvalidPda);
        }

        Ok(avs_operator_ticket)
    }
}

pub struct SanitizedAvsOperatorTicket<'a, 'info> {
    account: &'a AccountInfo<'info>,
    avs_operator_ticket: Box<AvsOperatorTicket>,
}

impl<'a, 'info> SanitizedAvsOperatorTicket<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        avs: &Pubkey,
        operator: &Pubkey,
    ) -> RestakingCoreResult<SanitizedAvsOperatorTicket<'a, 'info>> {
        if expect_writable && !account.is_writable {
            return Err(RestakingCoreError::AvsOperatorTicketExpectedWritable);
        }
        let avs_operator_ticket = Box::new(AvsOperatorTicket::deserialize_checked(
            program_id, account, avs, operator,
        )?);

        Ok(SanitizedAvsOperatorTicket {
            account,
            avs_operator_ticket,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn avs_operator_ticket(&self) -> &AvsOperatorTicket {
        &self.avs_operator_ticket
    }

    pub fn avs_operator_ticket_mut(&mut self) -> &mut AvsOperatorTicket {
        &mut self.avs_operator_ticket
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(
            &mut self.account.data.borrow_mut()[..],
            &self.avs_operator_ticket,
        )?;
        Ok(())
    }
}
