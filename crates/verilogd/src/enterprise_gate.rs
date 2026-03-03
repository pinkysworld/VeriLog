use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use verilog_enterprise_api::EnterpriseFeature;
use verilog_license::{verify_license, DeviceId, LicenseStore, VerifiedLicense};

/// Helper to gate enterprise-only commands based on an installed license.
///
/// This is intentionally kept in the `verilogd` binary crate so the core engine stays usable
/// without any licensing concerns.
pub struct EnterpriseGate {
    store_dir: PathBuf,
    verified: Option<VerifiedLicense>,
    device_id: String,
}

impl EnterpriseGate {
    pub fn load(
        store_dir: impl AsRef<Path>,
        vendor_pubkey_b64: &str,
        device_id: Option<String>,
    ) -> Result<Self> {
        let store_dir = store_dir.as_ref().to_path_buf();
        let device_id = device_id
            .or_else(DeviceId::detect_best_effort)
            .unwrap_or_else(|| DeviceId::random_hex_128());

        let lic_path = store_dir.join(verilog_core::storage::LICENSE_FILE);
        let ls = LicenseStore::new(lic_path);
        let lic = ls.load().context("load license")?;

        let verified = if let Some(lic) = lic {
            let now = verilog_core::util::now_unix_ms();
            Some(verify_license(
                vendor_pubkey_b64,
                &lic,
                now,
                Some(&device_id),
            )?)
        } else {
            None
        };

        Ok(Self {
            store_dir,
            verified,
            device_id,
        })
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    pub fn is_entitled(&self, feature: EnterpriseFeature) -> bool {
        self.verified
            .as_ref()
            .map(|v| v.entitles(feature))
            .unwrap_or(false)
    }

    pub fn require(&self, feature: EnterpriseFeature) -> Result<()> {
        if self.is_entitled(feature) {
            Ok(())
        } else {
            Err(anyhow!(
                "enterprise feature not entitled: {} (store: {}, device_id: {})",
                feature,
                self.store_dir.display(),
                self.device_id
            ))
        }
    }
}
