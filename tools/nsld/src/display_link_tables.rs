use super::{
    display::optional_usize_text,
    reports::{
        NsldLinkInputsEmitReport, NsldLinkInputsVerifyReport, NsldLinkUnitsEmitReport,
        NsldLinkUnitsVerifyReport,
    },
};

pub(crate) fn print_nsld_link_units_emit_report(report: &NsldLinkUnitsEmitReport) {
    println!("Nsld link units emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  unit_count: {}", report.unit_count);
    println!("  hetero_unit_count: {}", report.hetero_unit_count);
    println!("  link_input_count: {}", report.link_input_count);
    println!("  unit_table_hash: {}", report.unit_table_hash);
}

pub(crate) fn print_nsld_link_units_verify_report(report: &NsldLinkUnitsVerifyReport) {
    println!("Nsld link units verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!("  expected_unit_count: {}", report.expected_unit_count);
    println!(
        "  expected_hetero_unit_count: {}",
        report.expected_hetero_unit_count
    );
    println!(
        "  expected_link_input_count: {}",
        report.expected_link_input_count
    );
    println!(
        "  expected_unit_table_hash: {}",
        report.expected_unit_table_hash
    );
    println!(
        "  actual_unit_count: {}",
        optional_usize_text(report.actual_unit_count)
    );
    println!(
        "  actual_hetero_unit_count: {}",
        optional_usize_text(report.actual_hetero_unit_count)
    );
    println!(
        "  actual_link_input_count: {}",
        optional_usize_text(report.actual_link_input_count)
    );
    println!(
        "  actual_unit_table_hash: {}",
        report
            .actual_unit_table_hash
            .as_deref()
            .unwrap_or("missing")
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}

pub(crate) fn print_nsld_link_inputs_emit_report(report: &NsldLinkInputsEmitReport) {
    println!("Nsld link inputs");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  link_input_count: {}", report.link_input_count);
    println!(
        "  link_input_total_bytes: {}",
        report.link_input_total_bytes
    );
    println!("  link_input_table_hash: {}", report.link_input_table_hash);
}

pub(crate) fn print_nsld_link_inputs_verify_report(report: &NsldLinkInputsVerifyReport) {
    println!("Nsld link inputs verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_link_input_count: {}",
        report.expected_link_input_count
    );
    println!(
        "  expected_link_input_total_bytes: {}",
        report.expected_link_input_total_bytes
    );
    println!(
        "  expected_link_input_table_hash: {}",
        report.expected_link_input_table_hash
    );
    println!(
        "  actual_link_input_count: {}",
        optional_usize_text(report.actual_link_input_count)
    );
    println!(
        "  actual_link_input_total_bytes: {}",
        optional_usize_text(report.actual_link_input_total_bytes)
    );
    println!(
        "  actual_link_input_table_hash: {}",
        report
            .actual_link_input_table_hash
            .as_deref()
            .unwrap_or("missing")
    );
    for issue in &report.issues {
        println!("  issue: {issue}");
    }
}
