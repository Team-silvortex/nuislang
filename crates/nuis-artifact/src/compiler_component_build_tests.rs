use std::path::Path;

use super::*;

fn build_record() -> CompilerComponentBuild {
    build_record_with_operational_outputs(b"build manifest", b"compiled artifact")
}

fn build_record_with_operational_outputs(
    build_manifest: &[u8],
    compiled_artifact: &[u8],
) -> CompilerComponentBuild {
    build_record_for_stage_and_outputs(
        COMPILER_COMPONENT_STAGE0_ROLE,
        build_manifest,
        compiled_artifact,
    )
}

fn build_record_for_stage_and_outputs(
    stage_role: &str,
    build_manifest: &[u8],
    compiled_artifact: &[u8],
) -> CompilerComponentBuild {
    let dependencies = [
        CompilerComponentDependencyInput {
            kind: "nustar-manifest",
            identity: "official.cpu:cpu.toml",
            bytes: b"package_id = \"official.cpu\"\n",
        },
        CompilerComponentDependencyInput {
            kind: "component-source",
            identity: "main.ns",
            bytes: b"mod cpu Main {}\n",
        },
    ];
    build_compiler_component_build(&CompilerComponentBuildInput {
        stage_role,
        bootstrap_subset_protocol: "nuis-bootstrap-language-subset-v2",
        component_id: "compiler_scanner",
        component_domain: "cpu",
        component_unit: "Main",
        producer_id: "nuisc-stage0-reference",
        compiler_image: b"stage0 compiler image",
        stage_handoff_file: "nuis.compiler-stage-handoff.toml",
        stage_handoff_bundle_sha256:
            "1111111111111111111111111111111111111111111111111111111111111111",
        build_manifest_file: "nuis.build.manifest.toml",
        build_manifest,
        compiled_artifact_file: "nuis.compiled.artifact",
        compiled_artifact,
        native_binary_file: "compiler_scanner",
        native_binary: b"native binary",
        dependencies: &dependencies,
    })
    .expect("build compiler component record")
}

#[test]
fn candidate_stage1_role_is_explicit_and_identity_bound() {
    let stage0 = build_record();
    let candidate = build_record_for_stage_and_outputs(
        COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE,
        b"build manifest",
        b"compiled artifact",
    );

    assert_eq!(
        candidate.stage_role,
        COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE
    );
    assert_eq!(
        stage0.dependency_closure_sha256,
        candidate.dependency_closure_sha256
    );
    assert_eq!(
        stage0.compiler_image_sha256,
        candidate.compiler_image_sha256
    );
    assert_eq!(stage0.native_binary_sha256, candidate.native_binary_sha256);
    assert_ne!(
        stage0.reproducible_build_sha256,
        candidate.reproducible_build_sha256
    );
    assert_ne!(stage0.record_sha256, candidate.record_sha256);
}

#[test]
fn reproducible_identity_ignores_operational_cache_record_changes() {
    let miss = build_record_with_operational_outputs(b"cache = miss", b"container cache miss");
    let hit = build_record_with_operational_outputs(b"cache = hit", b"container cache hit");

    assert_ne!(miss.build_manifest_sha256, hit.build_manifest_sha256);
    assert_ne!(miss.compiled_artifact_sha256, hit.compiled_artifact_sha256);
    assert_ne!(miss.record_sha256, hit.record_sha256);
    assert_eq!(
        miss.reproducible_build_sha256,
        hit.reproducible_build_sha256
    );
}

#[test]
fn component_build_round_trips_with_sorted_dependency_closure() {
    let build = build_record();
    assert_eq!(build.protocol, COMPILER_COMPONENT_BUILD_PROTOCOL);
    assert_eq!(build.driver_contract, COMPILER_COMPONENT_DRIVER_CONTRACT);
    assert_eq!(build.dependencies[0].kind, "component-source");
    assert_eq!(build.dependencies[1].kind, "nustar-manifest");

    let source = render_compiler_component_build(&build);
    let parsed = parse_compiler_component_build_from_source(
        &source,
        Path::new("nuis.compiler-component-build.toml"),
    )
    .expect("parse compiler component record");
    assert_eq!(parsed, build);
    assert_eq!(render_compiler_component_build(&parsed), source);
}

#[test]
fn component_build_rejects_record_and_dependency_tampering() {
    let source = render_compiler_component_build(&build_record());
    let record_tamper = source.replacen("stage_role = \"stage0\"", "stage_role = \"stage1\"", 1);
    let error = parse_compiler_component_build_from_source(
        &record_tamper,
        Path::new("nuis.compiler-component-build.toml"),
    )
    .expect_err("stage role tamper must fail");
    assert!(error.to_string().contains("unsupported protocol contract"));

    let dependency_tamper = source.replacen("identity = \"main.ns\"", "identity = \"other.ns\"", 1);
    let error = parse_compiler_component_build_from_source(
        &dependency_tamper,
        Path::new("nuis.compiler-component-build.toml"),
    )
    .expect_err("dependency tamper must fail");
    assert!(error.to_string().contains("closure identity mismatch"));
}

#[test]
fn component_build_rejects_duplicate_dependencies_and_escaping_files() {
    let duplicate = CompilerComponentDependencyInput {
        kind: "component-source",
        identity: "main.ns",
        bytes: b"mod cpu Main {}\n",
    };
    let dependencies = [duplicate, duplicate];
    let mut input = CompilerComponentBuildInput {
        stage_role: COMPILER_COMPONENT_STAGE0_ROLE,
        bootstrap_subset_protocol: "nuis-bootstrap-language-subset-v2",
        component_id: "compiler_scanner",
        component_domain: "cpu",
        component_unit: "Main",
        producer_id: "nuisc-stage0-reference",
        compiler_image: b"stage0 compiler image",
        stage_handoff_file: "nuis.compiler-stage-handoff.toml",
        stage_handoff_bundle_sha256:
            "1111111111111111111111111111111111111111111111111111111111111111",
        build_manifest_file: "nuis.build.manifest.toml",
        build_manifest: b"build manifest",
        compiled_artifact_file: "nuis.compiled.artifact",
        compiled_artifact: b"compiled artifact",
        native_binary_file: "compiler_scanner",
        native_binary: b"native binary",
        dependencies: &dependencies,
    };
    let error = build_compiler_component_build(&input).expect_err("duplicate must fail");
    assert!(error.to_string().contains("duplicate compiler dependency"));

    input.dependencies = &dependencies[..1];
    input.native_binary_file = "../compiler_scanner";
    let error = build_compiler_component_build(&input).expect_err("escape must fail");
    assert!(error.to_string().contains("one relative file name"));
}

#[test]
fn compiler_image_verification_is_fail_closed() {
    let build = build_record();
    verify_compiler_component_build_image(&build, b"stage0 compiler image")
        .expect("matching image");
    let error = verify_compiler_component_build_image(&build, b"different image")
        .expect_err("different image must fail");
    assert!(error.to_string().contains("image identity mismatch"));
}
