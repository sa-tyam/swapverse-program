use crate::constants::*;
use crate::states::GlobalState;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use std::mem::size_of;

#[derive(Accounts)]
pub struct InitializeGlobalState<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        seeds = [GLOBAL_STATE_SEED.as_bytes()],
        bump,
        space = size_of::<GlobalState>() + 8,
    )]
    pub global_state: Box<Account<'info, GlobalState>>,

    /// CHECK: we only read from this address
    #[account(
        seeds = [SIGNING_AUTHORITY_SEED.as_bytes()],
        bump
    )]
    pub signing_authority: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = owner,
        seeds = [USDC_DEV_TOKEN_SEED.as_bytes()],
        bump,
        mint::decimals = 6,
        mint::authority = signing_authority,
        mint::freeze_authority = signing_authority
    )]
    pub usdc_token_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = owner,
        seeds = [USDT_DEV_TOKEN_SEED.as_bytes()],
        bump,
        mint::decimals = 6,
        mint::authority = signing_authority,
        mint::freeze_authority = signing_authority
    )]
    pub usdt_token_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = owner,
        seeds = [UXD_DEV_TOKEN_SEED.as_bytes()],
        bump,
        mint::decimals = 6,
        mint::authority = signing_authority,
        mint::freeze_authority = signing_authority
    )]
    pub uxd_token_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = owner,
        seeds = [PAI_DEV_TOKEN_SEED.as_bytes()],
        bump,
        mint::decimals = 6,
        mint::authority = signing_authority,
        mint::freeze_authority = signing_authority
    )]
    pub pai_token_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = owner,
        seeds = [USDH_DEV_TOKEN_SEED.as_bytes()],
        bump,
        mint::decimals = 6,
        mint::authority = signing_authority,
        mint::freeze_authority = signing_authority
    )]
    pub usdh_token_mint: Box<Account<'info, Mint>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeGlobalState<'info> {
    pub fn initialize_global_state(&mut self, signing_authority_bump: &u8) -> Result<()> {
        let token_mints = [
            self.usdc_token_mint.key(),
            self.usdt_token_mint.key(),
            self.uxd_token_mint.key(),
            self.pai_token_mint.key(),
            self.usdh_token_mint.key(),
        ];
        self.global_state
            .initialize(token_mints, signing_authority_bump)
    }
}
