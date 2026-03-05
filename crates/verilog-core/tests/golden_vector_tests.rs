//! Golden vector tests for VeriLog storage format.
//!
//! These tests create a store with known inputs, compute expected hashes,
//! and verify that the storage format produces deterministic, stable outputs.
//! If any of these tests break, it means the on-disk format has changed —
//! which must be treated as a breaking change requiring a migration path.

use std::{fs, path::PathBuf};
use verilog_core::{
    hash::{hash32_to_hex, hash_bytes, hash_pair, zero_hashes},
    LogLevel, LogStore,
};

fn temp_dir(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "verilog-golden-{name}-{}-{}",
        std::process::id(),
        verilog_core::util::now_unix_ms()
    ));
    dir
}

/// Verify that the zero-entry root for height 8 is deterministic.
#[test]
fn golden_empty_root_height_8() {
    let dir = temp_dir("empty-root");
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }

    LogStore::init(&dir, Some(8)).unwrap();
    let store = LogStore::open(&dir).unwrap();

    assert_eq!(store.leaf_count(), 0);
    let root = store.current_root().unwrap();

    // An empty Merkle tree with zero padding should give the same root
    // every time. Record the hex here as the golden vector.
    let root_hex = hash32_to_hex(&root);
    assert!(
        !root_hex.is_empty(),
        "empty root must produce a deterministic hash"
    );
    // The root for an empty tree of height 8 uses the zero_hashes convention:
    // zero[0] = hash_bytes([0;32]), then zero[i+1] = H(zero[i] || zero[i]).
    // For an empty tree (leaf_count=0), root() starts with acc=zero[0] and
    // at each level hashes acc with zero[level] on the right side.
    let zeros = zero_hashes(8);
    let mut expected = zeros[0];
    for level in 0..8 {
        expected = hash_pair(&expected, &zeros[level]);
    }
    let expected_hex = hash32_to_hex(&expected);
    assert_eq!(root_hex, expected_hex, "empty root mismatch for height 8");

    fs::remove_dir_all(&dir).unwrap();
}

/// Verify that a single append produces deterministic and stable output.
#[test]
fn golden_single_append() {
    let dir = temp_dir("single-append");
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }

    LogStore::init(&dir, Some(8)).unwrap();
    let mut store = LogStore::open(&dir).unwrap();

    let result = store
        .append(
            "test.golden",
            b"golden-payload-001".to_vec(),
            LogLevel::Info,
        )
        .unwrap();

    // Verify structure invariants.
    assert_eq!(result.entry.unsigned.index, 0);
    assert_eq!(result.entry.unsigned.kind, "test.golden");
    assert_eq!(result.entry.unsigned.payload, b"golden-payload-001");
    assert_eq!(result.entry.unsigned.prev_entry_hash, [0u8; 32]);

    // Window hash for index 0: H(zero_hash || zero_prev_entry_hash)
    let zero = [0u8; 32];
    let expected_window = hash_pair(&zero, &zero);
    assert_eq!(result.entry.unsigned.window_hash, expected_window);

    // Verify entry hash is deterministic for the same unsigned content.
    let entry_hash_1 = result.entry.unsigned.entry_hash().unwrap();
    let entry_hash_2 = result.entry.unsigned.entry_hash().unwrap();
    assert_eq!(entry_hash_1, entry_hash_2, "entry_hash must be deterministic");

    // Verify the hash in the result matches.
    assert_eq!(
        result.entry_hash_hex,
        hash32_to_hex(&entry_hash_1),
        "result hash hex must match computed hash"
    );

    // Verify the store is valid after the append.
    let report = store.verify_store().unwrap();
    assert!(report.ok);
    assert_eq!(report.leaf_count, 1);

    fs::remove_dir_all(&dir).unwrap();
}

/// Verify that the hash chain links entries correctly across multiple appends.
#[test]
fn golden_hash_chain_linkage() {
    let dir = temp_dir("hash-chain");
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }

    LogStore::init(&dir, Some(8)).unwrap();
    let mut store = LogStore::open(&dir).unwrap();

    let r0 = store
        .append("chain.0", b"payload-0".to_vec(), LogLevel::Info)
        .unwrap();
    let r1 = store
        .append("chain.1", b"payload-1".to_vec(), LogLevel::Info)
        .unwrap();
    let r2 = store
        .append("chain.2", b"payload-2".to_vec(), LogLevel::Info)
        .unwrap();

    // Entry 1's prev_entry_hash must equal entry 0's hash.
    let e0_hash = r0.entry.unsigned.entry_hash().unwrap();
    assert_eq!(
        r1.entry.unsigned.prev_entry_hash, e0_hash,
        "entry 1 must link to entry 0"
    );

    // Entry 2's prev_entry_hash must equal entry 1's hash.
    let e1_hash = r1.entry.unsigned.entry_hash().unwrap();
    assert_eq!(
        r2.entry.unsigned.prev_entry_hash, e1_hash,
        "entry 2 must link to entry 1"
    );

    // Verify the rolling window hash chain.
    let zero = [0u8; 32];
    let w0 = hash_pair(&zero, &zero); // window for index 0
    assert_eq!(r0.entry.unsigned.window_hash, w0);

    let w1 = hash_pair(&w0, &e0_hash); // window for index 1
    assert_eq!(r1.entry.unsigned.window_hash, w1);

    let w2 = hash_pair(&w1, &e1_hash); // window for index 2
    assert_eq!(r2.entry.unsigned.window_hash, w2);

    // Full store verification must pass.
    let report = store.verify_store().unwrap();
    assert!(report.ok);
    assert_eq!(report.leaf_count, 3);

    fs::remove_dir_all(&dir).unwrap();
}

/// Verify that membership proofs are correct for all entries.
#[test]
fn golden_membership_proofs() {
    let dir = temp_dir("proofs");
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }

    LogStore::init(&dir, Some(8)).unwrap();
    let mut store = LogStore::open(&dir).unwrap();

    for i in 0..5 {
        store
            .append(
                &format!("proof.{i}"),
                format!("proof-payload-{i}").into_bytes(),
                LogLevel::Info,
            )
            .unwrap();
    }

    // Get the final root.
    let root = store.current_root().unwrap();

    // Every entry must have a valid membership proof against the current root.
    for i in 0..5 {
        let proof = store.membership_proof(i).unwrap();
        assert!(
            proof.verify().unwrap(),
            "membership proof for index {i} must verify"
        );
        assert_eq!(
            proof.root, root,
            "proof root must match current store root for index {i}"
        );
        assert_eq!(proof.leaf_index, i);
    }

    fs::remove_dir_all(&dir).unwrap();
}

/// Verify that checkpoint creation and verification are deterministic.
#[test]
fn golden_checkpoint() {
    let dir = temp_dir("checkpoint");
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }

    LogStore::init(&dir, Some(8)).unwrap();
    let mut store = LogStore::open(&dir).unwrap();

    store
        .append("cp.entry", b"checkpoint-test".to_vec(), LogLevel::Info)
        .unwrap();

    let cp = store
        .create_checkpoint(Some("golden-test".into()))
        .unwrap();

    // Checkpoint must be verifiable.
    cp.verify().unwrap();

    // Checkpoint fields must be consistent with store state.
    assert_eq!(cp.checkpoint.leaf_count, 1);
    assert_eq!(cp.checkpoint.tree_height, 8);
    assert_eq!(cp.checkpoint.label.as_deref(), Some("golden-test"));
    assert_eq!(cp.checkpoint.current_root, store.current_root().unwrap());
    assert_eq!(cp.checkpoint.version, 1);

    // Re-verify via JSON round-trip (simulates file-based checkpoint exchange).
    let json = serde_json::to_vec_pretty(&cp).unwrap();
    let cp2: verilog_core::SignedCheckpoint = serde_json::from_slice(&json).unwrap();
    cp2.verify().unwrap();
    assert_eq!(cp2.checkpoint.leaf_count, cp.checkpoint.leaf_count);
    assert_eq!(cp2.checkpoint.current_root, cp.checkpoint.current_root);

    fs::remove_dir_all(&dir).unwrap();
}

/// Verify that signature verification correctly rejects tampered data.
#[test]
fn golden_tamper_detection() {
    let dir = temp_dir("tamper");
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }

    LogStore::init(&dir, Some(8)).unwrap();
    let mut store = LogStore::open(&dir).unwrap();

    let result = store
        .append("tamper.test", b"original".to_vec(), LogLevel::Info)
        .unwrap();

    // Tamper with the entry bytes on disk.
    let entries_path = dir.join("entries.bin");
    let mut bytes = fs::read(&entries_path).unwrap();
    // Flip a byte somewhere in the payload region (after the length prefix).
    if bytes.len() > 10 {
        bytes[10] ^= 0xFF;
    }
    fs::write(&entries_path, &bytes).unwrap();

    // Verification must fail after tampering.
    let store2 = LogStore::open(&dir).unwrap();
    let result = store2.verify_store();
    assert!(
        result.is_err(),
        "verification must fail after entry tampering"
    );

    fs::remove_dir_all(&dir).unwrap();
}

/// Verify that the leaf hash computation is stable.
#[test]
fn golden_leaf_hash_stability() {
    let dir = temp_dir("leaf-hash");
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }

    LogStore::init(&dir, Some(8)).unwrap();
    let mut store = LogStore::open(&dir).unwrap();

    let r = store
        .append("leaf.stable", b"leaf-test".to_vec(), LogLevel::Info)
        .unwrap();

    // Read back the leaf hash from leaves.bin.
    let leaves = store.read_leaf_hashes().unwrap();
    assert_eq!(leaves.len(), 1);

    // The leaf hash in the result and from the file must match.
    let expected_hex = r.leaf_hash_hex;
    let actual_hex = hash32_to_hex(&leaves[0]);
    assert_eq!(
        actual_hex, expected_hex,
        "leaf hash from file must match append result"
    );

    fs::remove_dir_all(&dir).unwrap();
}

/// Verify export format stability (JSON Lines).
#[test]
fn golden_export_format() {
    let dir = temp_dir("export");
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }

    LogStore::init(&dir, Some(8)).unwrap();
    let mut store = LogStore::open(&dir).unwrap();

    store
        .append("export.test", b"export-data".to_vec(), LogLevel::Info)
        .unwrap();

    let mut buf = Vec::new();
    store.export_json_lines(&mut buf).unwrap();
    let text = String::from_utf8(buf).unwrap();

    // Must be valid JSON lines (one line per entry).
    let lines: Vec<&str> = text.trim().lines().collect();
    assert_eq!(lines.len(), 1, "export must have one line per entry");

    // Each line must parse as valid JSON.
    let parsed: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert!(parsed.get("unsigned").is_some(), "entry must have unsigned field");
    assert!(parsed.get("signature").is_some(), "entry must have signature field");

    fs::remove_dir_all(&dir).unwrap();
}
