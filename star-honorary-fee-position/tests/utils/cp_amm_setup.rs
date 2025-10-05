use anchor_lang::prelude::*;
use solana_sdk::signature::Keypair;

/// Mock CP-AMM pool configuration
#[derive(Debug, Clone)]
pub struct MockPoolConfig {
    pub pool_id: Pubkey,
    pub token_0_mint: Pubkey,
    pub token_1_mint: Pubkey,
    pub token_0_vault: Pubkey,
    pub token_1_vault: Pubkey,
    pub current_tick: i32,
    pub sqrt_price_x64: u128,
    pub tick_spacing: u16,
}

impl MockPoolConfig {
    pub fn new(token_0_mint: Pubkey, token_1_mint: Pubkey) -> Self {
        Self {
            pool_id: Keypair::new().pubkey(),
            token_0_mint,
            token_1_mint,
            token_0_vault: Keypair::new().pubkey(),
            token_1_vault: Keypair::new().pubkey(),
            current_tick: 0,
            sqrt_price_x64: 1u128 << 64, // Price = 1.0
            tick_spacing: 1,
        }
    }
    
    pub fn with_current_tick(mut self, tick: i32) -> Self {
        self.current_tick = tick;
        self
    }
}

/// Calculate tick for a given price
/// Simplified version - real implementation would use proper fixed-point math
pub fn price_to_tick(price: f64) -> i32 {
    // tick = log(price) / log(1.0001)
    (price.ln() / 1.0001f64.ln()).round() as i32
}

/// Calculate price from tick
pub fn tick_to_price(tick: i32) -> f64 {
    // price = 1.0001^tick
    1.0001f64.powi(tick)
}

/// Validate quote-only position ticks
pub fn validate_quote_only_ticks(
    current_tick: i32,
    tick_lower: i32,
    tick_upper: i32,
    is_quote_token_0: bool,
) -> bool {
    if is_quote_token_0 {
        // Quote is token0: position must be above current price
        // This means we only provide token0 (quote) liquidity
        tick_lower > current_tick
    } else {
        // Quote is token1: position must be below current price
        // This means we only provide token1 (quote) liquidity
        tick_upper < current_tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_price_to_tick_conversions() {
        // Price = 1.0 should give tick ~ 0
        let tick = price_to_tick(1.0);
        assert!(tick.abs() < 10);
        
        // Higher price should give positive tick
        let tick = price_to_tick(2.0);
        assert!(tick > 0);
        
        // Lower price should give negative tick
        let tick = price_to_tick(0.5);
        assert!(tick < 0);
    }
    
    #[test]
    fn test_validate_quote_only_ticks() {
        let current_tick = 1000;
        
        // Quote is token0: position above current price
        assert!(validate_quote_only_ticks(current_tick, 1100, 1200, true));
        assert!(!validate_quote_only_ticks(current_tick, 900, 1100, true));
        
        // Quote is token1: position below current price
        assert!(validate_quote_only_ticks(current_tick, 800, 900, false));
        assert!(!validate_quote_only_ticks(current_tick, 900, 1100, false));
    }
}