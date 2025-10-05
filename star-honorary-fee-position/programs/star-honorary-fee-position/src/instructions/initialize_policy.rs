use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::constants::*;
use crate::errors::ErrorCode;
use crate::events::PolicyInitialized;
use crate::state::{PolicyState, ProgressState};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializePolicyParams {
    pub investor_fee_share_bps: u16,
    pub daily_cap_lamports: Option<u64>,
    pub min_payout_lamports: u64,
    pub y0_total_allocation: u64,
}

#[derive(Accounts)]
pub struct InitializePolicy<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    
    /// Vault account (can be any account representing the vault)
    /// CHECK: Used as seed for PDAs
    pub vault: UncheckedAccount<'info>,
    
    /// Quote token mint
    pub quote_mint: Account<'info, Mint>,
    
    /// Creator's quote token ATA
    #[account(
        constraint = creator_quote_ata.owner == creator.key() @ ErrorCode::InvalidPageParameters,
        constraint = creator_quote_ata.mint == quote_mint.key() @ ErrorCode::QuoteMintNotInPool,
    )]
    pub creator_quote_ata: Account<'info, TokenAccount>,
    
    /// Policy state PDA
    #[account(
        init,
        payer = creator,
        space = PolicyState::LEN,
        seeds = [
            VAULT_SEED,
            vault.key().as_ref(),
            POLICY_SEED,
        ],
        bump
    )]
    pub policy: Account<'info, PolicyState>,
    
    /// Progress state PDA
    #[account(
        init,
        payer = creator,
        space = ProgressState::LEN,
        seeds = [
            VAULT_SEED,
            vault.key().as_ref(),
            PROGRESS_SEED,
        ],
        bump
    )]
    pub progress: Account<'info, ProgressState>,
    
    /// Program quote treasury PDA (owned by program, holds collected fees)
    #[account(
        init,
        payer = creator,
        seeds = [
            VAULT_SEED,
            vault.key().as_ref(),
            TREASURY_SEED,
        ],
        bump,
        token::mint = quote_mint,
        token::authority = treasury_authority,
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
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<InitializePolicy>, params: InitializePolicyParams) -> Result<()> {
    // Validate parameters
    require!(
        params.investor_fee_share_bps <= MAX_BPS,
        ErrorCode::InvalidFeeShareBps
    );
    
    require!(
        params.y0_total_allocation > 0,
        ErrorCode::InvalidY0Allocation
    );
    
    // Initialize policy state
    let policy = &mut ctx.accounts.policy;
    policy.vault = ctx.accounts.vault.key();
    policy.creator_wallet = ctx.accounts.creator.key();
    policy.creator_quote_ata = ctx.accounts.creator_quote_ata.key();
    policy.quote_mint = ctx.accounts.quote_mint.key();
    policy.investor_fee_share_bps = params.investor_fee_share_bps;
    policy.daily_cap_lamports = params.daily_cap_lamports;
    policy.min_payout_lamports = params.min_payout_lamports;
    policy.y0_total_allocation = params.y0_total_allocation;
    policy.bump = ctx.bumps.policy;
    
    // Initialize progress state
    let progress = &mut ctx.accounts.progress;
    progress.vault = ctx.accounts.vault.key();
    progress.last_distribution_ts = 0; // No distributions yet
    progress.current_day_ts = 0;
    progress.daily_distributed = 0;
    progress.carry_over_dust = 0;
    progress.pagination_cursor = 0;
    progress.day_complete = true; // Ready for first distribution
    progress.total_claimed_lifetime = 0;
    progress.bump = ctx.bumps.progress;
    
    emit!(PolicyInitialized {
        vault: ctx.accounts.vault.key(),
        creator_wallet: policy.creator_wallet,
        quote_mint: policy.quote_mint,
        investor_fee_share_bps: params.investor_fee_share_bps,
        daily_cap_lamports: params.daily_cap_lamports,
        y0_total_allocation: params.y0_total_allocation,
    });
    
    Ok(())
}