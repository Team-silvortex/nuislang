use super::{display_text::*, reports::*};

pub(crate) fn print_nsld_final_executable_writer_plan_report(
    report: &NsldFinalExecutableWriterPlanReport,
) {
    println!("Nsld final executable writer plan");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  writer_kind: {}", report.writer_kind);
    println!("  writer_status: {}", report.writer_status);
    println!("  final_stage_plan_hash: {}", report.final_stage_plan_hash);
    println!("  final_stage_driver: {}", report.final_stage_driver);
    println!("  final_stage_link_mode: {}", report.final_stage_link_mode);
    println!("  host_wrapper_required: {}", report.host_wrapper_required);
    println!("  input_count: {}", report.input_count);
    for input in &report.inputs {
        println!(
            "  writer_input: order={} id={} kind={} required={} present={} hash={} path={}",
            input.order_index,
            input.input_id,
            input.input_kind,
            input.required,
            input.present,
            input.content_hash,
            input.path
        );
    }
    for step in &report.writer_steps {
        println!("  writer_step: {step}");
    }
    for blocker in &report.writer_blockers {
        println!("  writer_blocker: {blocker}");
    }
    for note in &report.notes {
        println!("  note: {note}");
    }
}

pub(crate) fn print_nsld_final_executable_writer_input_emit_report(
    report: &NsldFinalExecutableWriterInputEmitReport,
) {
    println!("Nsld final executable writer input emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  writer_input_hash: {}", report.writer_input_hash);
    println!("  writer_kind: {}", report.writer_kind);
    println!("  writer_status: {}", report.writer_status);
    println!("  final_stage_plan_hash: {}", report.final_stage_plan_hash);
    println!("  final_stage_driver: {}", report.final_stage_driver);
    println!("  final_stage_link_mode: {}", report.final_stage_link_mode);
    println!("  host_wrapper_required: {}", report.host_wrapper_required);
    println!("  command_arg_count: {}", report.command_arg_count);
    for blocker in &report.writer_blockers {
        println!("  writer_blocker: {blocker}");
    }
}

pub(crate) fn print_nsld_final_executable_writer_input_verify_report(
    report: &NsldFinalExecutableWriterInputVerifyReport,
) {
    println!("Nsld final executable writer input verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_writer_input_hash: {}",
        report.expected_writer_input_hash
    );
    println!(
        "  actual_writer_input_hash: {}",
        optional_string_text(report.actual_writer_input_hash.as_deref())
    );
    println!(
        "  expected_final_stage_plan_hash: {}",
        report.expected_final_stage_plan_hash
    );
    println!(
        "  actual_final_stage_plan_hash: {}",
        optional_string_text(report.actual_final_stage_plan_hash.as_deref())
    );
    println!("  expected_writer_kind: {}", report.expected_writer_kind);
    println!(
        "  actual_writer_kind: {}",
        optional_string_text(report.actual_writer_kind.as_deref())
    );
    println!(
        "  expected_writer_status: {}",
        report.expected_writer_status
    );
    println!(
        "  actual_writer_status: {}",
        optional_string_text(report.actual_writer_status.as_deref())
    );
    println!(
        "  expected_command_arg_count: {}",
        report.expected_command_arg_count
    );
    println!(
        "  actual_command_arg_count: {}",
        optional_usize_text(report.actual_command_arg_count)
    );
    for arg in &report.expected_command_args {
        println!("  expected_command_arg: {arg}");
    }
    for arg in &report.actual_command_args {
        println!("  actual_command_arg: {arg}");
    }
    for blocker in &report.expected_writer_blockers {
        println!("  expected_writer_blocker: {blocker}");
    }
    for blocker in &report.actual_writer_blockers {
        println!("  actual_writer_blocker: {blocker}");
    }
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_final_executable_host_dry_run_report(
    report: &NsldFinalExecutableHostDryRunReport,
) {
    println!("Nsld final executable host dry run");
    println!("  manifest: {}", report.manifest);
    println!("  writer_input_path: {}", report.writer_input_path);
    println!("  writer_input_valid: {}", report.writer_input_valid);
    println!(
        "  writer_input_hash: {}",
        optional_string_text(report.writer_input_hash.as_deref())
    );
    println!("  finalizer_contract: {}", report.finalizer_contract);
    println!(
        "  finalizer_registry_hash: {}",
        report.finalizer_registry_hash
    );
    println!(
        "  finalizer_registry_valid: {}",
        report.finalizer_registry_valid
    );
    println!("  finalizer_target_key: {}", report.finalizer_target_key);
    println!(
        "  finalizer_provider_id: {}",
        optional_string_text(report.finalizer_provider_id.as_deref())
    );
    println!(
        "  finalizer_provider_status: {}",
        optional_string_text(report.finalizer_provider_status.as_deref())
    );
    println!(
        "  finalizer_execution_kind: {}",
        optional_string_text(report.finalizer_execution_kind.as_deref())
    );
    print_finalizer_input_summary(report.finalizer_input_summary.as_ref());
    println!("  driver: {}", report.driver);
    println!("  driver_available: {}", report.driver_available);
    println!(
        "  driver_resolved_path: {}",
        optional_string_text(report.driver_resolved_path.as_deref())
    );
    println!("  command_arg_count: {}", report.command_arg_count);
    println!("  environment_ready: {}", report.environment_ready);
    println!("  invocation_policy: {}", report.invocation_policy);
    println!(
        "  invocation_policy_reason: {}",
        report.invocation_policy_reason
    );
    println!(
        "  can_invoke_host_finalizer: {}",
        report.can_invoke_host_finalizer
    );
    for arg in &report.command_args {
        println!("  command_arg: {arg}");
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
    for note in &report.notes {
        println!("  note: {note}");
    }
}

pub(crate) fn print_nsld_final_executable_host_invoke_plan_report(
    report: &NsldFinalExecutableHostInvokePlanReport,
) {
    println!("Nsld final executable host invoke plan");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  writer_input_path: {}", report.writer_input_path);
    println!("  finalizer_contract: {}", report.finalizer_contract);
    println!(
        "  finalizer_registry_hash: {}",
        report.finalizer_registry_hash
    );
    println!(
        "  finalizer_registry_valid: {}",
        report.finalizer_registry_valid
    );
    println!("  finalizer_target_key: {}", report.finalizer_target_key);
    println!(
        "  finalizer_provider_id: {}",
        optional_string_text(report.finalizer_provider_id.as_deref())
    );
    println!(
        "  finalizer_provider_status: {}",
        optional_string_text(report.finalizer_provider_status.as_deref())
    );
    println!(
        "  finalizer_execution_kind: {}",
        optional_string_text(report.finalizer_execution_kind.as_deref())
    );
    print_finalizer_input_summary(report.finalizer_input_summary.as_ref());
    println!("  invocation_kind: {}", report.invocation_kind);
    println!("  invocation_policy: {}", report.invocation_policy);
    println!(
        "  invocation_policy_reason: {}",
        report.invocation_policy_reason
    );
    println!(
        "  requires_explicit_allow: {}",
        report.requires_explicit_allow
    );
    println!(
        "  explicit_allow_present: {}",
        report.explicit_allow_present
    );
    println!("  environment_ready: {}", report.environment_ready);
    println!("  driver_available: {}", report.driver_available);
    println!(
        "  driver_resolved_path: {}",
        optional_string_text(report.driver_resolved_path.as_deref())
    );
    println!(
        "  can_invoke_host_finalizer: {}",
        report.can_invoke_host_finalizer
    );
    println!("  would_invoke: {}", report.would_invoke);
    println!("  command_arg_count: {}", report.command_arg_count);
    for arg in &report.command_args {
        println!("  command_arg: {arg}");
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
    for note in &report.notes {
        println!("  note: {note}");
    }
}

fn print_finalizer_input_summary(summary: Option<&NsldExecutableFinalizerInputSummary>) {
    let Some(summary) = summary else {
        println!("  finalizer_input_summary: none");
        return;
    };
    println!("  finalizer_input_contract: {}", summary.contract);
    println!("  finalizer_input_status: {}", summary.status);
    println!("  finalizer_input_object_count: {}", summary.object_count);
    println!("  finalizer_input_section_count: {}", summary.section_count);
    println!("  finalizer_input_symbol_count: {}", summary.symbol_count);
    println!(
        "  finalizer_input_relocation_count: {}",
        summary.relocation_count
    );
    println!(
        "  finalizer_input_defined_symbol_count: {}",
        summary.defined_symbol_count
    );
    println!(
        "  finalizer_input_undefined_symbol_count: {}",
        summary.undefined_symbol_count
    );
    println!(
        "  finalizer_input_internally_resolved_symbol_count: {}",
        summary.internally_resolved_symbol_count
    );
    println!(
        "  finalizer_input_unresolved_external_symbol_count: {}",
        summary.unresolved_external_symbol_count
    );
    for symbol in &summary.unresolved_external_symbols {
        println!("  finalizer_input_unresolved_external_symbol: {symbol}");
    }
    let placement = &summary.placement_binding;
    println!(
        "  finalizer_input_placement_contract: {}",
        placement.contract
    );
    println!("  finalizer_input_placement_status: {}", placement.status);
    println!(
        "  finalizer_input_placement_plan_hash: {}",
        placement.plan_hash
    );
    println!(
        "  finalizer_input_placement_image_span_bytes: {}",
        placement.image_span_bytes
    );
    for section in &placement.merged_sections {
        println!(
            "  finalizer_input_merged_section: id={} segment={} section={} flags=0x{:08x} align={} offset={} bytes={} contributions={} zero_fill={}",
            section.section_id,
            section.segment_name,
            section.section_name,
            section.flags,
            section.alignment,
            section.output_offset,
            section.size_bytes,
            section.contribution_count,
            section.zero_fill
        );
    }
    for section in &placement.section_placements {
        println!(
            "  finalizer_input_section_placement: object={} role={} ordinal={} input={}:{} output={} offset={} section_offset={} bytes={} align={} zero_fill={}",
            section.object_id,
            section.object_role,
            section.input_section_ordinal,
            section.input_segment_name,
            section.input_section_name,
            section.output_section_id,
            section.output_offset,
            section.output_section_offset,
            section.size_bytes,
            section.alignment,
            section.zero_fill
        );
    }
    for allocation in &placement.common_allocations {
        println!(
            "  finalizer_input_common_allocation: id={} symbol={} owner={}:{}:{} declarations={} bytes={} align={} section={} offset={} section_offset={}",
            allocation.allocation_id,
            allocation.symbol,
            allocation.owner_object_id,
            allocation.owner_object_role,
            allocation.owner_symbol_index,
            allocation.declaration_count,
            allocation.size_bytes,
            allocation.alignment,
            allocation.output_section_id,
            allocation.output_offset,
            allocation.output_section_offset
        );
    }
    for binding in &placement.symbol_bindings {
        println!(
            "  finalizer_input_symbol_binding: symbol={} reference={}:{} status={} target_object={} target_symbol={} target_kind={} target_section={} target_offset={}",
            binding.symbol,
            binding.reference_object_id,
            binding.reference_symbol_index,
            binding.status,
            binding.target_object_id.as_deref().unwrap_or("none"),
            binding
                .target_symbol_index
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned()),
            binding.target_kind.as_deref().unwrap_or("none"),
            binding.target_section_id.as_deref().unwrap_or("none"),
            binding
                .target_output_offset
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned())
        );
    }
    let relocation = &summary.relocation_application;
    println!(
        "  finalizer_input_relocation_application_contract: {}",
        relocation.contract
    );
    println!(
        "  finalizer_input_relocation_application_status: {}",
        relocation.status
    );
    println!(
        "  finalizer_input_relocation_application_plan_hash: {}",
        relocation.plan_hash
    );
    println!(
        "  finalizer_input_relocation_application_placement_plan_hash: {}",
        relocation.placement_plan_hash
    );
    println!(
        "  finalizer_input_relocation_application_count: {}",
        relocation.relocation_count
    );
    println!(
        "  finalizer_input_relocation_application_registered_kind_count: {}",
        relocation.registered_kind_count
    );
    println!(
        "  finalizer_input_relocation_application_ready_count: {}",
        relocation.ready_application_count
    );
    println!(
        "  finalizer_input_relocation_application_platform_structure_count: {}",
        relocation.platform_structure_count
    );
    println!(
        "  finalizer_input_relocation_application_external_compatibility_count: {}",
        relocation.external_compatibility_count
    );
    println!(
        "  finalizer_input_relocation_application_metadata_record_count: {}",
        relocation.metadata_record_count
    );
    for item in &relocation.applications {
        println!(
            "  finalizer_input_relocation_application: id={} object={} role={} section_ordinal={} source_section={} source_offset={} output_offset={} width={} pcrel={} external={} type={} kind={} action={} target={} target_index={} target_object={} target_section={} target_offset={} addend={} pair={} resolver={} status={}",
            item.relocation_id,
            item.object_id,
            item.object_role,
            item.input_section_ordinal,
            item.source_section_id,
            item.source_offset,
            item.source_output_offset,
            item.width_bytes,
            item.pc_relative,
            item.external,
            item.relocation_type,
            item.relocation_kind,
            item.action_kind,
            item.target_symbol.as_deref().unwrap_or("none"),
            item.target_symbol_index
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned()),
            item.target_object_id.as_deref().unwrap_or("none"),
            item.target_section_id.as_deref().unwrap_or("none"),
            item.target_output_offset
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned()),
            item.explicit_addend
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned()),
            item.pair_relocation_id.as_deref().unwrap_or("none"),
            item.resolver_status,
            item.application_status
        );
    }
    let materialization = &summary.materialization_preview;
    println!(
        "  finalizer_input_materialization: contract={} status={} image_span={} copied={} zero_fill={} image_hash={} patch_plan_hash={} planned={} previewed={} deferred={} metadata={}",
        materialization.contract,
        materialization.status,
        materialization.image_span_bytes,
        materialization.copied_bytes,
        materialization.zero_fill_bytes,
        materialization.image_hash,
        materialization.patch_plan_hash,
        materialization.planned_direct_count,
        materialization.previewed_patch_count,
        materialization.deferred_patch_count,
        materialization.metadata_record_count
    );
    for section in &materialization.section_audits {
        println!(
            "  finalizer_input_materialization_section: id={} offset={} bytes={} copied={} zero_fill={} hash={}",
            section.section_id,
            section.output_offset,
            section.size_bytes,
            section.copied_bytes,
            section.zero_fill_bytes,
            section.content_hash
        );
    }
    for patch in &materialization.patches {
        println!(
            "  finalizer_input_materialization_patch: id={} kind={} offset={} width={} target={} addend={} source={} encoded={} source_hash={} encoded_hash={} audit_hash={}",
            patch.relocation_id,
            patch.relocation_kind,
            patch.source_output_offset,
            patch.width_bytes,
            patch.target_output_offset,
            patch.effective_addend,
            patch.source_bytes_hex,
            patch.encoded_bytes_hex,
            patch.source_bytes_hash,
            patch.encoded_bytes_hash,
            patch.audit_hash
        );
    }
    let application = &summary.patch_application;
    println!(
        "  finalizer_input_patch_application: contract={} status={} placement_plan_hash={} relocation_plan_hash={} image_span={} original_image_hash={} applied_image_hash={} patch_plan_hash={} ledger_hash={} expected={} applied={} deferred={} write_once_spans={}",
        application.contract,
        application.status,
        application.placement_plan_hash,
        application.relocation_plan_hash,
        application.image_span_bytes,
        application.original_image_hash,
        application.applied_image_hash,
        application.patch_plan_hash,
        application.application_ledger_hash,
        application.expected_patch_count,
        application.applied_patch_count,
        application.deferred_patch_count,
        application.write_once_span_count
    );
    for patch in &application.patches {
        println!(
            "  finalizer_input_applied_patch: id={} offset={} width={} source_hash={} encoded_hash={} post_write_hash={} preview_audit_hash={} write_audit_hash={}",
            patch.relocation_id,
            patch.source_output_offset,
            patch.width_bytes,
            patch.source_bytes_hash,
            patch.encoded_bytes_hash,
            patch.post_write_bytes_hash,
            patch.preview_audit_hash,
            patch.write_audit_hash
        );
    }
    let platform = &summary.platform_structure_plan;
    println!(
        "  finalizer_input_platform_structure: contract={} status={} placement_plan_hash={} relocation_plan_hash={} application_ledger_hash={} applied_image_hash={} plan_hash={} base_span={} planned_span={} rules={} deferred={} targets={} stubs={}:{}+{} got={}:{}+{}",
        platform.contract,
        platform.status,
        platform.placement_plan_hash,
        platform.relocation_plan_hash,
        platform.patch_application_ledger_hash,
        platform.applied_image_hash,
        platform.plan_hash,
        platform.base_image_span_bytes,
        platform.planned_image_span_bytes,
        platform.registered_rule_count,
        platform.deferred_relocation_count,
        platform.target_count,
        platform.stub_entry_count,
        platform.stub_region_offset,
        platform.stub_region_bytes,
        platform.got_entry_count,
        platform.got_region_offset,
        platform.got_region_bytes
    );
    for target in &platform.targets {
        println!(
            "  finalizer_input_platform_target: id={} key={} symbol={} resolver={} target_object={} target_section={} target_offset={} got_slot={} got_offset={} stub_slot={} stub_offset={} relocations={} audit_hash={}",
            target.structure_id,
            target.target_key,
            target.target_symbol,
            target.resolver_status,
            target.target_object_id.as_deref().unwrap_or("none"),
            target.target_section_id.as_deref().unwrap_or("none"),
            display_option(target.target_output_offset),
            display_option(target.got_slot_index),
            display_option(target.got_output_offset),
            display_option(target.stub_slot_index),
            display_option(target.stub_output_offset),
            target.relocation_ids.join(","),
            target.audit_hash
        );
    }
    for binding in &platform.relocation_bindings {
        println!(
            "  finalizer_input_platform_binding: relocation={} kind={} action={} source_offset={} width={} target_id={} patch_target={} patch_offset={} audit_hash={}",
            binding.relocation_id,
            binding.relocation_kind,
            binding.action_kind,
            binding.source_output_offset,
            binding.width_bytes,
            binding.structure_id,
            binding.patch_target_kind,
            binding.patch_target_output_offset,
            binding.audit_hash
        );
    }
    let platform_application = &summary.platform_patch_application;
    println!(
        "  finalizer_input_platform_patch_application: contract={} status={} placement_plan_hash={} relocation_plan_hash={} direct_ledger_hash={} platform_plan_hash={} base_image_hash={} platform_image_hash={} ledger_hash={} base_span={} platform_span={} expected={} applied={} stubs={} got={} unresolved_binds={} write_once_spans={}",
        platform_application.contract,
        platform_application.status,
        platform_application.placement_plan_hash,
        platform_application.relocation_plan_hash,
        platform_application.direct_patch_application_ledger_hash,
        platform_application.platform_structure_plan_hash,
        platform_application.base_applied_image_hash,
        platform_application.platform_image_hash,
        platform_application.application_ledger_hash,
        platform_application.base_image_span_bytes,
        platform_application.platform_image_span_bytes,
        platform_application.expected_deferred_patch_count,
        platform_application.applied_deferred_patch_count,
        platform_application.stub_write_count,
        platform_application.got_write_count,
        platform_application.unresolved_bind_count,
        platform_application.write_once_span_count
    );
    for write in &platform_application.structure_writes {
        println!(
            "  finalizer_input_platform_write: id={} target_id={} kind={} symbol={} offset={} width={} encoded={} encoded_hash={} audit_hash={}",
            write.write_id,
            write.structure_id,
            write.write_kind,
            write.target_symbol,
            write.output_offset,
            write.width_bytes,
            write.encoded_bytes_hex,
            write.encoded_bytes_hash,
            write.write_audit_hash
        );
    }
    for patch in &platform_application.patches {
        println!(
            "  finalizer_input_platform_patch: relocation={} kind={} source_offset={} width={} target_offset={} addend={} source={} encoded={} source_hash={} encoded_hash={} binding_audit_hash={} write_audit_hash={}",
            patch.relocation_id,
            patch.relocation_kind,
            patch.source_output_offset,
            patch.width_bytes,
            patch.patch_target_output_offset,
            patch.effective_addend,
            patch.source_bytes_hex,
            patch.encoded_bytes_hex,
            patch.source_bytes_hash,
            patch.encoded_bytes_hash,
            patch.binding_audit_hash,
            patch.write_audit_hash
        );
    }
    for bind in &platform_application.bind_records {
        println!(
            "  finalizer_input_platform_bind: id={} target_id={} key={} symbol={} got_offset={} width={} placeholder_hash={} status={} audit_hash={}",
            bind.bind_id,
            bind.structure_id,
            bind.target_key,
            bind.target_symbol,
            bind.got_output_offset,
            bind.width_bytes,
            bind.placeholder_bytes_hash,
            bind.status,
            bind.audit_hash
        );
    }
    crate::display_final_macho_shell::print_macho_shell_layout_plan(&summary.shell_layout_plan);
    crate::display_final_macho_shell::print_macho_shell_image_serialization(
        &summary.shell_image_serialization,
    );
}

fn display_option(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_owned())
}

pub(crate) fn print_nsld_final_executable_host_invoke_plan_emit_report(
    report: &NsldFinalExecutableHostInvokePlanEmitReport,
) {
    println!("Nsld final executable host invoke plan emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  invoke_plan_hash: {}", report.invoke_plan_hash);
    println!("  invocation_policy: {}", report.invocation_policy);
    println!(
        "  requires_explicit_allow: {}",
        report.requires_explicit_allow
    );
    println!(
        "  explicit_allow_present: {}",
        report.explicit_allow_present
    );
    println!("  would_invoke: {}", report.would_invoke);
    println!("  blocker_count: {}", report.blocker_count);
}

pub(crate) fn print_nsld_final_executable_host_invoke_plan_verify_report(
    report: &NsldFinalExecutableHostInvokePlanVerifyReport,
) {
    println!("Nsld final executable host invoke plan verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_invoke_plan_hash: {}",
        report.expected_invoke_plan_hash
    );
    println!(
        "  actual_invoke_plan_hash: {}",
        optional_string_text(report.actual_invoke_plan_hash.as_deref())
    );
    println!(
        "  expected_invocation_policy: {}",
        report.expected_invocation_policy
    );
    println!(
        "  actual_invocation_policy: {}",
        optional_string_text(report.actual_invocation_policy.as_deref())
    );
    println!(
        "  expected_requires_explicit_allow: {}",
        report.expected_requires_explicit_allow
    );
    println!(
        "  actual_requires_explicit_allow: {}",
        optional_bool_text(report.actual_requires_explicit_allow)
    );
    println!(
        "  expected_explicit_allow_present: {}",
        report.expected_explicit_allow_present
    );
    println!(
        "  actual_explicit_allow_present: {}",
        optional_bool_text(report.actual_explicit_allow_present)
    );
    println!("  expected_would_invoke: {}", report.expected_would_invoke);
    println!(
        "  actual_would_invoke: {}",
        optional_bool_text(report.actual_would_invoke)
    );
    println!(
        "  expected_command_arg_count: {}",
        report.expected_command_arg_count
    );
    println!(
        "  actual_command_arg_count: {}",
        optional_usize_text(report.actual_command_arg_count)
    );
    for arg in &report.expected_command_args {
        println!("  expected_command_arg: {arg}");
    }
    for arg in &report.actual_command_args {
        println!("  actual_command_arg: {arg}");
    }
    println!(
        "  expected_blocker_count: {}",
        report.expected_blocker_count
    );
    println!(
        "  actual_blocker_count: {}",
        optional_usize_text(report.actual_blocker_count)
    );
    for blocker in &report.expected_blockers {
        println!("  expected_blocker: {blocker}");
    }
    for blocker in &report.actual_blockers {
        println!("  actual_blocker: {blocker}");
    }
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}
