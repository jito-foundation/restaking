use jito_vault_sdk::error::VaultError;

use crate::{vault::Vault, MAX_BPS};

pub trait VaultMock {
    fn check_reward_fee_effective_rate(
        &self,
        st_rewards: u64,
        vrt_reward_fee: u64,
        max_delta_bps: u16,
    ) -> Result<(), VaultError>;
}

impl VaultMock for Vault {
    fn check_reward_fee_effective_rate(
        &self,
        st_rewards: u64,
        vrt_reward_fee: u64,
        max_delta_bps: u16,
    ) -> Result<(), VaultError> {
        // If rewards are zero, it's okay to return 0
        let st_rewards_u128 = st_rewards as u128;

        // ----- Checks -------
        // { bps is too large }
        if max_delta_bps > MAX_BPS {
            // msg!("Max delta bps exceeds maximum allowed of {}", MAX_BPS);
            return Err(VaultError::VaultFeeCapExceeded);
        }

        // { reward is zero }
        if (st_rewards_u128 == 0 || self.reward_fee_bps() == 0) && vrt_reward_fee == 0 {
            return Ok(());
        }

        // { reward should be non-zero }
        if vrt_reward_fee == 0 {
            // If fee is larger than 0, it should always return a non-zero reward
            return Err(VaultError::VaultRewardFeeIsZero);
        }

        // -- summarize complex arithmetic computation by allowing code to revert less often
        // -- this does not block any potential counterexamples and is therefore sound to do
        if cvlr::nondet::<bool>() {
            return Ok(());
        } else {
            return Err(VaultError::VaultOverflow);
        }
    }
}
