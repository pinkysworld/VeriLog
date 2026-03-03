use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackStage {
    ShippingPrototype,
    Experimental,
    Planned,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ResearchTrack {
    pub id: &'static str,
    pub title: &'static str,
    pub stage: TrackStage,
    pub edition: &'static str,
    pub horizon: &'static str,
    pub summary: &'static str,
    pub next_milestone: &'static str,
}

const TRACKS: [ResearchTrack; 8] = [
    ResearchTrack {
        id: "r04",
        title: "Merkle-based verifiable audit trails",
        stage: TrackStage::ShippingPrototype,
        edition: "base",
        horizon: "now",
        summary: "Incremental frontier, full-store verification, and membership proofs are already available in the OSS build.",
        next_milestone: "Add indexed proofs and formal format vectors.",
    },
    ResearchTrack {
        id: "r09",
        title: "Sliding-window tamper evidence",
        stage: TrackStage::ShippingPrototype,
        edition: "base",
        horizon: "now",
        summary: "Each record carries the previous signed entry hash and a rolling window commitment for stronger forensic reconstruction.",
        next_milestone: "Add forward-secure ratcheting to contain post-compromise blast radius.",
    },
    ResearchTrack {
        id: "r10",
        title: "Long-term archival checkpoints",
        stage: TrackStage::ShippingPrototype,
        edition: "base",
        horizon: "now",
        summary: "Signed checkpoints make it practical to anchor roots externally without exposing private key material.",
        next_milestone: "Publish checkpoint envelopes and third-party verifier fixtures.",
    },
    ResearchTrack {
        id: "r03",
        title: "Per-event differential privacy",
        stage: TrackStage::Experimental,
        edition: "base-plus",
        horizon: "next",
        summary: "Numeric telemetry can already be noise-protected, but the project still needs schema packs and richer accounting.",
        next_milestone: "Ship audited schemas for common metric payloads.",
    },
    ResearchTrack {
        id: "r01",
        title: "Adaptive energy-aware logging",
        stage: TrackStage::Experimental,
        edition: "base-plus",
        horizon: "next",
        summary: "A rule-based scheduler exists today and forms the control for later TinyML policy experiments.",
        next_milestone: "Add simulation traces and benchmark the policy across device classes.",
    },
    ResearchTrack {
        id: "r05",
        title: "Energy-aware verifiable compression",
        stage: TrackStage::Planned,
        edition: "base-plus",
        horizon: "later",
        summary: "Compression is the most immediate way to lower write amplification without sacrificing auditability.",
        next_milestone: "Prototype deterministic payload compression with hash-stable envelopes.",
    },
    ResearchTrack {
        id: "r02",
        title: "Zero-knowledge log integrity",
        stage: TrackStage::Planned,
        edition: "enterprise",
        horizon: "later",
        summary: "The novel long-term differentiator is proving log integrity over ranges without disclosing raw events.",
        next_milestone: "Run a parallel Poseidon-commitment experiment before circuit work.",
    },
    ResearchTrack {
        id: "r08",
        title: "Privacy-preserving cross-device correlation",
        stage: TrackStage::Planned,
        edition: "enterprise",
        horizon: "later",
        summary: "This is where VeriLog moves from secure device logging into collaborative evidence graphs.",
        next_milestone: "Define transcript shapes and event alignment primitives.",
    },
];

pub fn tracks() -> &'static [ResearchTrack] {
    &TRACKS
}

pub fn track(id: &str) -> Option<&'static ResearchTrack> {
    TRACKS.iter().find(|track| track.id == id)
}
