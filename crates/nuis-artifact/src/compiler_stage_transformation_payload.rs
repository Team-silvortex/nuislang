use std::{
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use super::{
    compiler_projection_checkpoint_kind_tag, CompilerStageTransformationRecord,
    CompilerStageTransformations, COMPILER_STAGE_CHECKPOINT_WORD_COUNT,
};
use crate::{
    parse_compiler_structural_projection, render_compiler_structural_projection, ArtifactError,
    CompilerProjectionKind, CompilerProjectionRecordKind, CompilerStageKind,
    VerifiedCompilerStagePayload,
};

const DERIVED_PAYLOAD_MAGIC: &[u8; 8] = b"NSCSTG02";
const RECORD_KIND_BITS: usize = 4;
const RECORD_KIND_MASK: usize = (1 << RECORD_KIND_BITS) - 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DecodedCompilerStageTransformationPayload {
    pub checkpoint_words: Vec<usize>,
    pub source_payload: Vec<u8>,
}

pub(crate) fn payload_file(ordinal: usize) -> String {
    format!("nuis.compiler-stage-transformation.{ordinal}.bin")
}

pub(crate) fn encode_payload(
    stage: CompilerStageKind,
    source_payload: &[u8],
    checkpoint_words: &[usize],
) -> Result<Vec<u8>, ArtifactError> {
    let kind = projection_kind(stage)?;
    let kind_tag = compiler_projection_checkpoint_kind_tag(kind);
    if checkpoint_words.len() != COMPILER_STAGE_CHECKPOINT_WORD_COUNT
        || checkpoint_words.first().copied() != Some(kind_tag)
    {
        return Err(ArtifactError::new(format!(
            "compiler stage `{}` derived payload has an invalid checkpoint shape",
            stage.as_str()
        )));
    }
    let source = std::str::from_utf8(source_payload)
        .map_err(|_| ArtifactError::new("compiler stage derived source payload is not UTF-8"))?;
    let projection = parse_compiler_structural_projection(kind, source)?;
    let mut out = Vec::with_capacity(source_payload.len());
    out.extend_from_slice(DERIVED_PAYLOAD_MAGIC);
    push_varint(&mut out, kind_tag);
    push_varint(&mut out, source_payload.len());
    push_varint(&mut out, checkpoint_words.len());
    push_varint(&mut out, projection.records.len());
    for word in checkpoint_words {
        push_varint(&mut out, *word);
    }
    for record in &projection.records {
        let packed = record
            .depth
            .checked_mul(1 << RECORD_KIND_BITS)
            .and_then(|value| value.checked_add(record_kind_tag(record.kind)))
            .ok_or_else(|| ArtifactError::new("compiler stage derived record depth overflow"))?;
        push_varint(&mut out, packed);
        push_varint(&mut out, record.body.len());
        out.extend_from_slice(record.body.as_bytes());
    }
    Ok(out)
}

pub(crate) fn decode_payload(
    stage: CompilerStageKind,
    bytes: &[u8],
) -> Result<DecodedCompilerStageTransformationPayload, ArtifactError> {
    if bytes.len() < DERIVED_PAYLOAD_MAGIC.len()
        || &bytes[..DERIVED_PAYLOAD_MAGIC.len()] != DERIVED_PAYLOAD_MAGIC
    {
        return Err(ArtifactError::new(format!(
            "compiler stage `{}` derived payload has an invalid header",
            stage.as_str()
        )));
    }
    let kind = projection_kind(stage)?;
    let mut cursor = DERIVED_PAYLOAD_MAGIC.len();
    let kind_tag = read_varint(bytes, &mut cursor, "projection kind")?;
    if kind_tag != compiler_projection_checkpoint_kind_tag(kind) {
        return Err(ArtifactError::new(format!(
            "compiler stage `{}` derived payload kind tag mismatch",
            stage.as_str()
        )));
    }
    let source_len = read_varint(bytes, &mut cursor, "source length")?;
    let word_count = read_varint(bytes, &mut cursor, "checkpoint count")?;
    if word_count != COMPILER_STAGE_CHECKPOINT_WORD_COUNT {
        return Err(ArtifactError::new(format!(
            "compiler stage `{}` derived payload checkpoint count mismatch",
            stage.as_str()
        )));
    }
    let record_count = read_varint(bytes, &mut cursor, "record count")?;
    if source_len == 0 || record_count == 0 {
        return Err(ArtifactError::new(
            "compiler stage derived record count is invalid",
        ));
    }
    let mut checkpoint_words = Vec::with_capacity(word_count);
    for _ in 0..word_count {
        checkpoint_words.push(read_varint(bytes, &mut cursor, "checkpoint word")?);
    }
    if checkpoint_words.first().copied() != Some(kind_tag) {
        return Err(ArtifactError::new(
            "compiler stage derived checkpoint kind mismatch",
        ));
    }
    let minimum_source_bytes = record_count
        .checked_mul(2)
        .ok_or_else(|| ArtifactError::new("compiler stage derived record count overflow"))?;
    let minimum_encoded_bytes = record_count
        .checked_mul(3)
        .ok_or_else(|| ArtifactError::new("compiler stage derived record count overflow"))?;
    if minimum_source_bytes > source_len
        || minimum_encoded_bytes > bytes.len().saturating_sub(cursor)
    {
        return Err(ArtifactError::new(
            "compiler stage derived record count is invalid",
        ));
    }
    let mut encoded_records = Vec::with_capacity(record_count);
    let mut source_payload = String::new();
    for ordinal in 0..record_count {
        let packed = read_varint(bytes, &mut cursor, "record shape")?;
        let depth = packed >> RECORD_KIND_BITS;
        let record_kind = parse_record_kind(packed & RECORD_KIND_MASK)?;
        let body_len = read_varint(bytes, &mut cursor, "record body length")?;
        let body_start = cursor;
        let body_end = cursor
            .checked_add(body_len)
            .ok_or_else(|| ArtifactError::new("compiler stage derived body length overflow"))?;
        let body_bytes = bytes
            .get(cursor..body_end)
            .ok_or_else(|| ArtifactError::new("compiler stage derived record body is truncated"))?;
        let body = std::str::from_utf8(body_bytes)
            .map_err(|_| ArtifactError::new("compiler stage derived record body is not UTF-8"))?;
        cursor = body_end;
        let indentation = depth
            .checked_mul(2)
            .ok_or_else(|| ArtifactError::new("compiler stage derived indentation overflow"))?;
        let reconstructed_len = source_payload
            .len()
            .checked_add(indentation)
            .and_then(|value| value.checked_add(body_len))
            .and_then(|value| value.checked_add(1))
            .ok_or_else(|| ArtifactError::new("compiler stage derived source length overflow"))?;
        if reconstructed_len > source_len {
            return Err(ArtifactError::new(
                "compiler stage derived source length mismatch",
            ));
        }
        for _ in 0..indentation {
            source_payload.push(' ');
        }
        source_payload.push_str(body);
        source_payload.push('\n');
        encoded_records.push((ordinal, depth, record_kind, body_start, body_end));
    }
    if cursor != bytes.len() || source_payload.len() != source_len {
        return Err(ArtifactError::new(format!(
            "compiler stage `{}` derived payload length mismatch",
            stage.as_str()
        )));
    }
    let projection = parse_compiler_structural_projection(kind, &source_payload)?;
    if render_compiler_structural_projection(&projection) != source_payload
        || projection.records.len() != encoded_records.len()
        || projection.records.iter().zip(&encoded_records).any(
            |(record, (ordinal, depth, record_kind, body_start, body_end))| {
                record.ordinal != *ordinal
                    || record.depth != *depth
                    || record.kind != *record_kind
                    || record.body.as_bytes() != &bytes[*body_start..*body_end]
            },
        )
    {
        return Err(ArtifactError::new(
            "compiler stage derived structural record metadata mismatch",
        ));
    }
    Ok(DecodedCompilerStageTransformationPayload {
        checkpoint_words,
        source_payload: source_payload.into_bytes(),
    })
}

pub(crate) fn materialize_payloads(
    root: &Path,
    manifest: &CompilerStageTransformations,
    payloads: &[VerifiedCompilerStagePayload],
) -> Result<(), ArtifactError> {
    for record in &manifest.records {
        let source = source_payload(payloads, record.source_stage)?;
        let bytes = encode_payload(record.source_stage, &source.bytes, &record.output_words)?;
        validate_output_identity(record, &bytes)?;
        let path = output_path(root, record)?;
        reject_non_regular_existing_path(&path)?;
        fs::write(&path, bytes).map_err(|error| {
            ArtifactError::new(format!(
                "failed to write compiler stage derived payload `{}`: {error}",
                path.display()
            ))
        })?;
    }
    Ok(())
}

pub(crate) fn validate_materialized_payloads(
    root: &Path,
    manifest: &CompilerStageTransformations,
    payloads: &[VerifiedCompilerStagePayload],
) -> Result<(), ArtifactError> {
    for record in &manifest.records {
        let path = output_path(root, record)?;
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            ArtifactError::new(format!(
                "failed to inspect compiler stage derived payload `{}`: {error}",
                path.display()
            ))
        })?;
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            return Err(ArtifactError::new(format!(
                "compiler stage derived payload `{}` must be a regular file",
                path.display()
            )));
        }
        let bytes = fs::read(&path).map_err(|error| {
            ArtifactError::new(format!(
                "failed to read compiler stage derived payload `{}`: {error}",
                path.display()
            ))
        })?;
        validate_output_identity(record, &bytes)?;
        let decoded = decode_payload(record.source_stage, &bytes)?;
        let source = source_payload(payloads, record.source_stage)?;
        if decoded.checkpoint_words != record.output_words || decoded.source_payload != source.bytes
        {
            return Err(ArtifactError::new(format!(
                "compiler stage `{}` derived payload failed lossless replay",
                record.source_stage.as_str()
            )));
        }
    }
    Ok(())
}

pub(crate) fn validate_output_identity(
    record: &CompilerStageTransformationRecord,
    bytes: &[u8],
) -> Result<(), ArtifactError> {
    if bytes.len() != record.output_payload_bytes
        || sha256_hex(bytes) != record.output_payload_sha256
    {
        return Err(ArtifactError::new(format!(
            "compiler stage `{}` derived payload length or SHA-256 mismatch",
            record.source_stage.as_str()
        )));
    }
    Ok(())
}

fn projection_kind(stage: CompilerStageKind) -> Result<CompilerProjectionKind, ArtifactError> {
    Ok(match stage {
        CompilerStageKind::Ast => CompilerProjectionKind::Ast,
        CompilerStageKind::Nir => CompilerProjectionKind::Nir,
        _ => {
            return Err(ArtifactError::new(format!(
                "compiler stage `{}` has no derived structural payload",
                stage.as_str()
            )))
        }
    })
}

fn record_kind_tag(kind: CompilerProjectionRecordKind) -> usize {
    match kind {
        CompilerProjectionRecordKind::ModuleDocumentation => 0,
        CompilerProjectionRecordKind::Import => 1,
        CompilerProjectionRecordKind::ModuleHeader => 2,
        CompilerProjectionRecordKind::Item => 3,
        CompilerProjectionRecordKind::Member => 4,
        CompilerProjectionRecordKind::Nested => 5,
        CompilerProjectionRecordKind::Documentation => 6,
        CompilerProjectionRecordKind::OpaqueBody => 7,
        CompilerProjectionRecordKind::OpaqueTerminator => 8,
    }
}

fn parse_record_kind(tag: usize) -> Result<CompilerProjectionRecordKind, ArtifactError> {
    match tag {
        0 => Ok(CompilerProjectionRecordKind::ModuleDocumentation),
        1 => Ok(CompilerProjectionRecordKind::Import),
        2 => Ok(CompilerProjectionRecordKind::ModuleHeader),
        3 => Ok(CompilerProjectionRecordKind::Item),
        4 => Ok(CompilerProjectionRecordKind::Member),
        5 => Ok(CompilerProjectionRecordKind::Nested),
        6 => Ok(CompilerProjectionRecordKind::Documentation),
        7 => Ok(CompilerProjectionRecordKind::OpaqueBody),
        8 => Ok(CompilerProjectionRecordKind::OpaqueTerminator),
        _ => Err(ArtifactError::new(
            "compiler stage derived record kind is unsupported",
        )),
    }
}

fn push_varint(out: &mut Vec<u8>, value: usize) {
    let mut value = value as u64;
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

fn read_varint(bytes: &[u8], cursor: &mut usize, label: &str) -> Result<usize, ArtifactError> {
    let start = *cursor;
    let mut value = 0u64;
    for index in 0..10 {
        let byte = *bytes
            .get(*cursor)
            .ok_or_else(|| ArtifactError::new("compiler stage derived varint is truncated"))?;
        *cursor += 1;
        if index == 9 && byte > 1 {
            return Err(ArtifactError::new(format!(
                "compiler stage derived {label} exceeds u64"
            )));
        }
        value |= u64::from(byte & 0x7f) << (index * 7);
        if byte & 0x80 == 0 {
            let decoded = usize::try_from(value).map_err(|_| {
                ArtifactError::new(format!("compiler stage derived {label} exceeds usize"))
            })?;
            let mut canonical = Vec::new();
            push_varint(&mut canonical, decoded);
            if canonical.len() != *cursor - start {
                return Err(ArtifactError::new(format!(
                    "compiler stage derived {label} is not canonically encoded"
                )));
            }
            return Ok(decoded);
        }
    }
    Err(ArtifactError::new(format!(
        "compiler stage derived {label} varint is unterminated"
    )))
}

fn source_payload(
    payloads: &[VerifiedCompilerStagePayload],
    stage: CompilerStageKind,
) -> Result<&VerifiedCompilerStagePayload, ArtifactError> {
    payloads
        .iter()
        .find(|payload| payload.stage == stage)
        .ok_or_else(|| {
            ArtifactError::new(format!(
                "compiler stage `{}` derived source payload is missing",
                stage.as_str()
            ))
        })
}

fn output_path(
    root: &Path,
    record: &CompilerStageTransformationRecord,
) -> Result<PathBuf, ArtifactError> {
    if record.output_payload_file != payload_file(record.ordinal) {
        return Err(ArtifactError::new(format!(
            "compiler stage transformation record {} has a non-canonical derived payload file",
            record.ordinal
        )));
    }
    Ok(root.join(&record.output_payload_file))
}

fn reject_non_regular_existing_path(path: &Path) -> Result<(), ArtifactError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if !metadata.file_type().is_file() || metadata.file_type().is_symlink() => {
            Err(ArtifactError::new(format!(
                "compiler stage derived payload `{}` cannot replace a non-regular path",
                path.display()
            )))
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ArtifactError::new(format!(
            "failed to inspect compiler stage derived payload `{}`: {error}",
            path.display()
        ))),
    }
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
