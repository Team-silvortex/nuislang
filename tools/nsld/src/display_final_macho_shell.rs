use crate::reports::{
    NsldMachOArm64ShellImageSerializationReport, NsldMachOArm64ShellLayoutPlanReport,
};

pub(crate) fn print_macho_shell_image_serialization(
    report: &NsldMachOArm64ShellImageSerializationReport,
) {
    println!(
        "  finalizer_input_shell_image: contract={} status={} layout_hash={} image_hash={} ledger_hash={} span={} header={} commands={} copied_sections={}:{} rewrites={}:{}:{}:{} publication={}",
        report.contract,
        report.status,
        report.shell_layout_plan_hash,
        report.shell_image_hash,
        report.serialization_ledger_hash,
        report.shell_image_span_bytes,
        report.header_bytes,
        report.load_command_bytes,
        report.copied_section_count,
        report.copied_section_bytes,
        report.rewrite_count,
        report.relocation_rewrite_count,
        report.stub_rewrite_count,
        report.got_rewrite_count,
        report.publication_status
    );
    println!(
        "  finalizer_input_shell_image_linkedit: rebase_hash={} bind_hash={} symbol_hash={} indirect_hash={} string_hash={} linkedit_hash={}",
        report.rebase_stream_hash,
        report.bind_stream_hash,
        report.symbol_table_hash,
        report.indirect_symbol_table_hash,
        report.string_table_hash,
        report.linkedit_hash
    );
    println!(
        "  finalizer_input_shell_image_code_signature: status={} file_offset={}",
        report.code_signature_status, report.code_signature_file_offset
    );
    for rewrite in &report.rewrites {
        println!(
            "  finalizer_input_shell_image_rewrite: id={} kind={} source={} source_offset={} file_offset={} vm=0x{:x} target_vm={} addend={} width={} prewrite_hash={} source_hash={} encoded_hash={} audit_hash={}",
            rewrite.rewrite_id,
            rewrite.rewrite_kind,
            rewrite.source_id,
            rewrite.source_image_offset,
            rewrite.file_offset,
            rewrite.vm_address,
            option_u64_hex(rewrite.target_vm_address),
            rewrite
                .effective_addend
                .map_or_else(|| "none".to_owned(), |value| value.to_string()),
            rewrite.width_bytes,
            rewrite.prewrite_bytes_hash,
            rewrite.encoding_source_bytes_hash,
            rewrite.encoded_bytes_hash,
            rewrite.audit_hash
        );
    }
}

pub(crate) fn print_macho_shell_layout_plan(report: &NsldMachOArm64ShellLayoutPlanReport) {
    println!(
        "  finalizer_input_shell_layout: contract={} status={} object_linkage_hash={} plan_hash={} page_size={} image_base=0x{:x} header_bytes={} commands={}:{} first_content={} file_span={} rewrites={}",
        report.contract,
        report.status,
        report.object_linkage_hash,
        report.plan_hash,
        report.page_size,
        report.image_base_vm_address,
        report.header_size_bytes,
        report.load_command_count,
        report.load_command_size_bytes,
        report.first_content_file_offset,
        report.planned_file_span_bytes,
        report.required_address_rewrite_count
    );
    println!(
        "  finalizer_input_shell_entry: rule={} symbol={} source_offset={} file_offset={} vm_address=0x{:x}",
        report.entry_rule_id,
        report.entry_symbol,
        report.entry_source_image_offset,
        report.entry_file_offset,
        report.entry_vm_address
    );
    println!(
        "  finalizer_input_shell_linkedit: file_offset={} bytes={} rebase={}:{} bind={}:{} symtab={}:{} indirect={}:{} strings={}:{}",
        report.linkedit_file_offset,
        report.linkedit_bytes,
        report.rebase_stream_offset,
        report.rebase_stream_bytes,
        report.bind_stream_offset,
        report.bind_stream_bytes,
        report.symbol_table_offset,
        report.symbol_table_bytes,
        report.indirect_symbol_table_offset,
        report.indirect_symbol_table_bytes,
        report.string_table_offset,
        report.string_table_bytes
    );
    println!(
        "  finalizer_input_shell_code_signature: status={} file_offset={}",
        report.code_signature_status, report.code_signature_file_offset
    );
    for segment in &report.segments {
        println!(
            "  finalizer_input_shell_segment: id={} index={} name={} file={}:{} vm=0x{:x}:{} protection={}:{} sections={} audit_hash={}",
            segment.segment_id,
            segment.segment_index,
            segment.segment_name,
            segment.file_offset,
            segment.file_size_bytes,
            segment.vm_address,
            segment.vm_size_bytes,
            segment.max_protection,
            segment.initial_protection,
            segment.section_ids.join(","),
            segment.audit_hash
        );
    }
    for section in &report.sections {
        println!(
            "  finalizer_input_shell_section: id={} ordinal={} source={}:{} segment={} section={} source_offset={} source_bytes={} align={} file_offset={} file_bytes={} vm=0x{:x}:{} flags=0x{:08x} reserved={}:{} audit_hash={}",
            section.section_id,
            section.section_ordinal,
            section.source_kind,
            section.source_id,
            section.segment_name,
            section.section_name,
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
        );
    }
    for symbol in &report.symbols {
        println!(
            "  finalizer_input_shell_symbol: id={} index={} name={} kind={} object={} source_index={} section={} source_offset={} vm_address={} string_offset={} dylib={} audit_hash={}",
            symbol.symbol_id,
            symbol.symbol_table_index,
            symbol.name,
            symbol.record_kind,
            symbol.object_id.as_deref().unwrap_or("none"),
            option_usize(symbol.source_symbol_index),
            symbol.shell_section_id.as_deref().unwrap_or("none"),
            option_usize(symbol.source_image_offset),
            option_u64_hex(symbol.vm_address),
            symbol.string_table_offset,
            option_usize(symbol.dylib_ordinal),
            symbol.audit_hash
        );
    }
    for symbol in &report.indirect_symbols {
        println!(
            "  finalizer_input_shell_indirect: id={} section={} slot={} target={} symbol_index={} marker={} audit_hash={}",
            symbol.indirect_id,
            symbol.shell_section_id,
            symbol.slot_index,
            symbol.target_symbol,
            option_usize(symbol.symbol_table_index),
            symbol.marker.as_deref().unwrap_or("none"),
            symbol.audit_hash
        );
    }
    for bind in &report.binds {
        println!(
            "  finalizer_input_shell_bind: id={} source={} target={} dylib={} source_offset={} section={} segment={}:{} file_offset={} vm=0x{:x} encoded_bytes={} audit_hash={}",
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
        );
    }
    for rebase in &report.rebases {
        println!(
            "  finalizer_input_shell_rebase: id={} structure={} target={} got_source={} target_source={} section={} segment={}:{} file_offset={} vm=0x{:x} target_vm=0x{:x} encoded_bytes={} audit_hash={}",
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
        );
    }
    for command in &report.load_commands {
        println!(
            "  finalizer_input_shell_command: id={} kind={} value=0x{:x} offset={} bytes={} segment={} status={} audit_hash={}",
            command.command_id,
            command.command_kind,
            command.command_value,
            command.command_offset,
            command.command_size_bytes,
            command.segment_id.as_deref().unwrap_or("none"),
            command.status,
            command.audit_hash
        );
    }
}

fn option_usize(value: Option<usize>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| value.to_string())
}

fn option_u64_hex(value: Option<u64>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| format!("0x{value:x}"))
}
