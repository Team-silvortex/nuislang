use crate::{
    json_fields::*,
    reports::{
        NsldMachOArm64CodeSignatureReport, NsldMachOArm64ShellImageSerializationReport,
        NsldMachOArm64ShellLayoutPlanReport,
    },
};

pub(crate) fn macho_shell_image_serialization_json(
    report: &NsldMachOArm64ShellImageSerializationReport,
) -> String {
    let rewrites = report
        .rewrites
        .iter()
        .map(|rewrite| {
            let fields = [
                json_string_field("rewrite_id", &rewrite.rewrite_id),
                json_string_field("rewrite_kind", &rewrite.rewrite_kind),
                json_string_field("source_id", &rewrite.source_id),
                json_usize_field("source_image_offset", rewrite.source_image_offset),
                json_usize_field("file_offset", rewrite.file_offset),
                json_u64_field("vm_address", rewrite.vm_address),
                json_optional_u64_field("target_vm_address", rewrite.target_vm_address),
                json_optional_i64_field("effective_addend", rewrite.effective_addend),
                json_usize_field("width_bytes", rewrite.width_bytes),
                json_string_field("prewrite_bytes_hash", &rewrite.prewrite_bytes_hash),
                json_string_field(
                    "encoding_source_bytes_hash",
                    &rewrite.encoding_source_bytes_hash,
                ),
                json_string_field("encoded_bytes_hash", &rewrite.encoded_bytes_hash),
                json_string_field("audit_hash", &rewrite.audit_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    let fields = [
        json_string_field("contract", &report.contract),
        json_string_field("status", &report.status),
        json_string_field("shell_layout_plan_hash", &report.shell_layout_plan_hash),
        json_string_field(
            "platform_application_ledger_hash",
            &report.platform_application_ledger_hash,
        ),
        json_string_field("platform_image_hash", &report.platform_image_hash),
        json_usize_field("shell_image_span_bytes", report.shell_image_span_bytes),
        json_usize_field("header_bytes", report.header_bytes),
        json_usize_field("load_command_bytes", report.load_command_bytes),
        json_usize_field("copied_section_count", report.copied_section_count),
        json_usize_field("copied_section_bytes", report.copied_section_bytes),
        json_usize_field("relocation_rewrite_count", report.relocation_rewrite_count),
        json_usize_field("stub_rewrite_count", report.stub_rewrite_count),
        json_usize_field("got_rewrite_count", report.got_rewrite_count),
        json_usize_field("rewrite_count", report.rewrite_count),
        json_string_field("header_hash", &report.header_hash),
        json_string_field("load_commands_hash", &report.load_commands_hash),
        json_string_field("rebase_stream_hash", &report.rebase_stream_hash),
        json_string_field("bind_stream_hash", &report.bind_stream_hash),
        json_string_field("symbol_table_hash", &report.symbol_table_hash),
        json_string_field(
            "indirect_symbol_table_hash",
            &report.indirect_symbol_table_hash,
        ),
        json_string_field("string_table_hash", &report.string_table_hash),
        json_string_field("linkedit_hash", &report.linkedit_hash),
        json_string_field("shell_image_hash", &report.shell_image_hash),
        json_string_field(
            "serialization_ledger_hash",
            &report.serialization_ledger_hash,
        ),
        json_usize_field(
            "code_signature_file_offset",
            report.code_signature_file_offset,
        ),
        json_string_field("code_signature_status", &report.code_signature_status),
        json_string_field("publication_status", &report.publication_status),
        macho_code_signature_json(&report.code_signature),
        format!("\"rewrites\":[{rewrites}]"),
    ];
    format!("\"shell_image_serialization\":{{{}}}", fields.join(","))
}

fn macho_code_signature_json(report: &NsldMachOArm64CodeSignatureReport) -> String {
    let slots = report
        .slots
        .iter()
        .map(|slot| {
            let fields = [
                json_usize_field("slot_index", slot.slot_index),
                json_usize_field("file_offset", slot.file_offset),
                json_usize_field("file_size_bytes", slot.file_size_bytes),
                json_string_field("digest_sha256", &slot.digest_sha256),
                json_string_field("audit_hash", &slot.audit_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    let fields = [
        json_string_field("contract", &report.contract),
        json_string_field("status", &report.status),
        json_string_field("identifier", &report.identifier),
        json_u32_field("code_directory_version", report.code_directory_version),
        json_u32_field("flags", report.flags),
        json_string_field("hash_type", &report.hash_type),
        json_usize_field("hash_size_bytes", report.hash_size_bytes),
        json_usize_field("page_size_bytes", report.page_size_bytes),
        json_usize_field("code_limit", report.code_limit),
        json_usize_field("code_slot_count", report.code_slot_count),
        json_usize_field("verified_code_slot_count", report.verified_code_slot_count),
        json_usize_field("signature_file_offset", report.signature_file_offset),
        json_usize_field("signature_blob_bytes", report.signature_blob_bytes),
        json_usize_field("signature_payload_bytes", report.signature_payload_bytes),
        json_string_field("signed_content_sha256", &report.signed_content_sha256),
        json_string_field("code_directory_sha256", &report.code_directory_sha256),
        json_string_field("cdhash", &report.cdhash),
        json_string_field("signature_payload_sha256", &report.signature_payload_sha256),
        json_usize_field("load_command_count", report.load_command_count),
        json_usize_field(
            "verified_load_command_count",
            report.verified_load_command_count,
        ),
        json_usize_field("load_command_bytes", report.load_command_bytes),
        json_bool_field(
            "linkedit_covers_signature",
            report.linkedit_covers_signature,
        ),
        json_bool_field("signed_ranges_valid", report.signed_ranges_valid),
        json_bool_field("padding_valid", report.padding_valid),
        json_string_field("validation_contract", &report.validation_contract),
        json_string_field("validation_status", &report.validation_status),
        json_string_field(
            "publication_eligibility_contract",
            &report.publication_eligibility_contract,
        ),
        json_string_field(
            "publication_eligibility_status",
            &report.publication_eligibility_status,
        ),
        json_bool_field("publication_eligible", report.publication_eligible),
        json_string_array_field("publication_blockers", &report.publication_blockers),
        json_string_field("validation_ledger_hash", &report.validation_ledger_hash),
        format!("\"slots\":[{slots}]"),
    ];
    format!("\"code_signature\":{{{}}}", fields.join(","))
}

pub(crate) fn macho_shell_layout_plan_json(report: &NsldMachOArm64ShellLayoutPlanReport) -> String {
    let fields = [
        json_string_field("contract", &report.contract),
        json_string_field("status", &report.status),
        json_string_field("object_linkage_hash", &report.object_linkage_hash),
        json_string_field("placement_plan_hash", &report.placement_plan_hash),
        json_string_field(
            "platform_structure_plan_hash",
            &report.platform_structure_plan_hash,
        ),
        json_string_field(
            "platform_application_ledger_hash",
            &report.platform_application_ledger_hash,
        ),
        json_string_field("platform_image_hash", &report.platform_image_hash),
        json_usize_field("page_size", report.page_size),
        json_u64_field("image_base_vm_address", report.image_base_vm_address),
        json_usize_field("header_size_bytes", report.header_size_bytes),
        json_usize_field("load_command_count", report.load_command_count),
        json_usize_field("load_command_size_bytes", report.load_command_size_bytes),
        json_usize_field(
            "first_content_file_offset",
            report.first_content_file_offset,
        ),
        json_string_field("entry_rule_id", &report.entry_rule_id),
        json_string_field("entry_symbol", &report.entry_symbol),
        json_usize_field(
            "entry_source_image_offset",
            report.entry_source_image_offset,
        ),
        json_usize_field("entry_file_offset", report.entry_file_offset),
        json_u64_field("entry_vm_address", report.entry_vm_address),
        json_usize_field("segment_count", report.segment_count),
        json_usize_field("section_count", report.section_count),
        json_usize_field("defined_symbol_count", report.defined_symbol_count),
        json_usize_field("undefined_symbol_count", report.undefined_symbol_count),
        json_usize_field("symbol_table_offset", report.symbol_table_offset),
        json_usize_field("symbol_table_bytes", report.symbol_table_bytes),
        json_usize_field(
            "indirect_symbol_table_offset",
            report.indirect_symbol_table_offset,
        ),
        json_usize_field("indirect_symbol_count", report.indirect_symbol_count),
        json_usize_field(
            "indirect_symbol_table_bytes",
            report.indirect_symbol_table_bytes,
        ),
        json_usize_field("string_table_offset", report.string_table_offset),
        json_usize_field("string_table_bytes", report.string_table_bytes),
        json_usize_field("rebase_stream_offset", report.rebase_stream_offset),
        json_usize_field("rebase_stream_bytes", report.rebase_stream_bytes),
        json_usize_field("bind_stream_offset", report.bind_stream_offset),
        json_usize_field("bind_stream_bytes", report.bind_stream_bytes),
        json_usize_field("linkedit_file_offset", report.linkedit_file_offset),
        json_usize_field("linkedit_bytes", report.linkedit_bytes),
        json_usize_field(
            "code_signature_file_offset",
            report.code_signature_file_offset,
        ),
        json_string_field("code_signature_status", &report.code_signature_status),
        json_usize_field(
            "required_address_rewrite_count",
            report.required_address_rewrite_count,
        ),
        json_usize_field("planned_file_span_bytes", report.planned_file_span_bytes),
        json_string_field("plan_hash", &report.plan_hash),
        segments_json(report),
        sections_json(report),
        symbols_json(report),
        indirect_symbols_json(report),
        binds_json(report),
        rebases_json(report),
        load_commands_json(report),
    ];
    format!("\"shell_layout_plan\":{{{}}}", fields.join(","))
}

fn segments_json(report: &NsldMachOArm64ShellLayoutPlanReport) -> String {
    let records = report
        .segments
        .iter()
        .map(|segment| {
            let fields = [
                json_string_field("segment_id", &segment.segment_id),
                json_string_field("segment_name", &segment.segment_name),
                json_usize_field("segment_index", segment.segment_index),
                json_usize_field("file_offset", segment.file_offset),
                json_usize_field("file_size_bytes", segment.file_size_bytes),
                json_u64_field("vm_address", segment.vm_address),
                json_usize_field("vm_size_bytes", segment.vm_size_bytes),
                json_u32_field("max_protection", segment.max_protection),
                json_u32_field("initial_protection", segment.initial_protection),
                json_string_array_field("section_ids", &segment.section_ids),
                json_string_field("audit_hash", &segment.audit_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("\"segments\":[{records}]")
}

fn sections_json(report: &NsldMachOArm64ShellLayoutPlanReport) -> String {
    let records = report
        .sections
        .iter()
        .map(|section| {
            let fields = [
                json_string_field("section_id", &section.section_id),
                json_string_field("source_kind", &section.source_kind),
                json_string_field("source_id", &section.source_id),
                json_string_field("segment_name", &section.segment_name),
                json_string_field("section_name", &section.section_name),
                json_usize_field("section_ordinal", section.section_ordinal),
                json_optional_usize_field("source_image_offset", section.source_image_offset),
                json_usize_field("source_size_bytes", section.source_size_bytes),
                json_usize_field("alignment", section.alignment),
                json_optional_usize_field("file_offset", section.file_offset),
                json_usize_field("file_size_bytes", section.file_size_bytes),
                json_u64_field("vm_address", section.vm_address),
                json_usize_field("vm_size_bytes", section.vm_size_bytes),
                json_u32_field("flags", section.flags),
                json_u32_field("reserved1", section.reserved1),
                json_u32_field("reserved2", section.reserved2),
                json_string_field("audit_hash", &section.audit_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("\"sections\":[{records}]")
}

fn symbols_json(report: &NsldMachOArm64ShellLayoutPlanReport) -> String {
    let records = report
        .symbols
        .iter()
        .map(|symbol| {
            let fields = [
                json_string_field("symbol_id", &symbol.symbol_id),
                json_string_field("name", &symbol.name),
                json_string_field("record_kind", &symbol.record_kind),
                json_optional_string_field("object_id", symbol.object_id.as_deref()),
                json_optional_usize_field("source_symbol_index", symbol.source_symbol_index),
                json_optional_string_field("shell_section_id", symbol.shell_section_id.as_deref()),
                json_optional_usize_field("source_image_offset", symbol.source_image_offset),
                json_optional_u64_field("vm_address", symbol.vm_address),
                json_usize_field("symbol_table_index", symbol.symbol_table_index),
                json_usize_field("string_table_offset", symbol.string_table_offset),
                json_optional_usize_field("dylib_ordinal", symbol.dylib_ordinal),
                json_string_field("audit_hash", &symbol.audit_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("\"symbols\":[{records}]")
}

fn indirect_symbols_json(report: &NsldMachOArm64ShellLayoutPlanReport) -> String {
    let records = report
        .indirect_symbols
        .iter()
        .map(|symbol| {
            let fields = [
                json_string_field("indirect_id", &symbol.indirect_id),
                json_string_field("shell_section_id", &symbol.shell_section_id),
                json_usize_field("slot_index", symbol.slot_index),
                json_string_field("target_symbol", &symbol.target_symbol),
                json_optional_usize_field("symbol_table_index", symbol.symbol_table_index),
                json_optional_string_field("marker", symbol.marker.as_deref()),
                json_string_field("audit_hash", &symbol.audit_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("\"indirect_symbols\":[{records}]")
}

fn binds_json(report: &NsldMachOArm64ShellLayoutPlanReport) -> String {
    let records = report
        .binds
        .iter()
        .map(|bind| {
            let fields = [
                json_string_field("bind_id", &bind.bind_id),
                json_string_field("source_bind_id", &bind.source_bind_id),
                json_string_field("target_symbol", &bind.target_symbol),
                json_usize_field("dylib_ordinal", bind.dylib_ordinal),
                json_usize_field("got_source_image_offset", bind.got_source_image_offset),
                json_string_field("shell_section_id", &bind.shell_section_id),
                json_usize_field("segment_index", bind.segment_index),
                json_usize_field("segment_offset", bind.segment_offset),
                json_usize_field("file_offset", bind.file_offset),
                json_u64_field("vm_address", bind.vm_address),
                json_usize_field("encoded_size_bytes", bind.encoded_size_bytes),
                json_string_field("audit_hash", &bind.audit_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("\"binds\":[{records}]")
}

fn rebases_json(report: &NsldMachOArm64ShellLayoutPlanReport) -> String {
    let records = report
        .rebases
        .iter()
        .map(|rebase| {
            let fields = [
                json_string_field("rebase_id", &rebase.rebase_id),
                json_string_field("structure_id", &rebase.structure_id),
                json_string_field("target_symbol", &rebase.target_symbol),
                json_usize_field("got_source_image_offset", rebase.got_source_image_offset),
                json_usize_field(
                    "target_source_image_offset",
                    rebase.target_source_image_offset,
                ),
                json_string_field("shell_section_id", &rebase.shell_section_id),
                json_usize_field("segment_index", rebase.segment_index),
                json_usize_field("segment_offset", rebase.segment_offset),
                json_usize_field("file_offset", rebase.file_offset),
                json_u64_field("vm_address", rebase.vm_address),
                json_u64_field("target_vm_address", rebase.target_vm_address),
                json_usize_field("encoded_size_bytes", rebase.encoded_size_bytes),
                json_string_field("audit_hash", &rebase.audit_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("\"rebases\":[{records}]")
}

fn load_commands_json(report: &NsldMachOArm64ShellLayoutPlanReport) -> String {
    let records = report
        .load_commands
        .iter()
        .map(|command| {
            let fields = [
                json_string_field("command_id", &command.command_id),
                json_string_field("command_kind", &command.command_kind),
                json_u32_field("command_value", command.command_value),
                json_usize_field("command_offset", command.command_offset),
                json_usize_field("command_size_bytes", command.command_size_bytes),
                json_optional_string_field("segment_id", command.segment_id.as_deref()),
                json_string_field("status", &command.status),
                json_string_field("audit_hash", &command.audit_hash),
            ];
            format!("{{{}}}", fields.join(","))
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("\"load_commands\":[{records}]")
}

fn json_u32_field(name: &str, value: u32) -> String {
    format!("\"{name}\":{value}")
}

fn json_u64_field(name: &str, value: u64) -> String {
    format!("\"{name}\":{value}")
}

fn json_optional_u64_field(name: &str, value: Option<u64>) -> String {
    value.map_or_else(
        || format!("\"{name}\":null"),
        |value| json_u64_field(name, value),
    )
}
