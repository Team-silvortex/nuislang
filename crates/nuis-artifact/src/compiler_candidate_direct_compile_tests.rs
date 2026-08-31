use super::*;

fn hash(byte: char) -> String {
    byte.to_string().repeat(64)
}

fn capability() -> CompilerCandidateDirectCompileCapability {
    let mut capability = CompilerCandidateDirectCompileCapability {
        protocol: COMPILER_CANDIDATE_DIRECT_COMPILE_CAPABILITY_PROTOCOL.to_owned(),
        driver_contract: COMPILER_CANDIDATE_DIRECT_COMPILE_DRIVER_CONTRACT.to_owned(),
        authority: COMPILER_CANDIDATE_DIRECT_COMPILE_AUTHORITY.to_owned(),
        request_contract: COMPILER_CANDIDATE_DIRECT_COMPILE_REQUEST_CONTRACT.to_owned(),
        provider_contract: COMPILER_CANDIDATE_DIRECT_COMPILE_PROVIDER_CONTRACT.to_owned(),
        environment_contract: COMPILER_CANDIDATE_DIRECT_COMPILE_ENVIRONMENT_CONTRACT.to_owned(),
        input_contract: COMPILER_CANDIDATE_DIRECT_COMPILE_INPUT_CONTRACT.to_owned(),
        argument_contract: COMPILER_CANDIDATE_DIRECT_COMPILE_ARGUMENT_CONTRACT.to_owned(),
        stdin_contract: CLOSED_STDIN_CONTRACT.to_owned(),
        native_contract: COMPILER_CANDIDATE_DIRECT_COMPILE_NATIVE_CONTRACT.to_owned(),
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
        handoff_protocol: COMPILER_STAGE_HANDOFF_PROTOCOL.to_owned(),
        handoff_bundle_sha256: hash('6'),
        input_record_count: EXPECTED_STAGE_COUNT,
        input_identity_sha256: hash('7'),
        result_protocol: COMPILER_CANDIDATE_FRONTEND_RESULT_PROTOCOL.to_owned(),
        result_file: COMPILER_CANDIDATE_FRONTEND_RESULT_FILE.to_owned(),
        result_bytes: 202,
        result_sha256: hash('8'),
        result_bundle_fold: 303,
        exit_code: 0,
        stderr_bytes: 0,
        stderr_sha256: sha256_hex(&[]),
        provider_dependency_required: false,
        direct_stage1_compile: true,
        native_materialization: false,
        replacement_authorized: false,
        selection_authorized: false,
        verdict: COMPILER_CANDIDATE_DIRECT_COMPILE_VERDICT.to_owned(),
        proof_sha256: String::new(),
    };
    capability.proof_sha256 = capability_identity(&capability);
    capability
}

#[test]
fn direct_compile_capability_roundtrips_without_provider_or_selection() {
    let capability = capability();
    validate_compiler_candidate_direct_compile_capability(&capability)
        .expect("valid direct compile capability");
    let source = render_compiler_candidate_direct_compile_capability(&capability);
    let parsed = parse_compiler_candidate_direct_compile_capability_from_source(
        &source,
        Path::new(COMPILER_CANDIDATE_DIRECT_COMPILE_CAPABILITY_FILE),
    )
    .expect("parse canonical direct compile capability");
    assert_eq!(parsed, capability);
    assert!(!parsed.provider_dependency_required);
    assert!(parsed.direct_stage1_compile);
    assert!(!parsed.native_materialization);
}

#[test]
fn direct_compile_capability_tampering_fails_closed() {
    let capability = capability();
    let source = render_compiler_candidate_direct_compile_capability(&capability);
    for tampered in [
        source.replacen(
            "provider_dependency_required = false",
            "provider_dependency_required = true",
            1,
        ),
        source.replacen(
            "direct_stage1_compile = true",
            "direct_stage1_compile = false",
            1,
        ),
        source.replacen(&capability.result_sha256, &hash('9'), 1),
        source.trim_end().to_owned(),
    ] {
        assert!(
            parse_compiler_candidate_direct_compile_capability_from_source(
                &tampered,
                Path::new(COMPILER_CANDIDATE_DIRECT_COMPILE_CAPABILITY_FILE),
            )
            .is_err()
        );
    }
}
