use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use verilog_enterprise_api::EnterpriseFeature;

use crate::error::LicenseError;

pub const LICENSE_VERSION: u16 = 1;

/// The canonical payload that is signed.
///
/// Signature verification uses `postcard` serialization of this struct,
/// so the signed bytes are deterministic across platforms for the same codebase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensePayload {
    pub version: u16,
    pub license_id: String,
    pub issued_to: String,
    pub org: String,
    pub not_before_unix_ms: u64,
    pub not_after_unix_ms: u64,
    /// Optional device binding (string identifier).
    pub device_id: Option<String>,
    /// Entitled enterprise features.
    pub entitlements: Vec<EnterpriseFeature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedLicense {
    pub payload: LicensePayload,
    /// Base64-encoded Ed25519 signature over postcard(payload).
    pub signature_b64: String,
}

#[derive(Debug, Clone)]
pub struct VerifiedLicense {
    pub payload: LicensePayload,
    pub signature: Signature,
}

impl VerifiedLicense {
    pub fn entitles(&self, feature: EnterpriseFeature) -> bool {
        self.payload.entitlements.iter().any(|f| *f == feature)
    }
}

fn canonical_payload_bytes(payload: &LicensePayload) -> Result<Vec<u8>, LicenseError> {
    postcard::to_allocvec(payload)
        .map_err(|e| LicenseError::Format(format!("postcard encode: {e}")))
}

pub fn issue_license(
    vendor_seed_b64: &str,
    payload: LicensePayload,
) -> Result<SignedLicense, LicenseError> {
    if payload.version != LICENSE_VERSION {
        return Err(LicenseError::Format(format!(
            "unsupported payload.version {}; expected {}",
            payload.version, LICENSE_VERSION
        )));
    }

    let seed = decode_seed_32_b64(vendor_seed_b64)?;
    let sk = SigningKey::from_bytes(&seed);
    let msg = canonical_payload_bytes(&payload)?;
    let sig = sk.sign(&msg);
    let sig_b64 = B64.encode(sig.to_bytes());

    Ok(SignedLicense {
        payload,
        signature_b64: sig_b64,
    })
}

pub fn verify_license(
    vendor_public_key_b64: &str,
    license: &SignedLicense,
    now_unix_ms: u64,
    expected_device_id: Option<&str>,
) -> Result<VerifiedLicense, LicenseError> {
    if license.payload.version != LICENSE_VERSION {
        return Err(LicenseError::Format(format!(
            "unsupported payload.version {}; expected {}",
            license.payload.version, LICENSE_VERSION
        )));
    }

    let vk_bytes = B64.decode(vendor_public_key_b64.trim())?;
    let vk = VerifyingKey::from_bytes(
        vk_bytes
            .as_slice()
            .try_into()
            .map_err(|_| LicenseError::Format("vendor public key must be 32 bytes".into()))?,
    )
    .map_err(|_| LicenseError::Format("invalid vendor public key bytes".into()))?;

    if now_unix_ms < license.payload.not_before_unix_ms {
        return Err(LicenseError::NotYetValid);
    }
    if now_unix_ms > license.payload.not_after_unix_ms {
        return Err(LicenseError::Expired);
    }

    if let Some(bound) = license.payload.device_id.as_deref() {
        let expected = expected_device_id.ok_or(LicenseError::DeviceMismatch)?;
        if bound != expected {
            return Err(LicenseError::DeviceMismatch);
        }
    }

    let sig_bytes = B64.decode(license.signature_b64.trim())?;
    let sig = Signature::from_bytes(
        sig_bytes
            .as_slice()
            .try_into()
            .map_err(|_| LicenseError::Format("signature must be 64 bytes".into()))?,
    );

    let msg = canonical_payload_bytes(&license.payload)?;
    vk.verify_strict(&msg, &sig)
        .map_err(|_| LicenseError::BadSignature)?;

    Ok(VerifiedLicense {
        payload: license.payload.clone(),
        signature: sig,
    })
}

/// Decode a base64-encoded Ed25519 seed (32 bytes).
pub fn decode_seed_32_b64(seed_b64: &str) -> Result<[u8; 32], LicenseError> {
    let seed_bytes = B64.decode(seed_b64.trim())?;
    let seed: [u8; 32] = seed_bytes
        .as_slice()
        .try_into()
        .map_err(|_| LicenseError::Format("seed must be 32 bytes".into()))?;
    Ok(seed)
}

/// Generate a new vendor signing seed and corresponding public key (both base64).
pub fn generate_vendor_keypair_b64() -> (String, String) {
    use rand_core::RngCore;

    let mut seed = [0u8; 32];
    rand_core::OsRng.fill_bytes(&mut seed);
    let sk = SigningKey::from_bytes(&seed);
    let vk = VerifyingKey::from(&sk);

    let seed_b64 = B64.encode(seed);
    let vk_b64 = B64.encode(vk.to_bytes());

    (seed_b64, vk_b64)
}
