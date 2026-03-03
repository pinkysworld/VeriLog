use rand_core::RngCore;

/// Best-effort device identifier helper.
///
/// In production you may want to bind to secure hardware IDs or attestation.
/// This base helper uses:
/// 1) VERILOG_DEVICE_ID env var if set
/// 2) /etc/machine-id if present (Linux)
/// 3) random fallback (not stable across wipes unless persisted by caller)
pub struct DeviceId;

impl DeviceId {
    pub fn detect_best_effort() -> Option<String> {
        if let Ok(v) = std::env::var("VERILOG_DEVICE_ID") {
            let v = v.trim().to_string();
            if !v.is_empty() {
                return Some(v);
            }
        }

        if let Ok(s) = std::fs::read_to_string("/etc/machine-id") {
            let id = s.lines().next().unwrap_or("").trim().to_string();
            if !id.is_empty() {
                return Some(id);
            }
        }

        None
    }

    pub fn random_hex_128() -> String {
        let mut bytes = [0u8; 16];
        rand_core::OsRng.fill_bytes(&mut bytes);
        hex::encode(bytes)
    }
}
