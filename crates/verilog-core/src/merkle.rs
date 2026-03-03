use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    error::VeriLogError,
    hash::{hash_pair, zero_hashes, Hash32},
};

pub const DEFAULT_TREE_HEIGHT: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleSnapshot {
    pub tree_height: u8,
    pub leaf_count: u64,
    /// Frontier nodes per level; base64 encoding handled by serde for bytes arrays.
    pub frontier: Vec<Option<Hash32>>,
}

/// Fixed-height incremental Merkle frontier with zero padding.
#[derive(Debug, Clone)]
pub struct MerkleFrontier {
    tree_height: usize,
    zero: Vec<Hash32>,
    frontier: Vec<Option<Hash32>>,
    leaf_count: u64,
}

impl MerkleFrontier {
    pub fn new(tree_height: usize) -> Self {
        let zero = zero_hashes(tree_height);
        Self {
            tree_height,
            zero,
            frontier: vec![None; tree_height],
            leaf_count: 0,
        }
    }

    pub fn from_snapshot(snapshot: MerkleSnapshot) -> Result<Self, VeriLogError> {
        let h = snapshot.tree_height as usize;
        if snapshot.frontier.len() != h {
            return Err(VeriLogError::Format(format!(
                "snapshot frontier length {} != height {}",
                snapshot.frontier.len(),
                h
            )));
        }
        let zero = zero_hashes(h);
        Ok(Self {
            tree_height: h,
            zero,
            frontier: snapshot.frontier,
            leaf_count: snapshot.leaf_count,
        })
    }

    pub fn snapshot(&self) -> MerkleSnapshot {
        MerkleSnapshot {
            tree_height: self.tree_height as u8,
            leaf_count: self.leaf_count,
            frontier: self.frontier.clone(),
        }
    }

    pub fn tree_height(&self) -> usize {
        self.tree_height
    }

    pub fn leaf_count(&self) -> u64 {
        self.leaf_count
    }

    /// Push a new leaf hash, updating the frontier, returning the new root.
    pub fn push(&mut self, leaf: Hash32) -> Result<Hash32, VeriLogError> {
        // Prevent overflow: fixed-height tree has capacity 2^H leaves.
        if self.tree_height >= 64 {
            return Err(VeriLogError::Format("tree_height must be < 64".into()));
        }
        let capacity = 1u64 << self.tree_height;
        if self.leaf_count >= capacity {
            return Err(VeriLogError::Format("merkle tree is full".into()));
        }
        let mut node = leaf;
        let mut level = 0usize;

        while level < self.tree_height {
            match self.frontier[level] {
                None => {
                    self.frontier[level] = Some(node);
                    break;
                }
                Some(left) => {
                    self.frontier[level] = None;
                    node = hash_pair(&left, &node);
                    level += 1;
                }
            }
        }

        self.leaf_count += 1;
        self.root()
    }

    /// Compute current fixed-height root.
    pub fn root(&self) -> Result<Hash32, VeriLogError> {
        let mut acc = self.zero[0];
        let mut idx = self.leaf_count;

        for level in 0..self.tree_height {
            if (idx & 1) == 1 {
                let left = self.frontier[level].ok_or_else(|| {
                    VeriLogError::Format(format!(
                        "corrupt merkle frontier: missing node at level {} for odd idx",
                        level
                    ))
                })?;
                acc = hash_pair(&left, &acc);
            } else {
                acc = hash_pair(&acc, &self.zero[level]);
            }
            idx >>= 1;
        }

        Ok(acc)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub tree_height: u8,
    pub leaf_index: u64,
    pub leaf_hash: Hash32,
    pub siblings: Vec<Hash32>, // len == tree_height
    pub root: Hash32,
    pub leaf_count: u64,
}

impl MerkleProof {
    pub fn verify(&self) -> Result<bool, VeriLogError> {
        if self.siblings.len() != self.tree_height as usize {
            return Err(VeriLogError::Format(format!(
                "siblings len {} != tree height {}",
                self.siblings.len(),
                self.tree_height
            )));
        }
        let mut node = self.leaf_hash;
        let idx = self.leaf_index;

        for (level, sib) in self.siblings.iter().enumerate() {
            let bit = (idx >> level) & 1;
            node = if bit == 0 {
                hash_pair(&node, sib)
            } else {
                hash_pair(sib, &node)
            };
        }

        Ok(node == self.root)
    }
}

/// Compute a membership proof for `leaf_index` given all leaf hashes.
///
/// This reference implementation computes sibling hashes via recursive subtree hashing with memoization.
/// It is designed for correctness and clarity over maximum performance.
pub fn membership_proof_from_leaves(
    leaves: &[Hash32],
    tree_height: usize,
    leaf_index: u64,
) -> Result<MerkleProof, VeriLogError> {
    let leaf_count = leaves.len() as u64;
    if leaf_index >= leaf_count {
        return Err(VeriLogError::Format(format!(
            "leaf_index {} out of range (leaf_count {})",
            leaf_index, leaf_count
        )));
    }

    let zero = zero_hashes(tree_height);

    // Memoize subtree hashes: key = (level, start_index)
    let mut memo: HashMap<(usize, u64), Hash32> = HashMap::new();

    fn subtree_hash(
        leaves: &[Hash32],
        zero: &[Hash32],
        memo: &mut HashMap<(usize, u64), Hash32>,
        level: usize,
        start: u64,
    ) -> Hash32 {
        if let Some(v) = memo.get(&(level, start)) {
            return *v;
        }

        let leaf_count_u64 = leaves.len() as u64;
        if start >= leaf_count_u64 {
            // Entire subtree is outside the inserted leaves -> it is the deterministic zero hash at this level.
            let z = zero[level];
            memo.insert((level, start), z);
            return z;
        }

        let out = if level == 0 {
            let idx = start as usize;
            if idx < leaves.len() {
                leaves[idx]
            } else {
                zero[0]
            }
        } else {
            let half = 1u64 << (level - 1);
            let left = subtree_hash(leaves, zero, memo, level - 1, start);
            let right = subtree_hash(leaves, zero, memo, level - 1, start + half);
            hash_pair(&left, &right)
        };

        memo.insert((level, start), out);
        out
    }

    let mut siblings = Vec::with_capacity(tree_height);

    for level in 0..tree_height {
        let mask = !((1u64 << level) - 1);
        let start = leaf_index & mask;
        let sibling_start = start ^ (1u64 << level);
        let sib = subtree_hash(leaves, &zero, &mut memo, level, sibling_start);
        siblings.push(sib);
    }

    // Compute root from leaf+siblings (same as verify)
    let mut node = leaves[leaf_index as usize];
    for (level, sib) in siblings.iter().enumerate() {
        let bit = (leaf_index >> level) & 1;
        node = if bit == 0 {
            hash_pair(&node, sib)
        } else {
            hash_pair(sib, &node)
        };
    }
    let root = node;

    Ok(MerkleProof {
        tree_height: tree_height as u8,
        leaf_index,
        leaf_hash: leaves[leaf_index as usize],
        siblings,
        root,
        leaf_count,
    })
}
