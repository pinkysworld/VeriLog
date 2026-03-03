# Novelty Program

The existing research agenda is broad. This document narrows it into the parts
that make VeriLog meaningfully different rather than merely “another secure log.”

## Core novelty thesis

VeriLog becomes novel when it combines four properties in one deployable edge
system:

1. **Single-binary embedded deployment**
2. **Cryptographically verifiable append-only evidence**
3. **Privacy-preserving telemetry controls**
4. **A migration path to zero-knowledge and multi-device proofs**

Most products stop at one or two of these. The differentiation comes from
treating them as one protocol family instead of separate tools.

## Differentiation wedges

### Wedge A: Evidence-first edge telemetry

The strongest immediate story is not “logging,” but **evidence capture on
unreliable devices**.

That means the system should optimize for:

- tamper detection after compromise
- portable proof bundles
- external anchoring via checkpoints
- third-party validation without access to device secrets

This wedge is already partially realized in the prototype and should stay the
public foundation.

### Wedge B: Privacy-preserving observability

Differential privacy is only interesting here if it is attached to the same
proof surface as integrity.

The target direction:

- DP noise and budget usage become part of the operational story
- later, proofs can attest that privacy controls were applied correctly
- auditors can verify policy conformance without seeing raw telemetry

This is a strong “trust + privacy” narrative that few embedded logging systems
can credibly offer.

### Wedge C: Verifiable archival and selective disclosure

Signed checkpoints create a bridge from local logs to:

- long-term archives
- external timestamping
- third-party compliance review
- selective disclosure workflows

This is the practical stepping stone toward later ZK range proofs. It should be
treated as a first-class product feature, not just research scaffolding.

### Wedge D: Multi-device evidence graphs

The long-term research leap is moving from one device’s log to a **network of
correlated evidence transcripts**.

This is where the project can become genuinely hard to replicate:

- privacy-preserving correlation
- verifiable forwarding
- mesh sync
- federation across fleets

It should stay out of the OSS core implementation for now, but the public docs
should keep pointing toward it because it explains the larger ambition.

## Recommended research sequence

The highest-leverage order is:

1. checkpoint formalization and external anchoring
2. proof-friendly compression and indexing
3. stronger DP schemas plus accounting surfaces
4. parallel Poseidon commitment experiment
5. small-scope ZK range proof prototype
6. multi-device transcript and correlation primitives

This order keeps the work grounded in product value while opening the path to
the genuinely novel cryptographic features.

## Validation program

To make the novelty claims credible, each stage should ship with evidence:

- a benchmark
- a verifier artifact
- a demo transcript
- a clear threat assumption

Without that, the work remains an idea rather than a defensible product thesis.

## Paper and product angles

VeriLog has at least three strong future publication or pitch angles:

- **Edge Evidence Layer:** verifiable append-only logging for constrained devices
- **Private Audit Telemetry:** combining DP telemetry with proof-carrying audit trails
- **Selective-Disclosure Fleet Evidence:** checkpointed logs plus ZK-backed range claims

The product should keep all three visible, but prioritize the first one for
shipping credibility.
