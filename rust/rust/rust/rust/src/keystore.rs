//! Simple but structured key management
//! In production this should be backed by secure enclave / encrypted storage.

use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeystoreError {
    #[error("Key not found")]
    KeyNotFound,
    #[error("Invalid signature")]
    InvalidSignature,
}

pub struct Keystore {
    keys: HashMap<String, SigningKey>,
}

impl Keystore {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    pub fn generate_keypair(&mut self, identity: &str) -> VerifyingKey {
        let signing = SigningKey::generate(&mut OsRng);
        let verifying = signing.verifying_key();
        self.keys.insert(identity.to_string(), signing);
        verifying
    }

    pub fn sign(&self, identity: &str, message: &[u8]) -> Result<Signature, KeystoreError> {
        let key = self.keys.get(identity).ok_or(KeystoreError::KeyNotFound)?;
        Ok(key.sign(message))
    }

    pub fn verify(
        verifying_key: &VerifyingKey,
        message: &[u8],
        signature: &Signature,
    ) -> Result<(), KeystoreError> {
        verifying_key
            .verify(message, signature)
            .map_err(|_| KeystoreError::InvalidSignature)
    }

    pub fn has_key(&self, identity: &str) -> bool {
        self.keys.contains_key(identity)
    }
}
