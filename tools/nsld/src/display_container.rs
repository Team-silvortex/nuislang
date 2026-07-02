pub(crate) use super::display_container_verify::print_nsld_container_verify_report;

use super::{
    container::{
        NsldContainerEmitReport, NsldContainerPlanEmitReport, NsldContainerPlanReport,
        NsldContainerPlanVerifyReport, NsldContainerReport,
    },
    display::{optional_usize_text, print_assemble_sections},
};

pub(crate) fn print_nsld_container_plan_report(report: &NsldContainerPlanReport) {
    println!("Nsld container plan");
    println!("  manifest: {}", report.manifest);
    println!("  ready: {}", report.ready);
    println!("  container_magic: {}", report.container_magic);
    println!("  container_version: {}", report.container_version);
    println!("  section_count: {}", report.section_count);
    println!("  section_table_hash: {}", report.section_table_hash);
    println!("  container_layout_hash: {}", report.container_layout_hash);
    println!("  output_path: {}", report.output_path);
    print_assemble_sections(&report.sections);
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
}

pub(crate) fn print_nsld_container_plan_emit_report(report: &NsldContainerPlanEmitReport) {
    println!("Nsld container plan emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  ready: {}", report.ready);
    println!("  section_count: {}", report.section_count);
    println!("  container_layout_hash: {}", report.container_layout_hash);
}

pub(crate) fn print_nsld_container_plan_verify_report(report: &NsldContainerPlanVerifyReport) {
    println!("Nsld container plan verify");
    println!("  manifest: {}", report.manifest);
    println!("  input_path: {}", report.input_path);
    println!("  valid: {}", report.valid);
    println!(
        "  expected_container_layout_hash: {}",
        report.expected_container_layout_hash
    );
    println!(
        "  expected_section_count: {}",
        report.expected_section_count
    );
    println!(
        "  actual_container_layout_hash: {}",
        report
            .actual_container_layout_hash
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

pub(crate) fn print_nsld_container_report(report: &NsldContainerReport) {
    println!("Nsld container");
    println!("  manifest: {}", report.manifest);
    println!("  ready: {}", report.ready);
    println!("  container_magic: {}", report.container_magic);
    println!("  container_version: {}", report.container_version);
    println!("  metadata_table_hash: {}", report.metadata_table_hash);
    println!("  container_layout_hash: {}", report.container_layout_hash);
    println!("  container_hash: {}", report.container_hash);
    println!("  loader_readiness: {}", report.loader_readiness);
    for blocker in &report.loader_blockers {
        println!("  loader_blocker: {blocker}");
    }
    println!("  loader_entry_kind: {}", report.loader_entry_kind);
    println!("  loader_entry_symbol: {}", report.loader_entry_symbol);
    println!(
        "  loader_entry_section_id: {}",
        report.loader_entry_section_id
    );
    println!("  loader_symbols: {}", report.loader_symbols.len());
    println!(
        "  loader_symbol_table_hash: {}",
        report.loader_symbol_table_hash
    );
    for symbol in &report.loader_symbols {
        println!(
            "  loader_symbol: id={} kind={} name={} section={} offset={} size={} payload_hash={}",
            symbol.symbol_id,
            symbol.symbol_kind,
            symbol.symbol_name,
            symbol.section_id,
            symbol.offset,
            symbol.size_bytes,
            symbol.payload_hash
        );
    }
    println!("  relocations: {}", report.relocations.len());
    println!("  relocation_table_hash: {}", report.relocation_table_hash);
    for relocation in &report.relocations {
        println!(
            "  relocation: id={} kind={} source={} offset={} target={} addend={}",
            relocation.relocation_id,
            relocation.relocation_kind,
            relocation.source_section_id,
            relocation.source_offset,
            relocation.target_symbol_id,
            relocation.addend
        );
    }
    println!("  external_imports: {}", report.external_imports.len());
    println!(
        "  external_import_table_hash: {}",
        report.external_import_table_hash
    );
    for external_import in &report.external_imports {
        println!(
            "  external_import: id={} kind={} name={} provider={} required={}",
            external_import.import_id,
            external_import.import_kind,
            external_import.import_name,
            external_import.provider,
            external_import.required
        );
    }
    println!("  payload_size_bytes: {}", report.payload_size_bytes);
    println!("  payload_hash: {}", report.payload_hash);
    println!("  output_path: {}", report.output_path);
    println!("  payload_path: {}", report.payload_path);
    println!("  section_count: {}", report.section_count);
    println!(
        "  container_section_table_hash: {}",
        report.container_section_table_hash
    );
    for section in &report.sections {
        println!(
            "  section: order={} id={} kind={} required={} offset={} size={} payload_hash={} hash={} source={}",
            section.order_index,
            section.section_id,
            section.section_kind,
            section.required,
            section.offset,
            section.size_bytes,
            section.payload_hash,
            section.source_hash,
            section.source_path
        );
    }
    for blocker in &report.blockers {
        println!("  blocker: {blocker}");
    }
}

pub(crate) fn print_nsld_container_emit_report(report: &NsldContainerEmitReport) {
    println!("Nsld container emit");
    println!("  manifest: {}", report.manifest);
    println!("  output_path: {}", report.output_path);
    println!("  payload_path: {}", report.payload_path);
    println!("  ready: {}", report.ready);
    println!("  section_count: {}", report.section_count);
    println!("  metadata_table_hash: {}", report.metadata_table_hash);
    println!("  container_layout_hash: {}", report.container_layout_hash);
    println!("  container_hash: {}", report.container_hash);
    println!("  payload_size_bytes: {}", report.payload_size_bytes);
    println!("  payload_hash: {}", report.payload_hash);
}
