use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct SwapPool {
    pub pool_number: u64,
    pub active_for_swap: bool,
    pub open_for_investment: bool,
    pub open_for_withdrawal: bool,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    // tokens to be given to investors for their deposits
    pub pool_share_token_a_mint: Pubkey,
    pub pool_share_token_b_mint: Pubkey,
    // initial amount of token a
    pub initial_amount_a: u64,
    // initial amount of token b
    pub initial_amount_b: u64,
    // percentage of swapped amount for to be levied as fee
    pub swap_fee_percentage: u8,
    // percentage of withdrawal_fee_percentage going to swapverse treasury
    pub swapverse_fee_percentage: u8,
    // minimum investment amount
    pub min_investment_amount: u64,
    // maximum days from creation time to fill the pool
    pub max_days_to_fill: u8,
    // life of swap in days
    pub swap_life_in_days: u64,
    pub created_at: i64,
    pub activated_at: i64,
    // used for withdrawals
    pub token_a_amount_to_be_distributed: u64,
    pub token_b_amount_to_be_distributed: u64,
    pub profit_of_token_a_amount_to_be_distributed: u64,
    pub profit_of_token_b_amount_to_be_distributed: u64,
}

impl SwapPool {
    pub fn initialize(
        &mut self,
        pool_number: u64,
        token_a_mint: Pubkey,
        token_b_mint: Pubkey,
        pool_share_token_a_mint: Pubkey,
        pool_share_token_b_mint: Pubkey,
        initial_amount_a: u64,
        initial_amount_b: u64,
        swap_fee_percentage: u8,
        swapverse_fee_percentage: u8,
        min_investment_amount: u64,
        max_days_to_fill: u8,
        swap_life_in_days: u64,
    ) -> Result<()> {
        self.pool_number = pool_number;
        self.active_for_swap = false;
        self.open_for_investment = true;
        self.open_for_withdrawal = false;

        self.token_a_mint = token_a_mint;
        self.token_b_mint = token_b_mint;
        self.pool_share_token_a_mint = pool_share_token_a_mint;
        self.pool_share_token_b_mint = pool_share_token_b_mint;

        self.initial_amount_a = initial_amount_a;
        self.initial_amount_b = initial_amount_b;

        self.swap_fee_percentage = swap_fee_percentage;
        self.swapverse_fee_percentage = swapverse_fee_percentage;

        self.min_investment_amount = min_investment_amount;
        self.max_days_to_fill = max_days_to_fill;
        self.swap_life_in_days = swap_life_in_days;

        self.created_at = Clock::get()?.unix_timestamp;
        self.activated_at = i64::MAX;

        self.token_a_amount_to_be_distributed = 0;
        self.token_b_amount_to_be_distributed = 0;
        self.profit_of_token_a_amount_to_be_distributed = 0;
        self.profit_of_token_b_amount_to_be_distributed = 0;

        Ok(())
    }
}
