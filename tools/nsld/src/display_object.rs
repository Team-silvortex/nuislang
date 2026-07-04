use super::display::optional_usize_text;
use super::reports::*;

pub(crate) fn print_nsld_object_plan_report(report: &NsldObjectPlanReport) {
    println!("Nsld object plan");
    println!("  manifest: {}", report.manifest);
    println!("  ready: {}", report.ready);
    println!("  target_arch: {}", report.target_arch);
    println!("  target_os: {}", report.target_os);
    println!("  object_format: {}", report.object_format);
    println!("  calling_abi: {}", report.calling_abi);
    println!("  clang_target: {}", report.clang_target);
    println!("  output_path: {}", report.output_path);
    println!("  source_container_path: {}", report.source_container_path);
    println!("  source_payload_path: {}", report.source_payload_path);
    println!("  section_count: {}", report.section_count);
    println!("  section_table_hash: {}", report.section_table_hash);
    println!("  object_plan_hash: {}", report.object_plan_hash);
    println!("  object_layout_hash: {}", report.object_layout_hash);
    println!("  relocation_seed_count: {}", report.relocation_seed_count);
    println!(
        "  relocation_seed_table_hash: {}",
        report.relocation_seed_table_hash
    );
    println!("  writer_target_id: {}", report.writer_target_id);
    println!("  writer_status: {}", report.writer_status);
    if !report.unsupported_features.is_empty() {
        println!(
            "  unsupported_features: {}",
            report.unsupported_features.join(", ")
        );
    }
    println!("  emission_status: {}", report.emission_status);
    for section in &report.object_sections {
        println!(
            "  object_section: index={} source={} kind={} object={} role={} file_offset_seed={} file_size_seed={} align={} hash={}",
            section.order_index,
            section.source_section_id,
            section.source_section_kind,
            section.object_section_name,
            section.object_section_role,
            section.file_offset_seed,
            section.file_size_seed,
            section.alignment,
            section.source_hash
        );
    }
    for seed in &report.relocation_seeds {
        println!(
            "  relocation_seed: index={} id={} kind={} source={} offset_seed={} target={} ready={}",
            seed.order_index,
            seed.relocation_seed_id,
            seed.relocation_seed_kind,
            seed.source_section_id,
            seed.source_offset_seed,
            seed.target_symbol,
            seed.native_relocation_ready
        );
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
}

pub(crate) fn print_nsld_object_plan_emit_report(report: &NsldObjectPlanEmitReport) {
    println!("Nsld object plan emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  ready: {}", report.ready);
    println!("  object_plan_hash: {}", report.object_plan_hash);
    println!("  section_count: {}", report.section_count);
}

pub(crate) fn print_nsld_object_plan_verify_report(report: &NsldObjectPlanVerifyReport) {
    println!("Nsld object plan verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_object_plan_hash: {}",
        report.expected_object_plan_hash
    );
    println!(
        "  expected_section_count: {}",
        report.expected_section_count
    );
    println!(
        "  actual_object_plan_hash: {}",
        report
            .actual_object_plan_hash
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

pub(crate) fn print_nsld_object_writer_readiness_report(report: &NsldObjectWriterReadinessReport) {
    println!("Nsld object writer readiness");
    println!("  manifest: {}", report.manifest);
    println!("  writer_target_id: {}", report.writer_target_id);
    println!("  writer_status: {}", report.writer_status);
    println!("  object_plan_hash: {}", report.object_plan_hash);
    println!("  section_count: {}", report.section_count);
    println!("  can_emit_object: {}", report.can_emit_object);
    for feature in &report.unsupported_features {
        println!("  unsupported_feature: {feature}");
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
}

pub(crate) fn print_nsld_object_writer_input_verify_report(
    report: &NsldObjectWriterInputVerifyReport,
) {
    println!("Nsld object writer input verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_object_plan_hash: {}",
        report.expected_object_plan_hash
    );
    println!(
        "  expected_object_layout_hash: {}",
        report.expected_object_layout_hash
    );
    println!(
        "  expected_relocation_seed_table_hash: {}",
        report.expected_relocation_seed_table_hash
    );
    println!(
        "  actual_object_plan_hash: {}",
        report
            .actual_object_plan_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_object_layout_hash: {}",
        report
            .actual_object_layout_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_relocation_seed_table_hash: {}",
        report
            .actual_relocation_seed_table_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_section_count: {}",
        optional_usize_text(report.actual_section_count)
    );
    println!(
        "  actual_relocation_seed_count: {}",
        optional_usize_text(report.actual_relocation_seed_count)
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_object_writer_dry_run_report(report: &NsldObjectWriterDryRunReport) {
    println!("Nsld object writer dry run");
    println!("  manifest: {}", report.manifest);
    println!("  writer_input_path: {}", report.writer_input_path);
    println!("  planned_output_path: {}", report.planned_output_path);
    println!("  writer_target_id: {}", report.writer_target_id);
    println!("  object_plan_hash: {}", report.object_plan_hash);
    println!("  object_layout_hash: {}", report.object_layout_hash);
    println!(
        "  relocation_seed_table_hash: {}",
        report.relocation_seed_table_hash
    );
    println!("  section_count: {}", report.section_count);
    println!("  relocation_seed_count: {}", report.relocation_seed_count);
    println!("  writer_input_valid: {}", report.writer_input_valid);
    println!("  can_emit_object: {}", report.can_emit_object);
    println!("  dry_run_ready: {}", report.dry_run_ready);
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
}

pub(crate) fn print_nsld_object_writer_dry_run_emit_report(
    report: &NsldObjectWriterDryRunEmitReport,
) {
    println!("Nsld object writer dry run emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  dry_run_ready: {}", report.dry_run_ready);
    println!("  object_plan_hash: {}", report.object_plan_hash);
    println!("  section_count: {}", report.section_count);
    println!("  relocation_seed_count: {}", report.relocation_seed_count);
}

pub(crate) fn print_nsld_object_writer_dry_run_verify_report(
    report: &NsldObjectWriterDryRunVerifyReport,
) {
    println!("Nsld object writer dry run verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_object_plan_hash: {}",
        report.expected_object_plan_hash
    );
    println!(
        "  expected_object_layout_hash: {}",
        report.expected_object_layout_hash
    );
    println!(
        "  expected_relocation_seed_table_hash: {}",
        report.expected_relocation_seed_table_hash
    );
    println!(
        "  actual_object_plan_hash: {}",
        report
            .actual_object_plan_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_object_layout_hash: {}",
        report
            .actual_object_layout_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_relocation_seed_table_hash: {}",
        report
            .actual_relocation_seed_table_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_section_count: {}",
        optional_usize_text(report.actual_section_count)
    );
    println!(
        "  actual_relocation_seed_count: {}",
        optional_usize_text(report.actual_relocation_seed_count)
    );
    println!(
        "  actual_dry_run_ready: {}",
        report
            .actual_dry_run_ready
            .map(|value| value.to_string())
            .unwrap_or_else(|| "missing".to_owned())
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_object_byte_layout_report(report: &NsldObjectByteLayoutReport) {
    println!("Nsld object byte layout");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  object_plan_hash: {}", report.object_plan_hash);
    println!("  object_layout_hash: {}", report.object_layout_hash);
    println!("  byte_layout_hash: {}", report.byte_layout_hash);
    println!("  section_count: {}", report.section_count);
    println!("  total_size_bytes: {}", report.total_size_bytes);
    println!("  layout_ready: {}", report.layout_ready);
    for section in &report.sections {
        println!(
            "  byte_section: index={} source={} object={} offset={} size={} align={} hash={}",
            section.order_index,
            section.source_section_id,
            section.object_section_name,
            section.file_offset,
            section.size_bytes,
            section.alignment,
            section.source_hash
        );
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
}

pub(crate) fn print_nsld_object_byte_layout_emit_report(report: &NsldObjectByteLayoutEmitReport) {
    println!("Nsld object byte layout emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  layout_ready: {}", report.layout_ready);
    println!("  byte_layout_hash: {}", report.byte_layout_hash);
    println!("  section_count: {}", report.section_count);
    println!("  total_size_bytes: {}", report.total_size_bytes);
}

pub(crate) fn print_nsld_object_byte_layout_verify_report(
    report: &NsldObjectByteLayoutVerifyReport,
) {
    println!("Nsld object byte layout verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_byte_layout_hash: {}",
        report.expected_byte_layout_hash
    );
    println!(
        "  actual_byte_layout_hash: {}",
        report
            .actual_byte_layout_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_section_count: {}",
        optional_usize_text(report.actual_section_count)
    );
    println!(
        "  actual_total_size_bytes: {}",
        optional_usize_text(report.actual_total_size_bytes)
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_object_file_layout_report(report: &NsldObjectFileLayoutReport) {
    println!("Nsld object file layout");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  writer_target_id: {}", report.writer_target_id);
    println!("  backend_kind: {}", report.backend_kind);
    println!("  object_format: {}", report.object_format);
    println!("  object_plan_hash: {}", report.object_plan_hash);
    println!("  byte_layout_hash: {}", report.byte_layout_hash);
    println!("  file_layout_hash: {}", report.file_layout_hash);
    println!("  record_count: {}", report.record_count);
    println!("  total_file_size_bytes: {}", report.total_file_size_bytes);
    println!("  layout_ready: {}", report.layout_ready);
    for record in &report.records {
        println!(
            "  file_layout_record: index={} id={} kind={} offset={} size={} align={}",
            record.order_index,
            record.record_id,
            record.record_kind,
            record.file_offset,
            record.size_bytes,
            record.alignment
        );
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
}

pub(crate) fn print_nsld_object_file_layout_emit_report(report: &NsldObjectFileLayoutEmitReport) {
    println!("Nsld object file layout emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  layout_ready: {}", report.layout_ready);
    println!("  file_layout_hash: {}", report.file_layout_hash);
    println!("  record_count: {}", report.record_count);
    println!("  total_file_size_bytes: {}", report.total_file_size_bytes);
}

pub(crate) fn print_nsld_object_file_layout_verify_report(
    report: &NsldObjectFileLayoutVerifyReport,
) {
    println!("Nsld object file layout verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_file_layout_hash: {}",
        report.expected_file_layout_hash
    );
    println!(
        "  actual_file_layout_hash: {}",
        report
            .actual_file_layout_hash
            .as_deref()
            .unwrap_or("missing")
    );
    println!(
        "  actual_record_count: {}",
        optional_usize_text(report.actual_record_count)
    );
    println!(
        "  actual_total_file_size_bytes: {}",
        optional_usize_text(report.actual_total_file_size_bytes)
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}
