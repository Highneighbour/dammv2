use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct PositionState {
    /// Vault pubkey this position is associated with
    pub vault: Pubkey,
    
    /// CP-AMM pool ID
    pub pool_id: Pubkey,
    
    /// Position NFT mint address
    pub position_nft_mint: Pubkey,
    
    /// CP-AMM position account ID
    pub position_id: Pubkey,
    
    /// Lower tick boundary
    pub tick_lower: i32,
    
    /// Upper tick boundary
    pub tick_upper: i32,
    
    /// Position liquidity amount
    pub liquidity: u128,
    
    /// Whether quote token is token0 in the pool
    /// If true: quote = token0, position must be ABOVE current price
    /// If false: quote = token1, position must be BELOW current price
    pub is_quote_token_0: bool,
    
    /// PDA bump seed
    pub bump: u8,
}

impl PositionState {
    pub const LEN: usize = 8 + // discriminator
        32 + // vault
        32 + // pool_id
        32 + // position_nft_mint
        32 + // position_id
        4 + // tick_lower
        4 + // tick_upper
        16 + // liquidity (u128)
        1 + // is_quote_token_0
        1; // bump
}