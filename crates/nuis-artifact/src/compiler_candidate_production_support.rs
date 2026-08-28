use std::{collections::BTreeMap, path::Path};

use sha2::{Digest, Sha256};

use super::CompilerCandidateProductionRecord;
use crate::{
    toml::{parse_optional_map_usize, parse_required_map_string_in_block},
    ArtifactError,
};

pub(super) fn parse_record_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<CompilerCandidateProductionRecord>, ArtifactError> {
    let mut records = Vec::new();
    let mut values = BTreeMap::new();
    let mut in_block = false;
    for raw in source.lines() {
        let line = raw.trim();
        if line == "[[record]]" {
            if in_block {
                records.push(parse_record(&values, path)?);
                values.clear();
            }
            in_block = true;
            continue;
        }
        if line.starts_with('[') {
            if in_block {
                records.push(parse_record(&values, path)?);
                values.clear();
                in_block = false;
            }
            continue;
        }
        if in_block && !line.is_empty() && !line.starts_with('#') {
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_owned();
                if values
                    .insert(key.clone(), value.trim().to_owned())
                    .is_some()
                {
                    return Err(ArtifactError::new(format!(
                        "`{}` candidate production record repeats key `{key}`",
                        path.display()
                    )));
                }
            }
        }
    }
    if in_block {
        records.push(parse_record(&values, path)?);
    }
    Ok(records)
}

fn parse_record(
    values: &BTreeMap<String, String>,
    path: &Path,
) -> Result<CompilerCandidateProductionRecord, ArtifactError> {
    Ok(CompilerCandidateProductionRecord {
        ordinal: required_block_usize(values, "ordinal", path)?,
        stage: parse_required_map_string_in_block(values, "stage", path, "record")?,
        payload_bytes: required_block_usize(values, "payload_bytes", path)?,
        payload_sha256: parse_required_map_string_in_block(
            values,
            "payload_sha256",
            path,
            "record",
        )?,
        fold: required_block_usize(values, "fold", path)?,
    })
}

fn required_block_usize(
    values: &BTreeMap<String, String>,
    key: &str,
    path: &Path,
) -> Result<usize, ArtifactError> {
    parse_optional_map_usize(values, key, path, "record")?.ok_or_else(|| {
        ArtifactError::new(format!(
            "`{}` candidate production record is missing `{key}`",
            path.display()
        ))
    })
}

pub(super) fn validate_text(source: &str, path: &Path) -> Result<(), ArtifactError> {
    if source.is_empty()
        || !source.ends_with('\n')
        || source.contains('\r')
        || source.contains('\0')
    {
        return Err(ArtifactError::new(format!(
            "compiler candidate production `{}` must be canonical UTF-8/LF text",
            path.display()
        )));
    }
    Ok(())
}

pub(super) fn validate_token(value: &str, label: &str) -> Result<(), ArtifactError> {
    if !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Ok(());
    }
    Err(ArtifactError::new(format!(
        "compiler candidate production {label} is not a stable token"
    )))
}

pub(super) fn validate_file_name(value: &str, label: &str) -> Result<(), ArtifactError> {
    let path = Path::new(value);
    if !value.is_empty()
        && !value.contains('/')
        && !value.contains('\\')
        && path.components().count() == 1
        && path.file_name().and_then(|item| item.to_str()) == Some(value)
    {
        return Ok(());
    }
    Err(ArtifactError::new(format!(
        "compiler candidate production {label} must be a sibling file"
    )))
}

pub(super) fn validate_sha256(value: &str, label: &str) -> Result<(), ArtifactError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Ok(());
    }
    Err(ArtifactError::new(format!(
        "compiler candidate production {label} must be lowercase SHA-256"
    )))
}

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
