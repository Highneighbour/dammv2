// PDA seeds - MUST be deterministic
pub const VAULT_SEED: &[u8] = b"vault";
pub const POSITION_OWNER_SEED: &[u8] = b"investor_fee_pos_owner";
pub const POLICY_SEED: &[u8] = b"policy";
pub const PROGRESS_SEED: &[u8] = b"progress";
pub const TREASURY_SEED: &[u8] = b"treasury";
pub const POSITION_STATE_SEED: &[u8] = b"position_state";

// Time constraints
pub const SECONDS_PER_DAY: i64 = 86400;
pub const MAX_INVESTORS_PER_PAGE: usize = 20;

// Basis points max
pub const MAX_BPS: u16 = 10000;

// Raydium CP-AMM Program ID (using placeholder - update with actual)
// For localnet testing, this would be the deployed CP-AMM program
pub const CP_AMM_PROGRAM_ID: &str = "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK";