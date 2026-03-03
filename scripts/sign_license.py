#!/usr/bin/env python3
"""Convenience wrapper around `verilogd license issue`.

This script is optional. You can also run the CLI directly:

  cargo run -p verilogd -- license issue --vendor-seed-b64 <...> --issued-to "ACME" --entitle zk_proofs --out license.json

"""

import argparse
import subprocess
import sys

def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--vendor-seed-b64", required=True, help="Base64 Ed25519 vendor seed")
    ap.add_argument("--issued-to", required=True)
    ap.add_argument("--org", default="")
    ap.add_argument("--not-before-unix-ms", type=int, required=True)
    ap.add_argument("--not-after-unix-ms", type=int, required=True)
    ap.add_argument("--device-id", default="", help="Optional device binding")
    ap.add_argument("--entitle", action="append", default=[], help="Repeatable entitlement (EnterpriseFeature id)")
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    cmd = [
        "cargo", "run", "-p", "verilogd", "--",
        "license", "issue",
        "--vendor-seed-b64", args.vendor_seed_b64,
        "--issued-to", args.issued_to,
        "--org", args.org,
        "--not-before-unix-ms", str(args.not_before_unix_ms),
        "--not-after-unix-ms", str(args.not_after_unix_ms),
        "--out", args.out,
    ]
    if args.device_id:
        cmd += ["--device-id", args.device_id]
    for e in args.entitle:
        cmd += ["--entitle", e]

    print("Running:", " ".join(cmd))
    return subprocess.call(cmd)

if __name__ == "__main__":
    raise SystemExit(main())
