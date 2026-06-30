use super::*;

pub(crate) fn inspect_artifact_json(
    input: &Path,
    artifact: &aot::NuisCompiledArtifact,
    container: Option<&aot::NuisCompiledArtifactContainerInspect>,
    manifest_verify: Option<&aot::BuildManifestVerifyReport>,
) -> String {
    let mut fields = vec![
        json_string_field("kind", "nuis_artifact_inspect"),
        json_string_field("input", &input.display().to_string()),
        json_string_field("schema", &artifact.schema),
        json_string_field("packaging_mode", &artifact.packaging_mode),
        json_string_field("cpu_target_abi", &artifact.cpu_target_abi),
        json_string_field("cpu_target_machine_arch", &artifact.cpu_target_machine_arch),
        json_string_field("cpu_target_machine_os", &artifact.cpu_target_machine_os),
        json_string_field(
            "cpu_target_object_format",
            &artifact.cpu_target_object_format,
        ),
        json_string_field("cpu_target_calling_abi", &artifact.cpu_target_calling_abi),
        json_string_field("binary_name", &artifact.binary_name),
        json_usize_field("binary_bytes", artifact.binary_bytes),
        json_usize_field("build_manifest_bytes", artifact.build_manifest_bytes),
        json_string_field("envelope_schema", &artifact.envelope.schema),
        json_string_array_field(
            "envelope_contract_families",
            &artifact.envelope.contract_families,
        ),
        json_string_field("lifecycle_schema", &artifact.lifecycle.schema),
        json_string_field(
            "lifecycle_bootstrap_entry",
            &artifact.lifecycle.bootstrap_entry,
        ),
        json_string_field("lifecycle_tick_policy", &artifact.lifecycle.tick_policy),
        json_string_field(
            "lifecycle_shutdown_policy",
            &artifact.lifecycle.shutdown_policy,
        ),
        json_string_field("lifecycle_yalivia_rpc", &artifact.lifecycle.yalivia_rpc),
        json_usize_field(
            "lifecycle_hook_count",
            artifact.lifecycle.hook_surface.len(),
        ),
        json_string_array_field("lifecycle_hook_surface", &artifact.lifecycle.hook_surface),
        json_usize_field(
            "lifecycle_export_count",
            artifact.lifecycle.export_surface.len(),
        ),
        json_string_array_field(
            "lifecycle_export_surface",
            &artifact.lifecycle.export_surface,
        ),
        json_string_array_field(
            "lifecycle_runtime_capability_flags",
            &artifact.lifecycle.runtime_capability_flags,
        ),
    ];
    if let Some(container) = container {
        fields.push(json_string_field(
            "artifact_container_magic",
            &container.magic,
        ));
        fields.push(json_usize_field(
            "artifact_container_version",
            container.binary_version as usize,
        ));
        fields.push(json_string_field(
            "artifact_container_kind",
            &container.container_kind,
        ));
        fields.push(json_usize_field(
            "artifact_section_count",
            container.section_count,
        ));
        fields.push(json_string_array_field(
            "artifact_section_names",
            &container.section_names,
        ));
        fields.push(json_bool_field(
            "artifact_section_table_valid",
            container.section_table_valid,
        ));
        fields.push(json_usize_field(
            "lowering_unit_count",
            container.lowering_unit_count,
        ));
        fields.push(json_string_array_field(
            "lowering_domain_families",
            &container.lowering_domain_families,
        ));
        fields.push(json_string_array_field(
            "lowering_targets",
            &container.lowering_targets,
        ));
        fields.push(artifact_lowering_units_json(&container.lowering_units));
    }
    if let Some(report) = manifest_verify {
        let link_plan = linker::build_link_plan(report, artifact);
        let drift_checks = domain_build_contract_drift_checks(&report.domain_build_units);
        let drift_check_count = drift_checks.len();
        let drift_mismatch_count = drift_checks
            .iter()
            .filter(|check| !check.consistent)
            .count();
        let verdicts = collect_domain_build_unit_verdicts(report);
        let summary = summarize_domain_build_verification(&verdicts);
        fields.push(json_usize_field(
            "domain_build_unit_count",
            report.domain_build_unit_count,
        ));
        fields.push(json_usize_field(
            "domain_payload_blobs_checked",
            report.domain_payload_blobs_checked,
        ));
        fields.push(json_usize_field(
            "domain_payload_blob_sections_checked",
            report.domain_payload_blob_sections_checked,
        ));
        fields.push(json_usize_field(
            "domain_payload_contract_sections_checked",
            report.domain_payload_contract_sections_checked,
        ));
        fields.push(json_usize_field(
            "domain_payload_lowering_plans_checked",
            report.domain_payload_lowering_plans_checked,
        ));
        fields.push(json_usize_field(
            "domain_payload_backend_stubs_checked",
            report.domain_payload_backend_stubs_checked,
        ));
        fields.push(json_usize_field(
            "domain_payload_bridge_plans_checked",
            report.domain_payload_bridge_plans_checked,
        ));
        fields.push(json_usize_field(
            "domain_bridge_stubs_checked",
            report.domain_bridge_stubs_checked,
        ));
        fields.push(format!(
            "\"domain_build_units\":[{}]",
            report
                .domain_build_units
                .iter()
                .map(domain_build_unit_json)
                .collect::<Vec<_>>()
                .join(",")
        ));
        fields.push(format!(
            "\"domain_build_contracts\":[{}]",
            domain_build_unit_contracts_json(&report.domain_build_units)
        ));
        fields.push(json_usize_field(
            "domain_build_contract_drift_checked",
            drift_check_count,
        ));
        fields.push(json_usize_field(
            "domain_build_contract_drift_mismatches",
            drift_mismatch_count,
        ));
        fields.push(json_bool_field(
            "domain_build_contracts_consistent",
            drift_mismatch_count == 0,
        ));
        fields.push(json_usize_field(
            "bridge_registry_entries_checked",
            report.bridge_registry_entries_checked,
        ));
        fields.push(json_usize_field(
            "host_bridge_plan_entries_checked",
            report.host_bridge_plan_entries_checked,
        ));
        fields.push(json_usize_field(
            "lowering_plan_entries_checked",
            report.lowering_plan_entries_checked,
        ));
        fields.push(json_usize_field(
            "hetero_calculate_plan_entries_checked",
            report.hetero_calculate_plan_entries_checked,
        ));
        fields.push(format!(
            "\"domain_build_verification_summary\":{}",
            domain_build_verification_summary_json(&summary)
        ));
        fields.push(format!(
            "\"domain_build_unit_verdicts\":[{}]",
            verdicts
                .iter()
                .map(domain_build_unit_verification_verdict_json)
                .collect::<Vec<_>>()
                .join(",")
        ));
        fields.push(format!(
            "\"domain_build_contract_drift\":[{}]",
            drift_checks
                .iter()
                .map(domain_build_contract_drift_json)
                .collect::<Vec<_>>()
                .join(",")
        ));
        fields.push(format!("\"link_plan\":{}", link_plan_json(&link_plan)));
    }
    format!("{{{}}}", fields.join(","))
}
