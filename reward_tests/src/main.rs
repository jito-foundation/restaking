use jito_vault_core::{delegation_state::DelegationState, vault::Vault};
use jito_vault_sdk::error::VaultError;
use solana_sdk::pubkey::Pubkey;

pub fn main() {
    let mut run_count = 0;

    // We care about the ratio
    // reward size
    // fee

    let reward_fee_bps = [1000];

    let starting_balances = [1000];

    let ratios = [1.0];

    let max_delta_bps: u16 = 100;

    for reward_fee_bps in reward_fee_bps {
        println!(
            "\n\n\n\n\n\n - Reward Fee {:.2}%",
            reward_fee_bps as f64 / 100.0
        );

        for starting_balance in starting_balances {
            println!("\n\n\n\n\n\n --- Starting Balance x{:.2}", starting_balance);

            for ratio in ratios {
                println!("\n\n\n\n\n\n ----- Ratio x{:.2}", ratio);
                for st_rewards in 0..1_000_000 {
                    let vrt_balance = starting_balance as u64;
                    let st_balance = (starting_balance as f64 * ratio).round() as u64;

                    let result = run_test(
                        st_balance,
                        vrt_balance,
                        st_rewards,
                        reward_fee_bps,
                        max_delta_bps,
                    );

                    // match result {
                    //     Ok(_) => {
                    //         println!(
                    //             "✅ rue3br4n6 78jjn: {} [{} {} {} {}]",
                    //             run_count, st_balance, vrt_balance, st_rewards, reward_fee_bps
                    //         );
                    //     }
                    //     Err(_) => {
                    //         println!(
                    //             "❌ run: {} [{} {} {} {}]",
                    //             run_count, st_balance, vrt_balance, st_rewards, reward_fee_bps
                    //         );
                    //     }
                    // }

                    run_count += 1;
                }
            }
        }
    }
}

pub fn run_test(
    st_balance: u64,
    vrt_balance: u64,
    st_rewards: u64,
    reward_fee_bps: u16,
    max_delta_bps: u16,
) -> Result<(), VaultError> {
    let new_st_balance = st_balance + st_rewards;

    let vault = make_test_vault(
        0,
        0,
        reward_fee_bps,
        st_balance,
        vrt_balance,
        DelegationState::default(),
    );

    let reward_fee_in_vrt = vault.calculate_rewards_fee_in_vrt(new_st_balance).unwrap();

    vault.check_reward_fee_effective_rate(new_st_balance, reward_fee_in_vrt, max_delta_bps)
}

pub fn make_test_vault(
    deposit_fee_bps: u16,
    withdraw_fee_bps: u16,
    reward_fee_bps: u16,
    tokens_deposited: u64,
    vrt_supply: u64,
    delegation_state: DelegationState,
) -> Vault {
    let mut vault = Vault::new(
        Pubkey::new_unique(),
        Pubkey::new_unique(),
        Pubkey::new_unique(),
        0,
        Pubkey::new_unique(),
        deposit_fee_bps,
        withdraw_fee_bps,
        reward_fee_bps,
        0,
        0,
    );

    vault.set_tokens_deposited(tokens_deposited);
    vault.set_vrt_supply(vrt_supply);
    vault.delegation_state = delegation_state;
    vault
}
