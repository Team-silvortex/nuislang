use super::{RunnerReport, RUNNER_PROTOCOL};

pub(super) fn print_text_report(report: &RunnerReport) {
    println!("nuis-host-runner: {}", RUNNER_PROTOCOL);
    println!("  ready: {}", report.ready);
    println!(
        "  would_enter_lifecycle_hook: {}",
        report.would_enter_lifecycle_hook
    );
    println!("  manifest_path: {}", report.manifest_path);
    println!(
        "  nsb_path: {}",
        report.nsb_path.as_deref().unwrap_or("<none>")
    );
    println!("  nsb_readable: {}", report.nsb_readable);
    println!("  nsb_hash_matches: {}", report.nsb_hash_matches);
    println!(
        "  nsb_payload_offset: {}",
        optional_usize_text(report.nsb_payload_offset)
    );
    println!(
        "  nsb_payload_span: {}",
        optional_usize_text(report.nsb_payload_span)
    );
    println!(
        "  nsb_payload_region_mapped: {}",
        report.nsb_payload_region_mapped
    );
    println!(
        "  nsb_payload_region_bytes: {}",
        optional_usize_text(report.nsb_payload_region_bytes)
    );
    println!(
        "  nsb_payload_region_hash: {}",
        report
            .nsb_payload_region_hash
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  nsb_payload_scan_status: {}",
        report.nsb_payload_scan_status
    );
    println!("  nsb_payload_scan_kind: {}", report.nsb_payload_scan_kind);
    println!(
        "  nsb_payload_prefix_hex: {}",
        report.nsb_payload_prefix_hex.as_deref().unwrap_or("<none>")
    );
    println!(
        "  nsb_payload_prefix_text: {}",
        report
            .nsb_payload_prefix_text
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_status: {}",
        report.container_loader_status
    );
    println!(
        "  container_schema: {}",
        report.container_schema.as_deref().unwrap_or("<none>")
    );
    println!(
        "  container_schema_version: {}",
        optional_usize_text(report.container_schema_version)
    );
    println!(
        "  container_kind: {}",
        report.container_kind.as_deref().unwrap_or("<none>")
    );
    println!(
        "  container_producer: {}",
        report.container_producer.as_deref().unwrap_or("<none>")
    );
    println!(
        "  container_producer_phase: {}",
        report
            .container_producer_phase
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_ready: {}",
        report
            .container_ready
            .map(|value| value.to_string())
            .unwrap_or_else(|| "<none>".to_owned())
    );
    println!(
        "  container_blockers: {}",
        joined_or_none(&report.container_blockers)
    );
    println!(
        "  container_magic: {}",
        report.container_magic.as_deref().unwrap_or("<none>")
    );
    println!(
        "  container_version: {}",
        optional_usize_text(report.container_version)
    );
    println!(
        "  container_metadata_table_hash: {}",
        report
            .container_metadata_table_hash
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_section_table_hash: {}",
        report
            .container_section_table_hash
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_hash: {}",
        report.container_hash.as_deref().unwrap_or("<none>")
    );
    println!(
        "  container_section_count: {}",
        optional_usize_text(report.container_section_count)
    );
    println!(
        "  container_section_parsed_count: {}",
        report.container_section_parsed_count
    );
    println!(
        "  container_first_section_id: {}",
        report
            .container_first_section_id
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_first_section_kind: {}",
        report
            .container_first_section_kind
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_entry_section_found: {}",
        report.container_entry_section_found
    );
    println!(
        "  container_payload_size_bytes: {}",
        optional_usize_text(report.container_payload_size_bytes)
    );
    println!(
        "  container_payload_hash: {}",
        report.container_payload_hash.as_deref().unwrap_or("<none>")
    );
    println!(
        "  container_payload_path: {}",
        report.container_payload_path.as_deref().unwrap_or("<none>")
    );
    println!(
        "  container_loader_readiness: {}",
        report
            .container_loader_readiness
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_blockers: {}",
        joined_or_none(&report.container_loader_blockers)
    );
    println!(
        "  container_loader_entry_kind: {}",
        report
            .container_loader_entry_kind
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_entry_symbol: {}",
        report
            .container_loader_entry_symbol
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_entry_section_id: {}",
        report
            .container_loader_entry_section_id
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_symbol_count: {}",
        optional_usize_text(report.container_loader_symbol_count)
    );
    println!(
        "  loader_symbol_table_hash: {}",
        report
            .loader_symbol_table_hash
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_symbol_status: {}",
        report.container_loader_symbol_status
    );
    println!(
        "  container_loader_symbol_id: {}",
        report
            .container_loader_symbol_id
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_symbol_kind: {}",
        report
            .container_loader_symbol_kind
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_symbol_name: {}",
        report
            .container_loader_symbol_name
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_symbol_lifecycle_hook: {}",
        report
            .container_loader_symbol_lifecycle_hook
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_loader_symbol_section_id: {}",
        report
            .container_loader_symbol_section_id
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_relocation_count: {}",
        optional_usize_text(report.container_relocation_count)
    );
    println!(
        "  relocation_table_hash: {}",
        report.relocation_table_hash.as_deref().unwrap_or("<none>")
    );
    println!(
        "  container_relocation_parsed_count: {}",
        report.container_relocation_parsed_count
    );
    println!(
        "  container_first_relocation_kind: {}",
        report
            .container_first_relocation_kind
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_first_relocation_source_section_id: {}",
        report
            .container_first_relocation_source_section_id
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_first_relocation_target_symbol_id: {}",
        report
            .container_first_relocation_target_symbol_id
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  container_first_relocation_targets_loader_symbol: {}",
        report.container_first_relocation_targets_loader_symbol
    );
    println!(
        "  container_first_relocation_source_matches_loader_symbol: {}",
        report.container_first_relocation_source_matches_loader_symbol
    );
    println!(
        "  compatibility_domain_count: {}",
        optional_usize_text(report.compatibility_domain_count)
    );
    println!(
        "  compatibility_domain_table_hash: {}",
        report
            .compatibility_domain_table_hash
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  compatibility_domain_parsed_count: {}",
        report.compatibility_domain_parsed_count
    );
    println!(
        "  compatibility_domain_first_kind: {}",
        report
            .compatibility_domain_first_kind
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  compatibility_domain_required_count: {}",
        report.compatibility_domain_required_count
    );
    println!(
        "  external_import_count: {}",
        optional_usize_text(report.external_import_count)
    );
    println!(
        "  external_import_table_hash: {}",
        report
            .external_import_table_hash
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  external_import_parsed_count: {}",
        report.external_import_parsed_count
    );
    println!(
        "  external_import_first_kind: {}",
        report
            .external_import_first_kind
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  external_import_first_name: {}",
        report
            .external_import_first_name
            .as_deref()
            .unwrap_or("<none>")
    );
    println!(
        "  external_import_required_imports: {}",
        joined_or_none(&report.external_import_required_imports)
    );
    println!(
        "  container_loader_handoff_status: {}",
        report.container_loader_handoff_status
    );
    println!(
        "  container_loader_handoff_ready: {}",
        report.container_loader_handoff_ready
    );
    println!(
        "  container_loader_handoff_blockers: {}",
        joined_or_none(&report.container_loader_handoff_blockers)
    );
    println!(
        "  nsb_layout_hash: {}",
        report.nsb_layout_hash.as_deref().unwrap_or("<none>")
    );
    println!(
        "  nsb_byte_map_hash: {}",
        report.nsb_byte_map_hash.as_deref().unwrap_or("<none>")
    );
    println!("  scheduler_entry: {}", report.scheduler_entry);
    println!("  lifecycle_hook: {}", report.lifecycle_hook);
    println!("  launch_steps: {}", joined_or_none(&report.launch_steps));
    println!("  blockers: {}", joined_or_none(&report.blockers));
}

pub(super) fn render_json_report(report: &RunnerReport) -> String {
    format!(
        "{{\"kind\":\"nuis_host_runner\",\"protocol\":\"{}\",\"ready\":{},\"would_enter_lifecycle_hook\":{},\"manifest_path\":\"{}\",\"nsb_path\":{},\"nsb_readable\":{},\"nsb_hash_expected\":{},\"nsb_hash_actual\":{},\"nsb_hash_matches\":{},\"nsb_payload_offset\":{},\"nsb_payload_span\":{},\"nsb_payload_region_mapped\":{},\"nsb_payload_region_bytes\":{},\"nsb_payload_region_hash\":{},\"nsb_payload_scan_status\":\"{}\",\"nsb_payload_scan_kind\":\"{}\",\"nsb_payload_prefix_hex\":{},\"nsb_payload_prefix_text\":{},\"container_loader_status\":\"{}\",\"container_schema\":{},\"container_schema_version\":{},\"container_kind\":{},\"container_producer\":{},\"container_producer_phase\":{},\"container_ready\":{},\"container_blockers\":[{}],\"container_magic\":{},\"container_version\":{},\"container_metadata_table_hash\":{},\"container_section_table_hash\":{},\"container_hash\":{},\"container_section_count\":{},\"container_section_parsed_count\":{},\"container_first_section_id\":{},\"container_first_section_kind\":{},\"container_entry_section_found\":{},\"container_payload_size_bytes\":{},\"container_payload_hash\":{},\"container_payload_path\":{},\"container_loader_readiness\":{},\"container_loader_blockers\":[{}],\"container_loader_entry_kind\":{},\"container_loader_entry_symbol\":{},\"container_loader_entry_section_id\":{},\"container_loader_symbol_count\":{},\"loader_symbol_table_hash\":{},\"container_loader_symbol_status\":\"{}\",\"container_loader_symbol_id\":{},\"container_loader_symbol_kind\":{},\"container_loader_symbol_name\":{},\"container_loader_symbol_lifecycle_hook\":{},\"container_loader_symbol_section_id\":{},\"container_relocation_count\":{},\"relocation_table_hash\":{},\"container_relocation_parsed_count\":{},\"container_first_relocation_kind\":{},\"container_first_relocation_source_section_id\":{},\"container_first_relocation_target_symbol_id\":{},\"container_first_relocation_targets_loader_symbol\":{},\"container_first_relocation_source_matches_loader_symbol\":{},\"compatibility_domain_count\":{},\"compatibility_domain_table_hash\":{},\"compatibility_domain_parsed_count\":{},\"compatibility_domain_first_kind\":{},\"compatibility_domain_required_count\":{},\"external_import_count\":{},\"external_import_table_hash\":{},\"external_import_parsed_count\":{},\"external_import_first_kind\":{},\"external_import_first_name\":{},\"external_import_required_imports\":[{}],\"container_loader_handoff_status\":\"{}\",\"container_loader_handoff_ready\":{},\"container_loader_handoff_blockers\":[{}],\"nsb_layout_hash\":{},\"nsb_byte_map_hash\":{},\"scheduler_entry\":\"{}\",\"lifecycle_hook\":\"{}\",\"launch_steps\":[{}],\"blockers\":[{}]}}",
        json_escape(RUNNER_PROTOCOL),
        report.ready,
        report.would_enter_lifecycle_hook,
        json_escape(&report.manifest_path),
        json_optional_string(report.nsb_path.as_deref()),
        report.nsb_readable,
        json_optional_string(report.nsb_hash_expected.as_deref()),
        json_optional_string(report.nsb_hash_actual.as_deref()),
        report.nsb_hash_matches,
        json_optional_usize(report.nsb_payload_offset),
        json_optional_usize(report.nsb_payload_span),
        report.nsb_payload_region_mapped,
        json_optional_usize(report.nsb_payload_region_bytes),
        json_optional_string(report.nsb_payload_region_hash.as_deref()),
        json_escape(&report.nsb_payload_scan_status),
        json_escape(&report.nsb_payload_scan_kind),
        json_optional_string(report.nsb_payload_prefix_hex.as_deref()),
        json_optional_string(report.nsb_payload_prefix_text.as_deref()),
        json_escape(&report.container_loader_status),
        json_optional_string(report.container_schema.as_deref()),
        json_optional_usize(report.container_schema_version),
        json_optional_string(report.container_kind.as_deref()),
        json_optional_string(report.container_producer.as_deref()),
        json_optional_string(report.container_producer_phase.as_deref()),
        json_optional_bool(report.container_ready),
        json_string_array(&report.container_blockers),
        json_optional_string(report.container_magic.as_deref()),
        json_optional_usize(report.container_version),
        json_optional_string(report.container_metadata_table_hash.as_deref()),
        json_optional_string(report.container_section_table_hash.as_deref()),
        json_optional_string(report.container_hash.as_deref()),
        json_optional_usize(report.container_section_count),
        report.container_section_parsed_count,
        json_optional_string(report.container_first_section_id.as_deref()),
        json_optional_string(report.container_first_section_kind.as_deref()),
        report.container_entry_section_found,
        json_optional_usize(report.container_payload_size_bytes),
        json_optional_string(report.container_payload_hash.as_deref()),
        json_optional_string(report.container_payload_path.as_deref()),
        json_optional_string(report.container_loader_readiness.as_deref()),
        json_string_array(&report.container_loader_blockers),
        json_optional_string(report.container_loader_entry_kind.as_deref()),
        json_optional_string(report.container_loader_entry_symbol.as_deref()),
        json_optional_string(report.container_loader_entry_section_id.as_deref()),
        json_optional_usize(report.container_loader_symbol_count),
        json_optional_string(report.loader_symbol_table_hash.as_deref()),
        json_escape(&report.container_loader_symbol_status),
        json_optional_string(report.container_loader_symbol_id.as_deref()),
        json_optional_string(report.container_loader_symbol_kind.as_deref()),
        json_optional_string(report.container_loader_symbol_name.as_deref()),
        json_optional_string(report.container_loader_symbol_lifecycle_hook.as_deref()),
        json_optional_string(report.container_loader_symbol_section_id.as_deref()),
        json_optional_usize(report.container_relocation_count),
        json_optional_string(report.relocation_table_hash.as_deref()),
        report.container_relocation_parsed_count,
        json_optional_string(report.container_first_relocation_kind.as_deref()),
        json_optional_string(report.container_first_relocation_source_section_id.as_deref()),
        json_optional_string(report.container_first_relocation_target_symbol_id.as_deref()),
        report.container_first_relocation_targets_loader_symbol,
        report.container_first_relocation_source_matches_loader_symbol,
        json_optional_usize(report.compatibility_domain_count),
        json_optional_string(report.compatibility_domain_table_hash.as_deref()),
        report.compatibility_domain_parsed_count,
        json_optional_string(report.compatibility_domain_first_kind.as_deref()),
        report.compatibility_domain_required_count,
        json_optional_usize(report.external_import_count),
        json_optional_string(report.external_import_table_hash.as_deref()),
        report.external_import_parsed_count,
        json_optional_string(report.external_import_first_kind.as_deref()),
        json_optional_string(report.external_import_first_name.as_deref()),
        json_string_array(&report.external_import_required_imports),
        json_escape(&report.container_loader_handoff_status),
        report.container_loader_handoff_ready,
        json_string_array(&report.container_loader_handoff_blockers),
        json_optional_string(report.nsb_layout_hash.as_deref()),
        json_optional_string(report.nsb_byte_map_hash.as_deref()),
        json_escape(&report.scheduler_entry),
        json_escape(&report.lifecycle_hook),
        json_string_array(&report.launch_steps),
        json_string_array(&report.blockers)
    )
}

fn optional_usize_text(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "<none>".to_owned())
}

fn joined_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "<none>".to_owned()
    } else {
        values.join(", ")
    }
}

fn json_optional_usize(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_owned())
}

fn json_optional_bool(value: Option<bool>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_owned())
}

fn json_optional_string(value: Option<&str>) -> String {
    value
        .map(|value| format!("\"{}\"", json_escape(value)))
        .unwrap_or_else(|| "null".to_owned())
}

fn json_string_array(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(",")
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
