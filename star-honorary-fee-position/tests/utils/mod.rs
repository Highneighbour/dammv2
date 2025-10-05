pub mod cp_amm_setup;
pub mod streamflow_mock;

pub use cp_amm_setup::*;
pub use streamflow_mock::*;

use anchor_lang::prelude::*;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};

/// Helper to create a test token mint
pub async fn create_mint(
    context: &mut ProgramTestContext,
    authority: &Keypair,
    decimals: u8,
) -> Pubkey {
    let mint = Keypair::new();
    
    let rent = context.banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(82); // SPL Token Mint account size
    
    let create_mint_ix = solana_sdk::system_instruction::create_account(
        &context.payer.pubkey(),
        &mint.pubkey(),
        mint_rent,
        82,
        &spl_token::id(),
    );
    
    let init_mint_ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint.pubkey(),
        &authority.pubkey(),
        None,
        decimals,
    )
    .unwrap();
    
    let tx = Transaction::new_signed_with_payer(
        &[create_mint_ix, init_mint_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint],
        context.last_blockhash,
    );
    
    context.banks_client.process_transaction(tx).await.unwrap();
    
    mint.pubkey()
}

/// Helper to create a token account
pub async fn create_token_account(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    owner: &Pubkey,
) -> Pubkey {
    let account = Keypair::new();
    
    let rent = context.banks_client.get_rent().await.unwrap();
    let account_rent = rent.minimum_balance(165); // SPL Token Account size
    
    let create_account_ix = solana_sdk::system_instruction::create_account(
        &context.payer.pubkey(),
        &account.pubkey(),
        account_rent,
        165,
        &spl_token::id(),
    );
    
    let init_account_ix = spl_token::instruction::initialize_account(
        &spl_token::id(),
        &account.pubkey(),
        mint,
        owner,
    )
    .unwrap();
    
    let tx = Transaction::new_signed_with_payer(
        &[create_account_ix, init_account_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, &account],
        context.last_blockhash,
    );
    
    context.banks_client.process_transaction(tx).await.unwrap();
    
    account.pubkey()
}

/// Helper to mint tokens
pub async fn mint_tokens(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    destination: &Pubkey,
    authority: &Keypair,
    amount: u64,
) {
    let mint_to_ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        mint,
        destination,
        &authority.pubkey(),
        &[],
        amount,
    )
    .unwrap();
    
    let tx = Transaction::new_signed_with_payer(
        &[mint_to_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer, authority],
        context.last_blockhash,
    );
    
    context.banks_client.process_transaction(tx).await.unwrap();
}

/// Helper to get token balance
pub async fn get_token_balance(
    context: &mut ProgramTestContext,
    token_account: &Pubkey,
) -> u64 {
    let account = context
        .banks_client
        .get_account(*token_account)
        .await
        .unwrap()
        .unwrap();
    
    let token_account_data = spl_token::state::Account::unpack(&account.data).unwrap();
    token_account_data.amount
}