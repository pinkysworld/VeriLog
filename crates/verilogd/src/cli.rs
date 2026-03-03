use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "verilogd",
    about = "VeriLog single-executable verifiable logging engine"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialize a new store directory (keys, meta, empty logs).
    Init {
        /// Store directory path.
        #[arg(long)]
        store: PathBuf,

        /// Merkle tree height (default 32).
        #[arg(long)]
        tree_height: Option<usize>,
    },

    /// Append a log entry.
    Append {
        #[arg(long)]
        store: PathBuf,

        /// Event kind (e.g. "metric", "event", "alert").
        #[arg(long)]
        kind: String,

        /// Payload bytes. If it starts with '@', the rest is treated as a file path.
        #[arg(long)]
        payload: String,

        /// Log level.
        #[arg(long, default_value = "info")]
        level: String,
    },

    /// Verify the whole store (signatures, hash chain, Merkle roots).
    Verify {
        #[arg(long)]
        store: PathBuf,
    },

    /// Show a concise status summary for a store.
    Status {
        #[arg(long)]
        store: PathBuf,
    },

    /// Export entries as JSON Lines to stdout or a file.
    Export {
        #[arg(long)]
        store: PathBuf,

        /// Output file (defaults to stdout).
        #[arg(long)]
        out: Option<PathBuf>,
    },

    /// Proof utilities.
    Proof {
        #[command(subcommand)]
        command: ProofCommand,
    },

    /// Signed checkpoint snapshots for anchoring/export.
    Checkpoint {
        #[command(subcommand)]
        command: CheckpointCommand,
    },

    /// License utilities (monetization foundation).
    License {
        #[command(subcommand)]
        command: LicenseCommand,
    },

    /// Run the optional admin console (HTTP API/UI).
    Serve {
        #[arg(long)]
        store: PathBuf,

        /// Bind address, e.g. 127.0.0.1:8080
        #[arg(long, default_value = "127.0.0.1:8080")]
        bind: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ProofCommand {
    /// Generate a Merkle membership proof for a leaf index.
    Membership {
        #[arg(long)]
        store: PathBuf,
        #[arg(long)]
        index: u64,
        #[arg(long)]
        out: PathBuf,

        /// Include the entry JSON alongside the proof.
        #[arg(long, default_value_t = false)]
        include_entry: bool,
    },

    /// Verify a Merkle membership proof JSON.
    Verify {
        #[arg(long)]
        proof: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub enum CheckpointCommand {
    /// Create a signed checkpoint for the current store state.
    Create {
        #[arg(long)]
        store: PathBuf,

        /// Optional output file (defaults to stdout).
        #[arg(long)]
        out: Option<PathBuf>,

        /// Optional human-readable label for the checkpoint.
        #[arg(long)]
        label: Option<String>,
    },

    /// Verify a checkpoint JSON file.
    Verify {
        #[arg(long)]
        checkpoint: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub enum LicenseCommand {
    /// Install a license file into the store directory (as license.json).
    Install {
        #[arg(long)]
        store: PathBuf,
        #[arg(long)]
        license: PathBuf,
        /// Optional device id override (otherwise best-effort detection is used).
        #[arg(long)]
        device_id: Option<String>,
        /// Vendor public key (base64). If not provided, uses VERILOG_VENDOR_PUBKEY_B64 env var.
        #[arg(long)]
        vendor_pubkey_b64: Option<String>,
    },

    /// Show license status for a store.
    Status {
        #[arg(long)]
        store: PathBuf,
        /// Optional device id override.
        #[arg(long)]
        device_id: Option<String>,
        /// Vendor public key (base64). If not provided, uses VERILOG_VENDOR_PUBKEY_B64 env var.
        #[arg(long)]
        vendor_pubkey_b64: Option<String>,
    },

    /// Generate a vendor keypair (seed + public key) for signing licenses.
    /// Keep the seed OFFLINE.
    VendorKeygen {
        #[arg(long)]
        out_dir: PathBuf,
    },

    /// Issue a signed license (vendor operation).
    Issue {
        /// Vendor signing seed (base64).
        #[arg(long)]
        vendor_seed_b64: String,

        #[arg(long)]
        issued_to: String,

        #[arg(long, default_value = "")]
        org: String,

        #[arg(long)]
        not_before_unix_ms: u64,

        #[arg(long)]
        not_after_unix_ms: u64,

        /// Optional device binding.
        #[arg(long)]
        device_id: Option<String>,

        /// Repeatable entitlement string, e.g. --entitle zk_integrity_proofs
        #[arg(long, action = clap::ArgAction::Append)]
        entitle: Vec<String>,

        #[arg(long)]
        out: PathBuf,
    },
}
