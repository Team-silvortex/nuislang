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
                "\"nsld_final_executable_output_nsdb_replay_first_blocker\":\"handoff-metadata-missing\""
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
    assert!(
        run_json_stdout.contains("\"hetero_runtime_trace_available\":true")
            && run_json_stdout.contains("\"hetero_runtime_trace_status\":\"execution-pending\"")
            && run_json_stdout.contains("\"hetero_runtime_trace_debugger_contract\":\"nsdb-yir-hetero-runtime-trace-v1\"")
            && run_json_stdout.contains(&format!("\"hetero_runtime_trace_record_count\":{trace_record_count}"))
            && run_json_stdout.contains("\"hetero_runtime_trace_backend_execution_record_count\":1")
            && run_json_stdout.contains("\"hetero_runtime_trace_device_sample_descriptor_count\":1")
            && run_json_stdout.contains("\"hetero_runtime_trace_device_sample_pending_count\":1")
            && run_json_stdout.contains("\"hetero_runtime_trace_device_sample_pending_validation_count\":1")
            && run_json_stdout.contains("\"hetero_runtime_trace_device_sample_providers\":[\"nustar-deferred-device-sample-v1\"]")
            && run_json_stdout.contains(&format!("\"hetero_runtime_trace_device_sample_provider_families\":[\"{backend_family}:{target_device}\"]"))
            && run_json_stdout.contains(&format!("\"hetero_runtime_trace_backend_families\":[\"{backend_family}\"]"))
            && run_json_stdout.contains(&format!("\"hetero_runtime_trace_target_devices\":[\"{target_device}\"]"))
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
    assert_eq!(executed.record_count, 1);
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
    if executed.output_payload_count == 0 {
        assert_eq!(executed.first_output_payload_evidence, "none");
    } else {
        assert_eq!(executed.status, "provider-output-payloads-ready");
        assert!(executed.first_output_payload_evidence.contains(&format!(
            "nuis.nsdb.provider-output.{provider_family_artifact}.toml:hash=0x"
        )));
    }

    let materialized = nsdb::materialize_provider_samples(&output_dir, Some(&provider_family))
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
    assert_eq!(materialized.matched_record_count, 1);
    assert_eq!(materialized.materialized_record_count, 1);
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
    assert!(provider_samples.contains("ready_record_count = 1"));
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
        assert_file_contains(
            &output_dir.join(format!(
                "nuis.nsdb.provider-output.{provider_family_artifact}.toml"
            )),
            "output_payload_kind = \"real-device-adapter-output\"",
            "official galaxy provider output payload",
        );
    }
    if expects_std_pgm_marker {
        assert_file_contains(
            &output_dir.join(format!(
                "nuis.nsdb.provider-output.{provider_family_artifact}.toml"
            )),
            "std-preprocessed-pgm:input_bytes=20",
            "official galaxy provider output payload std image evidence",
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
}

#[test]
fn official_galaxy_hetero_projects_emit_shader_and_kernel_artifacts() {
    assert_official_galaxy_hetero_build(
        "pixelmagic_pipeline_demo",
        "../../examples/projects/domains/pixelmagic_pipeline_demo",
        "shader",
        "metal",
        "apple-silicon-gpu",
        2,
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
