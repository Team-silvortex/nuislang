use crate::{
    provider_sample_execute::ProviderSampleExecuteReport,
    provider_sample_materialize::ProviderSampleMaterializeReport,
};

pub(crate) fn provider_sample_materialize_json(report: &ProviderSampleMaterializeReport) -> String {
    format!(
        "{{\"tool\":\"nsdb\",\"kind\":\"device_provider_sample_materialize\",\"status\":\"{}\",\"path\":\"{}\",\"provider_family_filter\":{},\"provider_families\":{},\"record_count\":{},\"matched_record_count\":{},\"provider_bundle_registry_contract\":\"{}\",\"provider_bundle_manifest_contract\":\"{}\",\"provider_bundle_manifest_hash\":\"{}\",\"provider_bundle_manifest_entry_count\":{},\"first_provider_bundle_package_id\":\"{}\",\"first_provider_bundle_id\":\"{}\",\"selected_provider_bundle_set_contract\":\"{}\",\"selected_provider_bundle_count\":{},\"selected_provider_bundle_set_hash\":\"{}\",\"materialized_record_count\":{},\"skipped_record_count\":{},\"first_provider_family\":\"{}\",\"first_provider_runner_contract\":\"{}\",\"first_provider_runner_adapter_contract\":\"{}\",\"first_provider_runner_adapter_id\":\"{}\",\"first_provider_runner_adapter_capability_status\":\"{}\",\"first_provider_runner_registry_protocol\":\"{}\",\"first_provider_runner_registry_source\":\"{}\",\"first_provider_runner_real_device_capable\":{},\"first_provider_runner_kind\":\"{}\",\"first_provider_execution_mode\":\"{}\",\"first_provider_execution_comparison_contract\":\"{}\",\"first_provider_execution_comparison_status\":\"{}\",\"first_provider_execution_evidence_status\":\"{}\",\"first_provider_output_payload_contract\":\"{}\",\"first_provider_output_payload_status\":\"{}\",\"first_provider_output_payload_evidence_status\":\"{}\",\"first_provider_output_payload_evidence\":\"{}\",\"first_provider_output_payload_detail\":\"{}\",\"first_provider_output_payload_path\":\"{}\",\"first_provider_output_payload_hash\":\"{}\",\"first_provider_output_payload_attach_status\":\"{}\",\"first_output_evidence\":\"{}\",\"next_action\":\"{}\",\"next_command\":\"{}\",\"return_contract\":\"{}\",\"return_action\":\"{}\",\"return_command\":\"{}\",\"final_output_replay_contract\":\"{}\"}}",
        json_escape(&report.status),
        json_escape(&report.path),
        json_optional_string(report.provider_family_filter.as_deref()),
        json_string_array(&report.provider_families),
        report.record_count,
        report.matched_record_count,
        json_escape(&report.provider_bundle_registry_contract),
        json_escape(&report.provider_bundle_manifest_contract),
        json_escape(&report.provider_bundle_manifest_hash),
        report.provider_bundle_manifest_entry_count,
        json_escape(&report.first_provider_bundle_package_id),
        json_escape(&report.first_provider_bundle_id),
        json_escape(&report.selected_provider_bundle_set_contract),
        report.selected_provider_bundle_count,
        json_escape(&report.selected_provider_bundle_set_hash),
        report.materialized_record_count,
        report.skipped_record_count,
        json_escape(&report.first_provider_family),
        json_escape(&report.first_provider_runner_contract),
        json_escape(&report.first_provider_runner_adapter_contract),
        json_escape(&report.first_provider_runner_adapter_id),
        json_escape(&report.first_provider_runner_adapter_capability_status),
        json_escape(&report.first_provider_runner_registry_protocol),
        json_escape(&report.first_provider_runner_registry_source),
        report.first_provider_runner_real_device_capable,
        json_escape(&report.first_provider_runner_kind),
        json_escape(&report.first_provider_execution_mode),
        json_escape(&report.first_provider_execution_comparison_contract),
        json_escape(&report.first_provider_execution_comparison_status),
        json_escape(&report.first_provider_execution_evidence_status),
        json_escape(&report.first_provider_output_payload_contract),
        json_escape(&report.first_provider_output_payload_status),
        json_escape(&report.first_provider_output_payload_evidence_status),
        json_escape(&report.first_provider_output_payload_evidence),
        json_escape(&report.first_provider_output_payload_detail),
        json_escape(&report.first_provider_output_payload_path),
        json_escape(&report.first_provider_output_payload_hash),
        json_escape(&report.first_provider_output_payload_attach_status),
        json_escape(&report.first_output_evidence),
        json_escape(&report.next_action),
        json_escape(&report.next_command),
        json_escape(&report.return_contract),
        json_escape(&report.return_action),
        json_escape(&report.return_command),
        json_escape(&report.final_output_replay_contract),
    )
}

pub(crate) fn provider_sample_execute_json(report: &ProviderSampleExecuteReport) -> String {
    format!(
        "{{\"tool\":\"nsdb\",\"kind\":\"device_provider_sample_execute\",\"status\":\"{}\",\"provider_family_filter\":{},\"provider_families\":{},\"record_count\":{},\"matched_record_count\":{},\"executable_record_count\":{},\"output_payload_count\":{},\"first_provider_family\":\"{}\",\"provider_bundle_registry_contract\":\"{}\",\"provider_bundle_manifest_contract\":\"{}\",\"provider_bundle_manifest_hash\":\"{}\",\"provider_bundle_manifest_entry_count\":{},\"first_provider_bundle_package_id\":\"{}\",\"first_provider_bundle_id\":\"{}\",\"selected_provider_bundle_set_contract\":\"{}\",\"selected_provider_bundle_count\":{},\"selected_provider_bundle_set_hash\":\"{}\",\"final_image_dispatch_authority_status\":\"{}\",\"final_image_dispatch_image_path\":\"{}\",\"final_image_dispatch_count\":{},\"final_image_dispatch_matched_count\":{},\"final_image_dispatch_table_hash\":\"{}\",\"final_image_dispatch_selected_set_hash\":\"{}\",\"first_provider_runner_adapter_id\":\"{}\",\"first_provider_runner_adapter_capability_status\":\"{}\",\"first_provider_runner_real_device_capable\":{},\"first_provider_runner_real_device_probe_status\":\"{}\",\"first_provider_execution_mode\":\"{}\",\"first_output_payload_evidence\":\"{}\",\"first_output_payload_comparison_contract\":\"{}\",\"first_output_payload_comparison_status\":\"{}\",\"first_output_payload_input_evidence\":\"{}\",\"first_output_payload_input_evidence_hash\":\"{}\",\"first_output_payload_native_output_kind\":\"{}\",\"first_output_payload_native_output_status\":\"{}\",\"first_output_payload_native_output_bytes\":\"{}\",\"first_output_payload_native_output_hash\":\"{}\",\"first_output_payload_native_execution_contract\":\"{}\",\"first_output_payload_native_execution_status\":\"{}\",\"first_output_payload_native_device\":\"{}\",\"first_output_payload_native_compute_plan_contract\":\"{}\",\"first_output_payload_native_compute_plan_status\":\"{}\",\"first_output_payload_native_compute_plan_layer_count\":\"{}\",\"first_output_payload_native_compute_plan_preferred_devices\":\"{}\",\"first_output_payload_native_compute_plan_supported_devices\":\"{}\",\"next_action\":\"{}\",\"next_command\":\"{}\"}}",
        json_escape(&report.status),
        json_optional_string(report.provider_family_filter.as_deref()),
        json_string_array(&report.provider_families),
        report.record_count,
        report.matched_record_count,
        report.executable_record_count,
        report.output_payload_count,
        json_escape(&report.first_provider_family),
        json_escape(&report.provider_bundle_registry_contract),
        json_escape(&report.provider_bundle_manifest_contract),
        json_escape(&report.provider_bundle_manifest_hash),
        report.provider_bundle_manifest_entry_count,
        json_escape(&report.first_provider_bundle_package_id),
        json_escape(&report.first_provider_bundle_id),
        json_escape(&report.selected_provider_bundle_set_contract),
        report.selected_provider_bundle_count,
        json_escape(&report.selected_provider_bundle_set_hash),
        json_escape(&report.final_image_dispatch_authority_status),
        json_escape(&report.final_image_dispatch_image_path),
        report.final_image_dispatch_count,
        report.final_image_dispatch_matched_count,
        json_escape(&report.final_image_dispatch_table_hash),
        json_escape(&report.final_image_dispatch_selected_set_hash),
        json_escape(&report.first_provider_runner_adapter_id),
        json_escape(&report.first_provider_runner_adapter_capability_status),
        report.first_provider_runner_real_device_capable,
        json_escape(&report.first_provider_runner_real_device_probe_status),
        json_escape(&report.first_provider_execution_mode),
        json_escape(&report.first_output_payload_evidence),
        json_escape(&report.first_output_payload_comparison_contract),
        json_escape(&report.first_output_payload_comparison_status),
        json_escape(&report.first_output_payload_input_evidence),
        json_escape(&report.first_output_payload_input_evidence_hash),
        json_escape(&report.first_output_payload_native_output_kind),
        json_escape(&report.first_output_payload_native_output_status),
        json_escape(&report.first_output_payload_native_output_bytes),
        json_escape(&report.first_output_payload_native_output_hash),
        json_escape(&report.first_output_payload_native_execution_contract),
        json_escape(&report.first_output_payload_native_execution_status),
        json_escape(&report.first_output_payload_native_device),
        json_escape(&report.first_output_payload_native_compute_plan_contract),
        json_escape(&report.first_output_payload_native_compute_plan_status),
        json_escape(&report.first_output_payload_native_compute_plan_layer_count),
        json_escape(&report.first_output_payload_native_compute_plan_preferred_devices),
        json_escape(&report.first_output_payload_native_compute_plan_supported_devices),
        json_escape(&report.next_action),
        json_escape(&report.next_command),
    )
}

pub(crate) fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn json_optional_string(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", json_escape(value)))
        .unwrap_or_else(|| "null".to_owned())
}

fn json_string_array(values: &[String]) -> String {
    let values = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_provider_samples_json_exposes_runner_boundary() {
        let report = ProviderSampleExecuteReport {
            status: "provider-output-payloads-ready".to_owned(),
            provider_family_filter: Some("coreml:apple-ane".to_owned()),
            provider_families: vec!["coreml:apple-ane".to_owned()],
            record_count: 1,
            matched_record_count: 1,
            executable_record_count: 1,
            output_payload_count: 1,
            first_provider_family: "coreml:apple-ane".to_owned(),
            provider_bundle_registry_contract: "nuis-provider-bundle-registry-v1".to_owned(),
            provider_bundle_manifest_contract: "nuis-provider-bundle-manifest-v1".to_owned(),
            provider_bundle_manifest_hash: "fnv1a64:08a971e5a543be2e".to_owned(),
            provider_bundle_manifest_entry_count: 3,
            first_provider_bundle_package_id: "official.kernel".to_owned(),
            first_provider_bundle_id: "coreml.apple-ane.bundle.v1".to_owned(),
            selected_provider_bundle_set_contract: "nuis-selected-provider-bundle-set-v1"
                .to_owned(),
            selected_provider_bundle_count: 1,
            selected_provider_bundle_set_hash: "fnv1a64:e9a82b052c861b93".to_owned(),
            final_image_dispatch_authority_status: "verified".to_owned(),
            final_image_dispatch_image_path: "out/nuis-app.nsb".to_owned(),
            final_image_dispatch_count: 1,
            final_image_dispatch_matched_count: 1,
            final_image_dispatch_table_hash: "0x1234567890abcdef".to_owned(),
            final_image_dispatch_selected_set_hash: "fnv1a64:e9a82b052c861b93".to_owned(),
            first_provider_runner_adapter_id: "coreml.apple-ane.real-device".to_owned(),
            first_provider_runner_adapter_capability_status: "registered-real-device".to_owned(),
            first_provider_runner_real_device_capable: true,
            first_provider_runner_real_device_probe_status: "real-device-candidate-available"
                .to_owned(),
            first_provider_execution_mode: "real-device-provider-runner".to_owned(),
            first_output_payload_evidence:
                "nuis.nsdb.provider-output.coreml-apple-ane.toml:hash=0x1:status=written".to_owned(),
            first_output_payload_comparison_contract: "nuis-provider-execution-comparison-v1"
                .to_owned(),
            first_output_payload_comparison_status: "ready-for-comparison".to_owned(),
            first_output_payload_input_evidence: "tensor:shape=1x4".to_owned(),
            first_output_payload_input_evidence_hash: "0x1234".to_owned(),
            first_output_payload_native_output_kind: "none".to_owned(),
            first_output_payload_native_output_status: "none".to_owned(),
            first_output_payload_native_output_bytes: "none".to_owned(),
            first_output_payload_native_output_hash: "none".to_owned(),
            first_output_payload_native_execution_contract: "none".to_owned(),
            first_output_payload_native_execution_status: "none".to_owned(),
            first_output_payload_native_device: "none".to_owned(),
            first_output_payload_native_compute_plan_contract: "none".to_owned(),
            first_output_payload_native_compute_plan_status: "none".to_owned(),
            first_output_payload_native_compute_plan_layer_count: "0".to_owned(),
            first_output_payload_native_compute_plan_preferred_devices: "none".to_owned(),
            first_output_payload_native_compute_plan_supported_devices: "none".to_owned(),
            next_action: "materialize-provider-samples".to_owned(),
            next_command: "nsdb materialize-provider-samples out --json".to_owned(),
        };

        let json = provider_sample_execute_json(&report);
        assert!(json.contains("\"matched_record_count\":1"));
        assert!(json.contains(
            "\"provider_bundle_registry_contract\":\"nuis-provider-bundle-registry-v1\""
        ));
        assert!(json.contains(
            "\"provider_bundle_manifest_contract\":\"nuis-provider-bundle-manifest-v1\""
        ));
        assert!(json.contains("\"provider_bundle_manifest_entry_count\":3"));
        assert!(json.contains("\"first_provider_bundle_package_id\":\"official.kernel\""));
        assert!(json.contains("\"first_provider_bundle_id\":\"coreml.apple-ane.bundle.v1\""));
        assert!(json.contains(
            "\"selected_provider_bundle_set_contract\":\"nuis-selected-provider-bundle-set-v1\""
        ));
        assert!(json.contains("\"selected_provider_bundle_count\":1"));
        assert!(json.contains("\"selected_provider_bundle_set_hash\":\"fnv1a64:e9a82b052c861b93\""));
        assert!(json.contains("\"first_provider_family\":\"coreml:apple-ane\""));
        assert!(
            json.contains("\"first_provider_runner_adapter_id\":\"coreml.apple-ane.real-device\"")
        );
        assert!(json.contains(
            "\"first_provider_runner_adapter_capability_status\":\"registered-real-device\""
        ));
        assert!(json.contains("\"first_provider_runner_real_device_capable\":true"));
        assert!(json.contains(
            "\"first_provider_runner_real_device_probe_status\":\"real-device-candidate-available\""
        ));
        assert!(json.contains("\"first_provider_execution_mode\":\"real-device-provider-runner\""));
        assert!(json.contains(
            "\"first_output_payload_comparison_contract\":\"nuis-provider-execution-comparison-v1\""
        ));
        assert!(
            json.contains("\"first_output_payload_comparison_status\":\"ready-for-comparison\"")
        );
        assert!(json.contains("\"first_output_payload_input_evidence\":\"tensor:shape=1x4\""));
        assert!(json.contains("\"first_output_payload_input_evidence_hash\":\"0x1234\""));
        assert!(json.contains("\"first_output_payload_native_output_kind\":\"none\""));
        assert!(json.contains("\"first_output_payload_native_output_hash\":\"none\""));
        assert!(json.contains("\"first_output_payload_native_execution_contract\":\"none\""));
        assert!(json.contains("\"first_output_payload_native_execution_status\":\"none\""));
        assert!(json.contains("\"first_output_payload_native_device\":\"none\""));
        assert!(json.contains("\"first_output_payload_native_compute_plan_status\":\"none\""));
    }
}
