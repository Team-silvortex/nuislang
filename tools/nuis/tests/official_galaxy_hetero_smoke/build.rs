use super::*;

pub(super) fn assert_official_galaxy_hetero_build(
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
            run_json_stdout.contains("std-preprocessed-pgm:input_bytes=20")
                && run_json_stdout.contains(
                    "provider_buffer_descriptor_contract=nuis-provider-buffer-descriptor-v1",
                )
                && run_json_stdout.contains(
                    "provider_kernel_descriptor_contract=nuis-provider-kernel-descriptor-v1",
                )
                && run_json_stdout.contains("provider_buffer_id=input.pixels")
                && run_json_stdout.contains("provider_kernel_id=pixelmagic.gray8.invert")
                && run_json_stdout.contains("pixel_format=gray8")
                && run_json_stdout.contains("pixel_width=2")
                && run_json_stdout.contains("pixel_height=2")
                && run_json_stdout.contains(
                    "pixel_payload_path=nuis.pixelmagic.std-preprocessed.gray8.bin",
                ),
            "PixelMagic shader trace did not carry std-preprocessed PGM evidence\n{run_json_stdout}"
        );
        assert_eq!(
            fs::read(output_dir.join("nuis.pixelmagic.std-preprocessed.gray8.bin"))
                .expect("read persisted PixelMagic input payload"),
            [0, 4, 9, 8]
        );
    }
    let expects_coreml_vector = backend_family == "coreml" && target_device == "apple-ane";
    if expects_coreml_vector {
        assert!(
            run_json_stdout.contains("provider_buffer_element_type=f32")
                && run_json_stdout.contains("provider_buffer_layout=tensor-contiguous")
                && run_json_stdout.contains("provider_buffer_shape=16x64x64")
                && run_json_stdout
                    .contains("provider_kernel_id=witsage.feature-grid.projection")
                && run_json_stdout.contains(
                    "provider_model_asset_descriptor_contract=nuis-provider-model-asset-descriptor-v1",
                )
                && run_json_stdout.contains(
                    "provider_model_asset_path=nuis.witsage.feature-grid-projection.mlmodel",
                )
                && run_json_stdout.contains(
                    "provider_request_collection_contract=nuis-provider-request-collection-v1",
                )
                && run_json_stdout.contains("provider_request_count=5")
                && run_json_stdout.contains(
                    "provider_output_comparison_descriptor_contract=nuis-provider-output-comparison-descriptor-v1",
                )
                && run_json_stdout.contains(
                    "provider_request_0_input_binding_contract=nuis-provider-input-binding-v1",
                )
                && run_json_stdout
                    .contains("provider_request_1_kernel_id=witsage.vector.affine")
                && run_json_stdout.contains(
                    "provider_request_2_kernel_id=witsage.vector.affine.chained",
                )
                && run_json_stdout.contains(
                    "provider_request_2_dependency_0_producer_request_id=witsage.vector.affine",
                )
                && run_json_stdout.contains(
                    "provider_request_3_kernel_input_buffers=input.left,input.right",
                )
                && run_json_stdout.contains(
                    "provider_request_4_adapter_binding_contract=nuis-provider-request-adapter-binding-v1",
                )
                && run_json_stdout.contains(
                    "provider_request_4_adapter_binding_provider_family=metal:apple-silicon-gpu",
                )
                && run_json_stdout.contains(
                    "provider_request_4_dependency_0_transport_contract=nuis-provider-edge-transport-v1",
                )
                && run_json_stdout.contains(
                    "provider_request_4_dependency_0_transport_producer_clock_evidence=provider-clock:request-3:completed",
                )
                && run_json_stdout.contains(
                    "provider_request_4_dependency_0_transport_consumer_clock_evidence=provider-clock:request-4:dispatch-ready",
                ),
            "WitSage kernel trace did not carry the registered CoreML request\n{run_json_stdout}"
        );
        let dense_payload = fs::read(output_dir.join("nuis.witsage.feature-grid.f32.bin"))
            .expect("read persisted WitSage feature-grid payload");
        assert_eq!(dense_payload.len(), 16 * 64 * 64 * 4);
        assert!(dense_payload
            .chunks_exact(4)
            .all(|bytes| f32::from_le_bytes(bytes.try_into().unwrap()) == 1.0));
        assert_eq!(
            fs::read(output_dir.join("nuis.witsage.vector.f32.bin"))
                .expect("read persisted WitSage vector payload"),
            [
                0x00, 0x00, 0x80, 0x3f, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x40, 0x40, 0x00, 0x00,
                0x80, 0x40,
            ]
        );
        assert!(
            fs::metadata(output_dir.join("nuis.witsage.feature-grid-projection.mlmodel"))
                .expect("persisted WitSage dense CoreML model")
                .len()
                > 1_000
        );
        assert_eq!(
            fs::read(output_dir.join("nuis.witsage.vector-affine.expected.f32.bin"))
                .expect("read persisted WitSage affine expected output"),
            [
                0x00, 0x00, 0x40, 0x40, 0x00, 0x00, 0xa0, 0x40, 0x00, 0x00, 0xe0, 0x40, 0x00, 0x00,
                0x10, 0x41,
            ]
        );
        assert_eq!(
            fs::read(output_dir.join("nuis.witsage.vector-affine-chained.expected.f32.bin"))
                .expect("read persisted WitSage chained expected output"),
            [
                0x00, 0x00, 0xe0, 0x40, 0x00, 0x00, 0x30, 0x41, 0x00, 0x00, 0x70, 0x41, 0x00, 0x00,
                0x98, 0x41,
            ]
        );
        assert_eq!(
            fs::read(output_dir.join("nuis.witsage.vector-add.expected.f32.bin"))
                .expect("read persisted WitSage add expected output"),
            [
                0x00, 0x00, 0x20, 0x41, 0x00, 0x00, 0x80, 0x41, 0x00, 0x00, 0xb0, 0x41, 0x00, 0x00,
                0xe0, 0x41,
            ]
        );
        assert_eq!(
            fs::read(output_dir.join("nuis.witsage.vector-metal-bias.expected.f32.bin"))
                .expect("read persisted WitSage Metal expected output"),
            [
                0x00, 0x00, 0x30, 0x41, 0x00, 0x00, 0x88, 0x41, 0x00, 0x00, 0xb8, 0x41, 0x00, 0x00,
                0xe8, 0x41,
            ]
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
        "comparison-passed"
            | "ready-for-comparison"
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
                "nuis-metal-gray8-provider-runner-v1"
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
        assert_eq!(executed.first_output_payload_native_output_bytes, "4");
        assert!(executed
            .first_output_payload_native_output_hash
            .starts_with("0x"));
    }
    if expects_coreml_vector
        && executed.first_provider_execution_mode == "real-device-provider-runner"
    {
        assert_eq!(
            executed.first_output_payload_native_output_kind,
            "provider-tensor-f32"
        );
        assert_eq!(
            executed.first_output_payload_native_output_status,
            "coreml-api-output-ready"
        );
        assert_eq!(
            executed.first_output_payload_native_execution_contract,
            "nuis-coreml-model-prediction-provider-runner-v1"
        );
        assert_eq!(
            executed.first_output_payload_native_execution_status,
            "coreml-model-prediction-completed"
        );
        assert_eq!(executed.first_output_payload_native_output_bytes, "262144");
        assert!(executed
            .first_output_payload_native_device
            .contains("CoreML.framework"));
        assert!(executed
            .first_output_payload_native_output_hash
            .starts_with("0x"));
        assert_eq!(
            executed.first_output_payload_native_output_hash, "0x9d85be94894a2325",
            "CoreML feature-grid projection must return 65,536 deterministic f32 ones"
        );
        assert_eq!(
            executed.first_output_payload_native_compute_plan_contract,
            "nuis-coreml-compute-plan-evidence-v1"
        );
        assert_eq!(
            executed.first_output_payload_native_compute_plan_status,
            "ready"
        );
        assert!(
            executed
                .first_output_payload_native_compute_plan_layer_count
                .parse::<usize>()
                .expect("CoreML compute-plan layer count")
                > 0
        );
        assert_eq!(
            executed.first_output_payload_native_compute_plan_preferred_devices,
            "neural-engine"
        );
        assert!(executed
            .first_output_payload_native_compute_plan_supported_devices
            .contains("neural-engine"));
    }
    if executed.output_payload_count == 0 {
        assert_eq!(executed.first_output_payload_evidence, "none");
    } else {
        assert_eq!(executed.status, "provider-output-payloads-ready");
        assert_eq!(
            executed.first_output_payload_comparison_status,
            if expects_coreml_vector
                && executed.first_provider_execution_mode == "real-device-provider-runner"
            {
                "comparison-passed"
            } else {
                "ready-for-comparison"
            }
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
        let expected_payload_kind = if matches!(
            executed.first_output_payload_native_output_status.as_str(),
            "metal-api-output-ready" | "coreml-api-output-ready"
        ) {
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
            "provider_request_source = \"registered-descriptors\"",
            "official galaxy provider request source",
        );
        assert_file_contains(
            &provider_output_payload_path,
            "provider_buffer_descriptor_contract = \"nuis-provider-buffer-descriptor-v1\"",
            "official galaxy provider buffer descriptor contract",
        );
        assert_file_contains(
            &provider_output_payload_path,
            "provider_buffer_id = \"input.pixels\"",
            "official galaxy provider buffer id",
        );
        assert_file_contains(
            &provider_output_payload_path,
            "provider_kernel_descriptor_contract = \"nuis-provider-kernel-descriptor-v1\"",
            "official galaxy provider kernel descriptor contract",
        );
        assert_file_contains(
            &provider_output_payload_path,
            "provider_kernel_id = \"pixelmagic.gray8.invert\"",
            "official galaxy provider kernel id",
        );
        assert_file_contains(
            &provider_output_payload_path,
            "std-preprocessed-pgm:input_bytes=20",
            "official galaxy provider output payload std image evidence",
        );
        assert_file_contains(
            &provider_output_payload_path,
            "pixel_payload_path=nuis.pixelmagic.std-preprocessed.gray8.bin",
            "official galaxy provider output payload pixel path evidence",
        );
        assert_file_contains(
            &provider_output_payload_path,
            "native_output_kind = \"pixelmagic-image-bytes\"",
            "official galaxy provider output payload native output kind",
        );
        assert_file_contains(
            &provider_output_payload_path,
            "native_output_bytes = \"4\"",
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
    if expects_coreml_vector {
        assert_file_contains(
            &provider_output_payload_path,
            "provider_request_source = \"registered-collection\"",
            "official galaxy CoreML provider request source",
        );
        assert_file_contains(
            &provider_output_payload_path,
            "provider_buffer_element_type = \"f32\"",
            "official galaxy CoreML provider buffer element type",
        );
        assert_file_contains(
            &provider_output_payload_path,
            "provider_kernel_id = \"witsage.feature-grid.projection\"",
            "official galaxy CoreML provider kernel id",
        );
        assert_file_contains(
            &provider_output_payload_path,
            "provider_model_asset_descriptor_contract = \"nuis-provider-model-asset-descriptor-v1\"",
            "official galaxy CoreML model asset contract",
        );
        assert_file_contains(
            &provider_output_payload_path,
            "provider_output_comparison_descriptor_contract = \"nuis-provider-output-comparison-descriptor-v1\"",
            "official galaxy CoreML output comparison contract",
        );
        assert_file_contains(
            &provider_output_payload_path,
            "provider_input_binding_contract = \"nuis-provider-input-binding-v1\"",
            "official galaxy CoreML input binding contract",
        );
        assert_file_contains(
            &provider_output_payload_path,
            "provider_input_binding_0_source = \"artifact\"",
            "official galaxy CoreML first input binding source",
        );
        if executed.first_provider_execution_mode == "real-device-provider-runner" {
            assert_eq!(
                executed.first_output_payload_comparison_status,
                "comparison-passed"
            );
            assert_file_contains(
                &provider_output_payload_path,
                "comparison_status = \"comparison-passed\"",
                "official galaxy CoreML aggregate comparison status",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_execution_contract = \"nuis-coreml-model-prediction-provider-runner-v1\"",
                "official galaxy CoreML native execution contract",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_execution_status = \"coreml-model-prediction-completed\"",
                "official galaxy CoreML native execution status",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_compute_plan_contract = \"nuis-coreml-compute-plan-evidence-v1\"",
                "official galaxy CoreML compute-plan contract",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_compute_plan_status = \"ready\"",
                "official galaxy CoreML compute-plan status",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_collection_contract = \"nuis-provider-output-collection-v1\"",
                "official galaxy CoreML output collection contract",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_count = \"5\"",
                "official galaxy CoreML output count",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "provider_request_order = \"witsage.feature-grid.projection,witsage.vector.affine,witsage.vector.affine.chained,witsage.vector.add,witsage.vector.metal-bias\"",
                "official galaxy CoreML request order",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "provider_request_dependency_contract = \"nuis-provider-request-dependency-v1\"",
                "official galaxy CoreML dependency contract",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "provider_request_dependency_edge_count = \"4\"",
                "official galaxy CoreML dependency edge count",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "provider_request_dependency_edges = \"witsage.vector.affine.output.features->witsage.vector.affine.chained.input.features,witsage.vector.affine.output.features->witsage.vector.add.input.left,witsage.vector.affine.chained.output.features->witsage.vector.add.input.right,witsage.vector.add.output.features->witsage.vector.metal-bias.input.features\"",
                "official galaxy CoreML dependency edge",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_collection_hash = \"0x",
                "official galaxy CoreML output collection hash",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_0_request_id = \"witsage.feature-grid.projection\"",
                "official galaxy dense output identity",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_0_compute_plan_preferred_devices = \"neural-engine\"",
                "official galaxy dense output compute preference",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_0_comparison_status = \"comparison-passed\"",
                "official galaxy dense output comparison status",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_0_comparison_element_count = \"65536\"",
                "official galaxy dense output comparison element count",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_0_comparison_mismatch_count = \"0\"",
                "official galaxy dense output comparison mismatch count",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_1_request_id = \"witsage.vector.affine\"",
                "official galaxy affine output identity",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_1_hash = \"0x44cf8b51954a5de2\"",
                "official galaxy affine output hash",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_1_compute_plan_preferred_devices = \"cpu\"",
                "official galaxy affine output compute preference",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_1_comparison_contract = \"nuis-provider-output-comparison-descriptor-v1\"",
                "official galaxy affine output comparison contract",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_1_comparison_status = \"comparison-passed\"",
                "official galaxy affine output comparison status",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_1_comparison_element_count = \"4\"",
                "official galaxy affine output comparison element count",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_2_request_id = \"witsage.vector.affine.chained\"",
                "official galaxy chained affine output identity",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_2_hash = \"0x834758988854dc4a\"",
                "official galaxy chained affine output hash",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_2_comparison_status = \"comparison-passed\"",
                "official galaxy chained affine comparison status",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_3_request_id = \"witsage.vector.add\"",
                "official galaxy fan-in output identity",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_3_hash = \"0x3efcc146d99e0b55\"",
                "official galaxy fan-in output hash",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_3_comparison_status = \"comparison-passed\"",
                "official galaxy fan-in comparison status",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "provider_request_adapter_order = \"coreml:apple-ane,coreml:apple-ane,coreml:apple-ane,coreml:apple-ane,metal:apple-silicon-gpu\"",
                "official galaxy cross-provider adapter order",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "provider_edge_transport_contract = \"nuis-provider-edge-transport-v1\"",
                "official galaxy cross-provider transport contract",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "provider_edge_transport_count = \"4\"",
                "official galaxy cross-provider transport count",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "provider_edge_transport_ownership_tokens = \"glm:provider-edge:witsage.vector.affine:output.features->witsage.vector.affine.chained:input.features,glm:provider-edge:witsage.vector.affine:output.features->witsage.vector.add:input.left,glm:provider-edge:witsage.vector.affine.chained:output.features->witsage.vector.add:input.right,glm:provider-edge:witsage.vector.add:output.features->witsage.vector.metal-bias:input.features\"",
                "official galaxy cross-provider GLM ownership token",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "provider_edge_transport_staging_modes = \"auto,auto,auto,auto\"",
                "official galaxy cross-provider staging mode",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "provider_edge_transport_receipt_contract = \"nuis-provider-edge-transport-receipt-v1\"",
                "official galaxy cross-provider transport receipt contract",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "provider_edge_transport_receipt_count = \"4\"",
                "official galaxy cross-provider transport receipt count",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "provider_edge_transport_receipt_3_staging_registry_contract = \"nuis-provider-edge-staging-registry-v1\"",
                "official galaxy staging registry contract",
            );
            for index in 0..4 {
                assert_file_contains(&provider_output_payload_path, &format!(
                    "provider_edge_transport_receipt_{index}_staging_adapter_id = \"provider.output.transfer.v1\""
                ), "official galaxy transferred output staging adapter");
            }
            assert_file_contains(
                &provider_output_payload_path,
                "provider_edge_transport_receipt_3_staging_adapter_capability_status = \"registered-available\"",
                "official galaxy staging adapter capability",
            );
            for evidence in [
                "native_output_4_output_residency_contract = \"nuis-provider-output-residency-v1\"",
                "native_output_4_output_residency_kind = \"host-visible-file\"",
                "native_output_0_session_lease_contract = \"nuis-provider-session-lease-v1\"",
                "native_output_0_session_adapter_id = \"logical.request-process.v1\"",
                "native_output_0_session_lifecycle_hooks = \"graph-open,request-begin,request-complete,graph-close\"",
                "native_output_3_session_request_sequence = \"3\"",
                "native_output_4_session_request_sequence = \"0\"",
                "native_output_0_worker_lease_contract = \"nuis-provider-worker-lease-v1\"",
                "native_output_0_worker_resolver_contract = \"nuis-provider-worker-image-resolver-v1\"",
                "native_output_3_worker_request_sequence = \"3\"",
                "native_output_4_worker_request_sequence = \"0\"",
                "native_output_3_worker_descriptor_count = \"2\"",
                "native_output_4_worker_descriptor_count = \"1\"",
                "native_output_0_worker_pid = \"",
                "native_output_0_worker_payload_hash = \"0x",
                "native_output_0_worker_operation_token = \"operation:",
                "native_output_0_worker_dispatch_permit_contract = \"nuis-provider-worker-dispatch-permit-v1\"",
                "native_output_0_worker_dispatch_permit_status = \"granted\"",
                "native_output_4_worker_dispatch_permit_status = \"granted\"",
                "native_output_0_worker_dispatch_status = \"1\"",
                "native_output_3_worker_dispatch_status = \"4\"",
                "native_output_4_worker_dispatch_status = \"1\"",
                "native_output_4_output_handle_ownership_token = \"glm:provider-session-output:metal:apple-silicon-gpu:0:witsage.vector.metal-bias\"",
                "native_output_4_output_handle_release_status = \"released-at-graph-close\"",
            ] {
                assert_file_contains(&provider_output_payload_path, evidence, "official galaxy carrier channel");
            }
            for (field, status) in [
                ("materialize_status", "materialized"),
                ("consume_status", "consumed"),
                ("release_status", "released"),
            ] {
                assert_file_contains(
                    &provider_output_payload_path,
                    &format!("provider_edge_transport_receipt_3_{field} = \"{status}\""),
                    "official galaxy cross-provider transport receipt transition",
                );
            }
            for field in [
                "materialize_payload_hash",
                "consume_payload_hash",
                "release_payload_hash",
            ] {
                assert_file_contains(
                    &provider_output_payload_path,
                    &format!("provider_edge_transport_receipt_3_{field} = \"0x3efcc146d99e0b55\""),
                    "official galaxy cross-provider stable receipt hash",
                );
            }
            assert_file_contains(
                &provider_output_payload_path,
                "provider_edge_transport_receipt_3_byte_length = \"16\"",
                "official galaxy cross-provider receipt byte length",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_4_request_id = \"witsage.vector.metal-bias\"",
                "official galaxy cross-provider output identity",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_4_execution_contract = \"nuis-metal-f32-bias-provider-runner-v1\"",
                "official galaxy cross-provider Metal contract",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_4_hash = \"0xe31371394cd0b1bd\"",
                "official galaxy cross-provider output hash",
            );
            assert_file_contains(
                &provider_output_payload_path,
                "native_output_4_comparison_status = \"comparison-passed\"",
                "official galaxy cross-provider comparison status",
            );
        }
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
