# VeriLog

**VeriLog** is a Rust-based embedded/edge logging + telemetry engine focused on
turning device logs into auditable evidence.

It is designed to be:

- **Single-binary** (a single executable you ship to the device)
- **Tamper-evident** (hash-chained entries + Merkle-root commitments)
- **Auditable** (membership proofs for any entry; deterministic verification)
- **Privacy-aware** (optional event-level differential privacy for numeric telemetry)
- **Ready for enterprise extensions** (feature-gated hooks + license verification, without open-sourcing proprietary enterprise modules)

> This repository intentionally contains **only the open-source “base” edition**.  
> The **enterprise implementations** are designed to live in a **private crate** or a separately distributed module, unlocked via a signed license.

## What’s in the base edition

- Append-only log store on disk (or flash-like storage)
- Per-entry cryptographic integrity:
  - BLAKE3 hashes
  - Ed25519 signatures
  - Hash chaining (`prev_entry_hash`)
- Incremental Merkle commitment using a fixed-height **incremental Merkle frontier**
  - Root is updated in *O(height)* per entry (default height: 32)
  - Proofs can be produced for an entry index using stored leaf hashes
- CLI:
  - `init` – create a log store and signing keys
  - `append` – append an entry
  - `status` – inspect store metadata and the latest entry
  - `verify` – verify the entire store (signatures, chain, Merkle roots)
  - `checkpoint` – create/verify signed root snapshots for anchoring and archival
  - `proof membership` – generate/verify an inclusion proof
  - `license` – install/check a license file (monetization / gating foundation)
- Optional **admin console MVP** with:
  - live status view
  - proof lookup
  - verification on demand
  - research track visibility for demo sessions

## Enterprise-ready monetization (implemented here)

This repo includes a complete **offline license verification** system:

- License files are JSON + **Ed25519 signature**.
- VeriLog embeds the **vendor public key**.
- Licenses can grant entitlements (feature flags) and optional device binding.

The **enterprise code itself is not included**. Instead, the base edition provides:
- A stable **enterprise API crate** (`verilog-enterprise-api`)
- Compile-time hooks (`--features enterprise`) that you can satisfy with a **private crate**
- Runtime checks (`verilog-license`) that gate access to enterprise-only capabilities

See: `docs/06_license_and_enterprise.md`.

## Repository layout

- `crates/verilog-core` – base logging engine (hashing, signing, Merkle frontier, storage)
- `crates/verilog-license` – license formats + verification + local license store
- `crates/verilog-enterprise-api` – trait definitions and feature IDs for enterprise modules
- `crates/verilogd` – the single executable (CLI + optional admin console)

## Quick start (developer)

> Rust toolchain required (stable).

```bash
# Build
cargo build -p verilogd

# Initialize a store
./target/debug/verilogd init --store ./demo_store

# Append an event
./target/debug/verilogd append --store ./demo_store --kind metric --payload '{"name":"temp_c","value":42.0}'

# Verify integrity
./target/debug/verilogd verify --store ./demo_store

# Inspect status
./target/debug/verilogd status --store ./demo_store

# Emit a signed checkpoint
./target/debug/verilogd checkpoint create --store ./demo_store --out checkpoint.json --label "demo-session"

# Produce a membership proof for entry index 0
./target/debug/verilogd proof membership --store ./demo_store --index 0 --out proof.json

# Verify the proof
./target/debug/verilogd proof verify --proof proof.json

# Run the admin console
cargo run -p verilogd --features admin-console -- serve --store ./demo_store --bind 127.0.0.1:8080
```

## Prototype status

The repository now includes a **full end-to-end prototype** for the base edition:

- initialize a store and generate device-local signing keys
- append signed entries with deterministic hashing
- verify chain and Merkle integrity
- emit membership proofs
- export JSON lines
- generate signed checkpoints for archival/anchoring workflows
- inspect the system through a lightweight local admin UI

The remaining work is product hardening, benchmark coverage, and the novel research
tracks that extend the base protocol.

## Docs

Start here:

- `docs/00_project_status.md`
- `docs/01_architecture.md`
- `docs/02_storage_format.md`
- `docs/03_crypto.md`
- `docs/04_merkle_frontier.md`
- `docs/05_admin_console.md`
- `docs/06_license_and_enterprise.md`
- `docs/07_research_agenda.md`
- `docs/08_roadmap.md`
- `docs/13_delivery_backlog.md`
- `docs/14_novelty_program.md`

## GitHub Pages showcase

A publish-ready static showcase lives in `site/`.

- open `site/index.html` locally to review the presentation layer
- publish the `site/` directory as GitHub Pages or mirror it into your `minh.systems` deployment
- the page is intentionally static and portable, so it works both under a GitHub Pages subpath and a custom domain

## Security note

This is an early reference implementation scaffold. Before production use, you must:
- complete threat modeling (`docs/10_threat_model.md`)
- run cryptographic review + test vectors
- harden storage, crash consistency, and key management
- decide on your ZK system (Halo2/Plonky3) and integrate in enterprise modules

## License

Dual-licensed Apache-2.0 OR MIT (base edition). See `LICENSE-OSS.md`.
