use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use nuis_artifact::{
    parse_compiler_structural_projection, read_compiler_candidate_execution,
    read_compiler_component_build, read_compiler_stage_handoff, CompilerProjectionKind,
    CompilerProjectionRecordKind, CompilerStageKind, COMPILER_CANDIDATE_EXECUTION_AUTHORITY,
    COMPILER_CANDIDATE_EXECUTION_FILE, COMPILER_CANDIDATE_EXECUTION_ROLE,
    COMPILER_COMPONENT_BUILD_FILE,
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
