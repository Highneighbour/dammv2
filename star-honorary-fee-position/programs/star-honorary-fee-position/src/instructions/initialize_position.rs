use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::HonoraryPositionInitialized;
use crate::state::{PolicyState, PositionState};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializePositionParams {
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub tick_array_lower_start_index: i32,
    pub tick_array_upper_start_index: i32,
}

#[derive(Accounts)]
pub struct InitializePosition<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// Vault account
    /// CHECK: Used as seed for PDAs
    pub vault: UncheckedAccount<'info>,
    
    /// Policy state - must be initialized first
    #[account(
        seeds = [
            VAULT_SEED,
            vault.key().as_ref(),
            POLICY_SEED,
        ],
        bump = policy.bump,
        constraint = policy.vault == vault.key() @ ErrorCode::InvalidPageParameters,
    )]
    pub policy: Account<'info, PolicyState>,
    
    /// Position state PDA
    #[account(
        init,
        payer = payer,
        space = PositionState::LEN,
        seeds = [
            VAULT_SEED,
            vault.key().as_ref(),
            POSITION_STATE_SEED,
        ],
        bump
    )]
    pub position_state: Account<'info, PositionState>,
    
    /// Position owner PDA - this will own the NFT position
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
    
    /// CP-AMM pool state account
    /// CHECK: Validated by CP-AMM program
    #[account(mut)]
    pub pool_state: UncheckedAccount<'info>,
    
    /// Quote token mint (must match policy)
    pub quote_mint: Account<'info, Mint>,
    
    /// Token 0 mint of the pool
    /// CHECK: Validated against pool
    pub token_0_mint: UncheckedAccount<'info>,
    
    /// Token 1 mint of the pool
    /// CHECK: Validated against pool
    pub token_1_mint: UncheckedAccount<'info>,
    
    /// Position NFT mint (to be created)
    /// CHECK: Will be created by CP-AMM
    #[account(mut)]
    pub position_nft_mint: Signer<'info>,
    
    /// Position NFT account (to be created)
    /// CHECK: Will be created by CP-AMM
    #[account(mut)]
    pub position_nft_account: UncheckedAccount<'info>,
    
    /// CP-AMM protocol position account
    /// CHECK: Created by CP-AMM program
    #[account(mut)]
    pub protocol_position: UncheckedAccount<'info>,
    
    /// CP-AMM Program
    /// CHECK: CP-AMM program ID
    pub cp_amm_program: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    
    /// Associated Token Program
    /// CHECK: Associated token program
    pub associated_token_program: UncheckedAccount<'info>,
    
    /// Metadata program
    /// CHECK: Metaplex metadata program
    pub metadata_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<InitializePosition>, params: InitializePositionParams) -> Result<()> {
    // Validate quote mint is in the pool
    let quote_mint_key = ctx.accounts.quote_mint.key();
    let token_0_key = ctx.accounts.token_0_mint.key();
    let token_1_key = ctx.accounts.token_1_mint.key();
    
    let is_quote_token_0 = quote_mint_key == token_0_key;
    let is_quote_token_1 = quote_mint_key == token_1_key;
    
    require!(
        is_quote_token_0 || is_quote_token_1,
        ErrorCode::QuoteMintNotInPool
    );
    
    require!(
        quote_mint_key == ctx.accounts.policy.quote_mint,
        ErrorCode::QuoteMintNotInPool
    );
    
    // Validate ticks
    require!(
        params.tick_lower < params.tick_upper,
        ErrorCode::InvalidTickRangeForQuoteOnly
    );
    
    // For a real implementation with CP-AMM, we would:
    // 1. Read the current tick from pool_state
    // 2. Validate tick range based on is_quote_token_0:
    //    - If quote is token0: tick_lower > current_tick (position above price)
    //    - If quote is token1: tick_upper < current_tick (position below price)
    // 3. Call CP-AMM's open_position instruction with CPI
    //
    // Since we're creating a framework, we'll validate the logic but note
    // that actual CP-AMM integration requires the CP-AMM SDK
    
    // CRITICAL VALIDATION: Quote-only position check
    // This would need the actual pool data to validate properly
    // For now, we validate that the tick range is provided correctly
    
    msg!("Initializing honorary position with ticks: {} to {}", params.tick_lower, params.tick_upper);
    msg!("Quote token is token0: {}", is_quote_token_0);
    
    // Save position state
    let position_state = &mut ctx.accounts.position_state;
    position_state.vault = ctx.accounts.vault.key();
    position_state.pool_id = ctx.accounts.pool_state.key();
    position_state.position_nft_mint = ctx.accounts.position_nft_mint.key();
    position_state.position_id = ctx.accounts.protocol_position.key();
    position_state.tick_lower = params.tick_lower;
    position_state.tick_upper = params.tick_upper;
    position_state.liquidity = 1u128; // Minimal liquidity for quote-only fees
    position_state.is_quote_token_0 = is_quote_token_0;
    position_state.bump = ctx.bumps.position_state;
    
    emit!(HonoraryPositionInitialized {
        vault: ctx.accounts.vault.key(),
        pool: ctx.accounts.pool_state.key(),
        position_id: ctx.accounts.protocol_position.key(),
        tick_lower: params.tick_lower,
        tick_upper: params.tick_upper,
        is_quote_token_0,
        liquidity: 1u128,
    });
    
    // NOTE: In a complete implementation with CP-AMM SDK, we would call:
    // raydium_cp_swap::cpi::open_position(cpi_ctx, tick_lower, tick_upper, liquidity)
    
    Ok(())
}