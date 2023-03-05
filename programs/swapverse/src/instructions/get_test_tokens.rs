use crate::constants::*;
use crate::error::SwapverseError;
use crate::spl_token_utils::{mint_tokens};
use crate::states::GlobalState;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct GetTestTokens<'info> {
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
        constraint = is_mint_valid(global_state.as_ref(), token_mint.key()) @ SwapverseError::InvalidTokenMint
    )]
    pub token_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = investor,
        associated_token::mint = token_mint,
        associated_token::authority = investor
    )]
    pub investor_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
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

impl<'info> GetTestTokens<'info> {
    pub fn get_test_tokens(&mut self, amount: u64) -> Result<()> {
        require!(
            amount < 10_000_000,
            SwapverseError::TokenAmountLimitExceeded,
        );

        mint_tokens(
            amount,
            &self.token_program,
            &mut self.investor_token_account,
            &self.signing_authority,
            &mut self.token_mint,
            &self.global_state,
        )?;

        Ok(())
    }
}
