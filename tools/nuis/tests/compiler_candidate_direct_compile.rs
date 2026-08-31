use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use nuis_artifact::{
    parse_compiler_candidate_direct_compile_capability, parse_compiler_candidate_frontend_result,
    COMPILER_CANDIDATE_DIRECT_COMPILE_VERDICT, COMPILER_CANDIDATE_FRONTEND_RESULT_PROTOCOL,
};

fn temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should follow the Unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nuis_candidate_direct_compile_{nonce}"));
    fs::create_dir_all(&dir).expect("create candidate direct compile output");
    dir
}

#[test]
fn candidate_directly_compiles_canonical_frontend_result_without_provider() {
    let project = "../../examples/projects/tooling/bootstrap_structural_projection_candidate";
    let output_dir = temp_dir();
    let candidate_root = output_dir.join("candidate-root");
    let build = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-candidate-build")
        .arg(project)
        .arg(&candidate_root)
        .output()
        .expect("build stage1 candidate");
    assert_success(&build, "stage1 candidate build");

    let result_path = output_dir.join("front-end-result");
    let capability_path = output_dir.join("direct-compile-capability.toml");
    let direct = Command::new(env!("CARGO_BIN_EXE_nuis"))
        .arg("bootstrap-candidate-direct-compile")
        .arg(&candidate_root)
        .arg(&result_path)
        .arg(&capability_path)
        .env(
            "NUIS_BOOTSTRAP_STAGE0_PROVIDER_V1",
            "/provider-must-not-be-observed",
        )
        .output()
        .expect("execute direct stage1 compile");
    assert_success(&direct, "direct stage1 front-end compile");

    let result = parse_compiler_candidate_frontend_result(&result_path)
        .expect("verify canonical front-end result");
    assert_eq!(result.protocol, COMPILER_CANDIDATE_FRONTEND_RESULT_PROTOCOL);
    assert_eq!(result.stage_folds.len(), 5);
    assert!(result.bundle_fold > 0);
    let capability = parse_compiler_candidate_direct_compile_capability(&capability_path)
        .expect("verify direct compile capability");
    assert_eq!(
        capability.verdict,
        COMPILER_CANDIDATE_DIRECT_COMPILE_VERDICT
    );
    assert!(!capability.provider_dependency_required);
    assert!(capability.direct_stage1_compile);
    assert!(!capability.native_materialization);
    assert!(!capability.replacement_authorized);
    assert!(!capability.selection_authorized);
    assert_eq!(capability.result_bundle_fold, result.bundle_fold);
    assert_eq!(
        capability.result_bytes,
        fs::read(&result_path).unwrap().len()
    );
    let root_text = output_dir.display().to_string();
    assert!(!fs::read_to_string(&result_path)
        .unwrap()
        .contains(&root_text));
    assert!(!fs::read_to_string(&capability_path)
        .unwrap()
        .contains(&root_text));

    fs::remove_dir_all(output_dir).expect("remove candidate direct compile output");
}

fn assert_success(output: &std::process::Output, label: &str) {
    assert!(
        output.status.success(),
        "{label} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
