use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use std::mem::size_of;

declare_id!("3NQxMSYUnMzzWrANDx2MGEZpqj5Yu2oMsddhJp5HAigH");

#[program]
pub mod color_of_the_epoch {
    use anchor_lang::solana_program::clock::DEFAULT_SLOTS_PER_EPOCH;

    use super::*;

    pub fn initialize_color_of_the_epoch(
        ctx: Context<InitializeColorOfTheEpoch>,
        params: InitializeColorOfTheEpochParams,
    ) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        let color_of_the_epoch = &mut ctx.accounts.color_of_the_epoch;

        let slots_per_epoch = if params.slots_per_epoch == 0 {
            DEFAULT_SLOTS_PER_EPOCH
        } else {
            params.slots_per_epoch
        };

        color_of_the_epoch.authority = *ctx.accounts.authority.key;
        color_of_the_epoch.slots_per_epoch = slots_per_epoch;
        color_of_the_epoch.last_updated_epoch = color_of_the_epoch.get_epoch(current_slot);

        Ok(())
    }
}

// 5. Define the init token params
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitializeColorOfTheEpochParams {
    pub color_coin_decimals: u8,
    pub slots_per_epoch: u64,
}

#[derive(Accounts)]
#[instruction(
    params: InitializeColorOfTheEpochParams
)]
pub struct InitializeColorOfTheEpoch<'info> {
    #[account(
        init,
        payer = authority,
        space = ColorOfTheEpoch::SIZE,
        seeds = [ColorOfTheEpoch::SEED],
        bump
    )]
    pub color_of_the_epoch: Account<'info, ColorOfTheEpoch>,

    #[account(
        init,
        seeds = [ColorOfTheEpoch::MINT_SEED],
        bump,
        payer = authority,
        mint::decimals = params.color_coin_decimals,
        mint::authority = color_coin_mint,
    )]
    pub color_coin_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub votes: u8,
    pub cost: u32,
}

#[account]
pub struct ColorOfTheEpoch {
    pub authority: Pubkey,

    pub slots_per_epoch: u64,
    pub last_updated_epoch: u64,

    pub color_of_the_epoch: Color,
    pub actively_voting_colors: [Color; 3],
}

impl ColorOfTheEpoch {
    pub const SIZE: usize = 8 + size_of::<Self>();
    pub const SEED: &'static [u8] = b"COLOR";
    pub const MINT_SEED: &'static [u8] = b"COLOR_COIN";

    pub fn get_epoch(&self, current_slot: u64) -> u64 {
        current_slot.checked_div(self.slots_per_epoch).unwrap_or(0)
    }
}

pub fn derive_color_of_epoch_address(program_id: &Pubkey) -> Pubkey {
    let (address, _) = Pubkey::find_program_address(&[ColorOfTheEpoch::SEED], program_id);

    address
}

pub fn derive_color_coin_mint_address(program_id: &Pubkey) -> Pubkey {
    let (address, _) = Pubkey::find_program_address(&[ColorOfTheEpoch::MINT_SEED], program_id);
    address
}
