use crate::{
    final_executable_macho_shell_layout::{DYLINKER_PATH, SYSTEM_DYLIB_PATH},
    final_executable_macho_shell_uuid::macho_arm64_shell_uuid,
    reports::{
        NsldMachOArm64ShellLayoutPlanReport, NsldMachOArm64ShellLoadCommandPlan,
        NsldMachOArm64ShellSegmentPlan,
    },
};

const MH_MAGIC_64: u32 = 0xfeed_facf;
const CPU_TYPE_ARM64: u32 = 0x0100_000c;
const CPU_SUBTYPE_ARM64_ALL: u32 = 0;
const MH_EXECUTE: u32 = 2;
const MH_NOUNDEFS: u32 = 0x1;
const MH_DYLDLINK: u32 = 0x4;
const MH_TWOLEVEL: u32 = 0x80;
const MH_PIE: u32 = 0x20_0000;

const LC_SEGMENT_64: u32 = 0x19;
const LC_SYMTAB: u32 = 0x2;
const LC_DYSYMTAB: u32 = 0xb;
const LC_LOAD_DYLINKER: u32 = 0xe;
const LC_LOAD_DYLIB: u32 = 0xc;
const LC_DYLD_INFO_ONLY: u32 = 0x8000_0022;
const LC_MAIN: u32 = 0x8000_0028;
const LC_BUILD_VERSION: u32 = 0x32;
const LC_UUID: u32 = 0x1b;
const LC_CODE_SIGNATURE: u32 = 0x1d;
const PLATFORM_MACOS: u32 = 1;

pub(crate) struct EncodedShellCommands {
    pub(crate) header: Vec<u8>,
    pub(crate) load_commands: Vec<u8>,
}

pub(crate) fn encode_shell_header_and_commands(
    plan: &NsldMachOArm64ShellLayoutPlanReport,
    code_signature_payload_bytes: usize,
) -> Result<EncodedShellCommands, String> {
    if code_signature_payload_bytes == 0 {
        return Err("Mach-O code-signature payload must not be empty".to_owned());
    }
    let header = encode_header(plan)?;
    let mut load_commands = Vec::with_capacity(plan.load_command_size_bytes);
    for command in &plan.load_commands {
        let observed_offset = plan
            .header_size_bytes
            .checked_add(load_commands.len())
            .ok_or_else(|| "Mach-O command output offset overflows".to_owned())?;
        if command.command_offset != observed_offset {
            return Err(format!(
                "Mach-O command `{}` offset drift: plan={}, encoded={observed_offset}",
                command.command_id, command.command_offset
            ));
        }
        let encoded = encode_load_command(plan, command, code_signature_payload_bytes)?;
        if encoded.len() != command.command_size_bytes {
            return Err(format!(
                "Mach-O command `{}` size drift: plan={}, encoded={}",
                command.command_id,
                command.command_size_bytes,
                encoded.len()
            ));
        }
        load_commands.extend_from_slice(&encoded);
    }
    if load_commands.len() != plan.load_command_size_bytes
        || plan.load_commands.len() != plan.load_command_count
    {
        return Err("Mach-O encoded load-command coverage drift".to_owned());
    }
    Ok(EncodedShellCommands {
        header,
        load_commands,
    })
}

fn encode_header(plan: &NsldMachOArm64ShellLayoutPlanReport) -> Result<Vec<u8>, String> {
    if plan.header_size_bytes != 32 {
        return Err(format!(
            "Mach-O shell header size {} is unsupported",
            plan.header_size_bytes
        ));
    }
    let mut bytes = vec![0; plan.header_size_bytes];
    write_u32(&mut bytes, 0, MH_MAGIC_64)?;
    write_u32(&mut bytes, 4, CPU_TYPE_ARM64)?;
    write_u32(&mut bytes, 8, CPU_SUBTYPE_ARM64_ALL)?;
    write_u32(&mut bytes, 12, MH_EXECUTE)?;
    write_u32(&mut bytes, 16, checked_u32(plan.load_command_count)?)?;
    write_u32(&mut bytes, 20, checked_u32(plan.load_command_size_bytes)?)?;
    write_u32(
        &mut bytes,
        24,
        MH_NOUNDEFS | MH_DYLDLINK | MH_TWOLEVEL | MH_PIE,
    )?;
    write_u32(&mut bytes, 28, 0)?;
    Ok(bytes)
}

fn encode_load_command(
    plan: &NsldMachOArm64ShellLayoutPlanReport,
    command: &NsldMachOArm64ShellLoadCommandPlan,
    code_signature_payload_bytes: usize,
) -> Result<Vec<u8>, String> {
    let expected_value = command_value(&command.command_kind)?;
    if command.command_value != expected_value {
        return Err(format!(
            "Mach-O command `{}` value drift: plan=0x{:x}, expected=0x{expected_value:x}",
            command.command_id, command.command_value
        ));
    }
    match command.command_kind.as_str() {
        "segment-64" => encode_segment(plan, command, code_signature_payload_bytes),
        "dyld-info-only" => encode_dyld_info(plan, command),
        "symtab" => encode_symtab(plan, command),
        "dysymtab" => encode_dysymtab(plan, command),
        "load-dylinker" => encode_path_command(command, DYLINKER_PATH, 12),
        "load-dylib" => encode_dylib_command(command),
        "main" => encode_main(plan, command),
        "uuid" => encode_uuid(plan, command),
        "build-version" => encode_build_version(command),
        "code-signature" => encode_code_signature(plan, command, code_signature_payload_bytes),
        other => Err(format!(
            "Mach-O shell command kind `{other}` is unregistered"
        )),
    }
}

fn encode_segment(
    plan: &NsldMachOArm64ShellLayoutPlanReport,
    command: &NsldMachOArm64ShellLoadCommandPlan,
    code_signature_payload_bytes: usize,
) -> Result<Vec<u8>, String> {
    let segment_id = command
        .segment_id
        .as_deref()
        .ok_or_else(|| "Mach-O segment command has no segment id".to_owned())?;
    let segment = plan
        .segments
        .iter()
        .find(|segment| segment.segment_id == segment_id)
        .ok_or_else(|| format!("Mach-O command references missing segment `{segment_id}`"))?;
    let mut bytes = command_buffer(command)?;
    let (file_size_bytes, vm_size_bytes) =
        finalized_segment_sizes(plan, segment, code_signature_payload_bytes)?;
    write_fixed_name(&mut bytes, 8, &segment.segment_name)?;
    write_u64(&mut bytes, 24, segment.vm_address)?;
    write_u64(&mut bytes, 32, checked_u64(vm_size_bytes)?)?;
    write_u64(&mut bytes, 40, checked_u64(segment.file_offset)?)?;
    write_u64(&mut bytes, 48, checked_u64(file_size_bytes)?)?;
    write_u32(&mut bytes, 56, segment.max_protection)?;
    write_u32(&mut bytes, 60, segment.initial_protection)?;
    write_u32(&mut bytes, 64, checked_u32(segment.section_ids.len())?)?;
    write_u32(&mut bytes, 68, 0)?;
    encode_segment_sections(plan, segment, &mut bytes)?;
    Ok(bytes)
}

fn finalized_segment_sizes(
    plan: &NsldMachOArm64ShellLayoutPlanReport,
    segment: &NsldMachOArm64ShellSegmentPlan,
    code_signature_payload_bytes: usize,
) -> Result<(usize, usize), String> {
    if segment.segment_name != "__LINKEDIT" {
        return Ok((segment.file_size_bytes, segment.vm_size_bytes));
    }
    let signature_end = plan
        .code_signature_file_offset
        .checked_add(code_signature_payload_bytes)
        .ok_or_else(|| "Mach-O signed __LINKEDIT end overflows".to_owned())?;
    let file_size = signature_end
        .checked_sub(segment.file_offset)
        .ok_or_else(|| "Mach-O signed __LINKEDIT starts before its segment".to_owned())?;
    let vm_size = align_up(file_size, plan.page_size)?;
    Ok((file_size, vm_size))
}

fn encode_segment_sections(
    plan: &NsldMachOArm64ShellLayoutPlanReport,
    segment: &NsldMachOArm64ShellSegmentPlan,
    bytes: &mut [u8],
) -> Result<(), String> {
    let mut offset = 72usize;
    for section_id in &segment.section_ids {
        let section = plan
            .sections
            .iter()
            .find(|section| section.section_id == *section_id)
            .ok_or_else(|| format!("Mach-O segment references missing section `{section_id}`"))?;
        if section.segment_name != segment.segment_name {
            return Err(format!(
                "Mach-O section `{section_id}` segment identity drift"
            ));
        }
        write_fixed_name(bytes, offset, &section.section_name)?;
        write_fixed_name(bytes, offset + 16, &section.segment_name)?;
        write_u64(bytes, offset + 32, section.vm_address)?;
        write_u64(bytes, offset + 40, checked_u64(section.vm_size_bytes)?)?;
        write_u32(
            bytes,
            offset + 48,
            checked_u32(section.file_offset.unwrap_or(0))?,
        )?;
        if section.alignment == 0 || !section.alignment.is_power_of_two() {
            return Err(format!(
                "Mach-O section `{section_id}` has invalid alignment {}",
                section.alignment
            ));
        }
        write_u32(bytes, offset + 52, section.alignment.trailing_zeros())?;
        write_u32(bytes, offset + 56, 0)?;
        write_u32(bytes, offset + 60, 0)?;
        write_u32(bytes, offset + 64, section.flags)?;
        write_u32(bytes, offset + 68, section.reserved1)?;
        write_u32(bytes, offset + 72, section.reserved2)?;
        write_u32(bytes, offset + 76, 0)?;
        offset = offset
            .checked_add(80)
            .ok_or_else(|| "Mach-O section command offset overflows".to_owned())?;
    }
    if offset != bytes.len() {
        return Err(format!(
            "Mach-O segment `{}` command section span drift",
            segment.segment_id
        ));
    }
    Ok(())
}

fn encode_dyld_info(
    plan: &NsldMachOArm64ShellLayoutPlanReport,
    command: &NsldMachOArm64ShellLoadCommandPlan,
) -> Result<Vec<u8>, String> {
    let mut bytes = command_buffer(command)?;
    write_u32(&mut bytes, 8, checked_u32(plan.rebase_stream_offset)?)?;
    write_u32(&mut bytes, 12, checked_u32(plan.rebase_stream_bytes)?)?;
    write_u32(&mut bytes, 16, checked_u32(plan.bind_stream_offset)?)?;
    write_u32(&mut bytes, 20, checked_u32(plan.bind_stream_bytes)?)?;
    Ok(bytes)
}

fn encode_symtab(
    plan: &NsldMachOArm64ShellLayoutPlanReport,
    command: &NsldMachOArm64ShellLoadCommandPlan,
) -> Result<Vec<u8>, String> {
    let mut bytes = command_buffer(command)?;
    write_u32(&mut bytes, 8, checked_u32(plan.symbol_table_offset)?)?;
    write_u32(&mut bytes, 12, checked_u32(plan.symbols.len())?)?;
    write_u32(&mut bytes, 16, checked_u32(plan.string_table_offset)?)?;
    write_u32(&mut bytes, 20, checked_u32(plan.string_table_bytes)?)?;
    Ok(bytes)
}

fn encode_dysymtab(
    plan: &NsldMachOArm64ShellLayoutPlanReport,
    command: &NsldMachOArm64ShellLoadCommandPlan,
) -> Result<Vec<u8>, String> {
    let mut bytes = command_buffer(command)?;
    write_u32(&mut bytes, 8, 0)?;
    write_u32(&mut bytes, 12, 0)?;
    write_u32(&mut bytes, 16, 0)?;
    write_u32(&mut bytes, 20, checked_u32(plan.defined_symbol_count)?)?;
    write_u32(&mut bytes, 24, checked_u32(plan.defined_symbol_count)?)?;
    write_u32(&mut bytes, 28, checked_u32(plan.undefined_symbol_count)?)?;
    write_u32(
        &mut bytes,
        56,
        checked_u32(plan.indirect_symbol_table_offset)?,
    )?;
    write_u32(&mut bytes, 60, checked_u32(plan.indirect_symbol_count)?)?;
    Ok(bytes)
}

fn encode_path_command(
    command: &NsldMachOArm64ShellLoadCommandPlan,
    path: &str,
    path_offset: usize,
) -> Result<Vec<u8>, String> {
    let mut bytes = command_buffer(command)?;
    write_u32(&mut bytes, 8, checked_u32(path_offset)?)?;
    write_c_string(&mut bytes, path_offset, path)?;
    Ok(bytes)
}

fn encode_dylib_command(command: &NsldMachOArm64ShellLoadCommandPlan) -> Result<Vec<u8>, String> {
    let mut bytes = encode_path_command(command, SYSTEM_DYLIB_PATH, 24)?;
    write_u32(&mut bytes, 12, 0)?;
    write_u32(&mut bytes, 16, 0x0001_0000)?;
    write_u32(&mut bytes, 20, 0x0001_0000)?;
    Ok(bytes)
}

fn encode_main(
    plan: &NsldMachOArm64ShellLayoutPlanReport,
    command: &NsldMachOArm64ShellLoadCommandPlan,
) -> Result<Vec<u8>, String> {
    let mut bytes = command_buffer(command)?;
    write_u64(&mut bytes, 8, checked_u64(plan.entry_file_offset)?)?;
    write_u64(&mut bytes, 16, 0)?;
    Ok(bytes)
}

fn encode_build_version(command: &NsldMachOArm64ShellLoadCommandPlan) -> Result<Vec<u8>, String> {
    let mut bytes = command_buffer(command)?;
    write_u32(&mut bytes, 8, PLATFORM_MACOS)?;
    write_u32(&mut bytes, 12, 0)?;
    write_u32(&mut bytes, 16, 0)?;
    write_u32(&mut bytes, 20, 0)?;
    Ok(bytes)
}

fn encode_uuid(
    plan: &NsldMachOArm64ShellLayoutPlanReport,
    command: &NsldMachOArm64ShellLoadCommandPlan,
) -> Result<Vec<u8>, String> {
    let mut bytes = command_buffer(command)?;
    write_bytes(
        &mut bytes,
        8,
        &macho_arm64_shell_uuid(&plan.plan_hash),
        "UUID",
    )?;
    Ok(bytes)
}

fn encode_code_signature(
    plan: &NsldMachOArm64ShellLayoutPlanReport,
    command: &NsldMachOArm64ShellLoadCommandPlan,
    code_signature_payload_bytes: usize,
) -> Result<Vec<u8>, String> {
    if command.status != "payload-pending" {
        return Err("Mach-O code-signature command lost pending status".to_owned());
    }
    let mut bytes = command_buffer(command)?;
    write_u32(&mut bytes, 8, checked_u32(plan.code_signature_file_offset)?)?;
    write_u32(&mut bytes, 12, checked_u32(code_signature_payload_bytes)?)?;
    Ok(bytes)
}

fn command_buffer(command: &NsldMachOArm64ShellLoadCommandPlan) -> Result<Vec<u8>, String> {
    if command.command_size_bytes < 8 {
        return Err(format!(
            "Mach-O command `{}` is shorter than its header",
            command.command_id
        ));
    }
    let mut bytes = vec![0; command.command_size_bytes];
    write_u32(&mut bytes, 0, command.command_value)?;
    write_u32(&mut bytes, 4, checked_u32(command.command_size_bytes)?)?;
    Ok(bytes)
}

fn command_value(kind: &str) -> Result<u32, String> {
    match kind {
        "segment-64" => Ok(LC_SEGMENT_64),
        "dyld-info-only" => Ok(LC_DYLD_INFO_ONLY),
        "symtab" => Ok(LC_SYMTAB),
        "dysymtab" => Ok(LC_DYSYMTAB),
        "load-dylinker" => Ok(LC_LOAD_DYLINKER),
        "load-dylib" => Ok(LC_LOAD_DYLIB),
        "main" => Ok(LC_MAIN),
        "build-version" => Ok(LC_BUILD_VERSION),
        "uuid" => Ok(LC_UUID),
        "code-signature" => Ok(LC_CODE_SIGNATURE),
        other => Err(format!("Mach-O command kind `{other}` has no opcode")),
    }
}

fn write_fixed_name(bytes: &mut [u8], offset: usize, value: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > 16 || !value.is_ascii() {
        return Err(format!("Mach-O fixed name `{value}` is invalid"));
    }
    let range = checked_range(offset, 16, bytes.len(), "fixed name")?;
    bytes[range.clone()].fill(0);
    bytes[range.start..range.start + value.len()].copy_from_slice(value.as_bytes());
    Ok(())
}

fn write_c_string(bytes: &mut [u8], offset: usize, value: &str) -> Result<(), String> {
    if value.as_bytes().contains(&0) {
        return Err("Mach-O path contains an embedded NUL".to_owned());
    }
    let size = value
        .len()
        .checked_add(1)
        .ok_or_else(|| "Mach-O path size overflows".to_owned())?;
    let range = checked_range(offset, size, bytes.len(), "path")?;
    bytes[range.start..range.end - 1].copy_from_slice(value.as_bytes());
    bytes[range.end - 1] = 0;
    Ok(())
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) -> Result<(), String> {
    write_bytes(bytes, offset, &value.to_le_bytes(), "u32")
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64) -> Result<(), String> {
    write_bytes(bytes, offset, &value.to_le_bytes(), "u64")
}

fn write_bytes(bytes: &mut [u8], offset: usize, value: &[u8], label: &str) -> Result<(), String> {
    let range = checked_range(offset, value.len(), bytes.len(), label)?;
    bytes[range].copy_from_slice(value);
    Ok(())
}

fn checked_range(
    offset: usize,
    size: usize,
    limit: usize,
    label: &str,
) -> Result<std::ops::Range<usize>, String> {
    let end = offset
        .checked_add(size)
        .ok_or_else(|| format!("Mach-O {label} range overflows"))?;
    if end > limit {
        return Err(format!(
            "Mach-O {label} range {offset}..{end} exceeds {limit}"
        ));
    }
    Ok(offset..end)
}

fn checked_u32(value: usize) -> Result<u32, String> {
    u32::try_from(value).map_err(|_| format!("Mach-O value {value} exceeds u32"))
}

fn checked_u64(value: usize) -> Result<u64, String> {
    u64::try_from(value).map_err(|_| format!("Mach-O value {value} exceeds u64"))
}

fn align_up(value: usize, alignment: usize) -> Result<usize, String> {
    if alignment == 0 || !alignment.is_power_of_two() {
        return Err(format!("Mach-O alignment {alignment} is invalid"));
    }
    value
        .checked_add(alignment - 1)
        .map(|value| value & !(alignment - 1))
        .ok_or_else(|| "Mach-O aligned value overflows".to_owned())
}
