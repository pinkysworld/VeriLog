#!/usr/bin/env bash
set -euo pipefail

# Generates a vendor keypair for signing licenses.
# Keep the private key OFFLINE.

cargo run -p verilogd -- license vendor-keygen --out-dir ./vendor_keys
echo "Vendor keys written to ./vendor_keys"
