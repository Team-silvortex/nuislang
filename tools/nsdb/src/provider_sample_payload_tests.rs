use crate::{
    model::NsdbDeviceProviderSampleRecordInfo,
    provider_request::provider_request_from_evidence,
    provider_runner_registry::ProviderRunnerAdapter,
    provider_sample_payload::{
        pixelmagic_native_output_summary, render_real_device_provider_output_payload,
    },
};

const PIXEL_INPUT_EVIDENCE: &str = "std-preprocessed-pgm:input_bytes=20;pixel_format=gray8;pixel_width=2;pixel_height=2;pixel_stride=2;pixel_max_value=15;pixel_operation=invert;pixel_payload_path=nuis.pixelmagic.std-preprocessed.gray8.bin;pixel_payload_bytes=4;pixel_payload_hash=0x4475327f98e05411";

fn sample_record(input_evidence: &str) -> NsdbDeviceProviderSampleRecordInfo {
    NsdbDeviceProviderSampleRecordInfo {
        index: 0,
        valid: true,
        trace_id: "payload-trace:pixelmagic:0".to_owned(),
        provider: "PixelMagic".to_owned(),
        provider_family: "metal:apple-silicon-gpu".to_owned(),
        requested_runner_contract: "nuis-provider-runner-v1".to_owned(),
        requested_runner_adapter_contract: "nuis-provider-runner-adapter-v1".to_owned(),
        requested_runner_adapter_id: "metal.apple-silicon-gpu.real-device".to_owned(),
        requested_runner_adapter_capability_status: "registered-real-device".to_owned(),
        handoff_target: "device-provider-sample".to_owned(),
        sample_status: "provider-execution-ready".to_owned(),
        validation_status: "provider-execution-validated".to_owned(),
        input_evidence: input_evidence.to_owned(),
        output_evidence: "none".to_owned(),
        provider_output_payload_contract: "none".to_owned(),
        provider_output_payload_status: "none".to_owned(),
        provider_output_payload_evidence_status: "none".to_owned(),
        provider_output_payload_evidence: "none".to_owned(),
        provider_output_payload_detail: "none".to_owned(),
        provider_output_payload_next_action: "none".to_owned(),
        materialization_status: "provider-sample-materialized".to_owned(),
        materialization_detail: "test".to_owned(),
        next_action: "replay-device-sample".to_owned(),
        diagnostic: "none".to_owned(),
    }
}

#[test]
fn pixelmagic_native_output_summary_tracks_pixel_payload_bytes() {
    let summary = pixelmagic_native_output_summary(PIXEL_INPUT_EVIDENCE, "metal")
        .expect("pixelmagic output summary");
    assert_eq!(summary.kind, "pixelmagic-image-bytes");
    assert_eq!(summary.status, "deterministic-provider-output-ready");
    assert_eq!(summary.bytes, "4");
    assert!(summary.hash.starts_with("0x"));
}

#[test]
fn legacy_provider_request_rejects_inconsistent_shape() {
    let invalid = PIXEL_INPUT_EVIDENCE.replace("pixel_payload_bytes=4", "pixel_payload_bytes=3");
    assert!(provider_request_from_evidence(&invalid).is_none());
}

#[test]
fn real_device_payload_carries_pixelmagic_output_bytes() {
    let record = sample_record(PIXEL_INPUT_EVIDENCE);
    let adapter = ProviderRunnerAdapter {
        adapter_id: "metal.apple-silicon-gpu.real-device",
        capability_status: "registered-real-device",
        real_device_capable: true,
        kind: "metal-real-device-runner",
        execution_mode: "real-device-provider-runner",
    };
    let payload = render_real_device_provider_output_payload(&record, &adapter, &[]);
    assert!(payload.contains("comparison_input_kind = \"std-preprocessed-pgm\""));
    assert!(payload.contains("native_output_kind = \"pixelmagic-image-bytes\""));
    assert!(payload.contains("native_output_bytes = \"4\""));
    assert!(payload.contains("native_output_hash = \"0x"));
}
