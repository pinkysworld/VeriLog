use core::fmt;
use serde::{Deserialize, Serialize};

/// Feature identifiers for enterprise-only capabilities.
///
/// These IDs are referenced in license entitlements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnterpriseFeature {
    /// Zero-knowledge proof of log integrity over a time range.
    ZkIntegrityProofs,

    /// Compliance export bundles (GDPR/HIPAA/SOC2 templates).
    ComplianceExports,

    /// Cross-device privacy-preserving correlation (PSI/ZK).
    CrossDeviceCorrelation,

    /// Secure log forwarding with ratchets and delivery/order guarantees.
    SecureForwarding,

    /// Verifiable mesh synchronization with proofs.
    MeshSync,

    /// Zero-knowledge predicate/range queries on logs.
    ZkRangeQueries,

    /// Wasm-based user-defined logging rules (sandbox).
    WasmRules,

    /// Learned energy-aware policies beyond the base rule-based scheduler.
    AdvancedEnergyMl,

    /// Adaptive encryption policy proofs / enforcement.
    EncryptionPolicyProofs,

    /// Federated telemetry aggregation with DP + ZK.
    TelemetryFederation,

    /// Oblivious read/access pattern hiding.
    ObliviousReads,
}

impl EnterpriseFeature {
    pub fn as_str(&self) -> &'static str {
        match self {
            EnterpriseFeature::ZkIntegrityProofs => "zk_integrity_proofs",
            EnterpriseFeature::ComplianceExports => "compliance_exports",
            EnterpriseFeature::CrossDeviceCorrelation => "cross_device_correlation",
            EnterpriseFeature::SecureForwarding => "secure_forwarding",
            EnterpriseFeature::MeshSync => "mesh_sync",
            EnterpriseFeature::ZkRangeQueries => "zk_range_queries",
            EnterpriseFeature::WasmRules => "wasm_rules",
            EnterpriseFeature::AdvancedEnergyMl => "advanced_energy_ml",
            EnterpriseFeature::EncryptionPolicyProofs => "encryption_policy_proofs",
            EnterpriseFeature::TelemetryFederation => "telemetry_federation",
            EnterpriseFeature::ObliviousReads => "oblivious_reads",
        }
    }
}

impl fmt::Display for EnterpriseFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
