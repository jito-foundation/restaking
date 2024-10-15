//! Slot toggled state tracker, useful for activations and deactivations of certain features
//! based on slot time.

use std::{cmp::Ordering, fmt::Debug};

use bytemuck::{Pod, Zeroable};
use jito_bytemuck::types::PodU64;
use shank::ShankType;

use crate::error::JsmCoreError;

/// SlotToggle is a state tracker that allows for activation and deactivation of certain features
/// based on slot time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, ShankType)]
#[repr(C)]
pub struct SlotToggle {
    /// The slot at which the feature was added
    slot_added: PodU64,
    /// The slot at which the feature was removed
    slot_removed: PodU64,

    reserved: [u8; 32],
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
    /// This sets slot_added and slot_removed to the same value, meaning the feature is inactive upon creation
    pub fn new(slot: u64) -> Self {
        Self {
            slot_added: PodU64::from(slot),
            slot_removed: PodU64::from(slot),
            reserved: [0; 32],
        }
    }

    /// Get the slot at which the feature was added
    pub fn slot_added(&self) -> u64 {
        self.slot_added.into()
    }

    /// Get the slot at which the feature was removed
    pub fn slot_removed(&self) -> u64 {
        self.slot_removed.into()
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
            Ok(SlotToggleState::Inactive) => {
                if self.slot_added() == slot {
                    // this should only be possible if the feature is being activated for the first time
                    // and the slot is the same as the slot it was created at
                    false
                } else {
                    self.slot_added = PodU64::from(slot);
                    true
                }
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
            Ok(SlotToggleState::Active) => {
                self.slot_removed = PodU64::from(slot);
                true
            }
            _ => false,
        }
    }

    /// Check if the feature is active or in cooldown state at the given slot.
    pub fn is_active_or_cooldown(&self, slot: u64, epoch_length: u64) -> bool {
        matches!(
            self.state(slot, epoch_length),
            Ok(SlotToggleState::Active) | Ok(SlotToggleState::Cooldown)
        )
    }

    /// Check if the feature is active at the given slot.
    pub fn is_active(&self, slot: u64, epoch_length: u64) -> bool {
        matches!(self.state(slot, epoch_length), Ok(SlotToggleState::Active))
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
    pub fn state(&self, slot: u64, epoch_length: u64) -> Result<SlotToggleState, JsmCoreError> {
        let current_epoch = slot
            .checked_div(epoch_length)
            .ok_or(JsmCoreError::JsmCoreDivisionByZero)?;

        let slot_added: u64 = self.slot_added.into();
        let slot_removed: u64 = self.slot_removed.into();

        match slot_added.cmp(&slot_removed) {
            Ordering::Equal => Ok(SlotToggleState::Inactive),
            Ordering::Less => {
                let slot_removed_epoch = slot_removed
                    .checked_div(epoch_length)
                    .ok_or(JsmCoreError::JsmCoreDivisionByZero)?;
                if current_epoch
                    > slot_removed_epoch
                        .checked_add(1)
                        .ok_or(JsmCoreError::JsmCoreArithmeticOverflow)?
                {
                    Ok(SlotToggleState::Inactive)
                } else {
                    Ok(SlotToggleState::Cooldown)
                }
            }
            Ordering::Greater => {
                let slot_added_epoch = slot_added
                    .checked_div(epoch_length)
                    .ok_or(JsmCoreError::JsmCoreDivisionByZero)?;
                if current_epoch
                    > slot_added_epoch
                        .checked_add(1)
                        .ok_or(JsmCoreError::JsmCoreArithmeticOverflow)?
                {
                    Ok(SlotToggleState::Active)
                } else {
                    Ok(SlotToggleState::WarmUp)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use jito_bytemuck::types::PodU64;

    use crate::slot_toggle::{SlotToggle, SlotToggleState};

    #[test]
    fn test_slot_toggle_no_padding() {
        let slot_toggle_size = std::mem::size_of::<SlotToggle>();
        let sum_of_fields = size_of::<PodU64>() + // slot_added
            size_of::<PodU64>() + // slot_removed
            32; // reserved
        assert_eq!(slot_toggle_size, sum_of_fields);
    }

    #[test]
    fn test_slot_zero() {
        let epoch_length = 150;
        let toggle = SlotToggle::new(0);
        assert_eq!(toggle.state(0, epoch_length), Ok(SlotToggleState::Inactive));
        assert_eq!(
            toggle.state(10, epoch_length),
            Ok(SlotToggleState::Inactive)
        );
        assert_eq!(
            toggle.state(epoch_length + 1, epoch_length),
            Ok(SlotToggleState::Inactive)
        );
    }

    #[test]
    fn test_new() {
        let creation_slot = 100;
        let epoch_length = 150;

        let toggle = SlotToggle::new(creation_slot);
        assert_eq!(toggle.slot_added(), creation_slot);
        assert_eq!(toggle.slot_removed(), creation_slot);
        assert!(toggle.state(creation_slot, epoch_length) == Ok(SlotToggleState::Inactive));
    }

    #[test]
    fn test_cant_transition_same_slot_created() {
        let creation_slot = 100;
        let epoch_length = 150;

        let mut toggle = SlotToggle::new(creation_slot);

        // can't transition to activate the same slot it was created at
        assert!(!toggle.activate(creation_slot, epoch_length));
        assert!(!toggle.deactivate(creation_slot, epoch_length));
    }

    #[test]
    fn test_activate_deactivate_cycle() {
        let creation_slot = 100;
        let epoch_length = 150;

        let mut current_slot = creation_slot;
        let mut toggle = SlotToggle::new(creation_slot);

        // Assert inactive
        assert_eq!(
            toggle.state(current_slot, epoch_length),
            Ok(SlotToggleState::Inactive)
        );

        // Transition to warming up
        current_slot += 1;
        assert!(toggle.activate(current_slot, epoch_length));
        assert_eq!(
            toggle.state(current_slot, epoch_length),
            Ok(SlotToggleState::WarmUp)
        );

        // Assert warming up
        current_slot += epoch_length;
        assert_eq!(
            toggle.state(current_slot, epoch_length),
            Ok(SlotToggleState::WarmUp)
        );

        // Assert active
        current_slot += epoch_length;
        assert_eq!(
            toggle.state(current_slot, epoch_length),
            Ok(SlotToggleState::Active)
        );

        // Assert Deactivate
        assert!(toggle.deactivate(current_slot, epoch_length));
        assert_eq!(
            toggle.state(current_slot, epoch_length),
            Ok(SlotToggleState::Cooldown)
        );

        current_slot += epoch_length;
        assert_eq!(
            toggle.state(current_slot, epoch_length),
            Ok(SlotToggleState::Cooldown)
        );

        current_slot += epoch_length;
        assert_eq!(
            toggle.state(current_slot, epoch_length),
            Ok(SlotToggleState::Inactive)
        );
    }
}
