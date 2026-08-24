use super::*;

#[test]
fn two_trace_scopes_select_independent_pixelmagic_plans() {
    let metadata = vec![
        "@scope(trace=hetero-trace:shader:metal:first)|nuis.pixelmagic:filter-plan=pixelmagic.gray8.invert-threshold".to_owned(),
        "@scope(trace=hetero-trace:shader:metal:second)|nuis.pixelmagic:filter-plan=pixelmagic.gray8.threshold-only".to_owned(),
    ];
    let first = device_sample_contract_for_trace(DeviceSampleTraceInput {
        trace_role: "backend-artifact",
        status: "trace-ready",
        domain_family: "shader",
        trace_id: "hetero-trace:shader:metal:first",
        backend_family: Some("metal"),
        target_device: Some("apple-silicon-gpu"),
        payload_format: Some("metal-library"),
        payload_path: Some("first.metallib"),
        artifact_provider_metadata: &metadata,
    });
    let second = device_sample_contract_for_trace(DeviceSampleTraceInput {
        trace_role: "backend-artifact",
        status: "trace-ready",
        domain_family: "shader",
        trace_id: "hetero-trace:shader:metal:second",
        backend_family: Some("metal"),
        target_device: Some("apple-silicon-gpu"),
        payload_format: Some("metal-library"),
        payload_path: Some("second.metallib"),
        artifact_provider_metadata: &metadata,
    });

    assert!(first
        .input_evidence
        .contains("artifact_provider_metadata_source_count=2"));
    assert!(first
        .input_evidence
        .contains("artifact_provider_metadata_count=1"));
    assert!(first
        .input_evidence
        .contains("provider_filter_plan_id=pixelmagic.gray8.invert-threshold"));
    assert!(!first
        .input_evidence
        .contains("provider_filter_plan_id=pixelmagic.gray8.threshold-only"));
    assert!(second
        .input_evidence
        .contains("provider_filter_plan_id=pixelmagic.gray8.threshold-only"));
    assert!(!second
        .input_evidence
        .contains("provider_filter_plan_id=pixelmagic.gray8.invert-threshold"));
}

#[test]
fn unscoped_metadata_remains_visible_to_every_trace() {
    let metadata = vec!["nuis.pixelmagic:filter-plan=pixelmagic.gray8.threshold-only".to_owned()];
    for trace_id in [
        "hetero-trace:shader:metal:first",
        "hetero-trace:shader:metal:second",
    ] {
        let sample = device_sample_contract_for_trace(DeviceSampleTraceInput {
            trace_role: "backend-artifact",
            status: "trace-ready",
            domain_family: "shader",
            trace_id,
            backend_family: Some("metal"),
            target_device: Some("apple-silicon-gpu"),
            payload_format: Some("metal-library"),
            payload_path: Some("shader.metallib"),
            artifact_provider_metadata: &metadata,
        });
        assert!(sample
            .input_evidence
            .contains("provider_filter_plan_id=pixelmagic.gray8.threshold-only"));
    }
}
