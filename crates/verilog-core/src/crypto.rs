use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::VeriLogError;

pub const KEY_FILE_VERSION: u16 = 1;

#[derive(Debug, Clone)]
pub struct Keypair {
    signing: SigningKey,
    verifying: VerifyingKey,
    signing_seed: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredKeypair {
    pub version: u16,
    pub signing_seed_b64: String,
    pub verifying_key_b64: String,
}

impl Keypair {
    pub fn generate() -> Self {
        let mut seed = [0u8; 32];
        rand_core::OsRng.fill_bytes(&mut seed);
        let signing = SigningKey::from_bytes(&seed);
        let verifying = VerifyingKey::from(&signing);
        Self {
            signing,
            verifying,
            signing_seed: seed,
        }
    }

    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.verifying
    }

    pub fn sign(&self, msg: &[u8]) -> Signature {
        self.signing.sign(msg)
    }

    pub fn verify(&self, msg: &[u8], sig: &Signature) -> Result<(), VeriLogError> {
        self.verifying
            .verify_strict(msg, sig)
            .map_err(|e| VeriLogError::Signature(format!("{e}")))
    }

    pub fn to_stored(&self) -> StoredKeypair {
        StoredKeypair {
            version: KEY_FILE_VERSION,
            signing_seed_b64: B64.encode(self.signing_seed),
            verifying_key_b64: B64.encode(self.verifying.to_bytes()),
        }
    }

    pub fn from_stored(stored: &StoredKeypair) -> Result<Self, VeriLogError> {
        if stored.version != KEY_FILE_VERSION {
            return Err(VeriLogError::Format(format!(
                "unsupported key file version {}",
                stored.version
            )));
        }
        let seed_bytes = B64
            .decode(stored.signing_seed_b64.trim())
            .map_err(|e| VeriLogError::Format(format!("base64 decode signing seed: {e}")))?;
        let seed: [u8; 32] = seed_bytes
            .as_slice()
            .try_into()
            .map_err(|_| VeriLogError::Format("signing seed must be 32 bytes".into()))?;

        let signing = SigningKey::from_bytes(&seed);
        let verifying = VerifyingKey::from(&signing);

        Ok(Self {
            signing,
            verifying,
            signing_seed: seed,
        })
    }

    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<(), VeriLogError> {
        let stored = self.to_stored();
        let bytes = serde_json::to_vec_pretty(&stored)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    pub fn load_json(path: impl AsRef<Path>) -> Result<Self, VeriLogError> {
        let bytes = std::fs::read(path)?;
        let stored: StoredKeypair = serde_json::from_slice(&bytes)?;
        Self::from_stored(&stored)
    }
}
