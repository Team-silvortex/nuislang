pub(crate) use super::display_check::print_check_report;
pub(crate) use super::display_container::*;
pub(crate) use super::display_link_tables::*;
pub(crate) use super::display_object::*;
pub(crate) use super::display_object_emit::*;
pub(crate) use super::display_object_image::*;

use super::reports::*;

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
        "  container_metadata_table_hash: {}",
        report.container_metadata_table_hash
    );
    println!(
        "  container_loader_readiness: {}",
        report.container_loader_readiness
    );
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
    println!("  container_plan: {}", report.container_plan_path);
    println!("  container: {}", report.container_path);
    println!("  container_payload: {}", report.container_payload_path);
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
    println!("  metadata_table_hash: {}", report.metadata_table_hash);
    println!("  container_layout_hash: {}", report.container_layout_hash);
    println!("  container_hash: {}", report.container_hash);
    println!("  payload_size_bytes: {}", report.payload_size_bytes);
    println!("  payload_hash: {}", report.payload_hash);
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
