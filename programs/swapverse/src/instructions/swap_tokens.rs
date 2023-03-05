use crate::constants::*;
use crate::error::SwapverseError;
use crate::spl_token_utils::{signed_transfer_tokens, transfer_tokens};
use crate::states::{GlobalState, SwapPool};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct SwapToken<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

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
        constraint = user_token_a_account.owner == user.key() @ SwapverseError::InvalidInvestorTokenAccountOwner,
        constraint = user_token_a_account.mint == token_a_mint.key() @ SwapverseError::InvalidInvestorTokenAccountMint,
    )]
    pub user_token_a_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = user_token_b_account.owner == user.key() @ SwapverseError::InvalidInvestorTokenAccountOwner,
        constraint = user_token_b_account.mint == token_b_mint.key() @ SwapverseError::InvalidInvestorTokenAccountMint,
    )]
    pub user_token_b_account: Box<Account<'info, TokenAccount>>,

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
        init_if_needed,
        payer = user,
        seeds = [swap_pool.key().as_ref(), token_a_mint.key().as_ref(), SWAP_POOL_TREASURY_ACCOUNT_SEED.as_bytes()],
        bump,
        token::mint = token_a_mint,
        token::authority = signing_authority,
    )]
    pub swap_pool_treasury_token_a_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [swap_pool.key().as_ref(), token_b_mint.key().as_ref(), SWAP_POOL_TREASURY_ACCOUNT_SEED.as_bytes()],
        bump,
        token::mint = token_b_mint,
        token::authority = signing_authority,
    )]
    pub swap_pool_treasury_token_b_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> SwapToken<'info> {
    pub fn swap_token(
        &mut self,
        amount: u64,
        min_amount_out: u64,
        is_token_in_token_a: bool,
    ) -> Result<()> {
        let initial_amount_a_u128 = self.swap_pool.initial_amount_a as u128;
        let initial_amount_b_u128 = self.swap_pool.initial_amount_b as u128;

        let invariant = initial_amount_a_u128
            .checked_mul(initial_amount_b_u128)
            .unwrap();

        if is_token_in_token_a {
            require!(
                self.user_token_a_account.amount >= amount,
                SwapverseError::NotEnoughTokens
            );

            let effective_token_a_amount_u128 = self
                .swap_pool_token_a_account
                .amount
                .checked_add(amount)
                .unwrap() as u128;
            let effective_token_b_amount = invariant
                .checked_div(effective_token_a_amount_u128)
                .unwrap() as u64;
            let output_amount_u128 = self
                .swap_pool_token_b_account
                .amount
                .checked_sub(effective_token_b_amount)
                .unwrap() as u128;

            let treasury_share_u128 = output_amount_u128
                .checked_mul(self.swap_pool.swap_fee_percentage as u128)
                .unwrap()
                .checked_div(100)
                .unwrap();
            let user_share_u128 = output_amount_u128.checked_sub(treasury_share_u128).unwrap();

            require!(
                user_share_u128 as u64 >= min_amount_out,
                SwapverseError::NotEnoughOutput
            );

            self.swap_pool.profit_of_token_a_amount_to_be_distributed = self
                .swap_pool
                .profit_of_token_a_amount_to_be_distributed
                .checked_add(treasury_share_u128 as u64)
                .unwrap();

            signed_transfer_tokens(
                user_share_u128 as u64,
                &mut self.swap_pool_token_b_account,
                &mut self.user_token_b_account,
                &self.signing_authority,
                &self.token_program,
                &self.global_state,
            )?;

            signed_transfer_tokens(
                treasury_share_u128 as u64,
                &mut self.swap_pool_token_b_account,
                &mut self.swap_pool_treasury_token_b_account,
                &self.signing_authority,
                &self.token_program,
                &self.global_state,
            )?;

            transfer_tokens(
                amount,
                &mut self.user_token_a_account,
                &mut self.swap_pool_token_a_account,
                &self.user,
                &self.token_program,
            )?;
        } else {
            require!(
                self.user_token_b_account.amount >= amount,
                SwapverseError::NotEnoughTokens
            );

            let effective_token_b_amount_u128 = self
                .swap_pool_token_b_account
                .amount
                .checked_add(amount)
                .unwrap() as u128;
            let effective_token_a_amount = invariant
                .checked_div(effective_token_b_amount_u128)
                .unwrap() as u64;
            let output_amount_u128 = self
                .swap_pool_token_a_account
                .amount
                .checked_sub(effective_token_a_amount)
                .unwrap() as u128;

            let treasury_share_u128 = output_amount_u128
                .checked_mul(self.swap_pool.swap_fee_percentage as u128)
                .unwrap()
                .checked_div(100)
                .unwrap();
            let user_share_u128 = output_amount_u128.checked_sub(treasury_share_u128).unwrap();

            require!(
                user_share_u128 as u64 >= min_amount_out,
                SwapverseError::NotEnoughOutput
            );

            self.swap_pool.profit_of_token_b_amount_to_be_distributed = self
                .swap_pool
                .profit_of_token_b_amount_to_be_distributed
                .checked_add(treasury_share_u128 as u64)
                .unwrap();

            signed_transfer_tokens(
                user_share_u128 as u64,
                &mut self.swap_pool_token_a_account,
                &mut self.user_token_a_account,
                &self.signing_authority,
                &self.token_program,
                &self.global_state,
            )?;

            signed_transfer_tokens(
                treasury_share_u128 as u64,
                &mut self.swap_pool_token_a_account,
                &mut self.swap_pool_treasury_token_a_account,
                &self.signing_authority,
                &self.token_program,
                &self.global_state,
            )?;

            transfer_tokens(
                amount,
                &mut self.user_token_b_account,
                &mut self.swap_pool_token_b_account,
                &self.user,
                &self.token_program,
            )?;
        }

        Ok(())
    }
}
