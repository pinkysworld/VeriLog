# Cryptography

This doc describes the cryptographic primitives used in the base edition.

## Hashing

- **BLAKE3** is used for:
  - `entry_hash`
  - Merkle node hashing (`hash_pair(left, right)`)

Rationale:
- fast on embedded and x86/ARM
- strong modern cryptographic hash
- widely used and reviewed

## Signatures

- **Ed25519** signatures per entry.
- Each store has a signing keypair generated at `verilogd init`.

Signing scheme:

- message = `entry_hash` (32 bytes)
- signature = `ed25519.sign(message)`
- verify = `ed25519.verify(message, signature)`

## Merkle commitment

The base edition uses a **fixed-height incremental Merkle tree** (height 32 by default) with:
- leaf hash = `entry_hash`
- empty leaves = “zero hashes” (deterministically derived by hashing zeros up the tree)

The root is stored in every entry (`merkle_root`) and re-derived during verification.

See `docs/04_merkle_frontier.md`.

## Key management (base edition)

The reference implementation stores:

- private seed in `signing_key.json`
- verifying key in the same file for convenience

Production hardening ideas (not implemented here):
- encrypt private seed at rest
- hardware-backed keys (TPM / Secure Enclave / SE050 / TrustZone)
- key rotation with forward-secure chaining

## ZK-friendly hashing (enterprise)

Your research plan references Poseidon for ZK circuits.  
The OSS base does **not** ship Poseidon or ZK code.

Recommended boundary:
- keep BLAKE3 for the OSS store
- in enterprise builds, provide an additional *ZK commitment* pipeline if required
- alternatively, keep the store commitments BLAKE3 and build a Halo2 circuit for BLAKE3 (heavier)

