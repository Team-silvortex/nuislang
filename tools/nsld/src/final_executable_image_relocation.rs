use super::{
    container_verify,
    final_executable_image::FINAL_EXECUTABLE_IMAGE_HEADER_SIZE,
    fnv1a64_hex,
    reports::{
        NsldFinalExecutableByteMapEntry, NsldFinalExecutablePayloadDiagnostic,
        NsldFinalExecutableRelocationApplicationRecord,
        NsldFinalExecutableRelocationPatchPreviewRecord,
    },
};
use std::{collections::BTreeMap, fs, path::Path};

pub(crate) struct RelocationPatchPreview {
    pub(crate) status: String,
    pub(crate) count: usize,
    pub(crate) table_hash: String,
    pub(crate) records: Vec<NsldFinalExecutableRelocationPatchPreviewRecord>,
    pub(crate) blockers: Vec<String>,
}

pub(crate) struct RelocationPatchApplication {
    pub(crate) status: String,
    pub(crate) count: usize,
    pub(crate) table_hash: String,
    pub(crate) blockers: Vec<String>,
}

pub(crate) struct RelocationPatchByteAudit {
    pub(crate) status: String,
    pub(crate) count: usize,
    pub(crate) table_hash: String,
    pub(crate) blockers: Vec<String>,
}

pub(crate) struct RelocationApplicationAudit {
    pub(crate) status: String,
    pub(crate) count: usize,
    pub(crate) table_hash: String,
    pub(crate) blockers: Vec<String>,
}

pub(crate) fn relocation_patch_preview(
    applications: &[NsldFinalExecutableRelocationApplicationRecord],
    payloads: &[NsldFinalExecutablePayloadDiagnostic],
    byte_map_entries: &[NsldFinalExecutableByteMapEntry],
) -> RelocationPatchPreview {
    let symbol_offsets = resolved_symbol_image_offsets(payloads, byte_map_entries);
    let mut blockers = Vec::new();
    let records = applications
        .iter()
        .map(|application| {
            let target_symbol_image_offset =
                symbol_offsets.get(&application.target_symbol_id).copied();
            let resolved_patch_value = target_symbol_image_offset
                .and_then(|offset| checked_offset_addend(offset, application.addend));
            let (patch_kind, patch_value_hash, preview_status, resolver_status) =
                match resolved_patch_value {
                    Some(value) => (
                        "u64-le-resolved-image-offset".to_owned(),
                        fnv1a64_hex(&value.to_le_bytes()),
                        "resolved".to_owned(),
                        "resolved".to_owned(),
                    ),
                    None => {
                        blockers.push(format!(
                            "{}:{}",
                            application.relocation_id, application.target_symbol_id
                        ));
                        (
                            "u64-le-unresolved-placeholder".to_owned(),
                            fnv1a64_hex(&[0; 8]),
                            "blocked".to_owned(),
                            "target-symbol-unresolved".to_owned(),
                        )
                    }
                };
            NsldFinalExecutableRelocationPatchPreviewRecord {
                order_index: application.order_index,
                relocation_id: application.relocation_id.clone(),
                patch_kind,
                patch_offset: FINAL_EXECUTABLE_IMAGE_HEADER_SIZE
                    .saturating_add(application.image_offset),
                patch_width_bytes: 8,
                resolved_patch_value,
                patch_value_hash,
                target_symbol_id: application.target_symbol_id.clone(),
                target_symbol_image_offset,
                preview_status,
                resolver_status,
            }
        })
        .collect::<Vec<_>>();
    let table_hash = relocation_patch_preview_table_hash(&records);
    RelocationPatchPreview {
        status: if records.is_empty() {
            "empty".to_owned()
        } else if blockers.is_empty() {
            "resolved".to_owned()
        } else {
            "blocked".to_owned()
        },
        count: records.len(),
        table_hash,
        records,
        blockers,
    }
}

pub(crate) fn apply_relocation_patches(
    image: &mut [u8],
    records: &[NsldFinalExecutableRelocationPatchPreviewRecord],
) -> RelocationPatchApplication {
    let mut blockers = Vec::new();
    let mut applied_count = 0usize;
    let mut material = String::new();
    for record in records {
        let status = if record.preview_status != "resolved" || record.resolver_status != "resolved"
        {
            "resolver-blocked"
        } else if record.resolved_patch_value.is_none() {
            "missing-resolved-value"
        } else if record.patch_width_bytes != 8 {
            "unsupported-width"
        } else if record.patch_kind != "u64-le-resolved-image-offset" {
            "unsupported-kind"
        } else if record
            .patch_offset
            .checked_add(record.patch_width_bytes)
            .map(|end| end > image.len())
            .unwrap_or(true)
        {
            "out-of-bounds"
        } else {
            "applied"
        };
        if status == "applied" {
            let value = record.resolved_patch_value.unwrap_or(0) as u64;
            let start = record.patch_offset;
            let end = start + 8;
            image[start..end].copy_from_slice(&value.to_le_bytes());
            applied_count += 1;
        } else {
            blockers.push(format!("{}:{status}", record.relocation_id));
        }
        material.push_str(&record.order_index.to_string());
        material.push('\t');
        material.push_str(&record.relocation_id);
        material.push('\t');
        material.push_str(&record.patch_offset.to_string());
        material.push('\t');
        material.push_str(&record.patch_width_bytes.to_string());
        material.push('\t');
        material.push_str(
            &record
                .resolved_patch_value
                .map_or(0, |value| value)
                .to_string(),
        );
        material.push('\t');
        material.push_str(status);
        material.push('\n');
    }
    RelocationPatchApplication {
        status: if records.is_empty() {
            "empty".to_owned()
        } else if blockers.is_empty() {
            "applied".to_owned()
        } else {
            "blocked".to_owned()
        },
        count: applied_count,
        table_hash: fnv1a64_hex(material.as_bytes()),
        blockers,
    }
}

pub(crate) fn relocation_patch_byte_audit(
    image_path: &Path,
    records: &[NsldFinalExecutableRelocationPatchPreviewRecord],
) -> RelocationPatchByteAudit {
    let bytes = match fs::read(image_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            return RelocationPatchByteAudit {
                status: "blocked".to_owned(),
                count: 0,
                table_hash: fnv1a64_hex(b""),
                blockers: vec![format!("image-unreadable:{error}")],
            };
        }
    };
    let mut blockers = Vec::new();
    let mut verified_count = 0usize;
    let mut material = String::new();
    for record in records {
        let expected = record.resolved_patch_value;
        let actual = read_u64_patch_value(&bytes, record.patch_offset, record.patch_width_bytes);
        let status = if record.preview_status != "resolved" || record.resolver_status != "resolved"
        {
            "resolver-blocked"
        } else if expected.is_none() {
            "missing-resolved-value"
        } else if actual.is_none() {
            "patch-bytes-unreadable"
        } else if actual.map(|value| value as usize) != expected {
            "patch-value-mismatch"
        } else {
            "verified"
        };
        if status == "verified" {
            verified_count += 1;
        } else {
            blockers.push(format!("{}:{status}", record.relocation_id));
        }
        material.push_str(&record.order_index.to_string());
        material.push('\t');
        material.push_str(&record.relocation_id);
        material.push('\t');
        material.push_str(&record.patch_offset.to_string());
        material.push('\t');
        material.push_str(&record.patch_width_bytes.to_string());
        material.push('\t');
        material.push_str(&expected.map_or(0, |value| value).to_string());
        material.push('\t');
        material.push_str(&actual.unwrap_or(0).to_string());
        material.push('\t');
        material.push_str(status);
        material.push('\n');
    }
    RelocationPatchByteAudit {
        status: if records.is_empty() {
            "empty".to_owned()
        } else if blockers.is_empty() {
            "verified".to_owned()
        } else {
            "blocked".to_owned()
        },
        count: verified_count,
        table_hash: fnv1a64_hex(material.as_bytes()),
        blockers,
    }
}

pub(crate) fn relocation_application_audit(
    records: &[NsldFinalExecutableRelocationApplicationRecord],
    payload_byte_offset: usize,
    payload_byte_span: usize,
) -> RelocationApplicationAudit {
    let payload_end = payload_byte_offset.saturating_add(payload_byte_span);
    let mut blockers = Vec::new();
    let mut material = String::new();
    for record in records {
        let image_offset = payload_byte_offset.saturating_add(record.image_offset);
        let status = if record.application_status != "planned" {
            "status-blocked"
        } else if image_offset >= payload_end {
            "out-of-bounds"
        } else {
            "audited"
        };
        if status != "audited" {
            blockers.push(format!("{}:{status}", record.relocation_id));
        }
        material.push_str(&record.order_index.to_string());
        material.push('\t');
        material.push_str(&record.relocation_id);
        material.push('\t');
        material.push_str(&record.relocation_kind);
        material.push('\t');
        material.push_str(&record.source_payload_id);
        material.push('\t');
        material.push_str(&record.source_section_id);
        material.push('\t');
        material.push_str(&record.source_offset.to_string());
        material.push('\t');
        material.push_str(&image_offset.to_string());
        material.push('\t');
        material.push_str(&record.target_symbol_id);
        material.push('\t');
        material.push_str(&record.addend.to_string());
        material.push('\t');
        material.push_str(status);
        material.push('\n');
    }
    RelocationApplicationAudit {
        status: if blockers.is_empty() {
            "ready".to_owned()
        } else {
            "blocked".to_owned()
        },
        count: records.len(),
        table_hash: fnv1a64_hex(material.as_bytes()),
        blockers,
    }
}

fn resolved_symbol_image_offsets(
    payloads: &[NsldFinalExecutablePayloadDiagnostic],
    byte_map_entries: &[NsldFinalExecutableByteMapEntry],
) -> BTreeMap<String, usize> {
    let Some(container_payload) = payloads
        .iter()
        .find(|payload| payload.payload_id == "payload0000.container")
    else {
        return BTreeMap::new();
    };
    let executable_payload_offset = byte_map_entries
        .iter()
        .find(|entry| entry.payload_id == "payload0001.container-payload")
        .map(|entry| entry.offset)
        .unwrap_or(0);
    let source = fs::read_to_string(&container_payload.path).unwrap_or_default();
    container_verify::loader_symbol_entries(&source)
        .into_iter()
        .map(|symbol| {
            (
                symbol.symbol_id,
                FINAL_EXECUTABLE_IMAGE_HEADER_SIZE
                    .saturating_add(executable_payload_offset)
                    .saturating_add(symbol.offset),
            )
        })
        .collect()
}

fn checked_offset_addend(offset: usize, addend: isize) -> Option<usize> {
    if addend >= 0 {
        offset.checked_add(addend as usize)
    } else {
        offset.checked_sub(addend.unsigned_abs())
    }
}

fn read_u64_patch_value(bytes: &[u8], offset: usize, width: usize) -> Option<u64> {
    if width != 8 {
        return None;
    }
    let value: [u8; 8] = bytes
        .get(offset..offset.checked_add(width)?)?
        .try_into()
        .ok()?;
    Some(u64::from_le_bytes(value))
}

pub(crate) fn relocation_patch_preview_table_hash(
    records: &[NsldFinalExecutableRelocationPatchPreviewRecord],
) -> String {
    let mut material = String::new();
    for record in records {
        material.push_str(&record.order_index.to_string());
        material.push('\t');
        material.push_str(&record.relocation_id);
        material.push('\t');
        material.push_str(&record.patch_kind);
        material.push('\t');
        material.push_str(&record.patch_offset.to_string());
        material.push('\t');
        material.push_str(&record.patch_width_bytes.to_string());
        material.push('\t');
        material.push_str(
            &record
                .resolved_patch_value
                .map_or(0, |value| value)
                .to_string(),
        );
        material.push('\t');
        material.push_str(&record.patch_value_hash);
        material.push('\t');
        material.push_str(&record.target_symbol_id);
        material.push('\t');
        material.push_str(
            &record
                .target_symbol_image_offset
                .map_or(0, |value| value)
                .to_string(),
        );
        material.push('\t');
        material.push_str(&record.preview_status);
        material.push('\t');
        material.push_str(&record.resolver_status);
        material.push('\n');
    }
    fnv1a64_hex(material.as_bytes())
}
