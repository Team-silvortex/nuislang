use crate::{reports::NsldMachOArm64ShellLayoutPlanReport, toml};
use std::fmt::Write as _;

pub(crate) fn render_macho_shell_layout_plan(
    out: &mut String,
    report: &NsldMachOArm64ShellLayoutPlanReport,
) {
    for (name, value) in [
        ("contract", report.contract.as_str()),
        ("status", report.status.as_str()),
        ("object_linkage_hash", report.object_linkage_hash.as_str()),
        ("placement_plan_hash", report.placement_plan_hash.as_str()),
        (
            "platform_structure_plan_hash",
            report.platform_structure_plan_hash.as_str(),
        ),
        (
            "platform_application_ledger_hash",
            report.platform_application_ledger_hash.as_str(),
        ),
        ("platform_image_hash", report.platform_image_hash.as_str()),
        ("entry_rule_id", report.entry_rule_id.as_str()),
        ("entry_symbol", report.entry_symbol.as_str()),
        (
            "code_signature_status",
            report.code_signature_status.as_str(),
        ),
        ("plan_hash", report.plan_hash.as_str()),
    ] {
        string_field(out, name, value);
    }
    for (name, value) in [
        ("page_size", report.page_size),
        ("header_size_bytes", report.header_size_bytes),
        ("load_command_count", report.load_command_count),
        ("load_command_size_bytes", report.load_command_size_bytes),
        (
            "first_content_file_offset",
            report.first_content_file_offset,
        ),
        (
            "entry_source_image_offset",
            report.entry_source_image_offset,
        ),
        ("entry_file_offset", report.entry_file_offset),
        ("segment_count", report.segment_count),
        ("section_count", report.section_count),
        ("defined_symbol_count", report.defined_symbol_count),
        ("undefined_symbol_count", report.undefined_symbol_count),
        ("symbol_table_offset", report.symbol_table_offset),
        ("symbol_table_bytes", report.symbol_table_bytes),
        (
            "indirect_symbol_table_offset",
            report.indirect_symbol_table_offset,
        ),
        ("indirect_symbol_count", report.indirect_symbol_count),
        (
            "indirect_symbol_table_bytes",
            report.indirect_symbol_table_bytes,
        ),
        ("string_table_offset", report.string_table_offset),
        ("string_table_bytes", report.string_table_bytes),
        ("rebase_stream_offset", report.rebase_stream_offset),
        ("rebase_stream_bytes", report.rebase_stream_bytes),
        ("bind_stream_offset", report.bind_stream_offset),
        ("bind_stream_bytes", report.bind_stream_bytes),
        ("linkedit_file_offset", report.linkedit_file_offset),
        ("linkedit_bytes", report.linkedit_bytes),
        (
            "code_signature_file_offset",
            report.code_signature_file_offset,
        ),
        (
            "required_address_rewrite_count",
            report.required_address_rewrite_count,
        ),
        ("planned_file_span_bytes", report.planned_file_span_bytes),
    ] {
        usize_field(out, name, value);
    }
    u64_field(out, "image_base_vm_address", report.image_base_vm_address);
    u64_field(out, "entry_vm_address", report.entry_vm_address);

    let segments = report
        .segments
        .iter()
        .map(|segment| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                segment.segment_id,
                segment.segment_name,
                segment.segment_index,
                segment.file_offset,
                segment.file_size_bytes,
                segment.vm_address,
                segment.vm_size_bytes,
                segment.max_protection,
                segment.initial_protection,
                segment.section_ids.join(","),
                segment.audit_hash
            )
        })
        .collect::<Vec<_>>();
    array_field(out, "segments", &segments);
    let sections = report
        .sections
        .iter()
        .map(|section| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                section.section_id,
                section.source_kind,
                section.source_id,
                section.segment_name,
                section.section_name,
                section.section_ordinal,
                option_usize(section.source_image_offset),
                section.source_size_bytes,
                section.alignment,
                option_usize(section.file_offset),
                section.file_size_bytes,
                section.vm_address,
                section.vm_size_bytes,
                section.flags,
                section.reserved1,
                section.reserved2,
                section.audit_hash
            )
        })
        .collect::<Vec<_>>();
    array_field(out, "sections", &sections);
    let symbols = report
        .symbols
        .iter()
        .map(|symbol| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                symbol.symbol_id,
                symbol.name,
                symbol.record_kind,
                symbol.object_id.as_deref().unwrap_or("none"),
                option_usize(symbol.source_symbol_index),
                symbol.shell_section_id.as_deref().unwrap_or("none"),
                option_usize(symbol.source_image_offset),
                option_u64(symbol.vm_address),
                symbol.symbol_table_index,
                symbol.string_table_offset,
                option_usize(symbol.dylib_ordinal),
                symbol.audit_hash
            )
        })
        .collect::<Vec<_>>();
    array_field(out, "symbols", &symbols);
    let indirect = report
        .indirect_symbols
        .iter()
        .map(|symbol| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}",
                symbol.indirect_id,
                symbol.shell_section_id,
                symbol.slot_index,
                symbol.target_symbol,
                option_usize(symbol.symbol_table_index),
                symbol.marker.as_deref().unwrap_or("none"),
                symbol.audit_hash
            )
        })
        .collect::<Vec<_>>();
    array_field(out, "indirect_symbols", &indirect);
    let binds = report
        .binds
        .iter()
        .map(|bind| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                bind.bind_id,
                bind.source_bind_id,
                bind.target_symbol,
                bind.dylib_ordinal,
                bind.got_source_image_offset,
                bind.shell_section_id,
                bind.segment_index,
                bind.segment_offset,
                bind.file_offset,
                bind.vm_address,
                bind.encoded_size_bytes,
                bind.audit_hash
            )
        })
        .collect::<Vec<_>>();
    array_field(out, "binds", &binds);
    let rebases = report
        .rebases
        .iter()
        .map(|rebase| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                rebase.rebase_id,
                rebase.structure_id,
                rebase.target_symbol,
                rebase.got_source_image_offset,
                rebase.target_source_image_offset,
                rebase.shell_section_id,
                rebase.segment_index,
                rebase.segment_offset,
                rebase.file_offset,
                rebase.vm_address,
                rebase.target_vm_address,
                rebase.encoded_size_bytes,
                rebase.audit_hash
            )
        })
        .collect::<Vec<_>>();
    array_field(out, "rebases", &rebases);
    let commands = report
        .load_commands
        .iter()
        .map(|command| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}|{}",
                command.command_id,
                command.command_kind,
                command.command_value,
                command.command_offset,
                command.command_size_bytes,
                command.segment_id.as_deref().unwrap_or("none"),
                command.status,
                command.audit_hash
            )
        })
        .collect::<Vec<_>>();
    array_field(out, "load_commands", &commands);
}

fn string_field(out: &mut String, name: &str, value: &str) {
    writeln!(
        out,
        "finalizer_input_shell_layout_{name} = \"{}\"",
        toml::escape_toml_string(value)
    )
    .unwrap();
}

fn usize_field(out: &mut String, name: &str, value: usize) {
    writeln!(out, "finalizer_input_shell_layout_{name} = {value}").unwrap();
}

fn u64_field(out: &mut String, name: &str, value: u64) {
    writeln!(out, "finalizer_input_shell_layout_{name} = {value}").unwrap();
}

fn array_field(out: &mut String, name: &str, values: &[String]) {
    writeln!(
        out,
        "finalizer_input_shell_layout_{name} = [{}]",
        toml::toml_string_array_literal(values)
    )
    .unwrap();
}

fn option_usize(value: Option<usize>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

fn option_u64(value: Option<u64>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}
