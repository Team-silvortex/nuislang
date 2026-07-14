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
    let dir = std::env::temp_dir().join(format!("nuis_{label}_{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_nuis(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nuis"))
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to run nuis {:?}: {error}", args))
}

fn run_nsld(args: &[&str]) -> std::process::Output {
    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_nsld").map(PathBuf::from) {
        return Command::new(path)
            .args(args)
            .output()
            .unwrap_or_else(|error| panic!("failed to run nsld {:?}: {error}", args));
    }
    let fallback = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/nsld");
    if fallback.exists() {
        return Command::new(fallback)
            .args(args)
            .output()
            .unwrap_or_else(|error| panic!("failed to run nsld {:?}: {error}", args));
    }
    Command::new("cargo")
        .args(["run", "-q", "-p", "nsld", "--"])
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to run nsld through cargo {:?}: {error}", args))
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

fn ensure_host_runner_available() {
    let output = Command::new("cargo")
        .args(["build", "-q", "-p", "nuis-host-runner"])
        .output()
        .expect("failed to build nuis-host-runner");
    assert_success(&output, "cargo build nuis-host-runner");
}

#[test]
fn self_contained_nsb_route_moves_from_nsld_drive_to_run_artifact_handoff() {
    let project_dir = temp_dir("self_contained_nsb_project");
    let output_dir = temp_dir("self_contained_nsb_outputs");
    let source_path = project_dir.join("main.ns");
    fs::write(
        &source_path,
        "mod cpu Main { fn main() -> i64 { return 0; } }\n",
    )
    .unwrap();

    let build = run_nuis(&[
        "build",
        "--packaging-mode",
        "nuis-self-contained-image",
        &source_path.display().to_string(),
        &output_dir.display().to_string(),
    ]);
    assert_success(&build, "nuis build self-contained nsb smoke");

    let before_doctor = run_nuis(&[
        "artifact-doctor",
        "--json",
        &output_dir.display().to_string(),
    ]);
    assert_success(
        &before_doctor,
        "nuis artifact-doctor before nsld drive self-contained nsb smoke",
    );
    let before_stdout = String::from_utf8_lossy(&before_doctor.stdout);
    assert!(
        before_stdout.contains("\"ready_to_run\":false"),
        "self-contained route should not be runnable before nsld handoff\n{before_stdout}"
    );
    assert!(
        before_stdout.contains("\"recommended_next_step\":\"nsld_drive\""),
        "self-contained route should recommend nsld drive before handoff\n{before_stdout}"
    );
    assert!(
        before_stdout.contains(
            "\"artifact_closure_evidence_status\":\"self-contained-image-awaiting-nsld-handoff\""
        ),
        "self-contained route should block legacy host-binary fallback before handoff\n{before_stdout}"
    );

    let drive = run_nsld(&[
        "drive",
        &output_dir.display().to_string(),
        "--apply",
        "--until-clean",
        "--json",
    ]);
    assert_success(&drive, "nsld drive self-contained nsb smoke");
    let drive_stdout = String::from_utf8_lossy(&drive.stdout);
    assert!(
        drive_stdout.contains("\"completed\":true")
            && drive_stdout.contains("\"stop_reason\":\"clean\"")
            && drive_stdout.contains("\"last_command_id\":\"emit-final-executable-pipeline\""),
        "nsld drive should materialize the self-contained pipeline cleanly\n{drive_stdout}"
    );

    let after_doctor = run_nuis(&[
        "artifact-doctor",
        "--json",
        &output_dir.display().to_string(),
    ]);
    assert_success(
        &after_doctor,
        "nuis artifact-doctor after nsld drive self-contained nsb smoke",
    );
    let after_stdout = String::from_utf8_lossy(&after_doctor.stdout);
    assert!(
        after_stdout.contains("\"ready_to_run\":true")
            && after_stdout.contains("\"recommended_next_step\":\"run_artifact\""),
        "self-contained route should become runnable after nsld handoff\n{after_stdout}"
    );
    assert!(
        after_stdout.contains("\"artifact_closure_kind\":\"nsld-host-entrypoint\"")
            && after_stdout.contains("\"artifact_closure_status\":\"ready\"")
            && after_stdout.contains("\"artifact_closure_evidence_status\":\"entrypoint-ready\""),
        "artifact-doctor should prefer the nsld entrypoint handoff after drive\n{after_stdout}"
    );
    assert!(
        after_stdout.contains("\"nsld_final_executable_output_ready\":true")
            && after_stdout
                .contains("\"nsld_final_executable_output_nsld_owned\":true")
            && after_stdout.contains(
                "\"nsld_final_executable_output_materialization_status\":\"self-contained-image-ready\""
            )
            && after_stdout.contains(".nsb"),
        "artifact-doctor should report a ready Nsld-owned .nsb output\n{after_stdout}"
    );

    ensure_host_runner_available();

    let run_json = run_nuis(&["run-artifact", &output_dir.display().to_string(), "--json"]);
    assert_success(
        &run_json,
        "nuis run-artifact json after self-contained nsld drive",
    );
    let run_json_stdout = String::from_utf8_lossy(&run_json.stdout);
    assert!(
        run_json_stdout.contains("\"binary_resolved\":false")
            && run_json_stdout.contains("\"run_artifact_prelaunch_kind\":\"nsld-host-entrypoint\"")
            && run_json_stdout.contains("\"run_artifact_prelaunch_status\":\"ready\""),
        "run-artifact should use the nsld entrypoint handoff, not legacy host-binary fallback\n{run_json_stdout}"
    );
    assert!(
        run_json_stdout.contains("\"host_runner_invoked\":true")
            && run_json_stdout.contains("\"host_runner_status\":\"ready\"")
            && run_json_stdout.contains("\"host_runner_ready\":true")
            && run_json_stdout.contains("\"host_runner_nsb_readable\":true")
            && run_json_stdout.contains("\"host_runner_nsb_hash_matches\":true")
            && run_json_stdout.contains("\"host_runner_nsb_payload_region_mapped\":true")
            && run_json_stdout.contains("\"host_runner_nsb_payload_scan_kind\":\"nsld-container-toml\"")
            && run_json_stdout.contains("\"host_runner_container_loader_status\":\"parsed\"")
            && run_json_stdout.contains("\"host_runner_container_loader_handoff_ready\":true")
            && run_json_stdout.contains("\"host_runner_container_loader_handoff_status\":\"ready\""),
        "run-artifact json should surface the host runner image and container-loader evidence for the self-contained handoff\n{run_json_stdout}"
    );

    let run = run_nuis(&["run-artifact", &output_dir.display().to_string()]);
    assert_success(&run, "nuis run-artifact self-contained nsld handoff");
    let run_stdout = String::from_utf8_lossy(&run.stdout);
    assert!(
        run_stdout.contains("host_runner_program:") && run_stdout.contains("host_runner_status: 0"),
        "run-artifact should invoke nuis-host-runner for the self-contained handoff\n{run_stdout}"
    );
}
