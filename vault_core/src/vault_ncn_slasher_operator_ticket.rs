//! The [`VaultNcnSlasherOperatorTicket`] account tracks the amount an operator has been slashed
//! by a slasher for a given node consensus network (NCN) and vault for a given epoch.
use bytemuck::{Pod, Zeroable};
use jito_account_traits::{AccountDeserialize, Discriminator};
use solana_program::{account_info::AccountInfo, msg, program_error::ProgramError, pubkey::Pubkey};

impl Discriminator for VaultNcnSlasherOperatorTicket {
    const DISCRIMINATOR: u8 = 6;
}

/// The [`VaultNcnSlasherOperatorTicket`] account tracks the amount an operator has been slashed
/// by a slasher for a given node consensus network (NCN) and vault for a given epoch. It helps
/// ensure that the operator is held accountable for their actions and that slashing conditions
/// aren't exceeded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, AccountDeserialize)]
#[repr(C)]
pub struct VaultNcnSlasherOperatorTicket {
    /// The vault slashed
    pub vault: Pubkey,

    /// The node consensus network slashed
    pub ncn: Pubkey,

    /// The slasher
    pub slasher: Pubkey,

    /// The operator
    pub operator: Pubkey,

    /// The epoch
    pub epoch: u64,

    /// The amount slashed for the given epoch
    pub slashed: u64,

    /// The bump seed for the PDA
    pub bump: u8,

    /// Reserved space
    reserved: [u8; 7],
}

impl VaultNcnSlasherOperatorTicket {
    pub const fn new(
        vault: Pubkey,
        ncn: Pubkey,
        slasher: Pubkey,
        operator: Pubkey,
        epoch: u64,
        bump: u8,
    ) -> Self {
        Self {
            vault,
            ncn,
            slasher,
            operator,
            epoch,
            slashed: 0,
            bump,
            reserved: [0; 7],
        }
    }

    /// Returns the seeds for the PDA
    ///
    /// # Arguments
    /// * `vault` - The vault
    /// * `ncn` - The node consensus network
    /// * `slasher` - The slasher
    /// * `operator` - The operator
    /// * `epoch` - The NCN epoch
    ///
    /// # Returns
    /// The seeds for the PDA
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

    /// Find the program address for the [`VaultNcnSlasherOperatorTicket`]
    /// account.
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `vault` - The vault
    /// * `ncn` - The node consensus network
    /// * `slasher` - The slasher
    /// * `operator` - The operator
    /// * `epoch` - The NCN epoch
    ///
    /// # Returns
    /// * [`Pubkey`] - The program address
    /// * `u8` - The bump seed
    /// * `Vec<Vec<u8>>` - The seeds used to generate the PDA
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

    /// Loads the [`VaultNcnSlasherOperatorTicket`] account
    ///
    /// # Arguments
    /// * `program_id` - The program ID
    /// * `vault_ncn_slasher_operator_ticket` - The [`VaultNcnSlasherOperatorTicket`] account
    /// * `vault` - The [`Vault`] account
    /// * `ncn` - The ncn account
    /// * `slasher` - The slasher account
    /// * `operator` - The operator account
    /// * `ncn_epoch` - The ncn epoch
    /// * `expect_writable` - Whether the account should be writable
    ///
    /// # Returns
    /// * `Result<(), ProgramError>` - The result of the operation
    #[allow(clippy::too_many_arguments)]
    pub fn load(
        program_id: &Pubkey,
        vault_ncn_slasher_operator_ticket: &AccountInfo,
        vault: &AccountInfo,
        ncn: &AccountInfo,
        slasher: &AccountInfo,
        operator: &AccountInfo,
        ncn_epoch: u64,
        expect_writable: bool,
    ) -> Result<(), ProgramError> {
        if vault_ncn_slasher_operator_ticket.owner.ne(program_id) {
            msg!("Vault NCN slasher operator has an invalid owner");
            return Err(ProgramError::InvalidAccountOwner);
        }
        if vault_ncn_slasher_operator_ticket.data_is_empty() {
            msg!("Vault NCN slasher operator data is empty");
            return Err(ProgramError::InvalidAccountData);
        }
        if expect_writable && !vault_ncn_slasher_operator_ticket.is_writable {
            msg!("Vault NCN slasher operator is not writable");
            return Err(ProgramError::InvalidAccountData);
        }
        if vault_ncn_slasher_operator_ticket.data.borrow()[0].ne(&Self::DISCRIMINATOR) {
            msg!("Vault NCN slasher operator discriminator is invalid");
            return Err(ProgramError::InvalidAccountData);
        }
        let expected_pubkey = Self::find_program_address(
            program_id,
            vault.key,
            ncn.key,
            slasher.key,
            operator.key,
            ncn_epoch,
        )
        .0;
        if vault_ncn_slasher_operator_ticket.key.ne(&expected_pubkey) {
            msg!("Vault NCN slasher operator is not at the correct PDA");
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}
