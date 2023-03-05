use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct GlobalState{
    pub token_mints: [Pubkey; 5],
    pub signing_authority_bump: u8,
    pub no_of_swap_pools: u64,
}

impl GlobalState {
    pub fn initialize(&mut self, token_mints: [Pubkey; 5], signing_authority_bump: &u8) -> Result<()> {
        self.token_mints = token_mints;
        self.signing_authority_bump = *signing_authority_bump;
        self.no_of_swap_pools = 0;

        Ok(())
    }
}