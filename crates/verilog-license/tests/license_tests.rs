use verilog_enterprise_api::EnterpriseFeature;
use verilog_license::{
    generate_vendor_keypair_b64, issue_license, verify_license, LicensePayload, LICENSE_VERSION,
};

#[test]
fn license_roundtrip() {
    let (seed_b64, pub_b64) = generate_vendor_keypair_b64();

    let payload = LicensePayload {
        version: LICENSE_VERSION,
        license_id: "test-1".into(),
        issued_to: "UnitTest".into(),
        org: "Org".into(),
        not_before_unix_ms: 0,
        not_after_unix_ms: u64::MAX,
        device_id: Some("device123".into()),
        entitlements: vec![EnterpriseFeature::ZkIntegrityProofs],
    };

    let signed = issue_license(&seed_b64, payload).expect("issue");
    let verified = verify_license(&pub_b64, &signed, 1234, Some("device123")).expect("verify");
    assert!(verified.entitles(EnterpriseFeature::ZkIntegrityProofs));
}
