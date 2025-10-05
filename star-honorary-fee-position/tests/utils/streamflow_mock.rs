use anchor_lang::prelude::*;

/// Mock Streamflow stream data structure
/// Simplified version of actual Streamflow Stream account
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct MockStreamData {
    pub beneficiary: Pubkey,
    pub total_amount: u64,
    pub withdrawn_amount: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub cliff_time: i64,
}

impl MockStreamData {
    pub fn new(
        beneficiary: Pubkey,
        total_amount: u64,
        start_time: i64,
        end_time: i64,
        cliff_time: i64,
    ) -> Self {
        Self {
            beneficiary,
            total_amount,
            withdrawn_amount: 0,
            start_time,
            end_time,
            cliff_time,
        }
    }
    
    /// Calculate locked amount at given time
    pub fn calculate_locked_amount(&self, current_time: i64) -> u64 {
        // Before cliff: everything is locked
        if current_time < self.cliff_time {
            return self.total_amount;
        }
        
        // After end: nothing is locked
        if current_time >= self.end_time {
            return 0;
        }
        
        // During vesting: calculate based on linear vesting
        let elapsed = current_time - self.start_time;
        let duration = self.end_time - self.start_time;
        
        if duration == 0 {
            return 0;
        }
        
        let vested = self.total_amount
            .checked_mul(elapsed as u64)
            .unwrap()
            .checked_div(duration as u64)
            .unwrap();
        
        // Locked = total - vested - withdrawn
        self.total_amount
            .saturating_sub(vested)
            .saturating_sub(self.withdrawn_amount)
    }
}

/// Helper to create mock stream for testing
pub fn create_test_stream(
    beneficiary: Pubkey,
    total_amount: u64,
    vested_percentage: u8, // 0-100
    current_time: i64,
) -> MockStreamData {
    let start_time = current_time - 86400 * 30; // Started 30 days ago
    let duration = 86400 * 365; // 1 year vesting
    let end_time = start_time + duration;
    let cliff_time = start_time; // No cliff
    
    let mut stream = MockStreamData::new(
        beneficiary,
        total_amount,
        start_time,
        end_time,
        cliff_time,
    );
    
    // Adjust times to get desired vested percentage
    if vested_percentage > 0 {
        let elapsed = (duration * vested_percentage as i64) / 100;
        let adjusted_current = start_time + elapsed;
        stream.end_time = adjusted_current + (duration - elapsed);
    }
    
    stream
}

/// Helper to create fully vested stream
pub fn create_fully_vested_stream(
    beneficiary: Pubkey,
    total_amount: u64,
    current_time: i64,
) -> MockStreamData {
    MockStreamData::new(
        beneficiary,
        total_amount,
        current_time - 100,
        current_time - 1,
        current_time - 100,
    )
}

/// Helper to create fully locked stream
pub fn create_fully_locked_stream(
    beneficiary: Pubkey,
    total_amount: u64,
    current_time: i64,
) -> MockStreamData {
    MockStreamData::new(
        beneficiary,
        total_amount,
        current_time,
        current_time + 86400 * 365,
        current_time + 86400 * 30, // 30 day cliff
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signature::Keypair;
    
    #[test]
    fn test_fully_locked_stream() {
        let beneficiary = Keypair::new().pubkey();
        let current_time = 1000000;
        let stream = create_fully_locked_stream(beneficiary, 1_000_000, current_time);
        
        let locked = stream.calculate_locked_amount(current_time);
        assert_eq!(locked, 1_000_000);
    }
    
    #[test]
    fn test_fully_vested_stream() {
        let beneficiary = Keypair::new().pubkey();
        let current_time = 1000000;
        let stream = create_fully_vested_stream(beneficiary, 1_000_000, current_time);
        
        let locked = stream.calculate_locked_amount(current_time);
        assert_eq!(locked, 0);
    }
    
    #[test]
    fn test_partial_vesting() {
        let beneficiary = Keypair::new().pubkey();
        let start_time = 0;
        let duration = 100;
        let end_time = start_time + duration;
        
        let stream = MockStreamData::new(
            beneficiary,
            1000,
            start_time,
            end_time,
            start_time,
        );
        
        // At 50% through vesting
        let locked = stream.calculate_locked_amount(start_time + 50);
        assert!(locked >= 450 && locked <= 550); // Should be ~500
    }
}