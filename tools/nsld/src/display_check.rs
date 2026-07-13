use super::{
    display::{optional_bool_text, optional_string_text, optional_usize_text},
    reports::NsldCheckReport,
};

pub(crate) fn print_check_report(report: &NsldCheckReport) {
    println!("Nsld linker check");
    println!("  manifest: {}", report.manifest);
    println!("  valid: {}", report.valid);
    println!("  checks: {}", report.checks);
    println!("  failures: {}", report.failures);
    println!("  advisories: {}", report.advisory_count);
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
        "  next_action_source: {}",
        optional_string_text(report.next_action_source.as_deref())
    );
    println!("  next_action_available: {}", report.next_action_available);
    println!(
        "  artifact_lowering_alignment: consistent={} mismatches={}",
        report.artifact_lowering_alignment_consistent,
        report.artifact_lowering_alignment_mismatches
    );
    println!("  clock_protocol: valid={}", report.clock_protocol_valid);
    println!(
        "  hetero_calculate: valid={}",
        report.hetero_calculate_valid
    );
    println!(
        "  hetero_static_lifecycle: static_link={} lifecycle_driven={}",
        report.static_link, report.lifecycle_driven
    );
    println!(
        "  sidecar_capabilities: valid={} issues={}",
        report.sidecar_capability_valid,
        report.sidecar_capability_issues.len()
    );
    println!(
        "  link_input_table: present={} valid={}",
        report.link_input_table_present,
        optional_bool_text(report.link_input_table_valid)
    );
    println!(
        "  link_unit_table: present={} valid={}",
        report.link_unit_table_present,
        optional_bool_text(report.link_unit_table_valid)
    );
    println!(
        "  link_bundle: present={} valid={}",
        report.link_bundle_present,
        optional_bool_text(report.link_bundle_valid)
    );
    println!(
        "  assemble_plan: present={} valid={}",
        report.assemble_plan_present,
        optional_bool_text(report.assemble_plan_valid)
    );
    println!(
        "  section_manifest: present={} valid={}",
        report.section_manifest_present,
        optional_bool_text(report.section_manifest_valid)
    );
    println!(
        "  object_plan: present={} valid={}",
        report.object_plan_present,
        optional_bool_text(report.object_plan_valid)
    );
    println!(
        "  object_writer_input: present={} valid={}",
        report.object_writer_input_present,
        optional_bool_text(report.object_writer_input_valid)
    );
    println!(
        "  object_byte_layout: present={} valid={}",
        report.object_byte_layout_present,
        optional_bool_text(report.object_byte_layout_valid)
    );
    println!(
        "  object_file_layout: present={} valid={}",
        report.object_file_layout_present,
        optional_bool_text(report.object_file_layout_valid)
    );
    println!(
        "  object_image_dry_run: present={} valid={} bytes_present={}",
        report.object_image_dry_run_present,
        optional_bool_text(report.object_image_dry_run_valid),
        report.object_image_dry_run_bytes_present
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
    println!(
        "  object_emit_blocked: present={} valid={}",
        report.object_emit_blocked_present,
        optional_bool_text(report.object_emit_blocked_valid)
    );
    println!(
        "  object_output: present={} valid={} expected_size={} actual_size={} expected_hash={} actual_hash={}",
        report.object_output_present,
        optional_bool_text(report.object_output_valid),
        optional_usize_text(report.object_output_expected_size_bytes),
        optional_usize_text(report.object_output_actual_size_bytes),
        optional_string_text(report.object_output_expected_hash.as_deref()),
        optional_string_text(report.object_output_actual_hash.as_deref())
    );
    println!(
        "  object_writer_dry_run: present={} valid={}",
        report.object_writer_dry_run_present,
        optional_bool_text(report.object_writer_dry_run_valid)
    );
    println!(
        "  container_plan: present={} valid={}",
        report.container_plan_present,
        optional_bool_text(report.container_plan_valid)
    );
    println!(
        "  container: present={} valid={}",
        report.container_present,
        optional_bool_text(report.container_valid)
    );
    println!(
        "  container_tables: sections={} loader_symbols={} relocations={} compatibility_domains={} external_imports={}",
        report.container_section_issues.len(),
        report.container_loader_symbol_issues.len(),
        report.container_relocation_issues.len(),
        report.container_compatibility_domain_issues.len(),
        report.container_external_import_issues.len()
    );
    println!(
        "  container_payload: present={} issues={}",
        report.container_payload_present,
        report.container_payload_issues.len()
    );
    println!(
        "  closure_snapshot: present={} valid={} linker_contract_hash={} container_hash={} payload_size={} payload_hash={} issues={}",
        report.closure_snapshot_present,
        optional_bool_text(report.closure_snapshot_valid),
        optional_string_text(report.closure_snapshot_linker_contract_hash.as_deref()),
        optional_string_text(report.closure_snapshot_container_hash.as_deref()),
        optional_usize_text(report.closure_snapshot_payload_size_bytes),
        optional_string_text(report.closure_snapshot_payload_hash.as_deref()),
        report.closure_snapshot_issues.len()
    );
    println!(
        "  final_stage_plan: present={} valid={} ready={} hash={} blockers={} issues={}",
        report.final_stage_plan_present,
        optional_bool_text(report.final_stage_plan_valid),
        optional_bool_text(report.final_stage_plan_ready),
        optional_string_text(report.final_stage_plan_hash.as_deref()),
        optional_usize_text(report.final_stage_plan_blocker_count),
        report.final_stage_plan_issues.len()
    );
    println!(
        "  final_executable_writer_input: present={} valid={} hash={} command_args={} issues={}",
        report.final_executable_writer_input_present,
        optional_bool_text(report.final_executable_writer_input_valid),
        optional_string_text(report.final_executable_writer_input_hash.as_deref()),
        optional_usize_text(report.final_executable_writer_input_command_arg_count),
        report.final_executable_writer_input_issues.len()
    );
    println!(
        "  final_executable_host_invoke_plan: present={} valid={} hash={} policy={} explicit_allow={} allow_present={} would_invoke={} blockers={} issues={}",
        report.final_executable_host_invoke_plan_present,
        optional_bool_text(report.final_executable_host_invoke_plan_valid),
        optional_string_text(report.final_executable_host_invoke_plan_hash.as_deref()),
        optional_string_text(
            report
                .final_executable_host_invoke_plan_invocation_policy
                .as_deref()
        ),
        optional_bool_text(report.final_executable_host_invoke_plan_requires_explicit_allow),
        optional_bool_text(report.final_executable_host_invoke_plan_explicit_allow_present),
        optional_bool_text(report.final_executable_host_invoke_plan_would_invoke),
        optional_usize_text(report.final_executable_host_invoke_plan_blocker_count),
        report.final_executable_host_invoke_plan_issues.len()
    );
    println!(
        "  final_executable_layout_plan: present={} valid={} hash={} payloads={} issues={}",
        report.final_executable_layout_plan_present,
        optional_bool_text(report.final_executable_layout_plan_valid),
        optional_string_text(report.final_executable_layout_plan_hash.as_deref()),
        optional_usize_text(report.final_executable_layout_plan_payload_count),
        report.final_executable_layout_plan_issues.len()
    );
    println!(
        "  final_executable_image_dry_run: present={} valid={} hash={} size={} issues={}",
        report.final_executable_image_dry_run_present,
        optional_bool_text(report.final_executable_image_dry_run_valid),
        optional_string_text(report.final_executable_image_dry_run_hash.as_deref()),
        optional_usize_text(report.final_executable_image_dry_run_size_bytes),
        report.final_executable_image_dry_run_issues.len()
    );
    println!(
        "  final_executable_blocked: present={} valid={} emitted={} hash={} blockers={} issues={}",
        report.final_executable_blocked_present,
        optional_bool_text(report.final_executable_blocked_valid),
        optional_bool_text(report.final_executable_blocked_emitted),
        optional_string_text(report.final_executable_blocked_plan_hash.as_deref()),
        optional_usize_text(report.final_executable_blocked_blocker_count),
        report.final_executable_blocked_issues.len()
    );
    println!(
        "  final_executable_output: status={} path_present={} kind={} validation={} nsld_owned={} present={} header_required={} header_valid={} magic={} version={} runnable={} size={} hash={} blockers={} issues={}",
        report.final_executable_output_boundary_status,
        report.final_executable_output_path_present,
        report.final_executable_output_kind,
        report.final_executable_output_validation_mode,
        report.final_executable_output_nsld_owned,
        report.final_executable_output_present,
        optional_bool_text(report.final_executable_output_image_header_required),
        optional_bool_text(report.final_executable_output_image_header_valid),
        optional_string_text(report.final_executable_output_image_magic.as_deref()),
        optional_usize_text(report.final_executable_output_image_version),
        optional_bool_text(report.final_executable_output_runnable_candidate),
        optional_usize_text(report.final_executable_output_size_bytes),
        optional_string_text(report.final_executable_output_hash.as_deref()),
        optional_usize_text(report.final_executable_output_blocker_count),
        report.final_executable_output_issues.len()
    );
    println!(
        "  final_executable_output_image_hashes: layout={} byte_map={}",
        optional_string_text(report.final_executable_output_image_layout_hash.as_deref()),
        optional_string_text(
            report
                .final_executable_output_image_byte_map_hash
                .as_deref()
        )
    );
    println!(
        "  final_executable_launcher_manifest: present={} valid={} ready={} hash={} blockers={} issues={}",
        report.final_executable_launcher_manifest_present,
        optional_bool_text(report.final_executable_launcher_manifest_valid),
        optional_bool_text(report.final_executable_launcher_manifest_ready),
        optional_string_text(report.final_executable_launcher_manifest_hash.as_deref()),
        optional_usize_text(report.final_executable_launcher_manifest_blocker_count),
        report.final_executable_launcher_manifest_issues.len()
    );
    println!(
        "  final_executable_launcher_dry_run: present={} valid={} ready={} enters_hook={} hash={} blockers={} issues={}",
        report.final_executable_launcher_dry_run_present,
        optional_bool_text(report.final_executable_launcher_dry_run_valid),
        optional_bool_text(report.final_executable_launcher_dry_run_ready),
        optional_bool_text(report.final_executable_launcher_dry_run_would_enter_lifecycle_hook),
        optional_string_text(report.final_executable_launcher_dry_run_hash.as_deref()),
        optional_usize_text(report.final_executable_launcher_dry_run_blocker_count),
        report.final_executable_launcher_dry_run_issues.len()
    );
    println!(
        "  final_executable_pipeline: present={} valid={} ready={} emitted={} hash={} required_paths={} present_paths={} missing_paths={} blockers={} issues={}",
        report.final_executable_pipeline_present,
        optional_bool_text(report.final_executable_pipeline_valid),
        optional_bool_text(report.final_executable_pipeline_ready),
        optional_bool_text(report.final_executable_pipeline_emitted),
        optional_string_text(report.final_executable_pipeline_hash.as_deref()),
        optional_usize_text(report.final_executable_pipeline_required_stage_path_count),
        optional_usize_text(report.final_executable_pipeline_required_stage_path_present_count),
        report
            .final_executable_pipeline_missing_required_stage_paths
            .len(),
        optional_usize_text(report.final_executable_pipeline_blocker_count),
        report.final_executable_pipeline_issues.len()
    );
    println!(
        "  final_executable_pipeline_scheduler_metadata: payload_id={} present={} hash={}",
        optional_string_text(
            report
                .final_executable_pipeline_scheduler_metadata_payload_id
                .as_deref()
        ),
        optional_bool_text(report.final_executable_pipeline_scheduler_metadata_present),
        optional_string_text(
            report
                .final_executable_pipeline_scheduler_metadata_hash
                .as_deref()
        )
    );
    println!(
        "  container_loader: readiness={} blockers={} metadata_table_hash={} external_imports={}",
        optional_string_text(report.container_loader_readiness.as_deref()),
        report.container_loader_blockers.len(),
        optional_string_text(report.container_metadata_table_hash.as_deref()),
        optional_usize_text(report.container_external_import_count)
    );
    println!(
        "  container_compatibility_domain: count={} table_hash={} id={} kind={} paradigm={} hook={} abi={} wrapper={} required={}",
        optional_usize_text(report.container_compatibility_domain_count),
        optional_string_text(report.container_compatibility_domain_table_hash.as_deref()),
        optional_string_text(report.container_compatibility_domain_id.as_deref()),
        optional_string_text(report.container_compatibility_domain_kind.as_deref()),
        optional_string_text(report.container_compatibility_domain_paradigm.as_deref()),
        optional_string_text(report.container_compatibility_domain_lifecycle_hook.as_deref()),
        optional_string_text(report.container_compatibility_domain_abi_family.as_deref()),
        optional_string_text(report.container_compatibility_domain_wrapper_policy.as_deref()),
        optional_bool_text(report.container_compatibility_domain_required)
    );
    println!(
        "  container_native_object: section_present={} section_id={} loader_symbol_present={} loader_symbol_id={} relocation_present={} relocation_id={}",
        report.container_native_object_section_present,
        optional_string_text(report.container_native_object_section_id.as_deref()),
        report.container_native_object_loader_symbol_present,
        optional_string_text(report.container_native_object_loader_symbol_id.as_deref()),
        report.container_native_object_relocation_present,
        optional_string_text(report.container_native_object_relocation_id.as_deref())
    );
    println!(
        "  container_shader: section_present={} section_id={} loader_symbol_present={} loader_symbol_id={} relocation_present={} relocation_id={}",
        report.container_shader_section_present,
        optional_string_text(report.container_shader_section_id.as_deref()),
        report.container_shader_loader_symbol_present,
        optional_string_text(report.container_shader_loader_symbol_id.as_deref()),
        report.container_shader_relocation_present,
        optional_string_text(report.container_shader_relocation_id.as_deref())
    );
    println!(
        "  container_kernel: section_present={} section_id={} loader_symbol_present={} loader_symbol_id={} relocation_present={} relocation_id={}",
        report.container_kernel_section_present,
        optional_string_text(report.container_kernel_section_id.as_deref()),
        report.container_kernel_loader_symbol_present,
        optional_string_text(report.container_kernel_loader_symbol_id.as_deref()),
        report.container_kernel_relocation_present,
        optional_string_text(report.container_kernel_relocation_id.as_deref())
    );
    println!(
        "  artifact_chain: valid={} advisories={} issues={}",
        report.artifact_chain_valid,
        report.advisory_count,
        report.artifact_chain_issues.len()
    );
    println!(
        "  artifact_chain_advisory_command_id: {}",
        optional_string_text(report.artifact_chain_advisory_command_id.as_deref())
    );
    println!(
        "  artifact_chain_advisory_command_resolved: {}",
        optional_string_text(report.artifact_chain_advisory_command_resolved.as_deref())
    );
    println!(
        "  artifact_chain_next_action_command_id: {}",
        optional_string_text(report.artifact_chain_next_action_command_id.as_deref())
    );
    println!(
        "  artifact_chain_next_action_command: {}",
        optional_string_text(report.artifact_chain_next_action_command.as_deref())
    );
    println!(
        "  artifact_chain_next_action_command_resolved: {}",
        optional_string_text(
            report
                .artifact_chain_next_action_command_resolved
                .as_deref()
        )
    );
    println!(
        "  artifact_chain_next_action_source: {}",
        optional_string_text(report.artifact_chain_next_action_source.as_deref())
    );
    println!(
        "  artifact_chain_next_action_available: {}",
        report.artifact_chain_next_action_available
    );
    println!(
        "  artifact_chain_final_output_boundary_ready: {}",
        report.artifact_chain_final_output_boundary_ready
    );
    println!(
        "  artifact_chain_final_output_boundary_command_id: {}",
        optional_string_text(
            report
                .artifact_chain_final_output_boundary_command_id
                .as_deref()
        )
    );
    println!(
        "  artifact_chain_final_output_boundary_command: {}",
        optional_string_text(
            report
                .artifact_chain_final_output_boundary_command
                .as_deref()
        )
    );
    println!(
        "  artifact_chain_final_output_boundary_command_resolved: {}",
        optional_string_text(
            report
                .artifact_chain_final_output_boundary_command_resolved
                .as_deref()
        )
    );
    println!(
        "  artifact_chain_final_output_boundary_reason: {}",
        optional_string_text(
            report
                .artifact_chain_final_output_boundary_reason
                .as_deref()
        )
    );
    for blocker in &report.artifact_chain_final_output_boundary_blockers {
        println!("  artifact_chain_final_output_boundary_blocker: {blocker}");
    }
    println!("  final_stage_link_mode: {}", report.final_stage_link_mode);
    println!("  domains: {}", report.domains.len());
    for domain in &report.domains {
        println!(
            "  domain: {} package={} kind={} lowering={} backend={} alignment_consistent={}",
            domain.domain_family,
            domain.package_id,
            domain.kind,
            domain.lowering_target,
            domain.backend_family,
            domain.alignment_consistent
        );
        for issue in &domain.alignment_issues {
            println!("    domain_issue: {issue}");
        }
    }
    println!(
        "  sidecar_capabilities: {}",
        report.sidecar_capabilities.len()
    );
    for capability in &report.sidecar_capabilities {
        println!(
            "  sidecar_capability: {} package={} owner={} frontend={} native={} dispatch={} valid={} contracts={}",
            capability.domain_family,
            capability.package_id,
            capability.capability_owner,
            capability.frontend_ir,
            capability.native_ir,
            capability.dispatch_lowering,
            capability.valid,
            capability.validation_contracts.len()
        );
        for issue in &capability.issues {
            println!("    sidecar_capability_issue: {issue}");
        }
    }
    println!("  clock_edges: {}", report.clock_edges.len());
    for edge in &report.clock_edges {
        println!(
            "  clock_edge: index={} from={} to={} relation={} source={}",
            edge.index, edge.from, edge.to, edge.relation, edge.source
        );
    }
    println!("  data_segments: {}", report.data_segments.len());
    for segment in &report.data_segments {
        println!(
            "  data_segment: index={} id={} domain={} owner={} order={} phase={} source={}",
            segment.index,
            segment.segment_id,
            segment.domain_family,
            segment.owner_package,
            segment.order_key,
            segment.access_phase,
            segment.source_path
        );
    }
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
    for issue in &report.link_input_table_issues {
        println!("  link_input_table_issue: {issue}");
    }
    for issue in &report.link_unit_table_issues {
        println!("  link_unit_table_issue: {issue}");
    }
    for issue in &report.link_bundle_issues {
        println!("  link_bundle_issue: {issue}");
    }
    for issue in &report.assemble_plan_issues {
        println!("  assemble_plan_issue: {issue}");
    }
    for issue in &report.section_manifest_issues {
        println!("  section_manifest_issue: {issue}");
    }
    for issue in &report.object_plan_issues {
        println!("  object_plan_issue: {issue}");
    }
    for issue in &report.object_writer_input_issues {
        println!("  object_writer_input_issue: {issue}");
    }
    for issue in &report.object_byte_layout_issues {
        println!("  object_byte_layout_issue: {issue}");
    }
    for issue in &report.object_file_layout_issues {
        println!("  object_file_layout_issue: {issue}");
    }
    for issue in &report.object_image_dry_run_issues {
        println!("  object_image_dry_run_issue: {issue}");
    }
    for issue in &report.object_image_relocation_lowering_issues {
        println!("  object_image_relocation_lowering_issue: {issue}");
    }
    for issue in &report.object_emit_blocked_issues {
        println!("  object_emit_blocked_issue: {issue}");
    }
    for issue in &report.object_output_issues {
        println!("  object_output_issue: {issue}");
    }
    for issue in &report.object_writer_dry_run_issues {
        println!("  object_writer_dry_run_issue: {issue}");
    }
    for issue in &report.container_plan_issues {
        println!("  container_plan_issue: {issue}");
    }
    for issue in &report.container_issues {
        println!("  container_issue: {issue}");
    }
    for issue in &report.container_section_issues {
        println!("  container_section_issue: {issue}");
    }
    for issue in &report.container_loader_symbol_issues {
        println!("  container_loader_symbol_issue: {issue}");
    }
    for issue in &report.container_relocation_issues {
        println!("  container_relocation_issue: {issue}");
    }
    for issue in &report.container_compatibility_domain_issues {
        println!("  container_compatibility_domain_issue: {issue}");
    }
    for issue in &report.container_external_import_issues {
        println!("  container_external_import_issue: {issue}");
    }
    for issue in &report.container_payload_issues {
        println!("  container_payload_issue: {issue}");
    }
    for issue in &report.closure_snapshot_issues {
        println!("  closure_snapshot_issue: {issue}");
    }
    for issue in &report.final_stage_plan_issues {
        println!("  final_stage_plan_issue: {issue}");
    }
    for issue in &report.final_executable_writer_input_issues {
        println!("  final_executable_writer_input_issue: {issue}");
    }
    for issue in &report.final_executable_host_invoke_plan_issues {
        println!("  final_executable_host_invoke_plan_issue: {issue}");
    }
    for issue in &report.final_executable_layout_plan_issues {
        println!("  final_executable_layout_plan_issue: {issue}");
    }
    for issue in &report.final_executable_image_dry_run_issues {
        println!("  final_executable_image_dry_run_issue: {issue}");
    }
    for issue in &report.final_executable_blocked_issues {
        println!("  final_executable_blocked_issue: {issue}");
    }
    for blocker in &report.final_executable_output_blockers {
        println!("  final_executable_output_blocker: {blocker}");
    }
    for issue in &report.final_executable_output_issues {
        println!("  final_executable_output_issue: {issue}");
    }
    for issue in &report.final_executable_launcher_manifest_issues {
        println!("  final_executable_launcher_manifest_issue: {issue}");
    }
    for issue in &report.final_executable_launcher_dry_run_issues {
        println!("  final_executable_launcher_dry_run_issue: {issue}");
    }
    for issue in &report.final_executable_pipeline_issues {
        println!("  final_executable_pipeline_issue: {issue}");
    }
    for path in &report.final_executable_pipeline_missing_required_stage_paths {
        println!("  final_executable_pipeline_missing_required_stage_path: {path}");
    }
    for blocker in &report.container_loader_blockers {
        println!("  container_loader_blocker: {blocker}");
    }
    for advisory in &report.artifact_chain_advisories {
        println!("  artifact_chain_advisory: {advisory}");
    }
    if let Some(reason) = report.artifact_chain_advisory_command_reason.as_deref() {
        println!("  artifact_chain_advisory_command_reason: {reason}");
    }
    if let Some(reason) = report.artifact_chain_next_action_command_reason.as_deref() {
        println!("  artifact_chain_next_action_command_reason: {reason}");
    }
    if let Some(reason) = report.next_action_command_reason.as_deref() {
        println!("  next_action_command_reason: {reason}");
    }
    for issue in &report.artifact_chain_issues {
        println!("  artifact_chain_issue: {issue}");
    }
}
