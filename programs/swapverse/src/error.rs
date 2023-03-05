use anchor_lang::prelude::*;

#[error_code]
pub enum SwapverseError {
    #[msg("User not authorized to use this instruction")]
    UnauthorizedSigner,
    #[msg("Token mint should be one of the defined stable coin mints")]
    InvalidTokenMint,
    #[msg("Token a mint and token b mint can not be same")]
    SameTokenMints,
    #[msg("Initial amount a and initial amount b should be same")]
    InitialAmountsNotSame,
    #[msg("Swap pool not open for investments")]
    SwapPoolNotOpenForInvestment,
    #[msg("Swap pool not open for withdrawals")]
    SwapPoolNotOpenForWithdrawal,
    #[msg("Amount should be greater than minimum investment amount of pool")]
    InsufficientAmount,
    #[msg("Deposit amount is zero")]
    DepositAmountIsZero,
    #[msg("Withdraw amount is zero")]
    WithdrawAmountIsZero,
    #[msg("Token should be one of the tokens specified in the pool")]
    InvalidPoolTokenMint,
    #[msg("Token should be one of the pool share tokens specified in the pool")]
    InvalidPoolShareTokenMint,
    #[msg("Investor should be the owner of investor token account")]
    InvalidInvestorTokenAccountOwner,
    #[msg("Investor token account mint should be same as the token mint")]
    InvalidInvestorTokenAccountMint,
    #[msg("Swap pool not activated for swapping")]
    SwapPoolNotActivated,

    #[msg("Not enough tokens")]
    NotEnoughTokens,
    #[msg("Maximum 10_000_000 tokens can be minted at once")]
    TokenAmountLimitExceeded,
    #[msg("The output is not enough as asked by user")]
    NotEnoughOutput,
}