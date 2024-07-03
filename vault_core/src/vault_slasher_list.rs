use borsh::{BorshDeserialize, BorshSerialize};
use jito_jsm_core::slot_toggled_field::SlotToggle;
use jito_restaking_sanitization::realloc;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey, rent::Rent,
};

use crate::{
    result::{VaultCoreError, VaultCoreResult},
    AccountType,
};

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug)]
pub struct VaultSlasher {
    /// The vault slasher can slash funds from any operators that are running this AVS software
    avs: Pubkey,
    /// The slasher signer
    slasher: Pubkey,
    /// State of the AVS slasher
    state: SlotToggle,
    /// The max slashable funds per epoch
    max_slashable_per_epoch: u64,
}

impl VaultSlasher {
    pub const fn new(
        avs: Pubkey,
        slasher: Pubkey,
        max_slashable_per_epoch: u64,
        slot: u64,
    ) -> Self {
        Self {
            avs,
            slasher,
            state: SlotToggle::new(slot),
            max_slashable_per_epoch,
        }
    }

    pub const fn slasher(&self) -> Pubkey {
        self.slasher
    }

    pub const fn state(&self) -> &SlotToggle {
        &self.state
    }

    pub const fn max_slashable_per_epoch(&self) -> u64 {
        self.max_slashable_per_epoch
    }
}

#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
pub struct VaultSlasherList {
    /// The vault slasher list
    account_type: AccountType,

    /// The vault account this slasher list is for
    vault: Pubkey,

    /// The list of slashers
    slashers: Vec<VaultSlasher>,

    /// Reserved space
    reserved: [u8; 256],

    /// The bump seed
    bump: u8,
}

impl VaultSlasherList {
    pub const fn new(vault: Pubkey, bump: u8) -> Self {
        Self {
            account_type: AccountType::VaultSlasherList,
            vault,
            slashers: vec![],
            reserved: [0; 256],
            bump,
        }
    }

    pub const fn vault(&self) -> Pubkey {
        self.vault
    }

    pub fn get_active_slasher(
        &self,
        slasher: &Pubkey,
        avs: &Pubkey,
        slot: u64,
    ) -> Option<&VaultSlasher> {
        self.slashers
            .iter()
            .find(|v| v.slasher == *slasher && v.avs == *avs && v.state.is_active(slot))
    }

    /// Add a slasher to the list for a given AVS.
    ///
    /// # Arguments
    /// * `avs` - The AVS
    /// * `slasher` - The slasher
    /// * `max_slashable_per_epoch` - The max slashable funds per epoch
    /// * `slot` - The current slot
    pub fn add_slasher(
        &mut self,
        avs: &Pubkey,
        slasher: &Pubkey,
        max_slashable_per_epoch: u64,
        slot: u64,
    ) -> bool {
        let maybe_slasher = self
            .slashers
            .iter_mut()
            .find(|v| v.slasher == *slasher && v.avs == *avs);
        if let Some(slasher) = maybe_slasher {
            slasher.state.activate(slot)
        } else {
            self.slashers.push(VaultSlasher::new(
                *avs,
                *slasher,
                max_slashable_per_epoch,
                slot,
            ));
            true
        }
    }

    pub fn seeds(vault: &Pubkey) -> Vec<Vec<u8>> {
        vec![b"vault_slasher_list".to_vec(), vault.to_bytes().to_vec()]
    }

    pub fn find_program_address(program_id: &Pubkey, vault: &Pubkey) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(vault);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        vault: &Pubkey,
    ) -> VaultCoreResult<Self> {
        if account.data_is_empty() {
            return Err(VaultCoreError::VaultSlasherListDataEmpty);
        }
        if account.owner != program_id {
            return Err(VaultCoreError::VaultSlasherListInvalidProgramOwner);
        }

        let state = Self::deserialize(&mut account.data.borrow_mut().as_ref())
            .map_err(|e| VaultCoreError::VaultSlasherListInvalidData(e.to_string()))?;
        if state.account_type != AccountType::VaultSlasherList {
            return Err(VaultCoreError::VaultSlasherListInvalidAccountType);
        }
        // The AvsState shall be at the correct PDA as defined by the seeds and bump
        let mut seeds = Self::seeds(vault);
        seeds.push(vec![state.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| VaultCoreError::VaultSlasherListInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(VaultCoreError::VaultSlasherListInvalidPda);
        }

        Ok(state)
    }
}

pub struct SanitizedVaultSlasherList<'a, 'info> {
    account: &'a AccountInfo<'info>,
    vault_slasher_list: VaultSlasherList,
}

impl<'a, 'info> SanitizedVaultSlasherList<'a, 'info> {
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        vault: &Pubkey,
    ) -> VaultCoreResult<SanitizedVaultSlasherList<'a, 'info>> {
        if expect_writable && !account.is_writable {
            return Err(VaultCoreError::VaultSlasherListExpectedWritable);
        }
        let vault_slasher_list = VaultSlasherList::deserialize_checked(program_id, account, vault)?;

        Ok(SanitizedVaultSlasherList {
            account,
            vault_slasher_list,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn vault_slasher_list(&self) -> &VaultSlasherList {
        &self.vault_slasher_list
    }

    pub fn vault_slasher_list_mut(&mut self) -> &mut VaultSlasherList {
        &mut self.vault_slasher_list
    }

    pub fn save(&self, rent: &Rent, payer: &'a AccountInfo<'info>) -> ProgramResult {
        let serialized = self.vault_slasher_list.try_to_vec()?;

        if serialized.len() > self.account.data.borrow().len() {
            realloc(self.account, serialized.len(), payer, rent)?;
        }

        self.account.data.borrow_mut()[..serialized.len()].copy_from_slice(&serialized);

        Ok(())
    }
}
