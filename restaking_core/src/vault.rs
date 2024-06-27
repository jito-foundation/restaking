use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use solana_program::pubkey::Pubkey;

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct RestakingVault {
    /// The vault account
    vault: Pubkey,

    state: SlotToggle,

    /// Reserved space
    reserved: [u8; 256],
}

impl RestakingVault {
    pub const fn new(vault: Pubkey, slot_added: u64) -> Self {
        Self {
            vault,
            state: SlotToggle::new(slot_added),
            reserved: [0; 256],
        }
    }

    pub const fn vault(&self) -> Pubkey {
        self.vault
    }

    pub const fn state(&self) -> &SlotToggle {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut SlotToggle {
        &mut self.state
    }
}
