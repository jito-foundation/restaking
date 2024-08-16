use jito_account_traits::AccountDeserialize;
use jito_jsm_core::loader::{load_associated_token_account, load_token_program};
use jito_restaking_core::{
    loader::{
        load_ncn, load_ncn_operator_state, load_ncn_vault_slasher_ticket, load_ncn_vault_ticket,
        load_operator, load_operator_vault_ticket,
    },
    ncn_operator_state::NcnOperatorState,
    ncn_vault_slasher_ticket::NcnVaultSlasherTicket,
    ncn_vault_ticket::NcnVaultTicket,
    operator_vault_ticket::OperatorVaultTicket,
};
use jito_vault_core::{
    config::Config,
    loader::{
        load_config, load_vault, load_vault_ncn_slasher_operator_ticket, load_vault_ncn_ticket,
        load_vault_operator_delegation,
    },
    vault::Vault,
    vault_ncn_slasher_operator_ticket::VaultNcnSlasherOperatorTicket,
    vault_ncn_slasher_ticket::VaultNcnSlasherTicket,
    vault_ncn_ticket::VaultNcnTicket,
    vault_operator_delegation::VaultOperatorDelegation,
};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program::invoke_signed, program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};
use spl_token::instruction::transfer;

/// Processes the vault slash instruction: [`crate::VaultInstruction::Slash`]
pub fn process_slash(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    slash_amount: u64,
) -> ProgramResult {
    let [config, vault_info, ncn, operator, slasher, ncn_operator_state, ncn_vault_ticket, operator_vault_ticket, vault_ncn_ticket, vault_operator_delegation, ncn_vault_slasher_ticket, vault_ncn_slasher_ticket, vault_ncn_slasher_operator_ticket, vault_token_account, slasher_token_account, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_config(program_id, config, false)?;
    load_vault(program_id, vault_info, false)?;
    let config_data = config.data.borrow();
    let config = Config::try_from_slice(&config_data)?;
    load_ncn(&config.restaking_program, ncn, false)?;
    load_operator(&config.restaking_program, operator, false)?;
    // slasher
    load_ncn_operator_state(
        &config.restaking_program,
        ncn_operator_state,
        ncn,
        operator,
        false,
    )?;
    load_ncn_vault_ticket(
        &config.restaking_program,
        ncn_vault_ticket,
        ncn,
        vault_info,
        false,
    )?;
    load_operator_vault_ticket(
        &config.restaking_program,
        operator_vault_ticket,
        operator,
        vault_info,
        false,
    )?;
    load_vault_ncn_ticket(program_id, vault_ncn_ticket, vault_info, ncn, false)?;
    load_vault_operator_delegation(
        program_id,
        vault_operator_delegation,
        vault_info,
        operator,
        true,
    )?;
    load_ncn_vault_slasher_ticket(
        &config.restaking_program,
        ncn_vault_slasher_ticket,
        ncn,
        vault_info,
        slasher,
        false,
    )?;
    let ncn_epoch = Clock::get()?.slot.checked_div(config.epoch_length).unwrap();
    load_vault_ncn_slasher_operator_ticket(
        program_id,
        vault_ncn_slasher_operator_ticket,
        vault_info,
        ncn,
        slasher,
        operator,
        ncn_epoch,
        true,
    )?;
    let mut vault_data = vault_info.data.borrow_mut();
    let vault = Vault::try_from_slice_mut(&mut vault_data)?;
    load_associated_token_account(vault_token_account, vault_info.key, &vault.supported_mint)?;
    load_associated_token_account(slasher_token_account, slasher.key, &vault.supported_mint)?;
    load_token_program(token_program)?;

    let slot = Clock::get()?.slot;
    let epoch_length = config.epoch_length;

    // The vault shall be up-to-date before slashing
    if vault.is_update_needed(Clock::get()?.slot, config.epoch_length) {
        msg!("Vault update is needed");
        return Err(VaultError::VaultUpdateNeeded.into());
    }

    // All ticket states shall be active or cooling down
    let vault_ncn_ticket_data = vault_ncn_ticket.data.borrow();
    let vault_ncn_ticket = VaultNcnTicket::try_from_slice(&vault_ncn_ticket_data)?;
    let ncn_vault_ticket_data = ncn_vault_ticket.data.borrow();
    let ncn_vault_ticket = NcnVaultTicket::try_from_slice(&ncn_vault_ticket_data)?;
    let operator_vault_ticket_data = operator_vault_ticket.data.borrow();
    let operator_vault_ticket = OperatorVaultTicket::try_from_slice(&operator_vault_ticket_data)?;
    let ncn_operator_state_data = ncn_operator_state.data.borrow();
    let ncn_operator_state = NcnOperatorState::try_from_slice(&ncn_operator_state_data)?;
    let ncn_vault_slasher_ticket_data = ncn_vault_slasher_ticket.data.borrow();
    let ncn_vault_slasher_ticket =
        NcnVaultSlasherTicket::try_from_slice(&ncn_vault_slasher_ticket_data)?;
    let vault_ncn_slasher_ticket_data = vault_ncn_slasher_ticket.data.borrow();
    let vault_ncn_slasher_ticket =
        VaultNcnSlasherTicket::try_from_slice(&vault_ncn_slasher_ticket_data)?;
    check_states_active_or_cooling_down(
        &vault_ncn_slasher_ticket,
        &ncn_vault_slasher_ticket,
        &ncn_operator_state,
        &operator_vault_ticket,
        &vault_ncn_ticket,
        &ncn_vault_ticket,
        slot,
        epoch_length,
    )?;

    // The amount slashed for this operator shall not exceed the maximum slashable amount per epoch
    let mut vault_ncn_slasher_operator_ticket_data =
        vault_ncn_slasher_operator_ticket.data.borrow_mut();
    let vault_ncn_slasher_operator_ticket = VaultNcnSlasherOperatorTicket::try_from_slice_mut(
        &mut vault_ncn_slasher_operator_ticket_data,
    )?;
    check_slashing_amount_not_exceeded(
        vault_ncn_slasher_ticket,
        vault_ncn_slasher_operator_ticket,
        slash_amount,
    )?;

    // The VaultOperatorDelegation shall be slashed and the vault amounts shall be updated
    let mut vault_operator_delegation_data = vault_operator_delegation.data.borrow_mut();
    let vault_operator_delegation =
        VaultOperatorDelegation::try_from_slice_mut(&mut vault_operator_delegation_data)?;
    slash_and_update_vault(
        vault,
        vault_operator_delegation,
        vault_ncn_slasher_operator_ticket,
        slash_amount,
    )?;

    // transfer the slashed funds
    let mut vault_seeds = Vault::seeds(&vault.base);
    vault_seeds.push(vec![vault.bump]);
    let vault_seeds_slice = vault_seeds
        .iter()
        .map(|seed| seed.as_slice())
        .collect::<Vec<&[u8]>>();
    drop(vault_data);
    invoke_signed(
        &transfer(
            &spl_token::id(),
            vault_token_account.key,
            slasher_token_account.key,
            vault_info.key,
            &[],
            slash_amount,
        )?,
        &[
            vault_token_account.clone(),
            slasher_token_account.clone(),
            vault_info.clone(),
        ],
        &[vault_seeds_slice.as_slice()],
    )?;

    Ok(())
}

fn check_states_active_or_cooling_down(
    vault_ncn_slasher_ticket: &VaultNcnSlasherTicket,
    ncn_vault_slasher_ticket: &NcnVaultSlasherTicket,
    ncn_operator_state: &NcnOperatorState,
    operator_vault_ticket: &OperatorVaultTicket,
    vault_ncn_ticket: &VaultNcnTicket,
    ncn_vault_ticket: &NcnVaultTicket,
    slot: u64,
    epoch_length: u64,
) -> ProgramResult {
    if !vault_ncn_slasher_ticket
        .state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("Vault NCN slasher ticket is not active or in cooldown");
        return Err(VaultError::VaultNcnSlasherTicketUnslashable.into());
    }
    if !ncn_vault_slasher_ticket
        .state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("NCN vault slasher ticket is not active or in cooldown");
        return Err(VaultError::NcnVaultSlasherTicketUnslashable.into());
    }
    if !ncn_operator_state
        .ncn_opt_in_state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("NCN opt-in to operator is not active or in cooldown");
        return Err(VaultError::NcnOperatorStateUnslashable.into());
    }
    if !ncn_operator_state
        .operator_opt_in_state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("Operator opt-in to NCN is not active or in cooldown");
        return Err(VaultError::NcnOperatorStateUnslashable.into());
    }
    if !operator_vault_ticket
        .state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("Operator vault ticket is not active or in cooldown");
        return Err(VaultError::OperatorVaultTicketUnslashable.into());
    }
    if !vault_ncn_ticket
        .state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("Vault NCN ticket is not active or in cooldown");
        return Err(VaultError::VaultNcnTicketUnslashable.into());
    }
    if !ncn_vault_ticket
        .state
        .is_active_or_cooldown(slot, epoch_length)
    {
        msg!("NCN vault ticket is not active or in cooldown");
        return Err(VaultError::NcnVaultTicketUnslashable.into());
    }
    Ok(())
}

/// Checks the slashing amount for a given operator does not exceed the maximum slashable amount per epoch
/// as defined in the [`VaultNcnSlasherTicket`].
fn check_slashing_amount_not_exceeded(
    vault_ncn_slasher_ticket: &VaultNcnSlasherTicket,
    vault_ncn_slasher_operator_ticket: &VaultNcnSlasherOperatorTicket,
    slash_amount: u64,
) -> ProgramResult {
    let amount_after_slash = vault_ncn_slasher_operator_ticket
        .slashed
        .checked_add(slash_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    if amount_after_slash > vault_ncn_slasher_ticket.max_slashable_per_epoch {
        msg!("Slash amount exceeds the maximum slashable amount per epoch");
        return Err(VaultError::VaultNcnSlasherOperatorMaxSlashableExceeded.into());
    }
    Ok(())
}

/// Slashes the vault and updates the vault amounts based on the slashing amount.
fn slash_and_update_vault(
    vault: &mut Vault,
    vault_operator_delegation: &mut VaultOperatorDelegation,
    vault_ncn_slasher_operator_ticket: &mut VaultNcnSlasherOperatorTicket,
    slash_amount: u64,
) -> ProgramResult {
    let amount_enqueued_before_slash = vault_operator_delegation
        .enqueued_for_withdraw_amount
        .checked_add(vault_operator_delegation.enqueued_for_cooldown_amount)
        .ok_or(VaultError::VaultOverflow)?;
    let amount_cooling_down_before_slash = vault_operator_delegation
        .cooling_down_for_withdraw_amount
        .checked_add(vault_operator_delegation.cooling_down_amount)
        .ok_or(VaultError::VaultOverflow)?;

    vault_operator_delegation.slash(slash_amount)?;

    let amount_enqueued_after_slash = vault_operator_delegation
        .enqueued_for_withdraw_amount
        .checked_add(vault_operator_delegation.enqueued_for_cooldown_amount)
        .ok_or(VaultError::VaultOverflow)?;
    let amount_cooling_down_after_slash = vault_operator_delegation
        .cooling_down_for_withdraw_amount
        .checked_add(vault_operator_delegation.cooling_down_amount)
        .ok_or(VaultError::VaultOverflow)?;

    // Calculate how much was slashed from each category
    let enqueued_slashed = amount_enqueued_before_slash
        .checked_sub(amount_enqueued_after_slash)
        .ok_or(VaultError::VaultUnderflow)?;
    let cooling_down_slashed = amount_cooling_down_before_slash
        .checked_sub(amount_cooling_down_after_slash)
        .ok_or(VaultError::VaultUnderflow)?;

    vault.tokens_deposited = vault
        .tokens_deposited
        .checked_sub(slash_amount)
        .ok_or(VaultError::VaultOverflow)?;
    vault.amount_delegated = vault
        .amount_delegated
        .checked_sub(slash_amount)
        .ok_or(VaultError::VaultOverflow)?;

    // Update the vault amounts based on before and after slashing
    vault.amount_enqueued_for_cooldown = vault
        .amount_enqueued_for_cooldown
        .checked_sub(enqueued_slashed)
        .ok_or(VaultError::VaultUnderflow)?;
    vault.amount_cooling_down = vault
        .amount_cooling_down
        .checked_sub(cooling_down_slashed)
        .ok_or(VaultError::VaultUnderflow)?;

    vault_ncn_slasher_operator_ticket.slashed = vault_ncn_slasher_operator_ticket
        .slashed
        .checked_add(slash_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::slash::slash_and_update_vault;
    use jito_vault_core::vault::Vault;
    use jito_vault_core::vault_ncn_slasher_operator_ticket::VaultNcnSlasherOperatorTicket;
    use jito_vault_core::vault_operator_delegation::VaultOperatorDelegation;
    use jito_vault_sdk::error::VaultError;
    use solana_program::program_error::ProgramError;
    use solana_program::pubkey::Pubkey;

    struct TestConfig {
        vault_tokens_deposited: u64,
        vault_amount_delegated: u64,
        vault_amount_enqueued_for_cooldown: u64,
        vault_amount_cooling_down: u64,
        delegation_staked_amount: u64,
        delegation_enqueued_for_withdraw_amount: u64,
        delegation_enqueued_for_cooldown_amount: u64,
        delegation_cooling_down_for_withdraw_amount: u64,
        delegation_cooling_down_amount: u64,
        slash_amount: u64,
    }

    fn create_default_vault_and_vault_operator_delegation(
        config: &TestConfig,
    ) -> (Vault, VaultOperatorDelegation) {
        assert!(config.vault_tokens_deposited >= config.vault_amount_delegated);
        assert!(
            config.vault_amount_delegated
                >= config.vault_amount_enqueued_for_cooldown + config.vault_amount_cooling_down
        );
        assert!(
            config.vault_amount_delegated
                >= config.delegation_staked_amount
                    + config.delegation_enqueued_for_withdraw_amount
                    + config.delegation_enqueued_for_cooldown_amount
                    + config.delegation_cooling_down_for_withdraw_amount
                    + config.delegation_cooling_down_amount
        );
        assert!(
            config.vault_amount_enqueued_for_cooldown
                >= config.delegation_enqueued_for_withdraw_amount
                    + config.delegation_enqueued_for_cooldown_amount
        );
        assert!(
            config.vault_amount_cooling_down
                >= config.delegation_cooling_down_for_withdraw_amount
                    + config.delegation_cooling_down_amount
        );

        let mut vault = Vault::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            Pubkey::new_unique(),
            0,
            0,
            0,
        );
        vault.tokens_deposited = config.vault_tokens_deposited;
        vault.amount_delegated = config.vault_amount_delegated;
        vault.amount_enqueued_for_cooldown = config.vault_amount_enqueued_for_cooldown;
        vault.amount_cooling_down = config.vault_amount_cooling_down;

        let mut vault_operator_delegation =
            VaultOperatorDelegation::new(Pubkey::new_unique(), Pubkey::new_unique(), 0, 0);
        vault_operator_delegation.staked_amount = config.delegation_staked_amount;
        vault_operator_delegation.enqueued_for_withdraw_amount =
            config.delegation_enqueued_for_withdraw_amount;
        vault_operator_delegation.enqueued_for_cooldown_amount =
            config.delegation_enqueued_for_cooldown_amount;
        vault_operator_delegation.cooling_down_for_withdraw_amount =
            config.delegation_cooling_down_for_withdraw_amount;
        vault_operator_delegation.cooling_down_amount = config.delegation_cooling_down_amount;

        (vault, vault_operator_delegation)
    }

    fn create_default_vault_ncn_slasher_operator_ticket(
        slashed: u64,
    ) -> VaultNcnSlasherOperatorTicket {
        let mut vault_ncn_slasher_operator_ticket = VaultNcnSlasherOperatorTicket::new(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            0,
            0,
        );
        vault_ncn_slasher_operator_ticket.slashed = slashed;
        vault_ncn_slasher_operator_ticket
    }

    #[test]
    fn test_slash_greater_than_assets_fails() {
        const TEST_CONFIG: TestConfig = TestConfig {
            vault_tokens_deposited: 100,
            vault_amount_delegated: 100,
            vault_amount_enqueued_for_cooldown: 0,
            vault_amount_cooling_down: 0,
            delegation_staked_amount: 10,
            delegation_enqueued_for_withdraw_amount: 0,
            delegation_enqueued_for_cooldown_amount: 0,
            delegation_cooling_down_for_withdraw_amount: 0,
            delegation_cooling_down_amount: 0,
            slash_amount: 11,
        };
        const EXPECTED_VAULT_ERROR: VaultError = VaultError::VaultSlashUnderflow;

        let (mut vault, mut vault_operator_delegation) =
            create_default_vault_and_vault_operator_delegation(&TEST_CONFIG);
        let mut vault_ncn_slasher_opr_ticket = create_default_vault_ncn_slasher_operator_ticket(0);
        let result = slash_and_update_vault(
            &mut vault,
            &mut vault_operator_delegation,
            &mut vault_ncn_slasher_opr_ticket,
            TEST_CONFIG.slash_amount,
        );
        assert_eq!(
            result.unwrap_err(),
            ProgramError::Custom(EXPECTED_VAULT_ERROR.into())
        );
    }

    #[test]
    fn test_slash_exact_staked_amount() {
        let config = TestConfig {
            vault_tokens_deposited: 100,
            vault_amount_delegated: 100,
            vault_amount_enqueued_for_cooldown: 0,
            vault_amount_cooling_down: 0,
            delegation_staked_amount: 10,
            delegation_enqueued_for_withdraw_amount: 0,
            delegation_enqueued_for_cooldown_amount: 0,
            delegation_cooling_down_for_withdraw_amount: 0,
            delegation_cooling_down_amount: 0,
            slash_amount: 10,
        };
        let (mut vault, mut vault_operator_delegation) =
            create_default_vault_and_vault_operator_delegation(&config);
        let mut vault_ncn_slasher_opr_ticket = create_default_vault_ncn_slasher_operator_ticket(0);

        let result = slash_and_update_vault(
            &mut vault,
            &mut vault_operator_delegation,
            &mut vault_ncn_slasher_opr_ticket,
            config.slash_amount,
        );

        assert!(result.is_ok());
        assert_eq!(vault.tokens_deposited, 90);
        assert_eq!(vault.amount_delegated, 90);
        assert_eq!(vault_operator_delegation.staked_amount, 0);
        assert_eq!(vault_ncn_slasher_opr_ticket.slashed, 10);
    }

    #[test]
    fn test_slash_less_than_staked_amount() {
        let config = TestConfig {
            vault_tokens_deposited: 100,
            vault_amount_delegated: 100,
            vault_amount_enqueued_for_cooldown: 0,
            vault_amount_cooling_down: 0,
            delegation_staked_amount: 10,
            delegation_enqueued_for_withdraw_amount: 0,
            delegation_enqueued_for_cooldown_amount: 0,
            delegation_cooling_down_for_withdraw_amount: 0,
            delegation_cooling_down_amount: 0,
            slash_amount: 5,
        };
        let (mut vault, mut vault_operator_delegation) =
            create_default_vault_and_vault_operator_delegation(&config);
        let mut vault_ncn_slasher_opr_ticket = create_default_vault_ncn_slasher_operator_ticket(0);

        let result = slash_and_update_vault(
            &mut vault,
            &mut vault_operator_delegation,
            &mut vault_ncn_slasher_opr_ticket,
            config.slash_amount,
        );

        assert!(result.is_ok());
        assert_eq!(vault.tokens_deposited, 95);
        assert_eq!(vault.amount_delegated, 95);
        assert_eq!(vault_operator_delegation.staked_amount, 5);
        assert_eq!(vault_ncn_slasher_opr_ticket.slashed, 5);
    }

    #[test]
    fn test_slash_with_enqueued_for_cooldown() {
        let config = TestConfig {
            vault_tokens_deposited: 100,
            vault_amount_delegated: 100,
            vault_amount_enqueued_for_cooldown: 20,
            vault_amount_cooling_down: 0,
            delegation_staked_amount: 10,
            delegation_enqueued_for_withdraw_amount: 5,
            delegation_enqueued_for_cooldown_amount: 15,
            delegation_cooling_down_for_withdraw_amount: 0,
            delegation_cooling_down_amount: 0,
            slash_amount: 25,
        };
        let (mut vault, mut vault_operator_delegation) =
            create_default_vault_and_vault_operator_delegation(&config);
        let mut vault_ncn_slasher_opr_ticket = create_default_vault_ncn_slasher_operator_ticket(0);

        // vault_operator_delegation slashed 10 from staked account and 10 from delegation_enqueued_for_cooldown_amount
        let result = slash_and_update_vault(
            &mut vault,
            &mut vault_operator_delegation,
            &mut vault_ncn_slasher_opr_ticket,
            config.slash_amount,
        );

        assert!(result.is_ok());
        assert_eq!(vault.tokens_deposited, 75);
        assert_eq!(vault.amount_delegated, 75);
        assert_eq!(vault.amount_enqueued_for_cooldown, 5);
        assert_eq!(vault_operator_delegation.staked_amount, 0);
        assert_eq!(vault_operator_delegation.enqueued_for_withdraw_amount, 5);
        assert_eq!(vault_operator_delegation.enqueued_for_cooldown_amount, 0);
        assert_eq!(vault_ncn_slasher_opr_ticket.slashed, 25);
    }

    #[test]
    fn test_slash_with_cooling_down() {
        let config = TestConfig {
            vault_tokens_deposited: 100,
            vault_amount_delegated: 100,
            vault_amount_enqueued_for_cooldown: 0,
            vault_amount_cooling_down: 20,
            delegation_staked_amount: 10,
            delegation_enqueued_for_withdraw_amount: 0,
            delegation_enqueued_for_cooldown_amount: 0,
            delegation_cooling_down_for_withdraw_amount: 5,
            delegation_cooling_down_amount: 15,
            slash_amount: 25,
        };
        let (mut vault, mut vault_operator_delegation) =
            create_default_vault_and_vault_operator_delegation(&config);
        let mut vault_ncn_slasher_opr_ticket = create_default_vault_ncn_slasher_operator_ticket(0);

        let result = slash_and_update_vault(
            &mut vault,
            &mut vault_operator_delegation,
            &mut vault_ncn_slasher_opr_ticket,
            config.slash_amount,
        );

        assert!(result.is_ok());
        assert_eq!(vault.tokens_deposited, 75);
        assert_eq!(vault.amount_delegated, 75);
        assert_eq!(vault.amount_cooling_down, 5);
        assert_eq!(vault_operator_delegation.staked_amount, 0);
        assert_eq!(
            vault_operator_delegation.cooling_down_for_withdraw_amount,
            5
        );
        assert_eq!(vault_operator_delegation.cooling_down_amount, 0);
        assert_eq!(vault_ncn_slasher_opr_ticket.slashed, 25);
    }

    #[test]
    fn test_slash_with_mixed_states() {
        let config = TestConfig {
            vault_tokens_deposited: 100,
            vault_amount_delegated: 100,
            vault_amount_enqueued_for_cooldown: 20,
            vault_amount_cooling_down: 20,
            delegation_staked_amount: 10, // 1. after slash goes to 0 (-10)
            delegation_enqueued_for_withdraw_amount: 5,
            delegation_enqueued_for_cooldown_amount: 15, // 2. after slash goes to 0 (-15)
            delegation_cooling_down_for_withdraw_amount: 5,
            delegation_cooling_down_amount: 15, // after slash goes to 0 (-15)
            slash_amount: 40,
        };
        let (mut vault, mut vault_operator_delegation) =
            create_default_vault_and_vault_operator_delegation(&config);
        let mut vault_ncn_slasher_opr_ticket = create_default_vault_ncn_slasher_operator_ticket(0);

        let result = slash_and_update_vault(
            &mut vault,
            &mut vault_operator_delegation,
            &mut vault_ncn_slasher_opr_ticket,
            config.slash_amount,
        );

        assert!(result.is_ok());
        assert_eq!(vault.tokens_deposited, 60);
        assert_eq!(vault.amount_delegated, 60);
        assert_eq!(vault.amount_enqueued_for_cooldown, 5);
        assert_eq!(vault.amount_cooling_down, 5);
        assert_eq!(vault_operator_delegation.staked_amount, 0);
        assert_eq!(vault_operator_delegation.enqueued_for_withdraw_amount, 5);
        assert_eq!(vault_operator_delegation.enqueued_for_cooldown_amount, 0);
        assert_eq!(
            vault_operator_delegation.cooling_down_for_withdraw_amount,
            5
        );
        assert_eq!(vault_operator_delegation.cooling_down_amount, 0);
        assert_eq!(vault_ncn_slasher_opr_ticket.slashed, 40);
    }

    #[test]
    fn test_slash_zero_amount() {
        let config = TestConfig {
            vault_tokens_deposited: 100,
            vault_amount_delegated: 100,
            vault_amount_enqueued_for_cooldown: 0,
            vault_amount_cooling_down: 0,
            delegation_staked_amount: 10,
            delegation_enqueued_for_withdraw_amount: 0,
            delegation_enqueued_for_cooldown_amount: 0,
            delegation_cooling_down_for_withdraw_amount: 0,
            delegation_cooling_down_amount: 0,
            slash_amount: 0,
        };
        let (mut vault, mut vault_operator_delegation) =
            create_default_vault_and_vault_operator_delegation(&config);
        let mut vault_ncn_slasher_opr_ticket = create_default_vault_ncn_slasher_operator_ticket(0);

        let result = slash_and_update_vault(
            &mut vault,
            &mut vault_operator_delegation,
            &mut vault_ncn_slasher_opr_ticket,
            config.slash_amount,
        );

        assert!(result.is_ok());
        assert_eq!(vault.tokens_deposited, 100);
        assert_eq!(vault.amount_delegated, 100);
        assert_eq!(vault_operator_delegation.staked_amount, 10);
        assert_eq!(vault_ncn_slasher_opr_ticket.slashed, 0);
    }

    #[test]
    fn test_slash_with_previous_slashes() {
        let config = TestConfig {
            vault_tokens_deposited: 100,
            vault_amount_delegated: 100,
            vault_amount_enqueued_for_cooldown: 0,
            vault_amount_cooling_down: 0,
            delegation_staked_amount: 10,
            delegation_enqueued_for_withdraw_amount: 0,
            delegation_enqueued_for_cooldown_amount: 0,
            delegation_cooling_down_for_withdraw_amount: 0,
            delegation_cooling_down_amount: 0,
            slash_amount: 5,
        };
        let (mut vault, mut vault_operator_delegation) =
            create_default_vault_and_vault_operator_delegation(&config);
        let mut vault_ncn_slasher_opr_ticket = create_default_vault_ncn_slasher_operator_ticket(3);

        let result = slash_and_update_vault(
            &mut vault,
            &mut vault_operator_delegation,
            &mut vault_ncn_slasher_opr_ticket,
            config.slash_amount,
        );

        assert!(result.is_ok());
        assert_eq!(vault.tokens_deposited, 95);
        assert_eq!(vault.amount_delegated, 95);
        assert_eq!(vault_operator_delegation.staked_amount, 5);
        assert_eq!(vault_ncn_slasher_opr_ticket.slashed, 8);
    }

    #[test]
    fn test_slash_max_u64_value() {
        let config = TestConfig {
            vault_tokens_deposited: u64::MAX,
            vault_amount_delegated: u64::MAX,
            vault_amount_enqueued_for_cooldown: 0,
            vault_amount_cooling_down: 0,
            delegation_staked_amount: u64::MAX,
            delegation_enqueued_for_withdraw_amount: 0,
            delegation_enqueued_for_cooldown_amount: 0,
            delegation_cooling_down_for_withdraw_amount: 0,
            delegation_cooling_down_amount: 0,
            slash_amount: u64::MAX,
        };
        let (mut vault, mut vault_operator_delegation) =
            create_default_vault_and_vault_operator_delegation(&config);
        let mut vault_ncn_slasher_opr_ticket = create_default_vault_ncn_slasher_operator_ticket(0);

        let result = slash_and_update_vault(
            &mut vault,
            &mut vault_operator_delegation,
            &mut vault_ncn_slasher_opr_ticket,
            config.slash_amount,
        );

        assert!(result.is_ok());
        assert_eq!(vault.tokens_deposited, 0);
        assert_eq!(vault.amount_delegated, 0);
        assert_eq!(vault_operator_delegation.staked_amount, 0);
        assert_eq!(vault_ncn_slasher_opr_ticket.slashed, u64::MAX);
    }
}
