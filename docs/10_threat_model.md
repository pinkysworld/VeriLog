# Threat Model (starter)

This is a starter threat model for VeriLog. Expand before production.

## Assets

- Integrity of historical logs
- Authenticity of log producer (device signing key)
- Privacy of sensitive telemetry
- Correctness of exported proofs

## Adversaries

1. **Local attacker** with filesystem access (but no signing key)
2. **Post-compromise attacker** who gains access after time T
3. **Supply-chain attacker** who modifies binaries before deployment
4. **Malicious operator** who tries to hide incidents by truncating logs
5. **Network attacker** (for forwarding/sync features)

## Security goals

- Tampering with stored logs must be detectable by third parties.
- Entries must be attributable to a signing identity (keypair).
- Truncation or deletion must be detectable unless explicitly supported by a deletion protocol.
- Verification must not require secret keys.

## Non-goals (base edition)

- Preventing a fully compromised device from generating false logs *after compromise*
  - VeriLog provides **tamper evidence**, not magical truth.
- Hiding the existence of logging.
- Full read-access pattern obfuscation (R11 is research/enterprise).

## Major attack scenarios and mitigations

### Modify an entry in the middle

Mitigation:
- hash chain breaks (`prev_entry_hash`)
- signature fails
- Merkle roots diverge

### Truncate the log

Mitigation:
- last known checkpoint root (R10) can detect truncation
- external anchoring strengthens this (enterprise)

### Replay old logs

Mitigation:
- timestamps + monotonic index
- optional device clock sanity checks
- external time-stamping (enterprise)

### Steal signing key

Mitigation:
- not fully solvable in software
- recommend secure element or OS key store
- consider key rotation and forward-secure ratchets (future)

## Next steps

- Add explicit key lifecycle
- Add crash consistency design
- Add external anchoring design (optional)
- Add privacy threat model (DP, metadata leakage)

