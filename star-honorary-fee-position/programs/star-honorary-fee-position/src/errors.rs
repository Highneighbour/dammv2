use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Quote mint not found in pool")]
    QuoteMintNotInPool,
    
    #[msg("Position would accrue base fees - invalid tick range")]
    PositionWouldAccrueBaseFees,
    
    #[msg("Base fees detected - refusing to distribute")]
    BaseFeesDetected,
    
    #[msg("Too early for distribution - 24h not elapsed")]
    TooEarlyForDistribution,
    
    #[msg("Math overflow detected")]
    MathOverflow,
    
    #[msg("Invalid tick range for quote-only position")]
    InvalidTickRangeForQuoteOnly,
    
    #[msg("Pagination state mismatch")]
    PaginationStateMismatch,
    
    #[msg("Invalid fee share basis points")]
    InvalidFeeShareBps,
    
    #[msg("Daily cap exceeded")]
    DailyCapExceeded,
    
    #[msg("Invalid Y0 allocation")]
    InvalidY0Allocation,
    
    #[msg("Invalid investor count")]
    InvalidInvestorCount,
    
    #[msg("Policy already initialized")]
    PolicyAlreadyInitialized,
    
    #[msg("Position already initialized")]
    PositionAlreadyInitialized,
    
    #[msg("Invalid Streamflow data")]
    InvalidStreamflowData,
    
    #[msg("Insufficient fee balance")]
    InsufficientFeeBalance,
    
    #[msg("Distribution not complete")]
    DistributionNotComplete,
    
    #[msg("Invalid page parameters")]
    InvalidPageParameters,
    
    #[msg("Tick out of bounds")]
    TickOutOfBounds,
}