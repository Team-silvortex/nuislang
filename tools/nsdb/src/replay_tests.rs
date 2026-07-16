use crate::{
    model::{
        NsdbDeviceProviderSampleManifestInfo, NsdbDeviceProviderSampleRecordInfo,
        NsdbDeviceSampleHandoffRecord, NsdbDomainDebugInfo, NsdbHeteroRuntimeTraceInfo,
        NsdbHeteroRuntimeTraceRecord, NsdbInspectReport, NsdbPayloadDecoderManifestInfo,
        NsdbPayloadExecutionEvent, NsdbPayloadExecutionEventFilter,
        NsdbPayloadExecutionHandoffInfo, NsdbSidecarDebugInfo,
    },
    replay::build_replay_plan,
};
use std::{
    env, fs,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn probes_spirv_payload_magic_through_decoder_registry() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output_dir = env::temp_dir().join(format!("nsdb-replay-spirv-{nonce}"));
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(output_dir.join("shader.spv"), [0x03, 0x02, 0x23, 0x07]).unwrap();

    let decoded = crate::payload_decoder::decode_payload_content(
        output_dir.to_str().unwrap(),
        "spv",
        "shader.spv",
    );

    assert_eq!(decoded.decoder_id, "nsdb-spirv-opaque-decoder-v1");
    assert_eq!(decoded.decoder_status, "decoder-registered-opaque");
    assert_eq!(decoded.decoder_detail_level, "file-header");
    assert_eq!(decoded.decoder_format_probe_status, "format-probe-matched");
    assert_eq!(decoded.decoder_format_probe_detail, "magic:SPIR-V");

    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn loads_external_payload_decoder_manifest_specs() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output_dir = env::temp_dir().join(format!("nsdb-replay-external-decoder-{nonce}"));
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(output_dir.join("shader.wgslbin"), b"WGSLopaque").unwrap();
    fs::write(
        output_dir.join("nuis.nsdb.payload-decoders.toml"),
        r#"
protocol = "nuis-nsdb-payload-decoders-v1"
schema = "nsdb-payload-decoder-manifest-v1"

[[decoders]]
payload_format = "wgslbin"
decoder_id = "nsdb-wgslbin-external-decoder-v1"
decoder_capability = "shader-binary-header"
decoder_detail_level = "container-header"
magic_label = "WGSL"
magic_ascii = "WGSL"
"#,
    )
    .unwrap();

    let decoded = crate::payload_decoder::decode_payload_content(
        output_dir.to_str().unwrap(),
        "wgslbin",
        "shader.wgslbin",
    );

    assert_eq!(decoded.decoder_id, "nsdb-wgslbin-external-decoder-v1");
    assert_eq!(decoded.decoder_status, "decoder-registered-external-opaque");
    assert_eq!(decoded.decoder_capability, "shader-binary-header");
    assert_eq!(decoded.decoder_detail_level, "container-header");
    assert_eq!(
        decoded.decoder_manifest_status,
        "manifest-external-decoder-loaded"
    );
    assert_eq!(decoded.decoder_manifest_detail, "external-magic-ascii");
    assert_eq!(decoded.decoder_format_probe_status, "format-probe-matched");
    assert_eq!(decoded.decoder_format_probe_detail, "magic:WGSL");

    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn loads_external_payload_decoder_hex_magic_specs() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output_dir = env::temp_dir().join(format!("nsdb-replay-external-hex-decoder-{nonce}"));
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(
        output_dir.join("kernel.bin"),
        [0x03, 0x02, 0x23, 0x07, 0xaa],
    )
    .unwrap();
    fs::write(
        output_dir.join("nuis.nsdb.payload-decoders.toml"),
        r#"
protocol = "nuis-nsdb-payload-decoders-v1"
schema = "nsdb-payload-decoder-manifest-v1"

[[decoders]]
payload_format = "custom-spv"
decoder_id = "nsdb-custom-spv-external-decoder-v1"
magic_label = "SPIR-V"
magic_hex = "03 02 23 07"
"#,
    )
    .unwrap();

    let decoded = crate::payload_decoder::decode_payload_content(
        output_dir.to_str().unwrap(),
        "custom-spv",
        "kernel.bin",
    );

    assert_eq!(decoded.decoder_id, "nsdb-custom-spv-external-decoder-v1");
    assert_eq!(decoded.decoder_status, "decoder-registered-external-opaque");
    assert_eq!(
        decoded.decoder_manifest_status,
        "manifest-external-decoder-loaded"
    );
    assert_eq!(decoded.decoder_manifest_detail, "external-magic-hex");
    assert_eq!(decoded.decoder_format_probe_status, "format-probe-matched");
    assert_eq!(decoded.decoder_format_probe_detail, "magic:SPIR-V");

    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn reports_invalid_external_payload_decoder_magic_specs() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output_dir = env::temp_dir().join(format!("nsdb-replay-invalid-decoder-{nonce}"));
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(output_dir.join("kernel.bin"), [0x03, 0x02, 0x23, 0x07]).unwrap();
    fs::write(
        output_dir.join("nuis.nsdb.payload-decoders.toml"),
        r#"
protocol = "nuis-nsdb-payload-decoders-v1"
schema = "nsdb-payload-decoder-manifest-v1"

[[decoders]]
payload_format = "bad-spv"
decoder_id = "nsdb-bad-spv-external-decoder-v1"
magic_label = "BROKEN"
magic_hex = "03 0Z"
"#,
    )
    .unwrap();

    let decoded = crate::payload_decoder::decode_payload_content(
        output_dir.to_str().unwrap(),
        "bad-spv",
        "kernel.bin",
    );

    assert_eq!(decoded.decoder_id, "nsdb-bad-spv-external-decoder-v1");
    assert_eq!(
        decoded.decoder_manifest_status,
        "manifest-external-decoder-invalid-magic"
    );
    assert_eq!(decoded.decoder_manifest_detail, "invalid-magic-hex");
    assert_eq!(decoded.decoder_format_probe_status, "format-probe-generic");

    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn summarizes_payload_decoder_manifest_for_inspect_surfaces() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output_dir = env::temp_dir().join(format!("nsdb-decoder-manifest-summary-{nonce}"));
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(
        output_dir.join("nuis.nsdb.payload-decoders.toml"),
        r#"
protocol = "nuis-nsdb-payload-decoders-v1"
schema = "nsdb-payload-decoder-manifest-v1"

[[decoders]]
payload_format = "ok"
decoder_id = "nsdb-ok-decoder-v1"
magic_ascii = "OK"

[[decoders]]
payload_format = "broken"
decoder_id = "nsdb-broken-decoder-v1"
magic_hex = "0G"
"#,
    )
    .unwrap();

    let summary = crate::payload_decoder::read_payload_decoder_manifest_info(&output_dir);

    assert!(summary.available);
    assert_eq!(summary.protocol, "nuis-nsdb-payload-decoders-v1");
    assert_eq!(summary.schema, "nsdb-payload-decoder-manifest-v1");
    assert_eq!(summary.status, "invalid-records");
    assert_eq!(summary.record_count, 2);
    assert_eq!(summary.valid_record_count, 1);
    assert_eq!(summary.invalid_record_count, 1);
    assert_eq!(summary.first_payload_format, "ok");
    assert_eq!(summary.first_decoder_id, "nsdb-ok-decoder-v1");
    assert_eq!(summary.first_diagnostic, "manifest-external-decoder-loaded");
    assert_eq!(summary.records.len(), 2);
    assert!(summary.records[0].valid);
    assert_eq!(summary.records[0].payload_format, "ok");
    assert_eq!(
        summary.records[0].diagnostic,
        "manifest-external-decoder-loaded"
    );
    assert!(!summary.records[1].valid);
    assert_eq!(summary.records[1].payload_format, "broken");
    assert_eq!(
        summary.records[1].diagnostic,
        "manifest-external-decoder-invalid-magic"
    );

    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn reports_unsupported_payload_decoder_manifest_protocol() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output_dir = env::temp_dir().join(format!("nsdb-decoder-manifest-unsupported-{nonce}"));
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(
        output_dir.join("nuis.nsdb.payload-decoders.toml"),
        r#"
protocol = "nuis-nsdb-payload-decoders-v0"
schema = "nsdb-payload-decoder-manifest-v1"

[[decoders]]
payload_format = "ok"
decoder_id = "nsdb-ok-decoder-v1"
magic_ascii = "OK"
"#,
    )
    .unwrap();

    let summary = crate::payload_decoder::read_payload_decoder_manifest_info(&output_dir);

    assert!(summary.available);
    assert_eq!(summary.protocol, "nuis-nsdb-payload-decoders-v0");
    assert_eq!(summary.status, "unsupported-protocol");
    assert_eq!(summary.record_count, 1);
    assert_eq!(summary.valid_record_count, 1);

    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn summarizes_device_provider_sample_manifest_for_inspect_surfaces() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output_dir = env::temp_dir().join(format!("nsdb-provider-sample-summary-{nonce}"));
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(
        output_dir.join("nuis.nsdb.device-provider-samples.toml"),
        r#"
protocol = "nuis-device-provider-samples-v1"
schema = "nsdb-yir-device-provider-sample-v1"
status = "awaiting-provider-materialization"
record_count = 1
ready_record_count = 0
pending_record_count = 1

[[device_provider_samples]]
trace_id = "hetero-trace:shader:metal:apple-silicon-gpu"
provider = "nustar-deferred-device-sample-v1"
provider_family = "metal:apple-silicon-gpu"
handoff_target = "metal:apple-silicon-gpu"
sample_status = "pending-provider-execution"
validation_status = "pending-provider-execution"
input_evidence = "metallib:pixelmagic.metallib"
output_evidence = "not-materialized"
materialization_status = "provider-sample-pending"
materialization_detail = "awaiting-provider-runtime"
next_action = "execute-provider-sample"
"#,
    )
    .unwrap();

    let summary = crate::provider_sample::read_device_provider_sample_manifest_info(&output_dir);

    assert!(summary.available);
    assert_eq!(summary.protocol, "nuis-device-provider-samples-v1");
    assert_eq!(summary.schema, "nsdb-yir-device-provider-sample-v1");
    assert_eq!(summary.status, "awaiting-provider-materialization");
    assert_eq!(summary.record_count, 1);
    assert_eq!(summary.pending_record_count, 1);
    assert_eq!(summary.invalid_record_count, 0);
    assert_eq!(
        summary.first_trace_id,
        "hetero-trace:shader:metal:apple-silicon-gpu"
    );
    assert_eq!(summary.first_provider_family, "metal:apple-silicon-gpu");
    assert_eq!(
        summary.first_materialization_status,
        "provider-sample-pending"
    );
    assert_eq!(summary.first_diagnostic, "provider-sample-record-loaded");
    assert!(summary.records[0].valid);

    fs::remove_dir_all(output_dir).unwrap();
}

#[test]
fn builds_replay_checkpoints_from_payload_events() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output_dir = env::temp_dir().join(format!("nsdb-replay-payload-{nonce}"));
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(output_dir.join("pixelmagic.metallib"), b"MTLBpayload").unwrap();
    fs::write(
        output_dir.join("nuis.nsdb.device-provider-samples.toml"),
        r#"
protocol = "nuis-device-provider-samples-v1"
schema = "nsdb-yir-device-provider-sample-v1"
source = "run-artifact-provider-sample-manifest"
status = "awaiting-provider-materialization"
record_count = 1
ready_record_count = 0
pending_record_count = 1

[[device_provider_samples]]
trace_id = "hetero-trace:shader:metal:apple-silicon-gpu"
provider = "nustar-deferred-device-sample-v1"
provider_family = "metal:apple-silicon-gpu"
handoff_target = "metal:apple-silicon-gpu"
sample_status = "pending-provider-execution"
validation_status = "pending-provider-execution"
input_evidence = "metallib:pixelmagic.metallib"
output_evidence = "not-materialized"
materialization_status = "provider-sample-pending"
materialization_detail = "awaiting-provider-runtime"
next_action = "execute-provider-sample"
"#,
    )
    .unwrap();

    let mut report = NsdbInspectReport {
        manifest: "manifest.toml".to_owned(),
        output_dir: output_dir.display().to_string(),
        debug_model: "yir-metadata".to_owned(),
        native_debugger_visibility: "host-shell-only".to_owned(),
        nsdb_visibility: "domains+clock+segments+lowering-units".to_owned(),
        debug_readiness: "metadata-partial".to_owned(),
        yir_debuggable: false,
        domain_count: 0,
        hetero_domain_count: 0,
        clock_edge_count: 0,
        data_segment_count: 0,
        lowering_unit_count: 0,
        sidecar_count: 0,
        payload_execution_event_filter: NsdbPayloadExecutionEventFilter::default(),
        payload_execution_handoff: NsdbPayloadExecutionHandoffInfo {
            available: true,
            path: "nuis.nsdb.payload-execution-handoff.toml".to_owned(),
            protocol: "nuis-nsdb-payload-execution-handoff-v1".to_owned(),
            debugger_contract: "nsdb-yir-payload-execution-trace-v1".to_owned(),
            status: "ready".to_owned(),
            record_count: 2,
            ready_record_count: 1,
            first_trace_id: "payload-trace:container-loader:nuis.bootstrap.lifecycle.v1".to_owned(),
            first_status: "ready".to_owned(),
            first_next_action: "handoff-payload-trace-to-nsdb".to_owned(),
            first_entry_symbol: "nuis.bootstrap.lifecycle.v1".to_owned(),
            first_execution_phase: "container-loader-handoff".to_owned(),
            events: vec![
                NsdbPayloadExecutionEvent {
                    index: 0,
                    trace_id: "payload-trace:container-loader:nuis.bootstrap.lifecycle.v1"
                        .to_owned(),
                    status: "ready".to_owned(),
                    execution_phase: "container-loader-handoff".to_owned(),
                    target: "container-loader".to_owned(),
                    entry_symbol: "nuis.bootstrap.lifecycle.v1".to_owned(),
                    entry_kind: "lifecycle-bootstrap".to_owned(),
                    entry_section_id: "sec0000.compiled-artifact".to_owned(),
                    first_blocker: "none".to_owned(),
                    next_action: "handoff-payload-trace-to-nsdb".to_owned(),
                },
                NsdbPayloadExecutionEvent {
                    index: 1,
                    trace_id: "payload-trace:shader:pixelmagic.blur".to_owned(),
                    status: "blocked".to_owned(),
                    execution_phase: "device-dispatch".to_owned(),
                    target: "shader".to_owned(),
                    entry_symbol: "pixelmagic.blur".to_owned(),
                    entry_kind: "shader-kernel".to_owned(),
                    entry_section_id: "sec0002.shader".to_owned(),
                    first_blocker: "device-execution-sample-missing".to_owned(),
                    next_action: "materialize-device-execution-trace".to_owned(),
                },
            ],
        },
        hetero_runtime_trace: NsdbHeteroRuntimeTraceInfo {
            available: true,
            path: "nuis.nsdb.hetero-runtime-trace.toml".to_owned(),
            protocol: "nuis-nsdb-hetero-runtime-trace-v1".to_owned(),
            debugger_contract: "nsdb-yir-hetero-runtime-trace-v1".to_owned(),
            status: "execution-pending".to_owned(),
            record_count: 1,
            ready_record_count: 0,
            backend_execution_record_count: 1,
            device_sample_handoff_record_count: 1,
            device_sample_handoff_protocol: "nuis-device-sample-provider-handoff-v1".to_owned(),
            first_trace_id: "hetero-trace:shader:metal:apple-silicon-gpu".to_owned(),
            first_blocker: "none".to_owned(),
            next_action: "materialize-device-execution-trace".to_owned(),
            records: vec![NsdbHeteroRuntimeTraceRecord {
                index: 0,
                trace_id: "hetero-trace:shader:metal:apple-silicon-gpu".to_owned(),
                trace_role: "backend-artifact".to_owned(),
                status: "execution-pending".to_owned(),
                domain_family: "shader".to_owned(),
                backend_family: "metal".to_owned(),
                target_device: "apple-silicon-gpu".to_owned(),
                backend_artifact_key: "shader:metal:apple-silicon-gpu".to_owned(),
                selected_lowering_target: "metal".to_owned(),
                payload_format: "metallib".to_owned(),
                payload_path: "pixelmagic.metallib".to_owned(),
                bridge_stub_path: "pixelmagic.bridge".to_owned(),
                missing_signals: Vec::new(),
                next_action: "materialize-device-execution-trace".to_owned(),
            }],
            device_sample_handoffs: vec![NsdbDeviceSampleHandoffRecord {
                index: 0,
                trace_id: "hetero-trace:shader:metal:apple-silicon-gpu".to_owned(),
                protocol: "nuis-device-sample-provider-handoff-v1".to_owned(),
                provider: "nustar-deferred-device-sample-v1".to_owned(),
                provider_family: "metal:apple-silicon-gpu".to_owned(),
                handoff_target: "metal:apple-silicon-gpu".to_owned(),
                handoff_status: "awaiting-provider-handoff".to_owned(),
                validation_status: "pending-provider-execution".to_owned(),
                input_evidence: "metallib:pixelmagic.metallib".to_owned(),
                output_evidence: "not-materialized".to_owned(),
                next_action: "materialize-device-execution-sample".to_owned(),
            }],
        },
        payload_decoder_manifest: NsdbPayloadDecoderManifestInfo {
            available: false,
            path: output_dir
                .join("nuis.nsdb.payload-decoders.toml")
                .display()
                .to_string(),
            protocol: "none".to_owned(),
            schema: "none".to_owned(),
            status: "missing".to_owned(),
            record_count: 0,
            valid_record_count: 0,
            invalid_record_count: 0,
            first_payload_format: "none".to_owned(),
            first_decoder_id: "none".to_owned(),
            first_diagnostic: "manifest-not-found".to_owned(),
            records: Vec::new(),
        },
        device_provider_sample_manifest: NsdbDeviceProviderSampleManifestInfo {
            available: true,
            path: output_dir
                .join("nuis.nsdb.device-provider-samples.toml")
                .display()
                .to_string(),
            protocol: "nuis-device-provider-samples-v1".to_owned(),
            schema: "nsdb-yir-device-provider-sample-v1".to_owned(),
            status: "awaiting-provider-materialization".to_owned(),
            record_count: 1,
            ready_record_count: 0,
            pending_record_count: 1,
            invalid_record_count: 0,
            first_trace_id: "hetero-trace:shader:metal:apple-silicon-gpu".to_owned(),
            first_provider_family: "metal:apple-silicon-gpu".to_owned(),
            first_materialization_status: "provider-sample-pending".to_owned(),
            first_diagnostic: "provider-sample-record-loaded".to_owned(),
            records: vec![NsdbDeviceProviderSampleRecordInfo {
                index: 0,
                valid: true,
                trace_id: "hetero-trace:shader:metal:apple-silicon-gpu".to_owned(),
                provider: "nustar-deferred-device-sample-v1".to_owned(),
                provider_family: "metal:apple-silicon-gpu".to_owned(),
                requested_runner_contract: "nuis-provider-runner-v1".to_owned(),
                requested_runner_adapter_contract: "nuis-provider-runner-adapter-v1".to_owned(),
                requested_runner_adapter_id: "metal.apple-silicon-gpu.host-simulated".to_owned(),
                requested_runner_adapter_capability_status: "registered-host-simulated".to_owned(),
                handoff_target: "metal:apple-silicon-gpu".to_owned(),
                sample_status: "pending-provider-execution".to_owned(),
                validation_status: "pending-provider-execution".to_owned(),
                input_evidence: "metallib:pixelmagic.metallib".to_owned(),
                output_evidence: "not-materialized".to_owned(),
                materialization_status: "provider-sample-pending".to_owned(),
                materialization_detail: "awaiting-provider-runtime".to_owned(),
                next_action: "execute-provider-sample".to_owned(),
                diagnostic: "provider-sample-record-loaded".to_owned(),
            }],
        },
        domains: vec![NsdbDomainDebugInfo {
            domain_family: "shader".to_owned(),
            package_id: "pixelmagic".to_owned(),
            kind: "heterogeneous".to_owned(),
            lowering_target: "metal".to_owned(),
            backend_family: "metal".to_owned(),
            debug_scope: "yir-domain".to_owned(),
        }],
        clock_edges: Vec::new(),
        data_segments: Vec::new(),
        lowering_units: Vec::new(),
        sidecars: vec![NsdbSidecarDebugInfo {
            domain_family: "shader".to_owned(),
            package_id: "pixelmagic".to_owned(),
            path: "pixelmagic.shader.sidecar.json".to_owned(),
            schema: "nuis-yir-sidecar-v1".to_owned(),
            capability_owner: "shader".to_owned(),
            frontend_ir: "nuis-yir.shader".to_owned(),
            native_ir: "msl2.4".to_owned(),
            pipeline_lowering: "metal-compute-pipeline".to_owned(),
            resource_lowering: "metal-buffer".to_owned(),
            dispatch_lowering: "metal-dispatch-threadgroups".to_owned(),
            texture_lowering: "metal-texture".to_owned(),
            transport_lowering: "none".to_owned(),
            tensor_lowering: "none".to_owned(),
            memory_lowering: "metal-shared-buffer".to_owned(),
            result_lowering: "metal-buffer-readback".to_owned(),
            validation_contracts: vec!["shader-yir-contract".to_owned()],
            entry_symbol: "pixelmagic.blur".to_owned(),
            stage_kind: "compute".to_owned(),
        }],
        missing_metadata: Vec::new(),
    };

    let inspect_json = crate::json::nsdb_inspect_report_json(&report);
    assert!(inspect_json.contains("\"hetero_runtime_trace_device_sample_handoff_record_count\":1"));
    assert!(inspect_json.contains(
        "\"hetero_runtime_trace_device_sample_handoff_protocol\":\"nuis-device-sample-provider-handoff-v1\""
    ));
    assert!(inspect_json.contains("\"device_sample_handoffs\":[{"));
    assert!(inspect_json.contains("\"provider_family\":\"metal:apple-silicon-gpu\""));
    assert!(inspect_json.contains("\"handoff_status\":\"awaiting-provider-handoff\""));
    assert!(inspect_json.contains("\"device_provider_sample_manifest_available\":true"));
    assert!(inspect_json.contains(
        "\"device_provider_sample_manifest_status\":\"awaiting-provider-materialization\""
    ));
    assert!(inspect_json.contains("\"device_provider_sample_manifest_record_count\":1"));
    assert!(inspect_json.contains("\"device_provider_sample_manifest_pending_record_count\":1"));
    assert!(inspect_json.contains(
        "\"device_provider_sample_manifest_first_materialization_status\":\"provider-sample-pending\""
    ));
    assert!(inspect_json.contains("\"device_provider_sample_manifest_records\":[{"));

    let events_json = crate::json::nsdb_events_report_json(&report);
    assert!(events_json.contains(
        "\"payload_execution_event_query_contract\":\"nsdb-payload-execution-event-query-v1\""
    ));
    assert!(events_json
        .contains("\"payload_execution_event_source\":\"payload-execution-handoff-events\""));
    assert!(events_json.contains(
        "\"payload_execution_event_source_protocol\":\"nuis-nsdb-payload-execution-handoff-v1\""
    ));
    assert!(events_json.contains(
        "\"payload_execution_event_source_debugger_contract\":\"nsdb-yir-payload-execution-trace-v1\""
    ));
    assert!(events_json.contains("\"payload_execution_event_query_result_count\":2"));
    assert!(events_json.contains("\"payload_execution_events\":[{"));

    let plan = build_replay_plan(&report);

    assert_eq!(plan.protocol, "nsdb-payload-execution-replay-plan-v1");
    assert_eq!(plan.status, "blocked");
    assert_eq!(plan.checkpoint_count, 2);
    assert_eq!(plan.replayable_checkpoint_count, 1);
    assert_eq!(plan.checkpoints[0].checkpoint_kind, "loader-checkpoint");
    assert_eq!(plan.checkpoints[0].replay_status, "replayable");
    assert_eq!(plan.checkpoints[0].frame_id, "frame:payload:0:loader");
    assert_eq!(
        plan.checkpoints[0].value_snapshot_status,
        "snapshot-metadata-only"
    );
    assert_eq!(
        plan.checkpoints[0].value_content_status,
        "content-metadata-summary"
    );
    assert_eq!(
        plan.checkpoints[0].value_decoder_capability,
        "metadata-summary"
    );
    assert_eq!(
        plan.checkpoints[0].value_decoder_detail_level,
        "semantic-metadata"
    );
    assert!(!plan.checkpoints[0].value_decoder_reads_file_summary);
    assert_eq!(
        plan.checkpoints[1].checkpoint_kind,
        "device-dispatch-checkpoint"
    );
    assert_eq!(
        plan.checkpoints[1].value_sample_resolution_status,
        "provider-sample-observed"
    );
    assert!(plan.checkpoints[1]
        .value_sample_resolution_detail
        .contains("device-provider-sample:hetero-trace:shader:metal:apple-silicon-gpu"));
    assert_eq!(
        plan.checkpoints[1].value_sample_materialization_status,
        "provider-sample-pending"
    );
    assert!(plan.checkpoints[1]
        .value_sample_materialization_detail
        .contains("provider-sample:metal:apple-silicon-gpu"));
    assert_eq!(plan.checkpoints[1].value_sample_payload_format, "metallib");
    assert_eq!(
        plan.checkpoints[1].value_schema_hint,
        "opaque-runtime-payload:metallib"
    );
    assert_eq!(
        plan.checkpoints[1].value_snapshot_status,
        "snapshot-opaque-payload"
    );
    assert_eq!(plan.checkpoints[1].value_snapshot_type, "metallib");
    assert_eq!(
        plan.checkpoints[1].value_snapshot_ref,
        "pixelmagic.metallib"
    );
    assert_eq!(
        plan.checkpoints[1].value_content_status,
        "content-opaque-file-summary"
    );
    assert_eq!(
        plan.checkpoints[1].value_decoder_id,
        "nsdb-metallib-opaque-decoder-v1"
    );
    assert_eq!(
        plan.checkpoints[1].value_decoder_status,
        "decoder-registered-opaque"
    );
    assert_eq!(
        plan.checkpoints[1].value_decoder_capability,
        "opaque-file-summary"
    );
    assert_eq!(
        plan.checkpoints[1].value_decoder_detail_level,
        "file-header"
    );
    assert!(plan.checkpoints[1].value_decoder_reads_file_summary);
    assert_eq!(
        plan.checkpoints[1].value_decoder_format_probe_status,
        "format-probe-matched"
    );
    assert_eq!(
        plan.checkpoints[1].value_decoder_format_probe_detail,
        "magic:MTLB"
    );
    assert!(plan.checkpoints[1]
        .value_content_summary
        .contains("bytes=11"));
    assert_eq!(
        plan.first_blocker.as_deref(),
        Some("device-execution-sample-missing")
    );

    let replay_json = crate::json::nsdb_replay_plan_json(&report);
    assert!(replay_json
        .contains("\"replay_event_query_contract\":\"nsdb-payload-execution-event-query-v1\""));
    assert!(
        replay_json.contains("\"replay_checkpoint_source\":\"payload-execution-handoff-events\"")
    );
    assert!(replay_json
        .contains("\"replay_event_source_protocol\":\"nuis-nsdb-payload-execution-handoff-v1\""));
    assert!(replay_json.contains(
        "\"replay_event_source_debugger_contract\":\"nsdb-yir-payload-execution-trace-v1\""
    ));
    assert!(replay_json.contains("\"replay_event_query_result_count\":2"));
    assert!(replay_json.contains("\"replay_checkpoint_count\":2"));

    let materialize_report =
        crate::provider_sample_materialize::materialize_provider_samples(&output_dir, None)
            .unwrap();
    assert_eq!(
        materialize_report.provider_families,
        vec!["metal:apple-silicon-gpu"]
    );
    assert_eq!(materialize_report.matched_record_count, 1);
    assert_eq!(materialize_report.materialized_record_count, 1);

    report.device_provider_sample_manifest =
        crate::provider_sample::read_device_provider_sample_manifest_info(&output_dir);

    let ready_plan = build_replay_plan(&report);

    assert_eq!(ready_plan.status, "ready");
    assert_eq!(ready_plan.replayable_checkpoint_count, 2);
    assert!(ready_plan.first_blocker.is_none());
    assert_eq!(ready_plan.checkpoints[1].replay_status, "replayable");
    assert!(ready_plan.checkpoints[1].first_blocker.is_none());
    assert_eq!(
        ready_plan.checkpoints[1].value_sample_materialization_status,
        "provider-sample-materialized"
    );
    assert!(ready_plan.checkpoints[1]
        .value_sample_materialization_detail
        .contains("deterministic-provider-sample-artifact"));

    fs::remove_dir_all(output_dir).unwrap();
}
