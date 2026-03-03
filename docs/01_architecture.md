# Architecture

## High-level picture

VeriLog is designed as a **single executable** (`verilogd`) that embeds:

1. A verifiable append-only **log store** (`verilog-core`)
2. A **license verifier** and entitlement checker (`verilog-license`)
3. Optional **admin console** (HTTP UI/API), gated by a Cargo feature
4. Optional **enterprise hooks**, compiled only in private builds

### Trust boundaries

- The device running VeriLog may be partially compromised **after the fact**.
- The log store must provide *tamper evidence* for historical data even if the device later gets owned.
- Enterprise features must be gated by license and must be separable from the OSS base.

## Component diagram

- `verilogd` (binary)
  - CLI subcommands (init, append, verify, proof, export, license, serve)
  - Optional `admin-console` module (Axum routes)
  - Integrates license checks for enterprise-only operations

- `verilog-core` (library)
  - `LogStore`:
    - append entries (crash-safe append protocol is a future hardening step)
    - persist leaf hashes
    - maintain Merkle frontier snapshot
    - verify full store
  - `MerkleFrontier`:
    - fixed-height incremental Merkle tree with zero padding
    - O(height) update
  - `crypto`:
    - BLAKE3 hashing
    - Ed25519 signing
  - `dp` (base):
    - token-bucket privacy budget
    - Laplace noise for numeric telemetry (reference implementation)

- `verilog-license` (library)
  - License JSON format + Ed25519 signature verification
  - Device ID generation (best-effort, with fallback)
  - Local license store (install/status)

- `verilog-enterprise-api` (library)
  - `EnterpriseFeature` IDs
  - traits for enterprise implementations (ZK proof engines, exporters, etc.)

## Data flow

### Append

1. Client calls `LogStore::append(kind, payload, level)`
2. Store reads `Meta`:
   - last entry hash
   - Merkle frontier snapshot
3. Build `LogEntry`:
   - compute `entry_hash = blake3(postcard(entry_without_signature))`
   - sign `entry_hash`
4. Persist:
   - append the entry record (length-delimited) to `entries.bin`
   - append the leaf hash to `leaves.bin`
   - update `Meta` with:
     - new `prev_entry_hash`
     - updated Merkle frontier snapshot
     - new Merkle root

### Verify

1. Iterate over all entries, recompute:
   - hash chain
   - signature validity
   - Merkle frontier roots
2. Fail fast on mismatch
3. Return final root + entry count

### Membership proof

1. Load leaf hashes (`leaves.bin`)
2. Compute sibling nodes for every level (default height 32)
3. Output `MerkleProof` (leaf, index, siblings, expected root)
4. Verifier recomputes root from leaf+siblings and checks equality

## OSS vs Enterprise boundary

**OSS base** provides:
- log integrity (hash chain + signatures)
- Merkle commitment and membership proofs
- optional basic DP for numeric telemetry
- CLI and optional admin console

**Enterprise (private)** is expected to add:
- Halo2/Plonky3 ZK proof generation and verification
- compliance export bundles with proofs
- cross-device correlation (PSI, ZK)
- advanced energy-aware ML scheduling

The OSS base ships only **hooks** and **license gates**, not the enterprise code.

See `docs/06_license_and_enterprise.md`.

## Implementation blueprint

A suggested build order:

1. ✅ Storage format + cryptographic chaining (`verilog-core`)
2. ✅ Incremental Merkle frontier + proof generation
3. ✅ Full-store verifier and test vectors
4. ✅ CLI hardening + export formats
5. ⬜ Crash consistency + power-loss testing (journaling / fsync strategy)
6. ⬜ Embedded target support (`no_std` subset, flash backends)
7. ⬜ Admin console (Axum) panels per research track
8. ⬜ Enterprise private modules (ZK, compliance, mesh sync, etc.)

