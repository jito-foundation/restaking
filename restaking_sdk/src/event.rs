use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use solana_program::{instruction::Instruction, program::invoke_signed};

#[derive(Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
#[repr(u64)]
pub enum RestakingEvent {
    MintEvent(MintEvent) = 1,
}

#[derive(Clone, Debug, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct MintEvent {
    pub stake_amount: u64,
    pub depositor: Pubkey,
}

// pub fn emit_log(event: RestakingEvent) -> Result<(), ProgramError> {
//     let serialized = event.try_to_vec().unwrap();
//     // invoke_signed(
//     //     Instruction::new_with_borsh(program_id, data, accounts),
//     //     account_infos,
//     //     signers_seeds,
//     // )?;
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mint_event() {
        let event = RestakingEvent::MintEvent(MintEvent {
            stake_amount: 100,
            depositor: Pubkey::default(),
        });
        let serialized = event.try_to_vec().unwrap();
        assert_eq!(serialized.len(), 1 + 8 + 32);
    }
}