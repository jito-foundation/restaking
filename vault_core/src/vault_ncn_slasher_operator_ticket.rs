use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use crate::{
    result::{VaultCoreError, VaultCoreResult},
    AccountType,
};

/// Represents a vault node consensus network (NCN) slasher operator ticket, which tracks how much an operator
/// has been slashed by a slasher for a given NCN and vault for a given epoch.
#[derive(Debug, BorshSerialize, BorshDeserialize, Clone)]
pub struct VaultNcnSlasherOperatorTicket {
    /// The account type
    account_type: AccountType,

    /// The vault slashed
    vault: Pubkey,

    /// The node consensus network slashed
    ncn: Pubkey,

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

impl VaultNcnSlasherOperatorTicket {
    pub const fn new(
        vault: Pubkey,
        ncn: Pubkey,
        slasher: Pubkey,
        operator: Pubkey,
        epoch: u64,
        slashed: u64,
        bump: u8,
    ) -> Self {
        Self {
            account_type: AccountType::VaultNcnSlasherOperatorTicket,
            vault,
            ncn,
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

    pub const fn ncn(&self) -> Pubkey {
        self.ncn
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
            .ok_or(VaultCoreError::VaultNcnSlasherOperatorOverflow)?;
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
        let new_slashed_amount = self
            .slashed
            .checked_add(slash_amount)
            .ok_or(VaultCoreError::VaultNcnSlasherOperatorOverflow)?;
        if new_slashed_amount > max_slashable_per_epoch {
            msg!(
                "Max slashable per epoch exceeded ({} > {})",
                new_slashed_amount,
                max_slashable_per_epoch
            );
            return Err(VaultCoreError::VaultNcnSlasherOperatorMaxSlashableExceeded);
        }
        Ok(())
    }

    pub fn seeds(
        vault: &Pubkey,
        ncn: &Pubkey,
        slasher: &Pubkey,
        operator: &Pubkey,
        epoch: u64,
    ) -> Vec<Vec<u8>> {
        Vec::from_iter([
            b"vault_ncn_slasher_operator".to_vec(),
            vault.to_bytes().to_vec(),
            ncn.to_bytes().to_vec(),
            slasher.to_bytes().to_vec(),
            operator.to_bytes().to_vec(),
            epoch.to_le_bytes().to_vec(),
        ])
    }

    pub fn find_program_address(
        program_id: &Pubkey,
        vault: &Pubkey,
        ncn: &Pubkey,
        slasher: &Pubkey,
        operator: &Pubkey,
        epoch: u64,
    ) -> (Pubkey, u8, Vec<Vec<u8>>) {
        let seeds = Self::seeds(vault, ncn, slasher, operator, epoch);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_slice()).collect();
        let (pda, bump) = Pubkey::find_program_address(&seeds_iter, program_id);
        (pda, bump, seeds)
    }

    pub fn deserialize_checked(
        program_id: &Pubkey,
        account: &AccountInfo,
        vault: &Pubkey,
        ncn: &Pubkey,
        slasher: &Pubkey,
        operator: &Pubkey,
        epoch: u64,
    ) -> VaultCoreResult<Self> {
        if account.data_is_empty() {
            return Err(VaultCoreError::VaultNcnSlasherOperatorDataEmpty);
        }
        if account.owner != program_id {
            return Err(VaultCoreError::VaultNcnSlasherOperatorInvalidOwner);
        }

        let vault_ncn_slasher_operator_ticket =
            Self::deserialize(&mut account.data.borrow_mut().as_ref())
                .map_err(|e| VaultCoreError::VaultNcnSlasherOperatorInvalidData(e.to_string()))?;
        if vault_ncn_slasher_operator_ticket.account_type
            != AccountType::VaultNcnSlasherOperatorTicket
        {
            return Err(VaultCoreError::VaultNcnSlasherOperatorInvalidAccountType);
        }

        let mut seeds = Self::seeds(vault, ncn, slasher, operator, epoch);
        seeds.push(vec![vault_ncn_slasher_operator_ticket.bump]);
        let seeds_iter: Vec<_> = seeds.iter().map(|s| s.as_ref()).collect();
        let expected_pubkey = Pubkey::create_program_address(&seeds_iter, program_id)
            .map_err(|_| VaultCoreError::VaultNcnSlasherOperatorInvalidPda)?;
        if expected_pubkey != *account.key {
            return Err(VaultCoreError::VaultNcnSlasherOperatorInvalidPda);
        }

        Ok(vault_ncn_slasher_operator_ticket)
    }
}

pub struct SanitizedVaultNcnSlasherOperatorTicket<'a, 'info> {
    account: &'a AccountInfo<'info>,
    vault_ncn_slasher_operator_ticket: Box<VaultNcnSlasherOperatorTicket>,
}

impl<'a, 'info> SanitizedVaultNcnSlasherOperatorTicket<'a, 'info> {
    #[allow(clippy::too_many_arguments)]
    pub fn sanitize(
        program_id: &Pubkey,
        account: &'a AccountInfo<'info>,
        expect_writable: bool,
        vault: &Pubkey,
        ncn: &Pubkey,
        slasher: &Pubkey,
        operator: &Pubkey,
        epoch: u64,
    ) -> VaultCoreResult<Self> {
        if expect_writable && !account.is_writable {
            return Err(VaultCoreError::VaultNcnSlasherOperatorNotWritable);
        }
        let vault_ncn_slasher_operator_ticket =
            Box::new(VaultNcnSlasherOperatorTicket::deserialize_checked(
                program_id, account, vault, ncn, slasher, operator, epoch,
            )?);

        Ok(Self {
            account,
            vault_ncn_slasher_operator_ticket,
        })
    }

    pub const fn account(&self) -> &AccountInfo<'info> {
        self.account
    }

    pub const fn vault_ncn_slasher_operator_ticket(&self) -> &VaultNcnSlasherOperatorTicket {
        &self.vault_ncn_slasher_operator_ticket
    }

    pub fn vault_ncn_slasher_operator_ticket_mut(&mut self) -> &mut VaultNcnSlasherOperatorTicket {
        &mut self.vault_ncn_slasher_operator_ticket
    }

    pub fn save(&self) -> ProgramResult {
        borsh::to_writer(
            &mut self.account.data.borrow_mut()[..],
            &*self.vault_ncn_slasher_operator_ticket,
        )?;
        Ok(())
    }
}
