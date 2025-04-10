use base64::{engine::general_purpose::STANDARD as base64_engine, Engine};
use p256::{
    ecdsa::{signature::Signer as _, SigningKey},
    pkcs8::DecodePrivateKey,
};
use solana_sdk::{
    instruction::Instruction,
    message::Message,
    signature::{Signature, SIGNATURE_BYTES},
    transaction::Transaction,
};

use crate::error::KoraError;

pub struct VaultSigner {
    signing_key: SigningKey,
}

impl VaultSigner {
    pub fn new(private_key: &str) -> Result<Self, KoraError> {
        let decoded = base64_engine.decode(private_key).map_err(|e| {
            KoraError::InternalServerError(format!("Failed to decode private key: {}", e))
        })?;

        let signing_key = SigningKey::from_pkcs8_der(&decoded).map_err(|e| {
            KoraError::InternalServerError(format!("Failed to parse private key: {}", e))
        })?;

        Ok(Self { signing_key })
    }
}

impl super::Signer for VaultSigner {
    fn sign_transaction(&self, transaction: &mut Transaction) -> Result<(), KoraError> {
        let message = Message::new(&transaction.message.instructions, None);
        let signature = self
            .signing_key
            .sign(message.serialize().as_slice())
            .to_bytes();
        transaction.signatures[0] = Signature::new(&signature[..SIGNATURE_BYTES]);
        Ok(())
    }

    fn sign_instruction(&self, _instruction: &mut Instruction) -> Result<(), KoraError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::{pubkey::Pubkey, system_instruction};

    fn create_test_transaction() -> Transaction {
        let from = Pubkey::new_unique();
        let to = Pubkey::new_unique();
        let instruction = system_instruction::transfer(&from, &to, 1000);
        Transaction::new_unsigned(Message::new(&[instruction], None))
    }

    // This is a test private key in base64 format - DO NOT USE IN PRODUCTION
    const TEST_PRIVATE_KEY: &str = "MFECAQEwBQYDK2VwBCIEIJ2nXE4RFWaJ0KYAhH1oPVYWHkHZHEtXlgAtElF8ZxHigSEA+hESEYJbRgeBe8ux0LFLrVqHPWVYhwkHBN+3xbjJKQo=";

    #[test]
    fn test_new_valid_key() {
        let result = VaultSigner::new(TEST_PRIVATE_KEY);
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_invalid_base64() {
        let result = VaultSigner::new("invalid-base64");
        assert!(matches!(
            result,
            Err(KoraError::InternalServerError(msg)) if msg.contains("Failed to decode private key")
        ));
    }

    #[test]
    fn test_new_invalid_key_format() {
        // Valid base64 but invalid key format
        let result = VaultSigner::new("aW52YWxpZC1rZXk=");
        assert!(matches!(
            result,
            Err(KoraError::InternalServerError(msg)) if msg.contains("Failed to parse private key")
        ));
    }

    #[test]
    fn test_sign_transaction() {
        let signer = VaultSigner::new(TEST_PRIVATE_KEY).unwrap();
        let mut transaction = create_test_transaction();
        
        // Ensure transaction has space for signature
        transaction.signatures.resize(1, Signature::default());
        
        let result = signer.sign_transaction(&mut transaction);
        assert!(result.is_ok());
        
        // Verify signature is not default
        assert_ne!(transaction.signatures[0], Signature::default());
    }

    #[test]
    fn test_sign_instruction() {
        let signer = VaultSigner::new(TEST_PRIVATE_KEY).unwrap();
        let mut instruction = system_instruction::transfer(&Pubkey::new_unique(), &Pubkey::new_unique(), 1000);
        
        let result = signer.sign_instruction(&mut instruction);
        assert!(result.is_ok());
    }
}
