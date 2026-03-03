use serde::{Deserialize, Serialize};

use crate::{
    error::VeriLogError,
    hash::{hash_bytes, Hash32},
};

pub const LOG_ENTRY_VERSION: u16 = 1;

/// Log severity level (minimal set).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryUnsigned {
    pub version: u16,
    pub index: u64,
    pub ts_unix_ms: u64,
    pub level: LogLevel,
    pub kind: String,
    /// Application-defined payload. Often JSON bytes.
    pub payload: Vec<u8>,
    /// Hash of the previous entry (hash chain).
    pub prev_entry_hash: Hash32,
    /// Rolling commitment (base track for R09).
    pub window_hash: Hash32,
    /// Fixed-height Merkle root after inserting this entry’s leaf.
    pub merkle_root: Hash32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub unsigned: LogEntryUnsigned,
    /// Ed25519 signature over `entry_hash`.
    #[serde(with = "crate::util::serde_array_64")]
    pub signature: [u8; 64],
}

impl LogEntryUnsigned {
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, VeriLogError> {
        postcard::to_allocvec(self).map_err(VeriLogError::from)
    }

    pub fn entry_hash(&self) -> Result<Hash32, VeriLogError> {
        Ok(hash_bytes(&self.to_canonical_bytes()?))
    }
}

impl LogEntry {
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, VeriLogError> {
        postcard::to_allocvec(self).map_err(VeriLogError::from)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, VeriLogError> {
        postcard::from_bytes(bytes).map_err(VeriLogError::from)
    }

    pub fn entry_hash(&self) -> Result<Hash32, VeriLogError> {
        self.unsigned.entry_hash()
    }
}
