use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;

declare_id!("6Qs48uxeHV4ZaJeQsGczXNj2kbJSYFVyXs34QvXYPN5E");

pub const COUNTER_SEED: &[u8] = b"global_counter";

#[program]
pub mod global_counter_avs {
    use super::*;

    pub fn count(ctx: Context<Count>) -> ProgramResult {
        let global_counter = &mut ctx.accounts.global_counter;
        global_counter.count = global_counter.count.checked_add(1).unwrap();

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

pub fn derive_global_counter_address(program_id: &Pubkey) -> Pubkey {
    let (global_counter, _) = Pubkey::find_program_address(&[COUNTER_SEED], &program_id);

    global_counter
}
