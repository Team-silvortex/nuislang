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
        macho_relocation_application_json(&summary.relocation_application),
        macho_materialization_preview_json(&summary.materialization_preview),
        macho_patch_application_json(&summary.patch_application),
        macho_platform_structure_plan_json(&summary.platform_structure_plan),
    ];
    format!("\"finalizer_input_summary\":{{{}}}", fields.join(","))
}

fn macho_platform_structure_plan_json(
    report: &NsldMachOArm64PlatformStructurePlanReport,
) -> String {
    let targets = report
        .targets
        .iter()
        .map(|target| {
            let fields = [
                json_string_field("structure_id", &target.structure_id),
                json_string_field("target_key", &target.target_key),
                json_string_field("target_symbol", &target.target_symbol),
                json_string_field("resolver_status", &target.resolver_status),
                json_optional_string_field("target_object_id", target.target_object_id.as_deref()),
                json_optional_string_field(
                    "target_section_id",
                    target.target_section_id.as_deref(),
                ),
                json_optional_usize_field("target_output_offset", target.target_output_offset),
                json_optional_usize_field("got_slot_index", target.got_slot_index),
                json_optional_usize_field("got_output_offset", target.got_output_offset),
                json_optional_usize_field("stub_slot_index", target.stub_slot_index),
                json_optional_usize_field("stub_output_offset", target.stub_output_offset),
                json_string_array_field("relocation_ids", &target.relocation_ids),
                json_string_field("audit_hash", &target.audit_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    let bindings = report
        .relocation_bindings
        .iter()
        .map(|binding| {
            let fields = [
                json_string_field("relocation_id", &binding.relocation_id),
                json_string_field("relocation_kind", &binding.relocation_kind),
                json_string_field("action_kind", &binding.action_kind),
                json_usize_field("source_output_offset", binding.source_output_offset),
                json_usize_field("width_bytes", binding.width_bytes),
                json_string_field("structure_id", &binding.structure_id),
                json_string_field("patch_target_kind", &binding.patch_target_kind),
                json_usize_field(
                    "patch_target_output_offset",
                    binding.patch_target_output_offset,
                ),
                json_string_field("audit_hash", &binding.audit_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    let fields = [
        json_string_field("contract", &report.contract),
        json_string_field("status", &report.status),
        json_string_field("placement_plan_hash", &report.placement_plan_hash),
        json_string_field("relocation_plan_hash", &report.relocation_plan_hash),
        json_string_field(
            "patch_application_ledger_hash",
            &report.patch_application_ledger_hash,
        ),
        json_string_field("applied_image_hash", &report.applied_image_hash),
        json_usize_field("base_image_span_bytes", report.base_image_span_bytes),
        json_usize_field("planned_image_span_bytes", report.planned_image_span_bytes),
        json_usize_field("registered_rule_count", report.registered_rule_count),
        json_usize_field(
            "deferred_relocation_count",
            report.deferred_relocation_count,
        ),
        json_usize_field("target_count", report.target_count),
        json_usize_field("stub_region_offset", report.stub_region_offset),
        json_usize_field("stub_region_bytes", report.stub_region_bytes),
        json_usize_field("stub_entry_size", report.stub_entry_size),
        json_usize_field("stub_alignment", report.stub_alignment),
        json_usize_field("stub_entry_count", report.stub_entry_count),
        json_usize_field("got_region_offset", report.got_region_offset),
        json_usize_field("got_region_bytes", report.got_region_bytes),
        json_usize_field("got_entry_size", report.got_entry_size),
        json_usize_field("got_alignment", report.got_alignment),
        json_usize_field("got_entry_count", report.got_entry_count),
        json_string_field("plan_hash", &report.plan_hash),
        format!("\"targets\":[{targets}]"),
        format!("\"relocation_bindings\":[{bindings}]"),
    ];
    format!("\"platform_structure_plan\":{{{}}}", fields.join(","))
}

fn macho_patch_application_json(report: &NsldMachOArm64PatchApplicationReport) -> String {
    let patches = report
        .patches
        .iter()
        .map(|patch| {
            let fields = [
                json_string_field("relocation_id", &patch.relocation_id),
                json_usize_field("source_output_offset", patch.source_output_offset),
                json_usize_field("width_bytes", patch.width_bytes),
                json_string_field("source_bytes_hash", &patch.source_bytes_hash),
                json_string_field("encoded_bytes_hash", &patch.encoded_bytes_hash),
                json_string_field("post_write_bytes_hash", &patch.post_write_bytes_hash),
                json_string_field("preview_audit_hash", &patch.preview_audit_hash),
                json_string_field("write_audit_hash", &patch.write_audit_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    let fields = [
        json_string_field("contract", &report.contract),
        json_string_field("status", &report.status),
        json_string_field("placement_plan_hash", &report.placement_plan_hash),
        json_string_field("relocation_plan_hash", &report.relocation_plan_hash),
        json_string_field("patch_plan_hash", &report.patch_plan_hash),
        json_string_field("original_image_hash", &report.original_image_hash),
        json_string_field("applied_image_hash", &report.applied_image_hash),
        json_usize_field("image_span_bytes", report.image_span_bytes),
        json_usize_field("expected_patch_count", report.expected_patch_count),
        json_usize_field("applied_patch_count", report.applied_patch_count),
        json_usize_field("deferred_patch_count", report.deferred_patch_count),
        json_usize_field("write_once_span_count", report.write_once_span_count),
        json_string_field("application_ledger_hash", &report.application_ledger_hash),
        format!("\"patches\":[{patches}]"),
    ];
    format!("\"patch_application\":{{{}}}", fields.join(","))
}

fn macho_materialization_preview_json(
    report: &NsldMachOArm64MaterializationPreviewReport,
) -> String {
    let sections = report
        .section_audits
        .iter()
        .map(|section| {
            let fields = [
                json_string_field("section_id", &section.section_id),
                json_usize_field("output_offset", section.output_offset),
                json_usize_field("size_bytes", section.size_bytes),
                json_usize_field("copied_bytes", section.copied_bytes),
                json_usize_field("zero_fill_bytes", section.zero_fill_bytes),
                json_string_field("content_hash", &section.content_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    let patches = report
        .patches
        .iter()
        .map(|patch| {
            let fields = [
                json_string_field("relocation_id", &patch.relocation_id),
                json_string_field("relocation_kind", &patch.relocation_kind),
                json_usize_field("source_output_offset", patch.source_output_offset),
                json_usize_field("width_bytes", patch.width_bytes),
                json_usize_field("target_output_offset", patch.target_output_offset),
                json_i64_field("effective_addend", patch.effective_addend),
                json_string_field("source_bytes_hex", &patch.source_bytes_hex),
                json_string_field("encoded_bytes_hex", &patch.encoded_bytes_hex),
                json_string_field("source_bytes_hash", &patch.source_bytes_hash),
                json_string_field("encoded_bytes_hash", &patch.encoded_bytes_hash),
                json_string_field("audit_hash", &patch.audit_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    let fields = [
        json_string_field("contract", &report.contract),
        json_string_field("status", &report.status),
        json_string_field("placement_plan_hash", &report.placement_plan_hash),
        json_string_field("relocation_plan_hash", &report.relocation_plan_hash),
        json_usize_field("image_span_bytes", report.image_span_bytes),
        json_usize_field("copied_bytes", report.copied_bytes),
        json_usize_field("zero_fill_bytes", report.zero_fill_bytes),
        json_string_field("image_hash", &report.image_hash),
        json_usize_field("planned_direct_count", report.planned_direct_count),
        json_usize_field("previewed_patch_count", report.previewed_patch_count),
        json_usize_field("deferred_patch_count", report.deferred_patch_count),
        json_usize_field("metadata_record_count", report.metadata_record_count),
        json_string_field("patch_plan_hash", &report.patch_plan_hash),
        format!("\"section_audits\":[{sections}]"),
        format!("\"patches\":[{patches}]"),
    ];
    format!("\"materialization_preview\":{{{}}}", fields.join(","))
}

fn macho_relocation_application_json(report: &NsldMachOArm64RelocationApplicationReport) -> String {
    let applications = report
        .applications
        .iter()
        .map(|item| {
            let fields = [
                json_string_field("relocation_id", &item.relocation_id),
                json_string_field("object_id", &item.object_id),
                json_string_field("object_role", &item.object_role),
                json_usize_field("input_section_ordinal", item.input_section_ordinal),
                json_string_field("source_section_id", &item.source_section_id),
                json_usize_field("source_offset", item.source_offset),
                json_usize_field("source_output_offset", item.source_output_offset),
                json_usize_field("width_bytes", item.width_bytes),
                json_bool_field("pc_relative", item.pc_relative),
                json_bool_field("external", item.external),
                json_usize_field("relocation_type", item.relocation_type as usize),
                json_string_field("relocation_kind", &item.relocation_kind),
                json_string_field("action_kind", &item.action_kind),
                json_optional_string_field("target_symbol", item.target_symbol.as_deref()),
                json_optional_usize_field("target_symbol_index", item.target_symbol_index),
                json_optional_string_field("target_object_id", item.target_object_id.as_deref()),
                json_optional_string_field("target_section_id", item.target_section_id.as_deref()),
                json_optional_usize_field("target_output_offset", item.target_output_offset),
                json_optional_i64_field("explicit_addend", item.explicit_addend),
                json_optional_string_field(
                    "pair_relocation_id",
                    item.pair_relocation_id.as_deref(),
                ),
                json_string_field("resolver_status", &item.resolver_status),
                json_string_field("application_status", &item.application_status),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    let fields = [
        json_string_field("contract", &report.contract),
        json_string_field("status", &report.status),
        json_string_field("plan_hash", &report.plan_hash),
        json_string_field("placement_plan_hash", &report.placement_plan_hash),
        json_usize_field("relocation_count", report.relocation_count),
        json_usize_field("registered_kind_count", report.registered_kind_count),
        json_usize_field("ready_application_count", report.ready_application_count),
        json_usize_field("platform_structure_count", report.platform_structure_count),
        json_usize_field(
            "external_compatibility_count",
            report.external_compatibility_count,
        ),
        json_usize_field("metadata_record_count", report.metadata_record_count),
        format!("\"applications\":[{applications}]"),
    ];
    format!("\"relocation_application\":{{{}}}", fields.join(","))
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
