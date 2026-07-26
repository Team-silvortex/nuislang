use crate::workflow::NsldFinalExecutableOutputBoundarySummary;

#[derive(Clone)]
pub(crate) struct ProviderBundleClosureMirror {
    pub(crate) registry_contract: String,
    pub(crate) manifest_contract: String,
    pub(crate) manifest_hash: String,
    pub(crate) manifest_entry_count: usize,
    pub(crate) first_package_id: String,
    pub(crate) first_bundle_id: String,
    pub(crate) evidence_status: String,
    pub(crate) selected_set_contract: String,
    pub(crate) selected_count: usize,
    pub(crate) selected_set_hash: String,
    pub(crate) selected_set_validation_status: String,
}

impl ProviderBundleClosureMirror {
    pub(crate) fn from_final_output(
        final_output: &NsldFinalExecutableOutputBoundarySummary,
    ) -> Option<Self> {
        final_output
            .device_provider_sample_manifest_available
            .then(|| Self {
                registry_contract: final_output
                    .device_provider_sample_provider_bundle_registry_contract
                    .clone(),
                manifest_contract: final_output
                    .device_provider_sample_provider_bundle_manifest_contract
                    .clone(),
                manifest_hash: final_output
                    .device_provider_sample_provider_bundle_manifest_hash
                    .clone(),
                manifest_entry_count: final_output
                    .device_provider_sample_provider_bundle_manifest_entry_count,
                first_package_id: final_output
                    .device_provider_sample_manifest_first_provider_bundle_package_id
                    .clone(),
                first_bundle_id: final_output
                    .device_provider_sample_manifest_first_provider_bundle_id
                    .clone(),
                evidence_status: final_output
                    .device_provider_sample_provider_bundle_evidence_status
                    .clone(),
                selected_set_contract: final_output
                    .device_provider_sample_selected_provider_bundle_set_contract
                    .clone(),
                selected_count: final_output.device_provider_sample_selected_provider_bundle_count,
                selected_set_hash: final_output
                    .device_provider_sample_selected_provider_bundle_set_hash
                    .clone(),
                selected_set_validation_status: final_output
                    .device_provider_sample_selected_provider_bundle_set_validation_status
                    .clone(),
            })
    }

    pub(crate) fn json_fields(mirror: Option<&Self>) -> Vec<String> {
        vec![
            crate::json_optional_string_field(
                "closure_summary_object_package_provider_bundle_registry_contract",
                mirror.map(|value| value.registry_contract.as_str()),
            ),
            crate::json_optional_string_field(
                "closure_summary_object_package_provider_bundle_manifest_contract",
                mirror.map(|value| value.manifest_contract.as_str()),
            ),
            crate::json_optional_string_field(
                "closure_summary_object_package_provider_bundle_manifest_hash",
                mirror.map(|value| value.manifest_hash.as_str()),
            ),
            optional_usize_field(
                "closure_summary_object_package_provider_bundle_manifest_entry_count",
                mirror.map(|value| value.manifest_entry_count),
            ),
            crate::json_optional_string_field(
                "closure_summary_object_package_first_provider_bundle_package_id",
                mirror.map(|value| value.first_package_id.as_str()),
            ),
            crate::json_optional_string_field(
                "closure_summary_object_package_first_provider_bundle_id",
                mirror.map(|value| value.first_bundle_id.as_str()),
            ),
            crate::json_optional_string_field(
                "closure_summary_object_package_provider_bundle_evidence_status",
                mirror.map(|value| value.evidence_status.as_str()),
            ),
            crate::json_optional_string_field(
                "closure_summary_object_package_selected_provider_bundle_set_contract",
                mirror.map(|value| value.selected_set_contract.as_str()),
            ),
            optional_usize_field(
                "closure_summary_object_package_selected_provider_bundle_count",
                mirror.map(|value| value.selected_count),
            ),
            crate::json_optional_string_field(
                "closure_summary_object_package_selected_provider_bundle_set_hash",
                mirror.map(|value| value.selected_set_hash.as_str()),
            ),
            crate::json_optional_string_field(
                "closure_summary_object_package_selected_provider_bundle_set_validation_status",
                mirror.map(|value| value.selected_set_validation_status.as_str()),
            ),
        ]
    }
}

fn optional_usize_field(name: &str, value: Option<usize>) -> String {
    match value {
        Some(value) => format!("\"{name}\":{value}"),
        None => format!("\"{name}\":null"),
    }
}
