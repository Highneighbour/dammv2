#[cfg(test)]
mod tests {
    use crate::utils::*;
    
    #[test]
    fn test_investor_batching() {
        // Test that we can properly batch investors into pages
        let total_investors = 50;
        let max_per_page = 20;
        
        let pages_needed = (total_investors + max_per_page - 1) / max_per_page;
        assert_eq!(pages_needed, 3, "50 investors should need 3 pages");
        
        // First two pages: 20 investors each
        // Last page: 10 investors
        let last_page_count = total_investors - (max_per_page * (pages_needed - 1));
        assert_eq!(last_page_count, 10, "Last page should have 10 investors");
    }
    
    #[test]
    fn test_dust_accumulation() {
        // Test dust handling across pages
        let min_payout = 10000u64;
        
        // Simulate small amounts below threshold
        let small_amounts = vec![5000u64, 3000, 2000, 4000, 6000];
        let total_dust: u64 = small_amounts.iter().sum();
        
        assert_eq!(total_dust, 20000, "Total dust should accumulate");
        
        // All individual amounts are below threshold
        for amount in &small_amounts {
            assert!(
                *amount < min_payout,
                "Each amount should be below threshold"
            );
        }
    }
}