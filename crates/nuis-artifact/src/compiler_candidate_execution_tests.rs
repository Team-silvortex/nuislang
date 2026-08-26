use std::path::Path;

use super::*;
use crate::{
    CompilerComponentBuild, COMPILER_COMPONENT_DEPENDENCY_CLOSURE_CONTRACT,
    COMPILER_COMPONENT_DRIVER_CONTRACT, COMPILER_COMPONENT_REPRODUCIBLE_IDENTITY_CONTRACT,
};

fn component() -> CompilerComponentBuild {
    CompilerComponentBuild {
        protocol: COMPILER_COMPONENT_BUILD_PROTOCOL.to_owned(),
        driver_contract: COMPILER_COMPONENT_DRIVER_CONTRACT.to_owned(),
        stage_role: COMPILER_COMPONENT_STAGE0_ROLE.to_owned(),
        bootstrap_subset_protocol: "nuis-bootstrap-language-subset-v2".to_owned(),
        component_id: "projection-candidate".to_owned(),
        component_domain: "cpu".to_owned(),
        component_unit: "Main".to_owned(),
        producer_id: "nuisc-stage0-reference".to_owned(),
        compiler_image_bytes: 1,
        compiler_image_sha256: "1".repeat(64),
        stage_handoff_file: "nuis.compiler-stage-handoff.toml".to_owned(),
        stage_handoff_bundle_sha256: "2".repeat(64),
        build_manifest_file: "nuis.build.manifest.toml".to_owned(),
        build_manifest_bytes: 1,
        build_manifest_sha256: "3".repeat(64),
        compiled_artifact_file: "nuis.compiled.artifact".to_owned(),
        compiled_artifact_bytes: 1,
        compiled_artifact_sha256: "4".repeat(64),
        native_binary_file: "projection-candidate".to_owned(),
        native_binary_bytes: 16,
        native_binary_sha256: "5".repeat(64),
        dependency_closure_contract: COMPILER_COMPONENT_DEPENDENCY_CLOSURE_CONTRACT.to_owned(),
        dependency_count: 1,
        dependency_closure_sha256: "6".repeat(64),
        reproducible_identity_contract: COMPILER_COMPONENT_REPRODUCIBLE_IDENTITY_CONTRACT
            .to_owned(),
        reproducible_build_sha256: "7".repeat(64),
        record_sha256: "8".repeat(64),
        dependencies: vec![],
    }
}

#[test]
fn candidate_execution_is_canonical_and_explicitly_non_authoritative() {
    let execution = build_compiler_candidate_execution(&CompilerCandidateExecutionInput {
        component: &component(),
        exit_code: 0,
        stdout: &[],
        stderr: &[],
    })
    .expect("build candidate execution");
    assert_eq!(
        execution.authority,
        "execution-only-no-component-production"
    );
    assert_eq!(execution.probe_role, "stage1-candidate-probe");
    let source = render_compiler_candidate_execution(&execution);
    let parsed = parse_compiler_candidate_execution_from_source(
        &source,
        Path::new("nuis.compiler-candidate-execution.toml"),
    )
    .expect("parse candidate execution");
    assert_eq!(parsed, execution);
}

#[test]
fn candidate_execution_rejects_process_output_or_failure() {
    for (exit_code, stdout, stderr) in [
        (1, &b""[..], &b""[..]),
        (0, &b"output"[..], &b""[..]),
        (0, &b""[..], &b"error"[..]),
    ] {
        let error = build_compiler_candidate_execution(&CompilerCandidateExecutionInput {
            component: &component(),
            exit_code,
            stdout,
            stderr,
        })
        .expect_err("invalid process result must fail");
        assert!(error.to_string().contains("requires exit 0"));
    }
}

#[test]
fn candidate_execution_rejects_identity_tampering() {
    let execution = build_compiler_candidate_execution(&CompilerCandidateExecutionInput {
        component: &component(),
        exit_code: 0,
        stdout: &[],
        stderr: &[],
    })
    .expect("build candidate execution");
    let source = render_compiler_candidate_execution(&execution).replacen(
        &format!("execution_sha256 = \"{}\"", execution.execution_sha256),
        &format!("execution_sha256 = \"{}\"", "9".repeat(64)),
        1,
    );
    let error = parse_compiler_candidate_execution_from_source(
        &source,
        Path::new("nuis.compiler-candidate-execution.toml"),
    )
    .expect_err("tampered execution identity must fail");
    assert!(error.to_string().contains("identity mismatch"));
}
