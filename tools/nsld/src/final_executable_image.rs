use super::{
    fnv1a64_hex,
    reports::{NsldFinalExecutableLayoutPlanReport, NsldFinalExecutablePayloadDiagnostic},
};
use std::{fs, path::Path};

pub(crate) const FINAL_EXECUTABLE_IMAGE_MAGIC: &[u8; 8] = b"NUIFIMG\0";
pub(crate) const FINAL_EXECUTABLE_IMAGE_MAGIC_TEXT: &str = "NUIFIMG";
pub(crate) const FINAL_EXECUTABLE_IMAGE_FORMAT: &str = "nuis-final-executable-image-dry-run-v1";
pub(crate) const FINAL_EXECUTABLE_IMAGE_VERSION: u32 = 1;
pub(crate) const FINAL_EXECUTABLE_IMAGE_HEADER_SIZE: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FinalExecutableImageHeader {
    pub(crate) magic: String,
    pub(crate) version: u32,
    pub(crate) header_size: usize,
    pub(crate) payload_offset: usize,
    pub(crate) payload_span: usize,
    pub(crate) layout_hash: String,
    pub(crate) byte_map_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FinalExecutablePayloadRegionVerify {
    pub(crate) expected_hash: Option<String>,
    pub(crate) actual_hash: Option<String>,
    pub(crate) actual_count: Option<usize>,
}

pub(crate) fn encode_final_executable_image(
    layout: &NsldFinalExecutableLayoutPlanReport,
) -> Option<Vec<u8>> {
    if layout
        .payloads
        .iter()
        .any(|payload| payload.required && !payload.present)
    {
        return None;
    }
    let mut bytes = final_executable_image_header(layout)?;
    bytes.resize(FINAL_EXECUTABLE_IMAGE_HEADER_SIZE + layout.byte_span, 0);
    for entry in &layout.byte_map_entries {
        let payload = layout
            .payloads
            .iter()
            .find(|payload| payload.payload_id == entry.payload_id)?;
        if !payload.present {
            continue;
        }
        let payload_bytes = fs::read(&payload.path).ok()?;
        let start = FINAL_EXECUTABLE_IMAGE_HEADER_SIZE.saturating_add(entry.offset);
        let end = start.saturating_add(payload_bytes.len());
        if end > bytes.len() {
            return None;
        }
        bytes[start..end].copy_from_slice(&payload_bytes);
    }
    Some(bytes)
}

pub(crate) fn parse_final_executable_image_header(
    bytes: &[u8],
) -> Option<FinalExecutableImageHeader> {
    if bytes.len() < FINAL_EXECUTABLE_IMAGE_HEADER_SIZE {
        return None;
    }
    let magic = &bytes[0..8];
    let magic_text = if magic == FINAL_EXECUTABLE_IMAGE_MAGIC {
        FINAL_EXECUTABLE_IMAGE_MAGIC_TEXT.to_owned()
    } else {
        magic
            .iter()
            .copied()
            .take_while(|byte| *byte != 0)
            .map(char::from)
            .collect()
    };
    Some(FinalExecutableImageHeader {
        magic: magic_text,
        version: read_u32_le(bytes, 8)?,
        header_size: read_u32_le(bytes, 12)? as usize,
        payload_span: read_u64_le(bytes, 24)? as usize,
        payload_offset: read_u64_le(bytes, 32)? as usize,
        layout_hash: format!("0x{:016x}", read_u64_le(bytes, 40)?),
        byte_map_hash: format!("0x{:016x}", read_u64_le(bytes, 48)?),
    })
}

pub(crate) fn verify_final_executable_image_payload_region(
    layout: &NsldFinalExecutableLayoutPlanReport,
    image_path: &Path,
    expected_image: Option<&[u8]>,
    issues: &mut Vec<String>,
) -> FinalExecutablePayloadRegionVerify {
    let expected_payload_region =
        expected_image.and_then(|bytes| image_payload_region(bytes).map(|region| region.to_vec()));
    let expected_payload_region =
        expected_payload_region.or_else(|| encode_final_executable_payload_region(layout));
    let expected_hash = expected_payload_region
        .as_ref()
        .map(|bytes| fnv1a64_hex(bytes));
    let image_bytes = match fs::read(image_path) {
        Ok(bytes) => bytes,
        Err(_) => {
            return FinalExecutablePayloadRegionVerify {
                expected_hash,
                actual_hash: None,
                actual_count: None,
            };
        }
    };
    let Some(header) = parse_final_executable_image_header(&image_bytes) else {
        return FinalExecutablePayloadRegionVerify {
            expected_hash,
            actual_hash: None,
            actual_count: None,
        };
    };
    let payload_end = header.payload_offset.saturating_add(header.payload_span);
    let payload_region = match image_bytes.get(header.payload_offset..payload_end) {
        Some(bytes) => bytes,
        None => {
            issues.push(format!(
                "image_payload_region_out_of_bounds: offset {} span {} image_size {}",
                header.payload_offset,
                header.payload_span,
                image_bytes.len()
            ));
            return FinalExecutablePayloadRegionVerify {
                expected_hash,
                actual_hash: None,
                actual_count: Some(0),
            };
        }
    };
    let actual_hash = Some(fnv1a64_hex(payload_region));
    if actual_hash != expected_hash {
        issues.push(format!(
            "image_payload_region_hash mismatch: expected {}, found {}",
            expected_hash
                .clone()
                .unwrap_or_else(|| "missing".to_owned()),
            actual_hash.clone().unwrap_or_else(|| "missing".to_owned())
        ));
    }
    let mut actual_count = 0usize;
    for entry in &layout.byte_map_entries {
        let payload = layout
            .payloads
            .iter()
            .find(|payload| payload.payload_id == entry.payload_id);
        let Some(payload) = payload else {
            issues.push(format!(
                "image_payload_region_missing_layout_payload:{}",
                entry.payload_id
            ));
            continue;
        };
        if !payload.present {
            continue;
        }
        actual_count += 1;
        let start = header.payload_offset.saturating_add(entry.offset);
        let end = start.saturating_add(entry.size_bytes);
        let Some(slice) = image_bytes.get(start..end) else {
            issues.push(format!(
                "image_payload_region_entry_out_of_bounds:{}",
                entry.payload_id
            ));
            continue;
        };
        let expected_slice = expected_image
            .and_then(image_payload_region)
            .and_then(|region| {
                region.get(entry.offset..entry.offset.saturating_add(entry.size_bytes))
            });
        let expected_payload_hash = expected_slice
            .map(fnv1a64_hex)
            .unwrap_or_else(|| payload.content_hash.clone());
        let actual_payload_hash = fnv1a64_hex(slice);
        if actual_payload_hash != expected_payload_hash {
            issues.push(format!(
                "image_payload_region_entry_hash mismatch for {}: expected {}, found {}",
                entry.payload_id, expected_payload_hash, actual_payload_hash
            ));
        }
    }
    FinalExecutablePayloadRegionVerify {
        expected_hash,
        actual_hash,
        actual_count: Some(actual_count),
    }
}

fn image_payload_region(bytes: &[u8]) -> Option<&[u8]> {
    let header = parse_final_executable_image_header(bytes)?;
    let payload_end = header.payload_offset.saturating_add(header.payload_span);
    bytes.get(header.payload_offset..payload_end)
}

fn final_executable_image_header(layout: &NsldFinalExecutableLayoutPlanReport) -> Option<Vec<u8>> {
    let mut header = Vec::with_capacity(FINAL_EXECUTABLE_IMAGE_HEADER_SIZE);
    header.extend_from_slice(FINAL_EXECUTABLE_IMAGE_MAGIC);
    push_u32_le(&mut header, FINAL_EXECUTABLE_IMAGE_VERSION);
    push_u32_le(&mut header, FINAL_EXECUTABLE_IMAGE_HEADER_SIZE as u32);
    push_u32_le(&mut header, layout.payload_count as u32);
    push_u32_le(&mut header, layout.byte_alignment as u32);
    push_u64_le(&mut header, layout.byte_span as u64);
    push_u64_le(&mut header, FINAL_EXECUTABLE_IMAGE_HEADER_SIZE as u64);
    push_u64_le(&mut header, hash_hex_to_u64(&layout.layout_hash)?);
    push_u64_le(&mut header, hash_hex_to_u64(&layout.byte_map_hash)?);
    push_u64_le(&mut header, 0);
    (header.len() == FINAL_EXECUTABLE_IMAGE_HEADER_SIZE).then_some(header)
}

fn encode_final_executable_payload_region(
    layout: &NsldFinalExecutableLayoutPlanReport,
) -> Option<Vec<u8>> {
    if layout
        .payloads
        .iter()
        .any(|payload| payload.required && !payload.present)
    {
        return None;
    }
    let mut bytes = vec![0u8; layout.byte_span];
    for entry in &layout.byte_map_entries {
        let payload = layout
            .payloads
            .iter()
            .find(|payload| payload.payload_id == entry.payload_id)?;
        if !payload.present {
            continue;
        }
        let payload_bytes = fs::read(&payload.path).ok()?;
        let end = entry.offset.saturating_add(payload_bytes.len());
        if end > bytes.len() {
            return None;
        }
        bytes[entry.offset..end].copy_from_slice(&payload_bytes);
    }
    Some(bytes)
}

fn push_u32_le(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u64_le(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn hash_hex_to_u64(value: &str) -> Option<u64> {
    u64::from_str_radix(value.strip_prefix("0x").unwrap_or(value), 16).ok()
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Option<u32> {
    let value: [u8; 4] = bytes.get(offset..offset + 4)?.try_into().ok()?;
    Some(u32::from_le_bytes(value))
}

fn read_u64_le(bytes: &[u8], offset: usize) -> Option<u64> {
    let value: [u8; 8] = bytes.get(offset..offset + 8)?.try_into().ok()?;
    Some(u64::from_le_bytes(value))
}

pub(crate) fn final_executable_payload_size(
    payload: &NsldFinalExecutablePayloadDiagnostic,
) -> usize {
    if payload.present {
        fs::metadata(&payload.path)
            .map(|metadata| metadata.len() as usize)
            .unwrap_or(0)
    } else {
        0
    }
}
