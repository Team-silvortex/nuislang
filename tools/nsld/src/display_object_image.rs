use super::{display::optional_usize_text, reports::*};

pub(crate) fn print_nsld_object_image_dry_run_report(report: &NsldObjectImageDryRunReport) {
    println!("Nsld object image dry run");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  image_path: {}", report.image_path);
    println!("  writer_target_id: {}", report.writer_target_id);
    println!("  writer_backend_kind: {}", report.writer_backend_kind);
    println!("  object_family: {}", report.object_family);
    println!("  backend_kind: {}", report.backend_kind);
    println!("  backend_family: {}", report.backend_family);
    println!("  backend_status: {}", report.backend_status);
    for capability in &report.backend_capabilities {
        println!(
            "  backend_capability: id={} status={} required={}",
            capability.capability_id, capability.status, capability.required
        );
    }
    println!("  object_format: {}", report.object_format);
    println!("  file_layout_hash: {}", report.file_layout_hash);
    println!("  record_count: {}", report.record_count);
    println!("  total_file_size_bytes: {}", report.total_file_size_bytes);
    println!("  image_constructed: {}", report.image_constructed);
    println!("  image_ready: {}", report.image_ready);
    println!(
        "  image_size_bytes: {}",
        optional_usize_text(report.image_size_bytes)
    );
    println!(
        "  image_hash: {}",
        report.image_hash.as_deref().unwrap_or("missing")
    );
    println!(
        "  relocation_lowering_valid: {}",
        report.relocation_lowering_valid
    );
    println!(
        "  relocation_lowering_rule_count: {}",
        report.relocation_lowering_rule_count
    );
    for issue in &report.relocation_lowering_issues {
        println!("  relocation_lowering_issue: {issue}");
    }
    for rule in &report.relocation_lowering_rules {
        println!(
            "  relocation_lowering_rule: id={} source_seed_kind={} target={} pc_relative={} length_power={} external={} relocation_type={}",
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
        "  relocation_record_count: {}",
        report.relocation_record_count
    );
    println!(
        "  relocation_record_table_hash: {}",
        report.relocation_record_table_hash
    );
    for record in &report.relocation_records {
        println!(
            "  relocation_record: id={} relocation_seed_id={} source_section_id={} source_offset={} source_seed_kind={} target={} symbol_index={} pc_relative={} length_power={} external={} relocation_type={}",
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
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
}

pub(crate) fn print_nsld_object_image_dry_run_emit_report(
    report: &NsldObjectImageDryRunEmitReport,
) {
    println!("Nsld object image dry run emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  image_path: {}", report.image_path);
    println!("  image_emitted: {}", report.image_emitted);
    println!("  image_constructed: {}", report.image_constructed);
    println!("  image_ready: {}", report.image_ready);
    println!(
        "  image_size_bytes: {}",
        optional_usize_text(report.image_size_bytes)
    );
    println!(
        "  image_hash: {}",
        report.image_hash.as_deref().unwrap_or("missing")
    );
}

pub(crate) fn print_nsld_object_image_dry_run_verify_report(
    report: &NsldObjectImageDryRunVerifyReport,
) {
    println!("Nsld object image dry run verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  image_path: {}", report.image_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_writer_backend_kind: {}",
        report.expected_writer_backend_kind
    );
    println!(
        "  expected_object_family: {}",
        report.expected_object_family
    );
    println!(
        "  expected_file_layout_hash: {}",
        report.expected_file_layout_hash
    );
    println!(
        "  expected_backend_family: {}",
        report.expected_backend_family
    );
    println!(
        "  expected_backend_status: {}",
        report.expected_backend_status
    );
    println!(
        "  actual_file_layout_hash: {}",
        report
            .actual_file_layout_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_writer_backend_kind: {}",
        report
            .actual_writer_backend_kind
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_object_family: {}",
        report.actual_object_family.as_deref().unwrap_or("missing")
    );
    println!(
        "  actual_backend_family: {}",
        report.actual_backend_family.as_deref().unwrap_or("missing")
    );
    println!(
        "  actual_backend_status: {}",
        report.actual_backend_status.as_deref().unwrap_or("missing")
    );
    println!("  expected_image_ready: {}", report.expected_image_ready);
    println!(
        "  actual_image_ready: {}",
        report
            .actual_image_ready
            .map(|value| value.to_string())
            .unwrap_or_else(|| "missing".to_owned())
    );
    println!(
        "  expected_relocation_lowering_valid: {}",
        report.expected_relocation_lowering_valid
    );
    println!(
        "  actual_relocation_lowering_valid: {}",
        report
            .actual_relocation_lowering_valid
            .map(|value| value.to_string())
            .unwrap_or_else(|| "missing".to_owned())
    );
    println!(
        "  expected_relocation_lowering_rule_count: {}",
        report.expected_relocation_lowering_rule_count
    );
    println!(
        "  actual_relocation_lowering_rule_count: {}",
        report
            .actual_relocation_lowering_rule_count
            .map(|value| value.to_string())
            .unwrap_or_else(|| "missing".to_owned())
    );
    for issue in &report.expected_relocation_lowering_issues {
        println!("  expected_relocation_lowering_issue: {issue}");
    }
    for issue in report
        .actual_relocation_lowering_issues
        .as_deref()
        .unwrap_or(&[])
    {
        println!("  actual_relocation_lowering_issue: {issue}");
    }
    for rule in &report.expected_relocation_lowering_rules {
        println!(
            "  expected_relocation_lowering_rule: id={} source_seed_kind={} target={} pc_relative={} length_power={} external={} relocation_type={}",
            rule.rule_id,
            rule.source_seed_kind,
            rule.target_relocation_kind,
            rule.pc_relative,
            rule.length_power,
            rule.external,
            rule.relocation_type
        );
    }
    for rule in report
        .actual_relocation_lowering_rules
        .as_deref()
        .unwrap_or(&[])
    {
        println!(
            "  actual_relocation_lowering_rule: id={} source_seed_kind={} target={} pc_relative={} length_power={} external={} relocation_type={}",
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
        "  expected_relocation_record_count: {}",
        report.expected_relocation_record_count
    );
    println!(
        "  expected_relocation_record_table_hash: {}",
        report.expected_relocation_record_table_hash
    );
    println!(
        "  actual_relocation_record_count: {}",
        optional_usize_text(report.actual_relocation_record_count)
    );
    println!(
        "  actual_relocation_record_table_hash: {}",
        report
            .actual_relocation_record_table_hash
            .as_deref()
            .unwrap_or("missing")
    );
    for record in &report.expected_relocation_records {
        println!(
            "  expected_relocation_record: id={} relocation_seed_id={} source_section_id={} source_offset={} source_seed_kind={} target={} symbol_index={} pc_relative={} length_power={} external={} relocation_type={}",
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
    for record in report.actual_relocation_records.as_deref().unwrap_or(&[]) {
        println!(
            "  actual_relocation_record: id={} relocation_seed_id={} source_section_id={} source_offset={} source_seed_kind={} target={} symbol_index={} pc_relative={} length_power={} external={} relocation_type={}",
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
        "  actual_image_file_size_bytes: {}",
        optional_usize_text(report.actual_image_file_size_bytes)
    );
    println!(
        "  actual_image_file_hash: {}",
        report
            .actual_image_file_hash
            .as_deref()
            .unwrap_or("missing")
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}
