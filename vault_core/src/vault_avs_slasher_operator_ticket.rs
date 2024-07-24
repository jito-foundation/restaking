use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{
    result::{VaultCoreError, VaultCoreResult},
    AccountType,
};

/// Represents a vault AVS slasher operator ticket, which tracks how much an operator
/// has been slashed by a slasher for a given AVS and vault for a given epoch.
#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct VaultAvsSlasherOperatorTicket {
    /// The account type
    account_type: AccountType,

    /// The vault slashed
    vault: Pubkey,

    /// The AVS slashed
    avs: Pubkey,

    /// The slasher
    slasher: Pubkey,

    /// The operator
    operator: Pubkey,

    /// The epoch
    epoch: u64,

    /// The amount slashed for the given epoch
    slashed: u64,

    /// Reserved space
    reserved: [u8; 128],

    bump: u8,
}

impl VaultAvsSlasherOperatorTicket {
    pub const fn new(
        vault: Pubkey,
        avs: Pubkey,
        slasher: Pubkey,
        operator: Pubkey,
        epoch: u64,
        slashed: u64,
        bump: u8,
    ) -> Self {
        Self {
            account_type: AccountType::VaultAvsSlasherOperatorTicket,
            vault,
            avs,
            slasher,
            operator,
            epoch,
            slashed,
            reserved: [0; 128],
            bump,
        }
    }

    pub const fn vault(&self) -> Pubkey {
        self.vault
    }

    pub const fn avs(&self) -> Pubkey {
        self.avs
    }

    pub const fn slasher(&self) -> Pubkey {
        self.slasher
    }

    pub const fn operator(&self) -> Pubkey {
        self.operator
    }

    pub const fn epoch(&self) -> u64 {
        self.epoch
    }

    pub const fn slashed(&self) -> u64 {
        self.slashed
    }

    pub fn increment_slashed_amount(&mut self, amount: u64) -> VaultCoreResult<()> {
        self.slashed = self
            .slashed
            .checked_add(amount)
            .ok_or(VaultCoreError::VaultAvsSlasherOperatorOverflow)?;
        Ok(())
    }

    pub const fn bump(&self) -> u8 {
        self.bump
    }

    pub fn check_max_slashable_not_exceeded(
        &self,
        slash_amount: u64,
        max_slashable_per_epoch: u64,
    ) -> VaultCoreResult<()> {
        if self
            .slashed
            .checked_add(slash_amount)
            .ok_or(VaultCoreError::VaultAvsSlasherOperatorOverflow)?
            > max_slashable_per_epoch
        {
            return Err(VaultCoreError::VaultAvsSlasherOperatorMaxSlashableExceeded);
        }
        Ok(())
    }

    pub fn seeds(
        vault: &Pubkey,
        avs: &Pubkey,
        slasher: &Pubkey,
        operator: &Pubkey,
        epoch: u64,
    ) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"vault_avs_slasher_operator".to_vec(),
            vault.to_bytes().to_vec(),
            avs.to_bytes().to_vec(),
            slasher.to_bytes().to_vec(),
            operator.to_bytes().to_vec(),
            epoch.to_le_bytes().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        vault: &Pubkey,
        avs: &Pubkey,
        slasher: &Pubkey,
        operator: &Pubkey,
        epoch: u64,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(vault, avs, slasher, operator, epoch);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        vault: &Pubkey,
        avs: &Pubkey,
        slasher: &Pubkey,
        operator: &Pubkey,
        epoch: u64,
    ) -> VaultCoreResult<Self> {
        if account.data_is_empty() {
            return Err(VaultCoreError::VaultAvsSlasherOperatorDataEmpty);
        }
        if account.owner != program_id {
            return Err(VaultCoreError::VaultAvsSlasherOperatorInvalidOwner);
        }

        // The AvsState shall be properly deserialized and valid struct
        let vault_avs_slasher_operator_ticket =
            Self::deserialize(&mut account.data.borrow_mut().as_ref())
                .map_err(|e| VaultCoreError::VaultAvsSlasherOperatorInvalidData(e.to_string()))?;
        if vault_avs_slasher_operator_ticket.account_type
            != AccountType::VaultAvsSlasherOperatorTicket
        {
            return Err(VaultCoreError::VaultAvsSlasherOperatorInvalidAccountType);
        }

        let mut seeds = Self::seeds(vault, avs, slasher, operator, epoch);
        seeds.push(vec![vault_avs_slasher_operator_ticket.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| VaultCoreError::VaultAvsSlasherOperatorInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(VaultCoreError::VaultAvsSlasherOperatorInvalidPda);
        }

        Ok(vault_avs_slasher_operator_ticket)
    }
}

pub struct SanitizedVaultAvsSlasherOperatorTicket<'a, 'info> {
    account: &'a AccountInfo<'info>,
    vault_avs_slasher_operator_ticket: Box<VaultAvsSlasherOperatorTicket>,
}

impl<'a, 'info> SanitizedVaultAvsSlasherOperatorTicket<'a, 'info> {
    #[allow(clippy::too_many_arguments)]
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        vault: &Pubkey,
        avs: &Pubkey,
        slasher: &Pubkey,
        operator: &Pubkey,
        epoch: u64,
    ) -> VaultCoreResult<Self> {
        if expect_writable && !account.is_writable {
            return Err(VaultCoreError::VaultAvsSlasherOperatorNotWritable);
        }
        let vault_avs_slasher_operator_ticket =
            Box::new(VaultAvsSlasherOperatorTicket::deserialize_checked(
                program_id, account, vault, avs, slasher, operator, epoch,
            )?);

        Ok(Self {
            account,
            vault_avs_slasher_operator_ticket,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn vault_avs_slasher_operator_ticket(&self) -> &VaultAvsSlasherOperatorTicket {
        &self.vault_avs_slasher_operator_ticket
    }

    pub fn vault_avs_slasher_operator_ticket_mut(&mut self) -> &mut VaultAvsSlasherOperatorTicket {
        &mut self.vault_avs_slasher_operator_ticket
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(
            &mut self.account.data.borrow_mut()[..],
            &*self.vault_avs_slasher_operator_ticket,
        )?;
        Ok(())
    }
}
