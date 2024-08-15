//! Slot toggled state tracker, useful for activations and deactivations of certain features
//! based on slot time.
use std::fmt::Debug;

use bytemuck::{Pod, Zeroable};

/// SlotToggle is a state tracker that allows for activation and deactivation of certain features
/// based on slot time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct SlotToggle {
    /// The slot at which the feature was added
    slot_added: u64,
    /// The slot at which the feature was removed
    slot_removed: u64,
}

/// The state of the SlotToggle
#[derive(Debug, PartialEq, Eq)]
pub enum SlotToggleState {
    /// The feature is inactive
    Inactive,
    /// The feature is in warm-up state
    WarmUp,
    /// The feature is active
    Active,
    /// The feature is in cooldown state
    Cooldown,
}

impl SlotToggle {
    /// Create a new SlotToggle with the given slot
    pub const fn new(slot: u64) -> Self {
        Self {
            slot_added: slot,
            slot_removed: 0,
        }
    }

    /// Get the slot at which the feature was added
    pub const fn slot_added(&self) -> u64 {
        self.slot_added
    }

    /// Get the slot at which the feature was removed
    pub const fn slot_removed(&self) -> u64 {
        self.slot_removed
    }

    /// Activate the feature at the given slot, which can only happen if the feature is inactive.
    /// Once activated, it immediately transitions to warming up state, which takes place for
    /// one **full** epoch before transitioning to active state.
    ///
    /// # Arguments
    /// * `slot` - The slot at which the feature is being activated
    /// * `epoch_length` - The length of an epoch in slots
    ///
    /// # Returns
    /// * `bool` - Whether the feature was successfully activated
    pub fn activate(&mut self, slot: u64, epoch_length: u64) -> bool {
        match self.state(slot, epoch_length) {
            SlotToggleState::Inactive => {
                self.slot_added = slot;
                true
            }
            _ => false,
        }
    }

    /// Deactivate the feature at the given slot, which can only happen if the feature is active.
    /// Once deactivated, it immediately transitions to cooldown state, which takes place for
    /// one **full** epoch before transitioning to inactive state.
    ///
    /// # Arguments
    /// * `slot` - The slot at which the feature is being deactivated
    /// * `epoch_length` - The length of an epoch in slots
    ///
    /// # Returns
    /// * `bool` - Whether the feature was successfully deactivated
    pub fn deactivate(&mut self, slot: u64, epoch_length: u64) -> bool {
        match self.state(slot, epoch_length) {
            SlotToggleState::Active => {
                self.slot_removed = slot;
                true
            }
            _ => false,
        }
    }

    /// Check if the feature is active or in cooldown state at the given slot.
    pub fn is_active_or_cooldown(&self, slot: u64, epoch_length: u64) -> bool {
        matches!(
            self.state(slot, epoch_length),
            SlotToggleState::Active | SlotToggleState::Cooldown
        )
    }

    /// Check if the feature is active at the given slot.
    pub fn is_active(&self, slot: u64, epoch_length: u64) -> bool {
        matches!(self.state(slot, epoch_length), SlotToggleState::Active)
    }

    /// Get the state of the feature at the given slot.
    /// The state is determined based on the slot time and the epoch length.
    ///
    /// # Arguments
    /// * `slot` - The slot at which the state is being queried
    /// * `epoch_length` - The length of an epoch in slots
    ///
    /// # Returns
    /// * `SlotToggleState` - The state of the feature at the given slot
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
    use crate::slot_toggle::{SlotToggle, SlotToggleState};

    #[test]
    fn test_slot_zero() {
        let epoch_length = 150;
        let toggle = SlotToggle::new(0);
        assert_eq!(toggle.state(0, epoch_length), SlotToggleState::Inactive);
        assert_eq!(toggle.state(10, epoch_length), SlotToggleState::Inactive);
        assert_eq!(
            toggle.state(epoch_length + 1, epoch_length),
            SlotToggleState::Inactive
        );
    }

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
