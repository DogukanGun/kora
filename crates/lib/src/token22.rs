use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction,
    program_pack::Pack,
    pubkey::Pubkey,
    transaction::Transaction,
};
use spl_token_2022::{
    self,
    instruction::{decode_instruction_type, decode_instruction_data, TokenInstruction},
    state::{Account as TokenAccount, Mint},
};
use std::str::FromStr;

use crate::error::KoraError;
use crate::token::TokenProgram;

pub struct Token22Program;

impl TokenProgram for Token22Program {
    fn program_id() -> Pubkey {
        spl_token_2022::id()
    }

    fn decode_instruction(instruction: &Instruction) -> Result<TokenInstruction, KoraError> {
        let instruction_type = decode_instruction_type(&instruction.data)
            .map_err(|e| KoraError::InvalidTransaction(format!("Invalid Token22 instruction: {}", e)))?;
        
        decode_instruction_data(&instruction.data)
            .map_err(|e| KoraError::InvalidTransaction(format!("Invalid Token22 instruction data: {}", e)))
    }

    async fn check_valid_token(rpc_client: &RpcClient, token: &str) -> Result<(), KoraError> {
        let pubkey = Pubkey::from_str(token)
            .map_err(|e| KoraError::InternalServerError(format!("Invalid token address: {}", e)))?;

        // Check if the account exists and is a Token22 mint account
        match rpc_client.get_account(&pubkey).await {
            Ok(account) => {
                if account.owner == Self::program_id() {
                    // Try to unpack as a Token22 mint to verify it's a valid mint account
                    Mint::unpack(&account.data)
                        .map_err(|_| KoraError::InternalServerError(format!(
                            "Token {} is not a valid Token22 mint",
                            token
                        )))?;
                    Ok(())
                } else {
                    Err(KoraError::InternalServerError(format!(
                        "Token {} is not a valid Token22 mint",
                        token
                    )))
                }
            }
            Err(e) => {
                Err(KoraError::InternalServerError(format!("Token {} does not exist: {}", token, e)))
            }
        }
    }

    async fn get_token_account_data(
        rpc_client: &RpcClient,
        token_account: &Pubkey,
    ) -> Result<TokenAccount, KoraError> {
        let account = rpc_client
            .get_account(token_account)
            .await
            .map_err(|e| KoraError::RpcError(e.to_string()))?;

        if account.owner != Self::program_id() {
            return Err(KoraError::InvalidTransaction(format!(
                "Account {} is not a Token22 account",
                token_account
            )));
        }

        TokenAccount::unpack(&account.data).map_err(|e| {
            KoraError::InvalidTransaction(format!("Invalid Token22 account data: {}", e))
        })
    }
} 