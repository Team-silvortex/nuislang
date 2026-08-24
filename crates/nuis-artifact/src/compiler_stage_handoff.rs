use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path},
};

use sha2::{Digest, Sha256};

use crate::{
    compiler_structural_projection::{
        parse_compiler_structural_projection, verify_compiler_projection_identity,
        CompilerProjectionKind, COMPILER_AST_PROJECTION_ENCODING, COMPILER_NIR_PROJECTION_ENCODING,
    },
    toml::{
        escape_toml_string, parse_optional_map_usize, parse_required_map_string_in_block,
        parse_required_toml_string, parse_required_toml_usize,
    },
    ArtifactError,
};

pub const COMPILER_STAGE_HANDOFF_PROTOCOL: &str = "nuis-compiler-stage-handoff-v1";
pub const COMPILER_STAGE_PRODUCER_CONTRACT: &str = "nuis-compiler-stage-producer-v1";
const RECORD_IDENTITY_CONTRACT: &str = "nuis-compiler-stage-record-identity-v1";

const ORDERED_STAGES: [CompilerStageKind; 5] = [
    CompilerStageKind::Source,
    CompilerStageKind::Tokens,
    CompilerStageKind::Ast,
    CompilerStageKind::Nir,
    CompilerStageKind::Yir,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerStageKind {
    Source,
    Tokens,
    Ast,
    Nir,
    Yir,
}

impl CompilerStageKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Tokens => "tokens",
            Self::Ast => "ast",
            Self::Nir => "nir",
            Self::Yir => "yir",
        }
    }

    pub fn encoding(self) -> &'static str {
        match self {
            Self::Source => "utf8-lf-v1",
            Self::Tokens => "nuis-token-stream-v1",
            Self::Ast => COMPILER_AST_PROJECTION_ENCODING,
            Self::Nir => COMPILER_NIR_PROJECTION_ENCODING,
            Self::Yir => "yir-text-v1",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "source" => Some(Self::Source),
            "tokens" => Some(Self::Tokens),
            "ast" => Some(Self::Ast),
            "nir" => Some(Self::Nir),
            "yir" => Some(Self::Yir),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CompilerStagePayloadInput<'a> {
    pub stage: CompilerStageKind,
    pub payload_file: &'a str,
    pub bytes: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerStageHandoffRecord {
    pub ordinal: usize,
    pub stage: CompilerStageKind,
    pub encoding: String,
    pub payload_file: String,
    pub payload_bytes: usize,
    pub payload_sha256: String,
    pub parent_sha256: String,
    pub record_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerStageHandoff {
    pub protocol: String,
    pub producer_contract: String,
    pub producer_id: String,
    pub module_domain: String,
    pub module_unit: String,
    pub semantic_root_sha256: String,
    pub bundle_sha256: String,
    pub records: Vec<CompilerStageHandoffRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedCompilerStagePayload {
    pub stage: CompilerStageKind,
    pub bytes: Vec<u8>,
}

pub fn build_compiler_stage_handoff(
    producer_id: &str,
    module_domain: &str,
    module_unit: &str,
    payloads: &[CompilerStagePayloadInput<'_>],
) -> Result<CompilerStageHandoff, ArtifactError> {
    validate_header_values(producer_id, module_domain, module_unit)?;
    if payloads.len() != ORDERED_STAGES.len() {
        return Err(ArtifactError::new(format!(
            "compiler stage handoff requires {} records, found {}",
            ORDERED_STAGES.len(),
            payloads.len()
        )));
    }

    let semantic_root_sha256 = semantic_root_identity(module_domain, module_unit);
    let mut parent_sha256 = semantic_root_sha256.clone();
    let mut records = Vec::with_capacity(payloads.len());
    let mut payload_files = BTreeSet::new();
    for (ordinal, (input, expected_stage)) in payloads.iter().zip(ORDERED_STAGES).enumerate() {
        if input.stage != expected_stage {
            return Err(ArtifactError::new(format!(
                "compiler stage record {ordinal} must be `{}`, found `{}`",
                expected_stage.as_str(),
                input.stage.as_str()
            )));
        }
        validate_payload_file(input.payload_file)?;
        if !payload_files.insert(input.payload_file) {
            return Err(ArtifactError::new(format!(
                "compiler stage payload file `{}` is registered more than once",
                input.payload_file
            )));
        }
        validate_stage_payload(input.stage, input.bytes, module_domain, module_unit)?;
        let payload_sha256 = sha256_hex(input.bytes);
        let record_sha256 = record_identity(
            &semantic_root_sha256,
            &parent_sha256,
            ordinal,
            input.stage,
            input.stage.encoding(),
            input.bytes.len(),
            &payload_sha256,
        );
        records.push(CompilerStageHandoffRecord {
            ordinal,
            stage: input.stage,
            encoding: input.stage.encoding().to_owned(),
            payload_file: input.payload_file.to_owned(),
            payload_bytes: input.bytes.len(),
            payload_sha256,
            parent_sha256,
            record_sha256: record_sha256.clone(),
        });
        parent_sha256 = record_sha256;
    }

    Ok(CompilerStageHandoff {
        protocol: COMPILER_STAGE_HANDOFF_PROTOCOL.to_owned(),
        producer_contract: COMPILER_STAGE_PRODUCER_CONTRACT.to_owned(),
        producer_id: producer_id.to_owned(),
        module_domain: module_domain.to_owned(),
        module_unit: module_unit.to_owned(),
        semantic_root_sha256,
        bundle_sha256: parent_sha256,
        records,
    })
}

pub fn render_compiler_stage_handoff(handoff: &CompilerStageHandoff) -> String {
    let mut out = format!(
        "handoff_protocol = \"{}\"\nproducer_contract = \"{}\"\nproducer_id = \"{}\"\nmodule_domain = \"{}\"\nmodule_unit = \"{}\"\nrecord_count = {}\nsemantic_root_sha256 = \"{}\"\nbundle_sha256 = \"{}\"\n",
        escape_toml_string(&handoff.protocol),
        escape_toml_string(&handoff.producer_contract),
        escape_toml_string(&handoff.producer_id),
        escape_toml_string(&handoff.module_domain),
        escape_toml_string(&handoff.module_unit),
        handoff.records.len(),
        handoff.semantic_root_sha256,
        handoff.bundle_sha256,
    );
    for record in &handoff.records {
        out.push_str(&format!(
            "\n[[record]]\nordinal = {}\nstage = \"{}\"\nencoding = \"{}\"\npayload_file = \"{}\"\npayload_bytes = {}\npayload_sha256 = \"{}\"\nparent_sha256 = \"{}\"\nrecord_sha256 = \"{}\"\n",
            record.ordinal,
            record.stage.as_str(),
            escape_toml_string(&record.encoding),
            escape_toml_string(&record.payload_file),
            record.payload_bytes,
            record.payload_sha256,
            record.parent_sha256,
            record.record_sha256,
        ));
    }
    out
}

pub fn parse_compiler_stage_handoff_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerStageHandoff, ArtifactError> {
    let protocol = parse_required_toml_string(source, "handoff_protocol", path)?;
    let producer_contract = parse_required_toml_string(source, "producer_contract", path)?;
    let producer_id = parse_required_toml_string(source, "producer_id", path)?;
    let module_domain = parse_required_toml_string(source, "module_domain", path)?;
    let module_unit = parse_required_toml_string(source, "module_unit", path)?;
    let record_count = parse_required_toml_usize(source, "record_count", path)?;
    let semantic_root_sha256 = parse_required_toml_string(source, "semantic_root_sha256", path)?;
    let bundle_sha256 = parse_required_toml_string(source, "bundle_sha256", path)?;
    let records = parse_record_blocks(source, path)?
        .into_iter()
        .map(|values| parse_record(values, path))
        .collect::<Result<Vec<_>, _>>()?;
    if record_count != records.len() {
        return Err(ArtifactError::new(format!(
            "`{}` declares {record_count} compiler stage records but contains {}",
            path.display(),
            records.len()
        )));
    }
    let handoff = CompilerStageHandoff {
        protocol,
        producer_contract,
        producer_id,
        module_domain,
        module_unit,
        semantic_root_sha256,
        bundle_sha256,
        records,
    };
    validate_manifest_metadata(&handoff)?;
    Ok(handoff)
}

pub fn read_compiler_stage_handoff(
    manifest_path: &Path,
) -> Result<(CompilerStageHandoff, Vec<VerifiedCompilerStagePayload>), ArtifactError> {
    let source = fs::read_to_string(manifest_path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler stage handoff `{}`: {error}",
            manifest_path.display()
        ))
    })?;
    let handoff = parse_compiler_stage_handoff_from_source(&source, manifest_path)?;
    if render_compiler_stage_handoff(&handoff) != source {
        return Err(ArtifactError::new(format!(
            "compiler stage handoff `{}` is not canonically encoded",
            manifest_path.display()
        )));
    }
    let root = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let canonical_root = root.canonicalize().map_err(|error| {
        ArtifactError::new(format!(
            "failed to resolve compiler stage handoff root `{}`: {error}",
            root.display()
        ))
    })?;
    let mut payloads = Vec::with_capacity(handoff.records.len());
    for record in &handoff.records {
        let payload_path = root.join(&record.payload_file);
        let canonical_payload = payload_path.canonicalize().map_err(|error| {
            ArtifactError::new(format!(
                "failed to resolve compiler stage payload `{}`: {error}",
                payload_path.display()
            ))
        })?;
        if canonical_payload.strip_prefix(&canonical_root).is_err() {
            return Err(ArtifactError::new(format!(
                "compiler stage payload `{}` escapes its handoff root",
                record.payload_file
            )));
        }
        let bytes = fs::read(&canonical_payload).map_err(|error| {
            ArtifactError::new(format!(
                "failed to read compiler stage payload `{}`: {error}",
                canonical_payload.display()
            ))
        })?;
        if bytes.len() != record.payload_bytes || sha256_hex(&bytes) != record.payload_sha256 {
            return Err(ArtifactError::new(format!(
                "compiler stage payload `{}` failed length or SHA-256 verification",
                record.payload_file
            )));
        }
        validate_stage_payload(
            record.stage,
            &bytes,
            &handoff.module_domain,
            &handoff.module_unit,
        )?;
        payloads.push(VerifiedCompilerStagePayload {
            stage: record.stage,
            bytes,
        });
    }
    Ok((handoff, payloads))
}

fn parse_record(
    values: BTreeMap<String, String>,
    path: &Path,
) -> Result<CompilerStageHandoffRecord, ArtifactError> {
    let ordinal = required_map_usize(&values, "ordinal", path)?;
    let stage_name = parse_required_map_string_in_block(&values, "stage", path, "record")?;
    let stage = CompilerStageKind::parse(&stage_name).ok_or_else(|| {
        ArtifactError::new(format!(
            "`{}` record {ordinal} has unsupported stage `{stage_name}`",
            path.display()
        ))
    })?;
    Ok(CompilerStageHandoffRecord {
        ordinal,
        stage,
        encoding: parse_required_map_string_in_block(&values, "encoding", path, "record")?,
        payload_file: parse_required_map_string_in_block(&values, "payload_file", path, "record")?,
        payload_bytes: required_map_usize(&values, "payload_bytes", path)?,
        payload_sha256: parse_required_map_string_in_block(
            &values,
            "payload_sha256",
            path,
            "record",
        )?,
        parent_sha256: parse_required_map_string_in_block(
            &values,
            "parent_sha256",
            path,
            "record",
        )?,
        record_sha256: parse_required_map_string_in_block(
            &values,
            "record_sha256",
            path,
            "record",
        )?,
    })
}

fn parse_record_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<BTreeMap<String, String>>, ArtifactError> {
    let mut blocks = Vec::new();
    let mut current: Option<BTreeMap<String, String>> = None;
    for raw in source.lines() {
        let line = raw.trim();
        if line == "[[record]]" {
            if let Some(values) = current.take() {
                blocks.push(values);
            }
            current = Some(BTreeMap::new());
            continue;
        }
        if line.is_empty() || line.starts_with('#') || current.is_none() {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(ArtifactError::new(format!(
                "`{}` contains malformed compiler stage record line `{line}`",
                path.display()
            )));
        };
        let values = current.as_mut().expect("record block is active");
        if values
            .insert(key.trim().to_owned(), value.trim().to_owned())
            .is_some()
        {
            return Err(ArtifactError::new(format!(
                "`{}` contains duplicate compiler stage record key `{}`",
                path.display(),
                key.trim()
            )));
        }
    }
    if let Some(values) = current {
        blocks.push(values);
    }
    Ok(blocks)
}

fn validate_manifest_metadata(handoff: &CompilerStageHandoff) -> Result<(), ArtifactError> {
    if handoff.protocol != COMPILER_STAGE_HANDOFF_PROTOCOL {
        return Err(ArtifactError::new(format!(
            "unsupported compiler stage handoff protocol `{}`",
            handoff.protocol
        )));
    }
    if handoff.producer_contract != COMPILER_STAGE_PRODUCER_CONTRACT {
        return Err(ArtifactError::new(format!(
            "unsupported compiler stage producer contract `{}`",
            handoff.producer_contract
        )));
    }
    validate_header_values(
        &handoff.producer_id,
        &handoff.module_domain,
        &handoff.module_unit,
    )?;
    if handoff.records.len() != ORDERED_STAGES.len() {
        return Err(ArtifactError::new(format!(
            "compiler stage handoff requires {} records, found {}",
            ORDERED_STAGES.len(),
            handoff.records.len()
        )));
    }
    let semantic_root = semantic_root_identity(&handoff.module_domain, &handoff.module_unit);
    if handoff.semantic_root_sha256 != semantic_root {
        return Err(ArtifactError::new(
            "compiler stage semantic root identity does not match its module header",
        ));
    }
    let mut parent = semantic_root.clone();
    let mut payload_files = BTreeSet::new();
    for (ordinal, (record, expected_stage)) in
        handoff.records.iter().zip(ORDERED_STAGES).enumerate()
    {
        if record.ordinal != ordinal || record.stage != expected_stage {
            return Err(ArtifactError::new(format!(
                "compiler stage record {ordinal} must be `{}` in canonical order",
                expected_stage.as_str()
            )));
        }
        if record.encoding != record.stage.encoding() {
            return Err(ArtifactError::new(format!(
                "compiler stage `{}` requires encoding `{}`",
                record.stage.as_str(),
                record.stage.encoding()
            )));
        }
        validate_payload_file(&record.payload_file)?;
        if !payload_files.insert(record.payload_file.as_str()) {
            return Err(ArtifactError::new(format!(
                "compiler stage payload file `{}` is registered more than once",
                record.payload_file
            )));
        }
        validate_sha256(&record.payload_sha256, "payload")?;
        if record.parent_sha256 != parent {
            return Err(ArtifactError::new(format!(
                "compiler stage `{}` has a broken parent identity",
                record.stage.as_str()
            )));
        }
        let expected_record = record_identity(
            &semantic_root,
            &parent,
            ordinal,
            record.stage,
            &record.encoding,
            record.payload_bytes,
            &record.payload_sha256,
        );
        if record.record_sha256 != expected_record {
            return Err(ArtifactError::new(format!(
                "compiler stage `{}` record identity does not match its metadata",
                record.stage.as_str()
            )));
        }
        parent = expected_record;
    }
    if handoff.bundle_sha256 != parent {
        return Err(ArtifactError::new(
            "compiler stage bundle identity does not match the final record",
        ));
    }
    Ok(())
}

fn validate_header_values(
    producer_id: &str,
    module_domain: &str,
    module_unit: &str,
) -> Result<(), ArtifactError> {
    for (label, value) in [
        ("producer", producer_id),
        ("module domain", module_domain),
        ("module unit", module_unit),
    ] {
        if value.is_empty()
            || value.trim() != value
            || value.chars().any(|character| character.is_control())
        {
            return Err(ArtifactError::new(format!(
                "compiler stage {label} must be non-empty canonical text without surrounding whitespace or control characters"
            )));
        }
    }
    Ok(())
}

fn validate_payload_file(value: &str) -> Result<(), ArtifactError> {
    let path = Path::new(value);
    let mut components = path.components();
    if value.contains(['/', '\\'])
        || !matches!(components.next(), Some(Component::Normal(_)))
        || components.next().is_some()
    {
        return Err(ArtifactError::new(format!(
            "compiler stage payload path `{value}` must be one relative file name"
        )));
    }
    Ok(())
}

fn validate_text_payload(stage: CompilerStageKind, bytes: &[u8]) -> Result<(), ArtifactError> {
    let source = std::str::from_utf8(bytes).map_err(|error| {
        ArtifactError::new(format!(
            "compiler stage `{}` payload is not UTF-8: {error}",
            stage.as_str()
        ))
    })?;
    if source.contains('\r') || source.contains('\0') {
        return Err(ArtifactError::new(format!(
            "compiler stage `{}` payload violates the UTF-8/LF text contract",
            stage.as_str()
        )));
    }
    Ok(())
}

fn validate_stage_payload(
    stage: CompilerStageKind,
    bytes: &[u8],
    module_domain: &str,
    module_unit: &str,
) -> Result<(), ArtifactError> {
    validate_text_payload(stage, bytes)?;
    let projection_kind = match stage {
        CompilerStageKind::Ast => Some(CompilerProjectionKind::Ast),
        CompilerStageKind::Nir => Some(CompilerProjectionKind::Nir),
        _ => None,
    };
    let Some(projection_kind) = projection_kind else {
        return Ok(());
    };
    let source = std::str::from_utf8(bytes)
        .expect("compiler structural projection was already validated as UTF-8");
    let projection = parse_compiler_structural_projection(projection_kind, source)?;
    verify_compiler_projection_identity(&projection, module_domain, module_unit)
}

fn required_map_usize(
    values: &BTreeMap<String, String>,
    key: &str,
    path: &Path,
) -> Result<usize, ArtifactError> {
    parse_optional_map_usize(values, key, path, "record")?.ok_or_else(|| {
        ArtifactError::new(format!(
            "`{}` record block is missing required key `{key}`",
            path.display()
        ))
    })
}

fn validate_sha256(value: &str, label: &str) -> Result<(), ArtifactError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Ok(());
    }
    Err(ArtifactError::new(format!(
        "compiler stage {label} identity must be lowercase SHA-256"
    )))
}

fn semantic_root_identity(module_domain: &str, module_unit: &str) -> String {
    sha256_fields(&[
        COMPILER_STAGE_HANDOFF_PROTOCOL.as_bytes(),
        COMPILER_STAGE_PRODUCER_CONTRACT.as_bytes(),
        module_domain.as_bytes(),
        module_unit.as_bytes(),
    ])
}

fn record_identity(
    semantic_root: &str,
    parent: &str,
    ordinal: usize,
    stage: CompilerStageKind,
    encoding: &str,
    payload_bytes: usize,
    payload_sha256: &str,
) -> String {
    sha256_fields(&[
        RECORD_IDENTITY_CONTRACT.as_bytes(),
        semantic_root.as_bytes(),
        parent.as_bytes(),
        &(ordinal as u64).to_le_bytes(),
        stage.as_str().as_bytes(),
        encoding.as_bytes(),
        &(payload_bytes as u64).to_le_bytes(),
        payload_sha256.as_bytes(),
    ])
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn sha256_fields(fields: &[&[u8]]) -> String {
    let mut hash = Sha256::new();
    for field in fields {
        hash.update((field.len() as u64).to_le_bytes());
        hash.update(field);
    }
    let digest = hash.finalize();
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[cfg(test)]
#[path = "compiler_stage_handoff_tests.rs"]
mod tests;
