use super::{json_fields::*, json_final_fragments::final_stage_inputs_json, reports::*};

pub(crate) fn nsld_final_executable_writer_plan_report_json(
    report: &NsldFinalExecutableWriterPlanReport,
) -> String {
    let fields = [
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
    let fields = [
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
    let fields = [
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
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_host_dry_run"),
        json_string_field("manifest", &report.manifest),
        json_string_field("writer_input_path", &report.writer_input_path),
        json_bool_field("writer_input_valid", report.writer_input_valid),
        json_optional_string_field("writer_input_hash", report.writer_input_hash.as_deref()),
        json_string_field("finalizer_contract", &report.finalizer_contract),
        json_string_field("finalizer_registry_hash", &report.finalizer_registry_hash),
        json_bool_field("finalizer_registry_valid", report.finalizer_registry_valid),
        json_string_field("finalizer_target_key", &report.finalizer_target_key),
        json_optional_string_field(
            "finalizer_provider_id",
            report.finalizer_provider_id.as_deref(),
        ),
        json_optional_string_field(
            "finalizer_provider_status",
            report.finalizer_provider_status.as_deref(),
        ),
        json_optional_string_field(
            "finalizer_execution_kind",
            report.finalizer_execution_kind.as_deref(),
        ),
        finalizer_input_summary_json(report.finalizer_input_summary.as_ref()),
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
    let fields = [
        json_string_field("tool", "nsld"),
        json_string_field("kind", "nsld_final_executable_host_invoke_plan"),
        json_string_field("manifest", &report.manifest),
        json_string_field("output_path", &report.output_path),
        json_string_field("writer_input_path", &report.writer_input_path),
        json_string_field("finalizer_contract", &report.finalizer_contract),
        json_string_field("finalizer_registry_hash", &report.finalizer_registry_hash),
        json_bool_field("finalizer_registry_valid", report.finalizer_registry_valid),
        json_string_field("finalizer_target_key", &report.finalizer_target_key),
        json_optional_string_field(
            "finalizer_provider_id",
            report.finalizer_provider_id.as_deref(),
        ),
        json_optional_string_field(
            "finalizer_provider_status",
            report.finalizer_provider_status.as_deref(),
        ),
        json_optional_string_field(
            "finalizer_execution_kind",
            report.finalizer_execution_kind.as_deref(),
        ),
        finalizer_input_summary_json(report.finalizer_input_summary.as_ref()),
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

fn finalizer_input_summary_json(summary: Option<&NsldExecutableFinalizerInputSummary>) -> String {
    let Some(summary) = summary else {
        return "\"finalizer_input_summary\":null".to_owned();
    };
    let fields = [
        json_string_field("contract", &summary.contract),
        json_string_field("status", &summary.status),
        json_usize_field("object_count", summary.object_count),
        json_usize_field("section_count", summary.section_count),
        json_usize_field("symbol_count", summary.symbol_count),
        json_usize_field("relocation_count", summary.relocation_count),
        json_usize_field("defined_symbol_count", summary.defined_symbol_count),
        json_usize_field("undefined_symbol_count", summary.undefined_symbol_count),
        json_usize_field(
            "internally_resolved_symbol_count",
            summary.internally_resolved_symbol_count,
        ),
        json_usize_field(
            "unresolved_external_symbol_count",
            summary.unresolved_external_symbol_count,
        ),
        json_string_array_field(
            "unresolved_external_symbols",
            &summary.unresolved_external_symbols,
        ),
        macho_placement_binding_json(&summary.placement_binding),
    ];
    format!("\"finalizer_input_summary\":{{{}}}", fields.join(","))
}

fn macho_placement_binding_json(report: &NsldMachOPlacementBindingReport) -> String {
    let merged_sections = report
        .merged_sections
        .iter()
        .map(|section| {
            let fields = [
                json_string_field("section_id", &section.section_id),
                json_string_field("segment_name", &section.segment_name),
                json_string_field("section_name", &section.section_name),
                json_string_field("flags", &format!("0x{:08x}", section.flags)),
                json_usize_field("alignment", section.alignment),
                json_usize_field("output_offset", section.output_offset),
                json_usize_field("size_bytes", section.size_bytes),
                json_usize_field("contribution_count", section.contribution_count),
                json_bool_field("zero_fill", section.zero_fill),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    let placements = report
        .section_placements
        .iter()
        .map(|placement| {
            let fields = [
                json_string_field("object_id", &placement.object_id),
                json_string_field("object_role", &placement.object_role),
                json_usize_field("input_section_ordinal", placement.input_section_ordinal),
                json_string_field("input_segment_name", &placement.input_segment_name),
                json_string_field("input_section_name", &placement.input_section_name),
                json_string_field("output_section_id", &placement.output_section_id),
                json_usize_field("output_offset", placement.output_offset),
                json_usize_field("output_section_offset", placement.output_section_offset),
                json_usize_field("size_bytes", placement.size_bytes),
                json_usize_field("alignment", placement.alignment),
                json_bool_field("zero_fill", placement.zero_fill),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    let bindings = report
        .symbol_bindings
        .iter()
        .map(|binding| {
            let fields = [
                json_string_field("symbol", &binding.symbol),
                json_string_field("reference_object_id", &binding.reference_object_id),
                json_usize_field("reference_symbol_index", binding.reference_symbol_index),
                json_string_field("status", &binding.status),
                json_optional_string_field("target_object_id", binding.target_object_id.as_deref()),
                json_optional_usize_field("target_symbol_index", binding.target_symbol_index),
                json_optional_string_field("target_kind", binding.target_kind.as_deref()),
                json_optional_string_field(
                    "target_section_id",
                    binding.target_section_id.as_deref(),
                ),
                json_optional_usize_field("target_output_offset", binding.target_output_offset),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    let fields = [
        json_string_field("contract", &report.contract),
        json_string_field("status", &report.status),
        json_string_field("plan_hash", &report.plan_hash),
        json_usize_field("image_span_bytes", report.image_span_bytes),
        json_usize_field("merged_section_count", report.merged_sections.len()),
        json_usize_field("section_placement_count", report.section_placements.len()),
        json_usize_field("symbol_binding_count", report.symbol_bindings.len()),
        json_usize_field(
            "internally_bound_symbol_count",
            report.internally_bound_symbol_count,
        ),
        json_usize_field(
            "external_compatibility_symbol_count",
            report.external_compatibility_symbol_count,
        ),
        format!("\"merged_sections\":[{merged_sections}]"),
        format!("\"section_placements\":[{placements}]"),
        format!("\"symbol_bindings\":[{bindings}]"),
    ];
    format!("\"placement_binding\":{{{}}}", fields.join(","))
}

pub(crate) fn nsld_final_executable_host_invoke_plan_emit_report_json(
    report: &NsldFinalExecutableHostInvokePlanEmitReport,
) -> String {
    let fields = [
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
    let fields = [
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
