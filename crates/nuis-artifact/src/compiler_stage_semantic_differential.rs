use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use sha2::{Digest, Sha256};

use crate::{
    compiler_stage_transformation::{
        decode_compiler_stage_transformation_payload, encode_compiler_stage_transformation_payload,
        verify_compiler_stage_transformation_payloads,
    },
    compiler_stage_transformation_payload_file,
    toml::{
        escape_toml_string, parse_optional_map_usize, parse_required_map_string_in_block,
        parse_required_toml_bool, parse_required_toml_string, parse_required_toml_usize,
    },
    verify_compiler_stage_transformations, ArtifactError, CompilerStageHandoff, CompilerStageKind,
    CompilerStageTransformations, VerifiedCompilerStagePayload,
    COMPILER_STAGE_TRANSFORMATION_OUTPUT_ENCODING,
};

pub const COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_PROTOCOL: &str =
    "nuis-compiler-stage-semantic-differential-v1";
pub const COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_PRODUCER_CONTRACT: &str =
    "nuis-compiler-stage-semantic-differential-producer-v1";
pub const COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_AUTHORITY: &str =
    "lossless-derived-stage-equivalence-no-replacement";
pub const COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_FILE: &str =
    "nuis.compiler-stage-semantic-diff.toml";
pub const COMPILER_STAGE_SEMANTIC_EQUIVALENCE_CONTRACT: &str =
    "nuis-lossless-derived-stage-equivalence-v1";
pub const COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_VERDICT: &str =
    "semantic-equivalent-derived-representation-awaiting-authorization";

#[derive(Debug, Clone, Copy)]
pub struct CompilerStageSemanticDifferentialInput<'a> {
    pub producer_id: &'a str,
    pub handoff: &'a CompilerStageHandoff,
    pub payloads: &'a [VerifiedCompilerStagePayload],
    pub transformations: &'a CompilerStageTransformations,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerStageSemanticComparison {
    pub ordinal: usize,
    pub source_stage: CompilerStageKind,
    pub source_encoding: String,
    pub source_payload_bytes: usize,
    pub source_payload_sha256: String,
    pub derived_encoding: String,
    pub derived_payload_file: String,
    pub derived_payload_bytes: usize,
    pub derived_payload_sha256: String,
    pub checkpoint_sha256: String,
    pub recovered_source_payload_sha256: String,
    pub byte_identical: bool,
    pub semantically_equivalent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerStageSemanticDifferential {
    pub protocol: String,
    pub producer_contract: String,
    pub authority: String,
    pub equivalence_contract: String,
    pub producer_id: String,
    pub stage_handoff_bundle_sha256: String,
    pub stage_transformations_proof_sha256: String,
    pub comparison_count: usize,
    pub equivalent_count: usize,
    pub deterministic_semantic_equivalent: bool,
    pub replacement_authorized: bool,
    pub verdict: String,
    pub proof_sha256: String,
    pub comparisons: Vec<CompilerStageSemanticComparison>,
}

pub fn build_compiler_stage_semantic_differential(
    input: &CompilerStageSemanticDifferentialInput<'_>,
) -> Result<CompilerStageSemanticDifferential, ArtifactError> {
    validate_input(input)?;
    let mut comparisons = Vec::with_capacity(input.transformations.records.len());
    for transformation in &input.transformations.records {
        let source = source_payload(input.payloads, transformation.source_stage)?;
        let derived = encode_compiler_stage_transformation_payload(
            transformation.source_stage,
            &source.bytes,
            &transformation.output_words,
        )?;
        let (checkpoint_words, recovered_source) =
            decode_compiler_stage_transformation_payload(transformation.source_stage, &derived)?;
        if checkpoint_words != transformation.output_words || recovered_source != source.bytes {
            return Err(ArtifactError::new(format!(
                "compiler stage `{}` semantic differential failed lossless replay",
                transformation.source_stage.as_str()
            )));
        }
        let source_sha256 = sha256_hex(&source.bytes);
        let recovered_sha256 = sha256_hex(&recovered_source);
        comparisons.push(CompilerStageSemanticComparison {
            ordinal: transformation.ordinal,
            source_stage: transformation.source_stage,
            source_encoding: transformation.source_stage.encoding().to_owned(),
            source_payload_bytes: source.bytes.len(),
            source_payload_sha256: source_sha256.clone(),
            derived_encoding: transformation.output_encoding.clone(),
            derived_payload_file: transformation.output_payload_file.clone(),
            derived_payload_bytes: derived.len(),
            derived_payload_sha256: sha256_hex(&derived),
            checkpoint_sha256: transformation.output_checkpoint_sha256.clone(),
            recovered_source_payload_sha256: recovered_sha256.clone(),
            byte_identical: source.bytes == derived,
            semantically_equivalent: source_sha256 == recovered_sha256,
        });
    }
    let equivalent_count = comparisons
        .iter()
        .filter(|comparison| comparison.semantically_equivalent)
        .count();
    let deterministic_semantic_equivalent = equivalent_count == comparisons.len()
        && comparisons
            .iter()
            .all(|comparison| !comparison.byte_identical);
    let mut differential = CompilerStageSemanticDifferential {
        protocol: COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_PROTOCOL.to_owned(),
        producer_contract: COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_PRODUCER_CONTRACT.to_owned(),
        authority: COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_AUTHORITY.to_owned(),
        equivalence_contract: COMPILER_STAGE_SEMANTIC_EQUIVALENCE_CONTRACT.to_owned(),
        producer_id: input.producer_id.to_owned(),
        stage_handoff_bundle_sha256: input.handoff.bundle_sha256.clone(),
        stage_transformations_proof_sha256: input.transformations.proof_sha256.clone(),
        comparison_count: comparisons.len(),
        equivalent_count,
        deterministic_semantic_equivalent,
        replacement_authorized: false,
        verdict: COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_VERDICT.to_owned(),
        proof_sha256: String::new(),
        comparisons,
    };
    differential.proof_sha256 = differential_identity(&differential);
    validate_differential(&differential)?;
    Ok(differential)
}

pub fn render_compiler_stage_semantic_differential(
    differential: &CompilerStageSemanticDifferential,
) -> String {
    let mut out = format!(
        "protocol = \"{}\"\nproducer_contract = \"{}\"\nauthority = \"{}\"\nequivalence_contract = \"{}\"\nproducer_id = \"{}\"\nstage_handoff_bundle_sha256 = \"{}\"\nstage_transformations_proof_sha256 = \"{}\"\ncomparison_count = {}\nequivalent_count = {}\ndeterministic_semantic_equivalent = {}\nreplacement_authorized = {}\nverdict = \"{}\"\nproof_sha256 = \"{}\"\n",
        differential.protocol,
        differential.producer_contract,
        differential.authority,
        differential.equivalence_contract,
        escape_toml_string(&differential.producer_id),
        differential.stage_handoff_bundle_sha256,
        differential.stage_transformations_proof_sha256,
        differential.comparison_count,
        differential.equivalent_count,
        differential.deterministic_semantic_equivalent,
        differential.replacement_authorized,
        differential.verdict,
        differential.proof_sha256,
    );
    for comparison in &differential.comparisons {
        out.push_str(&format!(
            "\n[[comparison]]\nordinal = {}\nsource_stage = \"{}\"\nsource_encoding = \"{}\"\nsource_payload_bytes = {}\nsource_payload_sha256 = \"{}\"\nderived_encoding = \"{}\"\nderived_payload_file = \"{}\"\nderived_payload_bytes = {}\nderived_payload_sha256 = \"{}\"\ncheckpoint_sha256 = \"{}\"\nrecovered_source_payload_sha256 = \"{}\"\nbyte_identical = {}\nsemantically_equivalent = {}\n",
            comparison.ordinal,
            comparison.source_stage.as_str(),
            comparison.source_encoding,
            comparison.source_payload_bytes,
            comparison.source_payload_sha256,
            comparison.derived_encoding,
            escape_toml_string(&comparison.derived_payload_file),
            comparison.derived_payload_bytes,
            comparison.derived_payload_sha256,
            comparison.checkpoint_sha256,
            comparison.recovered_source_payload_sha256,
            comparison.byte_identical,
            comparison.semantically_equivalent,
        ));
    }
    out
}

pub fn parse_compiler_stage_semantic_differential(
    path: &Path,
) -> Result<CompilerStageSemanticDifferential, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler stage semantic differential `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_stage_semantic_differential_from_source(&source, path)
}

pub fn parse_compiler_stage_semantic_differential_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerStageSemanticDifferential, ArtifactError> {
    validate_text(source, path)?;
    let comparisons = parse_comparison_blocks(source, path)?
        .into_iter()
        .map(|values| parse_comparison(values, path))
        .collect::<Result<Vec<_>, _>>()?;
    let differential = CompilerStageSemanticDifferential {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        producer_contract: parse_required_toml_string(source, "producer_contract", path)?,
        authority: parse_required_toml_string(source, "authority", path)?,
        equivalence_contract: parse_required_toml_string(source, "equivalence_contract", path)?,
        producer_id: parse_required_toml_string(source, "producer_id", path)?,
        stage_handoff_bundle_sha256: parse_required_toml_string(
            source,
            "stage_handoff_bundle_sha256",
            path,
        )?,
        stage_transformations_proof_sha256: parse_required_toml_string(
            source,
            "stage_transformations_proof_sha256",
            path,
        )?,
        comparison_count: parse_required_toml_usize(source, "comparison_count", path)?,
        equivalent_count: parse_required_toml_usize(source, "equivalent_count", path)?,
        deterministic_semantic_equivalent: parse_required_toml_bool(
            source,
            "deterministic_semantic_equivalent",
            path,
        )?,
        replacement_authorized: parse_required_toml_bool(source, "replacement_authorized", path)?,
        verdict: parse_required_toml_string(source, "verdict", path)?,
        proof_sha256: parse_required_toml_string(source, "proof_sha256", path)?,
        comparisons,
    };
    validate_differential(&differential)?;
    if render_compiler_stage_semantic_differential(&differential) != source {
        return Err(ArtifactError::new(format!(
            "compiler stage semantic differential `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(differential)
}

pub fn read_compiler_stage_semantic_differential(
    path: &Path,
    input: &CompilerStageSemanticDifferentialInput<'_>,
) -> Result<CompilerStageSemanticDifferential, ArtifactError> {
    let differential = parse_compiler_stage_semantic_differential(path)?;
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    verify_compiler_stage_transformation_payloads(root, input.transformations, input.payloads)?;
    verify_compiler_stage_semantic_differential(&differential, input)?;
    Ok(differential)
}

pub fn verify_compiler_stage_semantic_differential(
    differential: &CompilerStageSemanticDifferential,
    input: &CompilerStageSemanticDifferentialInput<'_>,
) -> Result<(), ArtifactError> {
    let rebuilt = build_compiler_stage_semantic_differential(input)?;
    if rebuilt != *differential {
        return Err(ArtifactError::new(
            "compiler stage semantic differential does not match its bound evidence",
        ));
    }
    Ok(())
}

fn validate_input(input: &CompilerStageSemanticDifferentialInput<'_>) -> Result<(), ArtifactError> {
    if input.producer_id != input.handoff.producer_id
        || input.producer_id != input.transformations.producer_id
        || input.handoff.bundle_sha256 != input.transformations.stage_handoff_bundle_sha256
    {
        return Err(ArtifactError::new(
            "compiler stage semantic differential producer or handoff binding mismatch",
        ));
    }
    verify_compiler_stage_transformations(input.transformations, input.handoff, input.payloads)
}

fn validate_differential(
    differential: &CompilerStageSemanticDifferential,
) -> Result<(), ArtifactError> {
    if differential.protocol != COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_PROTOCOL
        || differential.producer_contract != COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_PRODUCER_CONTRACT
        || differential.authority != COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_AUTHORITY
        || differential.equivalence_contract != COMPILER_STAGE_SEMANTIC_EQUIVALENCE_CONTRACT
        || differential.verdict != COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_VERDICT
        || differential.replacement_authorized
        || !differential.deterministic_semantic_equivalent
    {
        return Err(ArtifactError::new(
            "compiler stage semantic differential declares an unsupported authority or verdict",
        ));
    }
    validate_token(&differential.producer_id, "semantic differential producer")?;
    validate_sha256(
        &differential.stage_handoff_bundle_sha256,
        "stage handoff bundle",
    )?;
    validate_sha256(
        &differential.stage_transformations_proof_sha256,
        "stage transformation proof",
    )?;
    validate_sha256(&differential.proof_sha256, "semantic differential proof")?;
    if differential.comparison_count == 0
        || differential.comparison_count != differential.comparisons.len()
        || differential.equivalent_count != differential.comparison_count
    {
        return Err(ArtifactError::new(
            "compiler stage semantic differential comparison counts are invalid",
        ));
    }
    let mut stages = BTreeSet::new();
    for (ordinal, comparison) in differential.comparisons.iter().enumerate() {
        if comparison.ordinal != ordinal
            || !stages.insert(comparison.source_stage.as_str())
            || !matches!(
                comparison.source_stage,
                CompilerStageKind::Ast | CompilerStageKind::Nir
            )
            || comparison.source_encoding != comparison.source_stage.encoding()
            || comparison.derived_encoding != COMPILER_STAGE_TRANSFORMATION_OUTPUT_ENCODING
            || comparison.derived_payload_file
                != compiler_stage_transformation_payload_file(ordinal)
            || comparison.source_payload_bytes == 0
            || comparison.derived_payload_bytes <= comparison.source_payload_bytes
            || comparison.source_payload_sha256 == comparison.derived_payload_sha256
            || comparison.source_payload_sha256 != comparison.recovered_source_payload_sha256
            || comparison.byte_identical
            || !comparison.semantically_equivalent
        {
            return Err(ArtifactError::new(format!(
                "compiler stage semantic comparison {ordinal} is invalid"
            )));
        }
        for (label, value) in [
            ("source payload", comparison.source_payload_sha256.as_str()),
            (
                "derived payload",
                comparison.derived_payload_sha256.as_str(),
            ),
            ("checkpoint", comparison.checkpoint_sha256.as_str()),
            (
                "recovered source payload",
                comparison.recovered_source_payload_sha256.as_str(),
            ),
        ] {
            validate_sha256(value, label)?;
        }
    }
    if differential.proof_sha256 != differential_identity(differential) {
        return Err(ArtifactError::new(
            "compiler stage semantic differential proof identity mismatch",
        ));
    }
    Ok(())
}

fn parse_comparison(
    values: BTreeMap<String, String>,
    path: &Path,
) -> Result<CompilerStageSemanticComparison, ArtifactError> {
    let ordinal = required_map_usize(&values, "ordinal", path)?;
    let stage_name = required_map_string(&values, "source_stage", path)?;
    let source_stage = CompilerStageKind::parse(&stage_name).ok_or_else(|| {
        ArtifactError::new(format!(
            "`{}` semantic comparison {ordinal} has unsupported stage `{stage_name}`",
            path.display()
        ))
    })?;
    if values.len() != 13 {
        return Err(ArtifactError::new(format!(
            "`{}` semantic comparison {ordinal} contains unknown or missing keys",
            path.display()
        )));
    }
    Ok(CompilerStageSemanticComparison {
        ordinal,
        source_stage,
        source_encoding: required_map_string(&values, "source_encoding", path)?,
        source_payload_bytes: required_map_usize(&values, "source_payload_bytes", path)?,
        source_payload_sha256: required_map_string(&values, "source_payload_sha256", path)?,
        derived_encoding: required_map_string(&values, "derived_encoding", path)?,
        derived_payload_file: required_map_string(&values, "derived_payload_file", path)?,
        derived_payload_bytes: required_map_usize(&values, "derived_payload_bytes", path)?,
        derived_payload_sha256: required_map_string(&values, "derived_payload_sha256", path)?,
        checkpoint_sha256: required_map_string(&values, "checkpoint_sha256", path)?,
        recovered_source_payload_sha256: required_map_string(
            &values,
            "recovered_source_payload_sha256",
            path,
        )?,
        byte_identical: required_map_bool(&values, "byte_identical", path)?,
        semantically_equivalent: required_map_bool(&values, "semantically_equivalent", path)?,
    })
}

fn parse_comparison_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<BTreeMap<String, String>>, ArtifactError> {
    let mut blocks = Vec::new();
    let mut current: Option<BTreeMap<String, String>> = None;
    for raw in source.lines() {
        let line = raw.trim();
        if line == "[[comparison]]" {
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
                "`{}` contains malformed semantic comparison line `{line}`",
                path.display()
            ))
        })?;
        let values = current.as_mut().expect("comparison block is active");
        if values
            .insert(key.trim().to_owned(), value.trim().to_owned())
            .is_some()
        {
            return Err(ArtifactError::new(format!(
                "`{}` contains duplicate semantic comparison key `{}`",
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

fn required_map_string(
    values: &BTreeMap<String, String>,
    key: &str,
    path: &Path,
) -> Result<String, ArtifactError> {
    parse_required_map_string_in_block(values, key, path, "semantic comparison")
}

fn required_map_usize(
    values: &BTreeMap<String, String>,
    key: &str,
    path: &Path,
) -> Result<usize, ArtifactError> {
    parse_optional_map_usize(values, key, path, "semantic comparison")?.ok_or_else(|| {
        ArtifactError::new(format!(
            "`{}` semantic comparison is missing required key `{key}`",
            path.display()
        ))
    })
}

fn required_map_bool(
    values: &BTreeMap<String, String>,
    key: &str,
    path: &Path,
) -> Result<bool, ArtifactError> {
    match values.get(key).map(String::as_str) {
        Some("true") => Ok(true),
        Some("false") => Ok(false),
        _ => Err(ArtifactError::new(format!(
            "`{}` semantic comparison key `{key}` must be a boolean",
            path.display()
        ))),
    }
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
                "compiler stage `{}` semantic source payload is missing",
                stage.as_str()
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
            "compiler stage semantic differential `{}` must use canonical UTF-8/LF text",
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

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn differential_identity(differential: &CompilerStageSemanticDifferential) -> String {
    let mut hash = Sha256::new();
    for value in [
        differential.protocol.as_bytes(),
        differential.producer_contract.as_bytes(),
        differential.authority.as_bytes(),
        differential.equivalence_contract.as_bytes(),
        differential.producer_id.as_bytes(),
        differential.stage_handoff_bundle_sha256.as_bytes(),
        differential.stage_transformations_proof_sha256.as_bytes(),
        differential.verdict.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [
        differential.comparison_count,
        differential.equivalent_count,
        usize::from(differential.deterministic_semantic_equivalent),
        usize::from(differential.replacement_authorized),
    ] {
        hash_field(&mut hash, &(value as u64).to_le_bytes());
    }
    for comparison in &differential.comparisons {
        for value in [
            comparison.ordinal,
            comparison.source_payload_bytes,
            comparison.derived_payload_bytes,
            usize::from(comparison.byte_identical),
            usize::from(comparison.semantically_equivalent),
        ] {
            hash_field(&mut hash, &(value as u64).to_le_bytes());
        }
        for value in [
            comparison.source_stage.as_str().as_bytes(),
            comparison.source_encoding.as_bytes(),
            comparison.source_payload_sha256.as_bytes(),
            comparison.derived_encoding.as_bytes(),
            comparison.derived_payload_file.as_bytes(),
            comparison.derived_payload_sha256.as_bytes(),
            comparison.checkpoint_sha256.as_bytes(),
            comparison.recovered_source_payload_sha256.as_bytes(),
        ] {
            hash_field(&mut hash, value);
        }
    }
    format!("{:x}", hash.finalize())
}

fn hash_field(hash: &mut Sha256, value: &[u8]) {
    hash.update((value.len() as u64).to_le_bytes());
    hash.update(value);
}

#[cfg(test)]
#[path = "compiler_stage_semantic_differential_tests.rs"]
mod tests;
