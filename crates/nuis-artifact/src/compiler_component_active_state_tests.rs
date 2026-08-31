use std::path::Path;

use super::*;
use crate::{
    build_compiler_component_attestation, build_compiler_component_replacement_authorization,
    compiler_component_reproducibility::build_from_runs, render_compiler_component_attestation,
    render_compiler_component_reproducibility, CompilerComponentAttestationInput,
    CompilerComponentReplacementAuthorizationInput, CompilerComponentReproducibilityRun,
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

fn fixture_authorization(challenge: char) -> (CompilerComponentReplacementAuthorization, String) {
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
    let authorization = build_compiler_component_replacement_authorization(
        CompilerComponentReplacementAuthorizationInput {
            reproducibility: &report,
            reproducibility_source: &report_source,
            attestation: &attestation,
            attestation_source: &attestation_source,
            challenge_sha256: &hash(challenge),
            authorization_id: "projection-relay-genesis",
            authorizer_id: "compiler-owner-1",
            environment_id: "release-control",
        },
        &signing_key_hex(9),
    )
    .expect("build authorization");
    let source = render_compiler_component_replacement_authorization(&authorization);
    (authorization, source)
}

#[test]
fn verified_authorization_derives_one_reversible_canonical_state() {
    let (authorization, authorization_source) = fixture_authorization('0');
    let state = build_compiler_component_active_state(&authorization, &authorization_source)
        .expect("build active state");
    let source = render_compiler_component_active_state(&state);
    let parsed = parse_compiler_component_active_state_from_source(
        &source,
        Path::new(COMPILER_COMPONENT_ACTIVE_STATE_FILE),
    )
    .expect("parse active state");
    verify_compiler_component_active_state(&parsed, &authorization, &authorization_source)
        .expect("verify active state");

    assert_eq!(parsed.generation, 1);
    assert_eq!(parsed.authorization_generation, 1);
    assert_eq!(
        parsed.authorization_proof_sha256,
        authorization.proof_sha256
    );
    assert_eq!(
        parsed.authorization_attestation_proof_sha256,
        authorization.attestation_proof_sha256
    );
    assert_eq!(
        parsed.active_reproducible_build_sha256,
        authorization.to_reproducible_build_sha256
    );
    assert_eq!(
        parsed.rollback_reproducible_build_sha256,
        authorization.from_reproducible_build_sha256
    );

    let active = select_compiler_component_active_target(
        &parsed,
        &authorization,
        &authorization_source,
        CompilerComponentActiveSelection::Active,
    )
    .expect("select active candidate");
    assert_eq!(active.selector, "active");
    assert_eq!(active.stage_role, "stage1-candidate");
    assert_eq!(
        active.reproducible_build_sha256,
        authorization.to_reproducible_build_sha256
    );

    let rollback = select_compiler_component_active_target(
        &parsed,
        &authorization,
        &authorization_source,
        CompilerComponentActiveSelection::Rollback,
    )
    .expect("select stage0 rollback");
    assert_eq!(rollback.selector, "rollback");
    assert_eq!(rollback.stage_role, "stage0");
    assert_eq!(
        rollback.reproducible_build_sha256,
        authorization.from_reproducible_build_sha256
    );
}

#[test]
fn state_or_authorization_lineage_tampering_fails_closed() {
    let (authorization, authorization_source) = fixture_authorization('0');
    let state = build_compiler_component_active_state(&authorization, &authorization_source)
        .expect("build active state");
    let source = render_compiler_component_active_state(&state);
    let tampered = source.replacen(
        &format!(
            "rollback_reproducible_build_sha256 = \"{}\"",
            authorization.from_reproducible_build_sha256
        ),
        &format!("rollback_reproducible_build_sha256 = \"{}\"", hash('a')),
        1,
    );
    let error = parse_compiler_component_active_state_from_source(
        &tampered,
        Path::new(COMPILER_COMPONENT_ACTIVE_STATE_FILE),
    )
    .expect_err("tampered rollback must fail");
    assert!(error.to_string().contains("identity mismatch"));

    let (other_authorization, other_source) = fixture_authorization('1');
    let error = verify_compiler_component_active_state(&state, &other_authorization, &other_source)
        .expect_err("different authorization must fail");
    assert!(error.to_string().contains("does not match"));

    assert!(CompilerComponentActiveSelection::parse("active").is_ok());
    assert!(CompilerComponentActiveSelection::parse("rollback").is_ok());
    assert!(CompilerComponentActiveSelection::parse("candidate").is_err());
}
