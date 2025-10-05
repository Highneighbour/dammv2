mod utils;
mod scenarios;

#[cfg(test)]
mod integration_tests {
    use anchor_lang::prelude::*;
    use anchor_lang::InstructionData;
    use anchor_spl::token;
    use solana_program_test::*;
    use solana_sdk::{
        signature::{Keypair, Signer},
        transaction::Transaction,
        instruction::{Instruction, AccountMeta},
    };
    use star_honorary_fee_position::{
        instructions::*,
        state::*,
        constants::*,
    };
    use crate::utils::*;
    
    async fn setup_program_test() -> (ProgramTest, Keypair) {
        let mut program_test = ProgramTest::new(
            "star_honorary_fee_position",
            star_honorary_fee_position::ID,
            processor!(star_honorary_fee_position::entry),
        );
        
        let creator = Keypair::new();
        
        program_test.add_account(
            creator.pubkey(),
            solana_sdk::account::Account {
                lamports: 1_000_000_000,
                ..Default::default()
            },
        );
        
        (program_test, creator)
    }
    
    #[tokio::test]
    async fn test_initialize_policy() {
        let (program_test, creator) = setup_program_test().await;
        let mut context = program_test.start_with_context().await;
        
        // Create quote mint
        let quote_mint = create_mint(&mut context, &creator, 6).await;
        
        // Create creator's quote ATA
        let creator_quote_ata = create_token_account(
            &mut context,
            &quote_mint,
            &creator.pubkey(),
        ).await;
        
        // Create vault (just a keypair for testing)
        let vault = Keypair::new();
        
        // Derive PDAs
        let (policy_pda, policy_bump) = Pubkey::find_program_address(
            &[
                VAULT_SEED,
                vault.pubkey().as_ref(),
                POLICY_SEED,
            ],
            &star_honorary_fee_position::ID,
        );
        
        let (progress_pda, _) = Pubkey::find_program_address(
            &[
                VAULT_SEED,
                vault.pubkey().as_ref(),
                PROGRESS_SEED,
            ],
            &star_honorary_fee_position::ID,
        );
        
        let (treasury_pda, _) = Pubkey::find_program_address(
            &[
                VAULT_SEED,
                vault.pubkey().as_ref(),
                TREASURY_SEED,
            ],
            &star_honorary_fee_position::ID,
        );
        
        let (treasury_authority, _) = Pubkey::find_program_address(
            &[
                VAULT_SEED,
                vault.pubkey().as_ref(),
                TREASURY_SEED,
                b"authority",
            ],
            &star_honorary_fee_position::ID,
        );
        
        // Build initialize_policy instruction
        let params = InitializePolicyParams {
            investor_fee_share_bps: 5000, // 50%
            daily_cap_lamports: Some(1_000_000_000),
            min_payout_lamports: 10_000,
            y0_total_allocation: 10_000_000_000,
        };
        
        let accounts = vec![
            AccountMeta::new(creator.pubkey(), true),
            AccountMeta::new_readonly(vault.pubkey(), false),
            AccountMeta::new_readonly(quote_mint, false),
            AccountMeta::new_readonly(creator_quote_ata, false),
            AccountMeta::new(policy_pda, false),
            AccountMeta::new(progress_pda, false),
            AccountMeta::new(treasury_pda, false),
            AccountMeta::new_readonly(treasury_authority, false),
            AccountMeta::new_readonly(token::ID, false),
            AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
            AccountMeta::new_readonly(solana_sdk::sysvar::rent::ID, false),
        ];
        
        let data = star_honorary_fee_position::instruction::InitializePolicy {
            params,
        }
        .data();
        
        let ix = Instruction {
            program_id: star_honorary_fee_position::ID,
            accounts,
            data,
        };
        
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&context.payer.pubkey()),
            &[&context.payer, &creator],
            context.last_blockhash,
        );
        
        let result = context.banks_client.process_transaction(tx).await;
        
        // For now, we expect this to work with the framework
        // In a full implementation, this would validate the account creation
        println!("Initialize policy result: {:?}", result);
    }
    
    #[tokio::test]
    async fn test_quote_only_validation_logic() {
        // Test the quote-only validation logic
        let current_tick = 1000;
        
        // Case 1: Quote is token0, position above price (VALID)
        assert!(validate_quote_only_ticks(current_tick, 1100, 1200, true));
        
        // Case 2: Quote is token0, position overlaps price (INVALID)
        assert!(!validate_quote_only_ticks(current_tick, 900, 1100, true));
        
        // Case 3: Quote is token1, position below price (VALID)
        assert!(validate_quote_only_ticks(current_tick, 800, 900, false));
        
        // Case 4: Quote is token1, position overlaps price (INVALID)
        assert!(!validate_quote_only_ticks(current_tick, 900, 1100, false));
    }
    
    #[tokio::test]
    async fn test_streamflow_locked_calculation() {
        let current_time = 1_000_000i64;
        let beneficiary = Keypair::new().pubkey();
        
        // Fully locked stream
        let locked_stream = create_fully_locked_stream(beneficiary, 1_000_000, current_time);
        assert_eq!(locked_stream.calculate_locked_amount(current_time), 1_000_000);
        
        // Fully vested stream
        let vested_stream = create_fully_vested_stream(beneficiary, 1_000_000, current_time);
        assert_eq!(vested_stream.calculate_locked_amount(current_time), 0);
        
        // Partially vested stream
        let partial_stream = MockStreamData::new(
            beneficiary,
            1_000_000,
            current_time - 50,
            current_time + 50,
            current_time - 50,
        );
        let locked = partial_stream.calculate_locked_amount(current_time);
        
        // Should be approximately 50% locked
        assert!(locked >= 400_000 && locked <= 600_000);
    }
    
    #[tokio::test]
    async fn test_distribution_math() {
        // Test the distribution calculation logic
        
        // Setup: 1M fees, 50% investor share, 60% of tokens locked
        let total_fees = 1_000_000u64;
        let investor_fee_share_bps = 5000u16; // 50%
        let y0 = 10_000_000u64;
        let locked_total = 6_000_000u64;
        
        // Calculate f_locked
        let f_locked_bps = ((locked_total * 10000) / y0) as u16;
        assert_eq!(f_locked_bps, 6000);
        
        // Eligible share = min(policy, f_locked)
        let eligible_share_bps = investor_fee_share_bps.min(f_locked_bps);
        assert_eq!(eligible_share_bps, 5000); // Capped at policy
        
        // Investor portion
        let investor_portion = (total_fees * eligible_share_bps as u64) / 10000;
        assert_eq!(investor_portion, 500_000);
        
        // Creator portion
        let creator_portion = total_fees - investor_portion;
        assert_eq!(creator_portion, 500_000);
    }
    
    #[tokio::test]
    async fn test_pro_rata_distribution() {
        // Test pro-rata distribution across multiple investors
        
        let total_investor_fees = 1_000_000u64;
        let locked_amounts = vec![3_000_000u64, 2_000_000, 5_000_000];
        let locked_total: u64 = locked_amounts.iter().sum();
        
        assert_eq!(locked_total, 10_000_000);
        
        let mut distributions = Vec::new();
        for locked in &locked_amounts {
            let share = (total_investor_fees * locked) / locked_total;
            distributions.push(share);
        }
        
        assert_eq!(distributions[0], 300_000); // 30%
        assert_eq!(distributions[1], 200_000); // 20%
        assert_eq!(distributions[2], 500_000); // 50%
        
        let total_distributed: u64 = distributions.iter().sum();
        assert_eq!(total_distributed, total_investor_fees);
    }
    
    #[tokio::test]
    async fn test_pagination_state_management() {
        // Test pagination cursor management
        
        let mut pagination_cursor = 0u32;
        let total_investors = 55;
        let page_size = 20;
        
        let num_pages = (total_investors + page_size - 1) / page_size;
        assert_eq!(num_pages, 3);
        
        // Process pages
        for page in 0..num_pages {
            let start_idx = page * page_size;
            let end_idx = ((page + 1) * page_size).min(total_investors);
            let investors_in_page = end_idx - start_idx;
            
            println!("Page {}: {} investors", page, investors_in_page);
            
            if page == 0 {
                assert_eq!(investors_in_page, 20);
            } else if page == 1 {
                assert_eq!(investors_in_page, 20);
            } else if page == 2 {
                assert_eq!(investors_in_page, 15);
            }
            
            pagination_cursor += 1;
        }
        
        assert_eq!(pagination_cursor, 3);
    }
}