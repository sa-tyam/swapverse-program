use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct InvestorPoolInfo {
    pub token_a_withdrawn: u64,
    pub token_b_withdrawn: u64,
    pub profit_for_token_a_withdrawn: u64,
    pub profit_for_token_b_withdrawn: u64,
}
