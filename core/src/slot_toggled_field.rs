use std::fmt::Debug;

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct SlotToggle {
    slot_added: u64,
    slot_removed: u64,
}

impl SlotToggle {
    pub const fn new(slot: u64) -> Self {
        Self {
            slot_added: slot,
            slot_removed: 0,
        }
    }

    pub const fn slot_added(&self) -> u64 {
        self.slot_added
    }

    pub const fn slot_removed(&self) -> u64 {
        self.slot_removed
    }

    pub fn activate(&mut self, slot: u64) -> bool {
        if self.slot_added >= self.slot_removed {
            false
        } else {
            self.slot_added = slot;
            true
        }
    }

    pub fn deactivate(&mut self, slot: u64) -> bool {
        if self.slot_added < self.slot_removed {
            false
        } else {
            self.slot_removed = slot;
            true
        }
    }

    pub const fn is_active(&self, slot: u64) -> bool {
        self.slot_added >= self.slot_removed && slot >= self.slot_added
    }
}
