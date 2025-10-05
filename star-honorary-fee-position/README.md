# Star Honorary Fee Position Module

Production-ready Anchor module for managing quote-only fee positions in DAMM v2 (Raydium CP-AMM).

## 🎯 Overview

This module implements an "honorary" liquidity position in Raydium's Concentrated Liquidity AMM (CP-AMM) that:

- ✅ **Collects ONLY quote token fees** (guaranteed no base token fees)
- ✅ **Distributes fees pro-rata** based on Streamflow locked amounts
- ✅ **Enforces 24-hour distribution cycles** with pagination support
- ✅ **Handles edge cases** (all locked, all unlocked, dust, caps)
- ✅ **Idempotent and resumable** operations
- ✅ **Permissionless cranking** (anyone can trigger distributions)

## 🏗️ Architecture

```
┌──────────────────────────────────────────────────────────────┐
│  CP-AMM Pool (Raydium)                                        │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ Honorary Position (Quote-Only)                         │  │
│  │ - Minimal liquidity (1 unit)                           │  │
│  │ - Tick range ensures ONLY quote fees                   │  │
│  │ - Owned by program PDA                                 │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                            │
                            ▼ collect_fees (every 24h)
┌──────────────────────────────────────────────────────────────┐
│  Program Treasury (PDA)                                       │
│  - Holds collected quote fees                                │
└──────────────────────────────────────────────────────────────┘
                            │
                            ▼ crank_distribution
        ┌───────────────────┴───────────────────┐
        ▼                                       ▼
┌──────────────────┐                  ┌──────────────────┐
│  Investors       │                  │  Creator         │
│  (Pro-rata)      │ ◄────────────── │  (Remainder)     │
│                  │   Based on      │                  │
│  Share based on: │   f_locked      │  Gets:           │
│  - Locked amount │                 │  - Remainder     │
│  - f_locked cap  │                 │  - Dust          │
│  - Min threshold │                 │  - All fees if   │
│                  │                 │    f_locked=0    │
└──────────────────┘                  └──────────────────┘
         ▲
         │
         └─── Locked amounts from Streamflow
```

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Install Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor --tag v0.29.0 anchor-cli

# Install Node dependencies
npm install
```

### Build and Test

```bash
# Build the program
anchor build

# Run tests
cargo test

# Run integration tests
cargo test --features integration
```

### Local Deployment

```bash
# Start local validator and deploy
./scripts/deploy-local.sh

# Set up test pool and policy
npm run setup-pool

# Test crank distribution
npm run test-crank
```

## 📚 Core Concepts

### 1. Quote-Only Fee Collection

The **CRITICAL GUARANTEE** of this module: the position will ONLY collect quote token fees.

**How it works:**

- **If quote is token0**: Position is placed ABOVE current price
  - `tick_lower > current_tick`
  - Only provides token0 (quote) liquidity
  - Only receives token0 fees

- **If quote is token1**: Position is placed BELOW current price
  - `tick_upper < current_tick`
  - Only provides token1 (quote) liquidity
  - Only receives token1 fees

**Validation:**
- At initialization: Tick range is validated
- At runtime: Any base fees cause transaction failure

### 2. Distribution Formula

```
f_locked(t) = locked_total(t) / Y0

eligible_share_bps = min(investor_fee_share_bps, floor(f_locked(t) * 10000))

investor_amount = floor(claimed_fees * eligible_share_bps / 10000)

creator_amount = claimed_fees - investor_amount
```

**Where:**
- `locked_total(t)`: Sum of all locked tokens at time t (from Streamflow)
- `Y0`: Total investor allocation at TGE (Time of Generation Event)
- `investor_fee_share_bps`: Maximum investor share (set in policy, 0-10000)
- `eligible_share_bps`: Actual investor share (capped by f_locked)

**Examples:**

| Locked % | Policy Share | f_locked | Eligible Share | Result |
|----------|--------------|----------|----------------|--------|
| 100% | 50% | 10000 bps | 5000 bps | Investors get 50% |
| 60% | 50% | 6000 bps | 5000 bps | Investors get 50% |
| 40% | 50% | 4000 bps | 4000 bps | Investors get 40% |
| 0% | 50% | 0 bps | 0 bps | Creator gets 100% |

### 3. Pagination

Distribution can handle unlimited investors through pagination:

```rust
// Page 1: Investors 0-19
crank_distribution(investor_count: 20, is_last_page: false)

// Page 2: Investors 20-39
crank_distribution(investor_count: 20, is_last_page: false)

// Page 3: Investors 40-49 (last page)
crank_distribution(investor_count: 10, is_last_page: true)
```

**Features:**
- State preserved between pages
- Dust accumulated and given to creator on last page
- Idempotent: safe to retry failed pages
- 24-hour gate only checked on first page

### 4. 24-Hour Gate

Distributions can only happen once per 24 hours:

```rust
require!(
    now >= progress.last_distribution_ts + SECONDS_PER_DAY,
    ErrorCode::TooEarlyForDistribution
);
```

## 🔧 Integration Guide

### Required Accounts

#### Initialize Policy

```typescript
await program.methods
  .initializePolicy({
    investorFeeShareBps: 5000,      // 50% max to investors
    dailyCapLamports: 1_000_000_000, // Optional: 1000 USDC/day cap
    minPayoutLamports: 10_000,       // Dust threshold: 0.01 USDC
    y0TotalAllocation: 10_000_000_000, // Total at TGE
  })
  .accounts({
    creator: creatorPubkey,
    vault: vaultPubkey,
    quoteMint: quoteMintPubkey,
    creatorQuoteAta: creatorAtaPubkey,
    policy: policyPda,
    progress: progressPda,
    programQuoteTreasury: treasuryPda,
    treasuryAuthority: treasuryAuthorityPda,
    // ... system accounts
  })
  .rpc();
```

#### Initialize Honorary Position

```typescript
await program.methods
  .initializePosition({
    tickLower: 1100,  // Must ensure quote-only
    tickUpper: 1200,
    tickArrayLowerStartIndex: -100,
    tickArrayUpperStartIndex: 100,
  })
  .accounts({
    payer: payerPubkey,
    vault: vaultPubkey,
    policy: policyPda,
    positionState: positionStatePda,
    positionOwnerPda: positionOwnerPda,
    poolState: poolPubkey,
    quoteMint: quoteMintPubkey,
    token0Mint: token0Pubkey,
    token1Mint: token1Pubkey,
    // ... CP-AMM accounts
  })
  .rpc();
```

#### Crank Distribution

```typescript
// Build remaining accounts: [stream, investor_ata] pairs
const remainingAccounts = [];
for (const investor of investors) {
  remainingAccounts.push(
    { pubkey: investor.streamAccount, isSigner: false, isWritable: false },
    { pubkey: investor.quoteAta, isSigner: false, isWritable: true }
  );
}

await program.methods
  .crankDistribution({
    investorCount: investors.length,
    isLastPage: true,
  })
  .accounts({
    cranker: crankerPubkey,
    vault: vaultPubkey,
    policy: policyPda,
    progress: progressPda,
    positionState: positionStatePda,
    positionOwnerPda: positionOwnerPda,
    programQuoteTreasury: treasuryPda,
    treasuryAuthority: treasuryAuthorityPda,
    creatorQuoteAta: creatorAtaPubkey,
    poolState: poolPubkey,
    protocolPosition: protocolPositionPubkey,
    positionNftAccount: positionNftPubkey,
    cpAmmProgram: cpAmmProgramId,
    // ... system accounts
  })
  .remainingAccounts(remainingAccounts)
  .rpc();
```

### PDA Derivations

```rust
// Policy PDA
let (policy, _) = Pubkey::find_program_address(
    &[VAULT_SEED, vault.as_ref(), POLICY_SEED],
    &program_id,
);

// Progress PDA
let (progress, _) = Pubkey::find_program_address(
    &[VAULT_SEED, vault.as_ref(), PROGRESS_SEED],
    &program_id,
);

// Position State PDA
let (position_state, _) = Pubkey::find_program_address(
    &[VAULT_SEED, vault.as_ref(), POSITION_STATE_SEED],
    &program_id,
);

// Position Owner PDA (owns the NFT)
let (position_owner, _) = Pubkey::find_program_address(
    &[VAULT_SEED, vault.as_ref(), POSITION_OWNER_SEED],
    &program_id,
);

// Treasury PDA
let (treasury, _) = Pubkey::find_program_address(
    &[VAULT_SEED, vault.as_ref(), TREASURY_SEED],
    &program_id,
);

// Treasury Authority PDA
let (treasury_authority, _) = Pubkey::find_program_address(
    &[VAULT_SEED, vault.as_ref(), TREASURY_SEED, b"authority"],
    &program_id,
);
```

## 🧪 Testing

### Unit Tests

```bash
# Test quote-only validation
cargo test test_quote_only_validation

# Test distribution math
cargo test test_distribution_math

# Test pagination
cargo test test_pagination

# Test edge cases
cargo test test_edge_cases
```

### Integration Tests

```bash
# Full integration test
cargo test --features integration test_complete_integration_flow

# Test with real CP-AMM (requires local validator with CP-AMM)
./scripts/deploy-local.sh
cargo test --features integration
```

### Test Coverage

- ✅ Quote-only position validation
- ✅ Tick range calculations
- ✅ Distribution formula (f_locked)
- ✅ Pro-rata distribution
- ✅ Pagination state management
- ✅ 24-hour gate enforcement
- ✅ Daily cap enforcement
- ✅ Dust handling
- ✅ Edge cases (all locked/unlocked)
- ✅ Streamflow mock integration

## 🔒 Security Considerations

### Critical Validations

1. **Quote-Only Guarantee**
   ```rust
   // At initialization
   require!(
       validate_quote_only_position(...),
       ErrorCode::PositionWouldAccrueBaseFees
   );
   
   // At runtime
   require!(
       base_fees == 0,
       ErrorCode::BaseFeesDetected
   );
   ```

2. **Math Overflow Protection**
   ```rust
   // All arithmetic uses checked operations
   let result = amount
       .checked_mul(factor)
       .ok_or(ErrorCode::MathOverflow)?;
   ```

3. **Access Control**
   - Policy initialization: Only creator
   - Position initialization: Only with valid policy
   - Crank: Permissionless (anyone can crank)

4. **State Consistency**
   - Pagination state validated
   - 24-hour gate enforced
   - Idempotent operations

### Audit Checklist

- [ ] Quote-only position validated at initialization
- [ ] Base fees cause transaction failure
- [ ] All math uses checked operations
- [ ] PDAs derived correctly
- [ ] Account ownership verified
- [ ] State transitions are atomic
- [ ] Pagination is idempotent
- [ ] 24-hour gate is enforced
- [ ] Daily caps are respected
- [ ] Dust handling is correct

## 📊 Events

The program emits events for monitoring:

```rust
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
```

## 🐛 Troubleshooting

### Common Issues

**1. "Position would accrue base fees"**
- Check that tick range is correct for quote token position
- If quote is token0: `tick_lower > current_tick`
- If quote is token1: `tick_upper < current_tick`

**2. "Too early for distribution"**
- Distributions must be 24 hours apart
- Check `progress.last_distribution_ts`

**3. "Base fees detected"**
- Position is not quote-only
- Review position initialization
- Check pool state and current tick

**4. "Pagination state mismatch"**
- Ensure pages are processed in order
- Don't start new day before completing current

## 📖 Additional Resources

- [Raydium CP-AMM Documentation](https://docs.raydium.io/)
- [Streamflow Documentation](https://docs.streamflow.finance/)
- [Anchor Framework](https://www.anchor-lang.com/)
- [Solana Cookbook](https://solanacookbook.com/)

## 📝 License

MIT

## 🤝 Contributing

This is a production module for the Star fundraising platform. For contributions or issues, please contact the Star development team.

## ⚠️ Disclaimer

This module is designed for integration with Raydium CP-AMM. Ensure thorough testing on devnet before mainnet deployment. The quote-only fee guarantee is critical and must be validated in your specific pool configuration.

---

**Built for Star - Fundraising Platform on Solana**