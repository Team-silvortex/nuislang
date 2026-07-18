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

fn run_nsdb(args: &[&str]) -> std::process::Output {
    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_nsdb").map(PathBuf::from) {
        return Command::new(path)
            .args(args)
            .output()
            .unwrap_or_else(|error| panic!("failed to run nsdb {:?}: {error}", args));
    }
    let fallback = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/nsdb");
    if fallback.exists() {
        return Command::new(fallback)
            .args(args)
            .output()
            .unwrap_or_else(|error| panic!("failed to run nsdb {:?}: {error}", args));
    }
    Command::new("cargo")
        .args(["run", "-q", "-p", "nsdb", "--"])
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to run nsdb through cargo {:?}: {error}", args))
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

fn workflow_default_output_dir(input: &Path) -> PathBuf {
    let label = input
        .file_stem()
        .or_else(|| input.file_name())
        .and_then(|item| item.to_str())
        .unwrap_or("input");
    PathBuf::from(format!(
        "target/nuis-build/{}",
        sanitize_workflow_label(label)
    ))
}

fn sanitize_workflow_label(label: &str) -> String {
    let mut out = String::new();
    let mut previous_was_sep = false;
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            previous_was_sep = false;
        } else if !previous_was_sep {
            out.push('-');
            previous_was_sep = true;
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "input".to_owned()
    } else {
        trimmed.to_owned()
    }
}

#[test]
fn self_contained_nsb_route_moves_from_nsld_drive_to_run_artifact_handoff() {
    let project_dir = temp_dir("self_contained_nsb_project");
    let output_dir = std::env::current_dir()
        .expect("current dir")
        .join(workflow_default_output_dir(&project_dir));
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).unwrap_or_else(|error| {
            panic!(
                "failed to clear workflow default output `{}`: {error}",
                output_dir.display()
            )
        });
    }
    fs::create_dir_all(&output_dir).unwrap();
    let source_path = project_dir.join("main.ns");
    fs::write(
        &source_path,
        "mod cpu Main { fn main() -> i64 { return 0; } }\n",
    )
    .unwrap();
    fs::write(
        project_dir.join("nuis.toml"),
        "name = \"self_contained_nsb_smoke\"\nversion = \"0.1.0\"\nentry = \"main.ns\"\nmodules = [\"main.ns\"]\n",
    )
    .unwrap();

    let build = run_nuis(&[
        "build",
        "--packaging-mode",
        "nuis-self-contained-image",
        &project_dir.display().to_string(),
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
            && drive_stdout.contains("\"last_command_id\":\"emit-final-executable-pipeline\"")
            && drive_stdout.contains("\"safe_next_contract\":\"nsld-drive-safe-next-v1\"")
            && drive_stdout.contains("\"safe_next_action\":\"clean\"")
            && drive_stdout.contains("\"safe_next_command\":null")
            && drive_stdout.contains("\"safe_next_gate_required\":false")
            && drive_stdout.contains("\"safe_next_reason\":\"drive reached a clean artifact chain\""),
        "nsld drive should materialize the self-contained pipeline cleanly with a safe-next gate\n{drive_stdout}"
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
            && run_json_stdout.contains("\"host_runner_container_loader_entry_kind\":\"lifecycle-bootstrap\"")
            && run_json_stdout.contains("\"host_runner_container_loader_entry_symbol\":\"nuis.bootstrap.lifecycle.v1\"")
            && run_json_stdout.contains("\"host_runner_container_loader_entry_section_id\":\"sec0000.compiled-artifact\"")
            && run_json_stdout.contains("\"host_runner_container_loader_handoff_ready\":true")
            && run_json_stdout.contains("\"host_runner_container_loader_handoff_status\":\"ready\"")
            && run_json_stdout.contains("\"host_runner_backend_artifact_payload_count\":0")
            && run_json_stdout.contains("\"host_runner_backend_artifact_payload_parsed_count\":0")
            && run_json_stdout.contains("\"host_runner_backend_artifact_payload_ready_count\":0")
            && run_json_stdout.contains("\"host_runner_backend_artifact_payload_first_kind\":null")
            && run_json_stdout.contains("\"launch_evidence_protocol\":\"nuis-run-artifact-launch-evidence-v1\"")
            && run_json_stdout.contains("\"launch_evidence_status\":\"ready\"")
            && run_json_stdout.contains("\"launch_evidence_route\":\"nsld-host-entrypoint\"")
            && run_json_stdout.contains("\"launch_evidence_debugger_contract\":\"nsdb-yir-launch-evidence-v1\"")
            && run_json_stdout.contains("\"launch_evidence_host_runner_probe_status\":\"ready\"")
            && run_json_stdout.contains("\"launch_evidence_host_runner_probe_ready\":true")
            && run_json_stdout.contains("\"launch_evidence_first_payload_status\":\"ready\"")
            && run_json_stdout.contains("\"launch_evidence_first_payload_ready\":true")
            && run_json_stdout.contains("\"launch_evidence_first_payload_target\":\"container-loader\"")
            && run_json_stdout.contains("\"launch_evidence_first_payload_entry_symbol\":\"nuis.bootstrap.lifecycle.v1\"")
            && run_json_stdout.contains("\"launch_evidence_first_payload_entry_kind\":\"lifecycle-bootstrap\"")
            && run_json_stdout.contains("\"launch_evidence_first_payload_entry_section_id\":\"sec0000.compiled-artifact\"")
            && run_json_stdout.contains("\"launch_evidence_first_payload_first_blocker\":null")
            && run_json_stdout.contains("\"launch_evidence_payload_execution_trace_protocol\":\"nsdb-yir-payload-execution-trace-v1\"")
            && run_json_stdout.contains("\"launch_evidence_payload_execution_trace_available\":true")
            && run_json_stdout.contains("\"launch_evidence_payload_execution_trace_record_count\":1")
            && run_json_stdout.contains("\"launch_evidence_payload_execution_trace_ready_record_count\":1")
            && run_json_stdout.contains("\"launch_evidence_payload_execution_trace_records\":[{")
            && run_json_stdout.contains("\"trace_id\":\"payload-trace:container-loader:nuis.bootstrap.lifecycle.v1\"")
            && run_json_stdout.contains("\"execution_phase\":\"container-loader-handoff\"")
            && run_json_stdout.contains("\"next_action\":\"handoff-payload-trace-to-nsdb\"")
            && run_json_stdout.contains("\"launch_evidence_nsdb_handoff_protocol\":\"nuis-nsdb-payload-execution-handoff-v1\"")
            && run_json_stdout.contains("\"launch_evidence_nsdb_handoff_persisted\":true")
            && run_json_stdout.contains("\"launch_evidence_nsdb_handoff_path\":")
            && run_json_stdout.contains("\"launch_evidence_nsdb_handoff_record_count\":1")
            && run_json_stdout.contains("\"launch_evidence_nsdb_handoff_ready_record_count\":1")
            && run_json_stdout.contains("\"launch_evidence_nsdb_handoff_first_trace_id\":\"payload-trace:container-loader:nuis.bootstrap.lifecycle.v1\"")
            && run_json_stdout.contains("\"launch_evidence_first_blocker\":null"),
        "run-artifact json should surface the host runner image and container-loader evidence for the self-contained handoff\n{run_json_stdout}"
    );
    let nsdb_handoff_path = output_dir.join("nuis.nsdb.payload-execution-handoff.toml");
    let nsdb_handoff = fs::read_to_string(&nsdb_handoff_path).unwrap_or_else(|error| {
        panic!(
            "missing nsdb payload execution handoff metadata `{}`: {error}",
            nsdb_handoff_path.display()
        )
    });
    assert!(
        nsdb_handoff.contains("protocol = \"nuis-nsdb-payload-execution-handoff-v1\"")
            && nsdb_handoff.contains("debugger_contract = \"nsdb-yir-payload-execution-trace-v1\"")
            && nsdb_handoff.contains("source = \"run-artifact-launch-evidence\"")
            && nsdb_handoff.contains("record_count = 1")
            && nsdb_handoff.contains("ready_record_count = 1")
            && nsdb_handoff.contains(
                "trace_id = \"payload-trace:container-loader:nuis.bootstrap.lifecycle.v1\""
            )
            && nsdb_handoff.contains("status = \"ready\"")
            && nsdb_handoff.contains("execution_phase = \"container-loader-handoff\"")
            && nsdb_handoff.contains("target = \"container-loader\"")
            && nsdb_handoff.contains("entry_symbol = \"nuis.bootstrap.lifecycle.v1\"")
            && nsdb_handoff.contains("entry_kind = \"lifecycle-bootstrap\"")
            && nsdb_handoff.contains("entry_section_id = \"sec0000.compiled-artifact\"")
            && nsdb_handoff.contains("next_action = \"handoff-payload-trace-to-nsdb\""),
        "run-artifact should persist nsdb payload execution handoff metadata\n{nsdb_handoff}"
    );

    let workflow_json = run_nuis(&["workflow", "--json", &project_dir.display().to_string()]);
    assert_success(
        &workflow_json,
        "nuis workflow json after self-contained nsld handoff",
    );
    let workflow_stdout = String::from_utf8_lossy(&workflow_json.stdout);
    assert!(
        workflow_stdout.contains("\"closure_summary_source\":\"workflow-link-plan\"")
            && workflow_stdout.contains("\"closure_summary_status\":\"ready\"")
            && workflow_stdout.contains("\"closure_summary_ready\":true")
            && workflow_stdout.contains("\"closure_summary_primary_blocker\":null")
            && workflow_stdout.contains("\"closure_summary_next_action\":\"run-artifact-or-replay-nsdb\"")
            && workflow_stdout.contains("\"closure_summary_next_command\":\"nsdb replay ")
            && workflow_stdout
                .contains("\"nsld_final_executable_output_nsdb_replay_ready\":true")
            && workflow_stdout.contains(
                "\"nsld_final_executable_output_nsdb_replay_status\":\"replay-evidence-ready\""
            )
            && workflow_stdout.contains(
                "\"nsld_final_executable_output_nsdb_replay_contract\":\"nsdb-payload-execution-replay-plan-v1\""
            )
            && workflow_stdout
                .contains("\"nsld_final_executable_output_nsdb_replay_command\":\"nsdb replay ")
            && workflow_stdout
                .contains("\"nsld_final_executable_output_nsdb_replay_checkpoint_count\":1")
            && workflow_stdout.contains(
                "\"nsld_final_executable_output_nsdb_replayable_checkpoint_count\":1"
            )
            && workflow_stdout.contains(
                "\"nsld_final_executable_output_nsdb_replay_next_action\":\"replay-nsdb-payload-execution\""
            )
            && workflow_stdout.contains(
                "\"nsld_final_executable_output_nsdb_replay_next_command\":\"nsdb replay "
            )
            && workflow_stdout
                .contains("\"nsld_final_executable_output_nsdb_replay_first_blocker\":null"),
        "workflow json should promote replay-ready self-contained final output into closure_summary ready\n{workflow_stdout}"
    );

    let replay = run_nsdb(&["replay", &output_dir.display().to_string(), "--json"]);
    assert_success(&replay, "nsdb replay self-contained nsb handoff");
    let replay_stdout = String::from_utf8_lossy(&replay.stdout);
    assert!(
        replay_stdout.contains("\"kind\":\"nsdb_yir_replay_transcript\"")
            && replay_stdout
                .contains("\"debugger_transcript_contract\":\"nsdb-yir-replay-transcript-v1\"")
            && replay_stdout.contains("\"debugger_transcript_status\":\"transcript-consumed\"")
            && replay_stdout.contains("\"debugger_transcript_ready\":true")
            && replay_stdout.contains("\"debugger_transcript_checkpoint_count\":1")
            && replay_stdout.contains("\"debugger_transcript_replayed_checkpoint_count\":1")
            && replay_stdout.contains("\"consumed\":true"),
        "nsdb replay should consume the replay-ready YIR checkpoint set\n{replay_stdout}"
    );

    let build_report_json =
        run_nuis(&["build-report", "--json", &output_dir.display().to_string()]);
    assert_success(
        &build_report_json,
        "nuis build-report json after self-contained nsld handoff",
    );
    let build_report_stdout = String::from_utf8_lossy(&build_report_json.stdout);
    assert!(
        build_report_stdout.contains("\"kind\":\"build_report\"")
            && build_report_stdout
                .contains("\"nsld_final_executable_output_nsdb_replay_ready\":true")
            && build_report_stdout.contains(
                "\"nsld_final_executable_output_nsdb_replay_status\":\"replay-evidence-ready\""
            )
            && build_report_stdout
                .contains("\"nsld_final_executable_output_nsdb_replay_checkpoint_count\":1")
            && build_report_stdout.contains(
                "\"nsld_final_executable_output_nsdb_replayable_checkpoint_count\":1"
            )
            && build_report_stdout.contains(
                "\"nsld_final_executable_output_nsdb_replay_next_action\":\"replay-nsdb-payload-execution\""
            ),
        "build-report json should mirror replay-ready self-contained closure evidence\n{build_report_stdout}"
    );

    let run = run_nuis(&["run-artifact", &output_dir.display().to_string()]);
    assert_success(&run, "nuis run-artifact self-contained nsld handoff");
    let run_stdout = String::from_utf8_lossy(&run.stdout);
    assert!(
        run_stdout.contains("host_runner_program:") && run_stdout.contains("host_runner_status: 0"),
        "run-artifact should invoke nuis-host-runner for the self-contained handoff\n{run_stdout}"
    );
    assert!(
        run_stdout.contains("launch_evidence_status: ready")
            && run_stdout
                .contains("launch_evidence_debugger_contract: nsdb-yir-launch-evidence-v1")
            && run_stdout.contains("launch_evidence_first_payload_status: ready")
            && run_stdout.contains(
                "launch_evidence_first_payload_entry_symbol: nuis.bootstrap.lifecycle.v1"
            )
            && run_stdout.contains(
                "launch_evidence_first_payload_entry_section_id: sec0000.compiled-artifact"
            )
            && run_stdout.contains(
                "launch_evidence_payload_execution_trace_protocol: nsdb-yir-payload-execution-trace-v1"
            )
            && run_stdout.contains("launch_evidence_payload_execution_trace_available: true")
            && run_stdout.contains("launch_evidence_payload_execution_trace_record_count: 1")
            && run_stdout.contains(
                "launch_evidence_payload_execution_trace_record: payload-trace:container-loader:nuis.bootstrap.lifecycle.v1 container-loader-handoff ready"
            )
            && run_stdout.contains(
                "launch_evidence_nsdb_handoff_protocol: nuis-nsdb-payload-execution-handoff-v1"
            )
            && run_stdout.contains("launch_evidence_nsdb_handoff_persisted: true")
            && run_stdout.contains("launch_evidence_nsdb_handoff_record_count: 1")
            && run_stdout.contains("launch_evidence_nsdb_handoff_ready_record_count: 1")
            && run_stdout.contains(
                "launch_evidence_nsdb_handoff_first_trace_id: payload-trace:container-loader:nuis.bootstrap.lifecycle.v1"
            )
            && run_stdout.contains("launch_evidence_first_blocker: <none>"),
        "run-artifact text should expose the launch evidence contract\n{run_stdout}"
    );
}
