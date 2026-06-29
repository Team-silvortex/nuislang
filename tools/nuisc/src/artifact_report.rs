use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain_build_report::{
    collect_domain_build_unit_verdicts, domain_build_contract_drift_checks,
    domain_build_contract_drift_json, domain_build_unit_contracts_json,
    domain_build_unit_verification_verdict_json, domain_build_verification_summary_json,
    summarize_domain_build_verification, DomainBuildVerificationSummary,
};
use crate::execution_inspect::{execution_inspect_issues, ExecutionInspectOverview};
use crate::execution_inspect_report::inspect_execution_json;
use crate::inspect_report::{collect_doc_indexes_from_manifest_input, inspect_docs_json};
use crate::json_report::{
    artifact_lowering_units_json, json_bool_field, json_optional_string_field,
    json_string_array_field, json_string_field, json_usize_field,
};
use crate::link_report::link_plan_json;
use crate::project_metadata_report::{
    inspect_project_metadata_json, project_metadata_summary_from_manifest_report,
    ProjectMetadataSummary,
};
use crate::{aot, frontend, linker, registry};

pub(crate) fn domain_build_contract_summary_json(
    summary: &registry::NustarDomainBuildContractSummary,
) -> String {
    let lowering_fields = vec![
        json_string_field("lane_policy", &summary.lowering.lane_policy),
        json_string_field("bridge_surface", &summary.lowering.bridge_surface),
        json_string_field("emission_kind", &summary.lowering.emission_kind),
    ];
    let backend_fields = vec![
        json_string_field("stub_kind", &summary.backend.stub_kind),
        json_string_field("bridge_entry", &summary.backend.bridge_entry),
        json_string_field("submission_mode", &summary.backend.submission_mode),
        json_string_field("wake_policy", &summary.backend.wake_policy),
        json_string_field("scheduler_binding", &summary.backend.scheduler_binding),
        json_optional_string_field("phase_bind", summary.backend.phase_bind.as_deref()),
        json_optional_string_field("phase_submit", summary.backend.phase_submit.as_deref()),
        json_optional_string_field("phase_wait", summary.backend.phase_wait.as_deref()),
        json_optional_string_field("phase_finalize", summary.backend.phase_finalize.as_deref()),
        json_optional_string_field(
            "transport_model",
            summary.backend.transport_model.as_deref(),
        ),
        json_optional_string_field("request_shape", summary.backend.request_shape.as_deref()),
        json_optional_string_field("response_shape", summary.backend.response_shape.as_deref()),
        json_optional_string_field("dispatch_shape", summary.backend.dispatch_shape.as_deref()),
        json_optional_string_field("memory_binding", summary.backend.memory_binding.as_deref()),
        json_optional_string_field(
            "resource_binding",
            summary.backend.resource_binding.as_deref(),
        ),
        json_optional_string_field(
            "completion_model",
            summary.backend.completion_model.as_deref(),
        ),
    ];
    let bridge_fields = vec![
        json_string_field("bridge_surface", &summary.bridge.bridge_surface),
        json_string_field("bridge_entry", &summary.bridge.bridge_entry),
        json_string_field("scheduler_binding", &summary.bridge.scheduler_binding),
        json_string_field("phase_bind", &summary.bridge.phase_bind),
        json_string_field("phase_submit", &summary.bridge.phase_submit),
        json_string_field("phase_wait", &summary.bridge.phase_wait),
        json_string_field("phase_finalize", &summary.bridge.phase_finalize),
        json_string_field("bridge_kind", &summary.bridge.bridge_kind),
    ];
    let host_bridge_fields = vec![
        json_string_field("host_ffi_surface", &summary.host_bridge.host_ffi_surface),
        json_string_field("handle_family", &summary.host_bridge.handle_family),
        json_string_array_field("phase_order", &summary.host_bridge.phase_order),
        json_string_array_field("phase_bind_inputs", &summary.host_bridge.phase_bind_inputs),
        json_string_array_field(
            "phase_bind_outputs",
            &summary.host_bridge.phase_bind_outputs,
        ),
        json_string_array_field(
            "phase_submit_inputs",
            &summary.host_bridge.phase_submit_inputs,
        ),
        json_string_array_field(
            "phase_submit_outputs",
            &summary.host_bridge.phase_submit_outputs,
        ),
        json_string_array_field("phase_wait_inputs", &summary.host_bridge.phase_wait_inputs),
        json_string_array_field(
            "phase_wait_outputs",
            &summary.host_bridge.phase_wait_outputs,
        ),
        json_string_array_field(
            "phase_finalize_inputs",
            &summary.host_bridge.phase_finalize_inputs,
        ),
        json_string_array_field(
            "phase_finalize_outputs",
            &summary.host_bridge.phase_finalize_outputs,
        ),
        json_string_field("phase_bind_wake", &summary.host_bridge.phase_bind_wake),
        json_string_field("phase_submit_wake", &summary.host_bridge.phase_submit_wake),
        json_string_field("phase_wait_wake", &summary.host_bridge.phase_wait_wake),
        json_string_field(
            "phase_finalize_wake",
            &summary.host_bridge.phase_finalize_wake,
        ),
        json_bool_field("bridge_plan_begin", summary.host_bridge.bridge_plan_begin),
        json_bool_field("bridge_plan_end", summary.host_bridge.bridge_plan_end),
    ];
    format!(
        "{{\"lowering\":{{{}}},\"backend\":{{{}}},\"bridge\":{{{}}},\"host_bridge\":{{{}}}}}",
        lowering_fields.join(","),
        backend_fields.join(","),
        bridge_fields.join(","),
        host_bridge_fields.join(","),
    )
}

pub(crate) fn domain_registry_json(
    registration: &registry::NustarDomainRegistration,
    manifest: &registry::NustarPackageManifest,
) -> String {
    let mut fields = registry::domain_registration_json(registration);
    fields.pop();
    fields.push_str(&format!(
        ",\"build_contract\":{}",
        domain_build_contract_summary_json(&registry::domain_build_contract_summary(manifest))
    ));
    fields.push('}');
    fields
}

pub(crate) fn domain_build_unit_json(unit: &aot::BuildManifestDomainBuildUnit) -> String {
    let fields = vec![
        json_string_field("package_id", &unit.package_id),
        json_string_field("domain_family", &unit.domain_family),
        json_optional_string_field("abi", unit.abi.as_deref()),
        json_optional_string_field("machine_arch", unit.machine_arch.as_deref()),
        json_optional_string_field("machine_os", unit.machine_os.as_deref()),
        json_optional_string_field("backend_family", unit.backend_family.as_deref()),
        json_optional_string_field(
            "selected_lowering_target",
            unit.selected_lowering_target.as_deref(),
        ),
        json_optional_string_field("artifact_stub_path", unit.artifact_stub_path.as_deref()),
        json_optional_string_field(
            "artifact_payload_path",
            unit.artifact_payload_path.as_deref(),
        ),
        json_optional_string_field(
            "artifact_bridge_stub_path",
            unit.artifact_bridge_stub_path.as_deref(),
        ),
        json_optional_string_field(
            "artifact_payload_blob_path",
            unit.artifact_payload_blob_path.as_deref(),
        ),
        match unit.artifact_payload_blob_bytes {
            Some(value) => json_usize_field("artifact_payload_blob_bytes", value),
            None => "\"artifact_payload_blob_bytes\":null".to_owned(),
        },
        json_optional_string_field(
            "artifact_payload_format",
            unit.artifact_payload_format.as_deref(),
        ),
        json_string_field("contract_family", &unit.contract_family),
        json_string_field("packaging_role", &unit.packaging_role),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn artifact_report_summary_lines(
    artifact_verify: &aot::NuisCompiledArtifactVerifyReport,
    verification_summary: &DomainBuildVerificationSummary,
    link_plan: Option<&linker::LinkPlan>,
    manifest_verify_reconstructed: bool,
    execution_overview: Option<&ExecutionInspectOverview>,
    doc_indexes: Option<&[frontend::AstDocIndex]>,
    project_metadata: Option<&ProjectMetadataSummary>,
) -> Vec<String> {
    let mut lines = vec![
        format!(
            "summary: artifact_roundtrip={} lifecycle={} runtime_flags={} all_units_consistent={}",
            if artifact_verify.artifact_roundtrip_verified {
                "ok"
            } else {
                "failed"
            },
            if artifact_verify.lifecycle_contract_consistent {
                "ok"
            } else {
                "failed"
            },
            if artifact_verify.lifecycle_runtime_capability_flags_consistent {
                "ok"
            } else {
                "failed"
            },
            if verification_summary.all_units_consistent {
                "true"
            } else {
                "false"
            }
        ),
        format!(
            "summary_units: total={} host={} hetero={} drift={} failing={}",
            verification_summary.total_units,
            verification_summary.host_units_checked,
            verification_summary.hetero_units_checked,
            verification_summary.registry_drift_units,
            if verification_summary.failing_units.is_empty() {
                "<none>".to_owned()
            } else {
                verification_summary.failing_units.join(", ")
            }
        ),
        format!(
            "summary_manifest: reconstructed={}",
            if manifest_verify_reconstructed {
                "true"
            } else {
                "false"
            }
        ),
    ];
    if let Some(plan) = link_plan {
        lines.push(format!(
            "summary_link: final_stage={} driver={} link_mode={} output={}",
            plan.final_stage.kind,
            plan.final_stage.driver,
            plan.final_stage.link_mode,
            plan.final_stage.output_path
        ));
    }
    if let Some(overview) = execution_overview {
        let issues = execution_inspect_issues(overview);
        lines.push(format!(
            "summary_execution: hetero_domains={} domains={}",
            overview.heterogeneous_domains,
            if overview.domains.is_empty() {
                "<none>".to_owned()
            } else {
                overview
                    .domains
                    .iter()
                    .map(|domain| {
                        let target = domain
                            .selected_lowering_target
                            .as_deref()
                            .unwrap_or("<none>");
                        format!(
                            "{}(target={} phases={} events={})",
                            domain.domain_family, target, domain.phase_count, domain.event_count
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        ));
        lines.push(format!(
            "summary_execution_issues: {}",
            if issues.is_empty() {
                "<none>".to_owned()
            } else {
                issues
                    .iter()
                    .map(|issue| format!("{}:{}", issue.domain_family, issue.issue))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        ));
    }
    if let Some(indexes) = doc_indexes {
        let module_count = indexes.len();
        let item_count = indexes.iter().map(|index| index.items.len()).sum::<usize>();
        lines.push(format!(
            "summary_docs: modules={} documented_items={} documented_modules={}",
            module_count,
            item_count,
            if indexes.is_empty() {
                "<none>".to_owned()
            } else {
                indexes
                    .iter()
                    .map(|index| index.module_path.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        ));
    }
    if let Some(project) = project_metadata {
        lines.push(format!(
            "summary_project: docs={}/{}/{} imports={}/{}/{}/{}/{} galaxies={}/{}/{}/{}",
            project.docs_module_count,
            project.docs_documented_module_count,
            project.docs_documented_item_count,
            project.imports_library_count,
            project.imports_visible_library_count,
            project.imports_visible_module_count,
            project.imports_documented_visible_module_count,
            project.imports_documented_visible_item_count,
            project.galaxy_count,
            project.documented_galaxy_count,
            project.documented_galaxy_library_module_count,
            project.documented_galaxy_item_count
        ));
    }
    lines
}

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

pub(crate) fn reconstruct_manifest_report_from_artifact(
    input: &Path,
    artifact: &aot::NuisCompiledArtifact,
) -> Result<(PathBuf, aot::BuildManifestVerifyReport), String> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("failed to read current time: {error}"))?
        .as_nanos();
    let temp_root = std::env::temp_dir().join(format!("nuis_artifact_report_{nonce}"));
    std::fs::create_dir_all(&temp_root)
        .map_err(|error| format!("failed to create `{}`: {error}", temp_root.display()))?;

    let manifest_path = temp_root.join("nuis.build.manifest.toml");
    let envelope_path = temp_root.join("nuis.executable.envelope.toml");
    let artifact_path = temp_root.join("nuis.compiled.artifact");
    let binary_path = temp_root.join(&artifact.binary_name);

    let result = (|| {
        std::fs::write(&binary_path, &artifact.binary_blob)
            .map_err(|error| format!("failed to write `{}`: {error}", binary_path.display()))?;
        aot::write_nuis_executable_envelope(&envelope_path, &artifact.envelope)?;
        let relocated_manifest = aot::render_relocated_unpacked_build_manifest(
            artifact,
            &temp_root,
            &envelope_path,
            &artifact_path,
            &binary_path,
        )?;
        let mut relocated_artifact = artifact.clone();
        relocated_artifact.build_manifest_source = relocated_manifest.clone();
        relocated_artifact.build_manifest_bytes = relocated_manifest.len();
        aot::write_nuis_compiled_artifact(&artifact_path, &relocated_artifact)?;
        std::fs::write(&manifest_path, relocated_manifest)
            .map_err(|error| format!("failed to write `{}`: {error}", manifest_path.display()))?;
        let report = aot::verify_build_manifest(&manifest_path)?;
        Ok((manifest_path.clone(), report))
    })();

    let _ = std::fs::remove_dir_all(&temp_root);
    result.map_err(|error: String| {
        format!(
            "failed to reconstruct build manifest context for `{}`: {error}",
            input.display()
        )
    })
}

pub(crate) fn artifact_report_json(
    input: &Path,
    artifact: &aot::NuisCompiledArtifact,
    artifact_verify_input: &Path,
    artifact_verify: &aot::NuisCompiledArtifactVerifyReport,
    manifest_input: &Path,
    manifest_verify: &aot::BuildManifestVerifyReport,
    manifest_verify_reconstructed: bool,
) -> String {
    let verdicts = collect_domain_build_unit_verdicts(manifest_verify);
    let summary = summarize_domain_build_verification(&verdicts);
    let link_plan = linker::build_link_plan(manifest_verify, artifact);
    let doc_indexes =
        collect_doc_indexes_from_manifest_input(manifest_verify).unwrap_or_else(|_| Vec::new());
    let execution_inspect = inspect_execution_json(manifest_input).unwrap_or_else(|error| {
        format!(
            "{{{},{},{}}}",
            json_string_field("kind", "nuis_execution_inspect_error"),
            json_string_field("input", &manifest_input.display().to_string()),
            json_string_field("error", &error)
        )
    });
    let project_metadata =
        inspect_project_metadata_json(&project_metadata_summary_from_manifest_report(
            "build-manifest",
            Some(manifest_input),
            Some(artifact_verify_input),
            manifest_verify,
        ));
    let artifact_container =
        aot::inspect_nuis_compiled_artifact_container(artifact_verify_input).ok();
    let fields = vec![
        json_string_field("kind", "nuis_artifact_report"),
        json_string_field("input", &input.display().to_string()),
        json_bool_field(
            "manifest_verify_reconstructed",
            manifest_verify_reconstructed,
        ),
        format!(
            "\"domain_build_verification_summary\":{}",
            domain_build_verification_summary_json(&summary)
        ),
        format!(
            "\"artifact_inspect\":{}",
            inspect_artifact_json(
                input,
                artifact,
                artifact_container.as_ref(),
                Some(manifest_verify),
            )
        ),
        format!(
            "\"artifact_verify\":{}",
            verify_artifact_json(artifact_verify_input, artifact_verify)
        ),
        format!(
            "\"manifest_verify\":{}",
            verify_build_manifest_json(manifest_input, manifest_verify)
        ),
        format!("\"project_metadata\":{}", project_metadata),
        format!(
            "\"doc_index\":{}",
            inspect_docs_json(Path::new(&manifest_verify.input), &doc_indexes)
        ),
        format!("\"execution_inspect\":{}", execution_inspect),
        format!("\"link_plan\":{}", link_plan_json(&link_plan)),
    ];
    format!("{{{}}}", fields.join(","))
}
