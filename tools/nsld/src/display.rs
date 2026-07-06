pub(crate) use super::display_check::print_check_report;
pub(crate) use super::display_container::*;
pub(crate) use super::display_link_tables::*;
pub(crate) use super::display_object::*;
pub(crate) use super::display_object_emit::*;
pub(crate) use super::display_object_image::*;

use super::reports::*;

pub(crate) fn print_nsld_artifact_chain_report(report: &NsldArtifactChainReport) {
    println!("Nsld artifact chain");
    println!("  manifest: {}", report.manifest);
    println!("  output_dir: {}", report.output_dir);
    println!("  valid: {}", report.valid);
    println!("  stage_count: {}", report.stage_count);
    println!("  present_count: {}", report.present_count);
    println!("  required_count: {}", report.required_count);
    println!(
        "  missing_required_count: {}",
        report.missing_required_count
    );
    println!(
        "  optional_present_count: {}",
        report.optional_present_count
    );
    println!(
        "  first_missing_required_stage: {}",
        optional_string_text(report.first_missing_required_stage.as_deref())
    );
    println!(
        "  next_required_stage: {}",
        optional_string_text(report.next_required_stage.as_deref())
    );
    println!(
        "  suggested_command_id: {}",
        optional_string_text(report.suggested_command_id.as_deref())
    );
    println!(
        "  suggested_command: {}",
        optional_string_text(report.suggested_command.as_deref())
    );
    println!(
        "  suggested_command_resolved: {}",
        optional_string_text(report.suggested_command_resolved.as_deref())
    );
    println!(
        "  suggested_command_reason: {}",
        optional_string_text(report.suggested_command_reason.as_deref())
    );
    println!(
        "  next_optional_stage: {}",
        optional_string_text(report.next_optional_stage.as_deref())
    );
    println!(
        "  next_optional_command_id: {}",
        optional_string_text(report.next_optional_command_id.as_deref())
    );
    println!(
        "  next_optional_command: {}",
        optional_string_text(report.next_optional_command.as_deref())
    );
    println!(
        "  next_optional_command_resolved: {}",
        optional_string_text(report.next_optional_command_resolved.as_deref())
    );
    println!(
        "  next_optional_command_reason: {}",
        optional_string_text(report.next_optional_command_reason.as_deref())
    );
    for stage in &report.stages {
        println!(
            "  stage: order={} id={} required={} present={} file={} path={}",
            stage.order_index,
            stage.stage_id,
            stage.required,
            stage.present,
            stage.file_name,
            stage.path
        );
    }
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_closure_report(report: &NsldClosureReport) {
    println!("Nsld linker closure");
    println!("  manifest: {}", report.manifest);
    println!("  closed: {}", report.closed);
    println!("  final_stage_link_mode: {}", report.final_stage_link_mode);
    println!("  host_wrapper_required: {}", report.host_wrapper_required);
    println!("  domain_count: {}", report.domain_count);
    println!("  hetero_domain_count: {}", report.hetero_domain_count);
    println!(
        "  sidecar_capability_count: {}",
        report.sidecar_capability_count
    );
    println!("  clock_edge_count: {}", report.clock_edge_count);
    println!("  data_segment_count: {}", report.data_segment_count);
    println!("  internal_contracts: {}", report.internal_contracts.len());
    for contract in &report.internal_contracts {
        println!("  internal_contract: {contract}");
    }
    println!("  linker_contract_hash: {}", report.linker_contract_hash);
    println!("  link_inputs: {}", report.link_inputs.len());
    println!("  link_input_count: {}", report.link_input_count);
    println!(
        "  link_input_total_bytes: {}",
        report.link_input_total_bytes
    );
    println!("  link_input_table_hash: {}", report.link_input_table_hash);
    println!(
        "  link_input_table: present={} valid={}",
        report.link_input_table_present,
        optional_bool_text(report.link_input_table_valid)
    );
    println!(
        "  prepared_artifact_chain: valid={} issues={}",
        report.prepared_artifact_chain_valid,
        report.prepared_artifact_chain_issues.len()
    );
    for issue in &report.prepared_artifact_chain_issues {
        println!("  prepared_artifact_chain_issue: {issue}");
    }
    println!(
        "  container_metadata_table_hash: {}",
        report.container_metadata_table_hash
    );
    println!("  container_layout_hash: {}", report.container_layout_hash);
    println!("  container_hash: {}", report.container_hash);
    println!("  payload_size_bytes: {}", report.payload_size_bytes);
    println!("  payload_hash: {}", report.payload_hash);
    println!(
        "  container_loader_readiness: {}",
        report.container_loader_readiness
    );
    println!(
        "  compatibility_domain: count={} table_hash={} id={} kind={} paradigm={} hook={} abi={} wrapper={} required={}",
        report.compatibility_domain_count,
        report.compatibility_domain_table_hash,
        optional_string_text(report.compatibility_domain_id.as_deref()),
        optional_string_text(report.compatibility_domain_kind.as_deref()),
        optional_string_text(report.compatibility_domain_paradigm.as_deref()),
        optional_string_text(report.compatibility_domain_lifecycle_hook.as_deref()),
        optional_string_text(report.compatibility_domain_abi_family.as_deref()),
        optional_string_text(report.compatibility_domain_wrapper_policy.as_deref()),
        optional_bool_text(report.compatibility_domain_required)
    );
    println!(
        "  object_image_relocation_lowering: valid={} rule_count={}",
        optional_bool_text(report.object_image_relocation_lowering_valid),
        optional_usize_text(report.object_image_relocation_lowering_rule_count)
    );
    for rule in &report.object_image_relocation_lowering_rules {
        println!(
            "  object_image_relocation_lowering_rule: id={} source_seed_kind={} target={} pc_relative={} length_power={} external={} relocation_type={}",
            rule.rule_id,
            rule.source_seed_kind,
            rule.target_relocation_kind,
            rule.pc_relative,
            rule.length_power,
            rule.external,
            rule.relocation_type
        );
    }
    for issue in &report.object_image_relocation_lowering_issues {
        println!("  object_image_relocation_lowering_issue: {issue}");
    }
    println!(
        "  object_image_relocation_records: count={} table_hash={}",
        optional_usize_text(report.object_image_relocation_record_count),
        report
            .object_image_relocation_record_table_hash
            .as_deref()
            .unwrap_or("missing")
    );
    for record in &report.object_image_relocation_records {
        println!(
            "  object_image_relocation_record: id={} relocation_seed_id={} source_section_id={} source_offset={} source_seed_kind={} target={} symbol_index={} pc_relative={} length_power={} external={} relocation_type={}",
            record.record_id,
            record.relocation_seed_id,
            record.source_section_id,
            record.source_offset,
            record.source_seed_kind,
            record.target_relocation_kind,
            record.symbol_index,
            record.pc_relative,
            record.length_power,
            record.external,
            record.relocation_type
        );
    }
    for input in &report.link_inputs {
        println!(
            "  link_input: order={} id={} kind={} domain={} package={} native={} dispatch={} contracts={} bytes={} hash={} path={}",
            input.order_index,
            input.input_id,
            input.input_kind,
            input.domain_family,
            input.package_id,
            input.native_ir,
            input.dispatch_lowering,
            input.contract_count,
            input.content_bytes,
            input.content_hash,
            input.path
        );
    }
    println!(
        "  external_dependencies: {}",
        report.external_dependencies.len()
    );
    for dependency in &report.external_dependencies {
        println!("  external_dependency: {dependency}");
    }
    println!("  unresolved: {}", report.unresolved.len());
    for item in &report.unresolved {
        println!("  unresolved_item: {item}");
    }
}

pub(crate) fn print_nsld_closure_emit_report(report: &NsldClosureEmitReport) {
    println!("Nsld linker closure emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  linker_contract_hash: {}", report.linker_contract_hash);
    println!("  closed: {}", report.closed);
    println!(
        "  internal_contract_count: {}",
        report.internal_contract_count
    );
    println!("  unresolved_count: {}", report.unresolved_count);
}

pub(crate) fn print_nsld_closure_verify_report(report: &NsldClosureVerifyReport) {
    println!("Nsld linker closure verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_linker_contract_hash: {}",
        report.expected_linker_contract_hash
    );
    println!(
        "  actual_linker_contract_hash: {}",
        optional_string_text(report.actual_linker_contract_hash.as_deref())
    );
    println!(
        "  expected_container_hash: {}",
        report.expected_container_hash
    );
    println!(
        "  actual_container_hash: {}",
        optional_string_text(report.actual_container_hash.as_deref())
    );
    println!(
        "  expected_payload_size_bytes: {}",
        report.expected_payload_size_bytes
    );
    println!(
        "  actual_payload_size_bytes: {}",
        optional_usize_text(report.actual_payload_size_bytes)
    );
    println!("  expected_payload_hash: {}", report.expected_payload_hash);
    println!(
        "  actual_payload_hash: {}",
        optional_string_text(report.actual_payload_hash.as_deref())
    );
    println!("  expected_closed: {}", report.expected_closed);
    println!(
        "  actual_closed: {}",
        optional_bool_text(report.actual_closed)
    );
    println!(
        "  expected_internal_contract_count: {}",
        report.expected_internal_contract_count
    );
    println!(
        "  actual_internal_contract_count: {}",
        optional_usize_text(report.actual_internal_contract_count)
    );
    println!(
        "  expected_unresolved_count: {}",
        report.expected_unresolved_count
    );
    println!(
        "  actual_unresolved_count: {}",
        optional_usize_text(report.actual_unresolved_count)
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_link_unit_report(report: &NsldLinkUnitReport) {
    println!("Nsld link units");
    println!("  manifest: {}", report.manifest);
    println!("  unit_count: {}", report.unit_count);
    println!("  hetero_unit_count: {}", report.hetero_unit_count);
    println!("  link_input_count: {}", report.link_input_count);
    println!("  clock_edge_count: {}", report.clock_edge_count);
    println!("  data_segment_count: {}", report.data_segment_count);
    println!("  unit_table_hash: {}", report.unit_table_hash);
    for unit in &report.units {
        println!(
            "  link_unit: order={} id={} kind={} domain={} package={} backend={} target={} role={} inputs={} clock_edges={} data_segments={} host_wrapper={} order_key={}",
            unit.order_index,
            unit.unit_id,
            unit.unit_kind,
            unit.domain_family,
            unit.package_id,
            unit.backend_family,
            unit.lowering_target,
            unit.packaging_role,
            unit.link_input_ids.join(","),
            unit.clock_edge_count,
            unit.data_segment_count,
            unit.requires_host_wrapper,
            unit.deterministic_order_key
        );
    }
}

pub(crate) fn print_nsld_link_bundle_report(report: &NsldLinkBundleReport) {
    println!("Nsld link bundle");
    println!("  manifest: {}", report.manifest);
    println!("  bundle_id: {}", report.bundle_id);
    println!("  bundle_hash: {}", report.bundle_hash);
    println!("  bundle_ready: {}", report.bundle_ready);
    println!("  unit_count: {}", report.unit_count);
    println!("  hetero_unit_count: {}", report.hetero_unit_count);
    println!("  link_input_count: {}", report.link_input_count);
    println!(
        "  link_input_total_bytes: {}",
        report.link_input_total_bytes
    );
    println!("  link_input_table_hash: {}", report.link_input_table_hash);
    println!("  unit_table_hash: {}", report.unit_table_hash);
    println!("  clock_edge_count: {}", report.clock_edge_count);
    println!("  data_segment_count: {}", report.data_segment_count);
    println!("  final_stage_link_mode: {}", report.final_stage_link_mode);
    println!("  host_wrapper_required: {}", report.host_wrapper_required);
    println!(
        "  compiled_artifact_path: {}",
        report.compiled_artifact_path
    );
    println!("  native_output_path: {}", report.native_output_path);
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_link_bundle_emit_report(report: &NsldLinkBundleEmitReport) {
    println!("Nsld link bundle emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  bundle_id: {}", report.bundle_id);
    println!("  bundle_hash: {}", report.bundle_hash);
    println!("  bundle_ready: {}", report.bundle_ready);
}

pub(crate) fn print_nsld_link_bundle_verify_report(report: &NsldLinkBundleVerifyReport) {
    println!("Nsld link bundle verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!("  expected_bundle_id: {}", report.expected_bundle_id);
    println!("  expected_bundle_hash: {}", report.expected_bundle_hash);
    println!(
        "  actual_bundle_id: {}",
        report.actual_bundle_id.as_deref().unwrap_or("missing")
    );
    println!(
        "  actual_bundle_hash: {}",
        report.actual_bundle_hash.as_deref().unwrap_or("missing")
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_final_stage_plan_report(report: &NsldFinalStagePlanReport) {
    println!("Nsld final-stage plan");
    println!("  manifest: {}", report.manifest);
    println!("  ready: {}", report.ready);
    println!("  plan_hash: {}", report.plan_hash);
    println!("  final_stage_kind: {}", report.final_stage_kind);
    println!("  final_stage_driver: {}", report.final_stage_driver);
    println!("  final_stage_link_mode: {}", report.final_stage_link_mode);
    println!("  final_output_path: {}", report.final_output_path);
    println!("  host_wrapper_required: {}", report.host_wrapper_required);
    println!("  compatibility_mode: {}", report.compatibility_mode);
    println!("  input_count: {}", report.input_count);
    println!("  container_hash: {}", report.container_hash);
    println!("  payload_hash: {}", report.payload_hash);
    println!("  linker_contract_hash: {}", report.linker_contract_hash);
    println!(
        "  native_object_required: {}",
        report.native_object_required
    );
    println!("  native_object_present: {}", report.native_object_present);
    for input in &report.inputs {
        println!(
            "  final_stage_input: order={} id={} kind={} required={} present={} hash={} path={}",
            input.order_index,
            input.input_id,
            input.input_kind,
            input.required,
            input.present,
            input.content_hash,
            input.path
        );
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
    for note in &report.notes {
        println!("  note: {note}");
    }
}

pub(crate) fn print_nsld_final_stage_plan_emit_report(report: &NsldFinalStagePlanEmitReport) {
    println!("Nsld final-stage plan emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  ready: {}", report.ready);
    println!("  plan_hash: {}", report.plan_hash);
    println!("  input_count: {}", report.input_count);
    println!("  blocker_count: {}", report.blocker_count);
}

pub(crate) fn print_nsld_final_stage_plan_verify_report(report: &NsldFinalStagePlanVerifyReport) {
    println!("Nsld final-stage plan verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!("  expected_plan_hash: {}", report.expected_plan_hash);
    println!(
        "  actual_plan_hash: {}",
        optional_string_text(report.actual_plan_hash.as_deref())
    );
    println!("  expected_input_count: {}", report.expected_input_count);
    println!(
        "  actual_input_count: {}",
        optional_usize_text(report.actual_input_count)
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_final_executable_emit_report(report: &NsldFinalExecutableEmitReport) {
    print_nsld_final_executable_report_with_title(report, "Nsld final executable emit");
}

pub(crate) fn print_nsld_final_executable_readiness_report(report: &NsldFinalExecutableEmitReport) {
    print_nsld_final_executable_report_with_title(report, "Nsld final executable readiness");
}

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
        report
            .actual_command_arg_count
            .map(|value| value.to_string())
            .unwrap_or_else(|| "missing".to_owned())
    );
    for arg in &report.expected_command_args {
        println!("  expected_command_arg: {arg}");
    }
    for arg in &report.actual_command_args {
        println!("  actual_command_arg: {arg}");
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
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_final_executable_layout_plan_report(
    report: &NsldFinalExecutableLayoutPlanReport,
) {
    println!("Nsld final executable layout plan");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  layout_hash: {}", report.layout_hash);
    println!("  final_stage_plan_hash: {}", report.final_stage_plan_hash);
    println!("  final_stage_link_mode: {}", report.final_stage_link_mode);
    println!(
        "  platform_envelope_family: {}",
        report.platform_envelope_family
    );
    println!(
        "  platform_envelope_policy: {}",
        report.platform_envelope_policy
    );
    println!(
        "  internal_binary_format: {}",
        report.internal_binary_format
    );
    println!("  lifecycle_entry_hook: {}", report.lifecycle_entry_hook);
    println!("  scheduler_contract: {}", report.scheduler_contract);
    println!("  data_segment_ordering: {}", report.data_segment_ordering);
    println!("  native_object_path: {}", report.native_object_path);
    println!(
        "  native_object_required: {}",
        report.native_object_required
    );
    println!("  native_object_present: {}", report.native_object_present);
    println!("  compatibility_domain: {}", report.compatibility_domain);
    println!(
        "  compatibility_lifecycle_hook: {}",
        report.compatibility_lifecycle_hook
    );
    println!("  payload_count: {}", report.payload_count);
    println!("  byte_alignment: {}", report.byte_alignment);
    println!("  byte_span: {}", report.byte_span);
    println!("  byte_map_hash: {}", report.byte_map_hash);
    for payload in &report.payload_names {
        println!("  payload: {payload}");
    }
    for payload in &report.payloads {
        println!(
            "  payload_diagnostic: order={} id={} kind={} hook={} required={} present={} hash={} path={}",
            payload.order_index,
            payload.payload_id,
            payload.payload_kind,
            payload.lifecycle_hook,
            payload.required,
            payload.present,
            payload.content_hash,
            payload.path
        );
    }
    for entry in &report.byte_map_entries {
        println!(
            "  byte_map_entry: order={} payload={} kind={} offset={} size={} align={} hash={}",
            entry.order_index,
            entry.payload_id,
            entry.payload_kind,
            entry.offset,
            entry.size_bytes,
            entry.alignment,
            entry.content_hash
        );
    }
    for note in &report.notes {
        println!("  note: {note}");
    }
}

pub(crate) fn print_nsld_final_executable_layout_plan_emit_report(
    report: &NsldFinalExecutableLayoutPlanEmitReport,
) {
    println!("Nsld final executable layout plan emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  layout_hash: {}", report.layout_hash);
    println!("  final_stage_plan_hash: {}", report.final_stage_plan_hash);
    println!("  payload_count: {}", report.payload_count);
    println!("  native_object_present: {}", report.native_object_present);
}

pub(crate) fn print_nsld_final_executable_layout_plan_verify_report(
    report: &NsldFinalExecutableLayoutPlanVerifyReport,
) {
    println!("Nsld final executable layout plan verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!("  expected_layout_hash: {}", report.expected_layout_hash);
    println!(
        "  actual_layout_hash: {}",
        optional_string_text(report.actual_layout_hash.as_deref())
    );
    println!(
        "  expected_payload_count: {}",
        report.expected_payload_count
    );
    println!(
        "  actual_payload_count: {}",
        optional_usize_text(report.actual_payload_count)
    );
    println!("  expected_byte_span: {}", report.expected_byte_span);
    println!(
        "  actual_byte_span: {}",
        optional_usize_text(report.actual_byte_span)
    );
    println!(
        "  expected_byte_map_hash: {}",
        report.expected_byte_map_hash
    );
    println!(
        "  actual_byte_map_hash: {}",
        optional_string_text(report.actual_byte_map_hash.as_deref())
    );
    println!(
        "  expected_lifecycle_entry_hook: {}",
        report.expected_lifecycle_entry_hook
    );
    println!(
        "  actual_lifecycle_entry_hook: {}",
        optional_string_text(report.actual_lifecycle_entry_hook.as_deref())
    );
    println!(
        "  expected_platform_envelope_family: {}",
        report.expected_platform_envelope_family
    );
    println!(
        "  actual_platform_envelope_family: {}",
        optional_string_text(report.actual_platform_envelope_family.as_deref())
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_final_executable_image_dry_run_report(
    report: &NsldFinalExecutableImageDryRunReport,
) {
    println!("Nsld final executable image dry run");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  image_path: {}", report.image_path);
    println!("  image_format: {}", report.image_format);
    println!("  image_magic: {}", report.image_magic);
    println!("  image_header_size: {}", report.image_header_size);
    println!("  payload_byte_offset: {}", report.payload_byte_offset);
    println!("  payload_byte_span: {}", report.payload_byte_span);
    println!("  layout_hash: {}", report.layout_hash);
    println!("  byte_map_hash: {}", report.byte_map_hash);
    println!("  payload_count: {}", report.payload_count);
    println!("  byte_span: {}", report.byte_span);
    println!("  image_constructed: {}", report.image_constructed);
    println!("  image_ready: {}", report.image_ready);
    println!(
        "  image_size_bytes: {}",
        optional_usize_text(report.image_size_bytes)
    );
    println!(
        "  image_hash: {}",
        optional_string_text(report.image_hash.as_deref())
    );
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
}

pub(crate) fn print_nsld_final_executable_image_dry_run_emit_report(
    report: &NsldFinalExecutableImageDryRunEmitReport,
) {
    println!("Nsld final executable image dry run emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  image_path: {}", report.image_path);
    println!("  image_emitted: {}", report.image_emitted);
    println!("  image_constructed: {}", report.image_constructed);
    println!("  image_ready: {}", report.image_ready);
    println!("  image_format: {}", report.image_format);
    println!("  image_header_size: {}", report.image_header_size);
    println!("  payload_byte_offset: {}", report.payload_byte_offset);
    println!(
        "  image_size_bytes: {}",
        optional_usize_text(report.image_size_bytes)
    );
    println!(
        "  image_hash: {}",
        optional_string_text(report.image_hash.as_deref())
    );
}

pub(crate) fn print_nsld_final_executable_image_dry_run_verify_report(
    report: &NsldFinalExecutableImageDryRunVerifyReport,
) {
    println!("Nsld final executable image dry run verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  image_path: {}", report.image_path);
    println!("  valid: {}", report.valid);
    println!("  expected_layout_hash: {}", report.expected_layout_hash);
    println!(
        "  actual_layout_hash: {}",
        optional_string_text(report.actual_layout_hash.as_deref())
    );
    println!(
        "  expected_byte_map_hash: {}",
        report.expected_byte_map_hash
    );
    println!(
        "  actual_byte_map_hash: {}",
        optional_string_text(report.actual_byte_map_hash.as_deref())
    );
    println!("  expected_image_magic: {}", report.expected_image_magic);
    println!(
        "  actual_image_magic: {}",
        optional_string_text(report.actual_image_magic.as_deref())
    );
    println!(
        "  expected_image_version: {}",
        report.expected_image_version
    );
    println!(
        "  actual_image_version: {}",
        report
            .actual_image_version
            .map(|value| value.to_string())
            .unwrap_or_else(|| "missing".to_owned())
    );
    println!(
        "  expected_image_header_size: {}",
        report.expected_image_header_size
    );
    println!(
        "  actual_image_header_size: {}",
        optional_usize_text(report.actual_image_header_size)
    );
    println!(
        "  expected_payload_byte_offset: {}",
        report.expected_payload_byte_offset
    );
    println!(
        "  actual_payload_byte_offset: {}",
        optional_usize_text(report.actual_payload_byte_offset)
    );
    println!(
        "  expected_payload_byte_span: {}",
        report.expected_payload_byte_span
    );
    println!(
        "  actual_payload_byte_span: {}",
        optional_usize_text(report.actual_payload_byte_span)
    );
    println!(
        "  actual_header_layout_hash: {}",
        optional_string_text(report.actual_header_layout_hash.as_deref())
    );
    println!(
        "  actual_header_byte_map_hash: {}",
        optional_string_text(report.actual_header_byte_map_hash.as_deref())
    );
    println!(
        "  expected_payload_region_count: {}",
        report.expected_payload_region_count
    );
    println!(
        "  actual_payload_region_count: {}",
        optional_usize_text(report.actual_payload_region_count)
    );
    println!(
        "  expected_payload_region_hash: {}",
        optional_string_text(report.expected_payload_region_hash.as_deref())
    );
    println!(
        "  actual_payload_region_hash: {}",
        optional_string_text(report.actual_payload_region_hash.as_deref())
    );
    println!(
        "  expected_image_constructed: {}",
        report.expected_image_constructed
    );
    println!(
        "  actual_image_constructed: {}",
        optional_bool_text(report.actual_image_constructed)
    );
    println!("  expected_image_ready: {}", report.expected_image_ready);
    println!(
        "  actual_image_ready: {}",
        optional_bool_text(report.actual_image_ready)
    );
    println!(
        "  expected_image_size_bytes: {}",
        optional_usize_text(report.expected_image_size_bytes)
    );
    println!(
        "  actual_image_size_bytes: {}",
        optional_usize_text(report.actual_image_size_bytes)
    );
    println!(
        "  expected_image_hash: {}",
        optional_string_text(report.expected_image_hash.as_deref())
    );
    println!(
        "  actual_image_hash: {}",
        optional_string_text(report.actual_image_hash.as_deref())
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

fn print_nsld_final_executable_report_with_title(
    report: &NsldFinalExecutableEmitReport,
    title: &str,
) {
    println!("{title}");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  blocked_report_path: {}", report.blocked_report_path);
    println!("  emitted: {}", report.emitted);
    println!(
        "  can_emit_final_executable: {}",
        report.can_emit_final_executable
    );
    println!("  final_stage_ready: {}", report.final_stage_ready);
    println!("  final_stage_plan_hash: {}", report.final_stage_plan_hash);
    println!("  final_stage_driver: {}", report.final_stage_driver);
    println!("  final_stage_link_mode: {}", report.final_stage_link_mode);
    println!("  host_wrapper_required: {}", report.host_wrapper_required);
    println!("  writer_kind: {}", report.writer_kind);
    println!("  writer_status: {}", report.writer_status);
    println!("  writer_input_path: {}", report.writer_input_path);
    println!(
        "  writer_input_valid: {}",
        report
            .writer_input_valid
            .map(|value| value.to_string())
            .unwrap_or_else(|| "not-checked".to_owned())
    );
    println!(
        "  writer_input_hash: {}",
        optional_string_text(report.writer_input_hash.as_deref())
    );
    println!("  input_count: {}", report.input_count);
    println!("  blocker_count: {}", report.blockers.len());
    for blocker in &report.writer_blockers {
        println!("  writer_blocker: {blocker}");
    }
    for issue in &report.writer_input_issues {
        println!("  writer_input_issue: {issue}");
    }
    println!(
        "  host_dry_run_environment_ready: {}",
        report
            .host_dry_run_environment_ready
            .map(|value| value.to_string())
            .unwrap_or_else(|| "not-checked".to_owned())
    );
    println!(
        "  host_dry_run_driver_available: {}",
        report
            .host_dry_run_driver_available
            .map(|value| value.to_string())
            .unwrap_or_else(|| "not-checked".to_owned())
    );
    println!(
        "  host_dry_run_driver_resolved_path: {}",
        optional_string_text(report.host_dry_run_driver_resolved_path.as_deref())
    );
    println!(
        "  host_dry_run_can_invoke: {}",
        report
            .host_dry_run_can_invoke
            .map(|value| value.to_string())
            .unwrap_or_else(|| "not-checked".to_owned())
    );
    println!(
        "  host_dry_run_invocation_policy: {}",
        optional_string_text(report.host_dry_run_invocation_policy.as_deref())
    );
    println!(
        "  host_dry_run_invocation_policy_reason: {}",
        optional_string_text(report.host_dry_run_invocation_policy_reason.as_deref())
    );
    println!(
        "  host_dry_run_command_arg_count: {}",
        report.host_dry_run_command_arg_count
    );
    for arg in &report.host_dry_run_command_args {
        println!("  host_dry_run_command_arg: {arg}");
    }
    for blocker in &report.host_dry_run_blockers {
        println!("  host_dry_run_blocker: {blocker}");
    }
    println!(
        "  host_dry_run_blocker_count: {}",
        report.host_dry_run_blocker_count
    );
    println!("  host_invoke_plan_path: {}", report.host_invoke_plan_path);
    println!(
        "  host_invoke_plan_valid: {}",
        optional_bool_text(report.host_invoke_plan_valid)
    );
    println!(
        "  host_invoke_plan_hash: {}",
        optional_string_text(report.host_invoke_plan_hash.as_deref())
    );
    println!(
        "  host_invoke_plan_would_invoke: {}",
        optional_bool_text(report.host_invoke_plan_would_invoke)
    );
    for issue in &report.host_invoke_plan_issues {
        println!("  host_invoke_plan_issue: {issue}");
    }
    println!("  layout_plan_path: {}", report.layout_plan_path);
    println!(
        "  layout_plan_valid: {}",
        optional_bool_text(report.layout_plan_valid)
    );
    println!(
        "  layout_plan_hash: {}",
        optional_string_text(report.layout_plan_hash.as_deref())
    );
    for issue in &report.layout_plan_issues {
        println!("  layout_plan_issue: {issue}");
    }
    println!("  image_dry_run_path: {}", report.image_dry_run_path);
    println!(
        "  image_dry_run_bytes_path: {}",
        report.image_dry_run_bytes_path
    );
    println!(
        "  image_dry_run_valid: {}",
        optional_bool_text(report.image_dry_run_valid)
    );
    println!(
        "  image_dry_run_hash: {}",
        optional_string_text(report.image_dry_run_hash.as_deref())
    );
    println!(
        "  image_dry_run_size_bytes: {}",
        optional_usize_text(report.image_dry_run_size_bytes)
    );
    for issue in &report.image_dry_run_issues {
        println!("  image_dry_run_issue: {issue}");
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
    for note in &report.notes {
        println!("  note: {note}");
    }
}

pub(crate) fn print_nsld_final_executable_emit_verify_report(
    report: &NsldFinalExecutableEmitVerifyReport,
) {
    println!("Nsld final executable emit verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_final_stage_plan_hash: {}",
        report.expected_final_stage_plan_hash
    );
    println!(
        "  actual_final_stage_plan_hash: {}",
        optional_string_text(report.actual_final_stage_plan_hash.as_deref())
    );
    println!("  expected_emitted: {}", report.expected_emitted);
    println!(
        "  actual_emitted: {}",
        report
            .actual_emitted
            .map(|value| value.to_string())
            .unwrap_or_else(|| "missing".to_owned())
    );
    println!(
        "  expected_writer_input_valid: {}",
        optional_bool_text(report.expected_writer_input_valid)
    );
    println!(
        "  actual_writer_input_valid: {}",
        optional_bool_text(report.actual_writer_input_valid)
    );
    println!(
        "  expected_writer_input_hash: {}",
        optional_string_text(report.expected_writer_input_hash.as_deref())
    );
    println!(
        "  actual_writer_input_hash: {}",
        optional_string_text(report.actual_writer_input_hash.as_deref())
    );
    for issue in &report.expected_writer_input_issues {
        println!("  expected_writer_input_issue: {issue}");
    }
    for issue in &report.actual_writer_input_issues {
        println!("  actual_writer_input_issue: {issue}");
    }
    println!(
        "  expected_host_dry_run_environment_ready: {}",
        optional_bool_text(report.expected_host_dry_run_environment_ready)
    );
    println!(
        "  actual_host_dry_run_environment_ready: {}",
        optional_bool_text(report.actual_host_dry_run_environment_ready)
    );
    println!(
        "  expected_host_dry_run_driver_available: {}",
        optional_bool_text(report.expected_host_dry_run_driver_available)
    );
    println!(
        "  actual_host_dry_run_driver_available: {}",
        optional_bool_text(report.actual_host_dry_run_driver_available)
    );
    println!(
        "  expected_host_dry_run_can_invoke: {}",
        optional_bool_text(report.expected_host_dry_run_can_invoke)
    );
    println!(
        "  actual_host_dry_run_can_invoke: {}",
        optional_bool_text(report.actual_host_dry_run_can_invoke)
    );
    println!(
        "  expected_host_dry_run_driver_resolved_path: {}",
        optional_string_text(report.expected_host_dry_run_driver_resolved_path.as_deref())
    );
    println!(
        "  actual_host_dry_run_driver_resolved_path: {}",
        optional_string_text(report.actual_host_dry_run_driver_resolved_path.as_deref())
    );
    println!(
        "  expected_host_dry_run_invocation_policy: {}",
        optional_string_text(report.expected_host_dry_run_invocation_policy.as_deref())
    );
    println!(
        "  actual_host_dry_run_invocation_policy: {}",
        optional_string_text(report.actual_host_dry_run_invocation_policy.as_deref())
    );
    println!(
        "  expected_host_dry_run_invocation_policy_reason: {}",
        optional_string_text(
            report
                .expected_host_dry_run_invocation_policy_reason
                .as_deref()
        )
    );
    println!(
        "  actual_host_dry_run_invocation_policy_reason: {}",
        optional_string_text(
            report
                .actual_host_dry_run_invocation_policy_reason
                .as_deref()
        )
    );
    println!(
        "  expected_host_dry_run_command_arg_count: {}",
        report.expected_host_dry_run_command_arg_count
    );
    println!(
        "  actual_host_dry_run_command_arg_count: {}",
        optional_usize_text(report.actual_host_dry_run_command_arg_count)
    );
    for arg in &report.expected_host_dry_run_command_args {
        println!("  expected_host_dry_run_command_arg: {arg}");
    }
    for arg in &report.actual_host_dry_run_command_args {
        println!("  actual_host_dry_run_command_arg: {arg}");
    }
    println!(
        "  expected_host_dry_run_blocker_count: {}",
        report.expected_host_dry_run_blocker_count
    );
    println!(
        "  actual_host_dry_run_blocker_count: {}",
        optional_usize_text(report.actual_host_dry_run_blocker_count)
    );
    for blocker in &report.expected_host_dry_run_blockers {
        println!("  expected_host_dry_run_blocker: {blocker}");
    }
    for blocker in &report.actual_host_dry_run_blockers {
        println!("  actual_host_dry_run_blocker: {blocker}");
    }
    println!(
        "  expected_host_invoke_plan_valid: {}",
        optional_bool_text(report.expected_host_invoke_plan_valid)
    );
    println!(
        "  actual_host_invoke_plan_valid: {}",
        optional_bool_text(report.actual_host_invoke_plan_valid)
    );
    println!(
        "  expected_host_invoke_plan_would_invoke: {}",
        optional_bool_text(report.expected_host_invoke_plan_would_invoke)
    );
    println!(
        "  actual_host_invoke_plan_would_invoke: {}",
        optional_bool_text(report.actual_host_invoke_plan_would_invoke)
    );
    println!(
        "  expected_host_invoke_plan_hash: {}",
        optional_string_text(report.expected_host_invoke_plan_hash.as_deref())
    );
    println!(
        "  actual_host_invoke_plan_hash: {}",
        optional_string_text(report.actual_host_invoke_plan_hash.as_deref())
    );
    for issue in &report.expected_host_invoke_plan_issues {
        println!("  expected_host_invoke_plan_issue: {issue}");
    }
    for issue in &report.actual_host_invoke_plan_issues {
        println!("  actual_host_invoke_plan_issue: {issue}");
    }
    println!(
        "  expected_layout_plan_valid: {}",
        optional_bool_text(report.expected_layout_plan_valid)
    );
    println!(
        "  actual_layout_plan_valid: {}",
        optional_bool_text(report.actual_layout_plan_valid)
    );
    println!(
        "  expected_layout_plan_hash: {}",
        optional_string_text(report.expected_layout_plan_hash.as_deref())
    );
    println!(
        "  actual_layout_plan_hash: {}",
        optional_string_text(report.actual_layout_plan_hash.as_deref())
    );
    for issue in &report.expected_layout_plan_issues {
        println!("  expected_layout_plan_issue: {issue}");
    }
    for issue in &report.actual_layout_plan_issues {
        println!("  actual_layout_plan_issue: {issue}");
    }
    println!(
        "  expected_image_dry_run_valid: {}",
        optional_bool_text(report.expected_image_dry_run_valid)
    );
    println!(
        "  actual_image_dry_run_valid: {}",
        optional_bool_text(report.actual_image_dry_run_valid)
    );
    println!(
        "  expected_image_dry_run_hash: {}",
        optional_string_text(report.expected_image_dry_run_hash.as_deref())
    );
    println!(
        "  actual_image_dry_run_hash: {}",
        optional_string_text(report.actual_image_dry_run_hash.as_deref())
    );
    println!(
        "  expected_image_dry_run_size_bytes: {}",
        optional_usize_text(report.expected_image_dry_run_size_bytes)
    );
    println!(
        "  actual_image_dry_run_size_bytes: {}",
        optional_usize_text(report.actual_image_dry_run_size_bytes)
    );
    for issue in &report.expected_image_dry_run_issues {
        println!("  expected_image_dry_run_issue: {issue}");
    }
    for issue in &report.actual_image_dry_run_issues {
        println!("  actual_image_dry_run_issue: {issue}");
    }
    println!(
        "  expected_blocker_count: {}",
        report.expected_blocker_count
    );
    println!(
        "  actual_blocker_count: {}",
        optional_usize_text(report.actual_blocker_count)
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_prepare_report(report: &NsldPrepareReport) {
    println!("Nsld prepare");
    println!("  manifest: {}", report.manifest);
    println!("  valid: {}", report.valid);
    println!("  output_dir: {}", report.output_dir);
    println!("  link_input_table: {}", report.link_input_table_path);
    println!("  link_unit_table: {}", report.link_unit_table_path);
    println!("  link_bundle: {}", report.link_bundle_path);
    println!("  assemble_plan: {}", report.assemble_plan_path);
    println!("  section_manifest: {}", report.section_manifest_path);
    println!("  object_plan: {}", report.object_plan_path);
    println!("  object_writer_input: {}", report.object_writer_input_path);
    println!("  object_byte_layout: {}", report.object_byte_layout_path);
    println!("  object_file_layout: {}", report.object_file_layout_path);
    println!(
        "  object_image_dry_run: {}",
        report.object_image_dry_run_path
    );
    println!(
        "  object_image_dry_run_bytes: {}",
        report.object_image_dry_run_bytes_path
    );
    println!("  object_emit_blocked: {}", report.object_emit_blocked_path);
    println!("  object_output: {}", report.object_output_path);
    println!(
        "  object_writer_dry_run: {}",
        report.object_writer_dry_run_path
    );
    println!("  container_plan: {}", report.container_plan_path);
    println!("  container: {}", report.container_path);
    println!("  container_payload: {}", report.container_payload_path);
    println!("  closure_snapshot: {}", report.closure_snapshot_path);
    println!("  final_stage_plan: {}", report.final_stage_plan_path);
    println!(
        "  final_executable_writer_input: {}",
        report.final_executable_writer_input_path
    );
    println!(
        "  final_executable_host_invoke_plan: {}",
        report.final_executable_host_invoke_plan_path
    );
    println!(
        "  final_executable_layout_plan: {}",
        report.final_executable_layout_plan_path
    );
    println!(
        "  final_executable_image_dry_run: {}",
        report.final_executable_image_dry_run_path
    );
    println!(
        "  final_executable_image_dry_run_bytes: {}",
        report.final_executable_image_dry_run_bytes_path
    );
    println!(
        "  final_executable_blocked: {}",
        report.final_executable_blocked_path
    );
    println!("  link_input_count: {}", report.link_input_count);
    println!("  link_input_table_hash: {}", report.link_input_table_hash);
    println!("  unit_count: {}", report.unit_count);
    println!("  unit_table_hash: {}", report.unit_table_hash);
    println!("  bundle_id: {}", report.bundle_id);
    println!("  bundle_hash: {}", report.bundle_hash);
    println!("  bundle_ready: {}", report.bundle_ready);
    println!("  assemble_plan_hash: {}", report.assemble_plan_hash);
    println!("  section_table_hash: {}", report.section_table_hash);
    println!("  object_plan_hash: {}", report.object_plan_hash);
    println!("  object_emitted: {}", report.object_emitted);
    println!("  byte_layout_hash: {}", report.byte_layout_hash);
    println!("  file_layout_hash: {}", report.file_layout_hash);
    println!(
        "  object_image_hash: {}",
        report.object_image_hash.as_deref().unwrap_or("missing")
    );
    println!(
        "  object_image_relocation_lowering: valid={} rule_count={}",
        report.object_image_relocation_lowering_valid,
        report.object_image_relocation_lowering_rule_count
    );
    for rule in &report.object_image_relocation_lowering_rules {
        println!(
            "  object_image_relocation_lowering_rule: id={} source_seed_kind={} target={} pc_relative={} length_power={} external={} relocation_type={}",
            rule.rule_id,
            rule.source_seed_kind,
            rule.target_relocation_kind,
            rule.pc_relative,
            rule.length_power,
            rule.external,
            rule.relocation_type
        );
    }
    for issue in &report.object_image_relocation_lowering_issues {
        println!("  object_image_relocation_lowering_issue: {issue}");
    }
    println!(
        "  object_image_relocation_records: count={} table_hash={}",
        report.object_image_relocation_record_count,
        report.object_image_relocation_record_table_hash
    );
    for record in &report.object_image_relocation_records {
        println!(
            "  object_image_relocation_record: id={} relocation_seed_id={} source_section_id={} source_offset={} source_seed_kind={} target={} symbol_index={} pc_relative={} length_power={} external={} relocation_type={}",
            record.record_id,
            record.relocation_seed_id,
            record.source_section_id,
            record.source_offset,
            record.source_seed_kind,
            record.target_relocation_kind,
            record.symbol_index,
            record.pc_relative,
            record.length_power,
            record.external,
            record.relocation_type
        );
    }
    println!("  metadata_table_hash: {}", report.metadata_table_hash);
    println!(
        "  compatibility_domain: count={} table_hash={} id={} kind={} paradigm={} hook={} abi={} wrapper={} required={}",
        optional_usize_text(report.compatibility_domain_count),
        optional_string_text(report.compatibility_domain_table_hash.as_deref()),
        optional_string_text(report.compatibility_domain_id.as_deref()),
        optional_string_text(report.compatibility_domain_kind.as_deref()),
        optional_string_text(report.compatibility_domain_paradigm.as_deref()),
        optional_string_text(report.compatibility_domain_lifecycle_hook.as_deref()),
        optional_string_text(report.compatibility_domain_abi_family.as_deref()),
        optional_string_text(report.compatibility_domain_wrapper_policy.as_deref()),
        optional_bool_text(report.compatibility_domain_required)
    );
    println!("  container_layout_hash: {}", report.container_layout_hash);
    println!("  container_hash: {}", report.container_hash);
    println!("  payload_size_bytes: {}", report.payload_size_bytes);
    println!("  payload_hash: {}", report.payload_hash);
    println!(
        "  final_stage_plan: ready={} hash={} blockers={}",
        report.final_stage_plan_ready,
        report.final_stage_plan_hash,
        report.final_stage_plan_blocker_count
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_assemble_plan_report(report: &NsldAssemblePlanReport) {
    println!("Nsld assemble plan");
    println!("  manifest: {}", report.manifest);
    println!("  ready: {}", report.ready);
    println!("  bundle_id: {}", report.bundle_id);
    println!("  bundle_hash: {}", report.bundle_hash);
    println!("  assemble_plan_hash: {}", report.assemble_plan_hash);
    println!("  section_count: {}", report.section_count);
    print_assemble_sections(&report.sections);
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
}

pub(crate) fn print_nsld_assemble_plan_emit_report(report: &NsldAssemblePlanEmitReport) {
    println!("Nsld assemble plan emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  ready: {}", report.ready);
    println!("  assemble_plan_hash: {}", report.assemble_plan_hash);
    println!("  section_count: {}", report.section_count);
}

pub(crate) fn print_nsld_assemble_plan_verify_report(report: &NsldAssemblePlanVerifyReport) {
    println!("Nsld assemble plan verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_assemble_plan_hash: {}",
        report.expected_assemble_plan_hash
    );
    println!(
        "  expected_section_count: {}",
        report.expected_section_count
    );
    println!(
        "  actual_assemble_plan_hash: {}",
        report
            .actual_assemble_plan_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_section_count: {}",
        optional_usize_text(report.actual_section_count)
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_section_manifest_report(report: &NsldSectionManifestReport) {
    println!("Nsld section manifest");
    println!("  manifest: {}", report.manifest);
    println!("  ready: {}", report.ready);
    println!("  assemble_plan_hash: {}", report.assemble_plan_hash);
    println!("  section_count: {}", report.section_count);
    println!("  section_table_hash: {}", report.section_table_hash);
    print_assemble_sections(&report.sections);
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
}

pub(crate) fn print_nsld_section_manifest_emit_report(report: &NsldSectionManifestEmitReport) {
    println!("Nsld section manifest emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  ready: {}", report.ready);
    println!("  section_count: {}", report.section_count);
    println!("  section_table_hash: {}", report.section_table_hash);
}

pub(crate) fn print_nsld_section_manifest_verify_report(report: &NsldSectionManifestVerifyReport) {
    println!("Nsld section manifest verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_section_count: {}",
        report.expected_section_count
    );
    println!(
        "  expected_section_table_hash: {}",
        report.expected_section_table_hash
    );
    println!(
        "  actual_section_count: {}",
        optional_usize_text(report.actual_section_count)
    );
    println!(
        "  actual_section_table_hash: {}",
        report
            .actual_section_table_hash
            .as_deref()
            .unwrap_or("missing")
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_assemble_sections(sections: &[NsldAssembleSectionDiagnostic]) {
    for section in sections {
        println!(
            "  section: order={} id={} kind={} required={} hash={} source={}",
            section.order_index,
            section.section_id,
            section.section_kind,
            section.required,
            section.source_hash,
            section.source_path
        );
    }
}

pub(crate) fn optional_usize_text(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "missing".to_owned())
}

pub(crate) fn optional_string_text(value: Option<&str>) -> String {
    value.unwrap_or("missing").to_owned()
}

pub(crate) fn optional_bool_text(value: Option<bool>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "absent".to_owned())
}
