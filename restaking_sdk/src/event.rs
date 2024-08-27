use borsh::{BorshDeserialize, BorshSerialize};
use jito_events::Event;
use solana_program::pubkey::Pubkey;
use jito_events::EventDiscriminator;

#[derive(Clone, Debug, PartialEq, Eq, Event, BorshSerialize, BorshDeserialize)]
#[discriminator(1)]
pub struct MintEvent {
    pub stake_amount: u64,
    pub depositor: Pubkey,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mint_event() {
        let event = MintEvent {
            stake_amount: 100,
            depositor: Pubkey::default(),
        };

        let serialized = event.serialize_event();
        assert_eq!(u64::from_le_bytes(serialized[..8].try_into().unwrap()), 1);
        let deserialized = MintEvent::load_event(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }
}