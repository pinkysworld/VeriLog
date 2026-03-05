# Research Agenda (30 tracks across four pillars)

This document expands the provided 20-track research blueprint into:
- hypotheses and evaluation metrics
- implementation milestones
- dependencies and integration points
- “base vs enterprise” placement guidance

> Note: Nothing in this file is a promise of performance numbers; they are **targets** to guide experiments.

---

## Global evaluation harness (recommended)

To keep the tracks comparable, create a shared benchmarking and evaluation setup:

- **Targets**
  - Cortex-M class (simulated or dev board)
  - Linux ARM (Raspberry Pi / CM4)
  - x86_64 edge gateway
- **Metrics**
  - energy per appended event (mJ/event)
  - latency to append (p50/p95)
  - storage overhead (bytes/event)
  - verification time for N events
  - proof size and proof generation time
  - privacy utility metrics (error vs ε)
- **Adversary simulations**
  - log truncation
  - entry modification
  - insertion of fake entries
  - flash bit flips / power-loss corruption

---

## R01 Learned Adaptive Logging Frequency

**Hypothesis:** A learned policy can reduce energy consumption while maintaining evidence quality and respecting privacy budgets.

**Implementation milestones**
1. Base: implement `EnergyPolicy` trait + rule-based policy (done as scaffold).
2. Integrate features: SOC, temperature, event entropy, connectivity state.
3. Add model interface for TinyML inference:
   - 8-bit quantized decision tree (first)
   - then LSTM if needed
4. Add confidence fallback logic.

**Experiments**
- Compare against fixed interval baseline:
  - energy usage
  - missed anomaly detection (if any)
  - total log utility for audits

**Deliverables**
- policy abstraction + simulation tooling
- paper angle: energy-aware telemetry with provable integrity

**Placement**
- Base: policy interface + rule-based fallback
- Enterprise: learned model packaging, retraining, on-device inference

---

## R02 Zero-Knowledge Proof of Log Integrity

**Hypothesis:** It is practical to generate succinct ZK proofs of integrity for embedded logs with acceptable latency.

**Milestones**
1. Define ZK statement: for a range [t1,t2], prove:
   - entries are sequential
   - hash chain + Merkle roots are consistent
2. Choose commitment strategy:
   - prove BLAKE3 (hard) OR
   - maintain a ZK-friendly Poseidon commitment in parallel (recommended)
3. Build circuit in Halo2 or Plonky3:
   - verify leaf commitments
   - verify root transitions
4. Build prover/verifier API in enterprise module.

**Metrics**
- proof size (bytes)
- prover runtime on ARM/Linux
- verifier runtime (server-side)

**Placement**
- Enterprise only (hooks exist in OSS)

---

## R03 Per-Event Differential Privacy

**Hypothesis:** Event-level DP can preserve causal order while protecting sensitive telemetry.

**Milestones**
1. Implement privacy budget token bucket (`verilog-core::dp`).
2. Implement Laplace noise for numeric metrics; add schema-driven redaction.
3. Add utility tracking:
   - record ε spent per event (without leaking private values)
4. Add DP-aware aggregate export (future).

**Metrics**
- mean absolute error vs ε
- privacy budget exhaustion behavior
- performance overhead

**Placement**
- Base: numeric metric DP + budget manager
- Enterprise: richer event schemas, formal privacy accounting, ZK proofs of correct accounting (R13)

---

## R04 Merkle-Based Verifiable Audit Trails

**Hypothesis:** A fixed-height incremental Merkle commitment provides compact and stable audit anchors for embedded logs.

**Milestones**
1. Implement incremental Merkle frontier (done as scaffold).
2. Persist frontier snapshot and leaf hashes.
3. Implement membership proofs.
4. Implement range proofs (non-ZK) by composing membership proofs and checkpoint roots.

**Metrics**
- update cost per entry
- proof size
- verify time

**Placement**
- Base

---

## R05 Energy-Aware Log Compression

**Hypothesis:** Adaptive compression can reduce storage writes and energy without losing auditability.

**Milestones**
1. Baseline: dictionary compression for payloads (e.g., zstd dictionary or bespoke small dict).
2. Energy-aware throttle:
   - when SOC low, reduce compression effort
3. “Verifiable compression” approach:
   - include compressed payload hash in entry commitment
   - keep deterministic decompression

**Metrics**
- compression ratio
- CPU cost vs battery SOC
- flash write reduction

**Placement**
- Base: hooks + deterministic compression option
- Enterprise: learned dictionary training + dashboard

---

## R06 Verifiable Deletion with Proof of Erasure

**Hypothesis:** You can delete old logs while preserving verifiable continuity.

**Milestones**
1. Define deletion policy:
   - time-based TTL
   - capacity-based
2. Implement tombstoning in base:
   - mark entries as deleted but keep commitments (cheap)
3. Enterprise research:
   - ZK proof of correct deletion
   - accumulator-based proofs
   - secure erasure on flash

**Metrics**
- deletion overhead
- audit continuity
- proof size (enterprise)

**Placement**
- Base: tombstones + policy
- Enterprise: cryptographic proof of erasure

---

## R07 Regulatory Export Formats

**Hypothesis:** Compliance exports can be automated with proof bundles.

**Milestones**
1. Base:
   - define export envelope: JSON + proofs + metadata
2. Enterprise:
   - GDPR/HIPAA/SOC2 templates
   - ZK proofs of completeness
   - structured redaction policies

**Metrics**
- completeness coverage
- verification by third party

**Placement**
- Enterprise-heavy; base provides envelope format only.

---

## R08 Cross-Device Log Correlation with Privacy

**Hypothesis:** Multiple devices can correlate events without revealing raw data.

**Milestones**
1. Define correlation primitives:
   - event IDs/hashes
   - time windows
2. Enterprise:
   - PSI protocol selection
   - ZK proofs of correctness
3. Add “correlation transcript” logging in base.

**Metrics**
- correlation accuracy
- leakage bounds
- bandwidth

**Placement**
- Enterprise

---

## R09 Tamper-Evident Log Chaining (sliding window)

**Hypothesis:** A rolling commitment improves resilience and forensics after compromise.

**Milestones**
1. Base:
   - rolling `window_hash = H(prev_window_hash || prev_entry_hash)` (implemented as scaffold)
2. Advanced:
   - store hash of previous N entries explicitly
   - forward-secure key ratchets

**Metrics**
- overhead bytes/event
- post-compromise forensic value

**Placement**
- Base rolling hash; enterprise can extend.

---

## R10 Long-Term Archival with Proof of Immutability

**Hypothesis:** Periodic checkpoints reduce audit costs for long archives.

**Milestones**
1. Base:
   - checkpoint command: emit signed checkpoint record with current root
2. Enterprise:
   - external anchoring (e.g., transparency log / timestamping)
   - ZK proof for “log up to T unchanged”

**Metrics**
- archive verification time reduction
- checkpoint overhead

**Placement**
- Base checkpoints; enterprise anchoring.

---

## R11 Oblivious Logging Modes

**Hypothesis:** You can hide access patterns for reads.

**Milestones**
- Research-heavy; likely enterprise:
  - ORAM-inspired read patterns
  - flash-friendly shuffling
  - measurable overhead

**Placement**
- Enterprise only.

---

## R12 Verifiable Anomaly-Triggered Logging

**Hypothesis:** Alerts can be cryptographically bound to detection logic.

**Milestones**
1. Base:
   - record “alert” entry kind, signed like any other
   - include rule ID and inputs (redacted as needed)
2. Enterprise:
   - ZK proof that rule condition was satisfied without revealing raw values

**Metrics**
- false positive/negative
- proof size (enterprise)

**Placement**
- Base alert records; enterprise ZK proofs.

---

## R13 Privacy-Preserving Telemetry Aggregation

**Hypothesis:** Aggregates can be verifiable without trusting an aggregator.

**Milestones**
1. Base:
   - define aggregation transcript format (inputs, DP params, outputs)
2. Enterprise:
   - federated aggregation protocol
   - ZK proof of correct aggregation + DP application

**Placement**
- Enterprise-heavy.

---

## R14 Secure Log Forwarding over Unreliable Links

**Hypothesis:** Logs can be forwarded securely with ordering guarantees.

**Milestones**
1. Base:
   - define forwarding envelope: chunk roots + sequence numbers
2. Enterprise:
   - ratcheting encryption, replay protection
   - ZK proof of delivery order

**Metrics**
- bandwidth
- reconnect recovery time
- loss handling

---

## R15 Zero-Knowledge Range Queries on Logs

**Hypothesis:** You can prove predicates over ranges without revealing values.

**Milestones**
- Enterprise:
  - define predicate DSL
  - compile to circuits
  - prove “all temps < 50°C” etc.

**Placement**
- Enterprise only.

---

## R16 Wasm-Based User-Defined Logging Rules

**Hypothesis:** Users can safely upload custom rules with sandboxing and verifiable outputs.

**Milestones**
1. Base:
   - define rule interface trait
2. Enterprise:
   - embed Wasm runtime (wasmtime/wasmi)
   - deterministic execution constraints
   - proof-carrying outputs (ZK optional)

**Placement**
- Likely enterprise (Wasm runtime footprint).

---

## R17 Energy-Proportional Encryption Strength

**Hypothesis:** Adaptive crypto can preserve battery while meeting policy.

**Milestones**
1. Base:
   - define cipher suite abstraction
2. Enterprise:
   - dynamic selection with policy proofs

**Placement**
- Enterprise (policy proofs); base may include abstraction only.

---

## R18 Learned False-Positive Reduction for Alerts

**Hypothesis:** Meta-learning reduces alert fatigue on-device.

**Milestones**
- Enterprise:
  - training pipeline
  - evaluation on real datasets
  - energy-aware inference

---

## R19 Cross-Platform Binary Optimization

**Hypothesis:** Compile-time options + runtime tuning can meaningfully reduce energy/latency.

**Milestones**
1. Base:
   - benchmarks, feature flags
2. Enterprise:
   - auto-tuning, dashboard

---

## R20 Verifiable Log Synchronization in Mesh Networks

**Hypothesis:** Secure gossip can keep partitions consistent with proofs.

**Milestones**
1. Base:
   - define sync transcript format
2. Enterprise:
   - mesh protocol
   - ZK proofs of consistency/completeness

---

## R21 Recursive Proof Aggregation

**Hypothesis:** Multiple log integrity proofs (from different time ranges or devices) can be aggregated into a single succinct proof, reducing verification cost for auditors.

**Milestones**
1. Define proof composition interface: combine N range proofs into 1.
2. Evaluate IVC (Incrementally Verifiable Computation) frameworks:
   - Nova / SuperNova for folding-based approach
   - Plonky3 recursion for STARK aggregation
3. Build aggregator service that collects proofs from fleet devices.
4. Benchmark: aggregated proof size vs N, verification time vs N.

**Metrics**
- aggregated proof size (should grow sub-linearly with N)
- aggregator computation cost
- verifier time for aggregated proof

**Placement**
- Enterprise only (hooks in OSS for proof format compatibility)

---

## R22 Post-Quantum Signature Migration

**Hypothesis:** VeriLog can provide a migration path from Ed25519 to post-quantum signatures without breaking existing verification of historical records.

**Milestones**
1. Define dual-signature entry format: Ed25519 + PQ signature.
2. Evaluate candidates:
   - SPHINCS+ (hash-based, conservative)
   - Dilithium/ML-DSA (lattice-based, compact)
3. Implement PQ signature module behind feature flag.
4. Define migration protocol: epoch-based transition with dual signing.
5. Benchmark signature size, generation time, and verification time on ARM.

**Metrics**
- signature size overhead
- signing latency on Cortex-M / ARM Linux
- verification latency
- storage overhead per entry

**Placement**
- Base: signature trait abstraction + migration protocol spec
- Enterprise: PQ signature implementations + migration tooling

---

## R23 Federated Privacy Budget Governance

**Hypothesis:** Multiple organizations sharing evidence from overlapping device populations can negotiate and enforce cross-organizational DP budgets.

**Milestones**
1. Define budget negotiation protocol: propose/accept/reject budget allocations.
2. Implement budget ledger: cryptographic commitment to cumulative spend.
3. Add ZK proof of budget compliance: prove ε_spent ≤ ε_allocated without revealing spend details.
4. Integrate with R03 per-event DP and R13 aggregation.

**Metrics**
- protocol round-trip time
- proof size for budget compliance
- correctness under concurrent budget modifications

**Placement**
- Enterprise only

---

## R24 Synthetic Telemetry Generation

**Hypothesis:** Privacy-safe synthetic datasets can be generated from DP-processed telemetry, enabling testing, model training, and data sharing without privacy risk.

**Milestones**
1. Implement statistical profile extraction from DP-processed logs.
2. Build synthetic event generator respecting temporal and causal patterns.
3. Add utility metrics: compare synthetic vs real data on downstream tasks.
4. Add provenance tracking: link synthetic data to DP parameters used.

**Metrics**
- statistical fidelity (distribution distance metrics)
- downstream task accuracy with synthetic vs real data
- generation throughput
- provenance verification time

**Placement**
- Base: profile extraction + simple generator
- Enterprise: advanced generators, utility optimization, provenance proofs

---

## R25 Hardware-Accelerated Hashing

**Hypothesis:** Leveraging hardware crypto extensions (ARM CE, AES-NI, RISC-V Crypto) for BLAKE3 and Poseidon can meaningfully reduce proof generation time and energy on constrained devices.

**Milestones**
1. Profile current BLAKE3 performance on target platforms.
2. Evaluate BLAKE3 SIMD optimizations and ARM Crypto Extension paths.
3. For Poseidon (ZK track): implement platform-specific field arithmetic.
4. Build hardware-detection layer that selects optimal implementation at startup.
5. Benchmark energy per hash operation across platforms.

**Metrics**
- hashes per second per platform
- energy per hash (mJ)
- proof generation speedup
- binary size impact of platform-specific code

**Placement**
- Base: BLAKE3 acceleration + detection layer
- Enterprise: Poseidon acceleration for ZK tracks

---

## R26 Decentralized Timestamping Anchors

**Hypothesis:** Anchoring signed checkpoints to multiple independent timestamping sources provides stronger tamper-evidence guarantees than any single anchor.

**Milestones**
1. Define generic anchoring interface trait.
2. Implement RFC 3161 timestamp authority client.
3. Implement Certificate Transparency log submission adapter.
4. Implement blockchain anchoring adapter (Bitcoin OP_RETURN or Ethereum).
5. Build multi-anchor checkpoint bundle format with proof of inclusion for each.
6. Define anchor freshness and redundancy policies.

**Metrics**
- time-to-anchor per target
- cost per anchor (financial, bandwidth)
- verification complexity with N anchors
- availability/redundancy improvement

**Placement**
- Base: anchoring trait + RFC 3161 client
- Enterprise: CT log, blockchain adapters, multi-anchor bundles

---

## R27 Verifiable Log Format Migration

**Hypothesis:** When the log format evolves (new entry fields, changed serialization), the migration can be proven correct so auditors trust migrated archives.

**Milestones**
1. Define format versioning protocol with explicit migration functions.
2. Implement deterministic migration: old format → new format with re-signed entries.
3. Generate migration proof: ZK or Merkle-based proof that migrated data is semantically equivalent.
4. Build migration verification tool.

**Metrics**
- migration throughput (entries/sec)
- proof size for migration correctness
- verification time for migration proof
- backward compatibility coverage

**Placement**
- Base: format versioning + deterministic migration
- Enterprise: migration proofs + verification tooling

---

## R28 Privacy-Preserving Log Search

**Hypothesis:** Encrypted search indexes can enable querying log stores without decrypting entries, preserving confidentiality during audits.

**Milestones**
1. Evaluate searchable symmetric encryption (SSE) schemes for log data.
2. Implement keyword-based encrypted index alongside the plain log.
3. Add range query support for timestamps and numeric fields.
4. Benchmark search latency and index size overhead.
5. Define access control: who holds search tokens, revocation.

**Metrics**
- search latency vs plaintext baseline
- index size overhead (bytes per entry)
- leakage profile (what the index reveals)
- token generation and revocation cost

**Placement**
- Enterprise only (research-heavy)

---

## R29 Predictive Maintenance Evidence

**Hypothesis:** Anomaly detection models running on-device can produce cryptographically attested maintenance predictions, combining integrity evidence with operational intelligence.

**Milestones**
1. Define attestation format for model predictions: model ID, input hash, output, confidence.
2. Implement lightweight anomaly detection (statistical + optional TinyML).
3. Sign predictions as special log entry kind with model provenance.
4. Add verifiable model update protocol: prove that a model update was authorized.
5. Energy-aware inference scheduling (integrate with R01).

**Metrics**
- prediction accuracy (F1, precision, recall)
- energy per inference
- attestation overhead
- model update verification time

**Placement**
- Base: attestation format + statistical anomaly detection
- Enterprise: TinyML models, update protocol, fleet-wide dashboards

---

## R30 Fleet-Wide Integrity Dashboards

**Hypothesis:** Aggregated compliance and integrity status across a fleet of devices can be computed and displayed without collecting raw log data.

**Milestones**
1. Define fleet status aggregation protocol: devices submit signed summaries.
2. Implement dashboard backend: collect summaries, compute fleet-wide metrics.
3. Add anomaly detection at fleet level: detect devices with integrity failures.
4. Integrate with R26 anchoring: show anchor freshness per device.
5. Add ZK-based fleet attestation: prove "all devices in fleet pass integrity check."

**Metrics**
- dashboard update latency
- bandwidth per device summary
- false positive rate for fleet anomaly detection
- ZK fleet attestation proof size

**Placement**
- Enterprise only

---

## Cross-track integration plan

1. Start with R04/R09 (integrity primitives) ✅
2. Add R10 checkpoints for long-lived devices
3. Add R03 DP for sensitive telemetry
4. Build admin console panels as instrumentation endpoints
5. Add enterprise ZK layer (R02/R15) once base format is stable
6. R25 hardware acceleration feeds into R02 ZK performance
7. R22 PQ migration depends on stable entry format (Phase 1)
8. R26 anchoring builds on R10 checkpoint infrastructure
9. R21 recursive aggregation requires R02 proof format first
10. R30 fleet dashboards integrate R26 anchoring + R12 anomaly triggers

