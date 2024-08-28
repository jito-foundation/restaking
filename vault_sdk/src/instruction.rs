use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankInstruction;
use solana_program::program_error::ProgramError;

#[rustfmt::skip]
#[derive(Debug, BorshSerialize, BorshDeserialize, ShankInstruction)]
pub enum VaultInstruction {
    /// Initializes global configuration
    #[account(0, writable, name = "config")]
    #[account(1, writable, signer, name = "admin")]
    #[account(2, name = "restaking_program")]
    #[account(3, name = "system_program")]
    InitializeConfig,

    /// Initializes the vault
    #[account(0, writable, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, writable, signer, name = "vrt_mint")]
    #[account(3, name = "token_mint")]
    #[account(4, writable, signer, name = "admin")]
    #[account(5, signer, name = "base")]
    #[account(6, name = "system_program")]
    #[account(7, name = "token_program")]
    InitializeVault {
        deposit_fee_bps: u16,
        withdrawal_fee_bps: u16,
        reward_fee_bps: u16,
    },

    /// Initializes a vault with an already-created VRT mint
    InitializeVaultWithMint,

    /// Vault adds support for an operator
    #[account(0, name = "config")]
    #[account(1, name = "vault")]
    #[account(2, writable, name = "operator")]
    #[account(3, name = "operator_vault_ticket")]
    #[account(4, writable, name = "vault_operator_delegation")]
    #[account(5, signer, name = "admin")]
    #[account(6, writable, signer, name = "payer")]
    #[account(7, name = "system_program")]
    InitializeVaultOperatorDelegation,

    /// Vault adds support for the NCN
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, name = "ncn")]
    #[account(3, name = "ncn_vault_ticket")]
    #[account(4, writable, name = "vault_ncn_ticket")]
    #[account(5, signer, name = "admin")]
    #[account(6, writable, signer, name = "payer")]
    #[account(7, name = "system_program")]
    InitializeVaultNcnTicket,

    /// Initializes the account which keeps track of how much an operator has been slashed
    /// by a slasher for a given NCN and vault for a given epoch.
    #[account(0, name = "config")]
    #[account(1, name = "vault")]
    #[account(2, name = "ncn")]
    #[account(3, name = "slasher")]
    #[account(4, name = "operator")]
    #[account(5, name = "vault_ncn_slasher_ticket")]
    #[account(6, writable, name = "vault_ncn_slasher_operator_ticket")]
    #[account(7, writable, signer, name = "payer")]
    #[account(8, name = "system_program")]
    InitializeVaultNcnSlasherOperatorTicket,

    /// Registers a slasher with the vault
    #[account(0, name = "config")]
    #[account(1, name = "vault")]
    #[account(2, name = "ncn")]
    #[account(3, name = "slasher")]
    #[account(4, name = "ncn_slasher_ticket")]
    #[account(5, writable, name = "vault_slasher_ticket")]
    #[account(6, signer, name = "admin")]
    #[account(7, signer, writable, name = "payer")]
    #[account(8, name = "system_program")]
    InitializeVaultNcnSlasherTicket,

    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, name = "ncn")]
    #[account(3, writable, name = "vault_ncn_ticket")]
    #[account(4, signer, name = "admin")]
    WarmupVaultNcnTicket,

    /// Vault removes support for an NCN
    #[account(0, name = "config")]
    #[account(1, name = "vault")]
    #[account(2, name = "ncn")]
    #[account(3, writable, name = "vault_ncn_ticket")]
    #[account(4, signer, name = "admin")]
    CooldownVaultNcnTicket,

    #[account(0, name = "config")]
    #[account(1, name = "vault")]
    #[account(2, name = "ncn")]
    #[account(3, name = "slasher")]
    #[account(4, writable, name = "vault_slasher_ticket")]
    #[account(5, signer, name = "admin")]
    WarmupVaultNcnSlasherTicket,

    #[account(0, name = "config")]
    #[account(1, name = "vault")]
    #[account(2, name = "ncn")]
    #[account(3, name = "slasher")]
    #[account(4, writable, name = "vault_ncn_slasher_ticket")]
    #[account(5, signer, name = "admin")]
    CooldownVaultNcnSlasherTicket,

    /// Mints VRT by depositing tokens into the vault
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, writable, name = "vrt_mint")]
    #[account(3, writable, signer, name = "depositor")]
    #[account(4, writable, name = "depositor_token_account")]
    #[account(5, writable, name = "vault_token_account")]
    #[account(6, writable, name = "depositor_vrt_token_account")]
    #[account(7, writable, name = "vault_fee_token_account")]
    #[account(8, name = "token_program")]
    #[account(9, signer, optional, name = "mint_signer", description = "Signer for minting")]
    MintTo {
        amount_in: u64,
        min_amount_out: u64,
    },

    /// Burns VRT by withdrawing tokens from the vault
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, writable, name = "vault_token_account")]
    #[account(3, writable, name = "vrt_mint")]
    #[account(4, signer, name = "staker")]
    #[account(5, writable, name = "staker_token_account")]
    #[account(6, signer, name = "staker_vrt_token_account")]
    #[account(7, writable, name = "vault_fee_token_account")]
    #[account(8, name = "token_program")]
    #[account(9, name = "system_program")]
    #[account(10, signer, optional, name = "burn_signer", description = "Signer for burning")]
    Burn {
        amount_in: u64,
        min_amount_out: u64
    },

    /// Enqueues a withdrawal of VRT tokens
    /// Used when there aren't enough idle assets in the vault to cover a withdrawal
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, writable, name = "vault_staker_withdrawal_ticket")]
    #[account(3, writable, name = "vault_staker_withdrawal_ticket_token_account")]
    #[account(4, writable, signer, name = "staker")]
    #[account(5, writable, name = "staker_vrt_token_account")]
    #[account(6, signer, name = "base")]
    #[account(7, name = "token_program")]
    #[account(8, name = "system_program")]
    #[account(9, signer, optional, name = "burn_signer", description = "Signer for burning")]
    EnqueueWithdrawal {
        amount: u64
    },

    #[account(0, name = "config")]
    #[account(1, name = "vault")]
    #[account(2, writable, name = "vault_staker_withdrawal_ticket")]
    #[account(3, signer, name = "old_owner")]
    #[account(4, name = "new_owner")]
    ChangeWithdrawalTicketOwner,

    /// Burns the withdraw ticket, returning funds to the staker. Withdraw tickets can be burned
    /// after one full epoch of being enqueued.
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, writable, name = "vault_token_account")]
    #[account(3, writable, name = "vrt_mint")]
    #[account(4, writable, name = "staker")]
    #[account(5, writable, name = "staker_token_account")]
    #[account(6, writable, name = "vault_staker_withdrawal_ticket")]
    #[account(7, writable, name = "vault_staker_withdrawal_ticket_token_account")]
    #[account(8, writable, name = "vault_fee_token_account")]
    #[account(9, name = "token_program")]
    #[account(10, name = "system_program")]
    #[account(11, signer, optional, name = "burn_signer", description = "Signer for burning")]
    BurnWithdrawTicket {
        min_amount_out: u64
    },

    /// Sets the max tokens that can be deposited into the VRT
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, signer, name = "admin")]
    SetDepositCapacity {
        amount: u64
    },

    /// Sets the fees for depositing and withdrawing
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, signer, name = "admin")]
    SetFees {
        deposit_fee_bps: Option<u16>,
        withdrawal_fee_bps: Option<u16>,
        reward_fee_bps: Option<u16>,
    },

    /// Delegate the token account to a third party
    #[account(0, name = "config")]
    #[account(1, name = "vault")]
    #[account(2, signer, name = "admin")]
    #[account(3, name = "token_mint")]
    #[account(4, writable, name = "token_account")]
    #[account(5, name = "delegate")]
    #[account(6, name = "token_program")]
    DelegateTokenAccount {
        amount: u64
    },

    /// Changes the signer for vault admin
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, signer, name = "old_admin")]
    #[account(3, signer, name = "new_admin")]
    SetAdmin,

    /// Changes the signer for vault delegation
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, signer, name = "admin")]
    #[account(3, name = "new_admin")]
    SetSecondaryAdmin(VaultAdminRole),

    /// Delegates a token amount to a specific node operator
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, name = "operator")]
    #[account(3, writable, name = "vault_operator_delegation")]
    #[account(4, signer, name = "admin")]
    #[account(5, writable, signer, name = "payer")]
    #[account(6, name = "system_program")]
    AddDelegation {
        amount: u64,
    },

    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, name = "operator")]
    #[account(3, writable, name = "vault_operator_delegation")]
    #[account(4, signer, name = "admin")]
    CooldownDelegation {
        amount: u64,
    },

    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, name = "vault_token_account")]
    #[account(3, writable, name = "vrt_mint")]
    #[account(4, writable, name = "vault_fee_token_account")]
    #[account(5, name = "token_program")]
    UpdateVaultBalance,

    /// Starts updating the vault
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, writable, name = "vault_update_state_tracker")]
    #[account(3, writable, name = "payer")]
    #[account(4, name = "system_program")]
    InitializeVaultUpdateStateTracker { withdrawal_allocation_method: WithdrawalAllocationMethod },

    /// Shall be called on every vault_operator_delegation
    #[account(0, name = "config")]
    #[account(1, name = "vault")]
    #[account(2, name = "operator")]
    #[account(3, writable, name = "vault_operator_delegation")]
    #[account(4, writable, name = "vault_update_state_tracker")]
    CrankVaultUpdateStateTracker,

    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, writable, name = "vault_update_state_tracker")]
    #[account(3, writable, signer, name = "payer")]
    CloseVaultUpdateStateTracker {
        ncn_epoch: u64
    },

    /// Creates token metadata for the vault VRT
    #[account(0, name = "vault")]
    #[account(1, signer, name = "admin")]
    #[account(2, name = "vrt_mint")]
    #[account(3, writable, signer, name = "payer")]
    #[account(4, writable, name = "metadata")]
    #[account(5, name = "mpl_token_metadata_program")]
    #[account(6, name = "system_program")]
    CreateTokenMetadata {
        name: String,
        symbol: String,
        uri: String,
    },

    /// Updates token metadata for the vault VRT
    #[account(0, name = "vault")]
    #[account(1, signer, name = "admin")]
    #[account(2, name = "vrt_mint")]
    #[account(3, writable, name = "metadata")]
    #[account(4, name = "mpl_token_metadata_program")]
    UpdateTokenMetadata {
        name: String,
        symbol: String,
        uri: String,
    },

    /// Slashes an amount of tokens from the vault
    #[account(0, name = "config")]
    #[account(1, writable, name = "vault")]
    #[account(2, name = "ncn")]
    #[account(3, name = "operator")]
    #[account(4, name = "slasher")]
    #[account(5, name = "ncn_operator_state")]
    #[account(6, name = "ncn_vault_ticket")]
    #[account(7, name = "operator_vault_ticket")]
    #[account(8, name = "vault_ncn_ticket")]
    #[account(9, writable, name = "vault_operator_delegation")]
    #[account(10, name = "ncn_vault_slasher_ticket")]
    #[account(11, name = "vault_ncn_slasher_ticket")]
    #[account(12, writable, name = "vault_ncn_slasher_operator_ticket")]
    #[account(13, writable, name = "vault_token_account")]
    #[account(14, name = "slasher_token_account")]
    #[account(15, name = "token_program")]
    Slash {
        amount: u64
    },
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum VaultAdminRole {
    DelegationAdmin,
    OperatorAdmin,
    NcnAdmin,
    SlasherAdmin,
    CapacityAdmin,
    FeeWallet,
    MintBurnAdmin,
    WithdrawAdmin,
    FeeAdmin,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
#[repr(u8)]
pub enum WithdrawalAllocationMethod {
    /// During withdrawal allocation, the greedy mode will subtract assets from operator delegations
    /// its iterating over in order to fulfill the withdrawal.
    Greedy,
}

impl TryFrom<u8> for WithdrawalAllocationMethod {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Greedy),
            _ => Err(ProgramError::InvalidArgument),
        }
    }
}
