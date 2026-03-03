# Implementation Blueprint

This is a “buildable plan” for turning the research agenda into a production system.

## 1) Base edition scope (OSS)

### 1.1 Core invariants

- **Append-only semantics**
  - entries are never modified in place
- **Integrity**
  - each entry commits to:
    - previous signed entry hash (`prev_entry_hash`)
    - current Merkle root (`merkle_root`)
  - each entry is signed
- **Verifiability**
  - a third party can verify:
    - signatures
    - chain consistency
    - Merkle root correctness
  - without any secret keys

### 1.2 Public APIs (Rust)

- `verilog_core::LogStore`
  - `init(dir, height)`
  - `open(dir)`
  - `append(kind, payload, level) -> AppendResult`
  - `verify_store() -> VerifyReport`
  - `membership_proof(index) -> MerkleProof`
  - `export_json_lines(writer)`

- `verilog_license`
  - `issue_license(seed_b64, payload) -> SignedLicense` (vendor tool)
  - `verify_license(pub_b64, signed, now, expected_device_id) -> VerifiedLicense`

- `verilog_enterprise_api`
  - enumerates `EnterpriseFeature`
  - defines `EnterpriseModule` trait for private extensions

## 2) Production hardening checklist

### 2.1 Storage crash consistency

- Use atomic rename for `meta.json` (done).
- Add:
  - `fsync(entries.bin)` and `fsync(leaves.bin)` on durable boundaries
  - monotonic append offsets checks
  - optional rolling checksum per record

**Recommended approach**
- Write record
- fsync record
- write leaf
- fsync leaf
- write meta temp + fsync
- rename meta temp -> meta.json

### 2.2 Indexing

Current proof generation loads leaves into memory.

For large stores:
- maintain a compact index (every K entries record file offset)
- optionally store internal Merkle nodes for faster proofs
- consider MMR as a later refactor

### 2.3 Key handling

- support key rotation:
  - store a key ID in each entry
  - rotate on a schedule
- protect private seed at rest:
  - OS key store or secure element
  - encrypted on disk with passphrase (if acceptable)

### 2.4 Configuration

Add `config.toml`:
- tree height
- log retention policy
- dp policy (per-kind)
- admin console bind/auth

## 3) Enterprise integration blueprint

### 3.1 Where enterprise code lives

- private crate `verilog-enterprise` implements:
  - ZK proof engines
  - compliance exporters
  - mesh sync
- `verilogd` binary links it only for enterprise builds

### 3.2 How feature gating works

- Enterprise operations call:
  - `EnterpriseGate::require(feature)`
- License entitlements determine allowed features.
- Without a valid license:
  - return a clear error
  - do not partially run enterprise logic

### 3.3 Suggested enterprise module surface

- `ZkIntegrityProver`
  - prove/verify range integrity
- `ComplianceExporter`
  - produce export bundles (JSON + proofs + metadata)
- Mesh sync service (future)

## 4) Suggested milestone plan (with deliverables)

### Milestone A — Stable base format
- freeze LogEntry v1
- add golden-vector tests
- publish format spec

### Milestone B — Admin console MVP
- status, verify, proof endpoints
- “one panel per track” navigation (links)

### Milestone C — Enterprise ZK MVP
- pick circuit strategy (parallel Poseidon commitment recommended)
- prove “range unchanged” for small ranges
- publish performance measurements

### Milestone D — Compliance export MVP
- export bundle schema + verifier tool
- integrate with ZK proofs

