#[cfg(test)]
mod tests {
    use jito_jsm_core::slot_toggled_field::{SlotToggle, SlotToggleState};

    #[tokio::test]
    async fn test_new() {
        let creation_slot = 100;
        let epoch_length = 150;

        let toggle = SlotToggle::new(creation_slot);
        assert!(toggle.state(creation_slot, epoch_length) == SlotToggleState::WarmUp);
        assert_eq!(toggle.slot_added(), creation_slot);
        assert_eq!(toggle.slot_removed(), 0);
    }

    #[tokio::test]
    async fn test_activate_deactivate_cycle() {
        let creation_slot = 100;
        let epoch_length = 150;

        let mut elapsed_slots = 0;
        let mut toggle = SlotToggle::new(creation_slot);

        // Assert Warming Up
        assert!(toggle.state(creation_slot, epoch_length) == SlotToggleState::WarmUp);
        assert!(!toggle.activate(elapsed_slots, epoch_length));
        assert!(!toggle.deactivate(elapsed_slots, epoch_length));

        // Assert Activated
        elapsed_slots += toggle.warmup_slots(creation_slot, epoch_length);
        assert!(toggle.state(elapsed_slots, epoch_length) == SlotToggleState::Active);
        assert!(!toggle.activate(elapsed_slots, epoch_length));

        // Assert Deactivate
        assert!(toggle.deactivate(elapsed_slots, epoch_length));
        assert!(toggle.state(elapsed_slots, epoch_length) == SlotToggleState::Cooldown);
        assert!(!toggle.activate(elapsed_slots, epoch_length));

        elapsed_slots += toggle.cooldown_slots(creation_slot, epoch_length);
        assert!(toggle.state(elapsed_slots, epoch_length) == SlotToggleState::Inactive);
        assert!(!toggle.deactivate(elapsed_slots, epoch_length));

        // Assert Activate
        assert!(toggle.activate(elapsed_slots, epoch_length));
        assert!(toggle.state(elapsed_slots, epoch_length) == SlotToggleState::WarmUp);
        assert!(!toggle.deactivate(elapsed_slots, epoch_length));
    }
}
