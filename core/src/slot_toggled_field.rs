use std::fmt::Debug;

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct SlotToggle {
    slot_added: u64,
    slot_removed: u64,
}

pub enum SlotToggleState {
    Inactive,
    WarmUp,
    Active,
    Cooldown,
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

    pub fn activate(&mut self, slot: u64, epoch_length: u64) -> bool {
        match self.state(slot, epoch_length) {
            SlotToggleState::Inactive => {
                self.slot_added = slot;
                true
            }
            _ => false,
        }
    }

    pub fn deactivate(&mut self, slot: u64, epoch_length: u64) -> bool {
        match self.state(slot, epoch_length) {
            SlotToggleState::Active => {
                self.slot_removed = slot;
                true
            }
            _ => false,
        }
    }

    pub const fn is_active(&self, slot: u64, epoch_length: u64) -> bool {
        match self.state(slot, epoch_length) {
            SlotToggleState::Active | SlotToggleState::Cooldown => true,
            _ => false,
        }
    }

    pub const fn warmup_slots(&self, slot: u64, epoch_length: u64) -> u64 {
        self.slot_added + epoch_length + slot % epoch_length
    }

    pub const fn cooldown_slots(&self, slot: u64, epoch_length: u64) -> u64 {
        self.slot_removed + epoch_length + slot % epoch_length
    }

    pub const fn state(&self, slot: u64, epoch_length: u64) -> SlotToggleState {
        if self.slot_added >= self.slot_removed {
            if slot <= self.warmup_slots(slot, epoch_length) {
                SlotToggleState::WarmUp
            } else {
                SlotToggleState::Active
            }
        } else {
            if slot <= self.cooldown_slots(slot, epoch_length) {
                SlotToggleState::Cooldown
            } else {
                SlotToggleState::Inactive
            }
        }
    }
}
