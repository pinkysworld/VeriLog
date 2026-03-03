# Admin Console (optional)

The research vision calls for an embedded admin console with one panel per research track.

In this repo, the admin console is **optional** and compiled only with:

```bash
cargo build -p verilogd --features admin-console
```

## Current prototype

The current admin console is now an **MVP dashboard** rather than a placeholder page.

It provides:

- live store status
- latest-entry summary
- on-demand full verification
- membership proof lookup
- signed checkpoint preview
- research track visibility for demos and planning

## Goals

- Provide a lightweight HTTP API for:
  - store status
  - integrity verification
  - proof generation
  - checkpoint generation
  - configuration and track instrumentation
- Provide a lightweight HTML UI that links the operator-facing flows to the research program.

## Suggested endpoints

Base edition endpoints (implemented as stubs or minimal):

- `GET /api/status`
  - leaf count
  - tree height
  - current Merkle root
  - last timestamp
  - license status (entitlements)
- `POST /api/verify`
  - run verification, return report
- `GET /api/checkpoint`
  - return a signed checkpoint snapshot for the current store state
- `GET /api/proofs/membership?index=...`
  - return membership proof JSON
- `GET /api/research/tracks`
  - list tracks + status
- `GET /api/research/track/{id}`
  - configuration + recent metrics for a track (placeholder)

Enterprise-only endpoints (hooks only):
- ZK range proof generation
- compliance exports
- mesh sync dashboards

## Security considerations

- Do not expose private keys.
- Require authentication in production:
  - local-only binding by default
  - or mutually authenticated TLS
- Rate-limit proof generation.
- Treat the HTML UI as a trusted-local surface only; do not expose it on the public Internet without authentication and transport security.
