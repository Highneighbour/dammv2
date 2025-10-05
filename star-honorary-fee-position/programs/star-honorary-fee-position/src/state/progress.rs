use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct ProgressState {
    /// Vault pubkey this progress is tracking
    pub vault: Pubkey,
    
    /// Timestamp of last completed distribution
    pub last_distribution_ts: i64,
    
    /// Timestamp when current day's distribution started
    pub current_day_ts: i64,
    
    /// Amount distributed to investors today (for daily cap tracking)
    pub daily_distributed: u64,
    
    /// Dust amount carried over from previous pages/distributions
    pub carry_over_dust: u64,
    
    /// Current pagination cursor (0 = new day, increments per page)
    pub pagination_cursor: u32,
    
    /// Whether current day's distribution is complete
    pub day_complete: bool,
    
    /// Total fees claimed from position over lifetime
    pub total_claimed_lifetime: u64,
    
    /// PDA bump seed
    pub bump: u8,
}

impl ProgressState {
    pub const LEN: usize = 8 + // discriminator
        32 + // vault
        8 + // last_distribution_ts
        8 + // current_day_ts
        8 + // daily_distributed
        8 + // carry_over_dust
        4 + // pagination_cursor
        1 + // day_complete
        8 + // total_claimed_lifetime
        1; // bump
}