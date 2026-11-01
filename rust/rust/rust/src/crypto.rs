//! Cryptographic primitives for YODPAY
//! Ed25519 signatures + Human approval in Extreme mode

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Human approval required in Extreme mode but missing or invalid")]
    HumanApprovalRequired,
}

#[derive(Clone)]
pub struct Keypair {
    pub signing: SigningKey,
    pub verifying: VerifyingKey,
}

impl Keypair {
    pub fn generate() -> Self {
        let signing = SigningKey::generate(&mut OsRng);
        let verifying = signing.verifying_key();
        Self { signing, verifying }
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing.sign(message)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedIntent {
    pub tx_id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub nonce: u64,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

impl SignedIntent {
    pub fn message_bytes(&self) -> Vec<u8> {
        format!(
            "{}:{}:{}:{}:{}:{}",
            self.tx_id, self.from, self.to, self.amount, self.nonce, self.timestamp
        )
        .into_bytes()
    }
}

pub fn require_human_approval_if_extreme(
    mode: &str,
    intent: &SignedIntent,
    human_key: &VerifyingKey,
    human_signature: &Signature,
) -> Result<(), CryptoError> {
    if mode == "Extreme" {
        human_key
            .verify(&intent.message_bytes(), human_signature)
            .map_err(|_| CryptoError::HumanApprovalRequired)?;
    }
    Ok(())
}
