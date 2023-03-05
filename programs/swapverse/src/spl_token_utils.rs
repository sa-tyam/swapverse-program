use crate::constants::SIGNING_AUTHORITY_SEED;
use crate::error::SwapverseError;
use crate::states::GlobalState;
use anchor_lang::prelude::*;
use anchor_spl::token::{
    burn, freeze_account, mint_to, thaw_account, transfer, Burn, FreezeAccount, Mint, MintTo,
    Token, Transfer,
};
use anchor_spl::token::{ThawAccount, TokenAccount};

pub fn signed_transfer_tokens<'a>(
    amount: u64,
    from: &mut Account<'a, TokenAccount>,
    to: &mut Account<'a, TokenAccount>,
    authority: &AccountInfo<'a>,
    token_program: &Program<'a, Token>,
    global_state: &Account<'a, GlobalState>,
) -> Result<()> {
    let seeds = &[
        SIGNING_AUTHORITY_SEED.as_bytes(),
        &[global_state.signing_authority_bump],
    ];
    _transfer_tokens(amount, from, to, authority, token_program, Some(&[seeds]))
}

pub fn transfer_tokens<'a>(
    amount: u64,
    from: &mut Account<'a, TokenAccount>,
    to: &mut Account<'a, TokenAccount>,
    authority: &AccountInfo<'a>,
    token_program: &Program<'a, Token>,
) -> Result<()> {
    _transfer_tokens(amount, from, to, authority, token_program, None)
}

fn _transfer_tokens<'a>(
    amount: u64,
    from: &mut Account<'a, TokenAccount>,
    to: &mut Account<'a, TokenAccount>,
    authority: &AccountInfo<'a>,
    token_program: &Program<'a, Token>,
    seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    let transfer_instruction = Transfer {
        from: from.to_account_info(),
        to: to.to_account_info(),
        authority: authority.to_account_info(),
    };
    let cpi_context = match seeds {
        Some(s) => {
            CpiContext::new_with_signer(token_program.to_account_info(), transfer_instruction, s)
        }
        None => CpiContext::new(token_program.to_account_info(), transfer_instruction),
    };
    transfer(cpi_context, amount)?;
    from.reload()?;
    to.reload()
}

pub fn burn_tokens<'a>(
    amount: u64,
    from: &mut Account<'a, TokenAccount>,
    mint: &mut Account<'a, Mint>,
    authority: &AccountInfo<'a>,
    token_program: &Program<'a, Token>,
    global_state: &Account<'a, GlobalState>,
    owner: &AccountInfo<'a>,
) -> Result<()> {
    thaw_token_account(from, mint, authority, token_program, global_state)?;

    let cpi_accounts = Burn {
        mint: mint.to_account_info(),
        from: from.to_account_info(),
        authority: owner.clone(),
    };

    let cpi_context = CpiContext::new(token_program.to_account_info(), cpi_accounts);

    if amount > from.amount {
        return err!(SwapverseError::NotEnoughTokens);
    }

    burn(cpi_context, amount)?;
    mint.reload()?;

    freeze_token_account(from, mint, authority, token_program, global_state)
}

pub fn mint_tokens<'a>(
    amount: u64,
    token_program: &Program<'a, Token>,
    to: &mut Account<'a, TokenAccount>,
    signing_authority: &AccountInfo<'a>,
    mint: &mut Account<'a, Mint>,
    global_state: &Account<'a, GlobalState>,
) -> Result<()> {
    let cpi_accounts = MintTo {
        mint: mint.to_account_info(),
        to: to.to_account_info(),
        authority: signing_authority.clone(),
    };

    let seeds = &[
        SIGNING_AUTHORITY_SEED.as_bytes(),
        &[global_state.signing_authority_bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let cpi_context =
        CpiContext::new_with_signer(token_program.to_account_info(), cpi_accounts, signer_seeds);

    mint_to(cpi_context, amount)?;
    mint.reload()?;

    Ok(())
}

pub fn mint_frozen_tokens<'a>(
    amount: u64,
    token_program: &Program<'a, Token>,
    to: &mut Account<'a, TokenAccount>,
    signing_authority: &AccountInfo<'a>,
    mint: &mut Account<'a, Mint>,
    global_state: &Account<'a, GlobalState>,
) -> Result<()> {
    thaw_token_account(to, mint, signing_authority, token_program, global_state)?;

    let cpi_accounts = MintTo {
        mint: mint.to_account_info(),
        to: to.to_account_info(),
        authority: signing_authority.clone(),
    };

    let seeds = &[
        SIGNING_AUTHORITY_SEED.as_bytes(),
        &[global_state.signing_authority_bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let cpi_context =
        CpiContext::new_with_signer(token_program.to_account_info(), cpi_accounts, signer_seeds);

    mint_to(cpi_context, amount)?;
    mint.reload()?;

    freeze_token_account(to, mint, signing_authority, token_program, global_state)
}

pub fn thaw_token_account<'a>(
    token_account: &mut Account<'a, TokenAccount>,
    mint: &Account<'a, Mint>,
    signing_authority: &AccountInfo<'a>,
    token_program: &Program<'a, Token>,
    global_state: &Account<'a, GlobalState>,
) -> Result<()> {
    if !token_account.is_frozen() {
        return Ok(());
    }

    let thaw_accounts = ThawAccount {
        mint: mint.to_account_info(),
        account: token_account.to_account_info(),
        authority: signing_authority.clone(),
    };

    let seeds = &[
        SIGNING_AUTHORITY_SEED.as_bytes(),
        &[global_state.signing_authority_bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let cpi_context =
        CpiContext::new_with_signer(token_program.to_account_info(), thaw_accounts, signer_seeds);

    thaw_account(cpi_context)?;
    token_account.reload()
}

pub fn freeze_token_account<'a>(
    token_account: &mut Account<'a, TokenAccount>,
    mint: &Account<'a, Mint>,
    signing_authority: &AccountInfo<'a>,
    token_program: &Program<'a, Token>,
    global_state: &Account<'a, GlobalState>,
) -> Result<()> {
    if token_account.is_frozen() {
        return Ok(());
    }

    let freeze_accounts = FreezeAccount {
        account: token_account.to_account_info(),
        mint: mint.to_account_info(),
        authority: signing_authority.clone(),
    };

    let seeds = &[
        SIGNING_AUTHORITY_SEED.as_bytes(),
        &[global_state.signing_authority_bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let cpi_context = CpiContext::new_with_signer(
        token_program.to_account_info(),
        freeze_accounts,
        signer_seeds,
    );

    freeze_account(cpi_context)?;
    token_account.reload()
}

pub fn signed_transfer_frozen_tokens<'a>(
    amount: u64,
    from: &mut Account<'a, TokenAccount>,
    to: &mut Account<'a, TokenAccount>,
    mint: &mut Account<'a, Mint>,
    signing_authority: &AccountInfo<'a>,
    token_program: &Program<'a, Token>,
    global_state: &Account<'a, GlobalState>,
) -> Result<()> {
    thaw_token_account(from, mint, signing_authority, token_program, global_state)?;

    thaw_token_account(to, mint, signing_authority, token_program, global_state)?;

    let seeds = &[
        SIGNING_AUTHORITY_SEED.as_bytes(),
        &[global_state.signing_authority_bump],
    ];

    _transfer_tokens(
        amount,
        from,
        to,
        signing_authority,
        token_program,
        Some(&[seeds]),
    )?;

    freeze_token_account(from, mint, signing_authority, token_program, global_state)?;

    freeze_token_account(to, mint, signing_authority, token_program, global_state)
}
