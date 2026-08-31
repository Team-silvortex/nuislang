use std::path::Path;

use ed25519_dalek::{Signer, SigningKey};

use super::*;
use crate::{
    build_compiler_component_replacement_authorizer_registry,
    CompilerComponentReplacementAuthorizerEntryInput,
};

fn hash(byte: char) -> String {
    byte.to_string().repeat(64)
}

fn fixture() -> CompilerCandidateSuccessor {
    let key = SigningKey::from_bytes(&[9u8; 32]);
    let mut value = CompilerCandidateSuccessor {
        protocol: COMPILER_CANDIDATE_SUCCESSOR_PROTOCOL.to_owned(),
        authority: COMPILER_CANDIDATE_SUCCESSOR_AUTHORITY.to_owned(),
        signature_contract: COMPILER_CANDIDATE_SUCCESSOR_SIGNATURE_CONTRACT.to_owned(),
        action: COMPILER_CANDIDATE_SUCCESSOR_ACTION.to_owned(),
        relation_contract: COMPILER_CANDIDATE_SUCCESSOR_RELATION_CONTRACT.to_owned(),
        component_id: "compiler-main".to_owned(),
        component_domain: "cpu".to_owned(),
        component_unit: "Main".to_owned(),
        successor_id: "candidate-successor-3".to_owned(),
        target_generation: 3,
        predecessor_preselection_protocol: COMPILER_CANDIDATE_PRESELECTION_PROTOCOL.to_owned(),
        predecessor_preselection_file: COMPILER_CANDIDATE_PRESELECTION_FILE.to_owned(),
        predecessor_preselection_file_bytes: 2200,
        predecessor_preselection_file_sha256: hash('0'),
        predecessor_preselection_id: "candidate-preselection-3".to_owned(),
        predecessor_preselection_proof_sha256: hash('1'),
        challenge_sha256: hash('2'),
        candidate_stage_role: COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE.to_owned(),
        candidate_record_sha256: hash('3'),
        candidate_reproducible_build_sha256: hash('4'),
        candidate_producer_id: "nuis-stage1-candidate".to_owned(),
        candidate_compiler_image_sha256: hash('5'),
        production_protocol: COMPILER_CANDIDATE_PRODUCTION_PROTOCOL.to_owned(),
        production_proof_sha256: hash('6'),
        direct_compile_capability_protocol: COMPILER_CANDIDATE_DIRECT_COMPILE_CAPABILITY_PROTOCOL
            .to_owned(),
        direct_compile_capability_file: COMPILER_CANDIDATE_DIRECT_COMPILE_CAPABILITY_FILE
            .to_owned(),
        direct_compile_capability_file_bytes: 1800,
        direct_compile_capability_file_sha256: hash('7'),
        direct_compile_capability_proof_sha256: hash('8'),
        direct_compile_driver_contract: "nuis-stage1-candidate-direct-front-end-driver-v1"
            .to_owned(),
        direct_compile_provider_contract: "no-runtime-compiler-provider-v1".to_owned(),
        direct_compile_input_identity_sha256: hash('9'),
        frontend_result_protocol: COMPILER_CANDIDATE_FRONTEND_RESULT_PROTOCOL.to_owned(),
        frontend_result_file: COMPILER_CANDIDATE_FRONTEND_RESULT_FILE.to_owned(),
        frontend_result_bytes: 1000,
        frontend_result_sha256: hash('a'),
        frontend_result_bundle_fold: 42,
        provider_dependency_required: false,
        direct_stage1_compile: true,
        fresh_source_compile: false,
        native_materialization: false,
        replacement_authorized: false,
        selection_authorized: false,
        preselection_authorized: true,
        successor_authorized: true,
        authorizer_id: "compiler-owner".to_owned(),
        authorizer_environment_id: "release-control".to_owned(),
        authorizer_public_key_id: public_key_id(&key.verifying_key()),
        verdict: COMPILER_CANDIDATE_SUCCESSOR_VERDICT.to_owned(),
        proof_sha256: String::new(),
        signature_hex: String::new(),
    };
    resign(&mut value, &key);
    value
}

fn resign(value: &mut CompilerCandidateSuccessor, key: &SigningKey) {
    value.proof_sha256 = successor_identity(value);
    value.signature_hex = encode_hex(&key.sign(&signature_message(&value.proof_sha256)).to_bytes());
}

#[test]
fn candidate_successor_roundtrips_without_selection_authority() {
    let value = fixture();
    let source = render_compiler_candidate_successor(&value);
    let parsed = parse_compiler_candidate_successor_from_source(
        &source,
        Path::new(COMPILER_CANDIDATE_SUCCESSOR_FILE),
    )
    .expect("parse candidate successor");
    assert_eq!(parsed, value);
    assert!(parsed.successor_authorized);
    assert!(parsed.direct_stage1_compile);
    assert!(!parsed.provider_dependency_required);
    assert!(!parsed.fresh_source_compile);
    assert!(!parsed.native_materialization);
    assert!(!parsed.selection_authorized);
    assert!(!parsed.replacement_authorized);
}

#[test]
fn candidate_successor_signature_resolves_through_component_owner_registry() {
    let value = fixture();
    let public_key_hex = SigningKey::from_bytes(&[9u8; 32])
        .verifying_key()
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let registry = build_compiler_component_replacement_authorizer_registry(
        1,
        &[CompilerComponentReplacementAuthorizerEntryInput {
            authorizer_id: "compiler-owner",
            environment_id: "release-control",
            component_id: "compiler-main",
            public_key_hex: &public_key_hex,
            status: "active",
        }],
    )
    .expect("build component-owner registry");
    verify_successor_signature(&value, &registry).expect("verify successor signature");

    let mut tampered = value;
    tampered.signature_hex.replace_range(0..2, "ff");
    let error = verify_successor_signature(&tampered, &registry)
        .expect_err("tampered successor signature must fail");
    assert!(error.to_string().contains("signature mismatch"));
}

#[test]
fn candidate_successor_rejects_native_or_final_selection_claims() {
    let key = SigningKey::from_bytes(&[9u8; 32]);
    for update in [
        |value: &mut CompilerCandidateSuccessor| value.native_materialization = true,
        |value: &mut CompilerCandidateSuccessor| value.selection_authorized = true,
        |value: &mut CompilerCandidateSuccessor| {
            value.direct_compile_provider_contract = "forged-provider".to_owned()
        },
    ] {
        let mut value = fixture();
        update(&mut value);
        resign(&mut value, &key);
        let source = render_compiler_candidate_successor(&value);
        let error = parse_compiler_candidate_successor_from_source(
            &source,
            Path::new(COMPILER_CANDIDATE_SUCCESSOR_FILE),
        )
        .expect_err("unsupported successor authority must fail");
        assert!(error.to_string().contains("contract mismatch"));
    }
}
