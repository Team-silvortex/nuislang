use std::collections::BTreeSet;

const MACH_O_64_HEADER_SIZE: usize = 32;
const MACH_O_64_LE_MAGIC: [u8; 4] = [0xcf, 0xfa, 0xed, 0xfe];
const MACH_O_CPU_TYPE_ARM64: u32 = 0x0100_000c;
const MACH_O_FILE_TYPE_OBJECT: u32 = 1;
const MACH_O_LOAD_COMMAND_SEGMENT_64: u32 = 0x19;
const REQUIRED_HOST_OBJECT_ROLES: [&str; 2] = ["program-llvm", "runtime-shim"];

pub(crate) fn validate_macho_host_object_handoff(
    artifact: &nuisc::aot::NuisCompiledArtifact,
    plan: &nuisc::linker::LinkPlan,
) -> Result<(), String> {
    if artifact.host_objects.is_empty() {
        return Err("compiled artifact has no relocatable host objects".to_owned());
    }
    if artifact.host_objects.len() != plan.compiled_artifact.host_objects.len() {
        return Err(format!(
            "host object count mismatch: plan={}, artifact={}",
            plan.compiled_artifact.host_objects.len(),
            artifact.host_objects.len()
        ));
    }

    let mut object_ids = BTreeSet::new();
    for object in &artifact.host_objects {
        if !object_ids.insert(object.object_id.as_str()) {
            return Err(format!("duplicate host object id `{}`", object.object_id));
        }
        if canonical_object_format(&object.object_format)
            != canonical_object_format(&plan.cpu_target.object_format)
        {
            return Err(format!(
                "host object `{}` format mismatch: object={}, target={}",
                object.object_id, object.object_format, plan.cpu_target.object_format
            ));
        }
        validate_plan_identity(object, plan)?;
        validate_thin_macho_arm64_object(&object.bytes)
            .map_err(|error| format!("host object `{}`: {error}", object.object_id))?;
    }

    for required_role in REQUIRED_HOST_OBJECT_ROLES {
        let count = artifact
            .host_objects
            .iter()
            .filter(|object| object.role == required_role)
            .count();
        if count != 1 {
            return Err(format!(
                "required host object role `{required_role}` must appear exactly once; found {count}"
            ));
        }
    }
    Ok(())
}

fn validate_plan_identity(
    object: &nuisc::aot::NuisCompiledArtifactHostObject,
    plan: &nuisc::linker::LinkPlan,
) -> Result<(), String> {
    let planned = plan
        .compiled_artifact
        .host_objects
        .iter()
        .find(|planned| planned.object_id == object.object_id)
        .ok_or_else(|| {
            format!(
                "host object `{}` is absent from the link plan",
                object.object_id
            )
        })?;
    let actual_hash = crate::fnv1a64_hex(&object.bytes);
    if planned.role != object.role
        || canonical_object_format(&planned.object_format)
            != canonical_object_format(&object.object_format)
        || planned.bytes != object.bytes.len()
        || planned.content_hash != actual_hash
    {
        return Err(format!(
            "host object `{}` identity drift: plan role={} format={} bytes={} hash={}, artifact role={} format={} bytes={} hash={}",
            object.object_id,
            planned.role,
            planned.object_format,
            planned.bytes,
            planned.content_hash,
            object.role,
            object.object_format,
            object.bytes.len(),
            actual_hash
        ));
    }
    Ok(())
}

fn validate_thin_macho_arm64_object(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() < MACH_O_64_HEADER_SIZE {
        return Err(format!(
            "Mach-O object is truncated: expected at least {MACH_O_64_HEADER_SIZE} bytes, found {}",
            bytes.len()
        ));
    }
    if bytes[..4] != MACH_O_64_LE_MAGIC {
        return Err("Mach-O object magic is not little-endian MH_MAGIC_64".to_owned());
    }
    let cpu_type = read_u32_le(bytes, 4)?;
    if cpu_type != MACH_O_CPU_TYPE_ARM64 {
        return Err(format!(
            "Mach-O object CPU type is 0x{cpu_type:08x}; expected ARM64"
        ));
    }
    let file_type = read_u32_le(bytes, 12)?;
    if file_type != MACH_O_FILE_TYPE_OBJECT {
        return Err(format!(
            "Mach-O file type is {file_type}; expected MH_OBJECT"
        ));
    }
    validate_load_commands(bytes)
}

fn validate_load_commands(bytes: &[u8]) -> Result<(), String> {
    let command_count = read_u32_le(bytes, 16)? as usize;
    let command_span = read_u32_le(bytes, 20)? as usize;
    let command_end = MACH_O_64_HEADER_SIZE
        .checked_add(command_span)
        .ok_or_else(|| "Mach-O object load-command span overflows address space".to_owned())?;
    if command_end > bytes.len() {
        return Err(format!(
            "Mach-O object load-command span ends at {command_end}, beyond object size {}",
            bytes.len()
        ));
    }

    let mut cursor = MACH_O_64_HEADER_SIZE;
    let mut segment_present = false;
    for index in 0..command_count {
        if cursor.checked_add(8).is_none_or(|end| end > command_end) {
            return Err(format!(
                "Mach-O object load command {index} header is truncated"
            ));
        }
        let command = read_u32_le(bytes, cursor)?;
        let command_size = read_u32_le(bytes, cursor + 4)? as usize;
        if command_size < 8 || command_size % 4 != 0 {
            return Err(format!(
                "Mach-O object load command {index} has invalid size {command_size}"
            ));
        }
        if command == MACH_O_LOAD_COMMAND_SEGMENT_64 {
            if command_size < 72 {
                return Err(format!(
                    "Mach-O object LC_SEGMENT_64 command {index} is shorter than 72 bytes"
                ));
            }
            segment_present = true;
        }
        cursor = cursor
            .checked_add(command_size)
            .filter(|end| *end <= command_end)
            .ok_or_else(|| format!("Mach-O object load command {index} exceeds declared span"))?;
    }
    if cursor != command_end {
        return Err(format!(
            "Mach-O object load-command count consumes {} bytes, declared span is {command_span}",
            cursor.saturating_sub(MACH_O_64_HEADER_SIZE)
        ));
    }
    if !segment_present {
        return Err("Mach-O object has no LC_SEGMENT_64 command".to_owned());
    }
    Ok(())
}

fn canonical_object_format(object_format: &str) -> &str {
    match object_format.trim().to_ascii_lowercase().as_str() {
        "mach-o" | "macho" => "mach-o",
        "elf" => "elf",
        "coff" | "pe" | "pe-coff" | "pe/coff" => "pe-coff",
        _ => "unknown",
    }
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let raw: [u8; 4] = bytes
        .get(offset..offset.saturating_add(4))
        .ok_or_else(|| format!("Mach-O object u32 at offset {offset} is truncated"))?
        .try_into()
        .map_err(|_| format!("Mach-O object u32 at offset {offset} is malformed"))?;
    Ok(u32::from_le_bytes(raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_minimal_arm64_relocatable_object() {
        assert_eq!(
            validate_thin_macho_arm64_object(&minimal_arm64_object()),
            Ok(())
        );
    }

    #[test]
    fn rejects_executable_file_type_at_object_boundary() {
        let mut bytes = minimal_arm64_object();
        bytes[12..16].copy_from_slice(&2u32.to_le_bytes());

        assert!(validate_thin_macho_arm64_object(&bytes)
            .unwrap_err()
            .contains("expected MH_OBJECT"));
    }

    fn minimal_arm64_object() -> Vec<u8> {
        let mut bytes = vec![0u8; 104];
        bytes[..4].copy_from_slice(&MACH_O_64_LE_MAGIC);
        bytes[4..8].copy_from_slice(&MACH_O_CPU_TYPE_ARM64.to_le_bytes());
        bytes[12..16].copy_from_slice(&MACH_O_FILE_TYPE_OBJECT.to_le_bytes());
        bytes[16..20].copy_from_slice(&1u32.to_le_bytes());
        bytes[20..24].copy_from_slice(&72u32.to_le_bytes());
        bytes[32..36].copy_from_slice(&MACH_O_LOAD_COMMAND_SEGMENT_64.to_le_bytes());
        bytes[36..40].copy_from_slice(&72u32.to_le_bytes());
        bytes
    }
}
