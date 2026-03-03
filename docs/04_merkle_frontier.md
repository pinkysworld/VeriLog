# Incremental Merkle Frontier (fixed-height tree)

VeriLog uses an **incremental Merkle frontier** to maintain a fixed-height Merkle root
efficiently as new leaves are appended.

This structure is common in append-only systems (and ZK systems) because it supports:
- O(height) incremental updates per leaf
- constant-size state on the device (frontier nodes)
- deterministic roots with zero padding

## Parameters

- Tree height: `H` (default 32)
- Leaves: indexed `0..(2^H - 1)`
- Empty leaves: treated as a deterministic “zero leaf hash”

We precompute `zero_hashes[level]`:

- `zero_hashes[0] = hash_bytes([0u8; 32])` (or simply `[0; 32]` depending on convention)
- `zero_hashes[i+1] = hash_pair(zero_hashes[i], zero_hashes[i])`

## Frontier state

The frontier holds `H` optional nodes:

- `frontier[level]` is either:
  - `None`, or
  - the root hash of a completed subtree of size `2^level` that ends at the current leaf count.

## Insert algorithm (append leaf)

When inserting a new leaf hash `x`:

```
node = x
for level in 0..H:
  if frontier[level] is None:
    frontier[level] = node
    break
  else:
    node = hash_pair(frontier[level], node)
    frontier[level] = None
```

This is binary “carry” behavior.

## Root computation

To compute the fixed-height root after `n` leaves:

```
hash = zero_hashes[0]
idx = n
for level in 0..H:
  if idx is odd:
    hash = hash_pair(frontier[level], hash)
  else:
    hash = hash_pair(hash, zero_hashes[level])
  idx >>= 1
root = hash
```

This yields a root that matches a full tree of height `H` where:
- leaves `0..n-1` are the inserted leaf hashes
- leaves `n..2^H-1` are zero leaves

## Membership proofs

The base edition outputs a classic Merkle inclusion proof:

- leaf index `i`
- leaf hash `L`
- sibling hashes at each level `siblings[0..H-1]`
- expected root

Verifier recomputes:

```
node = L
for level in 0..H:
  if bit(i, level) == 0:
     node = hash_pair(node, siblings[level])
  else:
     node = hash_pair(siblings[level], node)
assert node == root
```

In the reference implementation, sibling nodes are computed from `leaves.bin` with memoization.
This is not yet optimized for large stores; enterprise builds may want:
- persisted internal nodes
- MMR proofs
- ZK proofs of membership/ranges

