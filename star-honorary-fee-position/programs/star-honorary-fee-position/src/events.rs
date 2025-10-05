use anchor_lang::prelude::*;

#[event]
pub struct PolicyInitialized {
    pub vault: Pubkey,
    pub creator_wallet: Pubkey,
    pub quote_mint: Pubkey,
    pub investor_fee_share_bps: u16,
    pub daily_cap_lamports: Option<u64>,
    pub y0_total_allocation: u64,
}

#[event]
pub struct HonoraryPositionInitialized {
    pub vault: Pubkey,
    pub pool: Pubkey,
    pub position_id: Pubkey,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub is_quote_token_0: bool,
    pub liquidity: u128,
}

#[event]
pub struct QuoteFeesClaimed {
    pub vault: Pubkey,
    pub amount: u64,
    pub total_distributed: u64,
    pub page: u32,
    pub timestamp: i64,
}

#[event]
pub struct InvestorPayoutPage {
    pub vault: Pubkey,
    pub investor: Pubkey,
    pub amount: u64,
    pub locked_amount: u64,
    pub page: u32,
}

#[event]
pub struct CreatorPayoutDayClosed {
    pub vault: Pubkey,
    pub creator: Pubkey,
    pub amount: u64,
    pub day_ts: i64,
    pub total_distributed_to_investors: u64,
}

#[event]
pub struct DustCarriedOver {
    pub vault: Pubkey,
    pub dust_amount: u64,
    pub page: u32,
}