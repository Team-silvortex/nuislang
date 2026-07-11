use super::{display_text::*, reports::NsldFinalExecutableOutputReport};

pub(crate) fn print_nsld_final_executable_output_report(report: &NsldFinalExecutableOutputReport) {
    println!("Nsld final executable output");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  present: {}", report.present);
    println!("  size_bytes: {}", optional_usize_text(report.size_bytes));
    println!(
        "  output_hash: {}",
        optional_string_text(report.output_hash.as_deref())
    );
    println!(
        "  output_image_header_valid: {}",
        report.output_image_header_valid
    );
    println!(
        "  output_image_magic: {}",
        optional_string_text(report.output_image_magic.as_deref())
    );
    println!(
        "  output_image_version: {}",
        optional_usize_text(report.output_image_version)
    );
    println!(
        "  output_image_header_size: {}",
        optional_usize_text(report.output_image_header_size)
    );
    println!(
        "  output_payload_byte_offset: {}",
        optional_usize_text(report.output_payload_byte_offset)
    );
    println!(
        "  output_payload_byte_span: {}",
        optional_usize_text(report.output_payload_byte_span)
    );
    println!(
        "  output_layout_hash: {}",
        optional_string_text(report.output_layout_hash.as_deref())
    );
    println!(
        "  output_byte_map_hash: {}",
        optional_string_text(report.output_byte_map_hash.as_deref())
    );
    println!(
        "  expected_image_size_bytes: {}",
        optional_usize_text(report.expected_image_size_bytes)
    );
    println!(
        "  expected_image_hash: {}",
        optional_string_text(report.expected_image_hash.as_deref())
    );
    println!(
        "  matches_expected_image: {}",
        report.matches_expected_image
    );
    println!(
        "  final_stage_plan_valid: {}",
        report.final_stage_plan_valid
    );
    println!(
        "  final_stage_plan_hash: {}",
        optional_string_text(report.final_stage_plan_hash.as_deref())
    );
    println!(
        "  final_executable_emit_valid: {}",
        report.final_executable_emit_valid
    );
    println!(
        "  final_executable_emitted: {}",
        optional_bool_text(report.final_executable_emitted)
    );
    println!(
        "  final_executable_blocker_count: {}",
        optional_usize_text(report.final_executable_blocker_count)
    );
    println!("  runnable_candidate: {}", report.runnable_candidate);
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}
