use borsh::BorshSerialize;
pub use jito_events_derive::Event;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventDiscriminator(pub u64);

impl EventDiscriminator {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

pub trait Event: BorshSerialize {
    const DISCRIMINATOR: EventDiscriminator;

    fn serialize_event(&self) -> Vec<u8> {
        let mut result = Self::DISCRIMINATOR.0.to_le_bytes().to_vec();
        result.extend(self.try_to_vec().unwrap());
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(BorshSerialize, Event)]
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
        // Add more assertions as needed
    }
}