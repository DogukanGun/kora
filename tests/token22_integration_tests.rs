use kora_lib::{
    config::ValidationConfig,
    error::KoraError,
    token::{check_valid_token, check_valid_tokens},
    token22::Token22Program,
    transaction::validator::validate_token_payment,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token_2022::instruction as token_instruction;

#[tokio::test]
async fn test_token22_validation() {
    let rpc_client = kora_lib::rpc::test_utils::setup_test_rpc_client();
    let config = ValidationConfig {
        max_allowed_lamports: 1_000_000,
        max_signatures: 10,
        allowed_programs: vec![Token22Program::program_id().to_string()],
        allowed_tokens: vec![],
        allowed_spl_paid_tokens: vec![],
        disallowed_accounts: vec![],
    };

    // Test token validation
    let mint = Keypair::new().pubkey();
    let result = check_valid_token(&rpc_client, &mint.to_string()).await;
    assert!(result.is_err());

    // Test multiple tokens validation
    let tokens = vec![mint.to_string()];
    let result = check_valid_tokens(&rpc_client, &tokens).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_token22_payment_validation() {
    let rpc_client = kora_lib::rpc::test_utils::setup_test_rpc_client();
    let config = ValidationConfig {
        max_allowed_lamports: 1_000_000,
        max_signatures: 10,
        allowed_programs: vec![Token22Program::program_id().to_string()],
        allowed_tokens: vec![],
        allowed_spl_paid_tokens: vec![],
        disallowed_accounts: vec![],
    };

    let payer = Keypair::new();
    let mint = Keypair::new().pubkey();
    let source = Keypair::new().pubkey();
    let destination = Keypair::new().pubkey();

    // Create a Token22 transfer instruction
    let transfer_ix = token_instruction::transfer(
        &Token22Program::program_id(),
        &source,
        &destination,
        &payer.pubkey(),
        &[],
        1000,
    )
    .unwrap();

    let transaction = Transaction::new_with_payer(&[transfer_ix], Some(&payer.pubkey()));

    // Test payment validation
    let result = validate_token_payment(
        &rpc_client,
        &transaction,
        &config,
        1000,
        payer.pubkey(),
        &kora_lib::transaction::TokenPriceInfo { price: 1.0 },
    )
    .await;

    // Should fail because the accounts don't exist in the test environment
    assert!(result.is_err());
} 