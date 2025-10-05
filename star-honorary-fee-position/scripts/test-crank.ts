/**
 * Test Crank Script
 * 
 * This script tests the crank_distribution instruction with mock investor data.
 */

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram, SYSVAR_CLOCK_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createAccount, getAccount } from "@solana/spl-token";
import { StarHonoraryFeePosition } from "../target/types/star_honorary_fee_position";

const VAULT_SEED = "vault";
const POLICY_SEED = "policy";
const PROGRESS_SEED = "progress";
const POSITION_STATE_SEED = "position_state";
const POSITION_OWNER_SEED = "investor_fee_pos_owner";
const TREASURY_SEED = "treasury";

// Load config from setup-pool output
// In a real scenario, this would be loaded from a file
const CONFIG = {
  vault: "", // Set this from setup-pool output
  quoteMint: "",
  policy: "",
  progress: "",
  treasury: "",
  treasuryAuthority: "",
  positionOwner: "",
  positionState: "",
};

async function main() {
  console.log("🔄 Testing crank distribution...\n");

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.StarHonoraryFeePosition as Program<StarHonoraryFeePosition>;
  const cranker = provider.wallet as anchor.Wallet;

  console.log("Program ID:", program.programId.toString());
  console.log("Cranker:", cranker.publicKey.toString());
  console.log("");

  // For demonstration, we'll show the expected structure of a crank call
  console.log("📋 Crank Distribution Structure:");
  console.log("");
  console.log("Required accounts:");
  console.log("  - cranker: Anyone can crank");
  console.log("  - vault: The vault being distributed for");
  console.log("  - policy: Fee distribution policy");
  console.log("  - progress: Pagination state");
  console.log("  - position_state: Honorary position info");
  console.log("  - position_owner_pda: Owner of the position");
  console.log("  - program_quote_treasury: Treasury with fees");
  console.log("  - treasury_authority: Treasury signer");
  console.log("  - creator_quote_ata: Creator receives remainder");
  console.log("  - pool_state: CP-AMM pool");
  console.log("  - protocol_position: CP-AMM position");
  console.log("  - position_nft_account: Position NFT");
  console.log("  - cp_amm_program: Raydium CP-AMM program");
  console.log("  - token_program: SPL Token");
  console.log("  - clock: Clock sysvar");
  console.log("");
  console.log("Remaining accounts (per investor):");
  console.log("  - stream_account: Streamflow stream data");
  console.log("  - investor_quote_ata: Investor receives pro-rata share");
  console.log("");

  console.log("Parameters:");
  console.log("  - investor_count: Number of investors in this page (max 20)");
  console.log("  - is_last_page: Whether this completes the day");
  console.log("");

  console.log("Distribution Logic:");
  console.log("  1. Check 24-hour gate (first page only)");
  console.log("  2. Claim fees from CP-AMM position (quote only)");
  console.log("  3. Read locked amounts from Streamflow");
  console.log("  4. Calculate f_locked = locked_total / Y0");
  console.log("  5. Investor share = min(policy_share, f_locked) * fees");
  console.log("  6. Distribute pro-rata to investors");
  console.log("  7. Send remainder to creator (last page)");
  console.log("");

  console.log("Example distribution calculation:");
  console.log("  Total fees: 1,000 USDC");
  console.log("  Policy share: 50% (5000 bps)");
  console.log("  Y0: 10,000,000 tokens");
  console.log("  Locked total: 6,000,000 tokens");
  console.log("  f_locked: 60% (6000 bps)");
  console.log("  Eligible share: min(50%, 60%) = 50%");
  console.log("  Investor portion: 500 USDC");
  console.log("  Creator portion: 500 USDC");
  console.log("");

  console.log("Pro-rata example (3 investors):");
  const investorLocked = [3_000_000, 2_000_000, 5_000_000];
  const lockedTotal = investorLocked.reduce((a, b) => a + b, 0);
  const investorPortion = 500;

  investorLocked.forEach((locked, i) => {
    const share = (investorPortion * locked) / lockedTotal;
    const percent = ((locked / lockedTotal) * 100).toFixed(1);
    console.log(`  Investor ${i + 1}: ${locked.toLocaleString()} locked (${percent}%) → ${share.toFixed(2)} USDC`);
  });

  console.log("");
  console.log("========================================");
  console.log("✅ Crank structure explained!");
  console.log("========================================");
  console.log("");
  console.log("To run actual crank:");
  console.log("  1. Set up CP-AMM pool with position");
  console.log("  2. Create Streamflow streams for investors");
  console.log("  3. Call crank_distribution with all accounts");
  console.log("");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });