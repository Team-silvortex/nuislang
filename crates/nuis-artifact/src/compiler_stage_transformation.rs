use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use sha2::{Digest, Sha256};

#[path = "compiler_stage_transformation_payload.rs"]
mod payload;

use crate::{
    compiler_projection_two_page_identity,
    toml::{
        escape_toml_string, parse_optional_map_usize, parse_required_map_string_in_block,
        parse_required_toml_bool, parse_required_toml_string, parse_required_toml_usize,
    },
    ArtifactError, CompilerProjectionKind, CompilerProjectionTwoPageIdentity, CompilerStageHandoff,
    CompilerStageKind, VerifiedCompilerStagePayload,
};

pub const COMPILER_STAGE_TRANSFORMATION_PROTOCOL: &str = "nuis-compiler-stage-transformation-v3";
pub const COMPILER_STAGE_TRANSFORMATION_PRODUCER_CONTRACT: &str =
    "nuis-compiler-stage-transformation-producer-v3";
pub const COMPILER_STAGE_TRANSFORMATION_AUTHORITY: &str =
    "stage1-compact-structured-transformation-evidence-no-replacement";
pub const COMPILER_STAGE_TRANSFORMATION_FILE: &str = "nuis.compiler-stage-transformations.toml";
pub const COMPILER_STAGE_STRUCTURED_RECORD_CONTRACT: &str =
    "nuis-compiler-structured-record-codec-v1";
pub const COMPILER_STAGE_TRANSFORMATION_OUTPUT_ENCODING: &str =
    "nuis-derived-structural-records-v2";
pub const COMPILER_STAGE_CHECKPOINT_PAGE_COUNT: usize = 2;
pub const COMPILER_STAGE_CHECKPOINT_WORD_COUNT: usize = 22;

#[derive(Debug, Clone, Copy)]
pub struct CompilerStageTransformationRecordInput<'a> {
    pub source_stage: CompilerStageKind,
    pub transform_contract: &'a str,
    pub output_encoding: &'a str,
    pub output_words: &'a [usize],
}

#[derive(Debug, Clone, Copy)]
pub struct CompilerStageTransformationsInput<'a> {
    pub producer_id: &'a str,
    pub handoff: &'a CompilerStageHandoff,
    pub payloads: &'a [VerifiedCompilerStagePayload],
    pub records: &'a [CompilerStageTransformationRecordInput<'a>],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerStageTransformationRecord {
    pub ordinal: usize,
    pub source_stage: CompilerStageKind,
    pub input_payload_bytes: usize,
    pub input_payload_sha256: String,
    pub transform_contract: String,
    pub output_encoding: String,
    pub output_word_count: usize,
    pub output_checkpoint_sha256: String,
    pub output_payload_file: String,
    pub output_payload_bytes: usize,
    pub output_payload_sha256: String,
    pub output_words: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerStageTransformations {
    pub protocol: String,
    pub producer_contract: String,
    pub authority: String,
    pub producer_id: String,
    pub stage_handoff_bundle_sha256: String,
    pub record_count: usize,
    pub replacement_authorized: bool,
    pub proof_sha256: String,
    pub records: Vec<CompilerStageTransformationRecord>,
}

pub fn compiler_projection_checkpoint_kind_tag(kind: CompilerProjectionKind) -> usize {
    match kind {
        CompilerProjectionKind::Ast => 1,
        CompilerProjectionKind::Nir => 2,
    }
}

pub fn compiler_stage_structural_checkpoint_words(
    kind: CompilerProjectionKind,
    pages: CompilerProjectionTwoPageIdentity,
) -> [usize; COMPILER_STAGE_CHECKPOINT_WORD_COUNT] {
    let first_lanes = pages.first.cursor.lanes();
    let second_lanes = pages.second.cursor.lanes();
    let mut words = [0; COMPILER_STAGE_CHECKPOINT_WORD_COUNT];
    words[0] = compiler_projection_checkpoint_kind_tag(kind);
    words[1] = COMPILER_STAGE_CHECKPOINT_PAGE_COUNT;
    words[2] = pages.first.page.identity;
    words[3] = pages.first.cursor_identity;
    words[4..12].copy_from_slice(&first_lanes);
    words[12] = pages.second.page.identity;
    words[13] = pages.second.cursor_identity;
    words[14..22].copy_from_slice(&second_lanes);
    words
}

pub fn compiler_stage_transformation_payload_file(ordinal: usize) -> String {
    payload::payload_file(ordinal)
}

pub fn encode_compiler_stage_transformation_payload(
    stage: CompilerStageKind,
    source_payload: &[u8],
    checkpoint_words: &[usize],
) -> Result<Vec<u8>, ArtifactError> {
    payload::encode_payload(stage, source_payload, checkpoint_words)
}

pub(crate) fn decode_compiler_stage_transformation_payload(
    stage: CompilerStageKind,
    bytes: &[u8],
) -> Result<(Vec<usize>, Vec<u8>), ArtifactError> {
    let decoded = payload::decode_payload(stage, bytes)?;
    Ok((decoded.checkpoint_words, decoded.source_payload))
}

pub fn materialize_compiler_stage_transformation_payloads(
    root: &Path,
    manifest: &CompilerStageTransformations,
    handoff: &CompilerStageHandoff,
    payloads: &[VerifiedCompilerStagePayload],
) -> Result<(), ArtifactError> {
    verify_compiler_stage_transformations(manifest, handoff, payloads)?;
    payload::materialize_payloads(root, manifest, payloads)
}

pub(crate) fn verify_compiler_stage_transformation_payloads(
    root: &Path,
    manifest: &CompilerStageTransformations,
    payloads: &[VerifiedCompilerStagePayload],
) -> Result<(), ArtifactError> {
    payload::validate_materialized_payloads(root, manifest, payloads)
}

pub fn build_compiler_stage_transformations(
    input: &CompilerStageTransformationsInput<'_>,
) -> Result<CompilerStageTransformations, ArtifactError> {
    validate_input_binding(input)?;
    let mut records = Vec::with_capacity(input.records.len());
    for (ordinal, record) in input.records.iter().enumerate() {
        let payload = payload_for_stage(input.payloads, record.source_stage)?;
        let output_payload =
            payload::encode_payload(record.source_stage, &payload.bytes, record.output_words)?;
        records.push(CompilerStageTransformationRecord {
            ordinal,
            source_stage: record.source_stage,
            input_payload_bytes: payload.bytes.len(),
            input_payload_sha256: sha256_hex(&payload.bytes),
            transform_contract: record.transform_contract.to_owned(),
            output_encoding: record.output_encoding.to_owned(),
            output_word_count: record.output_words.len(),
            output_checkpoint_sha256: words_sha256(record.output_words),
            output_payload_file: payload::payload_file(ordinal),
            output_payload_bytes: output_payload.len(),
            output_payload_sha256: sha256_hex(&output_payload),
            output_words: record.output_words.to_vec(),
        });
    }
    let mut manifest = CompilerStageTransformations {
        protocol: COMPILER_STAGE_TRANSFORMATION_PROTOCOL.to_owned(),
        producer_contract: COMPILER_STAGE_TRANSFORMATION_PRODUCER_CONTRACT.to_owned(),
        authority: COMPILER_STAGE_TRANSFORMATION_AUTHORITY.to_owned(),
        producer_id: input.producer_id.to_owned(),
        stage_handoff_bundle_sha256: input.handoff.bundle_sha256.clone(),
        record_count: records.len(),
        replacement_authorized: false,
        proof_sha256: String::new(),
        records,
    };
    manifest.proof_sha256 = manifest_identity(&manifest);
    validate_manifest(&manifest)?;
    Ok(manifest)
}

pub fn render_compiler_stage_transformations(manifest: &CompilerStageTransformations) -> String {
    let mut out = format!(
        "protocol = \"{}\"\nproducer_contract = \"{}\"\nauthority = \"{}\"\nproducer_id = \"{}\"\nstage_handoff_bundle_sha256 = \"{}\"\nrecord_count = {}\nreplacement_authorized = {}\nproof_sha256 = \"{}\"\n",
        manifest.protocol,
        manifest.producer_contract,
        manifest.authority,
        escape_toml_string(&manifest.producer_id),
        manifest.stage_handoff_bundle_sha256,
        manifest.record_count,
        manifest.replacement_authorized,
        manifest.proof_sha256,
    );
    for record in &manifest.records {
        out.push_str(&format!(
            "\n[[record]]\nordinal = {}\nsource_stage = \"{}\"\ninput_payload_bytes = {}\ninput_payload_sha256 = \"{}\"\ntransform_contract = \"{}\"\noutput_encoding = \"{}\"\noutput_word_count = {}\noutput_checkpoint_sha256 = \"{}\"\noutput_payload_file = \"{}\"\noutput_payload_bytes = {}\noutput_payload_sha256 = \"{}\"\n",
            record.ordinal,
            record.source_stage.as_str(),
            record.input_payload_bytes,
            record.input_payload_sha256,
            record.transform_contract,
            record.output_encoding,
            record.output_word_count,
            record.output_checkpoint_sha256,
            escape_toml_string(&record.output_payload_file),
            record.output_payload_bytes,
            record.output_payload_sha256,
        ));
        for (index, word) in record.output_words.iter().enumerate() {
            out.push_str(&format!("output_word_{index} = {word}\n"));
        }
    }
    out
}

pub fn parse_compiler_stage_transformations(
    path: &Path,
) -> Result<CompilerStageTransformations, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler stage transformations `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_stage_transformations_from_source(&source, path)
}

pub fn parse_compiler_stage_transformations_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerStageTransformations, ArtifactError> {
    validate_text(source, path)?;
    let records = parse_record_blocks(source, path)?
        .into_iter()
        .map(|values| parse_record(values, path))
        .collect::<Result<Vec<_>, _>>()?;
    let manifest = CompilerStageTransformations {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        producer_contract: parse_required_toml_string(source, "producer_contract", path)?,
        authority: parse_required_toml_string(source, "authority", path)?,
        producer_id: parse_required_toml_string(source, "producer_id", path)?,
        stage_handoff_bundle_sha256: parse_required_toml_string(
            source,
            "stage_handoff_bundle_sha256",
            path,
        )?,
        record_count: parse_required_toml_usize(source, "record_count", path)?,
        replacement_authorized: parse_required_toml_bool(source, "replacement_authorized", path)?,
        proof_sha256: parse_required_toml_string(source, "proof_sha256", path)?,
        records,
    };
    validate_manifest(&manifest)?;
    if render_compiler_stage_transformations(&manifest) != source {
        return Err(ArtifactError::new(format!(
            "compiler stage transformations `{}` are not canonically encoded",
            path.display()
        )));
    }
    Ok(manifest)
}

pub fn read_compiler_stage_transformations(
    path: &Path,
    handoff: &CompilerStageHandoff,
    payloads: &[VerifiedCompilerStagePayload],
) -> Result<CompilerStageTransformations, ArtifactError> {
    let manifest = parse_compiler_stage_transformations(path)?;
    validate_bound_evidence(&manifest, handoff, payloads)?;
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    verify_compiler_stage_transformation_payloads(root, &manifest, payloads)?;
    Ok(manifest)
}

pub fn verify_compiler_stage_transformations(
    manifest: &CompilerStageTransformations,
    handoff: &CompilerStageHandoff,
    payloads: &[VerifiedCompilerStagePayload],
) -> Result<(), ArtifactError> {
    validate_manifest(manifest)?;
    validate_bound_evidence(manifest, handoff, payloads)
}

fn validate_input_binding(
    input: &CompilerStageTransformationsInput<'_>,
) -> Result<(), ArtifactError> {
    if input.producer_id != input.handoff.producer_id {
        return Err(ArtifactError::new(
            "compiler stage transformation producer does not match its handoff",
        ));
    }
    validate_handoff_payloads(input.handoff, input.payloads)?;
    if input.records.is_empty() {
        return Err(ArtifactError::new(
            "compiler stage transformations require at least one record",
        ));
    }
    let mut stages = BTreeSet::new();
    for record in input.records {
        validate_record_contract(
            record.source_stage,
            record.transform_contract,
            record.output_encoding,
            record.output_words,
        )?;
        if !stages.insert(record.source_stage.as_str()) {
            return Err(ArtifactError::new(format!(
                "compiler stage `{}` is transformed more than once",
                record.source_stage.as_str()
            )));
        }
        let payload = payload_for_stage(input.payloads, record.source_stage)?;
        let kind = projection_kind(record.source_stage)?;
        let pages = compiler_projection_two_page_identity(kind, &payload.bytes)?;
        let expected = compiler_stage_structural_checkpoint_words(kind, pages);
        if record.output_words != expected {
            return Err(ArtifactError::new(format!(
                "compiler stage `{}` transformation output does not match independent structural replay",
                record.source_stage.as_str()
            )));
        }
    }
    Ok(())
}

fn validate_bound_evidence(
    manifest: &CompilerStageTransformations,
    handoff: &CompilerStageHandoff,
    payloads: &[VerifiedCompilerStagePayload],
) -> Result<(), ArtifactError> {
    if manifest.producer_id != handoff.producer_id
        || manifest.stage_handoff_bundle_sha256 != handoff.bundle_sha256
    {
        return Err(ArtifactError::new(
            "compiler stage transformations do not bind their stage handoff",
        ));
    }
    validate_handoff_payloads(handoff, payloads)?;
    for record in &manifest.records {
        let payload = payload_for_stage(payloads, record.source_stage)?;
        if record.input_payload_bytes != payload.bytes.len()
            || record.input_payload_sha256 != sha256_hex(&payload.bytes)
        {
            return Err(ArtifactError::new(format!(
                "compiler stage `{}` transformation input identity mismatch",
                record.source_stage.as_str()
            )));
        }
        let kind = projection_kind(record.source_stage)?;
        let pages = compiler_projection_two_page_identity(kind, &payload.bytes)?;
        if record.output_words != compiler_stage_structural_checkpoint_words(kind, pages) {
            return Err(ArtifactError::new(format!(
                "compiler stage `{}` transformation failed independent replay",
                record.source_stage.as_str()
            )));
        }
        let output =
            payload::encode_payload(record.source_stage, &payload.bytes, &record.output_words)?;
        payload::validate_output_identity(record, &output)?;
    }
    Ok(())
}

fn validate_handoff_payloads(
    handoff: &CompilerStageHandoff,
    payloads: &[VerifiedCompilerStagePayload],
) -> Result<(), ArtifactError> {
    if handoff.records.len() != payloads.len() {
        return Err(ArtifactError::new(
            "compiler stage transformation handoff payload count mismatch",
        ));
    }
    for (record, payload) in handoff.records.iter().zip(payloads) {
        if record.stage != payload.stage
            || record.payload_bytes != payload.bytes.len()
            || record.payload_sha256 != sha256_hex(&payload.bytes)
        {
            return Err(ArtifactError::new(format!(
                "compiler stage `{}` transformation payload identity mismatch",
                record.stage.as_str()
            )));
        }
    }
    Ok(())
}

fn validate_manifest(manifest: &CompilerStageTransformations) -> Result<(), ArtifactError> {
    if manifest.protocol != COMPILER_STAGE_TRANSFORMATION_PROTOCOL
        || manifest.producer_contract != COMPILER_STAGE_TRANSFORMATION_PRODUCER_CONTRACT
        || manifest.authority != COMPILER_STAGE_TRANSFORMATION_AUTHORITY
        || manifest.replacement_authorized
    {
        return Err(ArtifactError::new(
            "compiler stage transformations declare an unsupported authority contract",
        ));
    }
    validate_token(&manifest.producer_id, "transformation producer")?;
    validate_sha256(
        &manifest.stage_handoff_bundle_sha256,
        "stage handoff bundle",
    )?;
    validate_sha256(&manifest.proof_sha256, "transformation proof")?;
    if manifest.record_count == 0 || manifest.record_count != manifest.records.len() {
        return Err(ArtifactError::new(
            "compiler stage transformation record count is invalid",
        ));
    }
    let mut stages = BTreeSet::new();
    for (ordinal, record) in manifest.records.iter().enumerate() {
        if record.ordinal != ordinal || !stages.insert(record.source_stage.as_str()) {
            return Err(ArtifactError::new(
                "compiler stage transformation records are not in unique canonical order",
            ));
        }
        validate_record_contract(
            record.source_stage,
            &record.transform_contract,
            &record.output_encoding,
            &record.output_words,
        )?;
        if record.input_payload_bytes == 0
            || record.output_word_count != record.output_words.len()
            || record.output_checkpoint_sha256 != words_sha256(&record.output_words)
            || record.output_payload_file != payload::payload_file(record.ordinal)
            || record.output_payload_bytes == 0
        {
            return Err(ArtifactError::new(format!(
                "compiler stage `{}` transformation record identity is invalid",
                record.source_stage.as_str()
            )));
        }
        validate_sha256(&record.input_payload_sha256, "transformation input payload")?;
        validate_sha256(
            &record.output_checkpoint_sha256,
            "transformation output checkpoint",
        )?;
        validate_sha256(
            &record.output_payload_sha256,
            "transformation output payload",
        )?;
    }
    if manifest.proof_sha256 != manifest_identity(manifest) {
        return Err(ArtifactError::new(
            "compiler stage transformation proof identity mismatch",
        ));
    }
    Ok(())
}

fn validate_record_contract(
    stage: CompilerStageKind,
    transform_contract: &str,
    output_encoding: &str,
    output_words: &[usize],
) -> Result<(), ArtifactError> {
    projection_kind(stage)?;
    if transform_contract != COMPILER_STAGE_STRUCTURED_RECORD_CONTRACT
        || output_encoding != COMPILER_STAGE_TRANSFORMATION_OUTPUT_ENCODING
        || output_words.len() != COMPILER_STAGE_CHECKPOINT_WORD_COUNT
        || output_words[0] != compiler_projection_checkpoint_kind_tag(projection_kind(stage)?)
        || output_words[1] != COMPILER_STAGE_CHECKPOINT_PAGE_COUNT
    {
        return Err(ArtifactError::new(format!(
            "compiler stage `{}` uses an unsupported transformation contract or output shape",
            stage.as_str()
        )));
    }
    Ok(())
}

fn projection_kind(stage: CompilerStageKind) -> Result<CompilerProjectionKind, ArtifactError> {
    match stage {
        CompilerStageKind::Ast => Ok(CompilerProjectionKind::Ast),
        CompilerStageKind::Nir => Ok(CompilerProjectionKind::Nir),
        _ => Err(ArtifactError::new(format!(
            "compiler stage `{}` has no structural checkpoint transformation",
            stage.as_str()
        ))),
    }
}

fn payload_for_stage(
    payloads: &[VerifiedCompilerStagePayload],
    stage: CompilerStageKind,
) -> Result<&VerifiedCompilerStagePayload, ArtifactError> {
    payloads
        .iter()
        .find(|payload| payload.stage == stage)
        .ok_or_else(|| {
            ArtifactError::new(format!(
                "compiler stage `{}` transformation payload is missing",
                stage.as_str()
            ))
        })
}

fn parse_record(
    values: BTreeMap<String, String>,
    path: &Path,
) -> Result<CompilerStageTransformationRecord, ArtifactError> {
    let ordinal = required_map_usize(&values, "ordinal", path)?;
    let stage_name =
        parse_required_map_string_in_block(&values, "source_stage", path, "transformation record")?;
    let source_stage = CompilerStageKind::parse(&stage_name).ok_or_else(|| {
        ArtifactError::new(format!(
            "`{}` transformation record {ordinal} has unsupported stage `{stage_name}`",
            path.display()
        ))
    })?;
    let output_word_count = required_map_usize(&values, "output_word_count", path)?;
    let mut output_words = Vec::with_capacity(output_word_count);
    for index in 0..output_word_count {
        output_words.push(required_map_usize(
            &values,
            &format!("output_word_{index}"),
            path,
        )?);
    }
    if values.len() != 11 + output_word_count {
        return Err(ArtifactError::new(format!(
            "`{}` transformation record {ordinal} contains unknown or missing keys",
            path.display()
        )));
    }
    Ok(CompilerStageTransformationRecord {
        ordinal,
        source_stage,
        input_payload_bytes: required_map_usize(&values, "input_payload_bytes", path)?,
        input_payload_sha256: parse_required_map_string_in_block(
            &values,
            "input_payload_sha256",
            path,
            "transformation record",
        )?,
        transform_contract: parse_required_map_string_in_block(
            &values,
            "transform_contract",
            path,
            "transformation record",
        )?,
        output_encoding: parse_required_map_string_in_block(
            &values,
            "output_encoding",
            path,
            "transformation record",
        )?,
        output_word_count,
        output_checkpoint_sha256: parse_required_map_string_in_block(
            &values,
            "output_checkpoint_sha256",
            path,
            "transformation record",
        )?,
        output_payload_file: parse_required_map_string_in_block(
            &values,
            "output_payload_file",
            path,
            "transformation record",
        )?,
        output_payload_bytes: required_map_usize(&values, "output_payload_bytes", path)?,
        output_payload_sha256: parse_required_map_string_in_block(
            &values,
            "output_payload_sha256",
            path,
            "transformation record",
        )?,
        output_words,
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
        let (key, value) = line.split_once('=').ok_or_else(|| {
            ArtifactError::new(format!(
                "`{}` contains malformed transformation record line `{line}`",
                path.display()
            ))
        })?;
        let values = current.as_mut().expect("record block is active");
        if values
            .insert(key.trim().to_owned(), value.trim().to_owned())
            .is_some()
        {
            return Err(ArtifactError::new(format!(
                "`{}` contains duplicate transformation record key `{}`",
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

fn required_map_usize(
    values: &BTreeMap<String, String>,
    key: &str,
    path: &Path,
) -> Result<usize, ArtifactError> {
    parse_optional_map_usize(values, key, path, "transformation record")?.ok_or_else(|| {
        ArtifactError::new(format!(
            "`{}` transformation record is missing required key `{key}`",
            path.display()
        ))
    })
}

fn validate_text(source: &str, path: &Path) -> Result<(), ArtifactError> {
    if source.is_empty()
        || !source.ends_with('\n')
        || source.contains('\r')
        || source.contains('\0')
    {
        return Err(ArtifactError::new(format!(
            "compiler stage transformations `{}` must use canonical UTF-8/LF text",
            path.display()
        )));
    }
    Ok(())
}

fn validate_token(value: &str, label: &str) -> Result<(), ArtifactError> {
    if value.is_empty()
        || value.trim() != value
        || value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(ArtifactError::new(format!(
            "compiler stage {label} must be canonical non-whitespace text"
        )));
    }
    Ok(())
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

fn words_sha256(words: &[usize]) -> String {
    let mut hash = Sha256::new();
    for word in words {
        hash.update((*word as u64).to_le_bytes());
    }
    format!("{:x}", hash.finalize())
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn manifest_identity(manifest: &CompilerStageTransformations) -> String {
    let mut hash = Sha256::new();
    for value in [
        manifest.protocol.as_bytes(),
        manifest.producer_contract.as_bytes(),
        manifest.authority.as_bytes(),
        manifest.producer_id.as_bytes(),
        manifest.stage_handoff_bundle_sha256.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    hash_field(&mut hash, &(manifest.record_count as u64).to_le_bytes());
    hash_field(
        &mut hash,
        &(usize::from(manifest.replacement_authorized) as u64).to_le_bytes(),
    );
    for record in &manifest.records {
        for value in [
            (record.ordinal as u64).to_le_bytes(),
            (record.input_payload_bytes as u64).to_le_bytes(),
            (record.output_word_count as u64).to_le_bytes(),
            (record.output_payload_bytes as u64).to_le_bytes(),
        ] {
            hash_field(&mut hash, &value);
        }
        for value in [
            record.source_stage.as_str().as_bytes(),
            record.input_payload_sha256.as_bytes(),
            record.transform_contract.as_bytes(),
            record.output_encoding.as_bytes(),
            record.output_checkpoint_sha256.as_bytes(),
            record.output_payload_file.as_bytes(),
            record.output_payload_sha256.as_bytes(),
        ] {
            hash_field(&mut hash, value);
        }
        for word in &record.output_words {
            hash_field(&mut hash, &(*word as u64).to_le_bytes());
        }
    }
    format!("{:x}", hash.finalize())
}

fn hash_field(hash: &mut Sha256, value: &[u8]) {
    hash.update((value.len() as u64).to_le_bytes());
    hash.update(value);
}

#[cfg(test)]
#[path = "compiler_stage_transformation_tests.rs"]
mod tests;
