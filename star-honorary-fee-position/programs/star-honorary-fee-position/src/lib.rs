use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod star_honorary_fee_position {
    use super::*;

    /// Initialize policy for a vault
    /// 
    /// Sets up the fee distribution policy including:
    /// - Maximum investor fee share (basis points)
    /// - Daily distribution cap (optional)
    /// - Minimum payout threshold
    /// - Y0 total allocation for f_locked calculation
    pub fn initialize_policy(
        ctx: Context<instructions::InitializePolicy>,
        params: instructions::InitializePolicyParams,
    ) -> Result<()> {
        instructions::initialize_policy::handler(ctx, params)
    }

    /// Initialize honorary position in CP-AMM pool
    /// 
    /// Creates a quote-only liquidity position that:
    /// - Collects ONLY quote token fees (no base token fees)
    /// - Uses minimal liquidity (1 unit)
    /// - Validates tick range for quote-only strategy
    pub fn initialize_position(
        ctx: Context<instructions::InitializePosition>,
        params: instructions::InitializePositionParams,
    ) -> Result<()> {
        instructions::initialize_position::handler(ctx, params)
    }

    /// Crank distribution for a page of investors
    /// 
    /// Distributes collected quote fees:
    /// 1. Enforces 24-hour gate (on first page)
    /// 2. Claims fees from CP-AMM position (quote only)
    /// 3. Calculates f_locked from Streamflow data
    /// 4. Distributes pro-rata to investors (paginated)
    /// 5. Sends remainder to creator (on last page)
    /// 
    /// Can be called by anyone (permissionless cranking)
    pub fn crank_distribution<'info>(
        ctx: Context<'_, '_, '_, 'info, instructions::CrankDistribution<'info>>,
        params: instructions::CrankDistributionParams,
    ) -> Result<()> {
        instructions::crank_distribution::handler(ctx, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_program_id() {
        assert_eq!(ID.to_string(), "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");
    }
}