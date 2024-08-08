use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    AccountType,
};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
#[repr(C)]
pub struct OperatorAvsTicket {
    account_type: AccountType,

    /// The operator account
    operator: Pubkey,

    /// The AVS account
    avs: Pubkey,

    index: u64,

    state: SlotToggle,

    /// Reserved space
    reserved: [u8; 128],

    bump: u8,
}

impl OperatorAvsTicket {
    pub const fn new(operator: Pubkey, avs: Pubkey, index: u64, slot_added: u64, bump: u8) -> Self {
        Self {
            account_type: AccountType::OperatorAvsTicket,
            operator,
            avs,
            index,
            state: SlotToggle::new(slot_added),
            reserved: [0; 128],
            bump,
        }
    }

    pub const fn operator(&self) -> Pubkey {
        self.operator
    }

    pub const fn avs(&self) -> Pubkey {
        self.avs
    }

    pub const fn state(&self) -> &SlotToggle {
        &self.state
    }

    pub fn deactivate(&mut self, slot: u64, epoch_length: u64) -> RestakingCoreResult<()> {
        if self.state.deactivate(slot, epoch_length) {
            Ok(())
        } else {
            Err(RestakingCoreError::OperatorAvsTicketAlreadyInactive)
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
            Err(RestakingCoreError::OperatorAvsTicketNotActive)
        }
    }

    pub fn seeds(operator: &Pubkey, avs: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"operator_avs_ticket".to_vec(),
            operator.to_bytes().to_vec(),
            avs.to_bytes().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        operator: &Pubkey,
        avs: &Pubkey,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(operator, avs);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        operator: &Pubkey,
        avs: &Pubkey,
    ) -> RestakingCoreResult<Self> {
        if account.data_is_empty() {
            return Err(RestakingCoreError::OperatorAvsTicketEmpty);
        }
        if account.owner != program_id {
            return Err(RestakingCoreError::OperatorAvsTicketInvalidOwner);
        }

        // The AvsState shall be properly deserialized and valid struct
        let avs_vault_ticket = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| RestakingCoreError::OperatorAvsTicketInvalidData(e.to_string()))?;
        if avs_vault_ticket.account_type != AccountType::OperatorAvsTicket {
            return Err(RestakingCoreError::OperatorAvsTicketInvalidAccountType);
        }

        let mut seeds = Self::seeds(operator, avs);
        seeds.push(vec![avs_vault_ticket.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| RestakingCoreError::OperatorAvsTicketInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(RestakingCoreError::OperatorAvsTicketInvalidPda);
        }

        Ok(avs_vault_ticket)
    }
}

pub struct SanitizedOperatorAvsTicket<'a, 'info> {
    account: &'a AccountInfo<'info>,
    operator_avs_ticket: Box<OperatorAvsTicket>,
}

impl<'a, 'info> SanitizedOperatorAvsTicket<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        operator: &Pubkey,
        avs: &Pubkey,
    ) -> RestakingCoreResult<Self> {
        if expect_writable && !account.is_writable {
            return Err(RestakingCoreError::OperatorAvsTicketNotWritable);
        }

        let operator_avs_ticket = Box::new(OperatorAvsTicket::deserialize_checked(
            program_id, account, operator, avs,
        )?);

        Ok(Self {
            account,
            operator_avs_ticket,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn operator_avs_ticket(&self) -> &OperatorAvsTicket {
        &self.operator_avs_ticket
    }

    pub fn operator_avs_ticket_mut(&mut self) -> &mut OperatorAvsTicket {
        &mut self.operator_avs_ticket
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(
            &mut self.account.data.borrow_mut()[..],
            &self.operator_avs_ticket,
        )?;
        Ok(())
    }
}
