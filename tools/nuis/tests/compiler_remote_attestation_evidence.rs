use std::path::PathBuf;

use nuis_artifact::{
    compiler_component_attester_trust_registry_sha256,
    parse_compiler_component_attester_trust_registry, read_compiler_component_attestation,
};

const CHALLENGE_SHA256: &str = "d5aeef8c1d33a5b473f11142197fd361df26dc0f2ec1c0188362f9ece139338c";
const REGISTRY_SHA256: &str = "90b8f7f4c9d336c72caa7dc4dc9a91c41ec263a7bfffa282ee8211088b164f01";

fn evidence_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("docs/evidence/compiler-attestation/linux-amd64-cleanroom/generation-1")
}

#[test]
fn checked_in_remote_attestation_verifies_and_fails_closed() {
    let evidence = evidence_dir();
    let aggregate = evidence.join("nuis.compiler-component-reproducibility.toml");
    let attestation = evidence.join("nuis.compiler-component-attestation.toml");
    let registry = evidence.join("nuis.compiler-component-attester-trust-registry.toml");

    let registry_source = std::fs::read_to_string(&registry).expect("read trust registry");
    assert_eq!(
        compiler_component_attester_trust_registry_sha256(&registry_source),
        REGISTRY_SHA256
    );
    let parsed_registry = parse_compiler_component_attester_trust_registry(&registry)
        .expect("parse canonical trust registry");
    assert_eq!(parsed_registry.generation, 1);
    assert_eq!(parsed_registry.entries.len(), 1);

    let claim = read_compiler_component_attestation(
        &attestation,
        &aggregate,
        &registry,
        REGISTRY_SHA256,
        CHALLENGE_SHA256,
    )
    .expect("verify checked-in remote attestation");
    assert_eq!(claim.attester_id, "kyuubiki-lab-1");
    assert_eq!(claim.environment_id, "linux-amd64-cleanroom");
    assert_eq!(claim.run_count, 2);
    assert!(!claim.replacement_authorized);

    let wrong_challenge = "0".repeat(64);
    let error = read_compiler_component_attestation(
        &attestation,
        &aggregate,
        &registry,
        REGISTRY_SHA256,
        &wrong_challenge,
    )
    .expect_err("challenge replay must fail closed");
    assert!(error.to_string().contains("verifier request"));

    let wrong_registry_pin = "0".repeat(64);
    let error = read_compiler_component_attestation(
        &attestation,
        &aggregate,
        &registry,
        &wrong_registry_pin,
        CHALLENGE_SHA256,
    )
    .expect_err("unpinned registry bytes must fail closed");
    assert!(error.to_string().contains("pinned SHA-256"));
}
