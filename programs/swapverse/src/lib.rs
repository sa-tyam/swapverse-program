use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod spl_token_utils;
pub mod states;

use crate::instructions::*;

declare_id!("AeFLgMmKmVjLUv4jBGjXsrNf4MKPaVate5fNmqrDDoin");

#[program]
pub mod swapverse {
    use super::*;

    pub fn initialize_global_state(ctx: Context<InitializeGlobalState>) -> Result<()> {
        ctx.accounts
            .initialize_global_state(ctx.bumps.get("signing_authority").unwrap())
    }

    pub fn get_test_tokens(ctx: Context<GetTestTokens>, amount: u64) -> Result<()> {
        ctx.accounts.get_test_tokens(amount)
    }

    pub fn create_swap_pool(
        ctx: Context<CreateSwapPool>,
        initial_amount_a: u64,
        initial_amount_b: u64,
        swap_fee_percentage: u8,
        swapverse_fee_percentage: u8,
        min_investment_amount: u64,
        max_days_to_fill: u8,
        swap_life_in_days: u64,
    ) -> Result<()> {
        ctx.accounts.create_swap_pool(
            initial_amount_a,
            initial_amount_b,
            swap_fee_percentage,
            swapverse_fee_percentage,
            min_investment_amount,
            max_days_to_fill,
            swap_life_in_days,
        )
    }

    pub fn invest_swap_pool(ctx: Context<InvestSwapPool>, amount: u64) -> Result<()> {
        ctx.accounts.invest_swap_pool(amount)
    }

    pub fn withdraw_swap_pool(ctx: Context<WithdrawSwapPool>, is_token_a: bool) -> Result<()> {
        ctx.accounts.withdraw_swap_pool(is_token_a)
    }

    pub fn swap_token(
        ctx: Context<SwapToken>,
        amount: u64,
        min_amount_out: u64,
        is_token_in_token_a: bool,
    ) -> Result<()> {
        ctx.accounts
            .swap_token(amount, min_amount_out, is_token_in_token_a)
    }

    pub fn claim_profit(ctx: Context<ClaimProfit>) -> Result<()> {
        ctx.accounts.claim_profit()
    }
}
