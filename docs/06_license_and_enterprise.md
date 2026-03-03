# Licensing, Monetization, and Enterprise Extensions

This repository contains the **full monetization foundation** (license verification + entitlements),
while intentionally excluding proprietary enterprise implementations.

## Design goals

- Enterprise features are **not** shipped in OSS source form.
- Enterprise features can be:
  - compiled in via a **private crate** (`verilog-enterprise`)
  - and/or delivered as a separately distributed module (future)
- The OSS binary must:
  - run fully without a license
  - deny enterprise-only commands without a valid license
  - not leak any proprietary logic

## License model (offline-first)

A license file is:

- JSON payload (license fields)
- Ed25519 signature over the canonical serialized payload bytes
- VeriLog embeds the **vendor public key**, so verification is offline

### Typical fields

- `license_id`
- `issued_to` / `org`
- `not_before_unix_ms`
- `not_after_unix_ms`
- `device_binding` (optional)
- `entitlements`: list of `EnterpriseFeature` IDs

See `crates/verilog-license/src/license.rs`.

## Where the license is stored

The CLI installs a license into the store directory:

- `<store>/license.json`

This is intentional for embedded/edge deployments where “home directories” may not exist.

## Gating rules

- If no license is present: **enterprise features are disabled**
- If a license is present:
  - verify signature
  - enforce time window
  - enforce optional device binding
  - allow only entitled features

## Keeping enterprise code private (recommended approach)

### Option A: Private crate compiled with `--features enterprise`

1. Create a private crate named `verilog-enterprise` that depends on:
   - `verilog-core`
   - `verilog-enterprise-api`
2. In your private crate, implement the hooks traits.
3. Build the binary with:

```bash
cargo build -p verilogd --features enterprise
```

In this OSS repo, the `enterprise` feature compiles only **stub wiring** by default.
Your private repo would override that by adding a workspace member or Cargo patch.

### Option B: Separate distribution module (future)

You may later load enterprise modules as:
- Wasm (portable, sandboxable)
- statically linked components in an enterprise build
- sealed/encrypted payloads (requires careful threat modeling)

This repo does not implement runtime module loading yet.

## “Why not just hide code behind feature flags?”

Because feature flags still ship source code.
The separation here is explicit:

- OSS repo: only API + gates
- Private repo: proprietary implementations

## Hardening suggestions

- Add a “grace period” and “clock sanity” checks for devices without trusted time
- Implement optional online activation (but keep offline mode for field deployments)
- Bind licenses to:
  - device hardware IDs
  - public signing key fingerprints
  - secure element attestation

