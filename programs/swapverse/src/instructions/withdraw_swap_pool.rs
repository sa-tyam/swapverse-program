use crate::constants::*;
use crate::error::SwapverseError;
use crate::spl_token_utils::signed_transfer_tokens;
use crate::states::{GlobalState, InvestorPoolInfo, SwapPool};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

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
        constraint = investor_token_a_account.owner == investor.key() @ SwapverseError::InvalidInvestorTokenAccountOwner,
        constraint = investor_token_a_account.mint == token_a_mint.key() @ SwapverseError::InvalidInvestorTokenAccountMint,
    )]
    pub investor_token_a_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = investor_token_b_account.owner == investor.key() @ SwapverseError::InvalidInvestorTokenAccountOwner,
        constraint = investor_token_b_account.mint == token_b_mint.key() @ SwapverseError::InvalidInvestorTokenAccountMint,
    )]
    pub investor_token_b_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = pool_share_token_a_mint.key() == swap_pool.pool_share_token_a_mint @ SwapverseError::InvalidPoolShareTokenMint
    )]
    pub pool_share_token_a_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = pool_share_token_b_mint.key() == swap_pool.pool_share_token_b_mint @ SwapverseError::InvalidPoolShareTokenMint
    )]
    pub pool_share_token_b_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = investor,
        associated_token::mint = pool_share_token_a_mint,
        associated_token::authority = investor
    )]
    pub investor_pool_share_token_a_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = investor,
        associated_token::mint = pool_share_token_b_mint,
        associated_token::authority = investor
    )]
    pub investor_pool_share_token_b_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [swap_pool.key().as_ref(), investor.key().as_ref()],
        bump,
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
                let max_activation_time = self
                    .swap_pool
                    .created_at
                    .checked_add((self.swap_pool.max_days_to_fill as i64) * 24 * 60 * 60)
                    .unwrap();
                if time_now > max_activation_time {
                    self.swap_pool.open_for_investment = false;
                    self.swap_pool.open_for_withdrawal = true;
                    self.set_withdrawable_values();
                }
            } else {
                let max_pool_life = self
                    .swap_pool
                    .created_at
                    .checked_add((self.swap_pool.swap_life_in_days as i64) * 24 * 60 * 60)
                    .unwrap();
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

    pub fn withdraw_swap_pool(&mut self, is_token_a: bool) -> Result<()> {
        self.check_for_withdrawal_open();

        require!(
            self.swap_pool.open_for_withdrawal,
            SwapverseError::SwapPoolNotOpenForWithdrawal
        );

        let investor_pool_share_amount = if is_token_a {
            self.investor_pool_share_token_a_account.amount
        } else {
            self.investor_pool_share_token_b_account.amount
        };

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
        let withdraw_amount = pool_distribution_token_amount_u128
            .checked_mul(investor_pool_share_amount as u128)
            .unwrap()
            .checked_div(initial_token_amount as u128)
            .unwrap() as u64;

        let withdraw_a_amount = if is_token_a {
            withdraw_amount
        } else {
            investor_pool_share_amount
                .checked_sub(withdraw_amount)
                .unwrap()
        };

        let withdraw_b_amount = if is_token_a {
            investor_pool_share_amount
                .checked_sub(withdraw_amount)
                .unwrap()
        } else {
            withdraw_amount
        };

        signed_transfer_tokens(
            withdraw_a_amount,
            &mut self.swap_pool_token_a_account,
            &mut self.investor_token_a_account,
            &self.signing_authority,
            &self.token_program,
            &self.global_state,
        )?;

        signed_transfer_tokens(
            withdraw_b_amount,
            &mut self.swap_pool_token_b_account,
            &mut self.investor_token_b_account,
            &self.signing_authority,
            &self.token_program,
            &self.global_state,
        )?;

        Ok(())
    }
}
