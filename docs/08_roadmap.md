# Roadmap

This roadmap is structured so the **base edition** remains useful and stable,
while enterprise research tracks can proceed in parallel.

## Phase 0 — Core scaffolding (this repo)

- [x] Store initialization, key generation
- [x] Append-only log format with signatures + hash chain
- [x] Incremental Merkle frontier + root stored per entry
- [x] Full-store verification
- [x] Membership proof generation + verification
- [x] Store status reporting
- [x] Signed checkpoint snapshots
- [x] License verification + entitlements (monetization foundation)
- [x] Admin console MVP: dashboard + status/proof/checkpoint/research endpoints (optional feature)
- [ ] Crash consistency hardening

## Phase 1 — Base hardening

- [x] Power-loss safe append protocol (fsync on entries, leaves, atomic meta writes)
- [ ] Backpressure + bounded memory
- [x] Config system (TOML) with defaults (store, admin, privacy, retention)
- [x] Durable fsync boundaries + golden vector test fixtures (8 tests)
- [ ] Storage backend abstraction (file / flash / ring buffer)
- [ ] Threat model sign-off + security review

## Phase 2 — Privacy and energy

- [ ] DP schemas for common telemetry events
- [ ] Per-event DP accounting tests and docs
- [ ] Energy policy improvements and instrumentation
- [ ] Compression options

## Phase 3 — Enterprise private modules

- [ ] ZK proofs of integrity (R02)
- [ ] Compliance export bundles (R07)
- [ ] Mesh sync and forwarding (R14/R20)
- [ ] PSI correlation (R08)
- [ ] ZK range queries (R15)

## Phase 4 — Publication / standardization

- [x] Launch the public project site (`site/`) with professional redesign and docs subpage
- [ ] Formal spec of the log format
- [ ] Third-party verifier reference implementation
- [ ] Interop test suite
