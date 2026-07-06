pub(crate) use super::json_container::*;
pub(crate) use super::json_object::*;
pub(crate) use super::json_object_emit::*;
pub(crate) use super::json_object_image::*;

use super::{json_fields::*, json_fragments::*, reports::*};

pub(crate) fn nsld_artifact_chain_report_json(report: &NsldArtifactChainReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_artifact_chain"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_dir", &report.output_dir),
        json_bool_field("valid", report.valid),
        json_usize_field("stage_count", report.stage_count),
        json_usize_field("present_count", report.present_count),
        json_usize_field("required_count", report.required_count),
        json_usize_field("missing_required_count", report.missing_required_count),
        json_usize_field("optional_present_count", report.optional_present_count),
        json_optional_string_field(
            "first_missing_required_stage",
            report.first_missing_required_stage.as_deref(),
        ),
        json_optional_string_field("next_required_stage", report.next_required_stage.as_deref()),
        json_optional_string_field(
            "suggested_command_id",
            report.suggested_command_id.as_deref(),
        ),
        json_optional_string_field("suggested_command", report.suggested_command.as_deref()),
        json_optional_string_field(
            "suggested_command_resolved",
            report.suggested_command_resolved.as_deref(),
        ),
        json_optional_string_field(
            "suggested_command_reason",
            report.suggested_command_reason.as_deref(),
        ),
        json_optional_string_field("next_optional_stage", report.next_optional_stage.as_deref()),
        json_optional_string_field(
            "next_optional_command_id",
            report.next_optional_command_id.as_deref(),
        ),
        json_optional_string_field(
            "next_optional_command",
            report.next_optional_command.as_deref(),
        ),
        json_optional_string_field(
            "next_optional_command_resolved",
            report.next_optional_command_resolved.as_deref(),
        ),
        json_optional_string_field(
            "next_optional_command_reason",
            report.next_optional_command_reason.as_deref(),
        ),
        format!(
            "\"stages\":[{}]",
            artifact_chain_stage_diagnostics_json(&report.stages)
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn check_report_json(report: &NsldCheckReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_linker_check"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("valid", report.valid),
        json_usize_field("checks", report.checks),
        json_usize_field("failures", report.failures),
        json_bool_field(
            "artifact_lowering_alignment_consistent",
            report.artifact_lowering_alignment_consistent,
        ),
        json_usize_field(
            "artifact_lowering_alignment_mismatches",
            report.artifact_lowering_alignment_mismatches,
        ),
        json_bool_field("clock_protocol_valid", report.clock_protocol_valid),
        json_string_array_field("clock_protocol_issues", &report.clock_protocol_issues),
        json_bool_field("hetero_calculate_valid", report.hetero_calculate_valid),
        json_string_array_field("hetero_calculate_issues", &report.hetero_calculate_issues),
        json_bool_field("static_link", report.static_link),
        json_bool_field("lifecycle_driven", report.lifecycle_driven),
        json_bool_field("sidecar_capability_valid", report.sidecar_capability_valid),
        json_string_array_field(
            "sidecar_capability_issues",
            &report.sidecar_capability_issues,
        ),
        json_bool_field("link_input_table_present", report.link_input_table_present),
        json_optional_bool_field("link_input_table_valid", report.link_input_table_valid),
        json_string_array_field("link_input_table_issues", &report.link_input_table_issues),
        json_bool_field("link_unit_table_present", report.link_unit_table_present),
        json_optional_bool_field("link_unit_table_valid", report.link_unit_table_valid),
        json_string_array_field("link_unit_table_issues", &report.link_unit_table_issues),
        json_bool_field("link_bundle_present", report.link_bundle_present),
        json_optional_bool_field("link_bundle_valid", report.link_bundle_valid),
        json_string_array_field("link_bundle_issues", &report.link_bundle_issues),
        json_bool_field("assemble_plan_present", report.assemble_plan_present),
        json_optional_bool_field("assemble_plan_valid", report.assemble_plan_valid),
        json_string_array_field("assemble_plan_issues", &report.assemble_plan_issues),
        json_bool_field("section_manifest_present", report.section_manifest_present),
        json_optional_bool_field("section_manifest_valid", report.section_manifest_valid),
        json_string_array_field("section_manifest_issues", &report.section_manifest_issues),
        json_bool_field("object_plan_present", report.object_plan_present),
        json_optional_bool_field("object_plan_valid", report.object_plan_valid),
        json_string_array_field("object_plan_issues", &report.object_plan_issues),
        json_bool_field(
            "object_writer_input_present",
            report.object_writer_input_present,
        ),
        json_optional_bool_field(
            "object_writer_input_valid",
            report.object_writer_input_valid,
        ),
        json_string_array_field(
            "object_writer_input_issues",
            &report.object_writer_input_issues,
        ),
        json_bool_field(
            "object_byte_layout_present",
            report.object_byte_layout_present,
        ),
        json_optional_bool_field("object_byte_layout_valid", report.object_byte_layout_valid),
        json_string_array_field(
            "object_byte_layout_issues",
            &report.object_byte_layout_issues,
        ),
        json_bool_field(
            "object_file_layout_present",
            report.object_file_layout_present,
        ),
        json_optional_bool_field("object_file_layout_valid", report.object_file_layout_valid),
        json_string_array_field(
            "object_file_layout_issues",
            &report.object_file_layout_issues,
        ),
        json_bool_field(
            "object_image_dry_run_present",
            report.object_image_dry_run_present,
        ),
        json_optional_bool_field(
            "object_image_dry_run_valid",
            report.object_image_dry_run_valid,
        ),
        json_string_array_field(
            "object_image_dry_run_issues",
            &report.object_image_dry_run_issues,
        ),
        json_optional_bool_field(
            "object_image_relocation_lowering_valid",
            report.object_image_relocation_lowering_valid,
        ),
        json_optional_usize_field(
            "object_image_relocation_lowering_rule_count",
            report.object_image_relocation_lowering_rule_count,
        ),
        format!(
            "\"object_image_relocation_lowering_rules\":[{}]",
            relocation_lowering_rules_json(&report.object_image_relocation_lowering_rules)
        ),
        json_string_array_field(
            "object_image_relocation_lowering_issues",
            &report.object_image_relocation_lowering_issues,
        ),
        json_optional_usize_field(
            "object_image_relocation_record_count",
            report.object_image_relocation_record_count,
        ),
        json_optional_string_field(
            "object_image_relocation_record_table_hash",
            report.object_image_relocation_record_table_hash.as_deref(),
        ),
        format!(
            "\"object_image_relocation_records\":[{}]",
            relocation_records_json(&report.object_image_relocation_records)
        ),
        json_bool_field(
            "object_image_dry_run_bytes_present",
            report.object_image_dry_run_bytes_present,
        ),
        json_bool_field(
            "object_emit_blocked_present",
            report.object_emit_blocked_present,
        ),
        json_optional_bool_field(
            "object_emit_blocked_valid",
            report.object_emit_blocked_valid,
        ),
        json_string_array_field(
            "object_emit_blocked_issues",
            &report.object_emit_blocked_issues,
        ),
        json_bool_field("object_output_present", report.object_output_present),
        json_optional_bool_field("object_output_valid", report.object_output_valid),
        json_optional_usize_field(
            "object_output_expected_size_bytes",
            report.object_output_expected_size_bytes,
        ),
        json_optional_usize_field(
            "object_output_actual_size_bytes",
            report.object_output_actual_size_bytes,
        ),
        json_optional_string_field(
            "object_output_expected_hash",
            report.object_output_expected_hash.as_deref(),
        ),
        json_optional_string_field(
            "object_output_actual_hash",
            report.object_output_actual_hash.as_deref(),
        ),
        json_string_array_field("object_output_issues", &report.object_output_issues),
        json_bool_field(
            "object_writer_dry_run_present",
            report.object_writer_dry_run_present,
        ),
        json_optional_bool_field(
            "object_writer_dry_run_valid",
            report.object_writer_dry_run_valid,
        ),
        json_string_array_field(
            "object_writer_dry_run_issues",
            &report.object_writer_dry_run_issues,
        ),
        json_bool_field("container_plan_present", report.container_plan_present),
        json_optional_bool_field("container_plan_valid", report.container_plan_valid),
        json_string_array_field("container_plan_issues", &report.container_plan_issues),
        json_bool_field("container_present", report.container_present),
        json_optional_bool_field("container_valid", report.container_valid),
        json_string_array_field("container_issues", &report.container_issues),
        json_string_array_field("container_section_issues", &report.container_section_issues),
        json_string_array_field(
            "container_loader_symbol_issues",
            &report.container_loader_symbol_issues,
        ),
        json_string_array_field(
            "container_relocation_issues",
            &report.container_relocation_issues,
        ),
        json_string_array_field(
            "container_compatibility_domain_issues",
            &report.container_compatibility_domain_issues,
        ),
        json_string_array_field(
            "container_external_import_issues",
            &report.container_external_import_issues,
        ),
        json_bool_field(
            "container_payload_present",
            report.container_payload_present,
        ),
        json_string_array_field("container_payload_issues", &report.container_payload_issues),
        json_bool_field("closure_snapshot_present", report.closure_snapshot_present),
        json_optional_bool_field("closure_snapshot_valid", report.closure_snapshot_valid),
        json_string_array_field("closure_snapshot_issues", &report.closure_snapshot_issues),
        json_optional_string_field(
            "closure_snapshot_linker_contract_hash",
            report.closure_snapshot_linker_contract_hash.as_deref(),
        ),
        json_optional_string_field(
            "closure_snapshot_container_hash",
            report.closure_snapshot_container_hash.as_deref(),
        ),
        json_optional_usize_field(
            "closure_snapshot_payload_size_bytes",
            report.closure_snapshot_payload_size_bytes,
        ),
        json_optional_string_field(
            "closure_snapshot_payload_hash",
            report.closure_snapshot_payload_hash.as_deref(),
        ),
        json_bool_field("final_stage_plan_present", report.final_stage_plan_present),
        json_optional_bool_field("final_stage_plan_valid", report.final_stage_plan_valid),
        json_optional_bool_field("final_stage_plan_ready", report.final_stage_plan_ready),
        json_optional_string_field(
            "final_stage_plan_hash",
            report.final_stage_plan_hash.as_deref(),
        ),
        json_optional_usize_field(
            "final_stage_plan_blocker_count",
            report.final_stage_plan_blocker_count,
        ),
        json_string_array_field("final_stage_plan_issues", &report.final_stage_plan_issues),
        json_bool_field(
            "final_executable_writer_input_present",
            report.final_executable_writer_input_present,
        ),
        json_optional_bool_field(
            "final_executable_writer_input_valid",
            report.final_executable_writer_input_valid,
        ),
        json_optional_string_field(
            "final_executable_writer_input_hash",
            report.final_executable_writer_input_hash.as_deref(),
        ),
        json_optional_usize_field(
            "final_executable_writer_input_command_arg_count",
            report.final_executable_writer_input_command_arg_count,
        ),
        json_string_array_field(
            "final_executable_writer_input_issues",
            &report.final_executable_writer_input_issues,
        ),
        json_bool_field(
            "final_executable_host_invoke_plan_present",
            report.final_executable_host_invoke_plan_present,
        ),
        json_optional_bool_field(
            "final_executable_host_invoke_plan_valid",
            report.final_executable_host_invoke_plan_valid,
        ),
        json_optional_string_field(
            "final_executable_host_invoke_plan_hash",
            report.final_executable_host_invoke_plan_hash.as_deref(),
        ),
        json_optional_string_field(
            "final_executable_host_invoke_plan_invocation_policy",
            report
                .final_executable_host_invoke_plan_invocation_policy
                .as_deref(),
        ),
        json_optional_bool_field(
            "final_executable_host_invoke_plan_requires_explicit_allow",
            report.final_executable_host_invoke_plan_requires_explicit_allow,
        ),
        json_optional_bool_field(
            "final_executable_host_invoke_plan_explicit_allow_present",
            report.final_executable_host_invoke_plan_explicit_allow_present,
        ),
        json_optional_bool_field(
            "final_executable_host_invoke_plan_would_invoke",
            report.final_executable_host_invoke_plan_would_invoke,
        ),
        json_optional_usize_field(
            "final_executable_host_invoke_plan_blocker_count",
            report.final_executable_host_invoke_plan_blocker_count,
        ),
        json_string_array_field(
            "final_executable_host_invoke_plan_issues",
            &report.final_executable_host_invoke_plan_issues,
        ),
        json_bool_field(
            "final_executable_layout_plan_present",
            report.final_executable_layout_plan_present,
        ),
        json_optional_bool_field(
            "final_executable_layout_plan_valid",
            report.final_executable_layout_plan_valid,
        ),
        json_optional_string_field(
            "final_executable_layout_plan_hash",
            report.final_executable_layout_plan_hash.as_deref(),
        ),
        json_optional_usize_field(
            "final_executable_layout_plan_payload_count",
            report.final_executable_layout_plan_payload_count,
        ),
        json_string_array_field(
            "final_executable_layout_plan_issues",
            &report.final_executable_layout_plan_issues,
        ),
        json_bool_field(
            "final_executable_image_dry_run_present",
            report.final_executable_image_dry_run_present,
        ),
        json_optional_bool_field(
            "final_executable_image_dry_run_valid",
            report.final_executable_image_dry_run_valid,
        ),
        json_optional_string_field(
            "final_executable_image_dry_run_hash",
            report.final_executable_image_dry_run_hash.as_deref(),
        ),
        json_optional_usize_field(
            "final_executable_image_dry_run_size_bytes",
            report.final_executable_image_dry_run_size_bytes,
        ),
        json_string_array_field(
            "final_executable_image_dry_run_issues",
            &report.final_executable_image_dry_run_issues,
        ),
        json_bool_field(
            "final_executable_blocked_present",
            report.final_executable_blocked_present,
        ),
        json_optional_bool_field(
            "final_executable_blocked_valid",
            report.final_executable_blocked_valid,
        ),
        json_optional_bool_field(
            "final_executable_blocked_emitted",
            report.final_executable_blocked_emitted,
        ),
        json_optional_string_field(
            "final_executable_blocked_plan_hash",
            report.final_executable_blocked_plan_hash.as_deref(),
        ),
        json_optional_usize_field(
            "final_executable_blocked_blocker_count",
            report.final_executable_blocked_blocker_count,
        ),
        json_string_array_field(
            "final_executable_blocked_issues",
            &report.final_executable_blocked_issues,
        ),
        json_bool_field(
            "final_executable_output_present",
            report.final_executable_output_present,
        ),
        json_optional_usize_field(
            "final_executable_output_size_bytes",
            report.final_executable_output_size_bytes,
        ),
        json_optional_string_field(
            "final_executable_output_hash",
            report.final_executable_output_hash.as_deref(),
        ),
        json_optional_bool_field(
            "final_executable_output_runnable_candidate",
            report.final_executable_output_runnable_candidate,
        ),
        json_optional_usize_field(
            "final_executable_output_blocker_count",
            report.final_executable_output_blocker_count,
        ),
        json_string_array_field(
            "final_executable_output_issues",
            &report.final_executable_output_issues,
        ),
        json_optional_string_field(
            "container_loader_readiness",
            report.container_loader_readiness.as_deref(),
        ),
        json_string_array_field(
            "container_loader_blockers",
            &report.container_loader_blockers,
        ),
        json_optional_string_field(
            "container_metadata_table_hash",
            report.container_metadata_table_hash.as_deref(),
        ),
        json_optional_usize_field(
            "container_compatibility_domain_count",
            report.container_compatibility_domain_count,
        ),
        json_optional_string_field(
            "container_compatibility_domain_table_hash",
            report.container_compatibility_domain_table_hash.as_deref(),
        ),
        json_optional_string_field(
            "container_compatibility_domain_id",
            report.container_compatibility_domain_id.as_deref(),
        ),
        json_optional_string_field(
            "container_compatibility_domain_kind",
            report.container_compatibility_domain_kind.as_deref(),
        ),
        json_optional_string_field(
            "container_compatibility_domain_paradigm",
            report.container_compatibility_domain_paradigm.as_deref(),
        ),
        json_optional_string_field(
            "container_compatibility_domain_lifecycle_hook",
            report
                .container_compatibility_domain_lifecycle_hook
                .as_deref(),
        ),
        json_optional_string_field(
            "container_compatibility_domain_abi_family",
            report.container_compatibility_domain_abi_family.as_deref(),
        ),
        json_optional_string_field(
            "container_compatibility_domain_wrapper_policy",
            report
                .container_compatibility_domain_wrapper_policy
                .as_deref(),
        ),
        json_optional_bool_field(
            "container_compatibility_domain_required",
            report.container_compatibility_domain_required,
        ),
        format!(
            "\"container_compatibility_domain_summary\":{}",
            compatibility_domain_summary_json(
                report.container_compatibility_domain_count,
                report.container_compatibility_domain_table_hash.as_deref(),
                report.container_compatibility_domain_id.as_deref(),
                report.container_compatibility_domain_kind.as_deref(),
                report.container_compatibility_domain_paradigm.as_deref(),
                report
                    .container_compatibility_domain_lifecycle_hook
                    .as_deref(),
                report.container_compatibility_domain_abi_family.as_deref(),
                report
                    .container_compatibility_domain_wrapper_policy
                    .as_deref(),
                report.container_compatibility_domain_required,
            )
        ),
        json_optional_usize_field(
            "container_external_import_count",
            report.container_external_import_count,
        ),
        json_bool_field(
            "container_native_object_section_present",
            report.container_native_object_section_present,
        ),
        json_optional_string_field(
            "container_native_object_section_id",
            report.container_native_object_section_id.as_deref(),
        ),
        json_bool_field(
            "container_native_object_loader_symbol_present",
            report.container_native_object_loader_symbol_present,
        ),
        json_optional_string_field(
            "container_native_object_loader_symbol_id",
            report.container_native_object_loader_symbol_id.as_deref(),
        ),
        json_bool_field(
            "container_native_object_relocation_present",
            report.container_native_object_relocation_present,
        ),
        json_optional_string_field(
            "container_native_object_relocation_id",
            report.container_native_object_relocation_id.as_deref(),
        ),
        json_bool_field("artifact_chain_valid", report.artifact_chain_valid),
        json_string_array_field("artifact_chain_issues", &report.artifact_chain_issues),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
        format!("\"domains\":[{}]", domains_json(&report.domains)),
        format!(
            "\"sidecar_capabilities\":[{}]",
            sidecar_capabilities_json(&report.sidecar_capabilities)
        ),
        format!(
            "\"clock_edges\":[{}]",
            clock_edges_json(&report.clock_edges)
        ),
        format!(
            "\"data_segments\":[{}]",
            data_segments_json(&report.data_segments)
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn artifact_chain_stage_diagnostics_json(stages: &[NsldArtifactStageDiagnostic]) -> String {
    stages
        .iter()
        .map(|stage| {
            let fields = vec![
                json_usize_field("order_index", stage.order_index),
                json_string_field("stage_id", &stage.stage_id),
                json_string_field("file_name", &stage.file_name),
                json_string_field("path", &stage.path),
                json_bool_field("required", stage.required),
                json_bool_field("present", stage.present),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn nsld_link_units_emit_report_json(report: &NsldLinkUnitsEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_units_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_usize_field("unit_count", report.unit_count),
        json_usize_field("hetero_unit_count", report.hetero_unit_count),
        json_usize_field("link_input_count", report.link_input_count),
        json_string_field("unit_table_hash", &report.unit_table_hash),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_units_verify_report_json(report: &NsldLinkUnitsVerifyReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_units_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_usize_field("expected_unit_count", report.expected_unit_count),
        json_usize_field(
            "expected_hetero_unit_count",
            report.expected_hetero_unit_count,
        ),
        json_usize_field(
            "expected_link_input_count",
            report.expected_link_input_count,
        ),
        json_string_field("expected_unit_table_hash", &report.expected_unit_table_hash),
        json_optional_usize_field("actual_unit_count", report.actual_unit_count),
        json_optional_usize_field("actual_hetero_unit_count", report.actual_hetero_unit_count),
        json_optional_usize_field("actual_link_input_count", report.actual_link_input_count),
        json_optional_string_field(
            "actual_unit_table_hash",
            report.actual_unit_table_hash.as_deref(),
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_bundle_report_json(report: &NsldLinkBundleReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_bundle"),
        json_string_field("manifest", &report.manifest),
        json_string_field("bundle_id", &report.bundle_id),
        json_string_field("bundle_hash", &report.bundle_hash),
        json_bool_field("bundle_ready", report.bundle_ready),
        json_usize_field("unit_count", report.unit_count),
        json_usize_field("hetero_unit_count", report.hetero_unit_count),
        json_usize_field("link_input_count", report.link_input_count),
        json_usize_field("link_input_total_bytes", report.link_input_total_bytes),
        json_string_field("link_input_table_hash", &report.link_input_table_hash),
        json_string_field("unit_table_hash", &report.unit_table_hash),
        json_usize_field("clock_edge_count", report.clock_edge_count),
        json_usize_field("data_segment_count", report.data_segment_count),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
        json_bool_field("host_wrapper_required", report.host_wrapper_required),
        json_string_field("compiled_artifact_path", &report.compiled_artifact_path),
        json_string_field("native_output_path", &report.native_output_path),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_bundle_emit_report_json(report: &NsldLinkBundleEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_bundle_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("bundle_id", &report.bundle_id),
        json_string_field("bundle_hash", &report.bundle_hash),
        json_bool_field("bundle_ready", report.bundle_ready),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_bundle_verify_report_json(report: &NsldLinkBundleVerifyReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_bundle_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field("expected_bundle_id", &report.expected_bundle_id),
        json_string_field("expected_bundle_hash", &report.expected_bundle_hash),
        json_optional_string_field("actual_bundle_id", report.actual_bundle_id.as_deref()),
        json_optional_string_field("actual_bundle_hash", report.actual_bundle_hash.as_deref()),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_prepare_report_json(report: &NsldPrepareReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_prepare"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("valid", report.valid),
        json_string_field("output_dir", &report.output_dir),
        json_string_field("link_input_table_path", &report.link_input_table_path),
        json_string_field("link_unit_table_path", &report.link_unit_table_path),
        json_string_field("link_bundle_path", &report.link_bundle_path),
        json_string_field("assemble_plan_path", &report.assemble_plan_path),
        json_string_field("section_manifest_path", &report.section_manifest_path),
        json_string_field("object_plan_path", &report.object_plan_path),
        json_string_field("object_writer_input_path", &report.object_writer_input_path),
        json_string_field("object_byte_layout_path", &report.object_byte_layout_path),
        json_string_field("object_file_layout_path", &report.object_file_layout_path),
        json_string_field(
            "object_image_dry_run_path",
            &report.object_image_dry_run_path,
        ),
        json_string_field(
            "object_image_dry_run_bytes_path",
            &report.object_image_dry_run_bytes_path,
        ),
        json_string_field("object_emit_blocked_path", &report.object_emit_blocked_path),
        json_string_field("object_output_path", &report.object_output_path),
        json_string_field(
            "object_writer_dry_run_path",
            &report.object_writer_dry_run_path,
        ),
        json_string_field("container_plan_path", &report.container_plan_path),
        json_string_field("container_path", &report.container_path),
        json_string_field("container_payload_path", &report.container_payload_path),
        json_string_field("closure_snapshot_path", &report.closure_snapshot_path),
        json_string_field("final_stage_plan_path", &report.final_stage_plan_path),
        json_string_field(
            "final_executable_writer_input_path",
            &report.final_executable_writer_input_path,
        ),
        json_string_field(
            "final_executable_host_invoke_plan_path",
            &report.final_executable_host_invoke_plan_path,
        ),
        json_string_field(
            "final_executable_layout_plan_path",
            &report.final_executable_layout_plan_path,
        ),
        json_string_field(
            "final_executable_image_dry_run_path",
            &report.final_executable_image_dry_run_path,
        ),
        json_string_field(
            "final_executable_image_dry_run_bytes_path",
            &report.final_executable_image_dry_run_bytes_path,
        ),
        json_string_field(
            "final_executable_blocked_path",
            &report.final_executable_blocked_path,
        ),
        json_usize_field("link_input_count", report.link_input_count),
        json_string_field("link_input_table_hash", &report.link_input_table_hash),
        json_usize_field("unit_count", report.unit_count),
        json_string_field("unit_table_hash", &report.unit_table_hash),
        json_string_field("bundle_id", &report.bundle_id),
        json_string_field("bundle_hash", &report.bundle_hash),
        json_bool_field("bundle_ready", report.bundle_ready),
        json_string_field("assemble_plan_hash", &report.assemble_plan_hash),
        json_string_field("section_table_hash", &report.section_table_hash),
        json_string_field("object_plan_hash", &report.object_plan_hash),
        json_bool_field("object_emitted", report.object_emitted),
        json_string_field("byte_layout_hash", &report.byte_layout_hash),
        json_string_field("file_layout_hash", &report.file_layout_hash),
        json_optional_string_field("object_image_hash", report.object_image_hash.as_deref()),
        json_bool_field(
            "object_image_relocation_lowering_valid",
            report.object_image_relocation_lowering_valid,
        ),
        json_usize_field(
            "object_image_relocation_lowering_rule_count",
            report.object_image_relocation_lowering_rule_count,
        ),
        format!(
            "\"object_image_relocation_lowering_rules\":[{}]",
            relocation_lowering_rules_json(&report.object_image_relocation_lowering_rules)
        ),
        json_string_array_field(
            "object_image_relocation_lowering_issues",
            &report.object_image_relocation_lowering_issues,
        ),
        json_usize_field(
            "object_image_relocation_record_count",
            report.object_image_relocation_record_count,
        ),
        json_string_field(
            "object_image_relocation_record_table_hash",
            &report.object_image_relocation_record_table_hash,
        ),
        format!(
            "\"object_image_relocation_records\":[{}]",
            relocation_records_json(&report.object_image_relocation_records)
        ),
        json_string_field("metadata_table_hash", &report.metadata_table_hash),
        json_optional_usize_field(
            "compatibility_domain_count",
            report.compatibility_domain_count,
        ),
        json_optional_string_field(
            "compatibility_domain_table_hash",
            report.compatibility_domain_table_hash.as_deref(),
        ),
        json_optional_string_field(
            "compatibility_domain_id",
            report.compatibility_domain_id.as_deref(),
        ),
        json_optional_string_field(
            "compatibility_domain_kind",
            report.compatibility_domain_kind.as_deref(),
        ),
        json_optional_string_field(
            "compatibility_domain_paradigm",
            report.compatibility_domain_paradigm.as_deref(),
        ),
        json_optional_string_field(
            "compatibility_domain_lifecycle_hook",
            report.compatibility_domain_lifecycle_hook.as_deref(),
        ),
        json_optional_string_field(
            "compatibility_domain_abi_family",
            report.compatibility_domain_abi_family.as_deref(),
        ),
        json_optional_string_field(
            "compatibility_domain_wrapper_policy",
            report.compatibility_domain_wrapper_policy.as_deref(),
        ),
        json_optional_bool_field(
            "compatibility_domain_required",
            report.compatibility_domain_required,
        ),
        format!(
            "\"compatibility_domain_summary\":{}",
            compatibility_domain_summary_json(
                report.compatibility_domain_count,
                report.compatibility_domain_table_hash.as_deref(),
                report.compatibility_domain_id.as_deref(),
                report.compatibility_domain_kind.as_deref(),
                report.compatibility_domain_paradigm.as_deref(),
                report.compatibility_domain_lifecycle_hook.as_deref(),
                report.compatibility_domain_abi_family.as_deref(),
                report.compatibility_domain_wrapper_policy.as_deref(),
                report.compatibility_domain_required,
            )
        ),
        json_string_field("container_layout_hash", &report.container_layout_hash),
        json_string_field("container_hash", &report.container_hash),
        json_usize_field("payload_size_bytes", report.payload_size_bytes),
        json_string_field("payload_hash", &report.payload_hash),
        json_bool_field("final_stage_plan_ready", report.final_stage_plan_ready),
        json_string_field("final_stage_plan_hash", &report.final_stage_plan_hash),
        json_usize_field(
            "final_stage_plan_blocker_count",
            report.final_stage_plan_blocker_count,
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_assemble_plan_report_json(report: &NsldAssemblePlanReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_assemble_plan"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("ready", report.ready),
        json_string_field("bundle_id", &report.bundle_id),
        json_string_field("bundle_hash", &report.bundle_hash),
        json_string_field("assemble_plan_hash", &report.assemble_plan_hash),
        json_usize_field("section_count", report.section_count),
        format!(
            "\"sections\":[{}]",
            nsld_assemble_sections_json(&report.sections)
        ),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_assemble_plan_emit_report_json(report: &NsldAssemblePlanEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_assemble_plan_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_bool_field("ready", report.ready),
        json_string_field("assemble_plan_hash", &report.assemble_plan_hash),
        json_usize_field("section_count", report.section_count),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_assemble_plan_verify_report_json(
    report: &NsldAssemblePlanVerifyReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_assemble_plan_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_assemble_plan_hash",
            &report.expected_assemble_plan_hash,
        ),
        json_usize_field("expected_section_count", report.expected_section_count),
        json_optional_string_field(
            "actual_assemble_plan_hash",
            report.actual_assemble_plan_hash.as_deref(),
        ),
        json_optional_usize_field("actual_section_count", report.actual_section_count),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_section_manifest_report_json(report: &NsldSectionManifestReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_section_manifest"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("ready", report.ready),
        json_string_field("assemble_plan_hash", &report.assemble_plan_hash),
        json_usize_field("section_count", report.section_count),
        json_string_field("section_table_hash", &report.section_table_hash),
        format!(
            "\"sections\":[{}]",
            nsld_assemble_sections_json(&report.sections)
        ),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_section_manifest_emit_report_json(
    report: &NsldSectionManifestEmitReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_section_manifest_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_bool_field("ready", report.ready),
        json_usize_field("section_count", report.section_count),
        json_string_field("section_table_hash", &report.section_table_hash),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_section_manifest_verify_report_json(
    report: &NsldSectionManifestVerifyReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_section_manifest_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_usize_field("expected_section_count", report.expected_section_count),
        json_string_field(
            "expected_section_table_hash",
            &report.expected_section_table_hash,
        ),
        json_optional_usize_field("actual_section_count", report.actual_section_count),
        json_optional_string_field(
            "actual_section_table_hash",
            report.actual_section_table_hash.as_deref(),
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_inputs_emit_report_json(report: &NsldLinkInputsEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_inputs_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_usize_field("link_input_count", report.link_input_count),
        json_usize_field("link_input_total_bytes", report.link_input_total_bytes),
        json_string_field("link_input_table_hash", &report.link_input_table_hash),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_inputs_verify_report_json(report: &NsldLinkInputsVerifyReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_inputs_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_usize_field(
            "expected_link_input_count",
            report.expected_link_input_count,
        ),
        json_usize_field(
            "expected_link_input_total_bytes",
            report.expected_link_input_total_bytes,
        ),
        json_string_field(
            "expected_link_input_table_hash",
            &report.expected_link_input_table_hash,
        ),
        json_optional_usize_field("actual_link_input_count", report.actual_link_input_count),
        json_optional_usize_field(
            "actual_link_input_total_bytes",
            report.actual_link_input_total_bytes,
        ),
        json_optional_string_field(
            "actual_link_input_table_hash",
            report.actual_link_input_table_hash.as_deref(),
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_stage_plan_report_json(report: &NsldFinalStagePlanReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_stage_plan"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("ready", report.ready),
        json_string_field("plan_hash", &report.plan_hash),
        json_string_field("final_stage_kind", &report.final_stage_kind),
        json_string_field("final_stage_driver", &report.final_stage_driver),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
        json_string_field("final_output_path", &report.final_output_path),
        json_bool_field("host_wrapper_required", report.host_wrapper_required),
        json_string_field("compatibility_mode", &report.compatibility_mode),
        json_usize_field("input_count", report.input_count),
        format!("\"inputs\":[{}]", final_stage_inputs_json(&report.inputs)),
        json_string_field("container_hash", &report.container_hash),
        json_string_field("payload_hash", &report.payload_hash),
        json_string_field("linker_contract_hash", &report.linker_contract_hash),
        json_bool_field("native_object_required", report.native_object_required),
        json_bool_field("native_object_present", report.native_object_present),
        json_string_array_field("blockers", &report.blockers),
        json_string_array_field("notes", &report.notes),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_stage_plan_emit_report_json(
    report: &NsldFinalStagePlanEmitReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_stage_plan_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_bool_field("ready", report.ready),
        json_string_field("plan_hash", &report.plan_hash),
        json_usize_field("input_count", report.input_count),
        json_usize_field("blocker_count", report.blocker_count),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_stage_plan_verify_report_json(
    report: &NsldFinalStagePlanVerifyReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_stage_plan_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field("expected_plan_hash", &report.expected_plan_hash),
        json_optional_string_field("actual_plan_hash", report.actual_plan_hash.as_deref()),
        json_usize_field("expected_input_count", report.expected_input_count),
        json_optional_usize_field("actual_input_count", report.actual_input_count),
        json_string_array_field("expected_input_ids", &report.expected_input_ids),
        json_string_array_field("actual_input_ids", &report.actual_input_ids),
        json_usize_field(
            "expected_input_entry_count",
            report.expected_input_entry_count,
        ),
        json_usize_field("actual_input_entry_count", report.actual_input_entry_count),
        json_string_array_field("expected_blockers", &report.expected_blockers),
        json_string_array_field("actual_blockers", &report.actual_blockers),
        json_string_array_field("expected_notes", &report.expected_notes),
        json_string_array_field("actual_notes", &report.actual_notes),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_emit_report_json(
    report: &NsldFinalExecutableEmitReport,
) -> String {
    nsld_final_executable_report_json_with_kind(report, "nsld_final_executable_emit")
}

pub(crate) fn nsld_final_executable_readiness_report_json(
    report: &NsldFinalExecutableEmitReport,
) -> String {
    nsld_final_executable_report_json_with_kind(report, "nsld_final_executable_readiness")
}

pub(crate) fn nsld_final_executable_writer_plan_report_json(
    report: &NsldFinalExecutableWriterPlanReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_writer_plan"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("writer_kind", &report.writer_kind),
        json_string_field("writer_status", &report.writer_status),
        json_string_field("final_stage_plan_hash", &report.final_stage_plan_hash),
        json_string_field("final_stage_driver", &report.final_stage_driver),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
        json_bool_field("host_wrapper_required", report.host_wrapper_required),
        json_usize_field("input_count", report.input_count),
        format!("\"inputs\":[{}]", final_stage_inputs_json(&report.inputs)),
        json_string_array_field("writer_steps", &report.writer_steps),
        json_string_array_field("writer_blockers", &report.writer_blockers),
        json_string_array_field("notes", &report.notes),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_writer_input_emit_report_json(
    report: &NsldFinalExecutableWriterInputEmitReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_writer_input_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("writer_input_hash", &report.writer_input_hash),
        json_string_field("writer_kind", &report.writer_kind),
        json_string_field("writer_status", &report.writer_status),
        json_string_field("final_stage_plan_hash", &report.final_stage_plan_hash),
        json_string_field("final_stage_driver", &report.final_stage_driver),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
        json_bool_field("host_wrapper_required", report.host_wrapper_required),
        json_usize_field("command_arg_count", report.command_arg_count),
        json_string_array_field("writer_blockers", &report.writer_blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_writer_input_verify_report_json(
    report: &NsldFinalExecutableWriterInputVerifyReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_writer_input_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_writer_input_hash",
            &report.expected_writer_input_hash,
        ),
        json_optional_string_field(
            "actual_writer_input_hash",
            report.actual_writer_input_hash.as_deref(),
        ),
        json_string_field(
            "expected_final_stage_plan_hash",
            &report.expected_final_stage_plan_hash,
        ),
        json_optional_string_field(
            "actual_final_stage_plan_hash",
            report.actual_final_stage_plan_hash.as_deref(),
        ),
        json_string_field("expected_writer_kind", &report.expected_writer_kind),
        json_optional_string_field("actual_writer_kind", report.actual_writer_kind.as_deref()),
        json_string_field("expected_writer_status", &report.expected_writer_status),
        json_optional_string_field(
            "actual_writer_status",
            report.actual_writer_status.as_deref(),
        ),
        json_usize_field(
            "expected_command_arg_count",
            report.expected_command_arg_count,
        ),
        json_optional_usize_field("actual_command_arg_count", report.actual_command_arg_count),
        json_string_array_field("expected_command_args", &report.expected_command_args),
        json_string_array_field("actual_command_args", &report.actual_command_args),
        json_string_array_field("expected_writer_blockers", &report.expected_writer_blockers),
        json_string_array_field("actual_writer_blockers", &report.actual_writer_blockers),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_host_dry_run_report_json(
    report: &NsldFinalExecutableHostDryRunReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_host_dry_run"),
        json_string_field("manifest", &report.manifest),
        json_string_field("writer_input_path", &report.writer_input_path),
        json_bool_field("writer_input_valid", report.writer_input_valid),
        json_optional_string_field("writer_input_hash", report.writer_input_hash.as_deref()),
        json_string_field("driver", &report.driver),
        json_bool_field("driver_available", report.driver_available),
        json_optional_string_field(
            "driver_resolved_path",
            report.driver_resolved_path.as_deref(),
        ),
        json_usize_field("command_arg_count", report.command_arg_count),
        json_string_array_field("command_args", &report.command_args),
        json_bool_field("environment_ready", report.environment_ready),
        json_string_field("invocation_policy", &report.invocation_policy),
        json_string_field("invocation_policy_reason", &report.invocation_policy_reason),
        json_bool_field(
            "can_invoke_host_finalizer",
            report.can_invoke_host_finalizer,
        ),
        json_string_array_field("blockers", &report.blockers),
        json_string_array_field("notes", &report.notes),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_host_invoke_plan_report_json(
    report: &NsldFinalExecutableHostInvokePlanReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_host_invoke_plan"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("writer_input_path", &report.writer_input_path),
        json_string_field("invocation_kind", &report.invocation_kind),
        json_string_field("invocation_policy", &report.invocation_policy),
        json_string_field("invocation_policy_reason", &report.invocation_policy_reason),
        json_bool_field("requires_explicit_allow", report.requires_explicit_allow),
        json_bool_field("explicit_allow_present", report.explicit_allow_present),
        json_bool_field("environment_ready", report.environment_ready),
        json_bool_field("driver_available", report.driver_available),
        json_optional_string_field(
            "driver_resolved_path",
            report.driver_resolved_path.as_deref(),
        ),
        json_bool_field(
            "can_invoke_host_finalizer",
            report.can_invoke_host_finalizer,
        ),
        json_bool_field("would_invoke", report.would_invoke),
        json_usize_field("command_arg_count", report.command_arg_count),
        json_string_array_field("command_args", &report.command_args),
        json_string_array_field("blockers", &report.blockers),
        json_string_array_field("notes", &report.notes),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_host_invoke_plan_emit_report_json(
    report: &NsldFinalExecutableHostInvokePlanEmitReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_host_invoke_plan_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("invoke_plan_hash", &report.invoke_plan_hash),
        json_string_field("invocation_policy", &report.invocation_policy),
        json_bool_field("requires_explicit_allow", report.requires_explicit_allow),
        json_bool_field("explicit_allow_present", report.explicit_allow_present),
        json_bool_field("would_invoke", report.would_invoke),
        json_usize_field("blocker_count", report.blocker_count),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_host_invoke_plan_verify_report_json(
    report: &NsldFinalExecutableHostInvokePlanVerifyReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_host_invoke_plan_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_invoke_plan_hash",
            &report.expected_invoke_plan_hash,
        ),
        json_optional_string_field(
            "actual_invoke_plan_hash",
            report.actual_invoke_plan_hash.as_deref(),
        ),
        json_string_field(
            "expected_invocation_policy",
            &report.expected_invocation_policy,
        ),
        json_optional_string_field(
            "actual_invocation_policy",
            report.actual_invocation_policy.as_deref(),
        ),
        json_bool_field(
            "expected_requires_explicit_allow",
            report.expected_requires_explicit_allow,
        ),
        json_optional_bool_field(
            "actual_requires_explicit_allow",
            report.actual_requires_explicit_allow,
        ),
        json_bool_field(
            "expected_explicit_allow_present",
            report.expected_explicit_allow_present,
        ),
        json_optional_bool_field(
            "actual_explicit_allow_present",
            report.actual_explicit_allow_present,
        ),
        json_bool_field("expected_would_invoke", report.expected_would_invoke),
        json_optional_bool_field("actual_would_invoke", report.actual_would_invoke),
        json_usize_field(
            "expected_command_arg_count",
            report.expected_command_arg_count,
        ),
        json_optional_usize_field("actual_command_arg_count", report.actual_command_arg_count),
        json_string_array_field("expected_command_args", &report.expected_command_args),
        json_string_array_field("actual_command_args", &report.actual_command_args),
        json_usize_field("expected_blocker_count", report.expected_blocker_count),
        json_optional_usize_field("actual_blocker_count", report.actual_blocker_count),
        json_string_array_field("expected_blockers", &report.expected_blockers),
        json_string_array_field("actual_blockers", &report.actual_blockers),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_layout_plan_report_json(
    report: &NsldFinalExecutableLayoutPlanReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_layout_plan"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("layout_hash", &report.layout_hash),
        json_string_field("final_stage_plan_hash", &report.final_stage_plan_hash),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
        json_string_field("platform_envelope_family", &report.platform_envelope_family),
        json_string_field("platform_envelope_policy", &report.platform_envelope_policy),
        json_string_field("internal_binary_format", &report.internal_binary_format),
        json_string_field("lifecycle_entry_hook", &report.lifecycle_entry_hook),
        json_string_field("scheduler_contract", &report.scheduler_contract),
        json_string_field("data_segment_ordering", &report.data_segment_ordering),
        json_string_field("native_object_path", &report.native_object_path),
        json_bool_field("native_object_required", report.native_object_required),
        json_bool_field("native_object_present", report.native_object_present),
        json_string_field("compatibility_domain", &report.compatibility_domain),
        json_string_field(
            "compatibility_lifecycle_hook",
            &report.compatibility_lifecycle_hook,
        ),
        json_usize_field("payload_count", report.payload_count),
        json_string_array_field("payloads", &report.payload_names),
        json_usize_field("byte_alignment", report.byte_alignment),
        json_usize_field("byte_span", report.byte_span),
        json_string_field("byte_map_hash", &report.byte_map_hash),
        format!(
            "\"payload_diagnostics\":[{}]",
            final_executable_payload_diagnostics_json(&report.payloads)
        ),
        format!(
            "\"byte_map_entries\":[{}]",
            final_executable_byte_map_entries_json(&report.byte_map_entries)
        ),
        json_string_array_field("notes", &report.notes),
    ];
    format!("{{{}}}", fields.join(","))
}

fn final_executable_payload_diagnostics_json(
    payloads: &[NsldFinalExecutablePayloadDiagnostic],
) -> String {
    payloads
        .iter()
        .map(|payload| {
            let fields = vec![
                json_usize_field("order_index", payload.order_index),
                json_string_field("payload_id", &payload.payload_id),
                json_string_field("payload_kind", &payload.payload_kind),
                json_string_field("lifecycle_hook", &payload.lifecycle_hook),
                json_string_field("path", &payload.path),
                json_string_field("content_hash", &payload.content_hash),
                json_bool_field("required", payload.required),
                json_bool_field("present", payload.present),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn final_executable_byte_map_entries_json(entries: &[NsldFinalExecutableByteMapEntry]) -> String {
    entries
        .iter()
        .map(|entry| {
            let fields = vec![
                json_usize_field("order_index", entry.order_index),
                json_string_field("payload_id", &entry.payload_id),
                json_string_field("payload_kind", &entry.payload_kind),
                json_usize_field("offset", entry.offset),
                json_usize_field("size_bytes", entry.size_bytes),
                json_usize_field("alignment", entry.alignment),
                json_string_field("content_hash", &entry.content_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn nsld_final_executable_layout_plan_emit_report_json(
    report: &NsldFinalExecutableLayoutPlanEmitReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_layout_plan_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("layout_hash", &report.layout_hash),
        json_string_field("final_stage_plan_hash", &report.final_stage_plan_hash),
        json_usize_field("payload_count", report.payload_count),
        json_bool_field("native_object_present", report.native_object_present),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_layout_plan_verify_report_json(
    report: &NsldFinalExecutableLayoutPlanVerifyReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_layout_plan_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field("expected_layout_hash", &report.expected_layout_hash),
        json_optional_string_field("actual_layout_hash", report.actual_layout_hash.as_deref()),
        json_usize_field("expected_payload_count", report.expected_payload_count),
        json_optional_usize_field("actual_payload_count", report.actual_payload_count),
        json_string_array_field("expected_payloads", &report.expected_payloads),
        json_string_array_field("actual_payloads", &report.actual_payloads),
        json_usize_field(
            "expected_payload_entry_count",
            report.expected_payload_entry_count,
        ),
        json_usize_field(
            "actual_payload_entry_count",
            report.actual_payload_entry_count,
        ),
        json_usize_field(
            "expected_byte_map_entry_count",
            report.expected_byte_map_entry_count,
        ),
        json_usize_field(
            "actual_byte_map_entry_count",
            report.actual_byte_map_entry_count,
        ),
        json_usize_field("expected_byte_span", report.expected_byte_span),
        json_optional_usize_field("actual_byte_span", report.actual_byte_span),
        json_string_field("expected_byte_map_hash", &report.expected_byte_map_hash),
        json_optional_string_field(
            "actual_byte_map_hash",
            report.actual_byte_map_hash.as_deref(),
        ),
        json_string_field(
            "expected_lifecycle_entry_hook",
            &report.expected_lifecycle_entry_hook,
        ),
        json_optional_string_field(
            "actual_lifecycle_entry_hook",
            report.actual_lifecycle_entry_hook.as_deref(),
        ),
        json_string_field(
            "expected_platform_envelope_family",
            &report.expected_platform_envelope_family,
        ),
        json_optional_string_field(
            "actual_platform_envelope_family",
            report.actual_platform_envelope_family.as_deref(),
        ),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_image_dry_run_report_json(
    report: &NsldFinalExecutableImageDryRunReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_image_dry_run"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("image_path", &report.image_path),
        json_string_field("image_format", &report.image_format),
        json_string_field("image_magic", &report.image_magic),
        json_usize_field("image_header_size", report.image_header_size),
        json_usize_field("payload_byte_offset", report.payload_byte_offset),
        json_usize_field("payload_byte_span", report.payload_byte_span),
        json_string_field("layout_hash", &report.layout_hash),
        json_string_field("byte_map_hash", &report.byte_map_hash),
        json_usize_field("payload_count", report.payload_count),
        json_usize_field("byte_span", report.byte_span),
        json_bool_field("image_constructed", report.image_constructed),
        json_bool_field("image_ready", report.image_ready),
        json_optional_usize_field("image_size_bytes", report.image_size_bytes),
        json_optional_string_field("image_hash", report.image_hash.as_deref()),
        json_string_array_field("blockers", &report.blockers),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_image_dry_run_emit_report_json(
    report: &NsldFinalExecutableImageDryRunEmitReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_image_dry_run_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("image_path", &report.image_path),
        json_bool_field("image_emitted", report.image_emitted),
        json_bool_field("image_constructed", report.image_constructed),
        json_bool_field("image_ready", report.image_ready),
        json_string_field("image_format", &report.image_format),
        json_usize_field("image_header_size", report.image_header_size),
        json_usize_field("payload_byte_offset", report.payload_byte_offset),
        json_optional_usize_field("image_size_bytes", report.image_size_bytes),
        json_optional_string_field("image_hash", report.image_hash.as_deref()),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_image_dry_run_verify_report_json(
    report: &NsldFinalExecutableImageDryRunVerifyReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_image_dry_run_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_string_field("image_path", &report.image_path),
        json_bool_field("valid", report.valid),
        json_string_field("expected_layout_hash", &report.expected_layout_hash),
        json_optional_string_field("actual_layout_hash", report.actual_layout_hash.as_deref()),
        json_string_field("expected_byte_map_hash", &report.expected_byte_map_hash),
        json_optional_string_field(
            "actual_byte_map_hash",
            report.actual_byte_map_hash.as_deref(),
        ),
        json_string_field("expected_image_magic", &report.expected_image_magic),
        json_optional_string_field("actual_image_magic", report.actual_image_magic.as_deref()),
        json_usize_field(
            "expected_image_version",
            report.expected_image_version as usize,
        ),
        json_optional_usize_field(
            "actual_image_version",
            report.actual_image_version.map(|value| value as usize),
        ),
        json_usize_field(
            "expected_image_header_size",
            report.expected_image_header_size,
        ),
        json_optional_usize_field("actual_image_header_size", report.actual_image_header_size),
        json_usize_field(
            "expected_payload_byte_offset",
            report.expected_payload_byte_offset,
        ),
        json_optional_usize_field(
            "actual_payload_byte_offset",
            report.actual_payload_byte_offset,
        ),
        json_usize_field(
            "expected_payload_byte_span",
            report.expected_payload_byte_span,
        ),
        json_optional_usize_field("actual_payload_byte_span", report.actual_payload_byte_span),
        json_optional_string_field(
            "actual_header_layout_hash",
            report.actual_header_layout_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_header_byte_map_hash",
            report.actual_header_byte_map_hash.as_deref(),
        ),
        json_usize_field(
            "expected_payload_region_count",
            report.expected_payload_region_count,
        ),
        json_optional_usize_field(
            "actual_payload_region_count",
            report.actual_payload_region_count,
        ),
        json_optional_string_field(
            "expected_payload_region_hash",
            report.expected_payload_region_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_payload_region_hash",
            report.actual_payload_region_hash.as_deref(),
        ),
        json_bool_field(
            "expected_image_constructed",
            report.expected_image_constructed,
        ),
        json_optional_bool_field("actual_image_constructed", report.actual_image_constructed),
        json_bool_field("expected_image_ready", report.expected_image_ready),
        json_optional_bool_field("actual_image_ready", report.actual_image_ready),
        json_optional_usize_field(
            "expected_image_size_bytes",
            report.expected_image_size_bytes,
        ),
        json_optional_usize_field("actual_image_size_bytes", report.actual_image_size_bytes),
        json_optional_string_field("expected_image_hash", report.expected_image_hash.as_deref()),
        json_optional_string_field("actual_image_hash", report.actual_image_hash.as_deref()),
        json_string_array_field("expected_blockers", &report.expected_blockers),
        json_string_array_field("actual_blockers", &report.actual_blockers),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn nsld_final_executable_report_json_with_kind(
    report: &NsldFinalExecutableEmitReport,
    kind: &str,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", kind),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("blocked_report_path", &report.blocked_report_path),
        json_bool_field("emitted", report.emitted),
        json_bool_field(
            "can_emit_final_executable",
            report.can_emit_final_executable,
        ),
        json_bool_field("final_stage_ready", report.final_stage_ready),
        json_string_field("final_stage_plan_hash", &report.final_stage_plan_hash),
        json_string_field("final_stage_driver", &report.final_stage_driver),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
        json_bool_field("host_wrapper_required", report.host_wrapper_required),
        json_string_field("writer_kind", &report.writer_kind),
        json_string_field("writer_status", &report.writer_status),
        json_string_array_field("writer_blockers", &report.writer_blockers),
        json_string_field("writer_input_path", &report.writer_input_path),
        json_optional_bool_field("writer_input_valid", report.writer_input_valid),
        json_optional_string_field("writer_input_hash", report.writer_input_hash.as_deref()),
        json_string_array_field("writer_input_issues", &report.writer_input_issues),
        json_optional_bool_field(
            "host_dry_run_environment_ready",
            report.host_dry_run_environment_ready,
        ),
        json_optional_bool_field(
            "host_dry_run_driver_available",
            report.host_dry_run_driver_available,
        ),
        json_optional_string_field(
            "host_dry_run_driver_resolved_path",
            report.host_dry_run_driver_resolved_path.as_deref(),
        ),
        json_optional_bool_field("host_dry_run_can_invoke", report.host_dry_run_can_invoke),
        json_optional_string_field(
            "host_dry_run_invocation_policy",
            report.host_dry_run_invocation_policy.as_deref(),
        ),
        json_optional_string_field(
            "host_dry_run_invocation_policy_reason",
            report.host_dry_run_invocation_policy_reason.as_deref(),
        ),
        json_usize_field(
            "host_dry_run_command_arg_count",
            report.host_dry_run_command_arg_count,
        ),
        json_string_array_field(
            "host_dry_run_command_args",
            &report.host_dry_run_command_args,
        ),
        json_usize_field(
            "host_dry_run_blocker_count",
            report.host_dry_run_blocker_count,
        ),
        json_string_array_field("host_dry_run_blockers", &report.host_dry_run_blockers),
        json_string_field("host_invoke_plan_path", &report.host_invoke_plan_path),
        json_optional_bool_field("host_invoke_plan_valid", report.host_invoke_plan_valid),
        json_optional_string_field(
            "host_invoke_plan_hash",
            report.host_invoke_plan_hash.as_deref(),
        ),
        json_optional_string_field(
            "host_invoke_plan_invocation_policy",
            report.host_invoke_plan_invocation_policy.as_deref(),
        ),
        json_optional_bool_field(
            "host_invoke_plan_requires_explicit_allow",
            report.host_invoke_plan_requires_explicit_allow,
        ),
        json_optional_bool_field(
            "host_invoke_plan_explicit_allow_present",
            report.host_invoke_plan_explicit_allow_present,
        ),
        json_optional_bool_field(
            "host_invoke_plan_would_invoke",
            report.host_invoke_plan_would_invoke,
        ),
        json_optional_usize_field(
            "host_invoke_plan_blocker_count",
            report.host_invoke_plan_blocker_count,
        ),
        json_string_array_field("host_invoke_plan_issues", &report.host_invoke_plan_issues),
        json_string_field("layout_plan_path", &report.layout_plan_path),
        json_optional_bool_field("layout_plan_valid", report.layout_plan_valid),
        json_optional_string_field("layout_plan_hash", report.layout_plan_hash.as_deref()),
        json_string_array_field("layout_plan_issues", &report.layout_plan_issues),
        json_string_field("image_dry_run_path", &report.image_dry_run_path),
        json_string_field("image_dry_run_bytes_path", &report.image_dry_run_bytes_path),
        json_optional_bool_field("image_dry_run_valid", report.image_dry_run_valid),
        json_optional_string_field("image_dry_run_hash", report.image_dry_run_hash.as_deref()),
        json_optional_usize_field("image_dry_run_size_bytes", report.image_dry_run_size_bytes),
        json_string_array_field("image_dry_run_issues", &report.image_dry_run_issues),
        json_usize_field("input_count", report.input_count),
        json_usize_field("blocker_count", report.blockers.len()),
        json_string_array_field("blockers", &report.blockers),
        json_string_array_field("notes", &report.notes),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_emit_verify_report_json(
    report: &NsldFinalExecutableEmitVerifyReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_emit_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_final_stage_plan_hash",
            &report.expected_final_stage_plan_hash,
        ),
        json_optional_string_field(
            "actual_final_stage_plan_hash",
            report.actual_final_stage_plan_hash.as_deref(),
        ),
        json_bool_field("expected_emitted", report.expected_emitted),
        json_optional_bool_field("actual_emitted", report.actual_emitted),
        json_optional_bool_field(
            "expected_writer_input_valid",
            report.expected_writer_input_valid,
        ),
        json_optional_bool_field(
            "actual_writer_input_valid",
            report.actual_writer_input_valid,
        ),
        json_optional_string_field(
            "expected_writer_input_hash",
            report.expected_writer_input_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_writer_input_hash",
            report.actual_writer_input_hash.as_deref(),
        ),
        json_string_array_field(
            "expected_writer_input_issues",
            &report.expected_writer_input_issues,
        ),
        json_string_array_field(
            "actual_writer_input_issues",
            &report.actual_writer_input_issues,
        ),
        json_optional_bool_field(
            "expected_host_dry_run_environment_ready",
            report.expected_host_dry_run_environment_ready,
        ),
        json_optional_bool_field(
            "actual_host_dry_run_environment_ready",
            report.actual_host_dry_run_environment_ready,
        ),
        json_optional_bool_field(
            "expected_host_dry_run_driver_available",
            report.expected_host_dry_run_driver_available,
        ),
        json_optional_bool_field(
            "actual_host_dry_run_driver_available",
            report.actual_host_dry_run_driver_available,
        ),
        json_optional_bool_field(
            "expected_host_dry_run_can_invoke",
            report.expected_host_dry_run_can_invoke,
        ),
        json_optional_bool_field(
            "actual_host_dry_run_can_invoke",
            report.actual_host_dry_run_can_invoke,
        ),
        json_optional_string_field(
            "expected_host_dry_run_driver_resolved_path",
            report.expected_host_dry_run_driver_resolved_path.as_deref(),
        ),
        json_optional_string_field(
            "actual_host_dry_run_driver_resolved_path",
            report.actual_host_dry_run_driver_resolved_path.as_deref(),
        ),
        json_optional_string_field(
            "expected_host_dry_run_invocation_policy",
            report.expected_host_dry_run_invocation_policy.as_deref(),
        ),
        json_optional_string_field(
            "actual_host_dry_run_invocation_policy",
            report.actual_host_dry_run_invocation_policy.as_deref(),
        ),
        json_optional_string_field(
            "expected_host_dry_run_invocation_policy_reason",
            report
                .expected_host_dry_run_invocation_policy_reason
                .as_deref(),
        ),
        json_optional_string_field(
            "actual_host_dry_run_invocation_policy_reason",
            report
                .actual_host_dry_run_invocation_policy_reason
                .as_deref(),
        ),
        json_usize_field(
            "expected_host_dry_run_command_arg_count",
            report.expected_host_dry_run_command_arg_count,
        ),
        json_optional_usize_field(
            "actual_host_dry_run_command_arg_count",
            report.actual_host_dry_run_command_arg_count,
        ),
        json_string_array_field(
            "expected_host_dry_run_command_args",
            &report.expected_host_dry_run_command_args,
        ),
        json_string_array_field(
            "actual_host_dry_run_command_args",
            &report.actual_host_dry_run_command_args,
        ),
        json_usize_field(
            "expected_host_dry_run_blocker_count",
            report.expected_host_dry_run_blocker_count,
        ),
        json_optional_usize_field(
            "actual_host_dry_run_blocker_count",
            report.actual_host_dry_run_blocker_count,
        ),
        json_string_array_field(
            "expected_host_dry_run_blockers",
            &report.expected_host_dry_run_blockers,
        ),
        json_string_array_field(
            "actual_host_dry_run_blockers",
            &report.actual_host_dry_run_blockers,
        ),
        json_optional_bool_field(
            "expected_host_invoke_plan_valid",
            report.expected_host_invoke_plan_valid,
        ),
        json_optional_bool_field(
            "actual_host_invoke_plan_valid",
            report.actual_host_invoke_plan_valid,
        ),
        json_optional_bool_field(
            "expected_host_invoke_plan_would_invoke",
            report.expected_host_invoke_plan_would_invoke,
        ),
        json_optional_bool_field(
            "actual_host_invoke_plan_would_invoke",
            report.actual_host_invoke_plan_would_invoke,
        ),
        json_optional_string_field(
            "expected_host_invoke_plan_hash",
            report.expected_host_invoke_plan_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_host_invoke_plan_hash",
            report.actual_host_invoke_plan_hash.as_deref(),
        ),
        json_optional_string_field(
            "expected_host_invoke_plan_invocation_policy",
            report
                .expected_host_invoke_plan_invocation_policy
                .as_deref(),
        ),
        json_optional_string_field(
            "actual_host_invoke_plan_invocation_policy",
            report.actual_host_invoke_plan_invocation_policy.as_deref(),
        ),
        json_optional_bool_field(
            "expected_host_invoke_plan_requires_explicit_allow",
            report.expected_host_invoke_plan_requires_explicit_allow,
        ),
        json_optional_bool_field(
            "actual_host_invoke_plan_requires_explicit_allow",
            report.actual_host_invoke_plan_requires_explicit_allow,
        ),
        json_optional_bool_field(
            "expected_host_invoke_plan_explicit_allow_present",
            report.expected_host_invoke_plan_explicit_allow_present,
        ),
        json_optional_bool_field(
            "actual_host_invoke_plan_explicit_allow_present",
            report.actual_host_invoke_plan_explicit_allow_present,
        ),
        json_optional_usize_field(
            "expected_host_invoke_plan_blocker_count",
            report.expected_host_invoke_plan_blocker_count,
        ),
        json_optional_usize_field(
            "actual_host_invoke_plan_blocker_count",
            report.actual_host_invoke_plan_blocker_count,
        ),
        json_string_array_field(
            "expected_host_invoke_plan_issues",
            &report.expected_host_invoke_plan_issues,
        ),
        json_string_array_field(
            "actual_host_invoke_plan_issues",
            &report.actual_host_invoke_plan_issues,
        ),
        json_optional_bool_field(
            "expected_layout_plan_valid",
            report.expected_layout_plan_valid,
        ),
        json_optional_bool_field("actual_layout_plan_valid", report.actual_layout_plan_valid),
        json_optional_string_field(
            "expected_layout_plan_hash",
            report.expected_layout_plan_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_layout_plan_hash",
            report.actual_layout_plan_hash.as_deref(),
        ),
        json_string_array_field(
            "expected_layout_plan_issues",
            &report.expected_layout_plan_issues,
        ),
        json_string_array_field(
            "actual_layout_plan_issues",
            &report.actual_layout_plan_issues,
        ),
        json_optional_bool_field(
            "expected_image_dry_run_valid",
            report.expected_image_dry_run_valid,
        ),
        json_optional_bool_field(
            "actual_image_dry_run_valid",
            report.actual_image_dry_run_valid,
        ),
        json_optional_string_field(
            "expected_image_dry_run_hash",
            report.expected_image_dry_run_hash.as_deref(),
        ),
        json_optional_string_field(
            "actual_image_dry_run_hash",
            report.actual_image_dry_run_hash.as_deref(),
        ),
        json_optional_usize_field(
            "expected_image_dry_run_size_bytes",
            report.expected_image_dry_run_size_bytes,
        ),
        json_optional_usize_field(
            "actual_image_dry_run_size_bytes",
            report.actual_image_dry_run_size_bytes,
        ),
        json_string_array_field(
            "expected_image_dry_run_issues",
            &report.expected_image_dry_run_issues,
        ),
        json_string_array_field(
            "actual_image_dry_run_issues",
            &report.actual_image_dry_run_issues,
        ),
        json_usize_field("expected_blocker_count", report.expected_blocker_count),
        json_optional_usize_field("actual_blocker_count", report.actual_blocker_count),
        json_string_array_field("expected_blockers", &report.expected_blockers),
        json_string_array_field("actual_blockers", &report.actual_blockers),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_output_report_json(
    report: &NsldFinalExecutableOutputReport,
) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_output"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_bool_field("present", report.present),
        json_optional_usize_field("size_bytes", report.size_bytes),
        json_optional_string_field("output_hash", report.output_hash.as_deref()),
        json_bool_field("final_stage_plan_valid", report.final_stage_plan_valid),
        json_optional_string_field(
            "final_stage_plan_hash",
            report.final_stage_plan_hash.as_deref(),
        ),
        json_bool_field(
            "final_executable_emit_valid",
            report.final_executable_emit_valid,
        ),
        json_optional_bool_field("final_executable_emitted", report.final_executable_emitted),
        json_optional_usize_field(
            "final_executable_blocker_count",
            report.final_executable_blocker_count,
        ),
        json_bool_field("runnable_candidate", report.runnable_candidate),
        json_string_array_field("blockers", &report.blockers),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

fn final_stage_inputs_json(inputs: &[NsldFinalStageInputDiagnostic]) -> String {
    inputs
        .iter()
        .map(|input| {
            let fields = vec![
                json_usize_field("order_index", input.order_index),
                json_string_field("input_id", &input.input_id),
                json_string_field("input_kind", &input.input_kind),
                json_string_field("path", &input.path),
                json_string_field("content_hash", &input.content_hash),
                json_bool_field("required", input.required),
                json_bool_field("present", input.present),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn nsld_closure_report_json(report: &NsldClosureReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_linker_closure"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("closed", report.closed),
        json_string_array_field("internal_contracts", &report.internal_contracts),
        json_string_field("linker_contract_hash", &report.linker_contract_hash),
        format!(
            "\"link_inputs\":[{}]",
            nsld_link_inputs_json(&report.link_inputs)
        ),
        json_usize_field("link_input_count", report.link_input_count),
        json_usize_field("link_input_total_bytes", report.link_input_total_bytes),
        json_string_field("link_input_table_hash", &report.link_input_table_hash),
        json_bool_field("link_input_table_present", report.link_input_table_present),
        json_optional_bool_field("link_input_table_valid", report.link_input_table_valid),
        json_bool_field(
            "prepared_artifact_chain_valid",
            report.prepared_artifact_chain_valid,
        ),
        json_string_array_field(
            "prepared_artifact_chain_issues",
            &report.prepared_artifact_chain_issues,
        ),
        json_string_field(
            "container_metadata_table_hash",
            &report.container_metadata_table_hash,
        ),
        json_string_field("container_layout_hash", &report.container_layout_hash),
        json_string_field("container_hash", &report.container_hash),
        json_usize_field("payload_size_bytes", report.payload_size_bytes),
        json_string_field("payload_hash", &report.payload_hash),
        json_string_field(
            "container_loader_readiness",
            &report.container_loader_readiness,
        ),
        json_usize_field(
            "compatibility_domain_count",
            report.compatibility_domain_count,
        ),
        json_string_field(
            "compatibility_domain_table_hash",
            &report.compatibility_domain_table_hash,
        ),
        json_optional_string_field(
            "compatibility_domain_id",
            report.compatibility_domain_id.as_deref(),
        ),
        json_optional_string_field(
            "compatibility_domain_kind",
            report.compatibility_domain_kind.as_deref(),
        ),
        json_optional_string_field(
            "compatibility_domain_paradigm",
            report.compatibility_domain_paradigm.as_deref(),
        ),
        json_optional_string_field(
            "compatibility_domain_lifecycle_hook",
            report.compatibility_domain_lifecycle_hook.as_deref(),
        ),
        json_optional_string_field(
            "compatibility_domain_abi_family",
            report.compatibility_domain_abi_family.as_deref(),
        ),
        json_optional_string_field(
            "compatibility_domain_wrapper_policy",
            report.compatibility_domain_wrapper_policy.as_deref(),
        ),
        json_optional_bool_field(
            "compatibility_domain_required",
            report.compatibility_domain_required,
        ),
        format!(
            "\"compatibility_domain_summary\":{}",
            compatibility_domain_summary_json(
                Some(report.compatibility_domain_count),
                Some(&report.compatibility_domain_table_hash),
                report.compatibility_domain_id.as_deref(),
                report.compatibility_domain_kind.as_deref(),
                report.compatibility_domain_paradigm.as_deref(),
                report.compatibility_domain_lifecycle_hook.as_deref(),
                report.compatibility_domain_abi_family.as_deref(),
                report.compatibility_domain_wrapper_policy.as_deref(),
                report.compatibility_domain_required,
            )
        ),
        json_optional_bool_field(
            "object_image_relocation_lowering_valid",
            report.object_image_relocation_lowering_valid,
        ),
        json_optional_usize_field(
            "object_image_relocation_lowering_rule_count",
            report.object_image_relocation_lowering_rule_count,
        ),
        format!(
            "\"object_image_relocation_lowering_rules\":[{}]",
            relocation_lowering_rules_json(&report.object_image_relocation_lowering_rules)
        ),
        json_string_array_field(
            "object_image_relocation_lowering_issues",
            &report.object_image_relocation_lowering_issues,
        ),
        json_optional_usize_field(
            "object_image_relocation_record_count",
            report.object_image_relocation_record_count,
        ),
        json_optional_string_field(
            "object_image_relocation_record_table_hash",
            report.object_image_relocation_record_table_hash.as_deref(),
        ),
        format!(
            "\"object_image_relocation_records\":[{}]",
            relocation_records_json(&report.object_image_relocation_records)
        ),
        json_string_array_field("external_dependencies", &report.external_dependencies),
        json_string_array_field("unresolved", &report.unresolved),
        json_bool_field("host_wrapper_required", report.host_wrapper_required),
        json_usize_field("domain_count", report.domain_count),
        json_usize_field("hetero_domain_count", report.hetero_domain_count),
        json_usize_field("sidecar_capability_count", report.sidecar_capability_count),
        json_usize_field("clock_edge_count", report.clock_edge_count),
        json_usize_field("data_segment_count", report.data_segment_count),
        json_string_field("final_stage_link_mode", &report.final_stage_link_mode),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_closure_emit_report_json(report: &NsldClosureEmitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_linker_closure_emit"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("linker_contract_hash", &report.linker_contract_hash),
        json_bool_field("closed", report.closed),
        json_usize_field("internal_contract_count", report.internal_contract_count),
        json_usize_field("unresolved_count", report.unresolved_count),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_closure_verify_report_json(report: &NsldClosureVerifyReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_linker_closure_verify"),
        json_string_field("manifest", &report.manifest),
        json_string_field("input_path", &report.input_path),
        json_bool_field("valid", report.valid),
        json_string_field(
            "expected_linker_contract_hash",
            &report.expected_linker_contract_hash,
        ),
        json_optional_string_field(
            "actual_linker_contract_hash",
            report.actual_linker_contract_hash.as_deref(),
        ),
        json_string_field("expected_container_hash", &report.expected_container_hash),
        json_optional_string_field(
            "actual_container_hash",
            report.actual_container_hash.as_deref(),
        ),
        json_usize_field(
            "expected_payload_size_bytes",
            report.expected_payload_size_bytes,
        ),
        json_optional_usize_field(
            "actual_payload_size_bytes",
            report.actual_payload_size_bytes,
        ),
        json_string_field("expected_payload_hash", &report.expected_payload_hash),
        json_optional_string_field("actual_payload_hash", report.actual_payload_hash.as_deref()),
        json_bool_field("expected_closed", report.expected_closed),
        json_optional_bool_field("actual_closed", report.actual_closed),
        json_usize_field(
            "expected_internal_contract_count",
            report.expected_internal_contract_count,
        ),
        json_optional_usize_field(
            "actual_internal_contract_count",
            report.actual_internal_contract_count,
        ),
        json_usize_field(
            "expected_unresolved_count",
            report.expected_unresolved_count,
        ),
        json_optional_usize_field("actual_unresolved_count", report.actual_unresolved_count),
        json_string_array_field("issues", &report.issues),
    ];
    format!("{{{}}}", fields.join(","))
}

pub(crate) fn nsld_link_unit_report_json(report: &NsldLinkUnitReport) -> String {
    let fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_link_units"),
        json_string_field("manifest", &report.manifest),
        json_usize_field("unit_count", report.unit_count),
        json_usize_field("hetero_unit_count", report.hetero_unit_count),
        json_usize_field("link_input_count", report.link_input_count),
        json_usize_field("clock_edge_count", report.clock_edge_count),
        json_usize_field("data_segment_count", report.data_segment_count),
        json_string_field("unit_table_hash", &report.unit_table_hash),
        format!("\"units\":[{}]", nsld_link_units_json(&report.units)),
    ];
    format!("{{{}}}", fields.join(","))
}
