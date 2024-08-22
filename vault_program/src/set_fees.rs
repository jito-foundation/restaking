use jito_bytemuck::AccountDeserialize;
use jito_jsm_core::loader::load_signer;
use jito_vault_core::{config::Config, vault::Vault};
use jito_vault_sdk::error::VaultError;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

pub fn process_set_fees(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    deposit_fee_bps: Option<u16>,
    withdrawal_fee_bps: Option<u16>,
    reward_fee_bps: Option<u16>,
) -> ProgramResult {
    let [config, vault, vault_fee_admin] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Config::load(program_id, config, false)?;
    let mut config_data = config.data.borrow_mut();
    let config = Config::try_from_slice_unchecked_mut(&mut config_data)?;
    Vault::load(program_id, vault, false)?;
    let mut vault_data = vault.data.borrow_mut();
    let vault = Vault::try_from_slice_unchecked_mut(&mut vault_data)?;
    load_signer(vault_fee_admin, false)?;

    vault.check_fee_admin(vault_fee_admin.key)?;

    // Fees changes have a cooldown of 1 full epoch
    let current_slot = Clock::get()?.slot;
    check_fee_cooldown_ok(
        current_slot,
        vault.last_fee_change_slot(),
        config.epoch_length(),
    )?;

    if deposit_fee_bps.is_none() && withdrawal_fee_bps.is_none() && reward_fee_bps.is_none() {
        msg!("No fees provided for update");
        return Err(ProgramError::InvalidInstructionData);
    }

    if let Some(deposit_fee_bps) = deposit_fee_bps {
        check_fee_change_ok(
            vault.deposit_fee_bps(),
            deposit_fee_bps,
            config.fee_cap_bps(),
            config.fee_bump_bps(),
            config.fee_rate_of_change_bps(),
        )?;

        vault.set_deposit_fee_bps(deposit_fee_bps);
    }

    if let Some(withdrawal_fee_bps) = withdrawal_fee_bps {
        check_fee_change_ok(
            vault.withdrawal_fee_bps(),
            withdrawal_fee_bps,
            config.fee_cap_bps(),
            config.fee_bump_bps(),
            config.fee_rate_of_change_bps(),
        )?;

        vault.set_withdrawal_fee_bps(withdrawal_fee_bps);
    }

    if let Some(reward_fee_bps) = reward_fee_bps {
        if reward_fee_bps > Config::MAX_BPS {
            msg!("Epoch fee exceeds maximum allowed of {}", Config::MAX_BPS);
            return Err(VaultError::VaultFeeCapExceeded.into());
        }

        vault.set_reward_fee_bps(reward_fee_bps);
    }

    vault.set_last_fee_change_slot(current_slot);

    Ok(())
}

pub fn check_fee_cooldown_ok(
    current_slot: u64,
    last_fee_change_slot: u64,
    epoch_length: u64,
) -> ProgramResult {
    let current_epoch = current_slot.checked_div(epoch_length).unwrap();
    let last_fee_change_epoch = last_fee_change_slot.checked_div(epoch_length).unwrap();

    if current_epoch <= last_fee_change_epoch.checked_add(1).unwrap() {
        msg!("Fee changes are only allowed once per epoch");
        return Err(VaultError::VaultFeeChangeTooSoon.into());
    }

    Ok(())
}

pub fn check_fee_change_ok(
    current_fee_bps: u16,
    new_fee_bps: u16,
    fee_cap_bps: u16,
    fee_bump_bps: u16,
    fee_rate_of_change_bps: u16,
) -> ProgramResult {
    let fee_delta = new_fee_bps.saturating_sub(current_fee_bps);
    let fee_cap_bps = fee_cap_bps.min(Config::MAX_BPS);

    if new_fee_bps > fee_cap_bps {
        msg!("Fee exceeds maximum allowed of {}", fee_cap_bps);
        return Err(VaultError::VaultFeeCapExceeded.into());
    }

    if fee_delta > fee_bump_bps {
        let deposit_percentage_increase_bps: u64 = (fee_delta as u128)
            .checked_mul(10000)
            .and_then(|product| product.checked_div(current_fee_bps as u128))
            .map(|result| result as u64)
            .unwrap_or(u64::MAX); // Divide by zero should result in max value

        if deposit_percentage_increase_bps > fee_rate_of_change_bps as u64 {
            msg!(
                "Fee increase exceeds maximum rate of change {} bps or flat bump of {} bps",
                fee_rate_of_change_bps,
                fee_bump_bps
            );
            return Err(VaultError::VaultFeeBumpTooLarge.into());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_fee_change_after_two_epochs() {
        let current_slot = 1_000_000;
        let epoch_length = 100_000;
        let last_fee_change_slot = current_slot - (2 * epoch_length) - 1;

        assert!(check_fee_cooldown_ok(current_slot, last_fee_change_slot, epoch_length).is_ok());
    }

    #[test]
    fn test_fee_change_within_same_epoch() {
        let current_slot = 150_000;
        let epoch_length = 100_000;
        let last_fee_change_slot = 140_000;

        assert!(check_fee_cooldown_ok(current_slot, last_fee_change_slot, epoch_length).is_err());
    }

    #[test]
    fn test_fee_change_in_next_epoch() {
        let current_slot = 110_000;
        let epoch_length = 100_000;
        let last_fee_change_slot = 90_000;

        assert!(check_fee_cooldown_ok(current_slot, last_fee_change_slot, epoch_length).is_err());
    }

    #[test]
    fn test_fee_change_at_epoch_boundary() {
        let current_slot = 200_000;
        let epoch_length = 100_000;
        let last_fee_change_slot = 100_000;

        assert!(check_fee_cooldown_ok(current_slot, last_fee_change_slot, epoch_length).is_err());
    }

    #[test]
    fn test_fee_change_just_after_epoch_boundary() {
        let current_slot = 200_001;
        let epoch_length = 100_000;
        let last_fee_change_slot = 99_999;

        assert!(check_fee_cooldown_ok(current_slot, last_fee_change_slot, epoch_length).is_ok());
    }

    #[test]
    fn test_fee_change_with_large_slot_numbers() {
        let current_slot = 1_000_000_000;
        let epoch_length = 100_000_000;
        let last_fee_change_slot = 799_999_999;

        assert!(check_fee_cooldown_ok(current_slot, last_fee_change_slot, epoch_length).is_ok());
    }

    #[test]
    fn test_fee_increase_within_limits() {
        let current_fee_bps = 100;
        let new_fee_bps = 125;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        // OK: 25% increase <= 25% limit
        assert!(check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_ok());
    }

    #[test]
    fn test_fee_increase_outside_limits() {
        let current_fee_bps = 100;
        let new_fee_bps = 126;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        // ERROR: 26% increase > 25% limit
        assert!(check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_err());
    }

    #[test]
    fn test_fee_increase_inside_bump_limits() {
        let current_fee_bps = 1;
        let new_fee_bps = 10;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        // OK: Δ <= bump
        assert!(check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_ok());
    }

    #[test]
    fn test_fee_increase_outside_bump_limits() {
        let current_fee_bps = 1;
        let new_fee_bps = 13;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        // ERROR: Δ > bump
        assert!(check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_err());
    }

    #[test]
    fn test_zero_ok() {
        let current_fee_bps = 0;
        let new_fee_bps = 10;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        // OK: Δ <= bump
        assert!(check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_ok());
    }

    #[test]
    fn test_zero_bad() {
        let current_fee_bps = 0;
        let new_fee_bps = 11;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        // Error: Δ > bump
        assert!(check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_err());
    }

    #[test]
    fn test_no_difference() {
        let current_fee_bps = 100;
        let new_fee_bps = 100;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        // OK: Δ <= bump
        assert!(check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_ok());
    }

    #[test]
    fn test_decrease() {
        let current_fee_bps = 100;
        let new_fee_bps = 0;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        // OK: Δ <= bump
        assert!(check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_ok());
    }

    #[test]
    fn test_max_fee_values() {
        let max_fee_bps = Config::MAX_BPS;

        let current_fee_bps = max_fee_bps - 1;
        let new_fee_bps = max_fee_bps;
        let fee_cap_bps = max_fee_bps;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        assert!(check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_ok());
    }

    #[test]
    fn test_max_decrease() {
        let current_fee_bps = u16::MAX;
        let new_fee_bps = 0;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        assert!(check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_ok());
    }

    #[test]
    fn test_max_increase() {
        let current_fee_bps = 0;
        let new_fee_bps = u16::MAX;
        let fee_cap_bps = u16::MAX;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        assert!(check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_err());
    }

    #[test]
    fn test_at_cap() {
        let current_fee_bps = 2999;
        let new_fee_bps = 3000;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        assert!(check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_ok());
    }

    #[test]
    fn test_above_cap() {
        let current_fee_bps = 2999;
        let new_fee_bps = 3001;
        let fee_cap_bps = 3000;
        let fee_bump_bps = 10;
        let fee_rate_of_change_bps = 2500;

        assert!(check_fee_change_ok(
            current_fee_bps,
            new_fee_bps,
            fee_cap_bps,
            fee_bump_bps,
            fee_rate_of_change_bps
        )
        .is_err());
    }
}
