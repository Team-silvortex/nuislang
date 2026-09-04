use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use nuis_artifact::{
    parse_build_manifest, read_compiler_component_build, read_compiler_diagnostic_report,
    read_compiler_stage_handoff, CompilerStageKind, COMPILER_COMPONENT_BUILD_FILE,
    COMPILER_COMPONENT_BUILD_PROTOCOL, COMPILER_COMPONENT_DRIVER_CONTRACT,
    COMPILER_COMPONENT_REPRODUCIBLE_IDENTITY_CONTRACT, COMPILER_DIAGNOSTIC_REPORT_FILE,
    COMPILER_DIAGNOSTIC_REPORT_PROTOCOL, COMPILER_STAGE_HANDOFF_PROTOCOL,
    COMPILER_STAGE_PRODUCER_CONTRACT,
};

fn temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should follow the Unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nuis_compiler_data_model_{nonce}"));
    fs::create_dir_all(&dir).expect("create compiler data model output directory");
    dir
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

fn assert_file_not_contains(path: &Path, needle: &str) {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    assert!(
        !source.contains(needle),
        "expected {} not to contain `{needle}`",
        path.display()
    );
}

fn assert_file_contains(path: &Path, needle: &str) {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    assert!(
        source.contains(needle),
        "expected {} to contain `{needle}`",
        path.display()
    );
}

#[test]
fn compiler_data_model_bootstrap_builds_and_runs_as_pure_nuis() {
    let project = "../../examples/projects/tooling/bootstrap_compiler_data_model_demo";
    let output_dir = temp_dir();
    let output_dir_text = output_dir.display().to_string();
    let build = run_nuis(&["bootstrap-build", project, &output_dir_text]);
    assert_success(&build, "compiler data model build");
    let llvm_path = output_dir.join("bootstrap_compiler_data_model_demo.ll");
    assert_file_not_contains(&llvm_path, "deferred lowering");
    for symbol in [
        "@nuis_fn_StdCompilerData.compiler_text_arena_store",
        "@nuis_fn_StdCompilerData.compiler_text_arena_get",
        "@nuis_fn_StdCompilerData.compiler_text_arena_identity",
        "@nuis_fn_StdCompilerPayload.compiler_payload_page_identity",
        "@nuis_fn_StdCompilerPayload.compiler_paged_text_arena_store",
        "@nuis_fn_StdCompilerPayload.compiler_paged_text_arena_get",
        "@nuis_fn_StdCompilerPayload.compiler_paged_text_arena_identity",
        "@nuis_fn_StdCompilerPayload.compiler_payload_copy_buffer_range",
        "@nuis_fn_StdCompilerPayloadRegistry.compiler_payload_registry_register",
        "@nuis_fn_StdCompilerPayloadRegistry.compiler_chunked_payload",
        "@nuis_fn_StdCompilerPayloadRegistry.compiler_aggregate_arena_store_chunked",
        "@nuis_fn_StdCompilerPayloadRegistry.compiler_aggregate_arena_get_chunked",
        "@nuis_fn_StdCompilerPayloadRegistry.compiler_aggregate_arena_store_text",
        "@nuis_fn_StdCompilerPayloadRegistry.compiler_aggregate_arena_store_source_span",
        "@nuis_fn_StdCompilerPayloadRegistry.compiler_aggregate_arena_get_text",
        "@nuis_fn_StdCompilerPayloadRegistry.compiler_aggregate_arena_get_source_span",
        "@nuis_fn_StdCompilerPayloadRegistry.compiler_aggregate_arena_identity",
        "@nuis_fn_StdCompilerPayloadRegistry.compiler_aggregate_arena_forward_checked",
        "@nuis_fn_StdCompilerPayloadRegistry.compiler_aggregate_arena_forward",
    ] {
        assert_file_contains(&llvm_path, symbol);
    }

    let stage_manifest_path = output_dir.join("nuis.compiler-stage-handoff.toml");
    let (handoff, payloads) =
        read_compiler_stage_handoff(&stage_manifest_path).expect("verify compiler stage handoff");
    assert_eq!(handoff.protocol, COMPILER_STAGE_HANDOFF_PROTOCOL);
    assert_eq!(handoff.producer_contract, COMPILER_STAGE_PRODUCER_CONTRACT);
    assert_eq!(handoff.producer_id, "nuisc-stage0-reference");
    assert_eq!(handoff.module_domain, "cpu");
    assert_eq!(handoff.module_unit, "Main");
    assert_eq!(handoff.bundle_sha256.len(), 64);
    assert_eq!(
        handoff
            .records
            .iter()
            .map(|record| record.stage)
            .collect::<Vec<_>>(),
        vec![
            CompilerStageKind::Source,
            CompilerStageKind::Tokens,
            CompilerStageKind::Ast,
            CompilerStageKind::Nir,
            CompilerStageKind::Yir,
        ]
    );
    assert_eq!(payloads.len(), 5);

    let component_build_path = output_dir.join(COMPILER_COMPONENT_BUILD_FILE);
    let first_component_build =
        read_compiler_component_build(&component_build_path).expect("verify component build");
    assert_eq!(
        first_component_build.protocol,
        COMPILER_COMPONENT_BUILD_PROTOCOL
    );
    assert_eq!(
        first_component_build.driver_contract,
        COMPILER_COMPONENT_DRIVER_CONTRACT
    );
    assert_eq!(first_component_build.stage_role, "stage0");
    assert_eq!(
        first_component_build.component_id,
        "bootstrap_compiler_data_model_demo"
    );
    assert_eq!(first_component_build.producer_id, "nuisc-stage0-reference");
    assert_eq!(
        first_component_build.stage_handoff_bundle_sha256,
        handoff.bundle_sha256
    );
    assert_eq!(
        first_component_build.reproducible_identity_contract,
        COMPILER_COMPONENT_REPRODUCIBLE_IDENTITY_CONTRACT
    );
    for identity in [
        first_component_build.compiler_image_sha256.as_str(),
        first_component_build.dependency_closure_sha256.as_str(),
        first_component_build.reproducible_build_sha256.as_str(),
        first_component_build.record_sha256.as_str(),
    ] {
        assert_eq!(identity.len(), 64);
    }
    for kind in [
        "component-manifest",
        "component-source",
        "galaxy-lock",
        "galaxy-manifest",
        "galaxy-source",
        "galaxy-library",
        "nustar-index",
        "nustar-manifest",
    ] {
        assert!(
            first_component_build
                .dependencies
                .iter()
                .any(|dependency| dependency.kind == kind),
            "expected component build to attest `{kind}`"
        );
    }
    let diagnostic_path = output_dir.join(COMPILER_DIAGNOSTIC_REPORT_FILE);
    let first_diagnostics = read_compiler_diagnostic_report(
        &diagnostic_path,
        &first_component_build.record_sha256,
        &first_component_build.producer_id,
    )
    .expect("verify compiler diagnostic report");
    assert_eq!(
        first_diagnostics.protocol,
        COMPILER_DIAGNOSTIC_REPORT_PROTOCOL
    );
    assert!(first_diagnostics.accepted);
    assert_eq!(first_diagnostics.semantic_pipeline, "checked");
    assert_eq!(first_diagnostics.diagnostic_count, 0);

    let build_manifest = parse_build_manifest(&output_dir.join("nuis.build.manifest.toml"))
        .expect("parse compiler data model build manifest");
    for kind in [
        "compiler_source",
        "compiler_tokens",
        "compiler_stage_handoff",
    ] {
        assert!(
            build_manifest
                .artifact_hashes
                .iter()
                .any(|artifact| artifact.kind == kind),
            "expected build manifest to hash `{kind}`"
        );
    }

    let run = Command::new(output_dir.join("bootstrap_compiler_data_model_demo"))
        .output()
        .expect("run compiler data model binary");
    assert_eq!(
        run.status.code(),
        Some(130),
        "compiler data model binary should return its deterministic compiler score"
    );

    let rebuild = run_nuis(&["bootstrap-build", project, &output_dir_text]);
    assert_success(&rebuild, "cached compiler data model rebuild");
    let second_component_build = read_compiler_component_build(&component_build_path)
        .expect("verify cached component build");
    assert_eq!(
        first_component_build.dependency_closure_sha256,
        second_component_build.dependency_closure_sha256
    );
    assert_eq!(
        first_component_build.stage_handoff_bundle_sha256,
        second_component_build.stage_handoff_bundle_sha256
    );
    assert_eq!(
        first_component_build.compiler_image_sha256,
        second_component_build.compiler_image_sha256
    );
    assert_eq!(
        first_component_build.native_binary_sha256,
        second_component_build.native_binary_sha256
    );
    assert_eq!(
        first_component_build.reproducible_build_sha256,
        second_component_build.reproducible_build_sha256
    );
    let second_diagnostics = read_compiler_diagnostic_report(
        &diagnostic_path,
        &second_component_build.record_sha256,
        &second_component_build.producer_id,
    )
    .expect("verify cached compiler diagnostic report");
    assert_eq!(
        first_diagnostics.diagnostics_sha256,
        second_diagnostics.diagnostics_sha256
    );

    let candidate_record_path = output_dir.join("candidate.compiler-component-build.toml");
    fs::copy(&component_build_path, &candidate_record_path)
        .expect("copy stage0 record into wrong-role candidate fixture");
    let differential_path = output_dir.join("nuis.compiler-component-diff.toml");
    let differential = run_nuis(&[
        "bootstrap-diff",
        &component_build_path.display().to_string(),
        &candidate_record_path.display().to_string(),
        &differential_path.display().to_string(),
    ]);
    assert!(!differential.status.success());
    assert!(String::from_utf8_lossy(&differential.stderr).contains("stage1-candidate"));
    assert!(!differential_path.exists());
    fs::remove_file(candidate_record_path).expect("remove wrong-role candidate fixture");

    let native_path = output_dir.join("bootstrap_compiler_data_model_demo");
    let mut tampered_native = fs::read(&native_path).expect("read native binary for tamper check");
    tampered_native.push(0);
    fs::write(&native_path, tampered_native).expect("tamper native binary");
    let error = read_compiler_component_build(&component_build_path)
        .expect_err("tampered native binary must invalidate component build");
    assert!(error
        .to_string()
        .contains("native binary length or SHA-256 mismatch"));

    fs::remove_dir_all(output_dir).expect("remove compiler data model output directory");
}
