use super::*;

pub(crate) fn verify_artifact_json(
    input: &Path,
    report: &aot::NuisCompiledArtifactVerifyReport,
) -> String {
    let fields = vec![
        json_string_field("kind", "nuis_artifact_verify"),
        json_string_field("input", &input.display().to_string()),
        json_string_field("schema", &report.schema),
        json_string_field("artifact_container_kind", &report.artifact_container_kind),
        json_usize_field(
            "artifact_container_version",
            report.artifact_container_version as usize,
        ),
        json_usize_field("artifact_section_count", report.artifact_section_count),
        json_string_array_field("artifact_section_names", &report.artifact_section_names),
        json_bool_field(
            "artifact_section_table_valid",
            report.artifact_section_table_valid,
        ),
        json_usize_field("lowering_unit_count", report.lowering_unit_count),
        json_string_array_field("lowering_domain_families", &report.lowering_domain_families),
        json_string_array_field("lowering_targets", &report.lowering_targets),
        artifact_lowering_units_json(&report.lowering_units),
        json_string_field("packaging_mode", &report.packaging_mode),
        json_string_field("binary_name", &report.binary_name),
        json_usize_field("binary_bytes", report.binary_bytes),
        json_usize_field("build_manifest_bytes", report.build_manifest_bytes),
        json_string_field("envelope_schema", &report.envelope_schema),
        json_usize_field("envelope_package_count", report.envelope_package_count),
        json_string_field("lifecycle_schema", &report.lifecycle_schema),
        json_string_field(
            "lifecycle_bootstrap_entry",
            &report.lifecycle_bootstrap_entry,
        ),
        json_string_field("lifecycle_tick_policy", &report.lifecycle_tick_policy),
        json_string_field(
            "lifecycle_shutdown_policy",
            &report.lifecycle_shutdown_policy,
        ),
        json_string_field("lifecycle_yalivia_rpc", &report.lifecycle_yalivia_rpc),
        json_usize_field("lifecycle_hook_count", report.lifecycle_hook_count),
        json_string_array_field("lifecycle_hook_surface", &report.lifecycle_hook_surface),
        json_usize_field("lifecycle_export_count", report.lifecycle_export_count),
        json_string_array_field("lifecycle_export_surface", &report.lifecycle_export_surface),
        json_string_array_field(
            "lifecycle_runtime_capability_flags",
            &report.lifecycle_runtime_capability_flags,
        ),
        json_bool_field(
            "lifecycle_contract_consistent",
            report.lifecycle_contract_consistent,
        ),
        json_bool_field(
            "lifecycle_runtime_capability_flags_consistent",
            report.lifecycle_runtime_capability_flags_consistent,
        ),
        json_usize_field(
            "execution_contracts_checked",
            report.execution_contracts_checked,
        ),
        json_string_field("cpu_target_abi", &report.cpu_target_abi),
        json_string_field("cpu_target_machine_arch", &report.cpu_target_machine_arch),
        json_string_field("cpu_target_machine_os", &report.cpu_target_machine_os),
        json_string_field("cpu_target_object_format", &report.cpu_target_object_format),
        json_string_field("cpu_target_calling_abi", &report.cpu_target_calling_abi),
        json_bool_field(
            "artifact_roundtrip_verified",
            report.artifact_roundtrip_verified,
        ),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn verify_build_manifest_json(
    input: &Path,
    report: &aot::BuildManifestVerifyReport,
) -> String {
    let domain_build_units = report
        .domain_build_units
        .iter()
        .map(domain_build_unit_json)
        .collect::<Vec<_>>()
        .join(",");
    let domain_build_contracts = domain_build_unit_contracts_json(&report.domain_build_units);
    let drift_checks = domain_build_contract_drift_checks(&report.domain_build_units);
    let drift_mismatch_count = drift_checks
        .iter()
        .filter(|check| !check.consistent)
        .count();
    let verdicts = collect_domain_build_unit_verdicts(report);
    let summary = summarize_domain_build_verification(&verdicts);
    let fields = vec![
        json_string_field("kind", "nuis_build_manifest_verify"),
        json_string_field("input", &input.display().to_string()),
        json_string_field("schema", &report.schema),
        json_string_field("manifest_input", &report.input),
        json_string_field("output_dir", &report.output_dir),
        json_string_field("packaging_mode", &report.packaging_mode),
        json_string_field("envelope_path", &report.envelope_path),
        json_string_field("envelope_schema", &report.envelope_schema),
        json_usize_field("envelope_package_count", report.envelope_package_count),
        json_string_field("artifact_path", &report.artifact_path),
        json_string_field("artifact_schema", &report.artifact_schema),
        json_string_field("artifact_binary_name", &report.artifact_binary_name),
        json_usize_field("artifact_binary_bytes", report.artifact_binary_bytes),
        json_string_field("lifecycle_schema", &report.lifecycle_schema),
        json_string_field(
            "lifecycle_bootstrap_entry",
            &report.lifecycle_bootstrap_entry,
        ),
        json_string_field("lifecycle_tick_policy", &report.lifecycle_tick_policy),
        json_string_field(
            "lifecycle_shutdown_policy",
            &report.lifecycle_shutdown_policy,
        ),
        json_string_field("lifecycle_yalivia_rpc", &report.lifecycle_yalivia_rpc),
        json_usize_field("lifecycle_hook_count", report.lifecycle_hook_count),
        json_string_array_field("lifecycle_hook_surface", &report.lifecycle_hook_surface),
        json_usize_field("lifecycle_export_count", report.lifecycle_export_count),
        json_string_array_field("lifecycle_export_surface", &report.lifecycle_export_surface),
        json_string_array_field(
            "lifecycle_runtime_capability_flags",
            &report.lifecycle_runtime_capability_flags,
        ),
        json_usize_field(
            "execution_contracts_checked",
            report.execution_contracts_checked,
        ),
        json_usize_field("domain_build_unit_count", report.domain_build_unit_count),
        json_usize_field(
            "heterogeneous_domain_count",
            report.heterogeneous_domain_count,
        ),
        json_usize_field(
            "domain_payload_blobs_checked",
            report.domain_payload_blobs_checked,
        ),
        json_usize_field(
            "domain_payload_blob_sections_checked",
            report.domain_payload_blob_sections_checked,
        ),
        json_usize_field(
            "domain_payload_contract_sections_checked",
            report.domain_payload_contract_sections_checked,
        ),
        json_usize_field(
            "domain_payload_lowering_plans_checked",
            report.domain_payload_lowering_plans_checked,
        ),
        json_usize_field(
            "domain_payload_backend_stubs_checked",
            report.domain_payload_backend_stubs_checked,
        ),
        json_usize_field(
            "domain_payload_bridge_plans_checked",
            report.domain_payload_bridge_plans_checked,
        ),
        json_usize_field(
            "domain_bridge_stubs_checked",
            report.domain_bridge_stubs_checked,
        ),
        format!("\"domain_build_units\":[{}]", domain_build_units),
        format!("\"domain_build_contracts\":[{}]", domain_build_contracts),
        json_usize_field("domain_build_contract_drift_checked", drift_checks.len()),
        json_usize_field(
            "domain_build_contract_drift_mismatches",
            drift_mismatch_count,
        ),
        json_bool_field(
            "domain_build_contracts_consistent",
            drift_mismatch_count == 0,
        ),
        format!(
            "\"domain_build_verification_summary\":{}",
            domain_build_verification_summary_json(&summary)
        ),
        format!(
            "\"domain_build_contract_drift\":[{}]",
            drift_checks
                .iter()
                .map(domain_build_contract_drift_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        json_optional_string_field(
            "bridge_registry_path",
            report.bridge_registry_path.as_deref(),
        ),
        json_usize_field("bridge_registry_units", report.bridge_registry_units),
        json_usize_field("bridge_registry_checked", report.bridge_registry_checked),
        json_usize_field(
            "bridge_registry_entries_checked",
            report.bridge_registry_entries_checked,
        ),
        json_optional_string_field(
            "host_bridge_plan_index_path",
            report.host_bridge_plan_index_path.as_deref(),
        ),
        json_usize_field("host_bridge_plan_units", report.host_bridge_plan_units),
        json_usize_field("host_bridge_plan_checked", report.host_bridge_plan_checked),
        json_usize_field(
            "host_bridge_plan_entries_checked",
            report.host_bridge_plan_entries_checked,
        ),
        json_optional_string_field(
            "lowering_plan_index_path",
            report.lowering_plan_index_path.as_deref(),
        ),
        json_usize_field("lowering_plan_units", report.lowering_plan_units),
        json_usize_field(
            "lowering_plan_index_checked",
            report.lowering_plan_index_checked,
        ),
        json_usize_field(
            "lowering_plan_entries_checked",
            report.lowering_plan_entries_checked,
        ),
        json_optional_string_field("clock_protocol_path", report.clock_protocol_path.as_deref()),
        json_usize_field("clock_protocol_domains", report.clock_protocol_domains),
        json_usize_field("clock_protocol_checked", report.clock_protocol_checked),
        json_usize_field(
            "clock_protocol_entries_checked",
            report.clock_protocol_entries_checked,
        ),
        json_optional_string_field(
            "hetero_calculate_plan_path",
            report.hetero_calculate_plan_path.as_deref(),
        ),
        json_usize_field(
            "hetero_calculate_plan_units",
            report.hetero_calculate_plan_units,
        ),
        json_usize_field(
            "hetero_calculate_plan_checked",
            report.hetero_calculate_plan_checked,
        ),
        json_usize_field(
            "hetero_calculate_plan_entries_checked",
            report.hetero_calculate_plan_entries_checked,
        ),
        json_optional_string_field("doc_index_path", report.doc_index_path.as_deref()),
        json_usize_field("doc_index_module_count", report.doc_index_module_count),
        json_usize_field(
            "doc_index_documented_item_count",
            report.doc_index_documented_item_count,
        ),
        json_usize_field("doc_index_checked", report.doc_index_checked),
        json_optional_string_field("project_docs_index", report.project_docs_index.as_deref()),
        json_usize_field(
            "project_docs_module_count",
            report.project_docs_module_count,
        ),
        json_usize_field(
            "project_docs_documented_module_count",
            report.project_docs_documented_module_count,
        ),
        json_usize_field(
            "project_docs_documented_item_count",
            report.project_docs_documented_item_count,
        ),
        json_optional_string_field(
            "project_imports_index",
            report.project_imports_index.as_deref(),
        ),
        json_usize_field(
            "project_imports_library_count",
            report.project_imports_library_count,
        ),
        json_usize_field(
            "project_imports_visible_library_count",
            report.project_imports_visible_library_count,
        ),
        json_usize_field(
            "project_imports_visible_module_count",
            report.project_imports_visible_module_count,
        ),
        json_usize_field(
            "project_imports_documented_visible_module_count",
            report.project_imports_documented_visible_module_count,
        ),
        json_usize_field(
            "project_imports_documented_visible_item_count",
            report.project_imports_documented_visible_item_count,
        ),
        json_optional_string_field(
            "project_galaxy_index",
            report.project_galaxy_index.as_deref(),
        ),
        json_usize_field("project_galaxy_count", report.project_galaxy_count),
        json_usize_field(
            "project_documented_galaxy_count",
            report.project_documented_galaxy_count,
        ),
        json_usize_field(
            "project_documented_galaxy_library_module_count",
            report.project_documented_galaxy_library_module_count,
        ),
        json_usize_field(
            "project_documented_galaxy_item_count",
            report.project_documented_galaxy_item_count,
        ),
        format!(
            "\"domain_build_unit_verdicts\":[{}]",
            verdicts
                .iter()
                .map(domain_build_unit_verification_verdict_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        json_string_field("cpu_target_abi", &report.cpu_target_abi),
        json_string_field("cpu_target_machine_arch", &report.cpu_target_machine_arch),
        json_string_field("cpu_target_machine_os", &report.cpu_target_machine_os),
        json_string_field("cpu_target_object_format", &report.cpu_target_object_format),
        json_string_field("cpu_target_calling_abi", &report.cpu_target_calling_abi),
        json_string_field("cpu_target_clang", &report.cpu_target_clang),
        json_bool_field("cpu_target_cross", report.cpu_target_cross),
        json_usize_field("artifacts_checked", report.artifacts_checked),
        json_usize_field("project_metadata_checked", report.project_metadata_checked),
    ];
    format!("{{{}}}", fields.join(","))
}
