# Delivery Backlog

This backlog is ordered by what most improves the project over the next few
iterations, not by what is easiest to build.

## Now

- [ ] Add explicit durable-append boundaries (`fsync`) and recovery tests.
- [ ] Add an indexed proof path so large stores do not need full leaf scans.
- [ ] Publish golden-vector fixtures for entries, proofs, and checkpoints.
- [ ] Add structured config (`config.toml`) for tree height, admin bind, and retention.
- [ ] Add CLI integration tests for `status`, `checkpoint`, and export flows.
- [ ] Add minimal auth strategy for the admin console (token or mTLS-ready config).

## Next

- [ ] Introduce retention policies and verifiable tombstones.
- [ ] Add deterministic payload compression envelopes.
- [ ] Add a benchmark harness across x86_64, ARM Linux, and microcontroller-class traces.
- [ ] Ship schema packs for privacy-safe metric logging.
- [ ] Add offset indexing or compact Merkle caches for faster proof serving.
- [ ] Publish a reference verifier as a separate small Rust crate or tool.

## Later

- [ ] Prototype external checkpoint anchoring (transparency log, signed transparency feed, or timestamp authority).
- [ ] Add secure forwarding and mesh synchronization transcript formats.
- [ ] Run a parallel Poseidon commitment experiment for future ZK work.
- [ ] Define cross-device correlation transcripts and privacy boundaries.
- [ ] Add a policy interface for fleet orchestration and compliance export bundles.

## Product-facing deliverables

- [ ] Record a scripted demo using the admin console plus CLI checkpoints.
- [ ] Publish the `site/` showcase through GitHub Pages.
- [ ] Prepare a one-page architecture brief for prospective partners.
- [ ] Freeze the v1 storage/proof/checkpoint format before outside integrations.

## Definition of “pilot ready”

The base edition is ready for pilot programs when all of these are true:

- durable append semantics are tested
- third-party verification is documented and repeatable
- admin access can be limited safely
- benchmark numbers exist for representative device classes
- checkpoint and proof formats are treated as stable contracts
