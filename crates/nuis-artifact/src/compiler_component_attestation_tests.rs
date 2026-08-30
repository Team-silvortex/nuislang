use std::path::Path;

use ed25519_dalek::SigningKey;

use super::*;
use crate::{
    build_compiler_component_attester_trust_registry,
    compiler_component_attester_trust_registry_sha256,
    compiler_component_reproducibility::build_from_runs,
    parse_compiler_component_attester_trust_registry_from_source,
    render_compiler_component_attester_trust_registry, render_compiler_component_reproducibility,
    CompilerComponentAttesterTrustEntryInput, CompilerComponentReproducibilityRun,
};

fn hash(byte: char) -> String {
    std::iter::repeat_n(byte, 64).collect()
}

fn run(ordinal: usize, witness: char) -> CompilerComponentReproducibilityRun {
    CompilerComponentReproducibilityRun {
        ordinal,
        run_id: format!("clean-build-{ordinal}"),
        component_id: "projection_relay".to_owned(),
        clean_root_state: "absent-or-empty-before-build".to_owned(),
        clean_root_witness_sha256: hash(witness),
        stage0_record_sha256: hash(if ordinal == 0 { '1' } else { '2' }),
        stage0_reproducible_build_sha256: hash('3'),
        candidate_record_sha256: hash(if ordinal == 0 { '4' } else { '5' }),
        candidate_reproducible_build_sha256: hash('6'),
        candidate_compiler_image_sha256: hash('7'),
        native_output_sha256: hash('8'),
        production_proof_sha256: hash(if ordinal == 0 { '9' } else { 'a' }),
        differential_report_sha256: hash(if ordinal == 0 { 'b' } else { 'c' }),
        comparison_count: 13,
        equivalent_count: 13,
        deterministic_artifact_equivalent: true,
        differential_verdict: "equivalent-awaiting-authorization".to_owned(),
        replacement_authorized: false,
    }
}

fn report() -> CompilerComponentReproducibility {
    build_from_runs(vec![run(0, 'd'), run(1, 'e')]).expect("build report")
}

fn signing_key_hex(seed: u8) -> String {
    std::iter::repeat_n(format!("{seed:02x}"), 32).collect()
}

fn public_key_hex(seed: u8) -> String {
    SigningKey::from_bytes(&[seed; 32])
        .verifying_key()
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn registry(seed: u8, status: &str) -> (CompilerComponentAttesterTrustRegistry, String, String) {
    let public_key = public_key_hex(seed);
    let registry = build_compiler_component_attester_trust_registry(
        3,
        &[CompilerComponentAttesterTrustEntryInput {
            attester_id: "linux-builder-1",
            environment_id: "linux-amd64-cleanroom",
            public_key_hex: &public_key,
            status,
        }],
    )
    .expect("build registry");
    let source = render_compiler_component_attester_trust_registry(&registry);
    let sha256 = compiler_component_attester_trust_registry_sha256(&source);
    (registry, source, sha256)
}

fn attestation() -> (
    CompilerComponentAttestation,
    CompilerComponentReproducibility,
    String,
) {
    let report = report();
    let source = render_compiler_component_reproducibility(&report);
    let attestation = build_compiler_component_attestation(
        CompilerComponentAttestationInput {
            reproducibility: &report,
            reproducibility_source: &source,
            challenge_sha256: &hash('f'),
            attester_id: "linux-builder-1",
            environment_id: "linux-amd64-cleanroom",
        },
        &signing_key_hex(7),
    )
    .expect("build attestation");
    (attestation, report, source)
}

#[test]
fn signed_attestation_round_trips_and_verifies_against_pinned_registry() {
    let (attestation, report, report_source) = attestation();
    let source = render_compiler_component_attestation(&attestation);
    assert!(!source.contains("/Users/"));
    assert!(!source.contains("timestamp"));
    assert_eq!(
        attestation.candidate_production_protocol,
        "nuis-compiler-candidate-production-v11"
    );
    assert_eq!(attestation.first_production_proof_sha256, hash('9'));
    assert_eq!(attestation.second_production_proof_sha256, hash('a'));
    assert!(!attestation.replacement_authorized);

    let parsed = parse_compiler_component_attestation_from_source(
        &source,
        Path::new(COMPILER_COMPONENT_ATTESTATION_FILE),
    )
    .expect("parse attestation");
    assert_eq!(parsed, attestation);
    let (trusted_registry, registry_source, registry_sha256) = registry(7, "active");
    verify_compiler_component_attestation(
        &parsed,
        &report,
        &report_source,
        &trusted_registry,
        &registry_source,
        &registry_sha256,
        &hash('f'),
    )
    .expect("verify attestation");
}

#[test]
fn signature_challenge_registry_pin_and_revocation_fail_closed() {
    let (attestation, report, report_source) = attestation();
    let (trusted_registry, registry_source, registry_sha256) = registry(7, "active");

    let mut signature_tamper = attestation.clone();
    signature_tamper.signature_hex.replace_range(0..2, "00");
    let error = verify_compiler_component_attestation(
        &signature_tamper,
        &report,
        &report_source,
        &trusted_registry,
        &registry_source,
        &registry_sha256,
        &hash('f'),
    )
    .expect_err("signature tampering must fail");
    assert!(error.to_string().contains("signature mismatch"));

    let error = verify_compiler_component_attestation(
        &attestation,
        &report,
        &report_source,
        &trusted_registry,
        &registry_source,
        &registry_sha256,
        &hash('e'),
    )
    .expect_err("challenge replay must fail");
    assert!(error.to_string().contains("verifier request"));

    let error = verify_compiler_component_attestation(
        &attestation,
        &report,
        &report_source,
        &trusted_registry,
        &registry_source,
        &hash('0'),
        &hash('f'),
    )
    .expect_err("unpinned registry must fail");
    assert!(error.to_string().contains("pinned SHA-256"));

    let (revoked, revoked_source, revoked_sha256) = registry(7, "revoked");
    let error = verify_compiler_component_attestation(
        &attestation,
        &report,
        &report_source,
        &revoked,
        &revoked_source,
        &revoked_sha256,
        &hash('f'),
    )
    .expect_err("revoked attester must fail");
    assert!(error.to_string().contains("revoked"));
}

#[test]
fn claim_or_bound_aggregate_tampering_fails_before_trust_is_granted() {
    let (attestation, report, report_source) = attestation();
    let source = render_compiler_component_attestation(&attestation);
    let tampered_claim = source.replacen(&hash('9'), &hash('0'), 1);
    let error = parse_compiler_component_attestation_from_source(
        &tampered_claim,
        Path::new(COMPILER_COMPONENT_ATTESTATION_FILE),
    )
    .expect_err("claim identity tampering must fail");
    assert!(error.to_string().contains("proof identity mismatch"));

    let (registry, registry_source, registry_sha256) = registry(7, "active");
    let tampered_report = report_source.replacen(&hash('8'), &hash('0'), 1);
    let error = verify_compiler_component_attestation(
        &attestation,
        &report,
        &tampered_report,
        &registry,
        &registry_source,
        &registry_sha256,
        &hash('f'),
    )
    .expect_err("aggregate source tampering must fail");
    assert!(error.to_string().contains("stable aggregate identity"));
}

#[test]
fn registry_is_canonical_sorted_and_environment_scoped() {
    let first_key = public_key_hex(7);
    let second_key = public_key_hex(9);
    let registry = build_compiler_component_attester_trust_registry(
        4,
        &[
            CompilerComponentAttesterTrustEntryInput {
                attester_id: "builder-z",
                environment_id: "linux-arm64",
                public_key_hex: &second_key,
                status: "active",
            },
            CompilerComponentAttesterTrustEntryInput {
                attester_id: "builder-a",
                environment_id: "linux-amd64",
                public_key_hex: &first_key,
                status: "active",
            },
        ],
    )
    .expect("build sorted registry");
    assert_eq!(registry.entries[0].attester_id, "builder-a");
    let source = render_compiler_component_attester_trust_registry(&registry);
    let parsed = parse_compiler_component_attester_trust_registry_from_source(
        &source,
        Path::new("registry.toml"),
    )
    .expect("parse registry");
    assert_eq!(parsed, registry);
}
