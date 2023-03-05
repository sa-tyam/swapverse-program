use std::cmp::min;
use std::mem::size_of;

use crate::constants::*;
use crate::error::SwapverseError;
use crate::spl_token_utils::{signed_transfer_tokens};
use crate::states::{SwapPool, GlobalState, InvestorPoolInfo};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;

#[derive(Accounts)]
pub struct ClaimProfit<'info> {
    #[account(mut)]
    pub investor: Signer<'info>,

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
        mut,
        seeds = [swap_pool.pool_number.to_le_bytes().as_ref(), SWAP_POOL_SEED.as_bytes()],
        bump,
    )]
    pub swap_pool: Box<Account<'info, SwapPool>>,

    #[account(
        constraint = (withdraw_token_mint.key() == swap_pool.token_a_mint) 
            || (withdraw_token_mint.key() == swap_pool.token_b_mint) @ SwapverseError::InvalidPoolTokenMint
    )]
    pub withdraw_token_mint: Box<Account<'info, Mint>>,

    #[account(
        constraint = token_a_mint.key() == swap_pool.token_a_mint @ SwapverseError::InvalidPoolTokenMint
    )]
    pub token_a_mint: Box<Account<'info, Mint>>,

    #[account(
        constraint = token_b_mint.key() == swap_pool.token_b_mint @ SwapverseError::InvalidPoolTokenMint,
        constraint = token_b_mint.key() != token_a_mint.key() @ SwapverseError::SameTokenMints
    )]
    pub token_b_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = investor,
        seeds = [swap_pool.key().as_ref(), token_a_mint.key().as_ref(), SWAP_POOL_TREASURY_ACCOUNT_SEED.as_bytes()],
        bump,
        token::mint = token_a_mint,
        token::authority = signing_authority,
    )]
    pub swap_pool_treasury_token_a_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = investor,
        seeds = [swap_pool.key().as_ref(), token_b_mint.key().as_ref(), SWAP_POOL_TREASURY_ACCOUNT_SEED.as_bytes()],
        bump,
        token::mint = token_b_mint,
        token::authority = signing_authority,
    )]
    pub swap_pool_treasury_token_b_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = investor_token_account.owner == investor.key() @ SwapverseError::InvalidInvestorTokenAccountOwner,
        constraint = investor_token_account.mint == withdraw_token_mint.key() @ SwapverseError::InvalidInvestorTokenAccountMint,
    )]
    pub investor_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = (pool_share_token_mint.key() == swap_pool.pool_share_token_a_mint && withdraw_token_mint.key() == swap_pool.token_a_mint) 
            || (pool_share_token_mint.key() == swap_pool.pool_share_token_b_mint && withdraw_token_mint.key() == swap_pool.token_b_mint) 
            @ SwapverseError::InvalidPoolShareTokenMint
    )]
    pub pool_share_token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = pool_share_token_mint,
        associated_token::authority = investor
    )]
    pub investor_pool_share_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = investor,
        seeds = [swap_pool.key().as_ref(), investor.key().as_ref()],
        bump,
        space = size_of::<InvestorPoolInfo>() + 8,
    )]
    pub investor_pool_info: Account<'info, InvestorPoolInfo>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> ClaimProfit<'info> {
    pub fn claim_profit(&mut self, amount: u64) -> Result<()> {

        require!(self.swap_pool.activated_at != i64::MAX, SwapverseError::SwapPoolNotActivated);

        let is_token_a = self.withdraw_token_mint.key() == self.swap_pool.token_a_mint;

        let investor_pool_share_amount = self.investor_pool_share_token_account.amount;
        let pool_distribution_token_amount = if is_token_a {
            self.swap_pool.profit_of_token_a_amount_to_be_distributed
        } else {
            self.swap_pool.profit_of_token_b_amount_to_be_distributed
        };
        let initial_token_amount = if is_token_a {
            self.swap_pool.initial_amount_a
        } else {
            self.swap_pool.initial_amount_b
        };
        
        let pool_distribution_token_amount_u128 = pool_distribution_token_amount as u128;
        let swapverse_share = pool_distribution_token_amount_u128.checked_mul(self.swap_pool.swapverse_fee_percentage as u128)
                                        .unwrap().checked_div(100).unwrap();

        let all_investors_share_u128 = pool_distribution_token_amount_u128.checked_sub(swapverse_share).unwrap();

        
        let investor_share = all_investors_share_u128.checked_mul(investor_pool_share_amount as u128).unwrap()
                                        .checked_div(initial_token_amount as u128).unwrap() as u64;

        let remaining_withdrawable_amount = if is_token_a {
            investor_share.checked_sub(self.investor_pool_info.profit_for_token_a_withdrawn).unwrap()
        } else {
            investor_share.checked_sub(self.investor_pool_info.profit_for_token_b_withdrawn).unwrap()
        };

        let withdraw_amount = min(amount, remaining_withdrawable_amount);

        require!(withdraw_amount > 0, SwapverseError::WithdrawAmountIsZero);

        let from = if is_token_a {
            &mut self.swap_pool_treasury_token_a_account
        } else {
            &mut self.swap_pool_treasury_token_b_account
        };

        if is_token_a {
            self.investor_pool_info.profit_for_token_a_withdrawn = self.investor_pool_info.profit_for_token_a_withdrawn
                                                                        .checked_add(withdraw_amount).unwrap();
        } else {
            self.investor_pool_info.profit_for_token_b_withdrawn = self.investor_pool_info.profit_for_token_b_withdrawn
                                                                        .checked_add(withdraw_amount).unwrap();
        }

        signed_transfer_tokens(
            withdraw_amount,
            from,
            &mut self.investor_token_account,
            &self.signing_authority,
            &self.token_program,
            &self.global_state
        )?;

        Ok(())
    }
}
