use std::cmp::min;

use crate::constants::*;
use crate::error::SwapverseError;
use crate::spl_token_utils::{transfer_tokens, mint_frozen_tokens};
use crate::states::{SwapPool, GlobalState};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct InvestSwapPool<'info> {
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
        constraint = swap_pool.open_for_investment == true @ SwapverseError::SwapPoolNotOpenForInvestment,
        constraint = amount >= swap_pool.min_investment_amount @ SwapverseError::InsufficientAmount
    )]
    pub swap_pool: Box<Account<'info, SwapPool>>,

    #[account(
        constraint = (token_mint.key() == swap_pool.token_a_mint) || (token_mint.key() == swap_pool.token_b_mint) @ SwapverseError::InvalidPoolTokenMint
    )]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = investor,
        seeds = [swap_pool.key().as_ref(), token_mint.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = signing_authority,
    )]
    pub swap_pool_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = investor_token_account.owner == investor.key() @ SwapverseError::InvalidInvestorTokenAccountOwner,
        constraint = investor_token_account.mint == token_mint.key() @ SwapverseError::InvalidInvestorTokenAccountMint,
    )]
    pub investor_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = (pool_share_token_mint.key() == swap_pool.pool_share_token_a_mint && token_mint.key() == swap_pool.token_a_mint) 
            || (pool_share_token_mint.key() == swap_pool.pool_share_token_b_mint && token_mint.key() == swap_pool.token_b_mint) 
            @ SwapverseError::InvalidPoolShareTokenMint
    )]
    pub pool_share_token_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = investor,
        associated_token::mint = pool_share_token_mint,
        associated_token::authority = investor
    )]
    pub investor_pool_share_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> InvestSwapPool<'info> {
    pub fn invest_swap_pool(&mut self, amount: u64) -> Result<()> {
        let pool_token_amount = self.swap_pool_token_account.amount;
        let is_token_a = self.token_mint.key() == self.swap_pool.token_a_mint;
        let pool_token_initial_amount = if is_token_a {
            self.swap_pool.initial_amount_a
        } else {
            self.swap_pool.initial_amount_b
        };
        let amount_remaining = pool_token_initial_amount
            .checked_sub(pool_token_amount)
            .unwrap();

        if (self.swap_pool.token_a_amount_to_be_distributed == self.swap_pool.initial_amount_a) 
            && (self.swap_pool.token_b_amount_to_be_distributed == self.swap_pool.initial_amount_b) {
                self.swap_pool.open_for_investment = false;
                self.swap_pool.active_for_swap = true;
                self.swap_pool.activated_at =  Clock::get()?.unix_timestamp;
        }
        require!(self.swap_pool.open_for_investment, SwapverseError::SwapPoolNotOpenForInvestment);

        let deposit_amount = min(amount, amount_remaining);
        require!(deposit_amount > 0, SwapverseError::DepositAmountIsZero);

        if is_token_a {
            self.swap_pool.token_a_amount_to_be_distributed = self.swap_pool.token_a_amount_to_be_distributed.checked_add(deposit_amount).unwrap();
        } else {
            self.swap_pool.token_b_amount_to_be_distributed = self.swap_pool.token_b_amount_to_be_distributed.checked_add(deposit_amount).unwrap();
        }

        transfer_tokens(
            deposit_amount,
            &mut self.investor_token_account,
            &mut self.swap_pool_token_account,
            &self.investor,
            &self.token_program,
        )?;

        mint_frozen_tokens(
            deposit_amount,
            &self.token_program,
            &mut self.investor_pool_share_token_account,
            &self.signing_authority,
            &mut self.pool_share_token_mint,
            &self.global_state,
        )?;

        if (self.swap_pool.token_a_amount_to_be_distributed == self.swap_pool.initial_amount_a) 
            && (self.swap_pool.token_b_amount_to_be_distributed == self.swap_pool.initial_amount_b) {
                self.swap_pool.open_for_investment = false;
                self.swap_pool.active_for_swap = true;
                self.swap_pool.activated_at =  Clock::get()?.unix_timestamp;
        }

        Ok(())
    }
}
