use cvlr::{clog, cvlr_assert, cvlr_assume, cvlr_satisfy, mathint::NativeInt, nondet, rule};

use {
    jito_jsm_core::slot_toggle::{*},
    solana_program::clock::{DEFAULT_SLOTS_PER_EPOCH},
};

#[rule]
// P-01. If state moves from warmup to active then it must have passed at least one epoch.
// This rule is verified
pub fn rule_warmup_to_active_1() {
    let slot0: u64 = nondet();
    let slot1: u64 = nondet();
    let slot2: u64 = nondet();
    let epoch_length: u64 = nondet();
    cvlr_assume!(slot2 > slot1);

    let mut toggle = SlotToggle::new(slot0);
    toggle.activate(slot1, epoch_length).unwrap();

    if toggle.is_active(slot2, epoch_length).unwrap() {
        cvlr_assert!(NativeInt::new(slot2) - NativeInt::new(slot1) > NativeInt::new(epoch_length));
    }
}

#[rule]
// If state moves from warmup to active then it must have passed at least two epochs.
// Similar to rule_warmup_to_active_1 but different epochs.
pub fn rule_warmup_to_active_2_witness() {
    let slot0: u64 = nondet();
    let activate_slot: u64 = nondet();
    let slot1: u64 = nondet();
    let epoch_length: u64 = nondet();
    cvlr_assume!(slot0 > 0); // to avoid starting as inactive
    cvlr_assume!(slot1 > slot0);

    let mut toggle = SlotToggle::new(slot0);
    toggle.activate(activate_slot, epoch_length).unwrap();
    cvlr_assume!(toggle.state(slot0, epoch_length).unwrap() == SlotToggleState::WarmUp);

     if toggle.is_active(slot1, epoch_length).unwrap() {
        let two_epoch_length = NativeInt::new(epoch_length) + NativeInt::new(epoch_length);
        clog!(slot0, slot1, epoch_length);
        cvlr_satisfy!(NativeInt::new(slot1) - NativeInt::new(slot0) >= two_epoch_length);
    }
}

#[rule]
// P-02. If state is in warmup and it passes two epochs or more then the state must be active
pub fn rule_warmup_to_active_3() {
    let slot0: u64 = nondet();
    let activate_slot: u64 = nondet();
    let slot1: u64 = nondet();
    let slot2: u64 = nondet();
    let epoch_length: u64 = nondet();
    cvlr_assume!(slot1 >= activate_slot);
    cvlr_assume!(slot2 > slot1);

    let mut toggle = SlotToggle::new(slot0);
    toggle.activate(activate_slot, epoch_length).unwrap();

    cvlr_assume!(toggle.state(slot1, epoch_length).unwrap() == SlotToggleState::WarmUp);

    let two_epochs = NativeInt::new(epoch_length) + NativeInt::new(epoch_length);
    clog!(slot0, activate_slot, slot1, slot2);
    if (NativeInt::new(slot2) - NativeInt::new(slot1)) >= two_epochs {
        cvlr_assert!(toggle.is_active(slot2, epoch_length).unwrap());
    }
}

#[rule]
// If state is in warmup and it passes one epoch or more then the state must be active
// Similar to rule_warmup_to_active_3 but different epochs.
pub fn rule_warmup_to_active_4_witness() {
    let slot0: u64 = nondet();
    let activate_slot: u64 = nondet();
    let slot1: u64 = nondet();
    let epoch_length: u64 = DEFAULT_SLOTS_PER_EPOCH; // realistic epoch length
    cvlr_assume!(slot0 > DEFAULT_SLOTS_PER_EPOCH); // has passed already one epoch
    cvlr_assume!(slot1 > slot0);

    let mut toggle = SlotToggle::new(slot0);
    toggle.activate(activate_slot, epoch_length).unwrap();
    cvlr_assume!(toggle.state(slot0, epoch_length).unwrap() == SlotToggleState::WarmUp);

    if (slot1 - slot0) >= epoch_length {
        cvlr_satisfy!(toggle.is_active(slot1, epoch_length).unwrap());
    }
}

#[rule]
// P-03. This rule is expected to be verified
pub fn rule_activate() {
    let slot1: u64 = nondet();
    let slot2:u64 = nondet();
    let slot3:u64 = nondet();
    let epoch_length: u64 = nondet();
    cvlr_assume!(slot1 > 0);
    cvlr_assume!(slot2 > slot1);
    cvlr_assume!(slot3 > slot2);

    // create inactive toggle. better (more general) way?
    let mut toggle = SlotToggle::new(0);
    cvlr_assert!(toggle.state(slot1, epoch_length).unwrap() == SlotToggleState::Inactive);

    // Activate
    cvlr_assert!(toggle.activate(slot2, epoch_length).unwrap());
    cvlr_assert!(toggle.state(slot2, epoch_length).unwrap() == SlotToggleState::WarmUp);

    // Property
    if toggle.state(slot3, epoch_length).unwrap() == SlotToggleState::Active {
        cvlr_assert!(slot3 - slot2 > epoch_length);
    }
}

#[rule]
// P-04. deactivate() post-condition is cooldown
// P-05. If the state transitions from cooldown to inactive then at least one epoch has elapsed
// This rule is expected to be verified but it is currently violated (due to bug or spec is not clear)
pub fn rule_deactivate() {
    let slot1:u64 = nondet();
    let slot2:u64 = nondet();
    let slot3:u64 = nondet();
    let slot4:u64 = nondet();
    let slot5:u64 = nondet();
    let epoch_length: u64 = nondet();
    cvlr_assume!(slot1 > 0);
    cvlr_assume!(slot2 > slot1);
    cvlr_assume!(slot3 > slot2);
    cvlr_assume!(slot4 > slot3);
    cvlr_assume!(slot5 > slot4);

    // Move toggle to active so that we can deactivate. Better (more general) way?
    let mut toggle = SlotToggle::new(0);
    cvlr_assert!(toggle.state(slot1, epoch_length).unwrap() == SlotToggleState::Inactive);
    cvlr_assert!(toggle.activate(slot2, epoch_length).unwrap());
    cvlr_assert!(toggle.state(slot2, epoch_length).unwrap() == SlotToggleState::WarmUp);
    cvlr_assume!(toggle.state(slot3, epoch_length).unwrap() == SlotToggleState::Active);

    // Deactivate
    cvlr_assert!(toggle.deactivate(slot4, epoch_length).unwrap());
    cvlr_assert!(toggle.state(slot4, epoch_length).unwrap() == SlotToggleState::Cooldown);

    // Property
    if toggle.state(slot5, epoch_length).unwrap() == SlotToggleState::Inactive {
        cvlr_assert!(slot5 - slot4 >  epoch_length);
    }
}





#[rule]
// This rule models a liveness property
pub fn rule_eventually_active_witness() {
    let slot1:u64 = nondet();
    let slot2:u64 = nondet();
    let slot3:u64 = nondet();
    let epoch_length: u64 = nondet();

    let mut toggle = SlotToggle::new(slot1);
    cvlr_assert!(toggle.state(slot1, epoch_length).unwrap() == SlotToggleState::Inactive);

    toggle.activate(slot2, epoch_length).unwrap();
    cvlr_assume!(toggle.state(slot2, epoch_length).unwrap() == SlotToggleState::WarmUp);
    cvlr_assume!(toggle.state(slot3, epoch_length).unwrap() == SlotToggleState::Active);
    cvlr_satisfy!(true);
}

#[rule]
//  Since the epoch is too large assert(false) is not reachable
pub fn rule_eventually_active_too_large_epoch() {
    let slot1:u64 = nondet();
    let slot2:u64 = nondet();
    cvlr_assume!(slot1 > 0);
    cvlr_assume!(slot2 > 0);
    let epoch_length: u64 = u64::MAX;

    let toggle = SlotToggle::new(slot1);
    cvlr_assume!(toggle.state(slot1, epoch_length).unwrap() == SlotToggleState::WarmUp);
    cvlr_assume!(toggle.state(slot2, epoch_length).unwrap() == SlotToggleState::Active);
    cvlr_assert!(false);
}

// #[rule]
// //  Since the epoch is too small assert(false) is not reachable
// pub fn rule_eventually_active_too_small_epoch() {
//     let slot1:u64 = nondet();
//     let slot2:u64 = nondet();
//     cvlr_assume!(slot1 > 0);
//     cvlr_assume!(slot2 > 0);
//     let epoch_length: u64 = 0;

//     let toggle = SlotToggle::new(slot1);
//     cvlr_assert!(toggle.state(slot1, epoch_length) == SlotToggleState::WarmUp);
//     cvlr_assume!(toggle.state(slot2, epoch_length) == SlotToggleState::Active);
//     cvlr_assert!(false);
// }


#[rule]
// P-06. Warmup and active post-state
// If the time elapses and no function is called then warmup *may* transition to active
pub fn rule_warmup_is_stable_or_become_active() {
    let slot_added_nondet: u64 = nondet();
    let slot_removed_nondet: u64 = nondet();
    let activate_slot: u64 = nondet();
    let slot1: u64 = nondet();
    let slot2: u64 = nondet();
    let epoch_length: u64 = nondet();

    let mut toggle = SlotToggle::new(slot_added_nondet);
    toggle.activate(activate_slot, epoch_length).unwrap();
    toggle.deactivate(slot_removed_nondet, epoch_length).unwrap();
    let state1 = toggle.state(slot1, epoch_length).unwrap();
    let state2 = toggle.state(slot2, epoch_length).unwrap();

    clog!(slot_added_nondet, slot1, epoch_length);
    if state1 == SlotToggleState::WarmUp && slot2 >= slot1 {
        cvlr_assert!(state2 == SlotToggleState::WarmUp || state2 == SlotToggleState::Active)
    };
}

#[rule]
// If the time elapses and no function is called then active *cannot* move to another state
pub fn rule_active_is_stable() {
    let slot_added_nondet: u64 = nondet();
    let slot_removed_nondet: u64 = nondet();
    let activate_slot: u64 = nondet();
    let slot1: u64 = nondet();
    let slot2: u64 = nondet();
    let epoch_length: u64 = nondet();

    let mut toggle = SlotToggle::new(slot_added_nondet);
    toggle.activate(activate_slot, epoch_length).unwrap();
    toggle.deactivate(slot_removed_nondet, epoch_length).unwrap();
    let state1 = toggle.state(slot1, epoch_length).unwrap();
    let state2 = toggle.state(slot2, epoch_length).unwrap();

    clog!(epoch_length, slot_added_nondet, slot1, slot2);
    if state1 == SlotToggleState::Active && slot2 >= slot1 {
        cvlr_assert!(state2 == SlotToggleState::Active)
    };
}

#[rule]
// P-07. Cooldown and Inactive Post State
// If the time elapses and no function is called then cooldown *may* transition to active
pub fn rule_cooldown_is_stable_or_become_inactive() {
    let slot_added_nondet: u64 = nondet();
    let slot_removed_nondet: u64 = nondet();
    let activate_slot: u64 = nondet();
    let slot1: u64 = nondet();
    let slot2: u64 = nondet();
    let epoch_length: u64 = nondet();

    let mut toggle = SlotToggle::new(slot_added_nondet);
    toggle.activate(activate_slot, epoch_length).unwrap();
    toggle.deactivate(slot_removed_nondet, epoch_length).unwrap();
    let state1 = toggle.state(slot1, epoch_length).unwrap();
    let state2 = toggle.state(slot2, epoch_length).unwrap();

    if state1 == SlotToggleState::Cooldown {
        cvlr_assert!(state2 == SlotToggleState::Cooldown || state2 == SlotToggleState::Inactive)
    }; 
}


#[rule]
// If the time elapses and no function is called then inactive *cannot* move to another state
pub fn rule_inactive_is_stable() {
    let slot_added_nondet: u64 = nondet();
    let slot_removed_nondet: u64 = nondet();
    let slot1: u64 = nondet();
    let slot2: u64 = nondet();
    let epoch_length: u64 = nondet();

    let mut toggle = SlotToggle::new(slot_added_nondet);
    toggle.deactivate(slot_removed_nondet, epoch_length).unwrap();
    let state1 = toggle.state(slot1, epoch_length).unwrap();
    let state2 = toggle.state(slot2, epoch_length).unwrap();

    if (state1 == SlotToggleState::Inactive) && (slot2 >= slot1) {
        clog!(slot_added_nondet, slot_removed_nondet, slot1, slot2);
        cvlr_assert!(state2 == SlotToggleState::Inactive)
    }; 
   

}