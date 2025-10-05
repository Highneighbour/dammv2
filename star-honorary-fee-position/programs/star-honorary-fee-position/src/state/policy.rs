use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct PolicyState {
    /// Vault pubkey this policy is associated with
    pub vault: Pubkey,
    
    /// Creator's wallet address
    pub creator_wallet: Pubkey,
    
    /// Creator's quote token ATA for receiving remainder fees
    pub creator_quote_ata: Pubkey,
    
    /// Quote token mint address
    pub quote_mint: Pubkey,
    
    /// Maximum investor fee share in basis points (0-10000)
    /// Actual share is min(investor_fee_share_bps, f_locked * 10000)
    pub investor_fee_share_bps: u16,
    
    /// Optional daily distribution cap in lamports
    pub daily_cap_lamports: Option<u64>,
    
    /// Minimum payout threshold to avoid dust transfers
    pub min_payout_lamports: u64,
    
    /// Total investor allocation at TGE (Y0) - used for f_locked calculation
    pub y0_total_allocation: u64,
    
    /// PDA bump seed
    pub bump: u8,
}

impl PolicyState {
    pub const LEN: usize = 8 + // discriminator
        32 + // vault
        32 + // creator_wallet
        32 + // creator_quote_ata
        32 + // quote_mint
        2 + // investor_fee_share_bps
        1 + 8 + // Option<u64> for daily_cap_lamports
        8 + // min_payout_lamports
        8 + // y0_total_allocation
        1; // bump
}