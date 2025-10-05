#[cfg(test)]
mod tests {
    use crate::utils::*;
    
    #[test]
    fn test_quote_only_tick_validation() {
        // Test case 1: Quote is token0, position above current price (VALID)
        let current_tick = 1000;
        let tick_lower = 1100;
        let tick_upper = 1200;
        let is_quote_token_0 = true;
        
        assert!(validate_quote_only_ticks(
            current_tick,
            tick_lower,
            tick_upper,
            is_quote_token_0
        ));
        
        // Test case 2: Quote is token0, position below current price (INVALID)
        let tick_lower = 800;
        let tick_upper = 900;
        
        assert!(!validate_quote_only_ticks(
            current_tick,
            tick_lower,
            tick_upper,
            is_quote_token_0
        ));
        
        // Test case 3: Quote is token1, position below current price (VALID)
        let is_quote_token_0 = false;
        
        assert!(validate_quote_only_ticks(
            current_tick,
            tick_lower,
            tick_upper,
            is_quote_token_0
        ));
        
        // Test case 4: Quote is token1, position above current price (INVALID)
        let tick_lower = 1100;
        let tick_upper = 1200;
        
        assert!(!validate_quote_only_ticks(
            current_tick,
            tick_lower,
            tick_upper,
            is_quote_token_0
        ));
    }
    
    #[test]
    fn test_price_to_tick_conversions() {
        // Test that we can convert prices to ticks correctly
        let price_1 = 1.0;
        let tick_1 = price_to_tick(price_1);
        assert!(tick_1.abs() < 10, "Price 1.0 should give tick near 0");
        
        let price_2 = 2.0;
        let tick_2 = price_to_tick(price_2);
        assert!(tick_2 > 6900 && tick_2 < 7000, "Price 2.0 should give tick ~6931");
        
        let price_half = 0.5;
        let tick_half = price_to_tick(price_half);
        assert!(tick_half > -7000 && tick_half < -6900, "Price 0.5 should give tick ~-6931");
    }
}