use std::path::{Path, PathBuf};

use crate::{error::LicenseError, license::SignedLicense};

/// A simple file-based license store.
///
/// For embedded/edge usage we store the license alongside the log store directory.
#[derive(Debug, Clone)]
pub struct LicenseStore {
    path: PathBuf,
}

impl LicenseStore {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<Option<SignedLicense>, LicenseError> {
        if !self.path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(&self.path)?;
        let lic: SignedLicense = serde_json::from_slice(&bytes)?;
        Ok(Some(lic))
    }

    pub fn save(&self, lic: &SignedLicense) -> Result<(), LicenseError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(lic)?;
        std::fs::write(&self.path, bytes)?;
        Ok(())
    }
}
