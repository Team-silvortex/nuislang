use crate::reports::NsldMachOArm64ShellLayoutPlanReport;
use sha2::{Digest, Sha256};

pub(crate) const MACHO_ARM64_AD_HOC_SIGNATURE_CONTRACT: &str =
    "nuis-nsld-macho-arm64-ad-hoc-signature-v1";
pub(crate) const CSMAGIC_CODEDIRECTORY: u32 = 0xfade_0c02;
pub(crate) const CSMAGIC_EMBEDDED_SIGNATURE: u32 = 0xfade_0cc0;
pub(crate) const CSSLOT_CODEDIRECTORY: u32 = 0;
pub(crate) const CODE_DIRECTORY_VERSION: u32 = 0x0002_0400;
pub(crate) const CODE_SIGNATURE_FLAGS: u32 = 0x0002_0002;
pub(crate) const HASH_TYPE_SHA256: u8 = 2;
pub(crate) const HASH_SIZE_BYTES: usize = 32;
pub(crate) const CODE_PAGE_SIZE_BYTES: usize = 4096;
pub(crate) const CODE_PAGE_SIZE_EXPONENT: u8 = 12;
pub(crate) const CODE_DIRECTORY_FIXED_BYTES: usize = 88;
pub(crate) const SUPERBLOB_INDEX_BYTES: usize = 20;
const SIGNATURE_ALIGNMENT: usize = 16;
const EXEC_SEG_MAIN_BINARY: u64 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MachOArm64AdHocSignaturePlan {
    pub(crate) identifier: String,
    pub(crate) code_limit: usize,
    pub(crate) code_slot_count: usize,
    pub(crate) code_directory_offset: usize,
    pub(crate) identifier_offset: usize,
    pub(crate) hash_offset: usize,
    pub(crate) code_directory_bytes: usize,
    pub(crate) signature_blob_bytes: usize,
    pub(crate) signature_payload_bytes: usize,
    pub(crate) exec_segment_base: u64,
    pub(crate) exec_segment_limit: u64,
    pub(crate) exec_segment_flags: u64,
}

pub(crate) fn plan_macho_arm64_ad_hoc_signature(
    shell: &NsldMachOArm64ShellLayoutPlanReport,
) -> Result<MachOArm64AdHocSignaturePlan, String> {
    let code_limit = shell.code_signature_file_offset;
    u32::try_from(code_limit)
        .map_err(|_| "Mach-O ad-hoc signature code limit exceeds u32".to_owned())?;
    let code_slot_count = code_limit
        .checked_add(CODE_PAGE_SIZE_BYTES - 1)
        .map(|value| value / CODE_PAGE_SIZE_BYTES)
        .ok_or_else(|| "Mach-O ad-hoc signature slot count overflows".to_owned())?;
    if code_slot_count == 0 {
        return Err("Mach-O ad-hoc signature has no signed code slots".to_owned());
    }
    let identifier = signature_identifier(&shell.entry_symbol)?;
    let identifier_offset = CODE_DIRECTORY_FIXED_BYTES;
    let hash_offset = identifier_offset
        .checked_add(identifier.len())
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| "Mach-O ad-hoc signature identifier span overflows".to_owned())?;
    let code_directory_bytes = code_slot_count
        .checked_mul(HASH_SIZE_BYTES)
        .and_then(|slots| hash_offset.checked_add(slots))
        .ok_or_else(|| "Mach-O ad-hoc CodeDirectory span overflows".to_owned())?;
    let signature_blob_bytes = SUPERBLOB_INDEX_BYTES
        .checked_add(code_directory_bytes)
        .ok_or_else(|| "Mach-O ad-hoc SuperBlob span overflows".to_owned())?;
    let signature_payload_bytes = align_up(signature_blob_bytes, SIGNATURE_ALIGNMENT)?;
    u32::try_from(signature_payload_bytes)
        .map_err(|_| "Mach-O ad-hoc signature payload exceeds u32".to_owned())?;
    let executable = shell
        .segments
        .iter()
        .find(|segment| segment.segment_name == "__TEXT" && segment.initial_protection & 4 != 0)
        .ok_or_else(|| "Mach-O ad-hoc signature cannot locate executable __TEXT".to_owned())?;
    let exec_segment_base = u64::try_from(executable.file_offset)
        .map_err(|_| "Mach-O executable segment base exceeds u64".to_owned())?;
    let exec_segment_limit = u64::try_from(executable.file_size_bytes)
        .map_err(|_| "Mach-O executable segment limit exceeds u64".to_owned())?;
    let exec_end = executable
        .file_offset
        .checked_add(executable.file_size_bytes)
        .ok_or_else(|| "Mach-O executable segment span overflows".to_owned())?;
    if executable.file_size_bytes == 0 || exec_end > code_limit {
        return Err("Mach-O executable segment exceeds the signed range".to_owned());
    }
    Ok(MachOArm64AdHocSignaturePlan {
        identifier,
        code_limit,
        code_slot_count,
        code_directory_offset: SUPERBLOB_INDEX_BYTES,
        identifier_offset,
        hash_offset,
        code_directory_bytes,
        signature_blob_bytes,
        signature_payload_bytes,
        exec_segment_base,
        exec_segment_limit,
        exec_segment_flags: EXEC_SEG_MAIN_BINARY,
    })
}

pub(crate) fn encode_macho_arm64_ad_hoc_signature(
    signed_content: &[u8],
    plan: &MachOArm64AdHocSignaturePlan,
) -> Result<Vec<u8>, String> {
    if signed_content.len() != plan.code_limit {
        return Err(format!(
            "Mach-O ad-hoc signed content drift: plan={}, bytes={}",
            plan.code_limit,
            signed_content.len()
        ));
    }
    let mut bytes = vec![0; plan.signature_payload_bytes];
    write_be_u32(&mut bytes, 0, CSMAGIC_EMBEDDED_SIGNATURE)?;
    write_be_u32(&mut bytes, 4, checked_u32(plan.signature_blob_bytes)?)?;
    write_be_u32(&mut bytes, 8, 1)?;
    write_be_u32(&mut bytes, 12, CSSLOT_CODEDIRECTORY)?;
    write_be_u32(&mut bytes, 16, checked_u32(plan.code_directory_offset)?)?;

    let base = plan.code_directory_offset;
    write_be_u32(&mut bytes, base, CSMAGIC_CODEDIRECTORY)?;
    write_be_u32(
        &mut bytes,
        base + 4,
        checked_u32(plan.code_directory_bytes)?,
    )?;
    write_be_u32(&mut bytes, base + 8, CODE_DIRECTORY_VERSION)?;
    write_be_u32(&mut bytes, base + 12, CODE_SIGNATURE_FLAGS)?;
    write_be_u32(&mut bytes, base + 16, checked_u32(plan.hash_offset)?)?;
    write_be_u32(&mut bytes, base + 20, checked_u32(plan.identifier_offset)?)?;
    write_be_u32(&mut bytes, base + 24, 0)?;
    write_be_u32(&mut bytes, base + 28, checked_u32(plan.code_slot_count)?)?;
    write_be_u32(&mut bytes, base + 32, checked_u32(plan.code_limit)?)?;
    write_u8(&mut bytes, base + 36, HASH_SIZE_BYTES as u8)?;
    write_u8(&mut bytes, base + 37, HASH_TYPE_SHA256)?;
    write_u8(&mut bytes, base + 38, 0)?;
    write_u8(&mut bytes, base + 39, CODE_PAGE_SIZE_EXPONENT)?;
    write_be_u32(&mut bytes, base + 40, 0)?;
    write_be_u32(&mut bytes, base + 44, 0)?;
    write_be_u32(&mut bytes, base + 48, 0)?;
    write_be_u32(&mut bytes, base + 52, 0)?;
    write_be_u64(&mut bytes, base + 56, 0)?;
    write_be_u64(&mut bytes, base + 64, plan.exec_segment_base)?;
    write_be_u64(&mut bytes, base + 72, plan.exec_segment_limit)?;
    write_be_u64(&mut bytes, base + 80, plan.exec_segment_flags)?;
    write_bytes(
        &mut bytes,
        base + plan.identifier_offset,
        plan.identifier.as_bytes(),
        "identifier",
    )?;
    write_u8(
        &mut bytes,
        base + plan.identifier_offset + plan.identifier.len(),
        0,
    )?;

    for slot_index in 0..plan.code_slot_count {
        let start = slot_index
            .checked_mul(CODE_PAGE_SIZE_BYTES)
            .ok_or_else(|| "Mach-O code slot offset overflows".to_owned())?;
        let end = start
            .checked_add(CODE_PAGE_SIZE_BYTES)
            .map(|end| end.min(plan.code_limit))
            .ok_or_else(|| "Mach-O code slot end overflows".to_owned())?;
        let digest = sha256_bytes(&signed_content[start..end]);
        let hash_offset = base
            .checked_add(plan.hash_offset)
            .and_then(|offset| offset.checked_add(slot_index * HASH_SIZE_BYTES))
            .ok_or_else(|| "Mach-O code slot hash offset overflows".to_owned())?;
        write_bytes(&mut bytes, hash_offset, &digest, "code-slot hash")?;
    }
    Ok(bytes)
}

pub(crate) fn sha256_bytes(bytes: &[u8]) -> [u8; HASH_SIZE_BYTES] {
    Sha256::digest(bytes).into()
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    hex_bytes(&sha256_bytes(bytes))
}

pub(crate) fn hex_bytes(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(out, "{byte:02x}").unwrap();
    }
    out
}

fn signature_identifier(entry_symbol: &str) -> Result<String, String> {
    let normalized = entry_symbol
        .trim_start_matches('_')
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    if normalized.is_empty() || normalized.as_bytes().contains(&0) {
        return Err("Mach-O ad-hoc signature entry identifier is invalid".to_owned());
    }
    Ok(format!("org.nuislang.nsld.private.{normalized}"))
}

fn write_be_u32(bytes: &mut [u8], offset: usize, value: u32) -> Result<(), String> {
    write_bytes(bytes, offset, &value.to_be_bytes(), "big-endian u32")
}

fn write_be_u64(bytes: &mut [u8], offset: usize, value: u64) -> Result<(), String> {
    write_bytes(bytes, offset, &value.to_be_bytes(), "big-endian u64")
}

fn write_u8(bytes: &mut [u8], offset: usize, value: u8) -> Result<(), String> {
    let slot = bytes
        .get_mut(offset)
        .ok_or_else(|| format!("Mach-O signature byte offset {offset} exceeds payload"))?;
    *slot = value;
    Ok(())
}

fn write_bytes(bytes: &mut [u8], offset: usize, value: &[u8], label: &str) -> Result<(), String> {
    let end = offset
        .checked_add(value.len())
        .ok_or_else(|| format!("Mach-O signature {label} span overflows"))?;
    let target = bytes
        .get_mut(offset..end)
        .ok_or_else(|| format!("Mach-O signature {label} exceeds payload"))?;
    target.copy_from_slice(value);
    Ok(())
}

fn checked_u32(value: usize) -> Result<u32, String> {
    u32::try_from(value).map_err(|_| format!("Mach-O signature value {value} exceeds u32"))
}

fn align_up(value: usize, alignment: usize) -> Result<usize, String> {
    value
        .checked_add(alignment - 1)
        .map(|value| value & !(alignment - 1))
        .ok_or_else(|| "Mach-O signature alignment overflows".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_standard_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
