use super::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    system_instruction,
};
use spl_token_2022::instruction as token_instruction;

#[tokio::test]
async fn test_check_valid_token22() {
    let rpc_client = crate::rpc::test_utils::setup_test_rpc_client();
    
    // Test with invalid token address
    let result = Token22Program::check_valid_token(&rpc_client, "invalid").await;
    assert!(result.is_err());
    
    // Test with non-existent token
    let random_pubkey = Keypair::new().pubkey();
    let result = Token22Program::check_valid_token(&rpc_client, &random_pubkey.to_string()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_decode_token22_instruction() {
    let mint = Pubkey::new_unique();
    let owner = Keypair::new();
    let account = Pubkey::new_unique();
    
    // Test Transfer instruction
    let transfer_ix = token_instruction::transfer(
        &Token22Program::program_id(),
        &account,
        &account,
        &owner.pubkey(),
        &[],
        1000,
    )
    .unwrap();
    
    let result = Token22Program::decode_instruction(&transfer_ix);
    assert!(result.is_ok());
    match result.unwrap() {
        TokenInstruction::Transfer { amount } => {
            assert_eq!(amount, 1000);
        }
        _ => panic!("Expected Transfer instruction"),
    }
    
    // Test invalid instruction data
    let invalid_ix = Instruction {
        program_id: Token22Program::program_id(),
        accounts: vec![],
        data: vec![0xFF; 32], // Invalid data
    };
    
    let result = Token22Program::decode_instruction(&invalid_ix);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_token22_account_data() {
    let rpc_client = crate::rpc::test_utils::setup_test_rpc_client();
    let account = Pubkey::new_unique();
    
    // Test with non-existent account
    let result = Token22Program::get_token_account_data(&rpc_client, &account).await;
    assert!(result.is_err());
} 