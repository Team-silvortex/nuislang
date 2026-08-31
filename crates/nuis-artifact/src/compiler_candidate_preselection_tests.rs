use std::path::Path;

use super::*;

fn hash(byte: char) -> String {
    byte.to_string().repeat(64)
}

fn fixture() -> CompilerCandidatePreselection {
    let mut value = CompilerCandidatePreselection {
        protocol: COMPILER_CANDIDATE_PRESELECTION_PROTOCOL.to_owned(),
        authority: COMPILER_CANDIDATE_PRESELECTION_AUTHORITY.to_owned(),
        signature_contract: COMPILER_CANDIDATE_PRESELECTION_SIGNATURE_CONTRACT.to_owned(),
        action: COMPILER_CANDIDATE_PRESELECTION_ACTION.to_owned(),
        component_id: "compiler-main".to_owned(),
        component_domain: "cpu".to_owned(),
        component_unit: "Main".to_owned(),
        preselection_id: "candidate-preselection-3".to_owned(),
        target_generation: 3,
        predecessor_transition_protocol: COMPILER_COMPONENT_TRANSITION_PROTOCOL.to_owned(),
        predecessor_transition_file: COMPILER_COMPONENT_TRANSITION_FILE.to_owned(),
        predecessor_transition_file_bytes: 1200,
        predecessor_transition_file_sha256: hash('0'),
        predecessor_transition_id: "rollback-2".to_owned(),
        predecessor_transition_generation: 2,
        predecessor_transition_proof_sha256: hash('1'),
        challenge_sha256: hash('2'),
        current_stage_role: COMPILER_COMPONENT_STAGE0_ROLE.to_owned(),
        current_record_sha256: hash('3'),
        current_reproducible_build_sha256: hash('4'),
        current_compiler_image_sha256: hash('5'),
        candidate_stage_role: COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE.to_owned(),
        candidate_record_sha256: hash('6'),
        candidate_reproducible_build_sha256: hash('7'),
        candidate_producer_id: "nuis-stage1-candidate".to_owned(),
        candidate_compiler_image_sha256: hash('8'),
        production_protocol: COMPILER_CANDIDATE_PRODUCTION_PROTOCOL.to_owned(),
        production_file: COMPILER_CANDIDATE_PRODUCTION_FILE.to_owned(),
        production_file_bytes: 2400,
        production_file_sha256: hash('9'),
        production_proof_sha256: hash('a'),
        compile_capability_protocol: COMPILER_CANDIDATE_COMPILE_CAPABILITY_PROTOCOL.to_owned(),
        compile_capability_file: COMPILER_CANDIDATE_COMPILE_CAPABILITY_FILE.to_owned(),
        compile_capability_file_bytes: 1800,
        compile_capability_file_sha256: hash('b'),
        compile_capability_proof_sha256: hash('c'),
        compile_driver_contract: "candidate-delegating-driver-v1".to_owned(),
        compile_provider_contract: "verified-stage0-provider-v1".to_owned(),
        compiled_artifact_semantic_sha256: hash('d'),
        compile_result_record_sha256: hash('e'),
        compile_result_reproducible_build_sha256: hash('4'),
        compile_result_native_binary_sha256: hash('f'),
        provider_dependency_contract: COMPILER_CANDIDATE_PRESELECTION_PROVIDER_CONTRACT.to_owned(),
        provider_dependency_required: true,
        direct_stage1_compile: false,
        replacement_authorized: false,
        selection_authorized: false,
        preselection_authorized: true,
        authorizer_id: "compiler-owner".to_owned(),
        authorizer_environment_id: "release-control".to_owned(),
        authorizer_public_key_id: format!("ed25519:sha256:{}", hash('a')),
        verdict: COMPILER_CANDIDATE_PRESELECTION_VERDICT.to_owned(),
        proof_sha256: String::new(),
        signature_hex: "00".repeat(64),
    };
    value.proof_sha256 = preselection_identity(&value);
    value
}

#[test]
fn candidate_preselection_roundtrips_without_selection_authority() {
    let value = fixture();
    let source = render_compiler_candidate_preselection(&value);
    let parsed = parse_compiler_candidate_preselection_from_source(
        &source,
        Path::new(COMPILER_CANDIDATE_PRESELECTION_FILE),
    )
    .expect("parse candidate preselection");
    assert_eq!(parsed, value);
    assert!(parsed.preselection_authorized);
    assert!(!parsed.selection_authorized);
    assert!(!parsed.replacement_authorized);
    assert!(!parsed.direct_stage1_compile);
}

#[test]
fn candidate_preselection_rejects_bound_capability_tampering() {
    let mut value = fixture();
    value.compile_capability_proof_sha256 = hash('d');
    let source = render_compiler_candidate_preselection(&value);
    let error = parse_compiler_candidate_preselection_from_source(
        &source,
        Path::new(COMPILER_CANDIDATE_PRESELECTION_FILE),
    )
    .expect_err("tampered capability binding must fail");
    assert!(error.to_string().contains("proof identity mismatch"));
}

#[test]
fn candidate_preselection_rejects_claimed_selection_authority() {
    let mut value = fixture();
    value.selection_authorized = true;
    value.proof_sha256 = preselection_identity(&value);
    let source = render_compiler_candidate_preselection(&value);
    let error = parse_compiler_candidate_preselection_from_source(
        &source,
        Path::new(COMPILER_CANDIDATE_PRESELECTION_FILE),
    )
    .expect_err("selection authority must remain false");
    assert!(error.to_string().contains("contract mismatch"));
}
