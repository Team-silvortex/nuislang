use super::{parse_args, CommandKind};
use std::path::PathBuf;

#[test]
fn parses_workflow_with_default_input() {
    let command = parse_args(["workflow".to_owned()].into_iter()).expect("workflow parses");
    assert_eq!(
        command,
        CommandKind::Workflow {
            input: PathBuf::from("."),
            json: false,
        }
    );
}

#[test]
fn parses_workflow_json_with_explicit_input() {
    let command = parse_args(
        [
            "workflow".to_owned(),
            "--json".to_owned(),
            "examples/demo.ns".to_owned(),
        ]
        .into_iter(),
    )
    .expect("workflow json parses");
    assert_eq!(
        command,
        CommandKind::Workflow {
            input: PathBuf::from("examples/demo.ns"),
            json: true,
        }
    );
}

#[test]
fn parses_project_imports_json_with_explicit_input() {
    let command = parse_args(
        [
            "project-imports".to_owned(),
            "--json".to_owned(),
            "examples/demo".to_owned(),
        ]
        .into_iter(),
    )
    .expect("project-imports parses");
    assert_eq!(
        command,
        CommandKind::ProjectImports {
            input: PathBuf::from("examples/demo"),
            json: true,
            apply_suggested: false,
        }
    );
}

#[test]
fn parses_project_imports_apply_suggested_with_default_input() {
    let command =
        parse_args(["project-imports".to_owned(), "--apply-suggested".to_owned()].into_iter())
            .expect("project-imports apply parses");
    assert_eq!(
        command,
        CommandKind::ProjectImports {
            input: PathBuf::from("."),
            json: false,
            apply_suggested: true,
        }
    );
}

#[test]
fn parses_inspect_artifact_json_with_manifest_input() {
    let command = parse_args(
        [
            "inspect-artifact".to_owned(),
            "--json".to_owned(),
            "target/demo/nuis.build.manifest.toml".to_owned(),
        ]
        .into_iter(),
    )
    .expect("inspect-artifact parses");
    assert_eq!(
        command,
        CommandKind::InspectArtifact {
            input: PathBuf::from("target/demo/nuis.build.manifest.toml"),
            json: true,
        }
    );
}

#[test]
fn parses_verify_artifact_with_compiled_artifact_input() {
    let command = parse_args(
        [
            "verify-artifact".to_owned(),
            "target/demo/nuis.compiled.artifact".to_owned(),
        ]
        .into_iter(),
    )
    .expect("verify-artifact parses");
    assert_eq!(
        command,
        CommandKind::VerifyArtifact {
            input: PathBuf::from("target/demo/nuis.compiled.artifact"),
            json: false,
        }
    );
}

#[test]
fn parses_run_artifact_with_manifest_input() {
    let command = parse_args(
        [
            "run-artifact".to_owned(),
            "target/demo/nuis.build.manifest.toml".to_owned(),
        ]
        .into_iter(),
    )
    .expect("run-artifact parses");
    assert_eq!(
        command,
        CommandKind::RunArtifact {
            input: PathBuf::from("target/demo/nuis.build.manifest.toml"),
            json: false,
        }
    );
}

#[test]
fn parses_run_artifact_json_with_manifest_input() {
    let command = parse_args(
        [
            "run-artifact".to_owned(),
            "--json".to_owned(),
            "target/demo/nuis.build.manifest.toml".to_owned(),
        ]
        .into_iter(),
    )
    .expect("run-artifact json parses");
    assert_eq!(
        command,
        CommandKind::RunArtifact {
            input: PathBuf::from("target/demo/nuis.build.manifest.toml"),
            json: true,
        }
    );
}

#[test]
fn parses_artifact_doctor_json_with_output_dir() {
    let command = parse_args(
        [
            "artifact-doctor".to_owned(),
            "--json".to_owned(),
            "target/demo".to_owned(),
        ]
        .into_iter(),
    )
    .expect("artifact-doctor parses");
    assert_eq!(
        command,
        CommandKind::ArtifactDoctor {
            input: PathBuf::from("target/demo"),
            json: true,
        }
    );
}

#[test]
fn parses_build_report_json_with_output_dir() {
    let command = parse_args(
        [
            "build-report".to_owned(),
            "--json".to_owned(),
            "target/demo".to_owned(),
        ]
        .into_iter(),
    )
    .expect("build-report parses");
    assert_eq!(
        command,
        CommandKind::BuildReport {
            input: PathBuf::from("target/demo"),
            json: true,
        }
    );
}

#[test]
fn parses_unpack_artifact_support_json_with_output_dir() {
    let command = parse_args(
        [
            "unpack-artifact-support".to_owned(),
            "--json".to_owned(),
            "target/demo/nuis.compiled.artifact".to_owned(),
            "target/unpacked".to_owned(),
        ]
        .into_iter(),
    )
    .expect("unpack-artifact-support parses");
    assert_eq!(
        command,
        CommandKind::UnpackArtifactSupport {
            input: PathBuf::from("target/demo/nuis.compiled.artifact"),
            output_dir: PathBuf::from("target/unpacked"),
            json: true,
        }
    );
}
#[test]
fn parses_materialize_artifact_with_output_dir() {
    let command = parse_args(
        [
            "materialize-artifact".to_owned(),
            "target/demo/nuis.build.manifest.toml".to_owned(),
            "target/materialized".to_owned(),
        ]
        .into_iter(),
    )
    .expect("materialize-artifact parses");
    assert_eq!(
        command,
        CommandKind::MaterializeArtifact {
            input: PathBuf::from("target/demo/nuis.build.manifest.toml"),
            output_dir: PathBuf::from("target/materialized"),
            json: false,
        }
    );
}
#[test]
fn parses_bench_with_default_input() {
    let command = parse_args(["bench".to_owned()].into_iter()).expect("bench parses");
    assert_eq!(
        command,
        CommandKind::Bench {
            input: PathBuf::from("."),
            list: false,
            json: false,
            exact: false,
            filter: None,
        }
    );
}

#[test]
fn parses_bench_with_list_exact_and_filter() {
    let command = parse_args(
        [
            "bench".to_owned(),
            "--list".to_owned(),
            "--json".to_owned(),
            "--exact".to_owned(),
            "examples/demo.ns".to_owned(),
            "sum_loop".to_owned(),
        ]
        .into_iter(),
    )
    .expect("bench with filter parses");
    assert_eq!(
        command,
        CommandKind::Bench {
            input: PathBuf::from("examples/demo.ns"),
            list: true,
            json: true,
            exact: true,
            filter: Some("sum_loop".to_owned()),
        }
    );
}
