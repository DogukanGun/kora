use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use spl_token::instruction::TokenInstruction;
use std::str::FromStr;

use crate::{error::KoraError, token22::Token22Program};

pub trait TokenProgram {
    fn program_id() -> Pubkey;
    fn decode_instruction(instruction: &Instruction) -> Result<TokenInstruction, KoraError>;
    async fn check_valid_token(rpc_client: &RpcClient, token: &str) -> Result<(), KoraError>;
    async fn get_token_account_data(
        rpc_client: &RpcClient,
        token_account: &Pubkey,
    ) -> Result<spl_token::state::Account, KoraError>;
}

pub async fn check_valid_token(rpc_client: &RpcClient, token: &str) -> Result<(), KoraError> {
    // Try Token22 first
    match Token22Program::check_valid_token(rpc_client, token).await {
        Ok(_) => return Ok(()),
        Err(_) => {
            // If Token22 fails, try regular SPL token
            match rpc_client.get_account(&Pubkey::from_str(token)?).await {
                Ok(account) => {
                    if account.owner == spl_token::id() {
                        Ok(())
                    } else {
                        Err(KoraError::InternalServerError(format!(
                            "Token {} is not a valid SPL token mint",
                            token
                        )))
                    }
                }
                Err(e) => {
                    Err(KoraError::InternalServerError(format!("Token {} does not exist: {}", token, e)))
                }
            }
        }
    }
}

pub async fn check_valid_tokens(
    rpc_client: &RpcClient,
    tokens: &[String],
) -> Result<(), KoraError> {
    for token in tokens {
        check_valid_token(rpc_client, token).await?;
    }
    Ok(())
}

pub fn is_token_program(program_id: &Pubkey) -> bool {
    *program_id == spl_token::id() || *program_id == Token22Program::program_id()
}

pub fn decode_token_instruction(instruction: &Instruction) -> Result<TokenInstruction, KoraError> {
    if instruction.program_id == Token22Program::program_id() {
        Token22Program::decode_instruction(instruction)
    } else if instruction.program_id == spl_token::id() {
        spl_token::instruction::TokenInstruction::unpack(&instruction.data)
            .map_err(|e| KoraError::InvalidTransaction(format!("Invalid token instruction: {}", e)))
    } else {
        Err(KoraError::InvalidTransaction("Not a token program instruction".to_string()))
    }
}
