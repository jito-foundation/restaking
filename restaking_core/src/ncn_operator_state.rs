use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use jito_jsm_core::slot_toggle::SlotToggle;
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

impl Discriminator for NcnOperatorState {
    const DISCRIMINATOR: u8 = 4;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize, ShankAccount)]
#[repr(C)]
pub struct NcnOperatorState {
    /// The NCN account
    pub ncn: Pubkey,

    /// The operator account
    pub operator: Pubkey,

    /// Index
    pub index: u64,

    /// State of the ncn opt-ing in to the operator
    pub ncn_opt_in_state: SlotToggle,

    /// State of the operator opt-ing in to the ncn
    pub operator_opt_in_state: SlotToggle,

    pub bump: u8,

    /// Reserved space
    reserved: [u8; 7],
}

impl NcnOperatorState {
    pub const fn new(ncn: Pubkey, operator: Pubkey, index: u64, bump: u8) -> Self {
        Self {
            ncn,
            operator,
            index,
            ncn_opt_in_state: SlotToggle::new(0),
            operator_opt_in_state: SlotToggle::new(0),
            bump,
            reserved: [0; 7],
        }
    }

    pub fn seeds(ncn: &Pubkey, operator: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"ncn_operator_state".to_vec(),
            ncn.to_bytes().to_vec(),
            operator.to_bytes().to_vec(),
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

    /// Loads the account as an [`NcnOperatorState`] account, returning an error if it is not.
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `ncn_operator_ticket` - The account to load the NCN operator ticket from
    /// * `ncn` - The NCN account
    /// * `operator` - The operator account
    /// * `expect_writable` - Whether the account should be writable
    ///
    /// # Returns
    /// * `Result<(), ProgramError>` - The result of the operation
    pub fn load(
        program_id: &Pubkey,
        ncn_operator_state: &AccountInfo,
        ncn: &AccountInfo,
        operator: &AccountInfo,
        expect_writable: bool,
    ) -> Result<(), ProgramError> {
        if ncn_operator_state.owner.ne(program_id) {
            msg!("NCNOperatorState account has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if ncn_operator_state.data_is_empty() {
            msg!("NCNOperatorState account data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !ncn_operator_state.is_writable {
            msg!("NCNOperatorState account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if ncn_operator_state.data.borrow()[0].ne(&Self::DISCRIMINATOR) {
            msg!("NCNOperatorState account discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        let expected_pubkey = Self::find_program_address(program_id, ncn.key, operator.key).0;
        if ncn_operator_state.key.ne(&expected_pubkey) {
            msg!("NCNOperatorState account is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}
