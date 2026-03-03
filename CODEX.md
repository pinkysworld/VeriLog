# CODEX Instructions (for AI coding agents)

This document is meant to be copied into an AI coding agent (e.g., Codex/ChatGPT) as the project’s working instructions.

## Prime directive

**Do not implement proprietary enterprise features in this open-source repository.**

This repository is the **base edition** only. It contains:
- the logging engine (`verilog-core`)
- license verification + storage (`verilog-license`)
- the enterprise *API surface* (`verilog-enterprise-api`)
- the single binary (`verilogd`) with feature gates

Enterprise-only logic must live in:
- a private crate (recommended) OR
- a separately distributed module loaded by private builds

The OSS edition must compile, run, and verify logs **without** enterprise code.

## Goals for changes you make

1. Preserve the **append-only, verifiable** properties:
   - every entry is hash-chained (`prev_entry_hash`)
   - every entry has a signature
   - the Merkle root stored in each entry must match the incremental Merkle frontier
2. Keep the binary **single-executable** (no required sidecar services).
3. Keep all security-sensitive code:
   - reviewed
   - test-covered (unit tests + golden vectors)
   - deterministic where possible
4. Maintain clear separation:
   - `verilog-enterprise-api` defines traits and feature IDs (public)
   - enterprise implementations are **not** added here

## Repository map

- `crates/verilog-core`
  - hashing, signatures, Merkle frontier, entries, storage, verification, DP stubs, energy policy stubs
- `crates/verilog-license`
  - license format, vendor public key verification, device ID, local license store
- `crates/verilog-enterprise-api`
  - `EnterpriseFeature` enum + hooks traits
- `crates/verilogd`
  - CLI and (optional) admin console

## Coding standards

- Prefer small modules with clear boundaries.
- Use `thiserror` for typed errors in libraries.
- Keep file formats versioned (`Meta.version` and `LogEntry.version`).
- Avoid panics in library code. Return `Result`.
- Prefer deterministic encodings for signing/hashing:
  - use `postcard`/`serde` consistently
  - never sign JSON directly unless canonicalized

## Security rules

- Never log private keys.
- Treat license checks as “defense in depth”:
  - the absence of a license must **deny enterprise features**
  - the presence of a license must be validated with signature + time window + optional device binding
- Avoid network dependencies in base edition.
- All crypto operations must be constant-time where practical (signature verification already is).
- All comparisons of MAC/signature bytes must be constant-time (use `subtle` if needed).

## How to add a new feature (base edition)

1. Add the core logic under `verilog-core/src/...`.
2. Add CLI surface in `verilogd/src/commands/...`.
3. Add tests:
   - unit tests for pure functions
   - integration tests for store verification
4. Update docs in `docs/`:
   - architecture changes
   - storage format changes
   - threat model changes

## How to add an enterprise feature (allowed in this repo)

You may add:
- new IDs in `EnterpriseFeature`
- new traits in `verilog-enterprise-api`
- new stub wiring in `verilogd` that is behind:
  - license entitlement checks, and
  - `#[cfg(feature = "enterprise")]` compile gates

You must NOT add:
- real enterprise implementations
- obfuscation / DRM bypass mechanisms
- code that leaks proprietary logic

## “Definition of done” checklist

- `cargo fmt` clean formatting
- `cargo clippy` no new warnings (when available)
- tests added/updated
- docs updated
- clear note if a change impacts log format

