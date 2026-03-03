use std::{fs, path::PathBuf};

use verilog_core::{LogLevel, LogStore};

fn temp_store_dir(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "verilog-{name}-{}-{}",
        std::process::id(),
        verilog_core::util::now_unix_ms()
    ));
    dir
}

#[test]
fn store_roundtrip_and_checkpoint_verification() {
    let dir = temp_store_dir("store-roundtrip");
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("remove pre-existing temp dir");
    }

    LogStore::init(&dir, Some(8)).expect("init");

    let mut store = LogStore::open(&dir).expect("open");
    store
        .append(
            "metric",
            br#"{"name":"temp_c","value":21.7}"#.to_vec(),
            LogLevel::Info,
        )
        .expect("append metric");
    store
        .append(
            "event",
            br#"{"event":"door_open"}"#.to_vec(),
            LogLevel::Warn,
        )
        .expect("append event");

    let reopened = LogStore::open(&dir).expect("re-open");
    let report = reopened.verify_store().expect("verify");
    assert!(report.ok);
    assert_eq!(report.leaf_count, 2);

    let proof = reopened.membership_proof(1).expect("proof");
    assert!(proof.verify().expect("proof verify"));

    let checkpoint = reopened
        .create_checkpoint(Some("integration-test".into()))
        .expect("create checkpoint");
    checkpoint.verify().expect("checkpoint verify");
    assert_eq!(checkpoint.checkpoint.leaf_count, 2);
    assert_eq!(
        checkpoint.checkpoint.label.as_deref(),
        Some("integration-test")
    );

    fs::remove_dir_all(&dir).expect("cleanup");
}
