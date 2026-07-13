use super::{display_text::*, reports::*};

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
    println!(
        "  advisory_command_id: {}",
        optional_string_text(report.advisory_command_id.as_deref())
    );
    println!(
        "  advisory_command: {}",
        optional_string_text(report.advisory_command.as_deref())
    );
    println!(
        "  advisory_command_resolved: {}",
        optional_string_text(report.advisory_command_resolved.as_deref())
    );
    println!(
        "  advisory_command_reason: {}",
        optional_string_text(report.advisory_command_reason.as_deref())
    );
    println!(
        "  next_action_command_id: {}",
        optional_string_text(report.next_action_command_id.as_deref())
    );
    println!(
        "  next_action_command: {}",
        optional_string_text(report.next_action_command.as_deref())
    );
    println!(
        "  next_action_command_resolved: {}",
        optional_string_text(report.next_action_command_resolved.as_deref())
    );
    println!(
        "  next_action_command_reason: {}",
        optional_string_text(report.next_action_command_reason.as_deref())
    );
    println!(
        "  next_action_source: {}",
        optional_string_text(report.next_action_source.as_deref())
    );
    println!("  next_action_available: {}", report.next_action_available);
    println!(
        "  final_output_boundary_ready: {}",
        report.final_output_boundary_ready
    );
    println!(
        "  final_output_boundary_command_id: {}",
        optional_string_text(report.final_output_boundary_command_id.as_deref())
    );
    println!(
        "  final_output_boundary_command: {}",
        optional_string_text(report.final_output_boundary_command.as_deref())
    );
    println!(
        "  final_output_boundary_command_resolved: {}",
        optional_string_text(report.final_output_boundary_command_resolved.as_deref())
    );
    println!(
        "  final_output_boundary_reason: {}",
        optional_string_text(report.final_output_boundary_reason.as_deref())
    );
    for blocker in &report.final_output_boundary_blockers {
        println!("  final_output_boundary_blocker: {blocker}");
    }
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
    for advisory in &report.advisories {
        println!("  advisory: {advisory}");
    }
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_closure_report(report: &NsldClosureReport) {
    println!("Nsld linker closure");
    println!("  manifest: {}", report.manifest);
    println!("  closed: {}", report.closed);
    println!(
        "  lowering_plan_index_source: {}",
        report.lowering_plan_index_source
    );
    println!(
        "  lowering_plan_index_available: {}",
        report.lowering_plan_index_available
    );
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
    println!("  hetero_node_count: {}", report.hetero_node_count);
    println!("  clock_edge_count: {}", report.clock_edge_count);
    println!("  data_segment_count: {}", report.data_segment_count);
    println!("  unit_table_hash: {}", report.unit_table_hash);
    for unit in &report.units {
        println!(
            "  link_unit: order={} id={} kind={} domain={} package={} backend={} target={} role={} inputs={} hetero_nodes={} timestamps={} lifecycle_hooks={} wait_events={} emit_events={} clock_edges={} data_segments={} host_wrapper={} order_key={}",
            unit.order_index,
            unit.unit_id,
            unit.unit_kind,
            unit.domain_family,
            unit.package_id,
            unit.backend_family,
            unit.lowering_target,
            unit.packaging_role,
            unit.link_input_ids.join(","),
            unit.hetero_node_count,
            unit.hetero_timestamps.join(","),
            unit.lifecycle_hooks.join(","),
            unit.wait_event_count,
            unit.emit_event_count,
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
