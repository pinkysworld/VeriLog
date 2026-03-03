# Storage Format

This document describes the on-disk layout used by `verilog-core::LogStore`.

> This is a **reference format** suitable for early development.  
> Production use should harden crash consistency (fsync, atomic meta updates, checksums).

## Store directory layout

A store is a directory (e.g., `./demo_store`) containing:

- `meta.json` – store metadata (version, entry count, Merkle frontier snapshot, last hashes)
- `signing_key.json` – Ed25519 seed (private) + verifying key (public)
- `entries.bin` – length-delimited binary log entries (postcard)
- `leaves.bin` – raw 32-byte leaf hashes (concatenated)

### `meta.json`

Example (human readable; actual values are base64/hex):

```json
{
  "version": 1,
  "tree_height": 32,
  "leaf_count": 12,
  "prev_entry_hash_b64": "...",
  "window_hash_b64": "...",
  "frontier_b64": ["...", null, "..."],
  "created_at_unix_ms": 1730000000000
}
```

Fields:

- `version`: format version (increment if you change fields)
- `tree_height`: fixed Merkle tree height (default 32)
- `leaf_count`: number of entries / leaves
- `prev_entry_hash_b64`: hash of the last entry (hash chain tip)
- `window_hash_b64`: rolling commitment (base track for R09)
- `frontier_b64`: Merkle frontier nodes (one per level, `null` if absent)
- `created_at_unix_ms`: creation timestamp

## `entries.bin` format

A sequence of records:

```
[u32 little-endian length][length bytes of postcard(LogEntry)]
[u32 little-endian length][length bytes of postcard(LogEntry)]
...
```

This allows streaming reads and append-only writes.

## `LogEntry` fields (version 1)

Each entry includes:

- `version: u16`
- `index: u64` – monotonic leaf index (0..)
- `ts_unix_ms: u64`
- `level: LogLevel`
- `kind: String` – application-defined event kind (e.g. `"metric"`)
- `payload: Vec<u8>` – application-defined payload (often JSON bytes)
- `prev_entry_hash: [u8;32]` – hash chain pointer
- `window_hash: [u8;32]` – rolling hash commitment
- `merkle_root: [u8;32]` – fixed-height Merkle root after inserting this leaf
- `signature: [u8;64]` – Ed25519 signature over `entry_hash`

### Entry hash and signature

To compute `entry_hash`:

1. Serialize the entry **without signature** using postcard.
2. `entry_hash = BLAKE3(serialized_bytes)`
3. `signature = Ed25519.sign(entry_hash)`

## `leaves.bin` format

A raw concatenation of 32-byte leaf hashes:

```
[leaf0_hash32][leaf1_hash32]...[leafN_hash32]
```

In this reference implementation the leaf hash equals the `entry_hash`.  
(If you later want ZK-friendly hashes, you may switch to Poseidon for the leaf commitment in an enterprise module.)

## Crash consistency notes

This reference implementation uses the following safe-ish order:

1. append entry record to `entries.bin`
2. append leaf hash to `leaves.bin`
3. update `meta.json`

If power is lost after step (1) but before step (3), verification can detect mismatch.  
For production, prefer:

- write-ahead journaling, or
- atomic meta update via temp file + rename, and
- explicit `fsync` at key points

