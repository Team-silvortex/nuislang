use crate::{
    final_executable_macho_shell_signature::{
        hex_bytes, sha256_bytes, sha256_hex, MachOArm64AdHocSignaturePlan,
        CODE_DIRECTORY_FIXED_BYTES, CODE_DIRECTORY_VERSION, CODE_PAGE_SIZE_BYTES,
        CODE_PAGE_SIZE_EXPONENT, CODE_SIGNATURE_FLAGS, CSMAGIC_CODEDIRECTORY,
        CSMAGIC_EMBEDDED_SIGNATURE, CSSLOT_CODEDIRECTORY, HASH_SIZE_BYTES, HASH_TYPE_SHA256,
        MACHO_ARM64_AD_HOC_SIGNATURE_CONTRACT,
    },
    final_executable_macho_shell_uuid::macho_arm64_shell_uuid,
    reports::{
        NsldMachOArm64CodeSignatureReport, NsldMachOArm64CodeSignatureSlotAudit,
        NsldMachOArm64ShellLayoutPlanReport,
    },
};
use std::fmt::Write as _;

pub(crate) const MACHO_ARM64_SIGNED_IMAGE_VALIDATION_CONTRACT: &str =
    "nuis-nsld-macho-arm64-signed-image-validation-v1";
pub(crate) const MACHO_ARM64_PUBLICATION_ELIGIBILITY_CONTRACT: &str =
    "nuis-nsld-macho-arm64-publication-eligibility-v1";

const MH_MAGIC_64: u32 = 0xfeed_facf;
const MH_EXECUTE: u32 = 2;
const MACHO_HEADER_BYTES: usize = 32;
const LC_SEGMENT_64: u32 = 0x19;
const LC_CODE_SIGNATURE: u32 = 0x1d;
const LC_UUID: u32 = 0x1b;
const SEGMENT_COMMAND_64_BYTES: usize = 72;
const SECTION_64_BYTES: usize = 80;

#[derive(Debug)]
struct LoadCommandValidation {
    count: usize,
    bytes: usize,
    signature_offset: usize,
    signature_bytes: usize,
    linkedit_offset: usize,
    linkedit_bytes: usize,
}

pub(crate) fn validate_macho_arm64_signed_shell_image(
    bytes: &[u8],
    shell: &NsldMachOArm64ShellLayoutPlanReport,
    plan: &MachOArm64AdHocSignaturePlan,
) -> Result<NsldMachOArm64CodeSignatureReport, String> {
    validate_plan_envelope(bytes, shell, plan)?;
    let commands = validate_load_commands(bytes, shell)?;
    if commands.signature_offset != plan.code_limit
        || commands.signature_bytes != plan.signature_payload_bytes
        || commands
            .signature_offset
            .checked_add(commands.signature_bytes)
            != Some(bytes.len())
    {
        return Err("Mach-O code-signature load-command span drift".to_owned());
    }
    let linkedit_end = commands
        .linkedit_offset
        .checked_add(commands.linkedit_bytes)
        .ok_or_else(|| "Mach-O signed __LINKEDIT span overflows".to_owned())?;
    if commands.linkedit_offset > commands.signature_offset || linkedit_end != bytes.len() {
        return Err("Mach-O signed __LINKEDIT does not cover the signature".to_owned());
    }

    let payload = checked_slice(
        bytes,
        commands.signature_offset,
        commands.signature_bytes,
        "code-signature payload",
    )?;
    let parsed = validate_code_directory(bytes, payload, plan)?;
    let validation_status = "signed-private-image-structurally-valid";
    let publication_eligibility_status = "blocked-independent-os-load-validation-pending";
    let publication_blockers = vec!["independent-os-load-validation-pending".to_owned()];
    let validation_ledger_hash = validation_ledger_hash(
        plan,
        &commands,
        &parsed,
        validation_status,
        publication_eligibility_status,
        &publication_blockers,
    );
    Ok(NsldMachOArm64CodeSignatureReport {
        contract: MACHO_ARM64_AD_HOC_SIGNATURE_CONTRACT.to_owned(),
        status: "ad-hoc-payload-validated".to_owned(),
        identifier: parsed.identifier,
        code_directory_version: CODE_DIRECTORY_VERSION,
        flags: CODE_SIGNATURE_FLAGS,
        hash_type: "sha256".to_owned(),
        hash_size_bytes: HASH_SIZE_BYTES,
        page_size_bytes: CODE_PAGE_SIZE_BYTES,
        code_limit: plan.code_limit,
        code_slot_count: plan.code_slot_count,
        verified_code_slot_count: parsed.slots.len(),
        signature_file_offset: commands.signature_offset,
        signature_blob_bytes: plan.signature_blob_bytes,
        signature_payload_bytes: commands.signature_bytes,
        signed_content_sha256: sha256_hex(&bytes[..plan.code_limit]),
        code_directory_sha256: parsed.code_directory_sha256,
        cdhash: parsed.cdhash,
        signature_payload_sha256: sha256_hex(payload),
        load_command_count: shell.load_command_count,
        verified_load_command_count: commands.count,
        load_command_bytes: commands.bytes,
        linkedit_covers_signature: true,
        signed_ranges_valid: true,
        padding_valid: true,
        validation_contract: MACHO_ARM64_SIGNED_IMAGE_VALIDATION_CONTRACT.to_owned(),
        validation_status: validation_status.to_owned(),
        publication_eligibility_contract: MACHO_ARM64_PUBLICATION_ELIGIBILITY_CONTRACT.to_owned(),
        publication_eligibility_status: publication_eligibility_status.to_owned(),
        publication_eligible: false,
        publication_blockers,
        validation_ledger_hash,
        slots: parsed.slots,
    })
}

fn validate_plan_envelope(
    bytes: &[u8],
    shell: &NsldMachOArm64ShellLayoutPlanReport,
    plan: &MachOArm64AdHocSignaturePlan,
) -> Result<(), String> {
    if shell.header_size_bytes != MACHO_HEADER_BYTES
        || plan.code_limit != shell.code_signature_file_offset
        || plan.signature_payload_bytes == 0
        || plan.code_slot_count == 0
        || bytes.len()
            != plan
                .code_limit
                .checked_add(plan.signature_payload_bytes)
                .ok_or_else(|| "Mach-O signed image span overflows".to_owned())?
    {
        return Err("Mach-O signed image validation envelope drift".to_owned());
    }
    Ok(())
}

fn validate_load_commands(
    bytes: &[u8],
    shell: &NsldMachOArm64ShellLayoutPlanReport,
) -> Result<LoadCommandValidation, String> {
    if read_le_u32(bytes, 0, "magic")? != MH_MAGIC_64
        || read_le_u32(bytes, 12, "file type")? != MH_EXECUTE
    {
        return Err("Mach-O signed image header is not an executable Mach-O 64 image".to_owned());
    }
    let command_count = usize::try_from(read_le_u32(bytes, 16, "command count")?)
        .map_err(|_| "Mach-O command count exceeds usize".to_owned())?;
    let command_bytes = usize::try_from(read_le_u32(bytes, 20, "command bytes")?)
        .map_err(|_| "Mach-O command bytes exceed usize".to_owned())?;
    if command_count != shell.load_command_count || command_bytes != shell.load_command_size_bytes {
        return Err("Mach-O signed image load-command header drift".to_owned());
    }
    let command_end = MACHO_HEADER_BYTES
        .checked_add(command_bytes)
        .ok_or_else(|| "Mach-O load-command span overflows".to_owned())?;
    if command_end > bytes.len() {
        return Err("Mach-O load-command span exceeds image".to_owned());
    }

    let mut cursor = MACHO_HEADER_BYTES;
    let mut signature = None;
    let mut linkedit = None;
    let mut uuid_seen = false;
    for index in 0..command_count {
        let command = read_le_u32(bytes, cursor, "load-command opcode")?;
        let size = usize::try_from(read_le_u32(bytes, cursor + 4, "load-command size")?)
            .map_err(|_| "Mach-O load-command size exceeds usize".to_owned())?;
        if size < 8 || !size.is_multiple_of(8) {
            return Err(format!(
                "Mach-O load command {index} has an invalid boundary"
            ));
        }
        let end = cursor
            .checked_add(size)
            .ok_or_else(|| "Mach-O load-command boundary overflows".to_owned())?;
        if end > command_end {
            return Err(format!(
                "Mach-O load command {index} exceeds the command span"
            ));
        }
        match command {
            LC_SEGMENT_64 => {
                validate_segment_command(bytes, cursor, size, index)?;
                if read_fixed_name(bytes, cursor + 8, 16, "segment name")? == "__LINKEDIT" {
                    if linkedit.is_some() {
                        return Err(
                            "Mach-O image contains duplicate __LINKEDIT segments".to_owned()
                        );
                    }
                    linkedit = Some((
                        read_le_usize_u64(bytes, cursor + 40, "__LINKEDIT file offset")?,
                        read_le_usize_u64(bytes, cursor + 48, "__LINKEDIT file size")?,
                    ));
                }
            }
            LC_CODE_SIGNATURE => {
                if size != 16 || signature.is_some() {
                    return Err("Mach-O image has an invalid code-signature command".to_owned());
                }
                signature = Some((
                    read_le_usize_u32(bytes, cursor + 8, "signature file offset")?,
                    read_le_usize_u32(bytes, cursor + 12, "signature payload size")?,
                ));
            }
            LC_UUID => {
                if size != 24 || uuid_seen {
                    return Err("Mach-O image has an invalid UUID command".to_owned());
                }
                let actual = checked_slice(bytes, cursor + 8, 16, "image UUID")?;
                if actual != macho_arm64_shell_uuid(&shell.plan_hash) {
                    return Err("Mach-O image UUID drift".to_owned());
                }
                uuid_seen = true;
            }
            _ => {}
        }
        cursor = end;
    }
    if cursor != command_end {
        return Err("Mach-O load-command coverage is incomplete".to_owned());
    }
    let (signature_offset, signature_bytes) =
        signature.ok_or_else(|| "Mach-O image has no code-signature command".to_owned())?;
    if !uuid_seen {
        return Err("Mach-O image has no UUID command".to_owned());
    }
    let (linkedit_offset, linkedit_bytes) =
        linkedit.ok_or_else(|| "Mach-O image has no __LINKEDIT segment".to_owned())?;
    Ok(LoadCommandValidation {
        count: command_count,
        bytes: command_bytes,
        signature_offset,
        signature_bytes,
        linkedit_offset,
        linkedit_bytes,
    })
}

fn validate_segment_command(
    bytes: &[u8],
    offset: usize,
    size: usize,
    index: usize,
) -> Result<(), String> {
    if size < SEGMENT_COMMAND_64_BYTES {
        return Err(format!("Mach-O segment command {index} is truncated"));
    }
    let section_count = read_le_usize_u32(bytes, offset + 64, "segment section count")?;
    let expected = section_count
        .checked_mul(SECTION_64_BYTES)
        .and_then(|value| value.checked_add(SEGMENT_COMMAND_64_BYTES))
        .ok_or_else(|| "Mach-O segment command span overflows".to_owned())?;
    if size != expected {
        return Err(format!(
            "Mach-O segment command {index} section boundary drift"
        ));
    }
    Ok(())
}

#[derive(Debug)]
struct CodeDirectoryValidation {
    identifier: String,
    code_directory_sha256: String,
    cdhash: String,
    slots: Vec<NsldMachOArm64CodeSignatureSlotAudit>,
}

fn validate_code_directory(
    image: &[u8],
    payload: &[u8],
    plan: &MachOArm64AdHocSignaturePlan,
) -> Result<CodeDirectoryValidation, String> {
    if read_be_u32(payload, 0, "SuperBlob magic")? != CSMAGIC_EMBEDDED_SIGNATURE
        || read_be_u32(payload, 8, "SuperBlob count")? != 1
        || read_be_u32(payload, 12, "SuperBlob slot")? != CSSLOT_CODEDIRECTORY
    {
        return Err("Mach-O code-signature SuperBlob envelope drift".to_owned());
    }
    let blob_bytes = read_be_usize_u32(payload, 4, "SuperBlob size")?;
    let directory_offset = read_be_usize_u32(payload, 16, "CodeDirectory offset")?;
    if blob_bytes != plan.signature_blob_bytes
        || blob_bytes > payload.len()
        || directory_offset != plan.code_directory_offset
    {
        return Err("Mach-O code-signature SuperBlob span drift".to_owned());
    }
    if payload[blob_bytes..].iter().any(|byte| *byte != 0) {
        return Err("Mach-O code-signature alignment padding is not zero".to_owned());
    }
    let directory_bytes = read_be_usize_u32(payload, directory_offset + 4, "CodeDirectory size")?;
    let directory_end = directory_offset
        .checked_add(directory_bytes)
        .ok_or_else(|| "Mach-O CodeDirectory span overflows".to_owned())?;
    if read_be_u32(payload, directory_offset, "CodeDirectory magic")? != CSMAGIC_CODEDIRECTORY
        || directory_bytes != plan.code_directory_bytes
        || directory_bytes < CODE_DIRECTORY_FIXED_BYTES
        || directory_end != blob_bytes
    {
        return Err("Mach-O CodeDirectory envelope drift".to_owned());
    }
    validate_code_directory_fields(payload, directory_offset, plan)?;
    let identifier = read_identifier(payload, directory_offset, plan)?;
    let slots = validate_code_slots(image, payload, directory_offset, plan)?;
    let directory = checked_slice(payload, directory_offset, directory_bytes, "CodeDirectory")?;
    let directory_digest = sha256_bytes(directory);
    Ok(CodeDirectoryValidation {
        identifier,
        code_directory_sha256: hex_bytes(&directory_digest),
        cdhash: hex_bytes(&directory_digest[..20]),
        slots,
    })
}

fn validate_code_directory_fields(
    payload: &[u8],
    base: usize,
    plan: &MachOArm64AdHocSignaturePlan,
) -> Result<(), String> {
    let valid = read_be_u32(payload, base + 8, "CodeDirectory version")? == CODE_DIRECTORY_VERSION
        && read_be_u32(payload, base + 12, "CodeDirectory flags")? == CODE_SIGNATURE_FLAGS
        && read_be_usize_u32(payload, base + 16, "CodeDirectory hash offset")? == plan.hash_offset
        && read_be_usize_u32(payload, base + 20, "CodeDirectory identifier offset")?
            == plan.identifier_offset
        && read_be_u32(payload, base + 24, "CodeDirectory special slots")? == 0
        && read_be_usize_u32(payload, base + 28, "CodeDirectory code slots")?
            == plan.code_slot_count
        && read_be_usize_u32(payload, base + 32, "CodeDirectory code limit")? == plan.code_limit
        && read_u8(payload, base + 36, "CodeDirectory hash size")? == HASH_SIZE_BYTES as u8
        && read_u8(payload, base + 37, "CodeDirectory hash type")? == HASH_TYPE_SHA256
        && read_u8(payload, base + 39, "CodeDirectory page size")? == CODE_PAGE_SIZE_EXPONENT
        && read_be_u64(payload, base + 64, "CodeDirectory executable base")?
            == plan.exec_segment_base
        && read_be_u64(payload, base + 72, "CodeDirectory executable limit")?
            == plan.exec_segment_limit
        && read_be_u64(payload, base + 80, "CodeDirectory executable flags")?
            == plan.exec_segment_flags;
    if !valid {
        return Err("Mach-O CodeDirectory field drift".to_owned());
    }
    if payload[base + 38] != 0 || payload[base + 40..base + 64].iter().any(|byte| *byte != 0) {
        return Err("Mach-O CodeDirectory reserved fields are not zero".to_owned());
    }
    Ok(())
}

fn read_identifier(
    payload: &[u8],
    base: usize,
    plan: &MachOArm64AdHocSignaturePlan,
) -> Result<String, String> {
    let start = base
        .checked_add(plan.identifier_offset)
        .ok_or_else(|| "Mach-O CodeDirectory identifier offset overflows".to_owned())?;
    let limit = base
        .checked_add(plan.hash_offset)
        .ok_or_else(|| "Mach-O CodeDirectory hash offset overflows".to_owned())?;
    let bytes = payload
        .get(start..limit)
        .ok_or_else(|| "Mach-O CodeDirectory identifier exceeds payload".to_owned())?;
    let terminator = bytes
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| "Mach-O CodeDirectory identifier is not terminated".to_owned())?;
    if terminator + 1 != bytes.len() {
        return Err("Mach-O CodeDirectory identifier span drift".to_owned());
    }
    let identifier = std::str::from_utf8(&bytes[..terminator])
        .map_err(|_| "Mach-O CodeDirectory identifier is not UTF-8".to_owned())?;
    if identifier != plan.identifier {
        return Err("Mach-O CodeDirectory identifier drift".to_owned());
    }
    Ok(identifier.to_owned())
}

fn validate_code_slots(
    image: &[u8],
    payload: &[u8],
    base: usize,
    plan: &MachOArm64AdHocSignaturePlan,
) -> Result<Vec<NsldMachOArm64CodeSignatureSlotAudit>, String> {
    let mut slots = Vec::with_capacity(plan.code_slot_count);
    for slot_index in 0..plan.code_slot_count {
        let file_offset = slot_index
            .checked_mul(CODE_PAGE_SIZE_BYTES)
            .ok_or_else(|| "Mach-O signed slot offset overflows".to_owned())?;
        let file_end = file_offset
            .checked_add(CODE_PAGE_SIZE_BYTES)
            .map(|end| end.min(plan.code_limit))
            .ok_or_else(|| "Mach-O signed slot end overflows".to_owned())?;
        let digest = sha256_bytes(checked_slice(
            image,
            file_offset,
            file_end - file_offset,
            "signed code slot",
        )?);
        let hash_offset = base
            .checked_add(plan.hash_offset)
            .and_then(|offset| offset.checked_add(slot_index * HASH_SIZE_BYTES))
            .ok_or_else(|| "Mach-O signed slot hash offset overflows".to_owned())?;
        let stored = checked_slice(payload, hash_offset, HASH_SIZE_BYTES, "code-slot digest")?;
        if stored != digest {
            return Err(format!("Mach-O code slot {slot_index} digest drift"));
        }
        let digest_sha256 = hex_bytes(&digest);
        let audit_hash = crate::fnv1a64_hex(
            format!(
                "{slot_index}|{file_offset}|{}|{digest_sha256}",
                file_end - file_offset
            )
            .as_bytes(),
        );
        slots.push(NsldMachOArm64CodeSignatureSlotAudit {
            slot_index,
            file_offset,
            file_size_bytes: file_end - file_offset,
            digest_sha256,
            audit_hash,
        });
    }
    Ok(slots)
}

fn validation_ledger_hash(
    plan: &MachOArm64AdHocSignaturePlan,
    commands: &LoadCommandValidation,
    parsed: &CodeDirectoryValidation,
    validation_status: &str,
    eligibility_status: &str,
    blockers: &[String],
) -> String {
    let mut out = String::new();
    for value in [
        MACHO_ARM64_AD_HOC_SIGNATURE_CONTRACT,
        MACHO_ARM64_SIGNED_IMAGE_VALIDATION_CONTRACT,
        MACHO_ARM64_PUBLICATION_ELIGIBILITY_CONTRACT,
        validation_status,
        eligibility_status,
        &plan.identifier,
        &parsed.code_directory_sha256,
        &parsed.cdhash,
    ] {
        writeln!(out, "text:{}:{value}", value.len()).unwrap();
    }
    writeln!(
        out,
        "spans={}|{}|{}|{}|{}",
        plan.code_limit,
        plan.signature_blob_bytes,
        plan.signature_payload_bytes,
        commands.count,
        commands.bytes
    )
    .unwrap();
    for blocker in blockers {
        writeln!(out, "blocker={blocker}").unwrap();
    }
    for slot in &parsed.slots {
        writeln!(out, "slot={}|{}", slot.slot_index, slot.audit_hash).unwrap();
    }
    crate::fnv1a64_hex(out.as_bytes())
}

fn read_fixed_name(
    bytes: &[u8],
    offset: usize,
    size: usize,
    label: &str,
) -> Result<String, String> {
    let raw = checked_slice(bytes, offset, size, label)?;
    let end = raw.iter().position(|byte| *byte == 0).unwrap_or(raw.len());
    if raw[end..].iter().any(|byte| *byte != 0) {
        return Err(format!("Mach-O {label} has non-zero trailing bytes"));
    }
    let name =
        std::str::from_utf8(&raw[..end]).map_err(|_| format!("Mach-O {label} is not UTF-8"))?;
    Ok(name.to_owned())
}

fn read_u8(bytes: &[u8], offset: usize, label: &str) -> Result<u8, String> {
    bytes
        .get(offset)
        .copied()
        .ok_or_else(|| format!("Mach-O {label} exceeds input"))
}

fn read_le_u32(bytes: &[u8], offset: usize, label: &str) -> Result<u32, String> {
    let raw = checked_slice(bytes, offset, 4, label)?;
    Ok(u32::from_le_bytes(raw.try_into().unwrap()))
}

fn read_le_u64(bytes: &[u8], offset: usize, label: &str) -> Result<u64, String> {
    let raw = checked_slice(bytes, offset, 8, label)?;
    Ok(u64::from_le_bytes(raw.try_into().unwrap()))
}

fn read_be_u32(bytes: &[u8], offset: usize, label: &str) -> Result<u32, String> {
    let raw = checked_slice(bytes, offset, 4, label)?;
    Ok(u32::from_be_bytes(raw.try_into().unwrap()))
}

fn read_be_u64(bytes: &[u8], offset: usize, label: &str) -> Result<u64, String> {
    let raw = checked_slice(bytes, offset, 8, label)?;
    Ok(u64::from_be_bytes(raw.try_into().unwrap()))
}

fn read_le_usize_u32(bytes: &[u8], offset: usize, label: &str) -> Result<usize, String> {
    usize::try_from(read_le_u32(bytes, offset, label)?)
        .map_err(|_| format!("Mach-O {label} exceeds usize"))
}

fn read_le_usize_u64(bytes: &[u8], offset: usize, label: &str) -> Result<usize, String> {
    usize::try_from(read_le_u64(bytes, offset, label)?)
        .map_err(|_| format!("Mach-O {label} exceeds usize"))
}

fn read_be_usize_u32(bytes: &[u8], offset: usize, label: &str) -> Result<usize, String> {
    usize::try_from(read_be_u32(bytes, offset, label)?)
        .map_err(|_| format!("Mach-O {label} exceeds usize"))
}

fn checked_slice<'a>(
    bytes: &'a [u8],
    offset: usize,
    size: usize,
    label: &str,
) -> Result<&'a [u8], String> {
    let end = offset
        .checked_add(size)
        .ok_or_else(|| format!("Mach-O {label} span overflows"))?;
    bytes
        .get(offset..end)
        .ok_or_else(|| format!("Mach-O {label} exceeds input"))
}
