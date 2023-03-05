use crate::constants::*;
use crate::error::SwapverseError;
use crate::states::{GlobalState, SwapPool};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use std::mem::size_of;

#[derive(Accounts)]
pub struct CreateSwapPool<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED.as_bytes()],
        bump,
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    /// CHECK: we only read from this address
    #[account(
        seeds = [SIGNING_AUTHORITY_SEED.as_bytes()],
        bump
    )]
    pub signing_authority: AccountInfo<'info>,

    #[account(
        init,
        payer = owner,
        seeds = [global_state.no_of_swap_pools.to_le_bytes().as_ref(), SWAP_POOL_SEED.as_bytes()],
        bump,
        space = size_of::<SwapPool>() + 8,
    )]
    pub swap_pool: Box<Account<'info, SwapPool>>,

    #[account(
        constraint = is_mint_valid(global_state.as_ref(), token_a_mint.key()) @ SwapverseError::InvalidTokenMint
    )]
    pub token_a_mint: Box<Account<'info, Mint>>,

    #[account(
        constraint = is_mint_valid(global_state.as_ref(), token_b_mint.key()) @ SwapverseError::InvalidTokenMint,
        constraint = token_b_mint.key() != token_a_mint.key() @ SwapverseError::SameTokenMints
    )]
    pub token_b_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = owner,
        seeds = [swap_pool.key().as_ref(), token_a_mint.key().as_ref(), SWAP_POOL_SHARE_TOKEN_SEED.as_bytes()],
        bump,
        mint::decimals = 6,
        mint::authority = signing_authority,
        mint::freeze_authority = signing_authority
    )]
    pub pool_share_token_a_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = owner,
        seeds = [swap_pool.key().as_ref(), token_b_mint.key().as_ref(), SWAP_POOL_SHARE_TOKEN_SEED.as_bytes()],
        bump,
        mint::decimals = 6,
        mint::authority = signing_authority,
        mint::freeze_authority = signing_authority
    )]
    pub pool_share_token_b_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

fn is_mint_valid(global_state: &GlobalState, mint_address: Pubkey) -> bool {
    for i in 0..5 {
        if global_state.token_mints[i] == mint_address {
            return true;
        }
    }
    false
}

impl<'info> CreateSwapPool<'info> {
    pub fn create_swap_pool(
        &mut self,
        initial_amount_a: u64,
        initial_amount_b: u64,
        swap_fee_percentage: u8,
        swapverse_fee_percentage: u8,
        min_investment_amount: u64,
        max_days_to_fill: u8,
        swap_life_in_days: u64,
    ) -> Result<()> {
        require!(
            initial_amount_a == initial_amount_b,
            SwapverseError::InitialAmountsNotSame
        );

        let pool_number = self.global_state.no_of_swap_pools;
        self.global_state.no_of_swap_pools =
            self.global_state.no_of_swap_pools.checked_add(1).unwrap();

        self.swap_pool.initialize(
            pool_number,
            self.token_a_mint.key(),
            self.token_b_mint.key(),
            self.pool_share_token_a_mint.key(),
            self.pool_share_token_b_mint.key(),
            initial_amount_a,
            initial_amount_b,
            swap_fee_percentage,
            swapverse_fee_percentage,
            min_investment_amount,
            max_days_to_fill,
            swap_life_in_days,
        )
    }
}
