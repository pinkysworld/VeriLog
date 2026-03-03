use serde::{Deserialize, Serialize};

use crate::{EnterpriseError, EnterpriseFeature};

/// Public inputs for a ZK integrity proof over a log range.
///
/// The exact semantics are defined by the enterprise implementation.
/// A typical statement is:
/// - there exists a sequence of valid, signed, hash-chained entries
/// - whose Merkle commitment transitions from start_root to end_root
/// - covering indices [start_index, end_index] (inclusive/exclusive as specified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityRangeStatement {
    pub tree_height: u8,
    pub start_index: u64,
    pub end_index: u64,
    pub start_root: [u8; 32],
    pub end_root: [u8; 32],
}

/// A proof bundle produced by an enterprise ZK engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProofBundle {
    pub statement: IntegrityRangeStatement,
    /// Opaque proof bytes (Halo2/Plonky3/etc).
    pub proof: Vec<u8>,
}

/// Enterprise trait for ZK integrity proofs.
pub trait ZkIntegrityProver: Send + Sync {
    fn prove_integrity_range(
        &self,
        statement: IntegrityRangeStatement,
        // enterprise implementations may take additional private witnesses
    ) -> Result<ZkProofBundle, EnterpriseError>;

    fn verify_integrity_range(&self, bundle: &ZkProofBundle) -> Result<bool, EnterpriseError>;
}

/// Enterprise trait for compliance exports.
pub trait ComplianceExporter: Send + Sync {
    fn export_bundle(&self, kind: &str, payload: &[u8]) -> Result<Vec<u8>, EnterpriseError>;
}

/// The top-level enterprise module that can provide optional capability implementations.
///
/// In the OSS base edition, this trait is present but not implemented by default.
/// Private builds can supply an implementation from a proprietary crate.
pub trait EnterpriseModule: Send + Sync {
    /// Returns true if the module provides the given feature.
    fn supports(&self, feature: EnterpriseFeature) -> bool;

    fn zk_integrity_prover(&self) -> Option<&dyn ZkIntegrityProver> {
        None
    }

    fn compliance_exporter(&self) -> Option<&dyn ComplianceExporter> {
        None
    }
}
