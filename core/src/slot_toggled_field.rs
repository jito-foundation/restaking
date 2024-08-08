use std::fmt::Debug;

use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct SlotToggle {
    slot_added: u64,
    slot_removed: u64,
}

#[derive(PartialEq, Eq)]
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

    // is_active_or_cooldown
    // check_active_or_cooldown
    pub fn is_active(&self, slot: u64, epoch_length: u64) -> bool {
        //TODO CK Check with LB about active state
        matches!(
            self.state(slot, epoch_length),
            SlotToggleState::Active | SlotToggleState::Cooldown
        )
    }

    pub fn state(&self, slot: u64, epoch_length: u64) -> SlotToggleState {
        let current_epoch = slot.checked_div(epoch_length).unwrap();

        if self.slot_added >= self.slot_removed {
            let slot_added_epoch = self.slot_added().checked_div(epoch_length).unwrap();

            if current_epoch > slot_added_epoch.checked_add(1).unwrap() {
                SlotToggleState::Active
            } else {
                SlotToggleState::WarmUp
            }
        } else {
            let slot_removed_epoch = self.slot_removed().checked_div(epoch_length).unwrap();

            if current_epoch > slot_removed_epoch.checked_add(1).unwrap() {
                SlotToggleState::Inactive
            } else {
                SlotToggleState::Cooldown
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::slot_toggled_field::{SlotToggle, SlotToggleState};

    #[test]
    fn test_new() {
        let creation_slot = 100;
        let epoch_length = 150;

        let toggle = SlotToggle::new(creation_slot);
        assert!(toggle.state(creation_slot, epoch_length) == SlotToggleState::WarmUp);
        assert_eq!(toggle.slot_added(), creation_slot);
        assert_eq!(toggle.slot_removed(), 0);
    }

    #[test]
    fn test_activate_deactivate_cycle() {
        let creation_slot = 100;
        let epoch_length = 150;

        let mut elapsed_slots = creation_slot;
        let mut toggle = SlotToggle::new(creation_slot);

        // Assert Warming Up
        assert!(toggle.state(creation_slot, epoch_length) == SlotToggleState::WarmUp);
        assert!(!toggle.activate(elapsed_slots, epoch_length));
        assert!(!toggle.deactivate(elapsed_slots, epoch_length));

        // Assert Activated
        elapsed_slots += epoch_length + epoch_length % creation_slot;
        assert!(toggle.state(elapsed_slots, epoch_length) == SlotToggleState::Active);
        assert!(!toggle.activate(elapsed_slots, epoch_length));

        // Assert Deactivate
        assert!(toggle.deactivate(elapsed_slots, epoch_length));
        assert!(toggle.state(elapsed_slots, epoch_length) == SlotToggleState::Cooldown);
        assert!(!toggle.activate(elapsed_slots, epoch_length));

        elapsed_slots += epoch_length * 2;
        assert!(toggle.state(elapsed_slots, epoch_length) == SlotToggleState::Inactive);
        assert!(!toggle.deactivate(elapsed_slots, epoch_length));

        // Assert Activate
        assert!(toggle.activate(elapsed_slots, epoch_length));
        assert!(toggle.state(elapsed_slots, epoch_length) == SlotToggleState::WarmUp);
        assert!(!toggle.deactivate(elapsed_slots, epoch_length));
    }
}
