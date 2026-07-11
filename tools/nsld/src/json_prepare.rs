use super::{json_fields::*, json_fragments::*, json_object_image::*, reports::*};

pub(crate) fn nsld_prepare_report_json(report: &NsldPrepareReport) -> String {
    let fields = [
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
        json_string_field(
            "final_executable_output_path",
            &report.final_executable_output_path,
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
    let fields = [
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
    let fields = [
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
    let fields = [
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
    let fields = [
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
    let fields = [
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
    let fields = [
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
    let fields = [
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
    let fields = [
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
