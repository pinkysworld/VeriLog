# Full Codex Instructions (drop-in prompt)

Copy/paste the following into your coding agent as the “project instruction” block.

---

You are contributing to **VeriLog (OSS base edition)**.

## Non-negotiables

1. **Do not add enterprise implementations to this repo.**
   - You may add trait definitions, feature IDs, and stub wiring.
   - You may not add ZK circuits, compliance exporters, PSI protocols, etc. unless they are strictly non-functional stubs.
2. Preserve log verifiability:
   - do not change the signed bytes without bumping versions and updating verifier
3. Keep file formats versioned, stable, and documented.

## Architecture constraints

- Single executable `verilogd` is the deliverable.
- `verilog-core` must not require network access.
- `verilog-license` must work offline.
- Admin console must be optional behind a Cargo feature.

## Code quality rules

- Library code must not panic on malformed input.
- Avoid allocations in hot paths where possible.
- All cryptographic operations must be explicit and test-covered.

## Tasks you are allowed to do

- Improve storage integrity, crash consistency, performance.
- Add better membership proof generation.
- Add tests and golden vectors.
- Improve docs and diagrams.
- Add new base features that are explicitly OSS.

## Tasks you are NOT allowed to do

- Implement proprietary enterprise features (ZK, PSI, compliance templates).
- Add obfuscation/DRM that tries to prevent reverse engineering of OSS.
- Add network calls for licensing that make the OSS edition unusable offline.

## When editing log formats

- bump `LogEntry.version` and `Meta.version`
- update `docs/02_storage_format.md`
- add migration guidance in `docs/08_roadmap.md`

## Security checklist for PRs

- No secrets in repo
- No keys printed
- Constant-time comparisons for signature bytes if you add any
- Verify all error paths return `Result`

--- 

