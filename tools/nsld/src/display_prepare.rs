use super::{display_text::*, reports::*};

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
