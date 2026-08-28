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
    ArtifactError, CompilerProjectionKind, CompilerStageKind, VerifiedCompilerStagePayload,
};

const DERIVED_PAYLOAD_MAGIC: &[u8; 8] = b"NSCSTG01";
const DERIVED_PAYLOAD_HEADER_WORDS: usize = 3;
const DERIVED_PAYLOAD_HEADER_BYTES: usize =
    DERIVED_PAYLOAD_MAGIC.len() + DERIVED_PAYLOAD_HEADER_WORDS * size_of::<u64>();

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
    let kind_tag = projection_kind_tag(stage)?;
    if checkpoint_words.len() != COMPILER_STAGE_CHECKPOINT_WORD_COUNT
        || checkpoint_words.first().copied() != Some(kind_tag)
    {
        return Err(ArtifactError::new(format!(
            "compiler stage `{}` derived payload has an invalid checkpoint shape",
            stage.as_str()
        )));
    }
    let source_len = u64::try_from(source_payload.len()).map_err(|_| {
        ArtifactError::new("compiler stage derived source payload length exceeds u64")
    })?;
    let word_count = u64::try_from(checkpoint_words.len())
        .map_err(|_| ArtifactError::new("compiler stage derived checkpoint length exceeds u64"))?;
    let capacity = DERIVED_PAYLOAD_HEADER_BYTES
        .checked_add(checkpoint_words.len() * size_of::<u64>())
        .and_then(|bytes| bytes.checked_add(source_payload.len()))
        .ok_or_else(|| ArtifactError::new("compiler stage derived payload length overflow"))?;
    let mut out = Vec::with_capacity(capacity);
    out.extend_from_slice(DERIVED_PAYLOAD_MAGIC);
    out.extend_from_slice(&(kind_tag as u64).to_le_bytes());
    out.extend_from_slice(&source_len.to_le_bytes());
    out.extend_from_slice(&word_count.to_le_bytes());
    for word in checkpoint_words {
        let word = u64::try_from(*word).map_err(|_| {
            ArtifactError::new("compiler stage derived checkpoint word exceeds u64")
        })?;
        out.extend_from_slice(&word.to_le_bytes());
    }
    out.extend_from_slice(source_payload);
    Ok(out)
}

pub(crate) fn decode_payload(
    stage: CompilerStageKind,
    bytes: &[u8],
) -> Result<DecodedCompilerStageTransformationPayload, ArtifactError> {
    if bytes.len() < DERIVED_PAYLOAD_HEADER_BYTES
        || &bytes[..DERIVED_PAYLOAD_MAGIC.len()] != DERIVED_PAYLOAD_MAGIC
    {
        return Err(ArtifactError::new(format!(
            "compiler stage `{}` derived payload has an invalid header",
            stage.as_str()
        )));
    }
    let kind_tag = read_u64(bytes, DERIVED_PAYLOAD_MAGIC.len())?;
    if kind_tag != projection_kind_tag(stage)? as u64 {
        return Err(ArtifactError::new(format!(
            "compiler stage `{}` derived payload kind tag mismatch",
            stage.as_str()
        )));
    }
    let source_len = usize::try_from(read_u64(bytes, DERIVED_PAYLOAD_MAGIC.len() + 8)?)
        .map_err(|_| ArtifactError::new("compiler stage derived source length exceeds usize"))?;
    let word_count = usize::try_from(read_u64(bytes, DERIVED_PAYLOAD_MAGIC.len() + 16)?)
        .map_err(|_| ArtifactError::new("compiler stage derived word count exceeds usize"))?;
    if word_count != COMPILER_STAGE_CHECKPOINT_WORD_COUNT {
        return Err(ArtifactError::new(format!(
            "compiler stage `{}` derived payload checkpoint count mismatch",
            stage.as_str()
        )));
    }
    let checkpoint_bytes = word_count
        .checked_mul(size_of::<u64>())
        .ok_or_else(|| ArtifactError::new("compiler stage derived checkpoint length overflow"))?;
    let payload_offset = DERIVED_PAYLOAD_HEADER_BYTES
        .checked_add(checkpoint_bytes)
        .ok_or_else(|| ArtifactError::new("compiler stage derived payload offset overflow"))?;
    let expected_len = payload_offset
        .checked_add(source_len)
        .ok_or_else(|| ArtifactError::new("compiler stage derived payload length overflow"))?;
    if bytes.len() != expected_len {
        return Err(ArtifactError::new(format!(
            "compiler stage `{}` derived payload length mismatch",
            stage.as_str()
        )));
    }
    let mut checkpoint_words = Vec::with_capacity(word_count);
    for index in 0..word_count {
        let offset = DERIVED_PAYLOAD_HEADER_BYTES + index * size_of::<u64>();
        checkpoint_words.push(usize::try_from(read_u64(bytes, offset)?).map_err(|_| {
            ArtifactError::new("compiler stage derived checkpoint word exceeds usize")
        })?);
    }
    Ok(DecodedCompilerStageTransformationPayload {
        checkpoint_words,
        source_payload: bytes[payload_offset..].to_vec(),
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

fn projection_kind_tag(stage: CompilerStageKind) -> Result<usize, ArtifactError> {
    let kind = match stage {
        CompilerStageKind::Ast => CompilerProjectionKind::Ast,
        CompilerStageKind::Nir => CompilerProjectionKind::Nir,
        _ => {
            return Err(ArtifactError::new(format!(
                "compiler stage `{}` has no derived structural payload",
                stage.as_str()
            )))
        }
    };
    Ok(compiler_projection_checkpoint_kind_tag(kind))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, ArtifactError> {
    let end = offset
        .checked_add(size_of::<u64>())
        .ok_or_else(|| ArtifactError::new("compiler stage derived payload offset overflow"))?;
    let word = bytes
        .get(offset..end)
        .ok_or_else(|| ArtifactError::new("compiler stage derived payload is truncated"))?;
    Ok(u64::from_le_bytes(
        word.try_into().expect("u64 slice length is checked"),
    ))
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
