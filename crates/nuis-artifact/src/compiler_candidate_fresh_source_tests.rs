use super::*;

fn hash(byte: char) -> String {
    byte.to_string().repeat(64)
}

fn capability() -> CompilerCandidateFreshSourceCapability {
    let mut capability = CompilerCandidateFreshSourceCapability {
        protocol: COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_PROTOCOL.to_owned(),
        driver_contract: COMPILER_CANDIDATE_FRESH_SOURCE_DRIVER_CONTRACT.to_owned(),
        authority: COMPILER_CANDIDATE_FRESH_SOURCE_AUTHORITY.to_owned(),
        snapshot_contract: COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT_CONTRACT.to_owned(),
        abi_contract: COMPILER_CANDIDATE_FRESH_SOURCE_ABI_CONTRACT.to_owned(),
        input_contract: COMPILER_CANDIDATE_FRESH_SOURCE_INPUT_CONTRACT.to_owned(),
        argument_contract: COMPILER_CANDIDATE_FRESH_SOURCE_ARGUMENT_CONTRACT.to_owned(),
        environment_contract: COMPILER_CANDIDATE_FRESH_SOURCE_ENVIRONMENT_CONTRACT.to_owned(),
        stdin_contract: CLOSED_STDIN_CONTRACT.to_owned(),
        native_contract: COMPILER_CANDIDATE_FRESH_SOURCE_NATIVE_CONTRACT.to_owned(),
        bootstrap_subset_protocol: "nuis-bootstrap-language-subset-v8".to_owned(),
        component_id: "projection_relay".to_owned(),
        component_domain: "cpu".to_owned(),
        component_unit: "Main".to_owned(),
        candidate_record_sha256: hash('1'),
        candidate_reproducible_build_sha256: hash('2'),
        candidate_producer_id: "nuis-stage1-compact-structured-nir-producer-v10".to_owned(),
        candidate_compiler_image_sha256: hash('3'),
        production_protocol: COMPILER_CANDIDATE_PRODUCTION_PROTOCOL.to_owned(),
        production_proof_sha256: hash('4'),
        adapter_file: COMPILER_CANDIDATE_ADAPTER_FILE.to_owned(),
        adapter_bytes: 101,
        adapter_sha256: hash('5'),
        predecessor_successor_protocol: COMPILER_CANDIDATE_SUCCESSOR_PROTOCOL.to_owned(),
        predecessor_successor_file: COMPILER_CANDIDATE_SUCCESSOR_FILE.to_owned(),
        predecessor_successor_file_bytes: 202,
        predecessor_successor_file_sha256: hash('6'),
        predecessor_successor_proof_sha256: hash('7'),
        source_bytes: 56,
        source_lines: 5,
        source_sha256: hash('8'),
        stage_count: 5,
        token_record_count: 16,
        ast_record_count: 5,
        nir_record_count: 6,
        yir_record_count: 6,
        token_identity: 11,
        ast_identity: 12,
        nir_identity: 13,
        yir_identity: 14,
        result_protocol: COMPILER_CANDIDATE_FRESH_SOURCE_RESULT_PROTOCOL.to_owned(),
        result_file: COMPILER_CANDIDATE_FRESH_SOURCE_RESULT_FILE.to_owned(),
        result_bytes: 303,
        result_sha256: hash('9'),
        result_bundle_fold: 15,
        exit_code: 0,
        stderr_bytes: 0,
        stderr_sha256: sha256_hex(&[]),
        stage0_handoff_required: false,
        provider_dependency_required: false,
        candidate_owned_source_processing: true,
        direct_stage1_compile: true,
        fresh_source_compile: true,
        native_materialization: false,
        replacement_authorized: false,
        selection_authorized: false,
        verdict: COMPILER_CANDIDATE_FRESH_SOURCE_VERDICT.to_owned(),
        proof_sha256: String::new(),
    };
    capability.proof_sha256 = capability_identity(&capability);
    capability
}

#[test]
fn fresh_source_capability_roundtrips_without_handoff_or_native_authority() {
    let capability = capability();
    validate_compiler_candidate_fresh_source_capability(&capability)
        .expect("valid fresh-source capability");
    let source = render_compiler_candidate_fresh_source_capability(&capability);
    let parsed = parse_compiler_candidate_fresh_source_capability_from_source(
        &source,
        Path::new(COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_FILE),
    )
    .expect("parse canonical fresh-source capability");
    assert_eq!(parsed, capability);
    assert!(parsed.fresh_source_compile);
    assert!(parsed.candidate_owned_source_processing);
    assert!(!parsed.stage0_handoff_required);
    assert!(!parsed.native_materialization);
}

#[test]
fn fresh_source_capability_tampering_fails_closed() {
    let capability = capability();
    let source = render_compiler_candidate_fresh_source_capability(&capability);
    for tampered in [
        source.replacen(
            "fresh_source_compile = true",
            "fresh_source_compile = false",
            1,
        ),
        source.replacen(
            "stage0_handoff_required = false",
            "stage0_handoff_required = true",
            1,
        ),
        source.replacen(&capability.source_sha256, &hash('a'), 1),
        source.trim_end().to_owned(),
    ] {
        assert!(
            parse_compiler_candidate_fresh_source_capability_from_source(
                &tampered,
                Path::new(COMPILER_CANDIDATE_FRESH_SOURCE_CAPABILITY_FILE),
            )
            .is_err()
        );
    }
}
