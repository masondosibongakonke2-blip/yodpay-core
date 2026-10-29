//! Cryptographic signatures (Ed25519) + Extreme-mode human key approval

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Human approval required in Extreme mode")]
    HumanApprovalRequired,
    #[error("Key error: {0}")]
    KeyError(String),
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

    pub fn sign(&self, msg: &[u8]) -> Signature {
        self.signing.sign(msg)
    }

    pub fn verify(verifying: &VerifyingKey, msg: &[u8], sig: &Signature) -> Result<(), CryptoError> {
        verifying.verify(msg, sig).map_err(|_| CryptoError::InvalidSignature)
    }
}

/// Payload that must be signed for any state transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedIntent {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub nonce: u64,
    pub mode: String, // "Normal" | "Partitioned" | "Extreme"
}

/// In Extreme (1f+1) mode, require an additional human-held key signature
pub fn require_human_approval_if_extreme(
    mode_is_extreme: bool,
    human_sig_present: bool,
) -> Result<(), CryptoError> {
    if mode_is_extreme && !human_sig_present {
        return Err(CryptoError::HumanApprovalRequired);
    }
    Ok(())
  }
