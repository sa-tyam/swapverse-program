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
pub struct WithdrawSwapPool<'info> {
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
        mut,
        seeds = [swap_pool.key().as_ref(), token_a_mint.key().as_ref()],
        bump,
        token::mint = token_a_mint,
        token::authority = signing_authority,
    )]
    pub swap_pool_token_a_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [swap_pool.key().as_ref(), token_b_mint.key().as_ref()],
        bump,
        token::mint = token_b_mint,
        token::authority = signing_authority,
    )]
    pub swap_pool_token_b_account: Box<Account<'info, TokenAccount>>,

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

impl<'info> WithdrawSwapPool<'info> {
    fn check_for_withdrawal_open(&mut self) {
        if !self.swap_pool.open_for_withdrawal {
            let time_now = Clock::get().unwrap().unix_timestamp;
            if !self.swap_pool.active_for_swap {
                let max_activation_time = self.swap_pool.created_at
                    .checked_add((self.swap_pool.max_days_to_fill as i64) * 24 * 60 * 60).unwrap();
                if time_now > max_activation_time {
                    self.swap_pool.open_for_investment = false;
                    self.swap_pool.open_for_withdrawal = true;
                    self.set_withdrawable_values();
                }
            } else {
                let max_pool_life = self.swap_pool.created_at
                    .checked_add((self.swap_pool.swap_life_in_days as i64) * 24 * 60 * 60).unwrap();
                if time_now > max_pool_life {
                    self.swap_pool.open_for_investment = false;
                    self.swap_pool.open_for_withdrawal = true;
                    self.set_withdrawable_values();
                }
            }
        }
    }

    fn set_withdrawable_values(&mut self) {
        self.swap_pool.token_a_amount_to_be_distributed = self.swap_pool_token_a_account.amount;
        self.swap_pool.token_b_amount_to_be_distributed = self.swap_pool_token_b_account.amount;
    }

    pub fn withdraw_swap_pool(&mut self, amount: u64) -> Result<()> {
        self.check_for_withdrawal_open();

        require!(self.swap_pool.open_for_withdrawal, SwapverseError::SwapPoolNotOpenForWithdrawal);

        let is_token_a = self.withdraw_token_mint.key() == self.swap_pool.token_a_mint;

        let investor_pool_share_amount = self.investor_pool_share_token_account.amount;
        let pool_distribution_token_amount = if is_token_a {
            self.swap_pool.token_a_amount_to_be_distributed
        } else {
            self.swap_pool.token_b_amount_to_be_distributed
        };
        let initial_token_amount = if self.swap_pool.activated_at != i64::MAX {
            if is_token_a {
                self.swap_pool.initial_amount_a
            } else {
                self.swap_pool.initial_amount_b
            }
        } else {
            pool_distribution_token_amount
        };

        let pool_distribution_token_amount_u128 = pool_distribution_token_amount as u128;
        let investor_share = pool_distribution_token_amount_u128.checked_mul(investor_pool_share_amount as u128).unwrap()
                                    .checked_div(initial_token_amount as u128).unwrap() as u64;

        let remaining_withdrawable_amount = if is_token_a {
            investor_share.checked_sub(self.investor_pool_info.token_a_withdrawn).unwrap()
        } else {
            investor_share.checked_sub(self.investor_pool_info.token_b_withdrawn).unwrap()
        };

        let withdraw_amount = min(amount, remaining_withdrawable_amount);

        let from = if is_token_a {
            &mut self.swap_pool_token_a_account
        } else {
            &mut self.swap_pool_token_b_account
        };

        if is_token_a {
            self.investor_pool_info.token_a_withdrawn = self.investor_pool_info.token_a_withdrawn.checked_add(withdraw_amount).unwrap();
        } else {
            self.investor_pool_info.token_b_withdrawn = self.investor_pool_info.token_b_withdrawn.checked_add(withdraw_amount).unwrap();
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
