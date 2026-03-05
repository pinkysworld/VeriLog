use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::{
    crypto::Keypair,
    entry::{LogEntry, LogEntryUnsigned, LogLevel, LOG_ENTRY_VERSION},
    error::VeriLogError,
    hash::{hash32_from_hex, hash32_to_hex, hash_bytes, hash_pair, Hash32},
    merkle::{
        membership_proof_from_leaves, MerkleFrontier, MerkleProof, MerkleSnapshot,
        DEFAULT_TREE_HEIGHT,
    },
    util::now_unix_ms,
};

pub const META_VERSION: u16 = 1;
pub const ENTRIES_FILE: &str = "entries.bin";
pub const LEAVES_FILE: &str = "leaves.bin";
pub const META_FILE: &str = "meta.json";
pub const KEY_FILE: &str = "signing_key.json";
pub const LICENSE_FILE: &str = "license.json";
pub const CHECKPOINT_VERSION: u16 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetaFile {
    pub version: u16,
    pub tree_height: u8,
    pub leaf_count: u64,
    pub prev_entry_hash_hex: String,
    pub window_hash_hex: String,
    pub frontier_hex: Vec<Option<String>>,
    pub created_at_unix_ms: u64,
}

impl MetaFile {
    fn new_empty(tree_height: usize) -> Self {
        Self {
            version: META_VERSION,
            tree_height: tree_height as u8,
            leaf_count: 0,
            prev_entry_hash_hex: hash32_to_hex(&[0u8; 32]),
            window_hash_hex: hash32_to_hex(&[0u8; 32]),
            frontier_hex: vec![None; tree_height],
            created_at_unix_ms: now_unix_ms(),
        }
    }
}

#[derive(Debug)]
pub struct LogStore {
    dir: PathBuf,
    keypair: Keypair,
    tree: MerkleFrontier,
    prev_entry_hash: Hash32,
    window_hash: Hash32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyReport {
    pub ok: bool,
    pub leaf_count: u64,
    pub final_root_hex: String,
    pub last_entry_hash_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendResult {
    pub entry: LogEntry,
    pub entry_hash_hex: String,
    pub leaf_hash_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointUnsigned {
    pub version: u16,
    pub tree_height: u8,
    pub leaf_count: u64,
    pub created_at_unix_ms: u64,
    pub current_root: Hash32,
    pub last_entry_hash: Hash32,
    pub verifying_key_b64: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedCheckpoint {
    pub checkpoint: CheckpointUnsigned,
    #[serde(with = "crate::util::serde_array_64")]
    pub signature: [u8; 64],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LeafPreimage {
    pub version: u16,
    pub index: u64,
    pub ts_unix_ms: u64,
    pub level: LogLevel,
    pub kind: String,
    pub payload: Vec<u8>,
    pub prev_entry_hash: Hash32,
    pub window_hash: Hash32,
}

impl From<&LogEntryUnsigned> for LeafPreimage {
    fn from(u: &LogEntryUnsigned) -> Self {
        Self {
            version: u.version,
            index: u.index,
            ts_unix_ms: u.ts_unix_ms,
            level: u.level,
            kind: u.kind.clone(),
            payload: u.payload.clone(),
            prev_entry_hash: u.prev_entry_hash,
            window_hash: u.window_hash,
        }
    }
}

fn leaf_hash_from_unsigned(u: &LogEntryUnsigned) -> Result<Hash32, VeriLogError> {
    let pre: LeafPreimage = u.into();
    let bytes = postcard::to_allocvec(&pre)?;
    Ok(hash_bytes(&bytes))
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), VeriLogError> {
    let tmp = path.with_extension("tmp");
    {
        let f = std::fs::File::create(&tmp)?;
        let mut w = std::io::BufWriter::new(f);
        w.write_all(bytes)?;
        w.flush()?;
        // Ensure data reaches durable storage before rename.
        w.get_ref().sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    // Sync the parent directory so the rename is durable.
    if let Some(parent) = path.parent() {
        if let Ok(d) = std::fs::File::open(parent) {
            let _ = d.sync_all();
        }
    }
    Ok(())
}

fn read_len_prefixed<R: Read>(r: &mut R) -> Result<Option<Vec<u8>>, VeriLogError> {
    let mut len_buf = [0u8; 4];
    match r.read_exact(&mut len_buf) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(VeriLogError::Io(e)),
    }
    let len = u32::from_le_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    Ok(Some(buf))
}

fn write_len_prefixed<W: Write>(w: &mut W, bytes: &[u8]) -> Result<(), VeriLogError> {
    let len = bytes.len();
    if len > (u32::MAX as usize) {
        return Err(VeriLogError::Format("record too large".into()));
    }
    w.write_all(&(len as u32).to_le_bytes())?;
    w.write_all(bytes)?;
    Ok(())
}

impl CheckpointUnsigned {
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, VeriLogError> {
        postcard::to_allocvec(self).map_err(VeriLogError::from)
    }

    pub fn checkpoint_hash(&self) -> Result<Hash32, VeriLogError> {
        Ok(hash_bytes(&self.to_canonical_bytes()?))
    }
}

impl SignedCheckpoint {
    pub fn verify(&self) -> Result<(), VeriLogError> {
        let checkpoint_hash = self.checkpoint.checkpoint_hash()?;
        let key_bytes = B64
            .decode(self.checkpoint.verifying_key_b64.trim())
            .map_err(|e| VeriLogError::Format(format!("base64 decode verifying key: {e}")))?;
        let key_bytes: [u8; 32] = key_bytes
            .as_slice()
            .try_into()
            .map_err(|_| VeriLogError::Format("verifying key must be 32 bytes".into()))?;
        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&key_bytes)
            .map_err(|e| VeriLogError::Format(format!("invalid verifying key: {e}")))?;
        let sig = ed25519_dalek::Signature::from_bytes(&self.signature);
        verifying_key
            .verify_strict(&checkpoint_hash, &sig)
            .map_err(|e| VeriLogError::Signature(format!("{e}")))
    }
}

impl LogStore {
    pub fn init(dir: impl AsRef<Path>, tree_height: Option<usize>) -> Result<(), VeriLogError> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)?;

        let height = tree_height.unwrap_or(DEFAULT_TREE_HEIGHT);
        if height == 0 || height > 63 {
            return Err(VeriLogError::Format(
                "tree_height must be in [1, 63]".into(),
            ));
        }

        // Generate signing keys.
        let keypair = Keypair::generate();
        keypair.save_json(dir.join(KEY_FILE))?;

        // Create empty files.
        std::fs::write(dir.join(ENTRIES_FILE), &[])?;
        std::fs::write(dir.join(LEAVES_FILE), &[])?;

        // Create meta.
        let meta = MetaFile::new_empty(height);
        let meta_bytes = serde_json::to_vec_pretty(&meta)?;
        write_atomic(&dir.join(META_FILE), &meta_bytes)?;

        Ok(())
    }

    pub fn open(dir: impl AsRef<Path>) -> Result<Self, VeriLogError> {
        let dir = dir.as_ref().to_path_buf();
        let keypair = Keypair::load_json(dir.join(KEY_FILE))?;

        let meta_bytes = std::fs::read(dir.join(META_FILE))?;
        let meta: MetaFile = serde_json::from_slice(&meta_bytes)?;

        if meta.version != META_VERSION {
            return Err(VeriLogError::Format(format!(
                "unsupported meta version {}",
                meta.version
            )));
        }

        let tree_height = meta.tree_height as usize;
        if meta.frontier_hex.len() != tree_height {
            return Err(VeriLogError::Format(format!(
                "frontier_hex len {} != tree_height {}",
                meta.frontier_hex.len(),
                tree_height
            )));
        }

        let mut frontier: Vec<Option<Hash32>> = Vec::with_capacity(tree_height);
        for opt in meta.frontier_hex.iter() {
            match opt {
                None => frontier.push(None),
                Some(hexs) => frontier.push(Some(
                    hash32_from_hex(hexs).map_err(|e| VeriLogError::Format(format!("{e}")))?,
                )),
            }
        }

        let snapshot = MerkleSnapshot {
            tree_height: meta.tree_height,
            leaf_count: meta.leaf_count,
            frontier,
        };
        let tree = MerkleFrontier::from_snapshot(snapshot)?;

        let prev_entry_hash = hash32_from_hex(&meta.prev_entry_hash_hex)
            .map_err(|e| VeriLogError::Format(format!("{e}")))?;
        let window_hash = hash32_from_hex(&meta.window_hash_hex)
            .map_err(|e| VeriLogError::Format(format!("{e}")))?;

        Ok(Self {
            dir,
            keypair,
            tree,
            prev_entry_hash,
            window_hash,
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn leaf_count(&self) -> u64 {
        self.tree.leaf_count()
    }

    pub fn tree_height(&self) -> usize {
        self.tree.tree_height()
    }

    pub fn current_root(&self) -> Result<Hash32, VeriLogError> {
        self.tree.root()
    }

    pub fn verifying_key_b64(&self) -> String {
        base64::engine::general_purpose::STANDARD.encode(self.keypair.verifying_key().to_bytes())
    }

    pub fn create_checkpoint(
        &self,
        label: Option<String>,
    ) -> Result<SignedCheckpoint, VeriLogError> {
        let current_root = self.current_root()?;
        let last_entry_hash = match self.last_entry()? {
            Some(entry) => entry.entry_hash()?,
            None => [0u8; 32],
        };

        let checkpoint = CheckpointUnsigned {
            version: CHECKPOINT_VERSION,
            tree_height: self.tree.tree_height() as u8,
            leaf_count: self.tree.leaf_count(),
            created_at_unix_ms: now_unix_ms(),
            current_root,
            last_entry_hash,
            verifying_key_b64: self.verifying_key_b64(),
            label,
        };

        let checkpoint_hash = checkpoint.checkpoint_hash()?;
        let signature = self.keypair.sign(&checkpoint_hash).to_bytes();

        Ok(SignedCheckpoint {
            checkpoint,
            signature,
        })
    }

    fn persist_meta(&self) -> Result<(), VeriLogError> {
        let snap = self.tree.snapshot();
        let mut frontier_hex = Vec::with_capacity(snap.frontier.len());
        for opt in snap.frontier.iter() {
            frontier_hex.push(opt.map(|h| hash32_to_hex(&h)));
        }
        let meta = MetaFile {
            version: META_VERSION,
            tree_height: snap.tree_height,
            leaf_count: snap.leaf_count,
            prev_entry_hash_hex: hash32_to_hex(&self.prev_entry_hash),
            window_hash_hex: hash32_to_hex(&self.window_hash),
            frontier_hex,
            created_at_unix_ms: now_unix_ms(), // updated on write; you may store separately if needed
        };
        let bytes = serde_json::to_vec_pretty(&meta)?;
        write_atomic(&self.dir.join(META_FILE), &bytes)?;
        Ok(())
    }

    pub fn append(
        &mut self,
        kind: impl Into<String>,
        payload: Vec<u8>,
        level: LogLevel,
    ) -> Result<AppendResult, VeriLogError> {
        let kind = kind.into();
        let index = self.tree.leaf_count();
        let ts_unix_ms = now_unix_ms();

        // Rolling commitment uses previous signed entry hash.
        let window_hash = hash_pair(&self.window_hash, &self.prev_entry_hash);

        // Build unsigned with placeholder root (will be updated after leaf hash push).
        let mut unsigned = LogEntryUnsigned {
            version: LOG_ENTRY_VERSION,
            index,
            ts_unix_ms,
            level,
            kind,
            payload,
            prev_entry_hash: self.prev_entry_hash,
            window_hash,
            merkle_root: [0u8; 32],
        };

        // Leaf hash excludes merkle_root/signature to avoid circularity.
        let leaf_hash = leaf_hash_from_unsigned(&unsigned)?;

        // Update Merkle frontier with the leaf commitment.
        let new_root = self.tree.push(leaf_hash)?;
        unsigned.merkle_root = new_root;

        // Compute signed entry hash.
        let entry_hash = unsigned.entry_hash()?;
        let sig = self.keypair.sign(&entry_hash);
        let entry = LogEntry {
            unsigned,
            signature: sig.to_bytes(),
        };

        // Append to entries.bin with durable fsync.
        let rec = entry.to_canonical_bytes()?;
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.dir.join(ENTRIES_FILE))?;
        write_len_prefixed(&mut f, &rec)?;
        f.flush()?;
        f.sync_all()?;

        // Append leaf hash to leaves.bin with durable fsync.
        let mut lf = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.dir.join(LEAVES_FILE))?;
        lf.write_all(&leaf_hash)?;
        lf.flush()?;
        lf.sync_all()?;

        // Update in-memory meta state.
        self.prev_entry_hash = entry_hash;
        self.window_hash = window_hash;

        // Persist meta.
        self.persist_meta()?;

        Ok(AppendResult {
            entry,
            entry_hash_hex: hash32_to_hex(&entry_hash),
            leaf_hash_hex: hash32_to_hex(&leaf_hash),
        })
    }

    pub fn iter_entries(&self) -> Result<Vec<LogEntry>, VeriLogError> {
        let mut f = OpenOptions::new()
            .read(true)
            .open(self.dir.join(ENTRIES_FILE))?;
        let mut out = Vec::new();
        while let Some(rec) = read_len_prefixed(&mut f)? {
            let e = LogEntry::from_bytes(&rec)?;
            out.push(e);
        }
        Ok(out)
    }

    pub fn verify_store(&self) -> Result<VerifyReport, VeriLogError> {
        let mut f = OpenOptions::new()
            .read(true)
            .open(self.dir.join(ENTRIES_FILE))?;

        let mut expected_prev: Hash32 = [0u8; 32];
        let mut window_state: Hash32 = [0u8; 32];
        let mut tree = MerkleFrontier::new(self.tree.tree_height());
        let mut leaf_count: u64 = 0;

        while let Some(rec) = read_len_prefixed(&mut f)? {
            let entry = LogEntry::from_bytes(&rec)?;
            if entry.unsigned.index != leaf_count {
                return Err(VeriLogError::Integrity(format!(
                    "index mismatch: got {}, expected {}",
                    entry.unsigned.index, leaf_count
                )));
            }
            if entry.unsigned.prev_entry_hash != expected_prev {
                return Err(VeriLogError::Integrity(format!(
                    "prev_entry_hash mismatch at index {}",
                    leaf_count
                )));
            }

            let expected_window = hash_pair(&window_state, &expected_prev);
            if entry.unsigned.window_hash != expected_window {
                return Err(VeriLogError::Integrity(format!(
                    "window_hash mismatch at index {}",
                    leaf_count
                )));
            }
            window_state = expected_window;

            // Recompute leaf hash and Merkle root.
            let leaf_hash = leaf_hash_from_unsigned(&entry.unsigned)?;
            let root = tree.push(leaf_hash)?;
            if entry.unsigned.merkle_root != root {
                return Err(VeriLogError::Integrity(format!(
                    "merkle_root mismatch at index {}",
                    leaf_count
                )));
            }

            // Verify signature.
            let entry_hash = entry.unsigned.entry_hash()?;
            let sig = ed25519_dalek::Signature::from_bytes(&entry.signature);
            self.keypair.verify(&entry_hash, &sig)?;

            expected_prev = entry_hash;
            leaf_count += 1;
        }

        let final_root = tree.root()?;
        Ok(VerifyReport {
            ok: true,
            leaf_count,
            final_root_hex: hash32_to_hex(&final_root),
            last_entry_hash_hex: hash32_to_hex(&expected_prev),
        })
    }

    pub fn read_leaf_hashes(&self) -> Result<Vec<Hash32>, VeriLogError> {
        let mut f = OpenOptions::new()
            .read(true)
            .open(self.dir.join(LEAVES_FILE))?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        if buf.len() % 32 != 0 {
            return Err(VeriLogError::Format(
                "leaves.bin length not multiple of 32".into(),
            ));
        }
        let mut leaves = Vec::with_capacity(buf.len() / 32);
        for chunk in buf.chunks_exact(32) {
            leaves.push(chunk.try_into().unwrap());
        }
        Ok(leaves)
    }

    pub fn membership_proof(&self, leaf_index: u64) -> Result<MerkleProof, VeriLogError> {
        let leaves = self.read_leaf_hashes()?;
        let proof = membership_proof_from_leaves(&leaves, self.tree.tree_height(), leaf_index)?;
        Ok(proof)
    }

    /// Export as JSON Lines: one JSON object per entry.
    pub fn export_json_lines<W: Write>(&self, mut out: W) -> Result<(), VeriLogError> {
        let mut f = OpenOptions::new()
            .read(true)
            .open(self.dir.join(ENTRIES_FILE))?;
        while let Some(rec) = read_len_prefixed(&mut f)? {
            let entry = LogEntry::from_bytes(&rec)?;
            let line = serde_json::to_string(&entry)?;
            out.write_all(line.as_bytes())?;
            out.write_all(b"\n")?;
        }
        Ok(())
    }

    /// Find an entry by index (linear scan; OK for small stores).
    pub fn get_entry(&self, index: u64) -> Result<Option<LogEntry>, VeriLogError> {
        let mut f = OpenOptions::new()
            .read(true)
            .open(self.dir.join(ENTRIES_FILE))?;
        while let Some(rec) = read_len_prefixed(&mut f)? {
            let entry = LogEntry::from_bytes(&rec)?;
            if entry.unsigned.index == index {
                return Ok(Some(entry));
            }
        }
        Ok(None)
    }

    pub fn last_entry(&self) -> Result<Option<LogEntry>, VeriLogError> {
        let leaf_count = self.leaf_count();
        if leaf_count == 0 {
            return Ok(None);
        }
        self.get_entry(leaf_count - 1)
    }

    /// Read raw entry record bytes by index using a sequential scan.
    /// Future optimization: store an index table.
    pub fn get_entry_bytes(&self, index: u64) -> Result<Option<Vec<u8>>, VeriLogError> {
        let mut f = OpenOptions::new()
            .read(true)
            .open(self.dir.join(ENTRIES_FILE))?;
        while let Some(rec) = read_len_prefixed(&mut f)? {
            let entry = LogEntry::from_bytes(&rec)?;
            if entry.unsigned.index == index {
                return Ok(Some(rec));
            }
        }
        Ok(None)
    }

    /// Simple store compaction hook placeholder (future).
    pub fn compact(&self) -> Result<(), VeriLogError> {
        // R06 research track: verifiable deletion/garbage collection.
        Ok(())
    }
}
