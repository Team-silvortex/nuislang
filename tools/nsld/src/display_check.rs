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
        "  artifact_chain: valid={} issues={}",
        report.artifact_chain_valid,
        report.artifact_chain_issues.len()
    );
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
    for blocker in &report.container_loader_blockers {
        println!("  container_loader_blocker: {blocker}");
    }
    for issue in &report.artifact_chain_issues {
        println!("  artifact_chain_issue: {issue}");
    }
}
