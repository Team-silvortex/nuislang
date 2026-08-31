use std::path::Path;

use ed25519_dalek::SigningKey;

use super::*;
use crate::{
    build_compiler_component_active_state, build_compiler_component_attestation,
    build_compiler_component_replacement_authorization,
    build_compiler_component_replacement_authorizer_registry,
    compiler_component_replacement_authorizer_registry_sha256,
    compiler_component_reproducibility::build_from_runs, render_compiler_component_active_state,
    render_compiler_component_attestation, render_compiler_component_replacement_authorization,
    render_compiler_component_replacement_authorizer_registry,
    render_compiler_component_reproducibility, CompilerComponentAttestationInput,
    CompilerComponentReplacementAuthorizationInput,
    CompilerComponentReplacementAuthorizerEntryInput, CompilerComponentReproducibilityRun,
};

const TRANSITION_CHALLENGE_SHA256: &str =
    "1111111111111111111111111111111111111111111111111111111111111111";

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
    authorization: CompilerComponentReplacementAuthorization,
    authorization_source: String,
    active_state: CompilerComponentActiveState,
    active_state_source: String,
    authorizer_registry: CompilerComponentReplacementAuthorizerRegistry,
    authorizer_registry_source: String,
    authorizer_registry_sha256: String,
}

fn fixture() -> Fixture {
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
            challenge_sha256: &hash('0'),
            authorization_id: "projection-relay-genesis",
            authorizer_id: "compiler-owner-1",
            environment_id: "release-control",
        },
        &signing_key_hex(9),
    )
    .expect("build authorization");
    let authorization_source = render_compiler_component_replacement_authorization(&authorization);
    let active_state = build_compiler_component_active_state(&authorization, &authorization_source)
        .expect("build active state");
    let active_state_source = render_compiler_component_active_state(&active_state);
    let authorizer_registry = build_compiler_component_replacement_authorizer_registry(
        1,
        &[CompilerComponentReplacementAuthorizerEntryInput {
            authorizer_id: "compiler-owner-1",
            environment_id: "release-control",
            component_id: "projection_relay",
            public_key_hex: &public_key_hex(9),
            status: "active",
        }],
    )
    .expect("build authorizer registry");
    let authorizer_registry_source =
        render_compiler_component_replacement_authorizer_registry(&authorizer_registry);
    let authorizer_registry_sha256 =
        compiler_component_replacement_authorizer_registry_sha256(&authorizer_registry_source);
    Fixture {
        authorization,
        authorization_source,
        active_state,
        active_state_source,
        authorizer_registry,
        authorizer_registry_source,
        authorizer_registry_sha256,
    }
}

fn verification_input<'a>(
    fixture: &'a Fixture,
) -> CompilerComponentTransitionVerificationInput<'a> {
    CompilerComponentTransitionVerificationInput {
        authorization: &fixture.authorization,
        authorization_source: &fixture.authorization_source,
        active_state: &fixture.active_state,
        active_state_source: &fixture.active_state_source,
        authorizer_registry: &fixture.authorizer_registry,
        authorizer_registry_source: &fixture.authorizer_registry_source,
        expected_authorizer_registry_sha256: &fixture.authorizer_registry_sha256,
        expected_transition_challenge_sha256: TRANSITION_CHALLENGE_SHA256,
    }
}

#[test]
fn generation_two_transition_restores_stage0_and_retains_forward_candidate() {
    let fixture = fixture();
    let transition = build_compiler_component_transition(
        CompilerComponentTransitionInput {
            authorization: &fixture.authorization,
            authorization_source: &fixture.authorization_source,
            active_state: &fixture.active_state,
            active_state_source: &fixture.active_state_source,
            challenge_sha256: TRANSITION_CHALLENGE_SHA256,
            transition_id: "projection-relay-rollback-2",
            authorizer_id: "compiler-owner-1",
            environment_id: "release-control",
        },
        &signing_key_hex(9),
    )
    .expect("build transition");
    let source = render_compiler_component_transition(&transition);
    let parsed = parse_compiler_component_transition_from_source(
        &source,
        Path::new(COMPILER_COMPONENT_TRANSITION_FILE),
    )
    .expect("parse transition");
    verify_compiler_component_transition(&parsed, verification_input(&fixture))
        .expect("verify transition");

    assert_eq!(parsed.generation, 2);
    assert_eq!(
        parsed.predecessor_authorization_proof_sha256,
        fixture.authorization.proof_sha256
    );
    assert_eq!(
        parsed.predecessor_state_sha256,
        fixture.active_state.state_sha256
    );
    let current = select_compiler_component_transition_target(
        &parsed,
        verification_input(&fixture),
        CompilerComponentTransitionSelection::Current,
    )
    .expect("select restored stage0");
    assert_eq!(current.stage_role, "stage0");
    assert_eq!(
        current.reproducible_build_sha256,
        fixture.authorization.from_reproducible_build_sha256
    );
    let forward = select_compiler_component_transition_target(
        &parsed,
        verification_input(&fixture),
        CompilerComponentTransitionSelection::Forward,
    )
    .expect("select retained candidate");
    assert_eq!(forward.stage_role, "stage1-candidate");
    assert_eq!(
        forward.reproducible_build_sha256,
        fixture.authorization.to_reproducible_build_sha256
    );
}

#[test]
fn transition_tampering_replay_or_owner_key_drift_fails_closed() {
    let fixture = fixture();
    let transition = build_compiler_component_transition(
        CompilerComponentTransitionInput {
            authorization: &fixture.authorization,
            authorization_source: &fixture.authorization_source,
            active_state: &fixture.active_state,
            active_state_source: &fixture.active_state_source,
            challenge_sha256: TRANSITION_CHALLENGE_SHA256,
            transition_id: "projection-relay-rollback-2",
            authorizer_id: "compiler-owner-1",
            environment_id: "release-control",
        },
        &signing_key_hex(9),
    )
    .expect("build transition");
    let source = render_compiler_component_transition(&transition);
    let tampered = source.replacen(
        &format!(
            "current_reproducible_build_sha256 = \"{}\"",
            fixture.authorization.from_reproducible_build_sha256
        ),
        &format!("current_reproducible_build_sha256 = \"{}\"", hash('a')),
        1,
    );
    let error = parse_compiler_component_transition_from_source(
        &tampered,
        Path::new(COMPILER_COMPONENT_TRANSITION_FILE),
    )
    .expect_err("tampered current build must fail");
    assert!(error.to_string().contains("proof identity mismatch"));

    let wrong_challenge_value = hash('2');
    let mut wrong_challenge = verification_input(&fixture);
    wrong_challenge.expected_transition_challenge_sha256 = &wrong_challenge_value;
    let error = verify_compiler_component_transition(&transition, wrong_challenge)
        .expect_err("challenge replay must fail");
    assert!(error.to_string().contains("verifier request"));

    let error = build_compiler_component_transition(
        CompilerComponentTransitionInput {
            authorization: &fixture.authorization,
            authorization_source: &fixture.authorization_source,
            active_state: &fixture.active_state,
            active_state_source: &fixture.active_state_source,
            challenge_sha256: TRANSITION_CHALLENGE_SHA256,
            transition_id: "projection-relay-rollback-2",
            authorizer_id: "compiler-owner-1",
            environment_id: "release-control",
        },
        &signing_key_hex(8),
    )
    .expect_err("owner key drift must fail");
    assert!(error.to_string().contains("genesis component-owner key"));

    assert!(CompilerComponentTransitionSelection::parse("current").is_ok());
    assert!(CompilerComponentTransitionSelection::parse("forward").is_ok());
    assert!(CompilerComponentTransitionSelection::parse("rollback").is_err());
}
