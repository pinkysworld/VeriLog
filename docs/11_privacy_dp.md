# Privacy & Differential Privacy (base edition)

VeriLog’s research vision includes strong privacy guarantees. This OSS base edition includes a **reference implementation**
of event-level DP for numeric telemetry.

## What is implemented

- A simple **token bucket** privacy budget manager:
  - capacity ε
  - optional refill rate
- Laplace noise for numeric values:
  - scale b = sensitivity / ε

## What is NOT implemented (yet)

- Formal privacy accounting across heterogeneous event types
- Secure/constant-time sampling guarantees
- ZK proofs of correct DP application
- DP for arbitrary structured events beyond numeric telemetry schemas

## Practical guidance

- Start by applying DP only to low-dimensional numeric metrics.
- Keep raw values out of the log if you can.
- Store only:
  - DP-noised value
  - ε spent (as metadata)
  - the schema ID / mechanism parameters (without leaking raw values)

## Metadata leakage

Even if you add DP to payload values, metadata can leak:
- event type frequency
- timestamps
- anomaly triggers

Mitigation strategies (research):
- batching
- random delays
- oblivious logging modes (R11)

