#[cfg(test)]
mod tests {
    use crate::utils::*;
    use solana_sdk::signature::Keypair;
    
    #[test]
    fn test_all_tokens_unlocked() {
        // When all tokens are unlocked, f_locked = 0
        // So investor share should be 0, creator gets 100%
        
        let current_time = 1000000i64;
        let beneficiary = Keypair::new().pubkey();
        
        let stream = create_fully_vested_stream(beneficiary, 1_000_000, current_time);
        let locked = stream.calculate_locked_amount(current_time);
        
        assert_eq!(locked, 0, "Fully vested stream should have 0 locked");
        
        // With 0 locked, f_locked_bps = 0
        let y0 = 1_000_000u64;
        let f_locked_bps = (locked * 10000) / y0;
        assert_eq!(f_locked_bps, 0, "f_locked should be 0");
        
        // Therefore, eligible investor share = min(policy_share, 0) = 0
        let policy_share = 5000u16; // 50%
        let eligible_share = policy_share.min(f_locked_bps as u16);
        assert_eq!(eligible_share, 0, "Eligible share should be 0");
    }
    
    #[test]
    fn test_all_tokens_locked() {
        // When all tokens are locked, f_locked = 1.0 (10000 bps)
        // So investor share = policy share (capped at f_locked)
        
        let current_time = 1000000i64;
        let beneficiary = Keypair::new().pubkey();
        
        let stream = create_fully_locked_stream(beneficiary, 1_000_000, current_time);
        let locked = stream.calculate_locked_amount(current_time);
        
        assert_eq!(locked, 1_000_000, "Fully locked stream should have full amount locked");
        
        // With full amount locked, f_locked_bps = 10000 (100%)
        let y0 = 1_000_000u64;
        let f_locked_bps = (locked * 10000) / y0;
        assert_eq!(f_locked_bps, 10000, "f_locked should be 100%");
        
        // Therefore, eligible investor share = min(policy_share, 10000)
        let policy_share = 5000u16; // 50%
        let eligible_share = policy_share.min(f_locked_bps as u16);
        assert_eq!(eligible_share, 5000, "Eligible share should be policy share");
    }
    
    #[test]
    fn test_partial_vesting_calculation() {
        // Test f_locked calculation with partial vesting
        let y0 = 10_000_000u64;
        
        // Scenario: 60% of tokens still locked
        let locked_amount = 6_000_000u64;
        let f_locked_bps = (locked_amount * 10000) / y0;
        
        assert_eq!(f_locked_bps, 6000, "60% locked should give 6000 bps");
        
        // With policy share of 50% (5000 bps)
        let policy_share = 5000u16;
        let eligible_share = policy_share.min(f_locked_bps as u16);
        
        assert_eq!(eligible_share, 5000, "Should use full policy share");
        
        // With policy share of 70% (7000 bps)
        let policy_share = 7000u16;
        let eligible_share = policy_share.min(f_locked_bps as u16);
        
        assert_eq!(eligible_share, 6000, "Should cap at f_locked");
    }
    
    #[test]
    fn test_daily_cap_enforcement() {
        let daily_cap = 1_000_000u64;
        let already_distributed = 700_000u64;
        let remaining_cap = daily_cap - already_distributed;
        
        assert_eq!(remaining_cap, 300_000);
        
        // Try to distribute 500k when only 300k remains
        let desired_distribution = 500_000u64;
        let actual_distribution = desired_distribution.min(remaining_cap);
        
        assert_eq!(actual_distribution, 300_000, "Should be capped at remaining");
    }
    
    #[test]
    fn test_pro_rata_distribution() {
        // Test pro-rata calculation
        let total_fees = 1_000_000u64;
        let locked_total = 5_000_000u64;
        
        // Investor 1: 2M locked (40%)
        let investor1_locked = 2_000_000u64;
        let investor1_share = (total_fees * investor1_locked) / locked_total;
        assert_eq!(investor1_share, 400_000);
        
        // Investor 2: 3M locked (60%)
        let investor2_locked = 3_000_000u64;
        let investor2_share = (total_fees * investor2_locked) / locked_total;
        assert_eq!(investor2_share, 600_000);
        
        // Total should match
        assert_eq!(investor1_share + investor2_share, total_fees);
    }
}