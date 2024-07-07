use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use jito_restaking_sanitization::{assert_with_msg, realloc};
use solana_program::{
    account_info::AccountInfo, entrypoint_deprecated::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent,
};

use crate::{
    result::{RestakingCoreError, RestakingCoreResult},
    AccountType,
};

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct AvsOperator {
    /// operator in the AVS
    operator: Pubkey,

    state: SlotToggle,

    /// reserved space
    reserved: [u8; 256],
}

impl AvsOperator {
    pub const fn new(operator: Pubkey, slot_added: u64) -> Self {
        Self {
            operator,
            state: SlotToggle::new(slot_added),
            reserved: [0; 256],
        }
    }

    pub const fn operator(&self) -> Pubkey {
        self.operator
    }

    pub const fn state(&self) -> &SlotToggle {
        &self.state
    }
}

/// The AVS operator list stores a list of operators the AVS has accepted
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct AvsOperatorList {
    /// The account type
    account_type: AccountType,

    /// The AVS
    avs: Pubkey,

    /// The list of operators in the AVS
    operators: Vec<AvsOperator>,

    /// Reserved space
    reserved: [u8; 1024],

    /// The bump seed for the PDA
    bump: u8,
}

impl AvsOperatorList {
    pub const fn new(avs: Pubkey, bump: u8) -> Self {
        Self {
            account_type: AccountType::AvsOperatorList,
            avs,
            bump,
            operators: vec![],
            reserved: [0; 1024],
        }
    }

    pub fn check_operator_active(&self, operator: &Pubkey, slot: u64) -> RestakingCoreResult<()> {
        let maybe_operator = self.operators.iter().find(|a| a.operator() == *operator);
        maybe_operator.map_or(Err(RestakingCoreError::OperatorNotFound), |operator| {
            if operator.state.is_active(slot) {
                Ok(())
            } else {
                Err(RestakingCoreError::OperatorNotActive)
            }
        })
    }

    pub fn get_active_operator(&self, operator: &Pubkey, slot: u64) -> Option<&AvsOperator> {
        self.operators
            .iter()
            .find(|a| a.operator() == *operator && a.state.is_active(slot))
    }

    pub fn add_operator(&mut self, operator: Pubkey, slot: u64) -> RestakingCoreResult<()> {
        let maybe_operator = self.operators.iter_mut().find(|a| a.operator() == operator);
        if let Some(operator) = maybe_operator {
            let activated = operator.state.activate(slot);
            if activated {
                Ok(())
            } else {
                Err(RestakingCoreError::OperatorAlreadyAdded)
            }
        } else {
            self.operators.push(AvsOperator::new(operator, slot));
            Ok(())
        }
    }

    pub fn remove_operator(&mut self, operator: Pubkey, slot: u64) -> RestakingCoreResult<()> {
        let maybe_operator = self.operators.iter_mut().find(|a| a.operator() == operator);
        if let Some(operator) = maybe_operator {
            let deactivated = operator.state.deactivate(slot);
            if deactivated {
                Ok(())
            } else {
                Err(RestakingCoreError::OperatorAlreadyRemoved)
            }
        } else {
            Err(RestakingCoreError::OperatorNotFound)
        }
    }

    pub const fn avs(&self) -> Pubkey {
        self.avs
    }

    pub fn operators(&self) -> &[AvsOperator] {
        &self.operators
    }

    pub const fn bump(&self) -> u8 {
        self.bump
    }

    pub fn seeds(avs: &Pubkey) -> Vec<Vec<u8>> {
        Vec::from_iter([b"avs_operator_list".to_vec(), avs.as_ref().to_vec()])
    }

    pub fn find_program_address(program_id: &Pubkey, avs: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(avs);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        avs: &Pubkey,
    ) -> Result<Self, ProgramError> {
        assert_with_msg(
            !account.data_is_empty(),
            ProgramError::UninitializedAccount,
            "AVS Operator List account is not initialized",
        )?;
        assert_with_msg(
            account.owner == program_id,
            ProgramError::IllegalOwner,
            "AVS Operator List account not owned by the correct program",
        )?;

        // The AvsState shall be properly deserialized and valid struct
        let avs_operator_list = Self::deserialize(&mut account.data.borrow_mut().as_ref())?;
        assert_with_msg(
            avs_operator_list.account_type == AccountType::AvsOperatorList,
            ProgramError::InvalidAccountData,
            "AVS Operator List account is invalid",
        )?;
        assert_with_msg(
            avs_operator_list.avs == *avs,
            ProgramError::InvalidAccountData,
            "AVS Operator List account is not for the correct AVS",
        )?;

        // The AvsState shall be at the correct PDA as defined by the seeds and bump
        let mut seeds = Self::seeds(avs);
        seeds.push(vec![avs_operator_list.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)?;

        assert_with_msg(
            expected_pubkey == *account.key,
            ProgramError::InvalidAccountData,
            "AVS Operator List account is not at the correct PDA",
        )?;

        Ok(avs_operator_list)
    }
}

pub struct SanitizedAvsOperatorList<'a, 'info> {
    account: &'a AccountInfo<'info>,
    avs_operator_list: AvsOperatorList,
}

impl<'a, 'info> SanitizedAvsOperatorList<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        avs: &Pubkey,
    ) -> Result<SanitizedAvsOperatorList<'a, 'info>, ProgramError> {
        if expect_writable {
            assert_with_msg(
                account.is_writable,
                ProgramError::InvalidAccountData,
                "Invalid writable flag for AVS Operator List",
            )?;
        }
        let avs_operator_list = AvsOperatorList::deserialize_checked(program_id, account, avs)?;

        Ok(SanitizedAvsOperatorList {
            account,
            avs_operator_list,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn avs_operator_list(&self) -> &AvsOperatorList {
        &self.avs_operator_list
    }

    pub fn avs_operator_list_mut(&mut self) -> &mut AvsOperatorList {
        &mut self.avs_operator_list
    }

    pub fn save_with_realloc(&self, rent: &Rent, payer: &'a AccountInfo<'info>) -> ProgramResult {
        let serialized = self.avs_operator_list.try_to_vec()?;

        if serialized.len() > self.account.data.borrow().len() {
            realloc(self.account, serialized.len(), payer, rent)?;
        }

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }

    pub fn save(&self) -> ProgramResult {
        let serialized = self.avs_operator_list.try_to_vec()?;

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }
}
