use std::{collections::BTreeMap, fs, path::Path};

use sha2::{Digest, Sha256};

use crate::{
    toml::{
        escape_toml_string, parse_optional_map_usize, parse_required_map_string_in_block,
        parse_required_toml_bool, parse_required_toml_string, parse_required_toml_usize,
    },
    ArtifactError, CompilerCandidateExecution, CompilerComponentBuild, CompilerStageHandoff,
    VerifiedCompilerStagePayload, COMPILER_COMPONENT_STAGE0_ROLE,
    COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE,
};

pub const COMPILER_CANDIDATE_PRODUCTION_PROTOCOL: &str = "nuis-compiler-candidate-production-v1";
pub const COMPILER_CANDIDATE_PRODUCER_CONTRACT: &str = "nuis-stage1-scalar-byte-producer-v1";
pub const COMPILER_CANDIDATE_PRODUCTION_AUTHORITY: &str =
    "stage1-candidate-component-production-no-replacement";
pub const COMPILER_CANDIDATE_PRODUCTION_FILE: &str = "nuis.compiler-candidate-production.toml";
pub const COMPILER_CANDIDATE_ADAPTER_FILE: &str = "nuis.compiler-candidate-adapter";

const FOLD_MODULUS: u64 = 2_147_483_629;
const EXPECTED_STAGE_COUNT: usize = 5;

#[derive(Debug, Clone, Copy)]
pub struct CompilerCandidateProductionInput<'a> {
    pub stage0: &'a CompilerComponentBuild,
    pub execution: &'a CompilerCandidateExecution,
    pub candidate: &'a CompilerComponentBuild,
    pub handoff: &'a CompilerStageHandoff,
    pub payloads: &'a [VerifiedCompilerStagePayload],
    pub stage_folds: &'a [usize],
    pub bundle_fold: usize,
    pub adapter_file: &'a str,
    pub adapter: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerCandidateProductionRecord {
    pub ordinal: usize,
    pub stage: String,
    pub payload_bytes: usize,
    pub payload_sha256: String,
    pub fold: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerCandidateProduction {
    pub protocol: String,
    pub producer_contract: String,
    pub authority: String,
    pub stage0_component_sha256: String,
    pub stage0_execution_sha256: String,
    pub candidate_component_sha256: String,
    pub candidate_producer_id: String,
    pub candidate_compiler_image_sha256: String,
    pub stage_handoff_bundle_sha256: String,
    pub adapter_file: String,
    pub adapter_bytes: usize,
    pub adapter_sha256: String,
    pub record_count: usize,
    pub bundle_fold: usize,
    pub replacement_authorized: bool,
    pub proof_sha256: String,
    pub records: Vec<CompilerCandidateProductionRecord>,
}

pub fn compiler_candidate_stage_fold(ordinal: usize, bytes: &[u8]) -> usize {
    let mut state = 97_u64 + ((ordinal as u64 + 1) * 17);
    for byte in bytes {
        state = ((state * 257) + u64::from(*byte) + ordinal as u64 + 1) % FOLD_MODULUS;
    }
    state as usize
}

pub fn compiler_candidate_bundle_fold(stage_folds: &[usize]) -> usize {
    let mut state = 193_u64;
    for (ordinal, fold) in stage_folds.iter().copied().enumerate() {
        state = ((state * 65_537) + fold as u64 + ordinal as u64 + 1) % FOLD_MODULUS;
    }
    state as usize
}

pub fn build_compiler_candidate_production(
    input: &CompilerCandidateProductionInput<'_>,
) -> Result<CompilerCandidateProduction, ArtifactError> {
    validate_evidence(input)?;
    let records = input
        .handoff
        .records
        .iter()
        .zip(input.payloads)
        .zip(input.stage_folds)
        .map(
            |((record, payload), fold)| CompilerCandidateProductionRecord {
                ordinal: record.ordinal,
                stage: record.stage.as_str().to_owned(),
                payload_bytes: payload.bytes.len(),
                payload_sha256: record.payload_sha256.clone(),
                fold: *fold,
            },
        )
        .collect::<Vec<_>>();
    let mut proof = CompilerCandidateProduction {
        protocol: COMPILER_CANDIDATE_PRODUCTION_PROTOCOL.to_owned(),
        producer_contract: COMPILER_CANDIDATE_PRODUCER_CONTRACT.to_owned(),
        authority: COMPILER_CANDIDATE_PRODUCTION_AUTHORITY.to_owned(),
        stage0_component_sha256: input.stage0.record_sha256.clone(),
        stage0_execution_sha256: input.execution.execution_sha256.clone(),
        candidate_component_sha256: input.candidate.record_sha256.clone(),
        candidate_producer_id: input.candidate.producer_id.clone(),
        candidate_compiler_image_sha256: input.candidate.compiler_image_sha256.clone(),
        stage_handoff_bundle_sha256: input.handoff.bundle_sha256.clone(),
        adapter_file: input.adapter_file.to_owned(),
        adapter_bytes: input.adapter.len(),
        adapter_sha256: sha256_hex(input.adapter),
        record_count: records.len(),
        bundle_fold: input.bundle_fold,
        replacement_authorized: false,
        proof_sha256: String::new(),
        records,
    };
    proof.proof_sha256 = production_identity(&proof);
    validate_proof(&proof)?;
    Ok(proof)
}

pub fn render_compiler_candidate_production(proof: &CompilerCandidateProduction) -> String {
    let mut out = format!(
        "protocol = \"{}\"\nproducer_contract = \"{}\"\nauthority = \"{}\"\nstage0_component_sha256 = \"{}\"\nstage0_execution_sha256 = \"{}\"\ncandidate_component_sha256 = \"{}\"\ncandidate_producer_id = \"{}\"\ncandidate_compiler_image_sha256 = \"{}\"\nstage_handoff_bundle_sha256 = \"{}\"\nadapter_file = \"{}\"\nadapter_bytes = {}\nadapter_sha256 = \"{}\"\nrecord_count = {}\nbundle_fold = {}\nreplacement_authorized = {}\nproof_sha256 = \"{}\"\n",
        proof.protocol,
        proof.producer_contract,
        proof.authority,
        proof.stage0_component_sha256,
        proof.stage0_execution_sha256,
        proof.candidate_component_sha256,
        escape_toml_string(&proof.candidate_producer_id),
        proof.candidate_compiler_image_sha256,
        proof.stage_handoff_bundle_sha256,
        escape_toml_string(&proof.adapter_file),
        proof.adapter_bytes,
        proof.adapter_sha256,
        proof.record_count,
        proof.bundle_fold,
        proof.replacement_authorized,
        proof.proof_sha256,
    );
    for record in &proof.records {
        out.push_str(&format!(
            "\n[[record]]\nordinal = {}\nstage = \"{}\"\npayload_bytes = {}\npayload_sha256 = \"{}\"\nfold = {}\n",
            record.ordinal,
            record.stage,
            record.payload_bytes,
            record.payload_sha256,
            record.fold,
        ));
    }
    out
}

pub fn parse_compiler_candidate_production(
    path: &Path,
) -> Result<CompilerCandidateProduction, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler candidate production `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_candidate_production_from_source(&source, path)
}

pub fn parse_compiler_candidate_production_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerCandidateProduction, ArtifactError> {
    validate_text(source, path)?;
    let proof = CompilerCandidateProduction {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        producer_contract: parse_required_toml_string(source, "producer_contract", path)?,
        authority: parse_required_toml_string(source, "authority", path)?,
        stage0_component_sha256: parse_required_toml_string(
            source,
            "stage0_component_sha256",
            path,
        )?,
        stage0_execution_sha256: parse_required_toml_string(
            source,
            "stage0_execution_sha256",
            path,
        )?,
        candidate_component_sha256: parse_required_toml_string(
            source,
            "candidate_component_sha256",
            path,
        )?,
        candidate_producer_id: parse_required_toml_string(source, "candidate_producer_id", path)?,
        candidate_compiler_image_sha256: parse_required_toml_string(
            source,
            "candidate_compiler_image_sha256",
            path,
        )?,
        stage_handoff_bundle_sha256: parse_required_toml_string(
            source,
            "stage_handoff_bundle_sha256",
            path,
        )?,
        adapter_file: parse_required_toml_string(source, "adapter_file", path)?,
        adapter_bytes: parse_required_toml_usize(source, "adapter_bytes", path)?,
        adapter_sha256: parse_required_toml_string(source, "adapter_sha256", path)?,
        record_count: parse_required_toml_usize(source, "record_count", path)?,
        bundle_fold: parse_required_toml_usize(source, "bundle_fold", path)?,
        replacement_authorized: parse_required_toml_bool(source, "replacement_authorized", path)?,
        proof_sha256: parse_required_toml_string(source, "proof_sha256", path)?,
        records: parse_record_blocks(source, path)?,
    };
    validate_proof(&proof)?;
    if render_compiler_candidate_production(&proof) != source {
        return Err(ArtifactError::new(format!(
            "compiler candidate production `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(proof)
}

pub fn read_compiler_candidate_production(
    path: &Path,
    stage0: &CompilerComponentBuild,
    execution: &CompilerCandidateExecution,
    candidate: &CompilerComponentBuild,
    handoff: &CompilerStageHandoff,
    payloads: &[VerifiedCompilerStagePayload],
) -> Result<CompilerCandidateProduction, ArtifactError> {
    let proof = parse_compiler_candidate_production(path)?;
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    let adapter = fs::read(root.join(&proof.adapter_file)).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler candidate adapter `{}`: {error}",
            proof.adapter_file
        ))
    })?;
    if adapter.len() != proof.adapter_bytes || sha256_hex(&adapter) != proof.adapter_sha256 {
        return Err(ArtifactError::new(
            "compiler candidate adapter length or SHA-256 mismatch",
        ));
    }
    let stage_folds = proof
        .records
        .iter()
        .map(|record| record.fold)
        .collect::<Vec<_>>();
    validate_evidence(&CompilerCandidateProductionInput {
        stage0,
        execution,
        candidate,
        handoff,
        payloads,
        stage_folds: &stage_folds,
        bundle_fold: proof.bundle_fold,
        adapter_file: &proof.adapter_file,
        adapter: &adapter,
    })?;
    let rebuilt = build_compiler_candidate_production(&CompilerCandidateProductionInput {
        stage0,
        execution,
        candidate,
        handoff,
        payloads,
        stage_folds: &stage_folds,
        bundle_fold: proof.bundle_fold,
        adapter_file: &proof.adapter_file,
        adapter: &adapter,
    })?;
    if rebuilt != proof {
        return Err(ArtifactError::new(
            "compiler candidate production does not match its bound evidence",
        ));
    }
    Ok(proof)
}

fn validate_evidence(input: &CompilerCandidateProductionInput<'_>) -> Result<(), ArtifactError> {
    if input.stage0.stage_role != COMPILER_COMPONENT_STAGE0_ROLE
        || input.candidate.stage_role != COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE
        || input.stage0.producer_id == input.candidate.producer_id
    {
        return Err(ArtifactError::new(
            "compiler candidate production requires distinct stage0 and stage1-candidate producers",
        ));
    }
    if input.execution.component_record_sha256 != input.stage0.record_sha256
        || input.execution.component_reproducible_build_sha256
            != input.stage0.reproducible_build_sha256
        || input.execution.candidate_binary_sha256 != input.stage0.native_binary_sha256
        || input.execution.exit_code != 0
    {
        return Err(ArtifactError::new(
            "compiler candidate production execution does not bind the stage0 candidate image",
        ));
    }
    if input.candidate.component_id != input.stage0.component_id
        || input.candidate.component_domain != input.stage0.component_domain
        || input.candidate.component_unit != input.stage0.component_unit
        || input.candidate.bootstrap_subset_protocol != input.stage0.bootstrap_subset_protocol
        || input.candidate.compiler_image_sha256 != input.stage0.native_binary_sha256
        || input.candidate.native_binary_sha256 != input.stage0.native_binary_sha256
        || input.candidate.dependency_closure_sha256 != input.stage0.dependency_closure_sha256
    {
        return Err(ArtifactError::new(
            "compiler candidate production changed a protected source component identity",
        ));
    }
    if input.handoff.producer_id != input.candidate.producer_id
        || input.handoff.module_domain != input.candidate.component_domain
        || input.handoff.module_unit != input.candidate.component_unit
        || input.handoff.bundle_sha256 != input.candidate.stage_handoff_bundle_sha256
        || input.handoff.bundle_sha256 != input.stage0.stage_handoff_bundle_sha256
    {
        return Err(ArtifactError::new(
            "compiler candidate production handoff does not match its candidate component",
        ));
    }
    if input.handoff.records.len() != EXPECTED_STAGE_COUNT
        || input.payloads.len() != EXPECTED_STAGE_COUNT
        || input.stage_folds.len() != EXPECTED_STAGE_COUNT
    {
        return Err(ArtifactError::new(
            "compiler candidate production requires exactly five stage records",
        ));
    }
    for (ordinal, ((record, payload), fold)) in input
        .handoff
        .records
        .iter()
        .zip(input.payloads)
        .zip(input.stage_folds)
        .enumerate()
    {
        if record.ordinal != ordinal
            || record.stage != payload.stage
            || record.payload_bytes != payload.bytes.len()
            || record.payload_sha256 != sha256_hex(&payload.bytes)
            || *fold != compiler_candidate_stage_fold(ordinal, &payload.bytes)
        {
            return Err(ArtifactError::new(format!(
                "compiler candidate production stage {ordinal} fold or payload identity mismatch"
            )));
        }
    }
    if input.bundle_fold != compiler_candidate_bundle_fold(input.stage_folds) {
        return Err(ArtifactError::new(
            "compiler candidate production bundle fold mismatch",
        ));
    }
    validate_file_name(input.adapter_file, "candidate adapter")?;
    if input.adapter.is_empty() {
        return Err(ArtifactError::new(
            "compiler candidate production adapter cannot be empty",
        ));
    }
    Ok(())
}

fn validate_proof(proof: &CompilerCandidateProduction) -> Result<(), ArtifactError> {
    if proof.protocol != COMPILER_CANDIDATE_PRODUCTION_PROTOCOL
        || proof.producer_contract != COMPILER_CANDIDATE_PRODUCER_CONTRACT
        || proof.authority != COMPILER_CANDIDATE_PRODUCTION_AUTHORITY
        || proof.replacement_authorized
    {
        return Err(ArtifactError::new(
            "compiler candidate production declares an unsupported authority contract",
        ));
    }
    if proof.record_count != EXPECTED_STAGE_COUNT || proof.records.len() != EXPECTED_STAGE_COUNT {
        return Err(ArtifactError::new(
            "compiler candidate production record count must be five",
        ));
    }
    for (label, value) in [
        ("stage0 component", proof.stage0_component_sha256.as_str()),
        ("stage0 execution", proof.stage0_execution_sha256.as_str()),
        (
            "candidate component",
            proof.candidate_component_sha256.as_str(),
        ),
        (
            "candidate compiler image",
            proof.candidate_compiler_image_sha256.as_str(),
        ),
        (
            "stage handoff bundle",
            proof.stage_handoff_bundle_sha256.as_str(),
        ),
        ("candidate adapter", proof.adapter_sha256.as_str()),
        ("production proof", proof.proof_sha256.as_str()),
    ] {
        validate_sha256(value, label)?;
    }
    validate_token(&proof.candidate_producer_id, "candidate producer id")?;
    validate_file_name(&proof.adapter_file, "candidate adapter")?;
    if proof.adapter_bytes == 0 || proof.bundle_fold >= FOLD_MODULUS as usize {
        return Err(ArtifactError::new(
            "compiler candidate production adapter length or bundle fold is invalid",
        ));
    }
    for (ordinal, record) in proof.records.iter().enumerate() {
        let expected_stage = ["source", "tokens", "ast", "nir", "yir"][ordinal];
        if record.ordinal != ordinal
            || record.stage != expected_stage
            || record.payload_bytes == 0
            || record.fold >= FOLD_MODULUS as usize
        {
            return Err(ArtifactError::new(format!(
                "compiler candidate production record {ordinal} is invalid"
            )));
        }
        validate_sha256(&record.payload_sha256, "candidate stage payload")?;
    }
    if proof.proof_sha256 != production_identity(proof) {
        return Err(ArtifactError::new(
            "compiler candidate production proof identity mismatch",
        ));
    }
    Ok(())
}

fn parse_record_blocks(
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

fn production_identity(proof: &CompilerCandidateProduction) -> String {
    let mut hash = Sha256::new();
    for value in [
        proof.protocol.as_bytes(),
        proof.producer_contract.as_bytes(),
        proof.authority.as_bytes(),
        proof.stage0_component_sha256.as_bytes(),
        proof.stage0_execution_sha256.as_bytes(),
        proof.candidate_component_sha256.as_bytes(),
        proof.candidate_producer_id.as_bytes(),
        proof.candidate_compiler_image_sha256.as_bytes(),
        proof.stage_handoff_bundle_sha256.as_bytes(),
        proof.adapter_file.as_bytes(),
        proof.adapter_sha256.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [
        proof.adapter_bytes,
        proof.record_count,
        proof.bundle_fold,
        usize::from(proof.replacement_authorized),
    ] {
        hash_field(&mut hash, &(value as u64).to_le_bytes());
    }
    for record in &proof.records {
        hash_field(&mut hash, &(record.ordinal as u64).to_le_bytes());
        hash_field(&mut hash, record.stage.as_bytes());
        hash_field(&mut hash, &(record.payload_bytes as u64).to_le_bytes());
        hash_field(&mut hash, record.payload_sha256.as_bytes());
        hash_field(&mut hash, &(record.fold as u64).to_le_bytes());
    }
    format!("{:x}", hash.finalize())
}

fn validate_text(source: &str, path: &Path) -> Result<(), ArtifactError> {
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

fn validate_token(value: &str, label: &str) -> Result<(), ArtifactError> {
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

fn validate_file_name(value: &str, label: &str) -> Result<(), ArtifactError> {
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

fn validate_sha256(value: &str, label: &str) -> Result<(), ArtifactError> {
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

fn hash_field(hash: &mut Sha256, value: &[u8]) {
    hash.update((value.len() as u64).to_le_bytes());
    hash.update(value);
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "compiler_candidate_production_tests.rs"]
mod tests;
