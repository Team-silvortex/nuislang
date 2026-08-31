use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use ed25519_dalek::SigningKey;
use nuis_artifact::{
    build_compiler_component_attester_trust_registry,
    build_compiler_component_replacement_authorizer_registry,
    compiler_component_attester_trust_registry_sha256,
    compiler_component_replacement_authorizer_registry_sha256, parse_build_manifest,
    parse_compiler_candidate_compile_capability, parse_compiler_candidate_preselection,
    parse_compiler_component_compile_dispatch_receipt, parse_compiler_component_differential,
    parse_compiler_component_dispatch_receipt, parse_compiler_structural_projection,
    read_compiler_candidate_execution, read_compiler_candidate_production,
    read_compiler_component_attestation, read_compiler_component_build,
    read_compiler_component_representation_differential, read_compiler_component_reproducibility,
    read_compiler_stage_handoff, read_compiler_stage_handoff_v2,
    read_compiler_stage_semantic_differential, render_compiler_component_attester_trust_registry,
    render_compiler_component_replacement_authorizer_registry,
    CompilerComponentAttesterTrustEntryInput, CompilerComponentReplacementAuthorizerEntryInput,
    CompilerProjectionKind, CompilerProjectionRecordKind, CompilerStageKind,
    CompilerStageSemanticDifferentialInput, COMPILER_CANDIDATE_ADAPTER_FILE,
    COMPILER_CANDIDATE_COMPILE_CAPABILITY_VERDICT, COMPILER_CANDIDATE_COMPILE_PROVIDER_ENVIRONMENT,
    COMPILER_CANDIDATE_EXECUTION_AUTHORITY, COMPILER_CANDIDATE_EXECUTION_FILE,
    COMPILER_CANDIDATE_EXECUTION_ROLE, COMPILER_CANDIDATE_PRESELECTION_VERDICT,
    COMPILER_CANDIDATE_PRODUCTION_FILE, COMPILER_COMPONENT_ATTESTATION_FILE,
    COMPILER_COMPONENT_BUILD_FILE, COMPILER_COMPONENT_COMPILE_DISPATCH_FILE,
    COMPILER_COMPONENT_COMPILE_DISPATCH_VERDICT, COMPILER_COMPONENT_DIFFERENTIAL_FILE,
    COMPILER_COMPONENT_DISPATCH_FILE, COMPILER_COMPONENT_DISPATCH_VERDICT,
    COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_FILE, COMPILER_COMPONENT_REPRODUCIBILITY_FILE,
    COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE, COMPILER_STAGE_HANDOFF_V2_FILE,
    COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_FILE, COMPILER_STAGE_TRANSFORMATION_FILE,
};

fn temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should follow the Unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nuis_projection_candidate_{nonce}"));
    fs::create_dir_all(&dir).expect("create structural projection candidate output directory");
    dir
}

#[test]
fn pure_nuis_candidate_produces_an_attested_equivalent_stage1_component() {
    let project = "../../examples/projects/tooling/bootstrap_structural_projection_candidate";
    let output_dir = temp_dir();
    let output_dir_text = output_dir.display().to_string();
    let build = run_nuis(&["bootstrap-candidate-build", project, &output_dir_text]);
    assert!(
        build.status.success(),
        "candidate component build failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr),
    );

    let stage0_dir = output_dir.join("stage0");
    let candidate_dir = output_dir.join("stage1-candidate");
    let stage0_record_path = stage0_dir.join(COMPILER_COMPONENT_BUILD_FILE);
    let candidate_record_path = candidate_dir.join(COMPILER_COMPONENT_BUILD_FILE);
    let stage0 =
        read_compiler_component_build(&stage0_record_path).expect("verify stage0 component");
    let execution =
        read_compiler_candidate_execution(&stage0_dir.join(COMPILER_CANDIDATE_EXECUTION_FILE))
            .expect("verify stage0 candidate execution");
    let candidate = read_compiler_component_build(&candidate_record_path)
        .expect("verify stage1 candidate component");
    assert_eq!(
        candidate.stage_role,
        COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE
    );
    assert_eq!(
        candidate.producer_id,
        "nuis-stage1-compact-structured-nir-producer-v10"
    );
    assert_ne!(candidate.producer_id, stage0.producer_id);
    assert_eq!(candidate.compiler_image_sha256, stage0.native_binary_sha256);

    let (handoff, payloads) =
        read_compiler_stage_handoff(&candidate_dir.join("nuis.compiler-stage-handoff.toml"))
            .expect("verify candidate handoff");
    let production = read_compiler_candidate_production(
        &candidate_dir.join(COMPILER_CANDIDATE_PRODUCTION_FILE),
        &stage0,
        &execution,
        &candidate,
        &handoff,
        &payloads,
    )
    .expect("verify candidate production proof");
    assert_eq!(production.record_count, 5);
    assert!(production.token_record_count > 0);
    assert!(production.token_semantic_fold > 0);
    assert_eq!(production.token_page_record_count, 4);
    assert_eq!(production.token_page_payload_bytes, 21);
    assert_eq!(production.token_page_canonical_bytes, 91);
    assert_eq!(production.token_page_canonical_hash, 1_277_127_995);
    assert_eq!(production.token_page_identity, 164_749_511_446);
    assert!(production.token_page_count > 1);
    assert!(production.token_terminal_page_hash > 0);
    assert!(production.token_page_chain_identity > 0);
    assert_eq!(production.ast_page_record_count, 3);
    assert_eq!(production.ast_page_bytes, 128);
    assert_eq!(production.ast_page_projection_hash, 65_460_735);
    assert_eq!(production.ast_page_continuation_indentation, 0);
    assert_eq!(production.ast_page_continuation_body_bytes, 2);
    assert_eq!(production.ast_page_continuation_body_hash, 28_497_819);
    assert_eq!(production.ast_page_state_hash, 1_349_056_749);
    assert_eq!(production.ast_page_identity, 174_028_320_749);
    assert_eq!(production.ast_page_cursor_identity, 1_136_712_771);
    assert_eq!(production.ast_continuation_page_identity, 149_528_711_957);
    assert_eq!(production.ast_continuation_cursor_identity, 1_472_919_348);
    assert_eq!(production.nir_page_record_count, 4);
    assert_eq!(production.nir_page_bytes, 128);
    assert_eq!(production.nir_page_projection_hash, 568_515_310);
    assert_eq!(production.nir_page_continuation_indentation, 0);
    assert_eq!(production.nir_page_continuation_body_bytes, 25);
    assert_eq!(production.nir_page_continuation_body_hash, 671_013_644);
    assert_eq!(production.nir_page_state_hash, 1_026_894_471);
    assert_eq!(production.nir_page_identity, 132_469_386_887);
    assert_eq!(production.nir_page_cursor_identity, 754_343_074);
    assert_eq!(production.nir_continuation_page_identity, 146_705_724_977);
    assert_eq!(production.nir_continuation_cursor_identity, 38_998_897);
    assert_eq!(
        production.stage_transformations_file,
        COMPILER_STAGE_TRANSFORMATION_FILE
    );
    let transformations = nuis_artifact::read_compiler_stage_transformations(
        &candidate_dir.join(COMPILER_STAGE_TRANSFORMATION_FILE),
        &handoff,
        &payloads,
    )
    .expect("verify Nuis stage transformation");
    assert_eq!(transformations.record_count, 2);
    assert_eq!(
        transformations.records[0].source_stage,
        CompilerStageKind::Ast
    );
    assert_eq!(
        transformations.records[1].source_stage,
        CompilerStageKind::Nir
    );
    let ast_transformation = transformations
        .records
        .iter()
        .find(|record| record.source_stage == CompilerStageKind::Ast)
        .expect("AST stage transformation");
    let nir_transformation = transformations
        .records
        .iter()
        .find(|record| record.source_stage == CompilerStageKind::Nir)
        .expect("NIR stage transformation");
    for (record, kind_tag, first_page, first_cursor, second_page, second_cursor) in [
        (
            ast_transformation,
            1,
            174_028_320_749,
            1_136_712_771,
            149_528_711_957,
            1_472_919_348,
        ),
        (
            nir_transformation,
            2,
            132_469_386_887,
            754_343_074,
            146_705_724_977,
            38_998_897,
        ),
    ] {
        assert_eq!(record.output_word_count, 22);
        assert_eq!(record.output_words[0], kind_tag);
        assert_eq!(record.output_words[1], 2);
        assert_eq!(record.output_words[2], first_page);
        assert_eq!(record.output_words[3], first_cursor);
        assert_eq!(record.output_words[12], second_page);
        assert_eq!(record.output_words[13], second_cursor);
        let source_payload = payloads
            .iter()
            .find(|payload| payload.stage == record.source_stage)
            .expect("transformed source payload");
        let derived_payload = fs::read(candidate_dir.join(&record.output_payload_file))
            .expect("read compact derived payload");
        let legacy_v2_bytes = 8
            + 3 * std::mem::size_of::<u64>()
            + 22 * std::mem::size_of::<u64>()
            + source_payload.bytes.len();
        assert_eq!(record.output_payload_bytes, derived_payload.len());
        assert!(derived_payload.len() < source_payload.bytes.len());
        assert!(derived_payload.len() < legacy_v2_bytes);
        assert!(!derived_payload
            .windows(source_payload.bytes.len())
            .any(|window| window == source_payload.bytes));
    }
    let semantic_differential = read_compiler_stage_semantic_differential(
        &candidate_dir.join(COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_FILE),
        &CompilerStageSemanticDifferentialInput {
            producer_id: &candidate.producer_id,
            handoff: &handoff,
            payloads: &payloads,
            transformations: &transformations,
        },
    )
    .expect("verify stage semantic differential");
    assert_eq!(semantic_differential.comparison_count, 2);
    assert_eq!(semantic_differential.equivalent_count, 2);
    assert!(semantic_differential.deterministic_semantic_equivalent);
    for stage in [CompilerStageKind::Ast, CompilerStageKind::Nir] {
        let comparison = semantic_differential
            .comparisons
            .iter()
            .find(|comparison| comparison.source_stage == stage)
            .expect("semantic comparison for registered stage");
        assert!(!comparison.byte_identical);
        assert!(comparison.semantically_equivalent);
    }
    let stage_handoff_v2 = read_compiler_stage_handoff_v2(
        &candidate_dir.join(COMPILER_STAGE_HANDOFF_V2_FILE),
        &handoff,
        &payloads,
    )
    .expect("verify stage handoff v2");
    assert_eq!(stage_handoff_v2.selection_count, 2);
    assert_eq!(
        stage_handoff_v2.selections[0].source_stage,
        CompilerStageKind::Ast
    );
    assert_eq!(
        stage_handoff_v2.selections[1].source_stage,
        CompilerStageKind::Nir
    );
    for stage in [CompilerStageKind::Ast, CompilerStageKind::Nir] {
        let selection = stage_handoff_v2
            .selection_for_stage(stage)
            .expect("handoff v2 selection for registered stage");
        assert!(selection.reversible);
        assert!(selection.semantically_equivalent);
    }
    assert_eq!(
        production.stage_handoff_v2_file,
        COMPILER_STAGE_HANDOFF_V2_FILE
    );
    assert_eq!(
        production.stage_handoff_v2_proof_sha256,
        stage_handoff_v2.proof_sha256
    );
    assert!(!production.replacement_authorized);

    let differential = parse_compiler_component_differential(
        &output_dir.join(COMPILER_COMPONENT_DIFFERENTIAL_FILE),
    )
    .expect("verify candidate differential");
    assert_eq!(differential.equivalent_count, 13);
    assert!(differential.deterministic_artifact_equivalent);
    assert_eq!(differential.verdict, "equivalent-awaiting-authorization");
    assert!(!differential.replacement_authorized);
    let representation_differential = read_compiler_component_representation_differential(
        &output_dir.join(COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_FILE),
        &stage0_record_path,
        &candidate_record_path,
    )
    .expect("verify candidate representation differential");
    assert_eq!(representation_differential.comparison_count, 2);
    assert_eq!(representation_differential.equivalent_count, 2);
    assert!(representation_differential.all_representations_equivalent);
    assert!(!representation_differential.replacement_authorized);
    for stage in [CompilerStageKind::Ast, CompilerStageKind::Nir] {
        let representation = representation_differential
            .comparisons
            .iter()
            .find(|comparison| comparison.source_stage == stage)
            .expect("representation comparison for registered stage");
        assert!(!representation.byte_identical);
        assert!(representation.reversible);
        assert!(representation.semantically_equivalent);
        assert!(representation.equivalent);
        assert_eq!(
            representation.stage0_payload_sha256,
            representation.candidate_recovered_payload_sha256
        );
        assert_ne!(
            representation.stage0_payload_sha256,
            representation.candidate_selected_payload_sha256
        );
    }

    let transformations_path = candidate_dir.join(COMPILER_STAGE_TRANSFORMATION_FILE);
    let transformation_source = fs::read(&transformations_path).expect("read transformations");
    let mut tampered_transformations = transformation_source.clone();
    tampered_transformations.push(0);
    fs::write(&transformations_path, tampered_transformations)
        .expect("tamper stage transformations");
    let error = read_compiler_candidate_production(
        &candidate_dir.join(COMPILER_CANDIDATE_PRODUCTION_FILE),
        &stage0,
        &execution,
        &candidate,
        &handoff,
        &payloads,
    )
    .expect_err("stage transformation tampering must invalidate production proof");
    assert!(error
        .to_string()
        .contains("stage transformations length or SHA-256 mismatch"));
    fs::write(&transformations_path, transformation_source).expect("restore transformations");

    let derived_payload_path = candidate_dir.join(&transformations.records[0].output_payload_file);
    let derived_payload = fs::read(&derived_payload_path).expect("read derived stage payload");
    let mut tampered_derived_payload = derived_payload.clone();
    tampered_derived_payload[0] ^= 0xff;
    fs::write(&derived_payload_path, tampered_derived_payload)
        .expect("tamper derived stage payload");
    let error = read_compiler_candidate_production(
        &candidate_dir.join(COMPILER_CANDIDATE_PRODUCTION_FILE),
        &stage0,
        &execution,
        &candidate,
        &handoff,
        &payloads,
    )
    .expect_err("derived stage payload tampering must invalidate production proof");
    assert!(error
        .to_string()
        .contains("derived payload length or SHA-256 mismatch"));
    fs::write(&derived_payload_path, derived_payload).expect("restore derived stage payload");

    let handoff_v2_path = candidate_dir.join(COMPILER_STAGE_HANDOFF_V2_FILE);
    let handoff_v2_source = fs::read(&handoff_v2_path).expect("read stage handoff v2");
    let mut tampered_handoff_v2 = handoff_v2_source.clone();
    tampered_handoff_v2.push(0);
    fs::write(&handoff_v2_path, tampered_handoff_v2).expect("tamper stage handoff v2");
    let error = read_compiler_candidate_production(
        &candidate_dir.join(COMPILER_CANDIDATE_PRODUCTION_FILE),
        &stage0,
        &execution,
        &candidate,
        &handoff,
        &payloads,
    )
    .expect_err("stage handoff v2 tampering must invalidate production proof");
    assert!(error
        .to_string()
        .contains("stage handoff v2 length or SHA-256 mismatch"));
    fs::write(&handoff_v2_path, handoff_v2_source).expect("restore stage handoff v2");

    let adapter_path = candidate_dir.join(COMPILER_CANDIDATE_ADAPTER_FILE);
    let adapter = fs::read(&adapter_path).expect("read candidate adapter");
    let mut tampered = adapter.clone();
    tampered.push(0);
    fs::write(&adapter_path, tampered).expect("tamper candidate adapter");
    let error = read_compiler_candidate_production(
        &candidate_dir.join(COMPILER_CANDIDATE_PRODUCTION_FILE),
        &stage0,
        &execution,
        &candidate,
        &handoff,
        &payloads,
    )
    .expect_err("adapter tampering must invalidate production proof");
    assert!(error
        .to_string()
        .contains("adapter length or SHA-256 mismatch"));
    let tampered_report_path = output_dir.join("tampered-differential.toml");
    let tampered_diff = run_nuis(&[
        "bootstrap-diff",
        &stage0_record_path.display().to_string(),
        &candidate_record_path.display().to_string(),
        &tampered_report_path.display().to_string(),
    ]);
    assert!(!tampered_diff.status.success());
    assert!(String::from_utf8_lossy(&tampered_diff.stderr)
        .contains("adapter length or SHA-256 mismatch"));
    assert!(!tampered_report_path.exists());

    fs::write(&adapter_path, adapter).expect("restore production-bound candidate adapter");
    let missing_provider_output = output_dir.join("missing-provider-result");
    let missing_provider = Command::new(&adapter_path)
        .arg("bootstrap-build")
        .arg(project)
        .arg(&missing_provider_output)
        .env_remove(COMPILER_CANDIDATE_COMPILE_PROVIDER_ENVIRONMENT)
        .output()
        .expect("execute candidate adapter without a provider");
    assert_eq!(missing_provider.status.code(), Some(66));
    assert!(missing_provider.stdout.is_empty());
    assert!(missing_provider.stderr.is_empty());
    assert!(!missing_provider_output.exists());

    let wrong_provider_output = output_dir.join("wrong-provider-result");
    let wrong_provider_capability = output_dir.join("wrong-provider-capability.toml");
    let wrong_provider = run_nuis(&[
        "bootstrap-candidate-compile-capability",
        &output_dir.display().to_string(),
        &candidate_dir
            .join(&candidate.native_binary_file)
            .display()
            .to_string(),
        project,
        &wrong_provider_output.display().to_string(),
        &wrong_provider_capability.display().to_string(),
    ]);
    assert!(!wrong_provider.status.success());
    assert!(String::from_utf8_lossy(&wrong_provider.stderr)
        .contains("compiler image identity mismatch"));
    assert!(!wrong_provider_output.exists());
    assert!(!wrong_provider_capability.exists());

    let capability_build = output_dir.join("candidate-compile-result");
    let capability_path = output_dir.join("candidate-compile-capability.toml");
    let capability = run_nuis(&[
        "bootstrap-candidate-compile-capability",
        &output_dir.display().to_string(),
        env!("CARGO_BIN_EXE_nuis"),
        project,
        &capability_build.display().to_string(),
        &capability_path.display().to_string(),
    ]);
    assert!(
        capability.status.success(),
        "candidate compile capability failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&capability.stdout),
        String::from_utf8_lossy(&capability.stderr),
    );
    let capability = parse_compiler_candidate_compile_capability(&capability_path)
        .expect("verify candidate compile capability");
    assert_eq!(
        capability.verdict,
        COMPILER_CANDIDATE_COMPILE_CAPABILITY_VERDICT
    );
    assert_eq!(capability.production_proof_sha256, production.proof_sha256);
    assert_eq!(capability.stage0_record_sha256, stage0.record_sha256);
    assert_eq!(capability.candidate_record_sha256, candidate.record_sha256);
    assert!(!capability.replacement_authorized);
    assert!(!capability.selection_authorized);
    let rebuilt =
        read_compiler_component_build(&capability_build.join(COMPILER_COMPONENT_BUILD_FILE))
            .expect("verify candidate-driven rebuild");
    assert_eq!(
        rebuilt.reproducible_build_sha256,
        stage0.reproducible_build_sha256
    );

    fs::remove_dir_all(output_dir).expect("remove candidate component output");
}

#[test]
fn two_uncached_clean_candidates_bind_one_reproducibility_aggregate() {
    let project = "../../examples/projects/tooling/bootstrap_structural_projection_candidate";
    let output_dir = temp_dir();
    let output_dir_text = output_dir.display().to_string();
    let build = run_nuis(&["bootstrap-reproducibility", project, &output_dir_text]);
    assert!(
        build.status.success(),
        "clean candidate builds failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr),
    );

    let roots = [
        output_dir.join("clean-build-0"),
        output_dir.join("clean-build-1"),
    ];
    let aggregate_path = output_dir.join(COMPILER_COMPONENT_REPRODUCIBILITY_FILE);
    let aggregate = read_compiler_component_reproducibility(&aggregate_path, &roots)
        .expect("verify clean-build aggregate");
    assert_eq!(aggregate.run_count, 2);
    assert_eq!(aggregate.equivalent_run_count, 2);
    assert_eq!(aggregate.comparison_count, 13);
    assert!(aggregate.all_runs_equivalent);
    assert_eq!(
        aggregate.verdict,
        "reproducible-equivalent-awaiting-authorization"
    );
    assert!(!aggregate.replacement_authorized);
    assert_eq!(
        aggregate.runs[0].candidate_reproducible_build_sha256,
        aggregate.runs[1].candidate_reproducible_build_sha256
    );
    assert_ne!(
        aggregate.runs[0].clean_root_witness_sha256,
        aggregate.runs[1].clean_root_witness_sha256
    );

    for root in &roots {
        let stage0 =
            read_compiler_component_build(&root.join("stage0").join(COMPILER_COMPONENT_BUILD_FILE))
                .expect("verify clean stage0 component");
        let manifest = parse_build_manifest(&root.join("stage0").join(&stage0.build_manifest_file))
            .expect("verify clean stage0 manifest");
        assert_eq!(manifest.compile_cache_status.as_deref(), Some("bypass"));
    }
    let source = fs::read_to_string(&aggregate_path).expect("read aggregate source");
    assert!(!source.contains(&output_dir_text));

    let signing_key_hex = "07".repeat(32);
    let challenge_sha256 = "f".repeat(64);
    let first_root_text = roots[0].display().to_string();
    let second_root_text = roots[1].display().to_string();
    let aggregate_text = aggregate_path.display().to_string();
    let attestation_path = output_dir.join(COMPILER_COMPONENT_ATTESTATION_FILE);
    let attestation_text = attestation_path.display().to_string();
    let signed = run_nuis_with_env(
        &[
            "bootstrap-attest-reproducibility",
            &aggregate_text,
            &first_root_text,
            &second_root_text,
            &challenge_sha256,
            "linux-builder-1",
            "linux-amd64-cleanroom",
            &attestation_text,
        ],
        "NUIS_COMPILER_ATTESTER_SIGNING_KEY_HEX",
        &signing_key_hex,
    );
    assert!(
        signed.status.success(),
        "attestation signing failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&signed.stdout),
        String::from_utf8_lossy(&signed.stderr),
    );

    let public_key_hex = SigningKey::from_bytes(&[7u8; 32])
        .verifying_key()
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let registry = build_compiler_component_attester_trust_registry(
        1,
        &[CompilerComponentAttesterTrustEntryInput {
            attester_id: "linux-builder-1",
            environment_id: "linux-amd64-cleanroom",
            public_key_hex: &public_key_hex,
            status: "active",
        }],
    )
    .expect("build attester registry");
    let registry_source = render_compiler_component_attester_trust_registry(&registry);
    let registry_sha256 = compiler_component_attester_trust_registry_sha256(&registry_source);
    let registry_path = output_dir.join("attester-registry.toml");
    fs::write(&registry_path, registry_source).expect("write attester registry");
    let registry_text = registry_path.display().to_string();
    let verified = run_nuis(&[
        "bootstrap-verify-reproducibility-attestation",
        &aggregate_text,
        &attestation_text,
        &registry_text,
        &registry_sha256,
        &challenge_sha256,
    ]);
    assert!(
        verified.status.success(),
        "attestation verification failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&verified.stdout),
        String::from_utf8_lossy(&verified.stderr),
    );
    assert!(String::from_utf8_lossy(&verified.stdout)
        .contains("bootstrap compiler attestation: verified"));
    let attestation = read_compiler_component_attestation(
        &attestation_path,
        &aggregate_path,
        &registry_path,
        &registry_sha256,
        &challenge_sha256,
    )
    .expect("read verified attestation");
    assert_eq!(
        attestation.candidate_production_protocol,
        "nuis-compiler-candidate-production-v11"
    );
    assert!(!attestation.replacement_authorized);

    let owner_key_hex = "09".repeat(32);
    let owner_public_key_hex = SigningKey::from_bytes(&[9u8; 32])
        .verifying_key()
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let owner_registry = build_compiler_component_replacement_authorizer_registry(
        1,
        &[CompilerComponentReplacementAuthorizerEntryInput {
            authorizer_id: "compiler-owner-1",
            environment_id: "release-control",
            component_id: &aggregate.component_id,
            public_key_hex: &owner_public_key_hex,
            status: "active",
        }],
    )
    .expect("build component-owner registry");
    let owner_registry_source =
        render_compiler_component_replacement_authorizer_registry(&owner_registry);
    let owner_registry_sha256 =
        compiler_component_replacement_authorizer_registry_sha256(&owner_registry_source);
    let owner_registry_path = output_dir.join("component-owner-registry.toml");
    fs::write(&owner_registry_path, owner_registry_source).expect("write component-owner registry");
    let authorization_path = output_dir.join("component-authorization.toml");
    let active_state_path = output_dir.join("component-active-state.toml");
    let transition_path = output_dir.join("component-transition.toml");
    let dispatch_path = output_dir.join(COMPILER_COMPONENT_DISPATCH_FILE);
    let authorization_challenge = "d".repeat(64);
    let transition_challenge = "e".repeat(64);

    let authorization = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-authorize-component-replacement")
        .arg(&aggregate_path)
        .arg(&attestation_path)
        .arg(&registry_path)
        .arg(&registry_sha256)
        .arg(&challenge_sha256)
        .arg(&owner_registry_path)
        .arg(&owner_registry_sha256)
        .arg(&authorization_challenge)
        .arg("compiler-owner-1")
        .arg("release-control")
        .arg("clean-build-genesis")
        .arg(&authorization_path)
        .env("NUIS_COMPILER_REPLACEMENT_SIGNING_KEY_HEX", &owner_key_hex)
        .output()
        .expect("authorize clean-build candidate");
    assert_success(&authorization, "clean-build component authorization");

    let activation = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-activate-component")
        .arg(&aggregate_path)
        .arg(&attestation_path)
        .arg(&registry_path)
        .arg(&registry_sha256)
        .arg(&challenge_sha256)
        .arg(&authorization_path)
        .arg(&owner_registry_path)
        .arg(&owner_registry_sha256)
        .arg(&authorization_challenge)
        .arg(&active_state_path)
        .output()
        .expect("activate clean-build candidate");
    assert_success(&activation, "clean-build component activation");

    let rollback = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-rollback-component")
        .arg(&aggregate_path)
        .arg(&attestation_path)
        .arg(&registry_path)
        .arg(&registry_sha256)
        .arg(&challenge_sha256)
        .arg(&authorization_path)
        .arg(&owner_registry_path)
        .arg(&owner_registry_sha256)
        .arg(&authorization_challenge)
        .arg(&active_state_path)
        .arg(&transition_challenge)
        .arg("compiler-owner-1")
        .arg("release-control")
        .arg("clean-build-rollback-2")
        .arg(&transition_path)
        .env("NUIS_COMPILER_REPLACEMENT_SIGNING_KEY_HEX", &owner_key_hex)
        .output()
        .expect("roll back clean-build candidate");
    assert_success(&rollback, "clean-build generation-two rollback");

    let selected_root = &roots[0];
    let stage0_dir = selected_root.join("stage0");
    let candidate_dir = selected_root.join("stage1-candidate");
    let stage0_record = stage0_dir.join(COMPILER_COMPONENT_BUILD_FILE);
    let candidate_record = candidate_dir.join(COMPILER_COMPONENT_BUILD_FILE);
    let candidate_component =
        read_compiler_component_build(&candidate_record).expect("verify dispatch candidate");
    let candidate_image = candidate_dir.join(&candidate_component.native_binary_file);
    let preselection_build = output_dir.join("preselection-capability-build");
    let preselection_capability = output_dir.join("preselection-capability.toml");
    let capability = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-candidate-compile-capability")
        .arg(selected_root)
        .arg(env!("CARGO_BIN_EXE_nuis"))
        .arg(project)
        .arg(&preselection_build)
        .arg(&preselection_capability)
        .output()
        .expect("build production-bound capability for preselection");
    assert_success(&capability, "generation-three candidate capability");
    let preselection_path = output_dir.join("candidate-preselection.toml");
    let preselection_challenge = "f".repeat(64);
    let transition_before = fs::read(&transition_path).expect("snapshot generation two");
    let preselection = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-preselect-candidate")
        .arg(&aggregate_path)
        .arg(&attestation_path)
        .arg(&registry_path)
        .arg(&registry_sha256)
        .arg(&challenge_sha256)
        .arg(&authorization_path)
        .arg(&owner_registry_path)
        .arg(&owner_registry_sha256)
        .arg(&authorization_challenge)
        .arg(&active_state_path)
        .arg(&transition_path)
        .arg(&transition_challenge)
        .arg(selected_root)
        .arg(&preselection_capability)
        .arg(&preselection_challenge)
        .arg("compiler-owner-1")
        .arg("release-control")
        .arg("clean-build-preselection-3")
        .arg(&preselection_path)
        .env("NUIS_COMPILER_REPLACEMENT_SIGNING_KEY_HEX", &owner_key_hex)
        .output()
        .expect("preselect production-bound compiler candidate");
    assert_success(
        &preselection,
        "signed generation-three candidate preselection",
    );
    assert_eq!(
        fs::read(&transition_path).expect("reread generation two"),
        transition_before
    );
    let preselection = parse_compiler_candidate_preselection(&preselection_path)
        .expect("verify candidate preselection record");
    assert_eq!(
        preselection.verdict,
        COMPILER_CANDIDATE_PRESELECTION_VERDICT
    );
    assert_eq!(preselection.target_generation, 3);
    assert!(preselection.provider_dependency_required);
    assert!(!preselection.direct_stage1_compile);
    assert!(!preselection.selection_authorized);
    assert!(!preselection.replacement_authorized);
    let preselection_source =
        fs::read_to_string(&preselection_path).expect("read candidate preselection source");
    assert!(!preselection_source.contains(&output_dir_text));
    let dispatch = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-dispatch-component")
        .arg(&aggregate_path)
        .arg(&attestation_path)
        .arg(&registry_path)
        .arg(&registry_sha256)
        .arg(&challenge_sha256)
        .arg(&authorization_path)
        .arg(&owner_registry_path)
        .arg(&owner_registry_sha256)
        .arg(&authorization_challenge)
        .arg(&active_state_path)
        .arg(&transition_path)
        .arg(&transition_challenge)
        .arg(&stage0_record)
        .arg(env!("CARGO_BIN_EXE_nuis"))
        .arg(&candidate_record)
        .arg(&candidate_image)
        .arg(&dispatch_path)
        .output()
        .expect("dispatch verified stage0 image");
    assert_success(&dispatch, "path-free stage-driver dispatch");
    assert!(String::from_utf8_lossy(&dispatch.stdout)
        .contains("bootstrap compiler component: dispatched"));
    let receipt = parse_compiler_component_dispatch_receipt(&dispatch_path)
        .expect("verify component dispatch receipt");
    assert_eq!(receipt.verdict, COMPILER_COMPONENT_DISPATCH_VERDICT);
    assert_eq!(receipt.selected_stage_role, "stage0");
    assert_eq!(receipt.forward_stage_role, "stage1-candidate");
    assert_eq!(
        receipt.selected_reproducible_build_sha256,
        aggregate.stage0_reproducible_build_sha256
    );
    assert_eq!(
        receipt.forward_reproducible_build_sha256,
        aggregate.candidate_reproducible_build_sha256
    );
    let receipt_source = fs::read_to_string(&dispatch_path).expect("read dispatch receipt");
    assert!(!receipt_source.contains(&output_dir_text));
    assert!(!fs::read_dir(&output_dir)
        .expect("scan dispatch output")
        .filter_map(Result::ok)
        .any(|entry| entry
            .file_name()
            .to_string_lossy()
            .starts_with(".nuis-stage-driver-")));

    let compile_output = output_dir.join("selected-current-build");
    let compile_dispatch_path = output_dir.join(COMPILER_COMPONENT_COMPILE_DISPATCH_FILE);
    let compile_dispatch = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-dispatch-compile")
        .arg(&aggregate_path)
        .arg(&attestation_path)
        .arg(&registry_path)
        .arg(&registry_sha256)
        .arg(&challenge_sha256)
        .arg(&authorization_path)
        .arg(&owner_registry_path)
        .arg(&owner_registry_sha256)
        .arg(&authorization_challenge)
        .arg(&active_state_path)
        .arg(&transition_path)
        .arg(&transition_challenge)
        .arg(&stage0_record)
        .arg(env!("CARGO_BIN_EXE_nuis"))
        .arg(&candidate_record)
        .arg(&candidate_image)
        .arg(project)
        .arg(&compile_output)
        .arg(&compile_dispatch_path)
        .output()
        .expect("compile through verified stage0 image");
    assert_success(
        &compile_dispatch,
        "canonical compile request through selected current image",
    );
    let compile_receipt = parse_compiler_component_compile_dispatch_receipt(&compile_dispatch_path)
        .expect("verify component compile dispatch receipt");
    assert_eq!(
        compile_receipt.verdict,
        COMPILER_COMPONENT_COMPILE_DISPATCH_VERDICT
    );
    assert_eq!(
        compile_receipt.request_reproducible_build_sha256,
        compile_receipt.result_reproducible_build_sha256
    );
    assert_eq!(
        compile_receipt.forward_reproducible_build_sha256,
        aggregate.candidate_reproducible_build_sha256
    );
    let compiled_component =
        read_compiler_component_build(&compile_output.join(COMPILER_COMPONENT_BUILD_FILE))
            .expect("verify selected-current compile result");
    assert_eq!(
        compiled_component.compiler_image_sha256,
        compile_receipt.selected_compiler_image_sha256
    );
    let compile_receipt_source =
        fs::read_to_string(&compile_dispatch_path).expect("read compile dispatch receipt");
    assert!(!compile_receipt_source.contains(&output_dir_text));
    assert!(!fs::read_dir(&output_dir)
        .expect("scan compile dispatch output")
        .filter_map(Result::ok)
        .any(|entry| entry
            .file_name()
            .to_string_lossy()
            .starts_with(".nuis-stage-driver-")));

    let replay = run_nuis(&[
        "bootstrap-verify-reproducibility-attestation",
        &aggregate_text,
        &attestation_text,
        &registry_text,
        &registry_sha256,
        &"e".repeat(64),
    ]);
    assert!(!replay.status.success());
    assert!(String::from_utf8_lossy(&replay.stderr).contains("verifier request"));

    let rerun = run_nuis(&["bootstrap-reproducibility", project, &output_dir_text]);
    assert!(!rerun.status.success());
    assert!(String::from_utf8_lossy(&rerun.stderr).contains("must be empty"));

    let candidate = read_compiler_component_build(
        &roots[1]
            .join("stage1-candidate")
            .join(COMPILER_COMPONENT_BUILD_FILE),
    )
    .expect("verify second clean candidate");
    let candidate_binary = roots[1]
        .join("stage1-candidate")
        .join(&candidate.native_binary_file);
    let mut tampered = fs::read(&candidate_binary).expect("read second candidate binary");
    tampered.push(0);
    fs::write(&candidate_binary, tampered).expect("tamper second candidate binary");
    let error = read_compiler_component_reproducibility(&aggregate_path, &roots)
        .expect_err("bound root tampering must invalidate aggregate");
    assert!(error
        .to_string()
        .contains("native binary length or SHA-256 mismatch"));

    fs::remove_dir_all(output_dir).expect("remove reproducibility output");
}

fn run_nuis(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nuis"))
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to run nuis {args:?}: {error}"))
}

fn assert_success(output: &std::process::Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} failed\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn run_nuis_with_env(args: &[&str], key: &str, value: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nuis"))
        .args(args)
        .env(key, value)
        .output()
        .unwrap_or_else(|error| panic!("failed to run nuis {args:?}: {error}"))
}

#[test]
fn pure_nuis_candidate_consumes_structural_ast_and_nir_records() {
    let project = "../../examples/projects/tooling/bootstrap_structural_projection_candidate";
    let output_dir = temp_dir();
    let output_dir_text = output_dir.display().to_string();
    let build = run_nuis(&["bootstrap-candidate-probe", project, &output_dir_text]);
    assert!(
        build.status.success(),
        "candidate build failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr),
    );

    let component = read_compiler_component_build(&output_dir.join(COMPILER_COMPONENT_BUILD_FILE))
        .expect("verify candidate component build");
    assert_eq!(component.stage_role, "stage0");
    assert_eq!(component.producer_id, "nuisc-stage0-reference");
    assert_eq!(
        component.component_id,
        "bootstrap_structural_projection_candidate"
    );
    assert!(component.dependencies.iter().any(|dependency| {
        dependency.kind == "galaxy-library"
            && dependency.identity == "nuis.std@workspace:lib/compiler_projection.ns"
    }));
    let execution =
        read_compiler_candidate_execution(&output_dir.join(COMPILER_CANDIDATE_EXECUTION_FILE))
            .expect("verify candidate execution proof");
    assert_eq!(execution.probe_role, COMPILER_CANDIDATE_EXECUTION_ROLE);
    assert_eq!(execution.authority, COMPILER_CANDIDATE_EXECUTION_AUTHORITY);
    assert_eq!(execution.component_record_sha256, component.record_sha256);
    assert_eq!(execution.exit_code, 0);

    let (handoff, payloads) =
        read_compiler_stage_handoff(&output_dir.join("nuis.compiler-stage-handoff.toml"))
            .expect("verify candidate stage handoff");
    assert_eq!(handoff.module_domain, "cpu");
    assert_eq!(handoff.module_unit, "Main");

    for (stage, kind) in [
        (CompilerStageKind::Ast, CompilerProjectionKind::Ast),
        (CompilerStageKind::Nir, CompilerProjectionKind::Nir),
    ] {
        let payload = payloads
            .iter()
            .find(|payload| payload.stage == stage)
            .expect("find structural projection payload");
        let source = std::str::from_utf8(&payload.bytes).expect("projection must be UTF-8");
        let projection = parse_compiler_structural_projection(kind, source)
            .expect("decode producer-neutral structural projection");
        assert_eq!(projection.module_domain, "cpu");
        assert_eq!(projection.module_unit, "Main");
        assert!(projection
            .records
            .iter()
            .any(|record| record.kind == CompilerProjectionRecordKind::Item));
    }

    let run = Command::new(output_dir.join("bootstrap_structural_projection_candidate"))
        .output()
        .expect("run structural projection candidate");
    assert_eq!(run.status.code(), Some(0));
    assert!(run.stdout.is_empty());
    assert!(run.stderr.is_empty());

    let native_path = output_dir.join("bootstrap_structural_projection_candidate");
    let mut tampered = fs::read(&native_path).expect("read candidate binary for tamper check");
    tampered.push(0);
    fs::write(&native_path, tampered).expect("tamper candidate binary");
    let error =
        read_compiler_candidate_execution(&output_dir.join(COMPILER_CANDIDATE_EXECUTION_FILE))
            .expect_err("tampered candidate binary must invalidate its execution proof");
    assert!(error
        .to_string()
        .contains("native binary length or SHA-256 mismatch"));

    fs::remove_dir_all(output_dir).expect("remove structural projection candidate output");
}
