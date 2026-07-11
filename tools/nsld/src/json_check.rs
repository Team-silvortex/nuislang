use super::{
    json_check_final::check_report_final_fields,
    json_check_sections::{
        check_report_container_fields, check_report_object_fields, check_report_tail_fields,
    },
    json_fields::*,
    reports::NsldCheckReport,
};

pub(crate) fn check_report_json(report: &NsldCheckReport) -> String {
    let mut fields = vec![
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_linker_check"),
        json_string_field("manifest", &report.manifest),
        json_bool_field("valid", report.valid),
        json_usize_field("checks", report.checks),
        json_usize_field("failures", report.failures),
        json_usize_field("advisory_count", report.advisory_count),
        json_optional_string_field(
            "next_action_command_id",
            report.next_action_command_id.as_deref(),
        ),
        json_optional_string_field("next_action_command", report.next_action_command.as_deref()),
        json_optional_string_field(
            "next_action_command_resolved",
            report.next_action_command_resolved.as_deref(),
        ),
        json_optional_string_field(
            "next_action_command_reason",
            report.next_action_command_reason.as_deref(),
        ),
        json_optional_string_field("next_action_source", report.next_action_source.as_deref()),
        json_bool_field("next_action_available", report.next_action_available),
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
    ];
    fields.extend(check_report_object_fields(report));
    fields.extend(check_report_container_fields(report));
    fields.extend(check_report_final_fields(report));
    fields.extend(check_report_tail_fields(report));
    format!("{{{}}}", fields.join(","))
}
