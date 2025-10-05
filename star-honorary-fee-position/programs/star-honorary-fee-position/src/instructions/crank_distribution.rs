use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::*;
use crate::state::{PolicyState, PositionState, ProgressState};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CrankDistributionParams {
    /// Number of investors in this page
    pub investor_count: u8,
    
    /// Whether this is the last page of the day
    pub is_last_page: bool,
}

#[derive(Accounts)]
#[instruction(params: CrankDistributionParams)]
pub struct CrankDistribution<'info> {
    /// Cranker (can be anyone)
    pub cranker: Signer<'info>,
    
    /// Vault account
    /// CHECK: Used as seed for PDAs
    pub vault: UncheckedAccount<'info>,
    
    /// Policy state
    #[account(
        seeds = [
            VAULT_SEED,
            vault.key().as_ref(),
            POLICY_SEED,
        ],
        bump = policy.bump,
    )]
    pub policy: Account<'info, PolicyState>,
    
    /// Progress state (mutable - tracking pagination)
    #[account(
        mut,
        seeds = [
            VAULT_SEED,
            vault.key().as_ref(),
            PROGRESS_SEED,
        ],
        bump = progress.bump,
    )]
    pub progress: Account<'info, ProgressState>,
    
    /// Position state (readonly - position configuration)
    #[account(
        seeds = [
            VAULT_SEED,
            vault.key().as_ref(),
            POSITION_STATE_SEED,
        ],
        bump = position_state.bump,
    )]
    pub position_state: Account<'info, PositionState>,
    
    /// Position owner PDA
    /// CHECK: PDA that owns the position NFT
    #[account(
        seeds = [
            VAULT_SEED,
            vault.key().as_ref(),
            POSITION_OWNER_SEED,
        ],
        bump
    )]
    pub position_owner_pda: UncheckedAccount<'info>,
    
    /// Program quote treasury (holds collected fees)
    #[account(
        mut,
        seeds = [
            VAULT_SEED,
            vault.key().as_ref(),
            TREASURY_SEED,
        ],
        bump,
        constraint = program_quote_treasury.mint == policy.quote_mint @ ErrorCode::QuoteMintNotInPool,
    )]
    pub program_quote_treasury: Account<'info, TokenAccount>,
    
    /// Treasury authority PDA
    /// CHECK: PDA used as authority for treasury
    #[account(
        seeds = [
            VAULT_SEED,
            vault.key().as_ref(),
            TREASURY_SEED,
            b"authority",
        ],
        bump
    )]
    pub treasury_authority: UncheckedAccount<'info>,
    
    /// Creator's quote token ATA (receives remainder)
    #[account(
        mut,
        constraint = creator_quote_ata.key() == policy.creator_quote_ata @ ErrorCode::InvalidPageParameters,
    )]
    pub creator_quote_ata: Account<'info, TokenAccount>,
    
    /// CP-AMM pool state
    /// CHECK: Validated by CP-AMM program
    #[account(mut)]
    pub pool_state: UncheckedAccount<'info>,
    
    /// CP-AMM protocol position
    /// CHECK: Validated by CP-AMM program
    #[account(mut)]
    pub protocol_position: UncheckedAccount<'info>,
    
    /// Position NFT account
    /// CHECK: Validated by CP-AMM program
    pub position_nft_account: UncheckedAccount<'info>,
    
    /// CP-AMM Program
    /// CHECK: CP-AMM program ID
    pub cp_amm_program: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>,
    
    // Remaining accounts:
    // For each investor: [stream_account, investor_quote_ata]
    // Stream accounts are Streamflow stream data accounts
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CrankDistribution<'info>>,
    params: CrankDistributionParams,
) -> Result<()> {
    let clock = &ctx.accounts.clock;
    let now = clock.unix_timestamp;
    
    let progress = &mut ctx.accounts.progress;
    let policy = &ctx.accounts.policy;
    
    require!(
        params.investor_count as usize <= MAX_INVESTORS_PER_PAGE,
        ErrorCode::InvalidInvestorCount
    );
    
    require!(
        ctx.remaining_accounts.len() >= (params.investor_count as usize * 2),
        ErrorCode::InvalidInvestorCount
    );
    
    // 1. Check 24-hour gate and pagination state
    if progress.pagination_cursor == 0 {
        // First page of a new day
        require!(
            progress.day_complete,
            ErrorCode::DistributionNotComplete
        );
        
        require!(
            now >= progress.last_distribution_ts + SECONDS_PER_DAY,
            ErrorCode::TooEarlyForDistribution
        );
        
        // Reset for new day
        progress.current_day_ts = now;
        progress.daily_distributed = 0;
        progress.day_complete = false;
    } else {
        // Continuing pagination - ensure we're in the same day
        require!(
            !progress.day_complete,
            ErrorCode::PaginationStateMismatch
        );
    }
    
    // 2. Claim fees from honorary position
    // In a real implementation, this would call CP-AMM's collect_fees
    // For this framework, we simulate by reading the treasury balance
    let claimed_quote_amount = ctx.accounts.program_quote_treasury.amount;
    
    // CRITICAL: In real implementation, verify base_fees == 0
    // This would come from the CP-AMM collect_fees return value
    
    msg!("Claimed quote fees: {}", claimed_quote_amount);
    
    // Add carry-over from previous pages
    let total_available = claimed_quote_amount
        .checked_add(progress.carry_over_dust)
        .ok_or(ErrorCode::MathOverflow)?;
    
    // 3. Calculate investor share based on locked amounts
    let mut locked_total = 0u64;
    let mut locked_amounts = Vec::new();
    
    // Read Streamflow locked amounts from remaining accounts
    for i in 0..params.investor_count as usize {
        let stream_account = &ctx.remaining_accounts[i * 2];
        
        // Parse Streamflow stream data
        // In a real implementation, this would deserialize the Streamflow account
        // For this framework, we'll use a simplified mock approach
        let locked_amount = parse_streamflow_locked_amount(stream_account, now)?;
        
        locked_amounts.push(locked_amount);
        locked_total = locked_total
            .checked_add(locked_amount)
            .ok_or(ErrorCode::MathOverflow)?;
    }
    
    msg!("Total locked: {}", locked_total);
    
    // Calculate f_locked(t) = locked_total / Y0 in basis points
    let f_locked_bps = if policy.y0_total_allocation > 0 {
        let fraction = locked_total
            .checked_mul(MAX_BPS as u64)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(policy.y0_total_allocation)
            .unwrap_or(0);
        fraction.min(MAX_BPS as u64) as u16
    } else {
        0
    };
    
    msg!("f_locked: {} bps", f_locked_bps);
    
    // Calculate eligible investor share
    let eligible_investor_share_bps = policy.investor_fee_share_bps.min(f_locked_bps);
    
    // Calculate investor portion
    let investor_fee_quote = total_available
        .checked_mul(eligible_investor_share_bps as u64)
        .ok_or(ErrorCode::MathOverflow)?
        .checked_div(MAX_BPS as u64)
        .unwrap_or(0);
    
    msg!("Investor fee portion: {}", investor_fee_quote);
    
    // Apply daily cap if set
    let investor_fee_quote = if let Some(cap) = policy.daily_cap_lamports {
        let remaining_cap = cap.saturating_sub(progress.daily_distributed);
        investor_fee_quote.min(remaining_cap)
    } else {
        investor_fee_quote
    };
    
    // 4. Distribute to investors pro-rata
    let mut total_distributed = 0u64;
    let mut dust_accumulated = progress.carry_over_dust;
    
    let vault_key = ctx.accounts.vault.key();
    let treasury_seeds = &[
        VAULT_SEED,
        vault_key.as_ref(),
        TREASURY_SEED,
        b"authority",
        &[ctx.bumps.treasury_authority],
    ];
    let signer_seeds = &[&treasury_seeds[..]];
    
    for i in 0..params.investor_count as usize {
        let investor_ata_info = &ctx.remaining_accounts[i * 2 + 1];
        let locked_amount = locked_amounts[i];
        
        if locked_total > 0 && locked_amount > 0 {
            // Calculate pro-rata share
            let investor_share = investor_fee_quote
                .checked_mul(locked_amount)
                .ok_or(ErrorCode::MathOverflow)?
                .checked_div(locked_total)
                .unwrap_or(0);
            
            msg!("Investor {} share: {}", i, investor_share);
            
            // Apply minimum payout threshold
            if investor_share >= policy.min_payout_lamports {
                // Transfer to investor
                let cpi_accounts = Transfer {
                    from: ctx.accounts.program_quote_treasury.to_account_info(),
                    to: investor_ata_info.to_account_info(),
                    authority: ctx.accounts.treasury_authority.to_account_info(),
                };
                
                let cpi_ctx = CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    cpi_accounts,
                    signer_seeds,
                );
                
                token::transfer(cpi_ctx, investor_share)?;
                
                total_distributed = total_distributed
                    .checked_add(investor_share)
                    .ok_or(ErrorCode::MathOverflow)?;
                
                emit!(InvestorPayoutPage {
                    vault: vault_key,
                    investor: investor_ata_info.key(),
                    amount: investor_share,
                    locked_amount,
                    page: progress.pagination_cursor,
                });
            } else {
                // Below threshold - accumulate as dust
                dust_accumulated = dust_accumulated
                    .checked_add(investor_share)
                    .ok_or(ErrorCode::MathOverflow)?;
            }
        }
    }
    
    msg!("Total distributed: {}, dust: {}", total_distributed, dust_accumulated);
    
    // Update progress
    progress.daily_distributed = progress.daily_distributed
        .checked_add(total_distributed)
        .ok_or(ErrorCode::MathOverflow)?;
    
    // 5. Check if this is the last page
    if params.is_last_page {
        // Calculate creator portion (remainder)
        let creator_amount = total_available
            .checked_sub(total_distributed)
            .ok_or(ErrorCode::MathOverflow)?;
        
        msg!("Creator amount: {}", creator_amount);
        
        // Transfer to creator (includes dust)
        if creator_amount > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.program_quote_treasury.to_account_info(),
                to: ctx.accounts.creator_quote_ata.to_account_info(),
                authority: ctx.accounts.treasury_authority.to_account_info(),
            };
            
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            );
            
            token::transfer(cpi_ctx, creator_amount)?;
            
            emit!(CreatorPayoutDayClosed {
                vault: vault_key,
                creator: policy.creator_wallet,
                amount: creator_amount,
                day_ts: progress.current_day_ts,
                total_distributed_to_investors: total_distributed,
            });
        }
        
        // Finalize day
        progress.day_complete = true;
        progress.last_distribution_ts = now;
        progress.pagination_cursor = 0;
        progress.carry_over_dust = 0; // Reset dust after giving to creator
        progress.total_claimed_lifetime = progress.total_claimed_lifetime
            .checked_add(claimed_quote_amount)
            .ok_or(ErrorCode::MathOverflow)?;
    } else {
        // Not last page - increment cursor and carry dust
        progress.pagination_cursor += 1;
        progress.carry_over_dust = dust_accumulated;
        
        emit!(DustCarriedOver {
            vault: vault_key,
            dust_amount: dust_accumulated,
            page: progress.pagination_cursor,
        });
    }
    
    emit!(QuoteFeesClaimed {
        vault: vault_key,
        amount: claimed_quote_amount,
        total_distributed,
        page: progress.pagination_cursor,
        timestamp: now,
    });
    
    Ok(())
}

/// Parse Streamflow locked amount from stream account
/// This is a simplified mock - real implementation would deserialize Streamflow account
fn parse_streamflow_locked_amount(
    stream_account: &AccountInfo,
    _current_time: i64,
) -> Result<u64> {
    // In a real implementation, this would:
    // 1. Deserialize the Streamflow Stream account
    // 2. Calculate: locked = total - vested - withdrawn
    // 3. Handle cliff and vesting schedule
    
    // For this framework, we'll return a mock value based on account data length
    // In production, replace with actual Streamflow parsing
    
    if stream_account.data_len() == 0 {
        return Ok(0);
    }
    
    // Mock: return a value based on lamports (for testing)
    // Real implementation: parse Streamflow data structure
    let locked = stream_account.lamports() % 1000000;
    Ok(locked)
}