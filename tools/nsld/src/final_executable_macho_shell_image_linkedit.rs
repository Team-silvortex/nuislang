use crate::reports::NsldMachOArm64ShellLayoutPlanReport;

const N_EXT: u8 = 0x01;
const N_SECT: u8 = 0x0e;
const N_ABS: u8 = 0x02;
const INDIRECT_SYMBOL_LOCAL: u32 = 0x8000_0000;
const INDIRECT_SYMBOL_ABS: u32 = 0x4000_0000;

const REBASE_OPCODE_DONE: u8 = 0x00;
const REBASE_OPCODE_SET_TYPE_IMM: u8 = 0x10;
const REBASE_OPCODE_SET_SEGMENT_AND_OFFSET_ULEB: u8 = 0x20;
const REBASE_OPCODE_DO_REBASE_IMM_TIMES: u8 = 0x50;

const BIND_OPCODE_DONE: u8 = 0x00;
const BIND_OPCODE_SET_DYLIB_ORDINAL_IMM: u8 = 0x10;
const BIND_OPCODE_SET_SYMBOL_TRAILING_FLAGS_IMM: u8 = 0x40;
const BIND_OPCODE_SET_TYPE_IMM: u8 = 0x50;
const BIND_OPCODE_SET_SEGMENT_AND_OFFSET_ULEB: u8 = 0x70;
const BIND_OPCODE_DO_BIND: u8 = 0x90;

const POINTER_TYPE: u8 = 1;

pub(crate) struct EncodedShellLinkedit {
    pub(crate) rebase_stream: Vec<u8>,
    pub(crate) bind_stream: Vec<u8>,
    pub(crate) symbol_table: Vec<u8>,
    pub(crate) indirect_symbol_table: Vec<u8>,
    pub(crate) string_table: Vec<u8>,
}

pub(crate) fn encode_shell_linkedit(
    plan: &NsldMachOArm64ShellLayoutPlanReport,
) -> Result<EncodedShellLinkedit, String> {
    let encoded = EncodedShellLinkedit {
        rebase_stream: encode_rebase_stream(plan)?,
        bind_stream: encode_bind_stream(plan)?,
        symbol_table: encode_symbol_table(plan)?,
        indirect_symbol_table: encode_indirect_symbols(plan)?,
        string_table: encode_string_table(plan)?,
    };
    validate_lengths(plan, &encoded)?;
    Ok(encoded)
}

fn encode_rebase_stream(plan: &NsldMachOArm64ShellLayoutPlanReport) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::with_capacity(plan.rebase_stream_bytes);
    for rebase in &plan.rebases {
        if rebase.segment_index > 15 {
            return Err(format!(
                "Mach-O rebase `{}` segment index exceeds opcode space",
                rebase.rebase_id
            ));
        }
        let start = bytes.len();
        bytes.push(REBASE_OPCODE_SET_TYPE_IMM | POINTER_TYPE);
        bytes.push(REBASE_OPCODE_SET_SEGMENT_AND_OFFSET_ULEB | rebase.segment_index as u8);
        encode_uleb(rebase.segment_offset, &mut bytes);
        bytes.push(REBASE_OPCODE_DO_REBASE_IMM_TIMES | 1);
        if bytes.len() - start != rebase.encoded_size_bytes {
            return Err(format!(
                "Mach-O rebase `{}` encoded size drift",
                rebase.rebase_id
            ));
        }
    }
    if !plan.rebases.is_empty() {
        bytes.push(REBASE_OPCODE_DONE);
    }
    Ok(bytes)
}

fn encode_bind_stream(plan: &NsldMachOArm64ShellLayoutPlanReport) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::with_capacity(plan.bind_stream_bytes);
    for bind in &plan.binds {
        if bind.dylib_ordinal == 0 || bind.dylib_ordinal > 15 || bind.segment_index > 15 {
            return Err(format!(
                "Mach-O bind `{}` ordinal or segment index exceeds opcode space",
                bind.bind_id
            ));
        }
        if bind.target_symbol.as_bytes().contains(&0) {
            return Err(format!(
                "Mach-O bind `{}` symbol contains an embedded NUL",
                bind.bind_id
            ));
        }
        let start = bytes.len();
        bytes.push(BIND_OPCODE_SET_DYLIB_ORDINAL_IMM | bind.dylib_ordinal as u8);
        bytes.push(BIND_OPCODE_SET_SYMBOL_TRAILING_FLAGS_IMM);
        bytes.extend_from_slice(bind.target_symbol.as_bytes());
        bytes.push(0);
        bytes.push(BIND_OPCODE_SET_TYPE_IMM | POINTER_TYPE);
        bytes.push(BIND_OPCODE_SET_SEGMENT_AND_OFFSET_ULEB | bind.segment_index as u8);
        encode_uleb(bind.segment_offset, &mut bytes);
        bytes.push(BIND_OPCODE_DO_BIND);
        if bytes.len() - start != bind.encoded_size_bytes {
            return Err(format!("Mach-O bind `{}` encoded size drift", bind.bind_id));
        }
    }
    if !plan.binds.is_empty() {
        bytes.push(BIND_OPCODE_DONE);
    }
    Ok(bytes)
}

fn encode_symbol_table(plan: &NsldMachOArm64ShellLayoutPlanReport) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::with_capacity(plan.symbol_table_bytes);
    for (expected_index, symbol) in plan.symbols.iter().enumerate() {
        if symbol.symbol_table_index != expected_index {
            return Err(format!(
                "Mach-O symbol `{}` table index drift",
                symbol.symbol_id
            ));
        }
        bytes.extend_from_slice(&checked_u32(symbol.string_table_offset)?.to_le_bytes());
        match symbol.record_kind.as_str() {
            "external-defined" | "external-defined-alias" => {
                let section_id = symbol.shell_section_id.as_deref().ok_or_else(|| {
                    format!(
                        "Mach-O defined symbol `{}` has no section",
                        symbol.symbol_id
                    )
                })?;
                let section = plan
                    .sections
                    .iter()
                    .find(|section| section.section_id == section_id)
                    .ok_or_else(|| {
                        format!(
                            "Mach-O defined symbol `{}` references missing section",
                            symbol.symbol_id
                        )
                    })?;
                let ordinal = u8::try_from(section.section_ordinal).map_err(|_| {
                    format!(
                        "Mach-O defined symbol `{}` section ordinal exceeds u8",
                        symbol.symbol_id
                    )
                })?;
                let address = symbol.vm_address.ok_or_else(|| {
                    format!(
                        "Mach-O defined symbol `{}` has no VM address",
                        symbol.symbol_id
                    )
                })?;
                bytes.push(N_SECT | N_EXT);
                bytes.push(ordinal);
                bytes.extend_from_slice(&0u16.to_le_bytes());
                bytes.extend_from_slice(&address.to_le_bytes());
            }
            "external-absolute" | "external-absolute-alias" => {
                if symbol.shell_section_id.is_some() || symbol.source_image_offset.is_some() {
                    return Err(format!(
                        "Mach-O absolute symbol `{}` unexpectedly owns a section",
                        symbol.symbol_id
                    ));
                }
                let value = symbol.vm_address.ok_or_else(|| {
                    format!("Mach-O absolute symbol `{}` has no value", symbol.symbol_id)
                })?;
                bytes.push(N_ABS | N_EXT);
                bytes.push(0);
                bytes.extend_from_slice(&0u16.to_le_bytes());
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            "external-undefined" => {
                let ordinal = u8::try_from(symbol.dylib_ordinal.ok_or_else(|| {
                    format!(
                        "Mach-O undefined symbol `{}` has no dylib ordinal",
                        symbol.symbol_id
                    )
                })?)
                .map_err(|_| {
                    format!(
                        "Mach-O undefined symbol `{}` dylib ordinal exceeds u8",
                        symbol.symbol_id
                    )
                })?;
                bytes.push(N_EXT);
                bytes.push(0);
                bytes.extend_from_slice(&(u16::from(ordinal) << 8).to_le_bytes());
                bytes.extend_from_slice(&0u64.to_le_bytes());
            }
            other => {
                return Err(format!(
                    "Mach-O symbol `{}` has unregistered kind `{other}`",
                    symbol.symbol_id
                ));
            }
        }
    }
    Ok(bytes)
}

fn encode_indirect_symbols(plan: &NsldMachOArm64ShellLayoutPlanReport) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::with_capacity(plan.indirect_symbol_table_bytes);
    for indirect in &plan.indirect_symbols {
        let value = match (indirect.symbol_table_index, indirect.marker.as_deref()) {
            (Some(index), None) => checked_u32(index)?,
            (None, Some("local-absolute")) => INDIRECT_SYMBOL_LOCAL | INDIRECT_SYMBOL_ABS,
            _ => {
                return Err(format!(
                    "Mach-O indirect symbol `{}` has an invalid index/marker pair",
                    indirect.indirect_id
                ));
            }
        };
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    Ok(bytes)
}

fn encode_string_table(plan: &NsldMachOArm64ShellLayoutPlanReport) -> Result<Vec<u8>, String> {
    let mut bytes = vec![0];
    for symbol in &plan.symbols {
        if symbol.name.as_bytes().contains(&0) {
            return Err(format!(
                "Mach-O symbol `{}` name contains an embedded NUL",
                symbol.symbol_id
            ));
        }
        if symbol.string_table_offset != bytes.len() {
            return Err(format!(
                "Mach-O symbol `{}` string offset drift",
                symbol.symbol_id
            ));
        }
        bytes.extend_from_slice(symbol.name.as_bytes());
        bytes.push(0);
    }
    Ok(bytes)
}

fn validate_lengths(
    plan: &NsldMachOArm64ShellLayoutPlanReport,
    encoded: &EncodedShellLinkedit,
) -> Result<(), String> {
    let observed = [
        (
            "rebase",
            encoded.rebase_stream.len(),
            plan.rebase_stream_bytes,
        ),
        ("bind", encoded.bind_stream.len(), plan.bind_stream_bytes),
        (
            "symbol",
            encoded.symbol_table.len(),
            plan.symbol_table_bytes,
        ),
        (
            "indirect-symbol",
            encoded.indirect_symbol_table.len(),
            plan.indirect_symbol_table_bytes,
        ),
        (
            "string",
            encoded.string_table.len(),
            plan.string_table_bytes,
        ),
    ];
    for (kind, actual, expected) in observed {
        if actual != expected {
            return Err(format!(
                "Mach-O {kind} table size drift: plan={expected}, encoded={actual}"
            ));
        }
    }
    if plan.indirect_symbols.len() != plan.indirect_symbol_count {
        return Err("Mach-O indirect symbol count drift".to_owned());
    }
    Ok(())
}

fn encode_uleb(mut value: usize, bytes: &mut Vec<u8>) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn checked_u32(value: usize) -> Result<u32, String> {
    u32::try_from(value).map_err(|_| format!("Mach-O value {value} exceeds u32"))
}
