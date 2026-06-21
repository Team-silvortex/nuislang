use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nuisc_{label}_{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_nuisc(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nuisc"))
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to run nuisc {:?}: {error}", args))
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

#[test]
fn cli_artifact_commands_report_benchmark_tooling_outputs() {
    let project = Path::new(
        "/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/benchmark_report_file_demo",
    );
    let output_dir = temp_dir("artifact_cli_benchmark_report_file_outputs");
    let output_dir_text = output_dir.display().to_string();

    let compile = run_nuisc(&["compile", &project.display().to_string(), &output_dir_text]);
    assert_success(&compile, "nuisc compile");

    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let artifact_path = output_dir.join("nuis.compiled.artifact");
    assert!(
        manifest_path.exists(),
        "expected {}",
        manifest_path.display()
    );
    assert!(
        artifact_path.exists(),
        "expected {}",
        artifact_path.display()
    );

    let inspect = run_nuisc(&[
        "inspect-artifact",
        "--json",
        &manifest_path.display().to_string(),
    ]);
    assert_success(&inspect, "nuisc inspect-artifact --json");
    let inspect_stdout = String::from_utf8_lossy(&inspect.stdout);
    assert!(inspect_stdout.contains("\"kind\":\"nuis_artifact_inspect\""));
    assert!(inspect_stdout.contains("\"binary_name\":\"benchmark_report_file_demo\""));
    assert!(inspect_stdout.contains("\"domain_build_units\":["));
    assert!(inspect_stdout.contains("\"link_plan\":{"));

    let report = run_nuisc(&[
        "artifact-report",
        "--json",
        &manifest_path.display().to_string(),
    ]);
    assert_success(&report, "nuisc artifact-report --json");
    let report_stdout = String::from_utf8_lossy(&report.stdout);
    assert!(report_stdout.contains("\"kind\":\"nuis_artifact_report\""));
    assert!(report_stdout.contains("\"artifact_inspect\":{"));
    assert!(report_stdout.contains("\"artifact_verify\":{"));
    assert!(report_stdout.contains("\"manifest_verify\":{"));
    assert!(report_stdout.contains("\"all_units_consistent\":true"));

    let verify_manifest = run_nuisc(&[
        "verify-build-manifest",
        "--json",
        &manifest_path.display().to_string(),
    ]);
    assert_success(&verify_manifest, "nuisc verify-build-manifest --json");
    let verify_manifest_stdout = String::from_utf8_lossy(&verify_manifest.stdout);
    assert!(verify_manifest_stdout.contains("\"kind\":\"nuis_build_manifest_verify\""));
    assert!(
        verify_manifest_stdout.contains("\"artifact_binary_name\":\"benchmark_report_file_demo\"")
    );
    assert!(verify_manifest_stdout.contains("\"domain_build_verification_summary\":{"));

    let verify_artifact = run_nuisc(&[
        "verify-artifact",
        "--json",
        &artifact_path.display().to_string(),
    ]);
    assert_success(&verify_artifact, "nuisc verify-artifact --json");
    let verify_artifact_stdout = String::from_utf8_lossy(&verify_artifact.stdout);
    assert!(verify_artifact_stdout.contains("\"kind\":\"nuis_artifact_verify\""));
    assert!(verify_artifact_stdout.contains("\"binary_name\":\"benchmark_report_file_demo\""));
    assert!(verify_artifact_stdout.contains("\"artifact_roundtrip_verified\":true"));
}
