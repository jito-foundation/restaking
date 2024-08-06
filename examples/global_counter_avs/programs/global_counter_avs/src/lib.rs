use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;

declare_id!("6Qs48uxeHV4ZaJeQsGczXNj2kbJSYFVyXs34QvXYPN5E");

pub const COUNTER_SEED: &[u8] = b"COUNTER";
pub const REWARD_SEED: &[u8] = b"REWARD";

#[program]
pub mod global_counter_avs {
    use super::*;

    pub fn count(ctx: Context<Count>) -> ProgramResult {
        let global_counter = &mut ctx.accounts.global_counter;
        let user_rewards = &mut ctx.accounts.user_rewards;

        global_counter.count = global_counter.count.saturating_add(1);

        user_rewards.count = user_rewards.count.saturating_add(global_counter.count);

        Ok(())
    }
}

// -------------------- IXs ---------------------------
#[derive(Accounts)]
pub struct Count<'info> {
    #[account(
        init_if_needed,
        seeds = [COUNTER_SEED],
        bump,
        payer = user,
        space = std::mem::size_of::<GlobalCounter>() + 8,
    )]
    pub global_counter: Account<'info, GlobalCounter>,

    #[account(
        init_if_needed,
        seeds = [REWARD_SEED, user.key.as_ref()],
        bump,
        payer = user,
        space = std::mem::size_of::<UserRewards>() + 8,
    )]
    pub user_rewards: Account<'info, UserRewards>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

// ------------------ ACCOUNTS ------------------------
#[account]
#[derive(Default)]
pub struct GlobalCounter {
    pub count: u64,
}

#[account]
#[derive(Default)]
pub struct UserRewards {
    pub count: u64,
}

// ------------------ HELPERS ------------------------

pub fn derive_global_counter_address(program_id: &Pubkey) -> Pubkey {
    let (global_counter, _) = Pubkey::find_program_address(&[COUNTER_SEED], &program_id);

    global_counter
}

pub fn derive_user_rewards_address(program_id: &Pubkey, user: &Pubkey) -> Pubkey {
    let (user_rewards, _) =
        Pubkey::find_program_address(&[REWARD_SEED, user.as_ref()], &program_id);

    user_rewards
}
