use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use nuis_artifact::{
    parse_build_manifest, parse_compiler_component_differential,
    parse_compiler_structural_projection, read_compiler_candidate_execution,
    read_compiler_candidate_production, read_compiler_component_build,
    read_compiler_component_reproducibility, read_compiler_stage_handoff,
    read_compiler_stage_semantic_differential, CompilerProjectionKind,
    CompilerProjectionRecordKind, CompilerStageKind, CompilerStageSemanticDifferentialInput,
    COMPILER_CANDIDATE_ADAPTER_FILE, COMPILER_CANDIDATE_EXECUTION_AUTHORITY,
    COMPILER_CANDIDATE_EXECUTION_FILE, COMPILER_CANDIDATE_EXECUTION_ROLE,
    COMPILER_CANDIDATE_PRODUCTION_FILE, COMPILER_COMPONENT_BUILD_FILE,
    COMPILER_COMPONENT_DIFFERENTIAL_FILE, COMPILER_COMPONENT_REPRODUCIBILITY_FILE,
    COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE, COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_FILE,
    COMPILER_STAGE_TRANSFORMATION_FILE,
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
    assert_eq!(transformations.record_count, 1);
    assert_eq!(
        transformations.records[0].source_stage,
        CompilerStageKind::Nir
    );
    assert_eq!(transformations.records[0].output_word_count, 22);
    assert_eq!(transformations.records[0].output_words[0], 2);
    assert_eq!(transformations.records[0].output_words[1], 2);
    assert_eq!(transformations.records[0].output_words[2], 132_469_386_887);
    assert_eq!(transformations.records[0].output_words[3], 754_343_074);
    assert_eq!(transformations.records[0].output_words[12], 146_705_724_977);
    assert_eq!(transformations.records[0].output_words[13], 38_998_897);
    let derived_payload_path = candidate_dir.join(&transformations.records[0].output_payload_file);
    let derived_payload = fs::read(&derived_payload_path).expect("read compact derived payload");
    let legacy_v2_bytes = 8
        + 3 * std::mem::size_of::<u64>()
        + 22 * std::mem::size_of::<u64>()
        + payloads[3].bytes.len();
    assert_eq!(
        transformations.records[0].output_payload_bytes,
        derived_payload.len()
    );
    assert!(derived_payload.len() < payloads[3].bytes.len());
    assert!(derived_payload.len() < legacy_v2_bytes);
    assert!(!derived_payload
        .windows(payloads[3].bytes.len())
        .any(|window| window == payloads[3].bytes));
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
    assert_eq!(semantic_differential.equivalent_count, 1);
    assert!(semantic_differential.deterministic_semantic_equivalent);
    assert!(!semantic_differential.comparisons[0].byte_identical);
    assert!(semantic_differential.comparisons[0].semantically_equivalent);
    assert!(!production.replacement_authorized);

    let differential = parse_compiler_component_differential(
        &output_dir.join(COMPILER_COMPONENT_DIFFERENTIAL_FILE),
    )
    .expect("verify candidate differential");
    assert_eq!(differential.equivalent_count, 13);
    assert!(differential.deterministic_artifact_equivalent);
    assert_eq!(differential.verdict, "equivalent-awaiting-authorization");
    assert!(!differential.replacement_authorized);

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

    let adapter_path = candidate_dir.join(COMPILER_CANDIDATE_ADAPTER_FILE);
    let mut tampered = fs::read(&adapter_path).expect("read candidate adapter");
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
