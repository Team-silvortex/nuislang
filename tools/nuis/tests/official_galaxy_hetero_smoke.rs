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
    let dir = std::env::temp_dir().join(format!("nuis_official_hetero_{label}_{nonce}"));
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
    Command::new("cargo")
        .args(["run", "-q", "-p", "nsdb", "--"])
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to run nsdb through cargo {:?}: {error}", args))
}

fn json_string_values(source: &str, key: &str) -> Vec<String> {
    let needle = format!("\"{key}\":\"");
    source
        .split(&needle)
        .skip(1)
        .filter_map(|tail| tail.split('"').next())
        .map(str::to_owned)
        .collect()
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

fn assert_file_contains(path: &Path, needle: &str, context: &str) {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    assert!(
        source.contains(needle),
        "expected {context} file {} to contain `{needle}`\n{source}",
        path.display()
    );
}

fn provider_family_artifact_component(provider_family: &str) -> String {
    provider_family
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}

fn assert_official_galaxy_hetero_build(
    label: &str,
    project: &str,
    domain: &str,
    backend_family: &str,
    target_device: &str,
    trace_record_count: usize,
    expected_trace_id: &str,
    yir_needles: &[&str],
    sidecar_needles: &[&str],
    payload_needles: &[&str],
) {
    let output_dir = temp_dir(label);
    let output_dir_text = output_dir.display().to_string();

    let build = run_nuis(&["build", project, &output_dir_text]);
    assert_success(&build, "nuis build official galaxy hetero smoke");

    let doctor_before_drive = run_nuis(&["artifact-doctor", "--json", &output_dir_text]);
    assert_success(
        &doctor_before_drive,
        "nuis artifact-doctor json before official galaxy nsld drive",
    );
    let doctor_before_drive_stdout = String::from_utf8_lossy(&doctor_before_drive.stdout);
    assert!(
        doctor_before_drive_stdout.contains(
            "\"nsld_final_executable_output_nsdb_replay_contract\":\"nsdb-payload-execution-replay-plan-v1\""
        )
            && doctor_before_drive_stdout
                .contains("\"nsld_final_executable_output_nsdb_replay_ready\":false")
            && doctor_before_drive_stdout
                .contains("\"nsld_final_executable_output_nsdb_replay_status\":\"blocked\"")
            && doctor_before_drive_stdout.contains(
                "\"nsld_final_executable_output_nsdb_replay_next_action\":\"resolve-final-output-nsdb-replay\""
            )
            && doctor_before_drive_stdout.contains(
                "\"nsld_final_executable_output_nsdb_replay_next_command\":\"nsld final-executable-output "
            )
            && doctor_before_drive_stdout.contains(
                "\"nsld_final_executable_output_nsdb_replay_first_blocker\":\"payload-execution-handoff-missing\""
            ),
        "official galaxy hetero artifact-doctor did not expose final-output replay blocked gate for {label}\n{doctor_before_drive_stdout}"
    );

    let drive_apply = run_nsld(&["drive", &output_dir_text, "--apply", "--json"]);
    assert_success(
        &drive_apply,
        "nsld drive apply official galaxy hetero smoke",
    );
    let drive_apply_stdout = String::from_utf8_lossy(&drive_apply.stdout);
    assert!(
        drive_apply_stdout.contains("\"kind\":\"nsld_drive_apply\"")
            && drive_apply_stdout.contains("\"applied\":true")
            && drive_apply_stdout.contains("\"mutates_artifacts\":true")
            && drive_apply_stdout.contains("\"mutation_policy\":\"whitelisted-artifact-mutation\"")
            && drive_apply_stdout.contains("\"command_id\":\"emit-inputs\"")
            && drive_apply_stdout.contains("\"safe_next_contract\":\"nsld-drive-safe-next-v1\"")
            && drive_apply_stdout
                .contains("\"safe_next_action\":\"rerun-drive-to-refresh-next-action\"")
            && drive_apply_stdout.contains("\"safe_next_command\":null")
            && drive_apply_stdout.contains("\"safe_next_gate_required\":false")
            && drive_apply_stdout.contains(
                "\"safe_next_reason\":\"drive applied one mutation; rerun drive to observe the next deterministic action\""
            ),
        "official galaxy hetero nsld drive did not expose safe-next mutation guidance for {label}\n{drive_apply_stdout}"
    );

    let yir_path = output_dir.join(format!("{label}.yir"));
    for needle in yir_needles {
        assert_file_contains(&yir_path, needle, "official galaxy hetero YIR");
    }

    assert_file_contains(
        &output_dir.join(format!("nuis.domain.{domain}.artifact.toml")),
        "schema = \"nuis-domain-build-unit-v1\"",
        "official galaxy hetero artifact",
    );
    assert_file_contains(
        &output_dir.join(format!("nuis.domain.{domain}.payload.toml")),
        "schema = \"nuis-domain-build-payload-v1\"",
        "official galaxy hetero payload",
    );
    assert_file_contains(
        &output_dir.join("nuis.hetero-calculate.plan.toml"),
        "schema = \"nuis-hetero-calculate-link-plan-v1\"",
        "official galaxy hetero plan",
    );

    let sidecar_path = output_dir.join(format!("nuis.domain.{domain}.lowering.ir.txt"));
    for needle in sidecar_needles {
        assert_file_contains(&sidecar_path, needle, "official galaxy hetero sidecar");
    }
    let payload_path = output_dir.join(format!("nuis.domain.{domain}.payload.toml"));
    for needle in payload_needles {
        assert_file_contains(&payload_path, needle, "official galaxy hetero payload");
    }
    assert_file_contains(
        &output_dir.join("nuis.hetero-calculate.plan.toml"),
        &format!("domain_family = \"{domain}\""),
        "official galaxy hetero plan",
    );

    let run_json = run_nuis(&["run-artifact", &output_dir_text, "--json"]);
    assert_success(
        &run_json,
        "nuis run-artifact json official galaxy hetero smoke",
    );
    let run_json_stdout = String::from_utf8_lossy(&run_json.stdout);
    let trace_id = format!("\"trace_id\":\"{expected_trace_id}\"");
    let expects_std_pgm_marker = backend_family == "metal" && target_device == "apple-silicon-gpu";
    let provider_record_count = if label == "pixelmagic_pipeline_demo" {
        2
    } else {
        1
    };
    assert!(
        run_json_stdout.contains("\"hetero_runtime_trace_available\":true")
            && run_json_stdout.contains("\"hetero_runtime_trace_status\":\"execution-pending\"")
            && run_json_stdout.contains("\"hetero_runtime_trace_debugger_contract\":\"nsdb-yir-hetero-runtime-trace-v1\"")
            && run_json_stdout.contains(&format!("\"hetero_runtime_trace_record_count\":{trace_record_count}"))
            && run_json_stdout.contains(&format!("\"hetero_runtime_trace_backend_execution_record_count\":{provider_record_count}"))
            && run_json_stdout.contains(&format!("\"hetero_runtime_trace_device_sample_descriptor_count\":{provider_record_count}"))
            && run_json_stdout.contains(&format!("\"hetero_runtime_trace_device_sample_pending_count\":{provider_record_count}"))
            && run_json_stdout.contains(&format!("\"hetero_runtime_trace_device_sample_pending_validation_count\":{provider_record_count}"))
            && run_json_stdout.contains("\"hetero_runtime_trace_device_sample_providers\":[\"nustar-deferred-device-sample-v1\"]")
            && run_json_stdout.contains(&format!("{backend_family}:{target_device}"))
            && run_json_stdout.contains(&format!("\"{backend_family}\""))
            && run_json_stdout.contains(&format!("\"{target_device}\""))
            && run_json_stdout.contains(&trace_id)
            && run_json_stdout.contains("\"trace_role\":\"backend-artifact\"")
            && run_json_stdout
                .contains("\"device_sample_provider\":\"nustar-deferred-device-sample-v1\"")
            && run_json_stdout.contains(&format!(
                "\"device_sample_provider_family\":\"{backend_family}:{target_device}\""
            ))
            && run_json_stdout
                .contains("\"device_sample_kind\":\"deferred-provider-sample-descriptor\"")
            && run_json_stdout.contains("\"device_sample_status\":\"device-execution-pending\"")
            && run_json_stdout
                .contains("\"device_sample_schema\":\"nsdb-yir-device-execution-sample-v1\"")
            && run_json_stdout.contains("\"device_sample_output_evidence\":\"not-materialized\"")
            && run_json_stdout
                .contains("\"device_sample_validation_status\":\"pending-provider-execution\"")
            && run_json_stdout.contains(
                "\"device_sample_next_action\":\"materialize-device-execution-sample\""
            )
            && run_json_stdout.contains("\"next_action\":\"materialize-device-execution-trace\""),
        "run-artifact json did not expose expected official galaxy hetero trace for {label}\n{run_json_stdout}"
    );
    if expects_std_pgm_marker {
        assert!(
            run_json_stdout.contains("std-preprocessed-pgm:input_bytes=20"),
            "PixelMagic shader trace did not carry std-preprocessed PGM evidence\n{run_json_stdout}"
        );
    }

    let provider_family = format!("{backend_family}:{target_device}");
    let provider_family_artifact = provider_family_artifact_component(&provider_family);
    let executed = nsdb::execute_provider_samples(&output_dir, Some(&provider_family))
        .expect("nsdb executes official galaxy provider samples");
    assert_eq!(
        executed.provider_family_filter.as_deref(),
        Some(provider_family.as_str())
    );
    assert_eq!(executed.record_count, provider_record_count);
    assert_eq!(executed.matched_record_count, 1);
    assert!(executed.provider_families.contains(&provider_family));
    assert_eq!(executed.first_provider_family, provider_family);
    assert!(executed
        .first_provider_runner_adapter_id
        .contains(&format!("{backend_family}.{target_device}")));
    assert!(matches!(
        executed
            .first_provider_runner_adapter_capability_status
            .as_str(),
        "registered-real-device" | "registered-host-simulated"
    ));
    assert_eq!(
        executed.first_provider_runner_real_device_capable,
        executed.first_provider_runner_adapter_capability_status == "registered-real-device"
    );
    assert!(matches!(
        executed
            .first_provider_runner_real_device_probe_status
            .as_str(),
        "real-device-candidate-available" | "real-device-candidate-unavailable"
    ));
    assert!(matches!(
        executed.first_provider_execution_mode.as_str(),
        "real-device-provider-runner" | "host-simulated-provider-runner"
    ));
    assert!(matches!(
        executed.status.as_str(),
        "provider-output-payloads-ready" | "no-real-device-provider-output"
    ));
    assert_eq!(
        executed.executable_record_count,
        executed.output_payload_count
    );
    assert_eq!(executed.next_action, "materialize-provider-samples");
    assert!(executed
        .next_command
        .contains("nsdb materialize-provider-samples "));
    assert_eq!(
        executed.first_output_payload_comparison_contract,
        "nuis-provider-execution-comparison-v1"
    );
    assert!(matches!(
        executed.first_output_payload_comparison_status.as_str(),
        "ready-for-comparison"
            | "awaiting-provider-output-payload"
            | "host-fallback-output-comparison-deferred"
    ));
    assert!(executed
        .first_output_payload_input_evidence_hash
        .starts_with("0x"));
    if expects_std_pgm_marker {
        assert!(
            executed
                .first_output_payload_input_evidence
                .contains("std-preprocessed-pgm:input_bytes=20"),
            "PixelMagic provider execution did not carry std input evidence\n{}",
            executed.first_output_payload_input_evidence
        );
        assert_eq!(
            executed.first_output_payload_native_output_kind,
            "pixelmagic-image-bytes"
        );
        if executed.first_provider_execution_mode == "real-device-provider-runner" {
            assert_eq!(
                executed.first_output_payload_native_output_status,
                "metal-api-output-ready"
            );
            assert_eq!(
                executed.first_output_payload_native_execution_contract,
                "nuis-metal-provider-runner-v1"
            );
            assert_eq!(
                executed.first_output_payload_native_execution_status,
                "metal-command-buffer-completed"
            );
            assert_ne!(executed.first_output_payload_native_device, "none");
        } else {
            assert_eq!(
                executed.first_output_payload_native_output_status,
                "deterministic-provider-output-ready"
            );
            assert_eq!(
                executed.first_output_payload_native_execution_contract,
                "nuis-deterministic-provider-output-v1"
            );
        }
        assert_eq!(executed.first_output_payload_native_output_bytes, "24");
        assert!(executed
            .first_output_payload_native_output_hash
            .starts_with("0x"));
    }
    if executed.output_payload_count == 0 {
        assert_eq!(executed.first_output_payload_evidence, "none");
    } else {
        assert_eq!(executed.status, "provider-output-payloads-ready");
        assert_eq!(
            executed.first_output_payload_comparison_status,
            "ready-for-comparison"
        );
        assert!(executed.first_output_payload_evidence.contains(&format!(
            "nuis.nsdb.provider-output.{provider_family_artifact}.toml:hash=0x"
        )));
    }

    let materialize_filter = (provider_record_count == 1).then_some(provider_family.as_str());
    let materialized = nsdb::materialize_provider_samples(&output_dir, materialize_filter)
        .expect("nsdb materializes official galaxy provider samples");
    let provider_samples =
        fs::read_to_string(output_dir.join("nuis.nsdb.device-provider-samples.toml"))
            .expect("device provider sample manifest remains available");
    let doctor_after = run_nuis(&["artifact-doctor", "--json", &output_dir_text]);
    assert_success(
        &doctor_after,
        "nuis artifact-doctor json after official galaxy sample materialization",
    );
    let doctor_after_stdout = String::from_utf8_lossy(&doctor_after.stdout);

    assert_eq!(materialized.status, "ready");
    assert_eq!(materialized.matched_record_count, provider_record_count);
    assert_eq!(
        materialized.materialized_record_count,
        provider_record_count
    );
    assert_eq!(materialized.skipped_record_count, 0);
    assert_eq!(materialized.next_action, "replay-provider-sample");
    assert!(materialized.next_command.contains("nsdb replay-plan "));
    assert_eq!(
        materialized.return_contract,
        "nsld-final-output-boundary-return-v1"
    );
    assert_eq!(
        materialized.final_output_replay_contract,
        "nsdb-payload-execution-replay-plan-v1"
    );
    assert!(materialized.return_command.contains("nsld check "));
    assert_eq!(
        materialized.first_provider_runner_registry_protocol,
        "nuis-provider-runner-registry-v1"
    );
    assert_eq!(
        materialized.first_provider_runner_registry_source,
        "builtin-nustar-provider-runner-registry"
    );
    assert!(matches!(
        materialized
            .first_provider_runner_adapter_capability_status
            .as_str(),
        "registered-real-device" | "registered-host-simulated"
    ));
    assert_eq!(
        materialized.first_provider_runner_real_device_capable,
        materialized.first_provider_runner_adapter_capability_status == "registered-real-device"
    );
    assert!(matches!(
        materialized.first_provider_execution_mode.as_str(),
        "real-device-provider-runner" | "host-simulated-provider-runner"
    ));
    assert!(provider_samples.contains("source = \"nsdb-materialize-provider-samples\""));
    assert!(provider_samples.contains("status = \"ready\""));
    assert!(provider_samples.contains(&format!("ready_record_count = {provider_record_count}")));
    assert!(provider_samples.contains("pending_record_count = 0"));
    assert!(provider_samples.contains("sample_status = \"provider-execution-ready\""));
    assert!(provider_samples.contains("validation_status = \"provider-execution-validated\""));
    if expects_std_pgm_marker {
        assert!(provider_samples.contains("std-preprocessed-pgm:input_bytes=20"));
    }
    assert!(provider_samples.contains("output_evidence = \"nuis.nsdb.provider-sample."));
    assert!(provider_samples.contains(":hash=0x"));
    assert!(provider_samples.contains("materialization_status = \"provider-sample-materialized\""));
    assert!(provider_samples.contains("requested_runner_contract = \"nuis-provider-runner-v1\""));
    assert!(provider_samples
        .contains("requested_runner_adapter_contract = \"nuis-provider-runner-adapter-v1\""));
    assert!(provider_samples.contains("requested_runner_adapter_id = \""));
    assert!(provider_samples
        .contains("requested_runner_adapter_capability_status = \"registered-host-simulated\""));
    assert!(provider_samples.contains("provider_runner_contract = \"nuis-provider-runner-v1\""));
    assert!(provider_samples
        .contains("provider_runner_adapter_contract = \"nuis-provider-runner-adapter-v1\""));
    assert!(provider_samples.contains("provider_runner_adapter_id = \""));
    assert!(
        provider_samples
            .contains("provider_runner_adapter_capability_status = \"registered-real-device\"")
            || provider_samples.contains(
                "provider_runner_adapter_capability_status = \"registered-host-simulated\""
            )
    );
    assert!(provider_samples
        .contains("provider_runner_registry_protocol = \"nuis-provider-runner-registry-v1\""));
    assert!(provider_samples
        .contains("provider_runner_registry_source = \"builtin-nustar-provider-runner-registry\""));
    assert!(
        provider_samples.contains("provider_runner_real_device_capable = true")
            || provider_samples.contains("provider_runner_real_device_capable = false")
    );
    assert!(provider_samples.contains("provider_runner_real_device_probe_status = \""));
    assert!(
        provider_samples.contains("provider_execution_mode = \"real-device-provider-runner\"")
            || provider_samples
                .contains("provider_execution_mode = \"host-simulated-provider-runner\"")
    );
    if executed.output_payload_count == 0 {
        assert!(provider_samples
            .contains("provider_output_payload_status = \"host-fallback-output-payload-ready\""));
    } else {
        assert!(provider_samples
            .contains("provider_output_payload_status = \"real-device-output-payload-attached\""));
        assert!(provider_samples.contains(&format!(
            "provider_output_payload_evidence = \"nuis.nsdb.provider-output.{provider_family_artifact}.toml:hash=0x"
        )));
        let expected_payload_kind = if expects_std_pgm_marker
            && executed.first_output_payload_native_output_status == "metal-api-output-ready"
        {
            "output_payload_kind = \"real-device-api-output\""
        } else {
            "output_payload_kind = \"real-device-adapter-output\""
        };
        assert_file_contains(
            &output_dir.join(format!(
                "nuis.nsdb.provider-output.{provider_family_artifact}.toml"
            )),
            expected_payload_kind,
            "official galaxy provider output payload",
        );
    }
    let provider_output_payload_path = output_dir.join(format!(
        "nuis.nsdb.provider-output.{provider_family_artifact}.toml"
    ));
    assert_file_contains(
        &provider_output_payload_path,
        "schema = \"nsdb-provider-output-payload-v1\"",
        "official galaxy provider output payload schema",
    );
    assert_file_contains(
        &provider_output_payload_path,
        "sample_execution_contract = \"nuis-provider-sample-execution-v1\"",
        "official galaxy provider output payload execution contract",
    );
    assert_file_contains(
        &provider_output_payload_path,
        "provider_runner_adapter_capability_status = \"",
        "official galaxy provider output payload adapter capability",
    );
    assert_file_contains(
        &provider_output_payload_path,
        "provider_runner_real_device_probe_status = \"",
        "official galaxy provider output payload probe status",
    );
    assert_file_contains(
        &provider_output_payload_path,
        "input_evidence_hash = \"0x",
        "official galaxy provider output payload input evidence hash",
    );
    if expects_std_pgm_marker {
        assert_file_contains(
            &provider_output_payload_path,
            "std-preprocessed-pgm:input_bytes=20",
            "official galaxy provider output payload std image evidence",
        );
        assert_file_contains(
            &provider_output_payload_path,
            "native_output_kind = \"pixelmagic-image-bytes\"",
            "official galaxy provider output payload native output kind",
        );
        assert_file_contains(
            &provider_output_payload_path,
            "native_output_bytes = \"24\"",
            "official galaxy provider output payload native output bytes",
        );
        assert_file_contains(
            &provider_output_payload_path,
            "native_output_execution_contract = \"",
            "official galaxy provider output payload native execution contract",
        );
        assert_file_contains(
            &provider_output_payload_path,
            "native_output_device = \"",
            "official galaxy provider output payload native device",
        );
    }
    assert!(provider_samples
        .contains("materialization_detail = \"deterministic-provider-sample-artifact:"));
    assert!(provider_samples.contains("next_action = \"replay-device-sample\""));
    if expects_std_pgm_marker {
        let provider_sample_artifact = fs::read_dir(&output_dir)
            .unwrap()
            .filter_map(Result::ok)
            .find(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("nuis.nsdb.provider-sample.")
            })
            .map(|entry| fs::read_to_string(entry.path()).expect("read provider sample artifact"))
            .expect("provider sample artifact should be materialized");
        assert!(provider_sample_artifact.contains("std-preprocessed-pgm:input_bytes=20"));
    }
    assert!(
        fs::read_dir(&output_dir)
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| entry
                .file_name()
                .to_string_lossy()
                .starts_with("nuis.nsdb.provider-sample.")),
        "provider sample artifact was not materialized in {}",
        output_dir.display()
    );
    assert!(doctor_after_stdout
        .contains("\"artifact_device_provider_sample_manifest_status\":\"ready\""));
    assert!(doctor_after_stdout
        .contains("\"artifact_device_provider_sample_manifest_pending_record_count\":0"));
    assert!(doctor_after_stdout.contains(
        "\"artifact_device_provider_sample_manifest_first_materialization_status\":\"provider-sample-materialized\""
    ));

    if label == "pixelmagic_pipeline_demo" {
        assert_multi_checkpoint_replay_resume(&output_dir);
    }
}

fn assert_multi_checkpoint_replay_resume(output_dir: &Path) {
    let output_dir_text = output_dir.display().to_string();
    let unavailable = run_nuis(&["debug-resume", "--json", &output_dir_text]);
    assert!(
        !unavailable.status.success()
            && String::from_utf8_lossy(&unavailable.stderr).contains("cursor-unavailable"),
        "Nuis debug-resume must reject an unavailable cursor before dispatch\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&unavailable.stdout),
        String::from_utf8_lossy(&unavailable.stderr)
    );
    let replay = run_nsdb(&["replay", &output_dir_text, "--json"]);
    assert_success(&replay, "nsdb replay official multi-checkpoint artifact");
    let replay_stdout = String::from_utf8_lossy(&replay.stdout);
    let frame_ids = json_string_values(&replay_stdout, "frame_id");
    assert!(
        frame_ids.len() >= 3,
        "official hetero replay should expose at least three YIR frames\n{replay_stdout}"
    );
    let first = &frame_ids[0];
    let second = &frame_ids[1];
    let third = &frame_ids[2];
    let cursor_path = output_dir.join("nuis.nsdb.replay-cursor.toml");
    let cursor_path_text = cursor_path.display().to_string();

    let stopped = run_nsdb(&[
        "replay",
        &output_dir_text,
        "--break-at",
        first,
        "--save-cursor",
        &cursor_path_text,
        "--json",
    ]);
    assert_success(&stopped, "nsdb persist first hetero replay cursor");
    assert_file_contains(
        &cursor_path,
        "protocol = \"nsdb-yir-replay-cursor-record-v1\"",
        "persisted replay cursor protocol",
    );
    assert_file_contains(
        &cursor_path,
        &format!("after_frame_id = \"{first}\""),
        "persisted replay cursor stopped frame",
    );
    assert_file_contains(
        &cursor_path,
        &format!("next_frame_id = \"{second}\""),
        "persisted replay cursor next frame",
    );

    let report = run_nuis(&["build-report", "--json", &output_dir_text]);
    assert_success(&report, "nuis mirror persisted debugger cursor");
    let report_stdout = String::from_utf8_lossy(&report.stdout);
    assert!(
        report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_handoff_contract\":\"nuis-debugger-cursor-handoff-v1\""
        ) && report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_ready\":true"
        ) && report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_status\":\"cursor-resume-ready\""
        ) && report_stdout.contains(
            "\"closure_summary_debugger_cursor_handoff_contract\":\"nuis-debugger-cursor-handoff-v1\""
        ) && report_stdout.contains("\"closure_summary_debugger_cursor_ready\":true")
            && report_stdout.contains(
                "\"closure_summary_debugger_cursor_status\":\"cursor-resume-ready\""
            )
            && report_stdout.contains(
                "\"nsld_final_executable_output_debugger_cursor_next_command\":\"nuis debug-resume "
            )
            && report_stdout.contains(
                "\"closure_summary_debugger_cursor_next_command\":\"nuis debug-resume "
            )
            && report_stdout.contains("--json")
            && report_stdout.contains("nuis.nsdb.replay-cursor.toml"),
        "Nuis frontdoors should mirror the persisted debugger cursor without Nsdb type coupling\n{report_stdout}"
    );

    let resumed = run_nuis(&[
        "debug-resume",
        "--json",
        "--break-at",
        second,
        "--save-cursor",
        &cursor_path_text,
        &output_dir_text,
    ]);
    assert_success(&resumed, "nuis first-class heterogeneous debug resume");
    let resumed_stdout = String::from_utf8_lossy(&resumed.stdout);
    assert!(
        resumed_stdout.contains("\"debugger_transcript_resume_input_status\":\"cursor-accepted\"")
            && resumed_stdout.contains("\"debugger_transcript_control_status\":\"breakpoint-hit\"")
            && resumed_stdout.contains(&format!(
                "\"debugger_transcript_selected_frame_id\":\"{second}\""
            ))
            && resumed_stdout.contains("\"debugger_transcript_replayed_checkpoint_count\":1"),
        "Nuis debug-resume should validate, resume, and stop at the selected heterogeneous frame\n{resumed_stdout}"
    );
    assert_file_contains(
        &cursor_path,
        &format!("after_frame_id = \"{second}\""),
        "replaced replay cursor stopped frame",
    );
    assert_file_contains(
        &cursor_path,
        &format!("next_frame_id = \"{third}\""),
        "replaced replay cursor next frame",
    );
    let lineage_path = output_dir.join("nuis.nsdb.replay-cursor.lineage.toml");
    assert_file_contains(
        &lineage_path,
        "protocol = \"nsdb-yir-replay-cursor-lineage-v1\"",
        "replay cursor lineage protocol",
    );
    assert_file_contains(
        &lineage_path,
        "entry_count = 2",
        "replay cursor lineage entry count",
    );
    assert_file_contains(
        &lineage_path,
        "sequence = 0",
        "initial replay cursor lineage sequence",
    );
    assert_file_contains(
        &lineage_path,
        "sequence = 1",
        "replacement replay cursor lineage sequence",
    );
    assert_file_contains(
        &lineage_path,
        "current_hash = \"0x",
        "replay cursor lineage content hash",
    );
    let lineage_source = fs::read_to_string(&lineage_path).expect("read replay cursor lineage");
    let latest_hash = lineage_source
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("current_hash = \"")
                .and_then(|value| value.strip_suffix('"'))
        })
        .last()
        .expect("replay cursor lineage latest hash");
    let lineage_report = run_nuis(&["build-report", "--json", &output_dir_text]);
    assert_success(&lineage_report, "nuis mirror debugger cursor lineage");
    let lineage_report_stdout = String::from_utf8_lossy(&lineage_report.stdout);
    assert!(
        lineage_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_contract\":\"nuis-debugger-cursor-lineage-mirror-v1\""
        ) && lineage_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_source_protocol\":\"nsdb-yir-replay-cursor-lineage-v1\""
        ) && lineage_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_ready\":true"
        ) && lineage_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_status\":\"lineage-ready\""
        ) && lineage_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_entry_count\":2"
        ) && lineage_report_stdout.contains(&format!(
            "\"nsld_final_executable_output_debugger_cursor_lineage_latest_hash\":\"{latest_hash}\""
        )) && lineage_report_stdout.contains(
            "\"closure_summary_debugger_cursor_lineage_ready\":true"
        ) && lineage_report_stdout.contains(
            "\"closure_summary_debugger_cursor_lineage_entry_count\":2"
        ) && lineage_report_stdout.contains(&format!(
            "\"closure_summary_debugger_cursor_lineage_latest_hash\":\"{latest_hash}\""
        )),
        "Nuis final-output and closure summaries should mirror the hash-checked debugger cursor lineage\n{lineage_report_stdout}"
    );
    fs::write(
        &lineage_path,
        lineage_source.replacen(latest_hash, "0x0000000000000000", 1),
    )
    .expect("damage replay cursor lineage latest hash");
    let invalid_report = run_nuis(&["build-report", "--json", &output_dir_text]);
    assert_success(
        &invalid_report,
        "nuis diagnose invalid debugger cursor lineage",
    );
    let invalid_report_stdout = String::from_utf8_lossy(&invalid_report.stdout);
    assert!(
        invalid_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_status\":\"lineage-invalid\""
        ) && invalid_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_first_blocker\":\"lineage-latest-hash-mismatch\""
        ) && invalid_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_next_action\":\"repair-cursor-lineage\""
        ) && invalid_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_next_command\":\"nuis debug-lineage-repair "
        ) && invalid_report_stdout.contains(
            "\"closure_summary_debugger_cursor_lineage_first_blocker\":\"lineage-latest-hash-mismatch\""
        ),
        "Nuis should expose an actionable stable blocker for stale cursor lineage\n{invalid_report_stdout}"
    );
    let repaired = run_nuis(&["debug-lineage-repair", &output_dir_text, "--json"]);
    assert_success(&repaired, "nuis repair debugger cursor lineage");
    let repaired_stdout = String::from_utf8_lossy(&repaired.stdout);
    assert!(
        repaired_stdout.contains("\"contract\":\"nsdb-yir-replay-cursor-lineage-repair-v2\"")
            && repaired_stdout.contains("\"status\":\"lineage-rebuilt\"")
            && repaired_stdout.contains("\"mutated\":true")
            && repaired_stdout.contains("\"archived_path\":\"")
            && repaired_stdout.contains("\"entry_count\":1")
            && repaired_stdout.contains("\"latest_hash\":\"0x"),
        "Nsdb should archive and rebuild invalid cursor lineage\n{repaired_stdout}"
    );
    let already_ready = run_nuis(&["debug-lineage-repair", &output_dir_text, "--json"]);
    assert_success(&already_ready, "nuis keep healthy cursor lineage unchanged");
    let already_ready_stdout = String::from_utf8_lossy(&already_ready.stdout);
    assert!(
        already_ready_stdout.contains("\"status\":\"already-ready\"")
            && already_ready_stdout.contains("\"mutated\":false"),
        "Nuis should preserve Nsdb's idempotent healthy-lineage result\n{already_ready_stdout}"
    );
    let repaired_report = run_nuis(&["build-report", "--json", &output_dir_text]);
    assert_success(
        &repaired_report,
        "nuis mirror repaired debugger cursor lineage",
    );
    let repaired_report_stdout = String::from_utf8_lossy(&repaired_report.stdout);
    assert!(
        repaired_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_status\":\"lineage-ready\""
        ) && repaired_report_stdout
            .contains("\"nsld_final_executable_output_debugger_cursor_lineage_entry_count\":1")
            && repaired_report_stdout.contains(
                "\"nsld_final_executable_output_debugger_cursor_lineage_first_blocker\":null"
            ),
        "Nuis should report repaired cursor lineage as ready\n{repaired_report_stdout}"
    );
    assert!(
        repaired_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_repair_contract\":\"nuis-debugger-cursor-lineage-repair-mirror-v1\""
        ) && repaired_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_repair_status\":\"repair-history-ready\""
        ) && repaired_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_repair_entry_count\":1"
        ) && repaired_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_repair_latest_mutated\":true"
        ) && repaired_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_repair_latest_archived_path\":\""
        ) && repaired_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_repair_latest_archived_hash\":\"0x"
        ) && repaired_report_stdout.contains(&format!(
            "\"nsld_final_executable_output_debugger_cursor_lineage_repair_latest_rebuilt_hash\":\"{latest_hash}\""
        )) && repaired_report_stdout.contains(
            "\"closure_summary_debugger_cursor_lineage_repair_status\":\"repair-history-ready\""
        ) && repaired_report_stdout.contains(
            "\"closure_summary_debugger_cursor_lineage_repair_entry_count\":1"
        ) && repaired_report_stdout.contains(&format!(
            "\"closure_summary_debugger_cursor_lineage_repair_latest_rebuilt_hash\":\"{latest_hash}\""
        )),
        "Nuis should preserve cursor-lineage repair audit evidence beyond command stdout\n{repaired_report_stdout}"
    );
    let repaired_lineage_source = fs::read_to_string(&lineage_path)
        .expect("read repaired cursor lineage before journal recovery smoke");
    fs::write(
        &lineage_path,
        repaired_lineage_source.replacen(latest_hash, "0x0000000000000000", 1),
    )
    .expect("damage cursor lineage before journal recovery smoke");
    let repair_journal_path = output_dir.join("nuis.nsdb.replay-cursor.lineage-repairs.toml");
    fs::write(&repair_journal_path, "protocol = \"damaged-journal\"\n")
        .expect("damage cursor lineage repair journal");
    let recovered = run_nuis(&["debug-lineage-repair", &output_dir_text, "--json"]);
    assert_success(&recovered, "nuis recover damaged lineage repair journal");
    let recovered_stdout = String::from_utf8_lossy(&recovered.stdout);
    assert!(
        recovered_stdout.contains("\"status\":\"lineage-rebuilt\"")
            && recovered_stdout.contains("\"archived_repair_journal_path\":\""),
        "Nuis should archive the damaged repair journal before rebuilding lineage\n{recovered_stdout}"
    );
    let recovered_report = run_nuis(&["build-report", "--json", &output_dir_text]);
    assert_success(&recovered_report, "nuis mirror recovered repair journal");
    let recovered_report_stdout = String::from_utf8_lossy(&recovered_report.stdout);
    assert!(
        recovered_report_stdout.contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_repair_status\":\"repair-history-ready\""
        ) && recovered_report_stdout.contains(
            "\"closure_summary_debugger_cursor_lineage_repair_status\":\"repair-history-ready\""
        ),
        "Nuis should mirror the recovered cursor-lineage repair journal\n{recovered_report_stdout}"
    );
    let healthy_lineage =
        fs::read(&lineage_path).expect("read healthy lineage before journal-only recovery smoke");
    fs::write(&repair_journal_path, "protocol = \"journal-only-damage\"\n")
        .expect("damage only cursor lineage repair journal");
    let journal_only = run_nuis(&["debug-lineage-repair", &output_dir_text, "--json"]);
    assert_success(
        &journal_only,
        "nuis recover journal without lineage rebuild",
    );
    let journal_only_stdout = String::from_utf8_lossy(&journal_only.stdout);
    assert!(
        journal_only_stdout.contains("\"contract\":\"nsdb-yir-replay-cursor-lineage-repair-v2\"")
            && journal_only_stdout.contains("\"status\":\"repair-history-recovered\"")
            && journal_only_stdout.contains("\"mutated\":true")
            && journal_only_stdout.contains("\"lineage_mutated\":false")
            && journal_only_stdout.contains("\"repair_journal_mutated\":true")
            && journal_only_stdout.contains("\"archived_repair_journal_path\":\""),
        "Nuis should report journal-only recovery with separate mutation scopes\n{journal_only_stdout}"
    );
    assert_eq!(
        fs::read(&lineage_path).expect("read lineage after journal-only recovery"),
        healthy_lineage,
        "journal-only recovery must preserve authoritative lineage bytes"
    );
    let journal_only_report = run_nuis(&["build-report", "--json", &output_dir_text]);
    assert_success(&journal_only_report, "nuis mirror journal-only recovery");
    assert!(
        String::from_utf8_lossy(&journal_only_report.stdout).contains(
            "\"nsld_final_executable_output_debugger_cursor_lineage_repair_status\":\"repair-history-ready\""
        ),
        "Nuis should return journal-only recovery to repair-history-ready"
    );

    let resumed_again = run_nuis(&[
        "debug-resume",
        "--json",
        "--break-at",
        third,
        &output_dir_text,
    ]);
    assert_success(&resumed_again, "nuis chained heterogeneous debug resume");
    let resumed_again_stdout = String::from_utf8_lossy(&resumed_again.stdout);
    assert!(
        resumed_again_stdout
            .contains("\"debugger_transcript_resume_input_status\":\"cursor-accepted\"")
            && resumed_again_stdout
                .contains("\"debugger_transcript_control_status\":\"breakpoint-hit\"")
            && resumed_again_stdout.contains(&format!(
                "\"debugger_transcript_selected_frame_id\":\"{third}\""
            ))
            && resumed_again_stdout
                .contains("\"debugger_transcript_replayed_checkpoint_count\":1"),
        "Nuis debug-resume should consume the replaced cursor and stop at the third heterogeneous frame\n{resumed_again_stdout}"
    );
}

#[test]
fn official_galaxy_hetero_projects_emit_shader_and_kernel_artifacts() {
    assert_official_galaxy_hetero_build(
        "pixelmagic_pipeline_demo",
        "../../examples/projects/domains/pixelmagic_pipeline_demo",
        "shader",
        "metal",
        "apple-silicon-gpu",
        3,
        "hetero-trace:shader:metal:apple-silicon-gpu",
        &[
            "shader.begin_pass",
            "shader.draw_instanced",
            "shader.inline_wgsl",
            "PixelMagicContracts.shader_pipeline_total",
        ],
        &[
            "shader_stage_model = \"metal-render-pipeline\"",
            "lowering_capabilities",
            "pipeline_lowering = \"metal-render-pipeline-state\"",
            "execution_route = \"unified-render-graph\"",
        ],
        &[
            "backend_family = \"metal\"",
            "target_device = \"apple-silicon-gpu\"",
            "shader.inline_wgsl",
        ],
    );

    assert_official_galaxy_hetero_build(
        "witsage_kernel_demo",
        "../../examples/projects/domains/witsage_kernel_demo",
        "kernel",
        "coreml",
        "apple-ane",
        1,
        "hetero-trace:kernel:coreml:apple-ane",
        &[
            "kernel.tensor",
            "kernel.reduce_mean_axis",
            "kernel.topk_axis",
            "WitSageContracts.kernel_pipeline_total",
        ],
        &[
            "kernel_ir = \"coreml-program\"",
            "kernel_entry_model = \"mlmodelc-function\"",
            "tensor_lowering = \"ranked-tensor-graph\"",
        ],
        &[
            "backend_family = \"coreml\"",
            "target_device = \"apple-ane\"",
            "kernel.reduce_mean_axis",
        ],
    );
}
