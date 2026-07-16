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

    let materialized = nsdb::materialize_provider_samples(
        &output_dir,
        Some(&format!("{backend_family}:{target_device}")),
    )
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
    assert!(materialized.return_command.contains("nsld check "));
    assert!(provider_samples.contains("source = \"nsdb-materialize-provider-samples\""));
    assert!(provider_samples.contains("status = \"ready\""));
    assert!(provider_samples.contains("ready_record_count = 1"));
    assert!(provider_samples.contains("pending_record_count = 0"));
    assert!(provider_samples.contains("sample_status = \"provider-execution-ready\""));
    assert!(provider_samples.contains("validation_status = \"provider-execution-validated\""));
    assert!(provider_samples.contains("output_evidence = \"nuis.nsdb.provider-sample."));
    assert!(provider_samples.contains(":hash=0x"));
    assert!(provider_samples.contains("materialization_status = \"provider-sample-materialized\""));
    assert!(provider_samples.contains("provider_runner_contract = \"nuis-provider-runner-v1\""));
    assert!(provider_samples
        .contains("provider_runner_adapter_contract = \"nuis-provider-runner-adapter-v1\""));
    assert!(provider_samples.contains("provider_runner_adapter_id = \""));
    assert!(provider_samples
        .contains("provider_runner_adapter_capability_status = \"registered-host-simulated\""));
    assert!(
        provider_samples.contains("provider_execution_mode = \"host-simulated-provider-runner\"")
    );
    assert!(provider_samples
        .contains("materialization_detail = \"deterministic-provider-sample-artifact:"));
    assert!(provider_samples.contains("next_action = \"replay-device-sample\""));
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
