use anchor_lang::prelude::*;
use anchor_lang::solana_program::entrypoint::ProgramResult;

declare_id!("6Qs48uxeHV4ZaJeQsGczXNj2kbJSYFVyXs34QvXYPN5E");

pub const COUNTER_SEED: &[u8] = b"COUNTER";
pub const OPERATOR_SEED: &[u8] = b"OPERATOR";
pub const SLOT_THRESHOLD_FOR_SLASHING: u64 = 10_000;

#[program]
pub mod global_counter_avs {
    use super::*;

    pub fn initialize_global_counter(ctx: Context<InitializeGlobalCounter>) -> ProgramResult {
        Ok(())
    }

    pub fn initialize_operator(ctx: Context<InitializeOperator>) -> ProgramResult {
        Ok(())
    }

    pub fn count(ctx: Context<Count>) -> ProgramResult {
        let global_counter = &mut ctx.accounts.global_counter;
        let operator = &mut ctx.accounts.operator;

        global_counter.count = global_counter.count.checked_add(1).unwrap();

        operator.rewards = operator.rewards.checked_add(global_counter.count).unwrap();

        operator.last_updated_slot = Clock::get()?.slot;

        Ok(())
    }
}

// -------------------- IXs ---------------------------
#[derive(Accounts)]
pub struct InitializeGlobalCounter<'info> {
    #[account(
        init,
        seeds = [COUNTER_SEED],
        bump,
        payer = payer,
        space = std::mem::size_of::<GlobalCounter>() + 8,
    )]
    pub global_counter: Account<'info, GlobalCounter>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeOperator<'info> {
    #[account(
        init,
        seeds = [OPERATOR_SEED, authority.key.as_ref()],
        bump,
        payer = authority,
        space = std::mem::size_of::<Operator>() + 8,
    )]
    pub operator: Account<'info, Operator>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Count<'info> {
    #[account(
        seeds = [COUNTER_SEED],
        bump,
    )]
    pub global_counter: Account<'info, GlobalCounter>,

    #[account(
        seeds = [OPERATOR_SEED, authority.key.as_ref()],
        bump,
    )]
    pub operator: Account<'info, Operator>,

    #[account(mut)]
    pub authority: Signer<'info>,

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
pub struct Operator {
    pub last_updated_slot: u64,
    pub rewards: u64,
}

// ------------------ HELPERS ------------------------

pub fn derive_global_counter_address(program_id: &Pubkey) -> Pubkey {
    let (global_counter, _) = Pubkey::find_program_address(&[COUNTER_SEED], &program_id);

    global_counter
}

pub fn derive_operator_address(program_id: &Pubkey, authority: &Pubkey) -> Pubkey {
    let (operator, _) =
        Pubkey::find_program_address(&[OPERATOR_SEED, authority.as_ref()], &program_id);

    operator
}
