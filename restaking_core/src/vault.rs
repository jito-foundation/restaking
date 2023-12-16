use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct RestakingVault {
    /// The vault account
    vault: Pubkey,

    /// The slot when the vault was last added
    slot_added: u64,

    /// The slot when the vault was last removed
    slot_removed: u64,

    /// The maximum delegation amount
    max_delegation: u64,

    /// Reserved space
    reserved: [u8; 256],
}

pub enum RestakingVaultState {
    Inactive,
    WarmingUp,
    Active,
    CoolingDown,
}

impl RestakingVault {
    pub const fn new(vault: Pubkey, slot_added: u64, max_delegation: u64) -> Self {
        Self {
            vault,
            slot_added,
            slot_removed: 0,
            max_delegation,
            reserved: [0; 256],
        }
    }

    pub const fn vault(&self) -> Pubkey {
        self.vault
    }

    pub const fn slot_added(&self) -> u64 {
        self.slot_added
    }

    pub const fn slot_removed(&self) -> u64 {
        self.slot_removed
    }

    pub fn set_slot_added(
        &mut self,
        slot: u64,
        _num_slots_per_epoch: u64,
        max_delegation: u64,
    ) -> bool {
        // match self.state(slot, num_slots_per_epoch) {
        //     RestakingVaultState::Inactive => {
        //         self.slot_added = slot;
        //         self.max_delegation = max_delegation;
        //         true
        //     }
        //     _ => false,
        // }
        self.slot_added = slot;
        self.max_delegation = max_delegation;
        true
    }

    pub fn set_slot_removed(&mut self, slot: u64, _num_slots_per_epoch: u64) -> bool {
        // match self.state(slot, num_slots_per_epoch) {
        //     RestakingVaultState::Active => {
        //         self.slot_removed = slot;
        //         true
        //     }
        //     _ => false,
        // }
        self.slot_removed = slot;
        true
    }

    pub fn state(&self, slot: u64, num_slots_per_epoch: u64) -> RestakingVaultState {
        if self.slot_removed > self.slot_added {
            // either cooling down or inactive
            let epoch = self.slot_removed.checked_div(num_slots_per_epoch).unwrap();
            let inactive_epoch = epoch.checked_add(2).unwrap();
            if slot < inactive_epoch.checked_mul(num_slots_per_epoch).unwrap() {
                RestakingVaultState::CoolingDown
            } else {
                RestakingVaultState::Inactive
            }
        } else {
            let epoch = self.slot_added.checked_div(num_slots_per_epoch).unwrap();
            let active_epoch = epoch.checked_add(2).unwrap();
            if slot < active_epoch.checked_mul(num_slots_per_epoch).unwrap() {
                RestakingVaultState::WarmingUp
            } else {
                RestakingVaultState::Active
            }
        }
    }
}
