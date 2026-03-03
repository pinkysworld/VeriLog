# Project Status

This file is the short operational view of the project: what exists now, what is
stable enough to demo, and what still needs focused engineering before the system
is ready for external pilots.

## Current snapshot

**Edition:** Open-source base edition  
**Implementation language:** Rust  
**Delivery shape:** Single-binary CLI with optional local admin console  
**Maturity:** Working prototype

## What is running today

- Store initialization with local signing-key generation
- Append-only signed log entries
- Hash chaining and rolling window commitments
- Incremental Merkle frontier with per-entry root commitments
- Full-store verification
- Membership proof generation and proof verification
- JSON Lines export
- Offline license install/status flow
- Signed checkpoints for archival and external anchoring workflows
- Admin console MVP for local demos and operator inspection

## What is stable enough to show

- End-to-end device log lifecycle:
  - initialize
  - append
  - verify
  - export
  - generate proofs
  - generate checkpoints
- Core audit story:
  - tamper evidence
  - deterministic verification
  - no required sidecar service
- Product positioning:
  - edge/embedded security logging
  - compliance-friendly evidence layer
  - future-proof path toward privacy and zero-knowledge features

## Current gaps

- Append durability still needs explicit `fsync` hardening and crash-recovery tests.
- Proof generation is correct but still uses a simple full-leaf load path.
- The admin console is local-first and intentionally unauthenticated for prototype speed.
- Differential privacy and energy scheduling exist as early primitives, not polished product modules yet.
- No external anchoring backend or third-party verifier has been published yet.

## Delivery posture

The project is now in the right state for:

- internal demos
- architecture reviews
- early partner conversations
- technical validation benchmarks

It is **not** yet in the right state for:

- production deployment on critical fleets
- hostile-network exposure of the admin surface
- claims about benchmarked performance or formal privacy guarantees

## Near-term objective

Turn the prototype into a pilot-ready system by focusing on:

1. storage hardening
2. benchmark harnesses
3. external verification tooling
4. clearer packaging of the novel privacy and proof roadmap
