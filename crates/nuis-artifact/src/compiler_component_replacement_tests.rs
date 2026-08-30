use std::path::Path;

use ed25519_dalek::SigningKey;

use super::*;
use crate::{
    build_compiler_component_attestation, build_compiler_component_attester_trust_registry,
    build_compiler_component_replacement_authorizer_registry,
    compiler_component_attester_trust_registry_sha256,
    compiler_component_replacement_authorizer_registry_sha256,
    compiler_component_reproducibility::build_from_runs, render_compiler_component_attestation,
    render_compiler_component_attester_trust_registry,
    render_compiler_component_replacement_authorizer_registry,
    render_compiler_component_reproducibility, CompilerComponentAttestationInput,
    CompilerComponentAttesterTrustEntryInput, CompilerComponentReplacementAuthorizerEntryInput,
    CompilerComponentReproducibilityRun,
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

struct Fixture {
    report: CompilerComponentReproducibility,
    report_source: String,
    attestation: CompilerComponentAttestation,
    attestation_source: String,
    attester_registry: CompilerComponentAttesterTrustRegistry,
    attester_registry_source: String,
    attester_registry_sha256: String,
    authorizer_registry: CompilerComponentReplacementAuthorizerRegistry,
    authorizer_registry_source: String,
    authorizer_registry_sha256: String,
}

fn fixture(authorizer_seed: u8, authorizer_status: &str) -> Fixture {
    let report = build_from_runs(vec![run(0, 'd'), run(1, 'e')]).expect("build report");
    let report_source = render_compiler_component_reproducibility(&report);
    let attestation = build_compiler_component_attestation(
        CompilerComponentAttestationInput {
            reproducibility: &report,
            reproducibility_source: &report_source,
            challenge_sha256: &hash('f'),
            attester_id: "linux-builder-1",
            environment_id: "linux-amd64-cleanroom",
        },
        &signing_key_hex(7),
    )
    .expect("build attestation");
    let attestation_source = render_compiler_component_attestation(&attestation);

    let attester_registry = build_compiler_component_attester_trust_registry(
        3,
        &[CompilerComponentAttesterTrustEntryInput {
            attester_id: "linux-builder-1",
            environment_id: "linux-amd64-cleanroom",
            public_key_hex: &public_key_hex(7),
            status: "active",
        }],
    )
    .expect("build attester registry");
    let attester_registry_source =
        render_compiler_component_attester_trust_registry(&attester_registry);
    let attester_registry_sha256 =
        compiler_component_attester_trust_registry_sha256(&attester_registry_source);

    let authorizer_registry = build_compiler_component_replacement_authorizer_registry(
        4,
        &[CompilerComponentReplacementAuthorizerEntryInput {
            authorizer_id: "compiler-owner-1",
            environment_id: "release-control",
            component_id: "projection_relay",
            public_key_hex: &public_key_hex(authorizer_seed),
            status: authorizer_status,
        }],
    )
    .expect("build authorizer registry");
    let authorizer_registry_source =
        render_compiler_component_replacement_authorizer_registry(&authorizer_registry);
    let authorizer_registry_sha256 =
        compiler_component_replacement_authorizer_registry_sha256(&authorizer_registry_source);

    Fixture {
        report,
        report_source,
        attestation,
        attestation_source,
        attester_registry,
        attester_registry_source,
        attester_registry_sha256,
        authorizer_registry,
        authorizer_registry_source,
        authorizer_registry_sha256,
    }
}

fn build_authorization(
    fixture: &Fixture,
    signing_seed: u8,
) -> Result<CompilerComponentReplacementAuthorization, ArtifactError> {
    build_compiler_component_replacement_authorization(
        CompilerComponentReplacementAuthorizationInput {
            reproducibility: &fixture.report,
            reproducibility_source: &fixture.report_source,
            attestation: &fixture.attestation,
            attestation_source: &fixture.attestation_source,
            challenge_sha256: &hash('e'),
            authorization_id: "projection-relay-genesis",
            authorizer_id: "compiler-owner-1",
            environment_id: "release-control",
        },
        &signing_key_hex(signing_seed),
    )
}

fn verify(
    authorization: &CompilerComponentReplacementAuthorization,
    fixture: &Fixture,
    authorization_challenge: &str,
    authorizer_pin: &str,
) -> Result<(), ArtifactError> {
    verify_compiler_component_replacement_authorization(
        authorization,
        CompilerComponentReplacementVerificationInput {
            reproducibility: &fixture.report,
            reproducibility_source: &fixture.report_source,
            attestation: &fixture.attestation,
            attestation_source: &fixture.attestation_source,
            attester_registry: &fixture.attester_registry,
            attester_registry_source: &fixture.attester_registry_source,
            expected_attester_registry_sha256: &fixture.attester_registry_sha256,
            expected_attestation_challenge_sha256: &hash('f'),
            authorizer_registry: &fixture.authorizer_registry,
            authorizer_registry_source: &fixture.authorizer_registry_source,
            expected_authorizer_registry_sha256: authorizer_pin,
            expected_authorization_challenge_sha256: authorization_challenge,
        },
    )
}

#[test]
fn independent_authorization_round_trips_and_verifies_both_pinned_roles() {
    let fixture = fixture(9, "active");
    let authorization = build_authorization(&fixture, 9).expect("build authorization");
    let source = render_compiler_component_replacement_authorization(&authorization);
    assert!(!source.contains("/Users/"));
    assert!(!source.contains("timestamp"));
    assert!(authorization.replacement_authorized);
    assert!(authorization.reversible);
    assert!(!authorization.attestation_replacement_authorized);
    assert_eq!(
        authorization.rollback_reproducible_build_sha256,
        authorization.from_reproducible_build_sha256
    );
    assert_ne!(
        authorization.authorizer_public_key_id,
        authorization.attester_public_key_id
    );

    let parsed = parse_compiler_component_replacement_authorization_from_source(
        &source,
        Path::new(COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_FILE),
    )
    .expect("parse authorization");
    assert_eq!(parsed, authorization);
    verify(
        &parsed,
        &fixture,
        &hash('e'),
        &fixture.authorizer_registry_sha256,
    )
    .expect("verify authorization");
}

#[test]
fn attester_identity_or_key_cannot_be_promoted_into_replacement_authority() {
    let fixture = fixture(9, "active");
    let error = build_compiler_component_replacement_authorization(
        CompilerComponentReplacementAuthorizationInput {
            reproducibility: &fixture.report,
            reproducibility_source: &fixture.report_source,
            attestation: &fixture.attestation,
            attestation_source: &fixture.attestation_source,
            challenge_sha256: &hash('e'),
            authorization_id: "projection-relay-genesis",
            authorizer_id: "linux-builder-1",
            environment_id: "release-control",
        },
        &signing_key_hex(9),
    )
    .expect_err("attester identity must not authorize replacement");
    assert!(error.to_string().contains("identity must differ"));

    let error = build_authorization(&fixture, 7)
        .expect_err("attester signing key must not authorize replacement");
    assert!(error.to_string().contains("key must differ"));
}

#[test]
fn authorization_signature_challenge_pin_and_revocation_fail_closed() {
    let active_fixture = fixture(9, "active");
    let authorization = build_authorization(&active_fixture, 9).expect("build authorization");

    let mut signature_tamper = authorization.clone();
    signature_tamper.signature_hex.replace_range(0..2, "00");
    let error = verify(
        &signature_tamper,
        &active_fixture,
        &hash('e'),
        &active_fixture.authorizer_registry_sha256,
    )
    .expect_err("signature tampering must fail");
    assert!(error.to_string().contains("signature mismatch"));

    let error = verify(
        &authorization,
        &active_fixture,
        &hash('d'),
        &active_fixture.authorizer_registry_sha256,
    )
    .expect_err("challenge replay must fail");
    assert!(error.to_string().contains("verifier request"));

    let error = verify(&authorization, &active_fixture, &hash('e'), &hash('0'))
        .expect_err("unpinned authorizer registry must fail");
    assert!(error.to_string().contains("pinned SHA-256"));

    let revoked = fixture(9, "revoked");
    let revoked_authorization = build_authorization(&revoked, 9).expect("build authorization");
    let error = verify(
        &revoked_authorization,
        &revoked,
        &hash('e'),
        &revoked.authorizer_registry_sha256,
    )
    .expect_err("revoked authorizer must fail");
    assert!(error.to_string().contains("revoked"));
}

#[test]
fn authorization_lineage_and_transition_tampering_fail_before_activation() {
    let fixture = fixture(9, "active");
    let authorization = build_authorization(&fixture, 9).expect("build authorization");
    let source = render_compiler_component_replacement_authorization(&authorization);

    let tampered_lineage = source.replacen(&hash('6'), &hash('0'), 1);
    let error = parse_compiler_component_replacement_authorization_from_source(
        &tampered_lineage,
        Path::new(COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_FILE),
    )
    .expect_err("lineage tampering must fail");
    assert!(error.to_string().contains("proof identity mismatch"));

    let tampered_transition = source.replacen(
        "rollback_reproducible_build_sha256 = \"3333",
        "rollback_reproducible_build_sha256 = \"4444",
        1,
    );
    let error = parse_compiler_component_replacement_authorization_from_source(
        &tampered_transition,
        Path::new(COMPILER_COMPONENT_REPLACEMENT_AUTHORIZATION_FILE),
    )
    .expect_err("rollback drift must fail");
    assert!(error.to_string().contains("separation or transition"));
}

#[test]
fn authorizer_registry_is_component_scoped_and_canonically_sorted() {
    let first_key = public_key_hex(9);
    let second_key = public_key_hex(11);
    let registry = build_compiler_component_replacement_authorizer_registry(
        5,
        &[
            CompilerComponentReplacementAuthorizerEntryInput {
                authorizer_id: "owner-z",
                environment_id: "release-control",
                component_id: "component-z",
                public_key_hex: &second_key,
                status: "active",
            },
            CompilerComponentReplacementAuthorizerEntryInput {
                authorizer_id: "owner-a",
                environment_id: "release-control",
                component_id: "component-a",
                public_key_hex: &first_key,
                status: "active",
            },
        ],
    )
    .expect("build sorted registry");
    assert_eq!(registry.entries[0].component_id, "component-a");
    let source = render_compiler_component_replacement_authorizer_registry(&registry);
    let parsed = parse_compiler_component_replacement_authorizer_registry_from_source(
        &source,
        Path::new("authorizers.toml"),
    )
    .expect("parse registry");
    assert_eq!(parsed, registry);
}
