use std::{collections::BTreeMap, fs, path::Path};

use sha2::{Digest, Sha256};

use crate::{
    read_compiler_stage_semantic_differential, read_compiler_stage_transformations,
    render_compiler_stage_semantic_differential, render_compiler_stage_transformations,
    toml::{
        escape_toml_string, parse_optional_map_usize, parse_required_map_string_in_block,
        parse_required_toml_bool, parse_required_toml_string, parse_required_toml_usize,
    },
    verify_compiler_stage_semantic_differential, verify_compiler_stage_transformations,
    ArtifactError, CompilerStageHandoff, CompilerStageKind, CompilerStageSemanticDifferential,
    CompilerStageSemanticDifferentialInput, CompilerStageTransformations,
    VerifiedCompilerStagePayload, COMPILER_STAGE_HANDOFF_PROTOCOL,
    COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_FILE, COMPILER_STAGE_TRANSFORMATION_FILE,
};

pub const COMPILER_STAGE_HANDOFF_V2_PROTOCOL: &str = "nuis-compiler-stage-handoff-v2";
pub const COMPILER_STAGE_HANDOFF_V2_SELECTION_CONTRACT: &str =
    "nuis-compiler-derived-stage-selection-v1";
pub const COMPILER_STAGE_HANDOFF_V2_AUTHORITY: &str =
    "reversible-derived-stage-selection-no-replacement";
pub const COMPILER_STAGE_HANDOFF_V2_VERDICT: &str =
    "registered-derived-stages-selected-awaiting-authorization";
pub const COMPILER_STAGE_HANDOFF_V2_FILE: &str = "nuis.compiler-stage-handoff-v2.toml";

#[derive(Debug, Clone, Copy)]
pub struct CompilerStageHandoffV2Input<'a> {
    pub handoff: &'a CompilerStageHandoff,
    pub payloads: &'a [VerifiedCompilerStagePayload],
    pub transformations: &'a CompilerStageTransformations,
    pub semantic_differential: &'a CompilerStageSemanticDifferential,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerStageSelectionRecord {
    pub ordinal: usize,
    pub transformation_ordinal: usize,
    pub comparison_ordinal: usize,
    pub source_stage: CompilerStageKind,
    pub source_encoding: String,
    pub source_record_sha256: String,
    pub source_payload_sha256: String,
    pub transform_contract: String,
    pub derived_encoding: String,
    pub derived_payload_file: String,
    pub derived_payload_bytes: usize,
    pub derived_payload_sha256: String,
    pub checkpoint_sha256: String,
    pub recovered_source_payload_sha256: String,
    pub reversible: bool,
    pub semantically_equivalent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerStageHandoffV2 {
    pub protocol: String,
    pub selection_contract: String,
    pub authority: String,
    pub producer_id: String,
    pub base_handoff_protocol: String,
    pub base_handoff_bundle_sha256: String,
    pub stage_transformations_file: String,
    pub stage_transformations_bytes: usize,
    pub stage_transformations_sha256: String,
    pub stage_transformations_proof_sha256: String,
    pub stage_semantic_differential_file: String,
    pub stage_semantic_differential_bytes: usize,
    pub stage_semantic_differential_sha256: String,
    pub stage_semantic_differential_proof_sha256: String,
    pub selection_count: usize,
    pub all_reversible: bool,
    pub replacement_authorized: bool,
    pub verdict: String,
    pub proof_sha256: String,
    pub selections: Vec<CompilerStageSelectionRecord>,
}

impl CompilerStageHandoffV2 {
    pub fn selection_for_stage(
        &self,
        stage: CompilerStageKind,
    ) -> Option<&CompilerStageSelectionRecord> {
        self.selections
            .iter()
            .find(|selection| selection.source_stage == stage)
    }
}

pub fn build_compiler_stage_handoff_v2(
    input: &CompilerStageHandoffV2Input<'_>,
) -> Result<CompilerStageHandoffV2, ArtifactError> {
    validate_input(input)?;
    let mut selections = Vec::with_capacity(input.transformations.records.len());
    for transformation in &input.transformations.records {
        let comparison = input
            .semantic_differential
            .comparisons
            .get(transformation.ordinal)
            .ok_or_else(|| {
                ArtifactError::new(format!(
                    "compiler stage selection is missing semantic comparison {}",
                    transformation.ordinal
                ))
            })?;
        let source_record = input
            .handoff
            .records
            .iter()
            .find(|record| record.stage == transformation.source_stage)
            .ok_or_else(|| {
                ArtifactError::new(format!(
                    "compiler stage selection source `{}` is missing from its base handoff",
                    transformation.source_stage.as_str()
                ))
            })?;
        validate_registered_selection(transformation, comparison, source_record)?;
        selections.push(CompilerStageSelectionRecord {
            ordinal: selections.len(),
            transformation_ordinal: transformation.ordinal,
            comparison_ordinal: comparison.ordinal,
            source_stage: transformation.source_stage,
            source_encoding: comparison.source_encoding.clone(),
            source_record_sha256: source_record.record_sha256.clone(),
            source_payload_sha256: comparison.source_payload_sha256.clone(),
            transform_contract: transformation.transform_contract.clone(),
            derived_encoding: comparison.derived_encoding.clone(),
            derived_payload_file: comparison.derived_payload_file.clone(),
            derived_payload_bytes: comparison.derived_payload_bytes,
            derived_payload_sha256: comparison.derived_payload_sha256.clone(),
            checkpoint_sha256: comparison.checkpoint_sha256.clone(),
            recovered_source_payload_sha256: comparison.recovered_source_payload_sha256.clone(),
            reversible: comparison.source_payload_sha256
                == comparison.recovered_source_payload_sha256,
            semantically_equivalent: comparison.semantically_equivalent,
        });
    }
    let transformations_source = render_compiler_stage_transformations(input.transformations);
    let differential_source =
        render_compiler_stage_semantic_differential(input.semantic_differential);
    let mut handoff = CompilerStageHandoffV2 {
        protocol: COMPILER_STAGE_HANDOFF_V2_PROTOCOL.to_owned(),
        selection_contract: COMPILER_STAGE_HANDOFF_V2_SELECTION_CONTRACT.to_owned(),
        authority: COMPILER_STAGE_HANDOFF_V2_AUTHORITY.to_owned(),
        producer_id: input.handoff.producer_id.clone(),
        base_handoff_protocol: input.handoff.protocol.clone(),
        base_handoff_bundle_sha256: input.handoff.bundle_sha256.clone(),
        stage_transformations_file: COMPILER_STAGE_TRANSFORMATION_FILE.to_owned(),
        stage_transformations_bytes: transformations_source.len(),
        stage_transformations_sha256: sha256_hex(transformations_source.as_bytes()),
        stage_transformations_proof_sha256: input.transformations.proof_sha256.clone(),
        stage_semantic_differential_file: COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_FILE.to_owned(),
        stage_semantic_differential_bytes: differential_source.len(),
        stage_semantic_differential_sha256: sha256_hex(differential_source.as_bytes()),
        stage_semantic_differential_proof_sha256: input.semantic_differential.proof_sha256.clone(),
        selection_count: selections.len(),
        all_reversible: selections.iter().all(|selection| selection.reversible),
        replacement_authorized: false,
        verdict: COMPILER_STAGE_HANDOFF_V2_VERDICT.to_owned(),
        proof_sha256: String::new(),
        selections,
    };
    handoff.proof_sha256 = handoff_identity(&handoff);
    validate_manifest(&handoff)?;
    Ok(handoff)
}

pub fn render_compiler_stage_handoff_v2(handoff: &CompilerStageHandoffV2) -> String {
    let mut out = format!(
        "protocol = \"{}\"\nselection_contract = \"{}\"\nauthority = \"{}\"\nproducer_id = \"{}\"\nbase_handoff_protocol = \"{}\"\nbase_handoff_bundle_sha256 = \"{}\"\nstage_transformations_file = \"{}\"\nstage_transformations_bytes = {}\nstage_transformations_sha256 = \"{}\"\nstage_transformations_proof_sha256 = \"{}\"\nstage_semantic_differential_file = \"{}\"\nstage_semantic_differential_bytes = {}\nstage_semantic_differential_sha256 = \"{}\"\nstage_semantic_differential_proof_sha256 = \"{}\"\nselection_count = {}\nall_reversible = {}\nreplacement_authorized = {}\nverdict = \"{}\"\nproof_sha256 = \"{}\"\n",
        handoff.protocol,
        handoff.selection_contract,
        handoff.authority,
        escape_toml_string(&handoff.producer_id),
        handoff.base_handoff_protocol,
        handoff.base_handoff_bundle_sha256,
        handoff.stage_transformations_file,
        handoff.stage_transformations_bytes,
        handoff.stage_transformations_sha256,
        handoff.stage_transformations_proof_sha256,
        handoff.stage_semantic_differential_file,
        handoff.stage_semantic_differential_bytes,
        handoff.stage_semantic_differential_sha256,
        handoff.stage_semantic_differential_proof_sha256,
        handoff.selection_count,
        handoff.all_reversible,
        handoff.replacement_authorized,
        handoff.verdict,
        handoff.proof_sha256,
    );
    for selection in &handoff.selections {
        out.push_str(&format!(
            "\n[[selection]]\nordinal = {}\ntransformation_ordinal = {}\ncomparison_ordinal = {}\nsource_stage = \"{}\"\nsource_encoding = \"{}\"\nsource_record_sha256 = \"{}\"\nsource_payload_sha256 = \"{}\"\ntransform_contract = \"{}\"\nderived_encoding = \"{}\"\nderived_payload_file = \"{}\"\nderived_payload_bytes = {}\nderived_payload_sha256 = \"{}\"\ncheckpoint_sha256 = \"{}\"\nrecovered_source_payload_sha256 = \"{}\"\nreversible = {}\nsemantically_equivalent = {}\n",
            selection.ordinal,
            selection.transformation_ordinal,
            selection.comparison_ordinal,
            selection.source_stage.as_str(),
            selection.source_encoding,
            selection.source_record_sha256,
            selection.source_payload_sha256,
            selection.transform_contract,
            selection.derived_encoding,
            escape_toml_string(&selection.derived_payload_file),
            selection.derived_payload_bytes,
            selection.derived_payload_sha256,
            selection.checkpoint_sha256,
            selection.recovered_source_payload_sha256,
            selection.reversible,
            selection.semantically_equivalent,
        ));
    }
    out
}

pub fn parse_compiler_stage_handoff_v2(
    path: &Path,
) -> Result<CompilerStageHandoffV2, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler stage handoff v2 `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_stage_handoff_v2_from_source(&source, path)
}

pub fn parse_compiler_stage_handoff_v2_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerStageHandoffV2, ArtifactError> {
    validate_text(source, path)?;
    let handoff = CompilerStageHandoffV2 {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        selection_contract: parse_required_toml_string(source, "selection_contract", path)?,
        authority: parse_required_toml_string(source, "authority", path)?,
        producer_id: parse_required_toml_string(source, "producer_id", path)?,
        base_handoff_protocol: parse_required_toml_string(source, "base_handoff_protocol", path)?,
        base_handoff_bundle_sha256: parse_required_toml_string(
            source,
            "base_handoff_bundle_sha256",
            path,
        )?,
        stage_transformations_file: parse_required_toml_string(
            source,
            "stage_transformations_file",
            path,
        )?,
        stage_transformations_bytes: parse_required_toml_usize(
            source,
            "stage_transformations_bytes",
            path,
        )?,
        stage_transformations_sha256: parse_required_toml_string(
            source,
            "stage_transformations_sha256",
            path,
        )?,
        stage_transformations_proof_sha256: parse_required_toml_string(
            source,
            "stage_transformations_proof_sha256",
            path,
        )?,
        stage_semantic_differential_file: parse_required_toml_string(
            source,
            "stage_semantic_differential_file",
            path,
        )?,
        stage_semantic_differential_bytes: parse_required_toml_usize(
            source,
            "stage_semantic_differential_bytes",
            path,
        )?,
        stage_semantic_differential_sha256: parse_required_toml_string(
            source,
            "stage_semantic_differential_sha256",
            path,
        )?,
        stage_semantic_differential_proof_sha256: parse_required_toml_string(
            source,
            "stage_semantic_differential_proof_sha256",
            path,
        )?,
        selection_count: parse_required_toml_usize(source, "selection_count", path)?,
        all_reversible: parse_required_toml_bool(source, "all_reversible", path)?,
        replacement_authorized: parse_required_toml_bool(source, "replacement_authorized", path)?,
        verdict: parse_required_toml_string(source, "verdict", path)?,
        proof_sha256: parse_required_toml_string(source, "proof_sha256", path)?,
        selections: parse_selection_blocks(source, path)?,
    };
    validate_manifest(&handoff)?;
    if render_compiler_stage_handoff_v2(&handoff) != source {
        return Err(ArtifactError::new(format!(
            "compiler stage handoff v2 `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(handoff)
}

pub fn read_compiler_stage_handoff_v2(
    path: &Path,
    base_handoff: &CompilerStageHandoff,
    payloads: &[VerifiedCompilerStagePayload],
) -> Result<CompilerStageHandoffV2, ArtifactError> {
    let handoff = parse_compiler_stage_handoff_v2(path)?;
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    let transformations_path = root.join(&handoff.stage_transformations_file);
    verify_sibling_identity(
        &transformations_path,
        handoff.stage_transformations_bytes,
        &handoff.stage_transformations_sha256,
        "stage transformations",
    )?;
    let transformations =
        read_compiler_stage_transformations(&transformations_path, base_handoff, payloads)?;
    let differential_path = root.join(&handoff.stage_semantic_differential_file);
    verify_sibling_identity(
        &differential_path,
        handoff.stage_semantic_differential_bytes,
        &handoff.stage_semantic_differential_sha256,
        "stage semantic differential",
    )?;
    let semantic_input = CompilerStageSemanticDifferentialInput {
        producer_id: &base_handoff.producer_id,
        handoff: base_handoff,
        payloads,
        transformations: &transformations,
    };
    let semantic_differential =
        read_compiler_stage_semantic_differential(&differential_path, &semantic_input)?;
    verify_compiler_stage_handoff_v2(
        &handoff,
        &CompilerStageHandoffV2Input {
            handoff: base_handoff,
            payloads,
            transformations: &transformations,
            semantic_differential: &semantic_differential,
        },
    )?;
    Ok(handoff)
}

pub fn verify_compiler_stage_handoff_v2(
    handoff: &CompilerStageHandoffV2,
    input: &CompilerStageHandoffV2Input<'_>,
) -> Result<(), ArtifactError> {
    let rebuilt = build_compiler_stage_handoff_v2(input)?;
    if rebuilt != *handoff {
        return Err(ArtifactError::new(
            "compiler stage handoff v2 does not match its registered evidence",
        ));
    }
    Ok(())
}

fn validate_input(input: &CompilerStageHandoffV2Input<'_>) -> Result<(), ArtifactError> {
    verify_compiler_stage_transformations(input.transformations, input.handoff, input.payloads)?;
    verify_compiler_stage_semantic_differential(
        input.semantic_differential,
        &CompilerStageSemanticDifferentialInput {
            producer_id: &input.handoff.producer_id,
            handoff: input.handoff,
            payloads: input.payloads,
            transformations: input.transformations,
        },
    )?;
    if input.handoff.protocol != COMPILER_STAGE_HANDOFF_PROTOCOL
        || input.transformations.producer_id != input.handoff.producer_id
        || input.semantic_differential.producer_id != input.handoff.producer_id
        || input.transformations.records.len() != input.semantic_differential.comparisons.len()
    {
        return Err(ArtifactError::new(
            "compiler stage handoff v2 evidence lineage is inconsistent",
        ));
    }
    Ok(())
}

fn validate_registered_selection(
    transformation: &crate::CompilerStageTransformationRecord,
    comparison: &crate::CompilerStageSemanticComparison,
    source_record: &crate::CompilerStageHandoffRecord,
) -> Result<(), ArtifactError> {
    if comparison.ordinal != transformation.ordinal
        || comparison.source_stage != transformation.source_stage
        || source_record.payload_sha256 != transformation.input_payload_sha256
        || comparison.source_payload_sha256 != transformation.input_payload_sha256
        || comparison.derived_encoding != transformation.output_encoding
        || comparison.derived_payload_file != transformation.output_payload_file
        || comparison.derived_payload_bytes != transformation.output_payload_bytes
        || comparison.derived_payload_sha256 != transformation.output_payload_sha256
        || comparison.checkpoint_sha256 != transformation.output_checkpoint_sha256
        || comparison.recovered_source_payload_sha256 != comparison.source_payload_sha256
        || comparison.byte_identical
        || !comparison.semantically_equivalent
    {
        return Err(ArtifactError::new(format!(
            "compiler stage `{}` registered selection evidence is inconsistent",
            transformation.source_stage.as_str()
        )));
    }
    Ok(())
}

fn validate_manifest(handoff: &CompilerStageHandoffV2) -> Result<(), ArtifactError> {
    if handoff.protocol != COMPILER_STAGE_HANDOFF_V2_PROTOCOL
        || handoff.selection_contract != COMPILER_STAGE_HANDOFF_V2_SELECTION_CONTRACT
        || handoff.authority != COMPILER_STAGE_HANDOFF_V2_AUTHORITY
        || handoff.base_handoff_protocol != COMPILER_STAGE_HANDOFF_PROTOCOL
        || handoff.verdict != COMPILER_STAGE_HANDOFF_V2_VERDICT
        || !handoff.all_reversible
        || handoff.replacement_authorized
    {
        return Err(ArtifactError::new(
            "compiler stage handoff v2 declares an unsupported authority or verdict",
        ));
    }
    validate_token(&handoff.producer_id, "producer")?;
    if handoff.stage_transformations_file != COMPILER_STAGE_TRANSFORMATION_FILE
        || handoff.stage_semantic_differential_file != COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_FILE
        || handoff.stage_transformations_bytes == 0
        || handoff.stage_semantic_differential_bytes == 0
    {
        return Err(ArtifactError::new(
            "compiler stage handoff v2 evidence file contract is invalid",
        ));
    }
    for (label, value) in [
        (
            "base handoff bundle",
            handoff.base_handoff_bundle_sha256.as_str(),
        ),
        (
            "stage transformations",
            handoff.stage_transformations_sha256.as_str(),
        ),
        (
            "stage transformation proof",
            handoff.stage_transformations_proof_sha256.as_str(),
        ),
        (
            "semantic differential",
            handoff.stage_semantic_differential_sha256.as_str(),
        ),
        (
            "semantic differential proof",
            handoff.stage_semantic_differential_proof_sha256.as_str(),
        ),
        ("handoff v2 proof", handoff.proof_sha256.as_str()),
    ] {
        validate_sha256(value, label)?;
    }
    if handoff.selection_count == 0 || handoff.selection_count != handoff.selections.len() {
        return Err(ArtifactError::new(
            "compiler stage handoff v2 selection count is invalid",
        ));
    }
    let mut seen_stages = std::collections::BTreeSet::new();
    for (ordinal, selection) in handoff.selections.iter().enumerate() {
        if selection.ordinal != ordinal
            || selection.transformation_ordinal != ordinal
            || selection.comparison_ordinal != ordinal
            || !seen_stages.insert(selection.source_stage.as_str())
            || selection.source_encoding != selection.source_stage.encoding()
            || selection.derived_payload_bytes == 0
            || !selection.reversible
            || !selection.semantically_equivalent
            || selection.source_payload_sha256 != selection.recovered_source_payload_sha256
        {
            return Err(ArtifactError::new(format!(
                "compiler stage handoff v2 selection {ordinal} is invalid"
            )));
        }
        validate_token(&selection.transform_contract, "transform contract")?;
        validate_token(&selection.derived_encoding, "derived encoding")?;
        validate_file_name(&selection.derived_payload_file, "derived payload")?;
        for (label, value) in [
            ("source record", selection.source_record_sha256.as_str()),
            ("source payload", selection.source_payload_sha256.as_str()),
            ("derived payload", selection.derived_payload_sha256.as_str()),
            ("checkpoint", selection.checkpoint_sha256.as_str()),
            (
                "recovered source payload",
                selection.recovered_source_payload_sha256.as_str(),
            ),
        ] {
            validate_sha256(value, label)?;
        }
    }
    if handoff.proof_sha256 != handoff_identity(handoff) {
        return Err(ArtifactError::new(
            "compiler stage handoff v2 proof identity mismatch",
        ));
    }
    Ok(())
}

fn parse_selection_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<CompilerStageSelectionRecord>, ArtifactError> {
    let mut blocks = Vec::new();
    let mut current: Option<BTreeMap<String, String>> = None;
    for raw in source.lines() {
        let line = raw.trim();
        if line == "[[selection]]" {
            if let Some(values) = current.take() {
                blocks.push(parse_selection(values, path)?);
            }
            current = Some(BTreeMap::new());
            continue;
        }
        if line.is_empty() || line.starts_with('#') || current.is_none() {
            continue;
        }
        let (key, value) = line.split_once('=').ok_or_else(|| {
            ArtifactError::new(format!(
                "`{}` contains malformed stage selection line `{line}`",
                path.display()
            ))
        })?;
        let values = current.as_mut().expect("selection block is active");
        if values
            .insert(key.trim().to_owned(), value.trim().to_owned())
            .is_some()
        {
            return Err(ArtifactError::new(format!(
                "`{}` contains duplicate stage selection key `{}`",
                path.display(),
                key.trim()
            )));
        }
    }
    if let Some(values) = current {
        blocks.push(parse_selection(values, path)?);
    }
    Ok(blocks)
}

fn parse_selection(
    values: BTreeMap<String, String>,
    path: &Path,
) -> Result<CompilerStageSelectionRecord, ArtifactError> {
    let ordinal = required_map_usize(&values, "ordinal", path)?;
    let stage_name = required_map_string(&values, "source_stage", path)?;
    let source_stage = CompilerStageKind::parse(&stage_name).ok_or_else(|| {
        ArtifactError::new(format!(
            "`{}` selection {ordinal} has unsupported stage `{stage_name}`",
            path.display()
        ))
    })?;
    if values.len() != 16 {
        return Err(ArtifactError::new(format!(
            "`{}` selection {ordinal} contains unknown or missing keys",
            path.display()
        )));
    }
    Ok(CompilerStageSelectionRecord {
        ordinal,
        transformation_ordinal: required_map_usize(&values, "transformation_ordinal", path)?,
        comparison_ordinal: required_map_usize(&values, "comparison_ordinal", path)?,
        source_stage,
        source_encoding: required_map_string(&values, "source_encoding", path)?,
        source_record_sha256: required_map_string(&values, "source_record_sha256", path)?,
        source_payload_sha256: required_map_string(&values, "source_payload_sha256", path)?,
        transform_contract: required_map_string(&values, "transform_contract", path)?,
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
        reversible: required_map_bool(&values, "reversible", path)?,
        semantically_equivalent: required_map_bool(&values, "semantically_equivalent", path)?,
    })
}

fn required_map_string(
    values: &BTreeMap<String, String>,
    key: &str,
    path: &Path,
) -> Result<String, ArtifactError> {
    parse_required_map_string_in_block(values, key, path, "selection")
}

fn required_map_usize(
    values: &BTreeMap<String, String>,
    key: &str,
    path: &Path,
) -> Result<usize, ArtifactError> {
    parse_optional_map_usize(values, key, path, "selection")?.ok_or_else(|| {
        ArtifactError::new(format!(
            "`{}` selection is missing required key `{key}`",
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
            "`{}` selection key `{key}` must be a boolean",
            path.display()
        ))),
    }
}

fn verify_sibling_identity(
    path: &Path,
    expected_bytes: usize,
    expected_sha256: &str,
    label: &str,
) -> Result<(), ArtifactError> {
    let bytes = fs::read(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler {label} `{}`: {error}",
            path.display()
        ))
    })?;
    if bytes.len() != expected_bytes || sha256_hex(&bytes) != expected_sha256 {
        return Err(ArtifactError::new(format!(
            "compiler {label} length or SHA-256 mismatch"
        )));
    }
    Ok(())
}

fn validate_text(source: &str, path: &Path) -> Result<(), ArtifactError> {
    if source.is_empty()
        || !source.ends_with('\n')
        || source.contains('\r')
        || source.contains('\0')
    {
        return Err(ArtifactError::new(format!(
            "compiler stage handoff v2 `{}` must use canonical UTF-8/LF text",
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
        "compiler stage handoff v2 {label} is not a stable token"
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
        "compiler stage handoff v2 {label} must be a sibling file"
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
        "compiler stage handoff v2 {label} must be lowercase SHA-256"
    )))
}

fn handoff_identity(handoff: &CompilerStageHandoffV2) -> String {
    let mut hash = Sha256::new();
    for value in [
        handoff.protocol.as_bytes(),
        handoff.selection_contract.as_bytes(),
        handoff.authority.as_bytes(),
        handoff.producer_id.as_bytes(),
        handoff.base_handoff_protocol.as_bytes(),
        handoff.base_handoff_bundle_sha256.as_bytes(),
        handoff.stage_transformations_file.as_bytes(),
        handoff.stage_transformations_sha256.as_bytes(),
        handoff.stage_transformations_proof_sha256.as_bytes(),
        handoff.stage_semantic_differential_file.as_bytes(),
        handoff.stage_semantic_differential_sha256.as_bytes(),
        handoff.stage_semantic_differential_proof_sha256.as_bytes(),
        handoff.verdict.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [
        handoff.stage_transformations_bytes,
        handoff.stage_semantic_differential_bytes,
        handoff.selection_count,
        usize::from(handoff.all_reversible),
        usize::from(handoff.replacement_authorized),
    ] {
        hash_field(&mut hash, &(value as u64).to_le_bytes());
    }
    for selection in &handoff.selections {
        for value in [
            selection.ordinal,
            selection.transformation_ordinal,
            selection.comparison_ordinal,
            selection.derived_payload_bytes,
            usize::from(selection.reversible),
            usize::from(selection.semantically_equivalent),
        ] {
            hash_field(&mut hash, &(value as u64).to_le_bytes());
        }
        for value in [
            selection.source_stage.as_str().as_bytes(),
            selection.source_encoding.as_bytes(),
            selection.source_record_sha256.as_bytes(),
            selection.source_payload_sha256.as_bytes(),
            selection.transform_contract.as_bytes(),
            selection.derived_encoding.as_bytes(),
            selection.derived_payload_file.as_bytes(),
            selection.derived_payload_sha256.as_bytes(),
            selection.checkpoint_sha256.as_bytes(),
            selection.recovered_source_payload_sha256.as_bytes(),
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

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "compiler_stage_handoff_v2_tests.rs"]
mod tests;
