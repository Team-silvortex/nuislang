use super::*;

fn hash(byte: char) -> String {
    byte.to_string().repeat(64)
}

fn capability() -> CompilerCandidateCompileCapability {
    let mut capability = CompilerCandidateCompileCapability {
        protocol: COMPILER_CANDIDATE_COMPILE_CAPABILITY_PROTOCOL.to_owned(),
        driver_contract: COMPILER_CANDIDATE_COMPILE_DRIVER_CONTRACT.to_owned(),
        authority: COMPILER_CANDIDATE_COMPILE_CAPABILITY_AUTHORITY.to_owned(),
        request_contract: COMPILER_CANDIDATE_COMPILE_REQUEST_CONTRACT.to_owned(),
        provider_contract: COMPILER_CANDIDATE_COMPILE_PROVIDER_CONTRACT.to_owned(),
        admission_contract: COMPILER_CANDIDATE_COMPILE_ADMISSION_CONTRACT.to_owned(),
        command: COMPILER_CANDIDATE_COMPILE_COMMAND.to_owned(),
        argument_contract: COMPILER_CANDIDATE_COMPILE_ARGUMENT_CONTRACT.to_owned(),
        provider_environment: COMPILER_CANDIDATE_COMPILE_PROVIDER_ENVIRONMENT.to_owned(),
        stdin_contract: CLOSED_STDIN_CONTRACT.to_owned(),
        bootstrap_subset_protocol: "nuis-bootstrap-language-subset-v8".to_owned(),
        component_id: "projection_relay".to_owned(),
        component_domain: "cpu".to_owned(),
        component_unit: "Main".to_owned(),
        stage0_record_sha256: hash('1'),
        stage0_reproducible_build_sha256: hash('2'),
        provider_image_bytes: 101,
        provider_image_sha256: hash('3'),
        candidate_record_sha256: hash('4'),
        candidate_reproducible_build_sha256: hash('5'),
        candidate_producer_id: "nuis-stage1-compact-structured-nir-producer-v10".to_owned(),
        candidate_compiler_image_sha256: hash('6'),
        production_protocol: COMPILER_CANDIDATE_PRODUCTION_PROTOCOL.to_owned(),
        production_proof_sha256: hash('7'),
        adapter_file: COMPILER_CANDIDATE_ADAPTER_FILE.to_owned(),
        adapter_bytes: 202,
        adapter_sha256: hash('8'),
        request_compiled_artifact_bytes: 303,
        request_compiled_artifact_sha256: hash('9'),
        compiled_artifact_identity_contract: COMPILER_COMPONENT_COMPILED_ARTIFACT_IDENTITY_CONTRACT
            .to_owned(),
        compiled_artifact_semantic_sha256: hash('a'),
        result_record_sha256: hash('b'),
        result_reproducible_build_sha256: hash('2'),
        result_compiled_artifact_bytes: 304,
        result_compiled_artifact_sha256: hash('c'),
        result_native_binary_bytes: 404,
        result_native_binary_sha256: hash('d'),
        exit_code: 0,
        stdout_bytes: 55,
        stdout_sha256: hash('e'),
        stderr_bytes: 0,
        stderr_sha256: sha256_hex(&[]),
        replacement_authorized: false,
        selection_authorized: false,
        verdict: COMPILER_CANDIDATE_COMPILE_CAPABILITY_VERDICT.to_owned(),
        proof_sha256: String::new(),
    };
    capability.proof_sha256 = capability_identity(&capability);
    capability
}

#[test]
fn candidate_compile_capability_roundtrips_without_selection_authority() {
    let capability = capability();
    validate_compiler_candidate_compile_capability(&capability).expect("valid capability");
    let source = render_compiler_candidate_compile_capability(&capability);
    let parsed = parse_compiler_candidate_compile_capability_from_source(
        &source,
        Path::new(COMPILER_CANDIDATE_COMPILE_CAPABILITY_FILE),
    )
    .expect("parse canonical capability");
    assert_eq!(parsed, capability);
    assert!(!parsed.replacement_authorized);
    assert!(!parsed.selection_authorized);
}

#[test]
fn candidate_compile_capability_tampering_fails_closed() {
    let capability = capability();
    let source = render_compiler_candidate_compile_capability(&capability);
    for tampered in [
        source.replacen(
            "selection_authorized = false",
            "selection_authorized = true",
            1,
        ),
        source.replacen(&capability.adapter_sha256, &hash('f'), 1),
        source.trim_end().to_owned(),
    ] {
        assert!(parse_compiler_candidate_compile_capability_from_source(
            &tampered,
            Path::new(COMPILER_CANDIDATE_COMPILE_CAPABILITY_FILE),
        )
        .is_err());
    }
}

#[test]
fn candidate_compile_admission_marker_is_exact() {
    assert!(contains_bytes(
        b"provider output\ncandidate_compile_admission=nuis-owned-stage-fold-v1\n",
        ADMISSION_MARKER,
    ));
    assert!(!contains_bytes(
        b"candidate_compile_admission=host-only\n",
        ADMISSION_MARKER,
    ));
}
