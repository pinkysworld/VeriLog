use verilog_core::hash::hash_bytes;
use verilog_core::merkle::{membership_proof_from_leaves, MerkleFrontier};

#[test]
fn merkle_frontier_matches_membership_root() {
    let height = 8;
    let mut tree = MerkleFrontier::new(height);

    let mut leaves = Vec::new();
    for i in 0..25u64 {
        let leaf = hash_bytes(format!("leaf-{i}").as_bytes());
        leaves.push(leaf);
        let _ = tree.push(leaf).expect("push");
    }

    let root1 = tree.root().expect("root");
    let proof = membership_proof_from_leaves(&leaves, height, 10).expect("proof");
    assert_eq!(root1, proof.root);
    assert!(proof.verify().expect("verify"));
}
