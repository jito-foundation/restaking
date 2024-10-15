use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankInstruction;

#[derive(Debug, BorshSerialize, BorshDeserialize, ShankInstruction)]
pub enum RestakingInstruction {
    /// Initializes the global configuration
    #[account(0, writable, name = "config")]
    #[account(1, writable, signer, name = "admin")]
    #[account(2, name = "vault_program")]
    #[account(3, name = "system_program")]
    InitializeConfig,

    /// Initializes the NCN
    #[account(0, writable, name = "config")]
    #[account(1, writable, name = "ncn")]
    #[account(2, writable, signer, name = "admin")]
    #[account(3, signer, name = "base")]
    #[account(4, name = "system_program")]
    InitializeNcn,

    /// Initializes a operator
    #[account(0, writable, name = "config")]
    #[account(1, writable, name = "operator")]
    #[account(2, writable, signer, name = "admin")]
    #[account(3, signer, name = "base")]
    #[account(4, name = "system_program")]
    InitializeOperator { operator_fee_bps: u16 },

    /// The NCN adds support for a vault slasher
    ///
    /// # Arguments
    /// * `u64` - The maximum amount that can be slashed from the vault per epoch
    #[account(0, name = "config")]
    #[account(1, writable, name = "ncn")]
    #[account(2, name = "vault")]
    #[account(3, name = "slasher")]
    #[account(4, name = "ncn_vault_ticket")]
    #[account(5, writable, name = "ncn_vault_slasher_ticket")]
    #[account(6, signer, name = "admin")]
    #[account(7, writable, signer, name = "payer")]
    #[account(8, name = "system_program")]
    InitializeNcnVaultSlasherTicket(u64),

    /// NCN adds support for receiving delegation from a vault
    #[account(0, name = "config")]
    #[account(1, writable, name = "ncn")]
    #[account(2, name = "vault")]
    #[account(3, writable, name = "ncn_vault_ticket")]
    #[account(4, signer, name = "admin")]
    #[account(5, writable, signer, name = "payer")]
    #[account(6, name = "system_program")]
    InitializeNcnVaultTicket,

    /// Operator adds support for receiving delegation from a vault
    #[account(0, name = "config")]
    #[account(1, writable, name = "operator")]
    #[account(2, name = "vault")]
    #[account(3, writable, name = "operator_vault_ticket")]
    #[account(4, signer, name = "admin")]
    #[account(5, writable, signer, name = "payer")]
    #[account(6, name = "system_program")]
    InitializeOperatorVaultTicket,

    /// After the operator has signaled they are ready to join the network,
    /// the NCN admin can add the operator to the NCN
    #[account(0, name = "config")]
    #[account(1, writable, name = "ncn")]
    #[account(2, writable, name = "operator")]
    #[account(3, writable, name = "ncn_operator_state")]
    #[account(4, signer, name = "admin")]
    #[account(5, writable, signer, name = "payer")]
    #[account(6, name = "system_program")]
    InitializeNcnOperatorState,

    #[account(0, name = "config")]
    #[account(1, name = "ncn")]
    #[account(2, name = "vault")]
    #[account(3, writable, name = "ncn_vault_ticket")]
    #[account(4, signer, name = "admin")]
    WarmupNcnVaultTicket,

    /// NCN removes support for receiving delegation from a vault
    #[account(0, name = "config")]
    #[account(1, name = "ncn")]
    #[account(2, name = "vault")]
    #[account(3, writable, name = "ncn_vault_ticket")]
    #[account(4, signer, name = "admin")]
    CooldownNcnVaultTicket,

    #[account(0, name = "config")]
    #[account(1, name = "ncn")]
    #[account(2, name = "operator")]
    #[account(3, writable, name = "ncn_operator_state")]
    #[account(4, signer, name = "admin")]
    NcnWarmupOperator,

    #[account(0, name = "config")]
    #[account(1, name = "ncn")]
    #[account(2, name = "operator")]
    #[account(3, writable, name = "ncn_operator_state")]
    #[account(4, signer, name = "admin")]
    NcnCooldownOperator,

    #[account(0, name = "config")]
    #[account(1, name = "ncn")]
    #[account(2, name = "operator")]
    #[account(3, writable, name = "ncn_operator_state")]
    #[account(4, signer, name = "admin")]
    OperatorWarmupNcn,

    #[account(0, name = "config")]
    #[account(1, name = "ncn")]
    #[account(2, name = "operator")]
    #[account(3, writable, name = "ncn_operator_state")]
    #[account(4, signer, name = "admin")]
    OperatorCooldownNcn,

    #[account(0, name = "config")]
    #[account(1, name = "ncn")]
    #[account(2, name = "vault")]
    #[account(3, name = "slasher")]
    #[account(4, name = "ncn_vault_ticket")]
    #[account(5, writable, name = "ncn_vault_slasher_ticket")]
    #[account(6, signer, name = "admin")]
    WarmupNcnVaultSlasherTicket,

    /// NCN removes support for a slasher
    #[account(0, name = "config")]
    #[account(1, name = "ncn")]
    #[account(2, name = "vault")]
    #[account(3, name = "slasher")]
    #[account(4, writable, name = "ncn_vault_slasher_ticket")]
    #[account(5, signer, name = "admin")]
    CooldownNcnVaultSlasherTicket,

    #[account(0, name = "config")]
    #[account(1, name = "operator")]
    #[account(2, name = "vault")]
    #[account(3, writable, name = "operator_vault_ticket")]
    #[account(4, signer, name = "admin")]
    WarmupOperatorVaultTicket,

    /// Node operator removes support for receiving delegation from a vault
    #[account(0, name = "config")]
    #[account(1, name = "operator")]
    #[account(2, name = "vault")]
    #[account(3, writable, name = "operator_vault_ticket")]
    #[account(4, signer, name = "admin")]
    CooldownOperatorVaultTicket,

    #[account(0, writable, name = "ncn")]
    #[account(1, signer, name = "old_admin")]
    #[account(2, signer, name = "new_admin")]
    NcnSetAdmin,

    #[account(0, writable, name = "ncn")]
    #[account(1, signer, name = "admin")]
    #[account(2, name = "new_admin")]
    NcnSetSecondaryAdmin(NcnAdminRole),

    /// Sets the admin for a node operator
    #[account(0, writable, name = "operator")]
    #[account(1, signer, name = "old_admin")]
    #[account(2, signer, name = "new_admin")]
    OperatorSetAdmin,

    /// Sets the voter for a node operator
    #[account(0, writable, name = "operator")]
    #[account(1, signer, name = "admin")]
    #[account(2, name = "new_admin")]
    OperatorSetSecondaryAdmin(OperatorAdminRole),

    /// Sets the fee for a node operator
    #[account(0, name = "config")]
    #[account(1, writable, name = "operator")]
    #[account(2, signer, name = "admin")]
    OperatorSetFee { new_fee_bps: u16 },

    #[account(0, name = "ncn")]
    #[account(1, signer, name = "delegate_admin")]
    #[account(2, name = "token_mint")]
    #[account(3, writable, name = "token_account")]
    #[account(4, name = "delegate")]
    #[account(5, name = "token_program")]
    NcnDelegateTokenAccount,

    #[account(0, name = "operator")]
    #[account(1, signer, name = "delegate_admin")]
    #[account(2, name = "token_mint")]
    #[account(3, writable, name = "token_account")]
    #[account(4, name = "delegate")]
    #[account(5, name = "token_program")]
    OperatorDelegateTokenAccount,
}

#[derive(Debug, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub enum NcnAdminRole {
    Operator,
    Vault,
    Slasher,
    Delegate,
}

#[derive(Debug, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub enum OperatorAdminRole {
    NcnAdmin,
    VaultAdmin,
    VoterAdmin,
    DelegateAdmin,
}
