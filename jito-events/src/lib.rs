use borsh::{BorshSerialize, BorshDeserialize};
pub use jito_events_derive::Event;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventDiscriminator(pub u64);

impl EventDiscriminator {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EventError {
    InvalidDiscriminator,
    InvalidData,
}

pub trait Event: BorshSerialize + BorshDeserialize {
    const DISCRIMINATOR: EventDiscriminator;

    fn serialize_event(&self) -> Vec<u8> {
        let mut result = Self::DISCRIMINATOR.0.to_le_bytes().to_vec();
        result.extend(self.try_to_vec().unwrap());
        result
    }

    fn load_event(data: &[u8]) -> Result<Self, EventError> {
        if data.len() < 8 {
            return Err(EventError::InvalidData);
        }
        let event_discriminator = EventDiscriminator(u64::from_le_bytes(data[..8].try_into().unwrap()));
        if event_discriminator != Self::DISCRIMINATOR {
            return Err(EventError::InvalidDiscriminator);
        }
        Ok(Self::try_from_slice(&data[8..]).map_err(|_| EventError::InvalidData)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(BorshSerialize, BorshDeserialize, Event, PartialEq, Eq, Debug)]
    #[discriminator(1234567890)]
    struct TestEvent {
        field1: u32,
        field2: String,
    }

    #[test]
    fn test_event_serialization() {
        let event = TestEvent {
            field1: 42,
            field2: "Hello, World!".to_string(),
        };

        let serialized = event.serialize_event();
        let discriminator = TestEvent::DISCRIMINATOR.0.to_le_bytes();

        assert_eq!(&serialized[..8], &discriminator);
        let deserialized = TestEvent::load_event(&serialized).unwrap();
        assert_eq!(event, deserialized);
    }
}