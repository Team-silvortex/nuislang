use std::{collections::BTreeMap, fs, path::Path};

use crate::{
    compare_compiler_component_paths, parse_compiler_component_differential_from_source,
    parse_compiler_stage_handoff_from_source, parse_compiler_stage_handoff_v2_from_source,
    read_compiler_component_build, read_compiler_stage_handoff, read_compiler_stage_handoff_v2,
    render_compiler_component_differential, render_compiler_stage_handoff,
    render_compiler_stage_handoff_v2,
    toml::{
        escape_toml_string, parse_optional_map_usize, parse_required_map_string_in_block,
        parse_required_toml_bool, parse_required_toml_string, parse_required_toml_usize,
    },
    ArtifactError, CompilerComponentDifferential, CompilerStageHandoff, CompilerStageHandoffV2,
    CompilerStageKind, COMPILER_COMPONENT_DIFFERENTIAL_FILE, COMPILER_STAGE_HANDOFF_V2_FILE,
    COMPILER_STAGE_HANDOFF_V2_PROTOCOL,
};

#[path = "compiler_component_representation_diff_identity.rs"]
mod identity;

use identity::representation_differential_identity;

pub const COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_PROTOCOL: &str =
    "nuis-compiler-component-representation-differential-v1";
pub const COMPILER_COMPONENT_REPRESENTATION_COMPARISON_CONTRACT: &str =
    "nuis-compiler-selected-reversible-representation-comparison-v1";
pub const COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_AUTHORITY: &str =
    "semantic-equivalence-evidence-no-replacement";
pub const COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_FILE: &str =
    "nuis.compiler-component-representation-diff.toml";

const EQUIVALENT_VERDICT: &str = "selected-representations-equivalent-awaiting-authorization";
const BLOCKED_VERDICT: &str = "blocked-representation-drift";
const BASE_COMPARISON_COUNT: usize = 13;

#[derive(Debug, Clone, Copy)]
pub struct CompilerComponentRepresentationDifferentialInput<'a> {
    pub base_differential: &'a CompilerComponentDifferential,
    pub stage0_handoff: &'a CompilerStageHandoff,
    pub candidate_handoff: &'a CompilerStageHandoff,
    pub candidate_handoff_v2: &'a CompilerStageHandoffV2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentRepresentationComparison {
    pub ordinal: usize,
    pub selection_ordinal: usize,
    pub transformation_ordinal: usize,
    pub semantic_comparison_ordinal: usize,
    pub base_comparison_ordinal: usize,
    pub subject: String,
    pub source_stage: CompilerStageKind,
    pub stage0_encoding: String,
    pub stage0_record_sha256: String,
    pub stage0_payload_sha256: String,
    pub candidate_source_encoding: String,
    pub candidate_source_record_sha256: String,
    pub candidate_source_payload_sha256: String,
    pub candidate_selected_encoding: String,
    pub candidate_selected_payload_file: String,
    pub candidate_selected_payload_bytes: usize,
    pub candidate_selected_payload_sha256: String,
    pub candidate_recovered_payload_sha256: String,
    pub transform_contract: String,
    pub checkpoint_sha256: String,
    pub byte_identical: bool,
    pub reversible: bool,
    pub semantically_equivalent: bool,
    pub equivalent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentRepresentationDifferential {
    pub protocol: String,
    pub comparison_contract: String,
    pub authority: String,
    pub component_id: String,
    pub base_differential_file: String,
    pub base_differential_report_sha256: String,
    pub stage0_handoff_bundle_sha256: String,
    pub candidate_handoff_bundle_sha256: String,
    pub candidate_handoff_v2_protocol: String,
    pub candidate_handoff_v2_proof_sha256: String,
    pub comparison_count: usize,
    pub equivalent_count: usize,
    pub all_representations_equivalent: bool,
    pub replacement_authorized: bool,
    pub verdict: String,
    pub report_sha256: String,
    pub comparisons: Vec<CompilerComponentRepresentationComparison>,
}

pub fn build_compiler_component_representation_differential(
    input: &CompilerComponentRepresentationDifferentialInput<'_>,
) -> Result<CompilerComponentRepresentationDifferential, ArtifactError> {
    validate_input(input)?;
    let mut comparisons = Vec::with_capacity(input.candidate_handoff_v2.selections.len());
    for selection in &input.candidate_handoff_v2.selections {
        let subject = format!("stage-{}", selection.source_stage.as_str());
        let (base_comparison_ordinal, base_comparison) = input
            .base_differential
            .comparisons
            .iter()
            .enumerate()
            .find(|(_, comparison)| comparison.subject == subject)
            .ok_or_else(|| {
                ArtifactError::new(format!(
                    "compiler representation differential base comparison `{subject}` is missing"
                ))
            })?;
        let stage0_record = stage_record(input.stage0_handoff, selection.source_stage, "stage0")?;
        let candidate_record =
            stage_record(input.candidate_handoff, selection.source_stage, "candidate")?;
        if base_comparison.stage0_sha256 != stage0_record.payload_sha256
            || base_comparison.candidate_sha256 != candidate_record.payload_sha256
            || selection.source_record_sha256 != candidate_record.record_sha256
            || selection.source_payload_sha256 != candidate_record.payload_sha256
            || selection.recovered_source_payload_sha256 != candidate_record.payload_sha256
        {
            return Err(ArtifactError::new(format!(
                "compiler representation differential `{subject}` lineage is inconsistent"
            )));
        }
        let equivalent = base_comparison.equivalent
            && selection.reversible
            && selection.semantically_equivalent
            && stage0_record.payload_sha256 == selection.recovered_source_payload_sha256;
        comparisons.push(CompilerComponentRepresentationComparison {
            ordinal: comparisons.len(),
            selection_ordinal: selection.ordinal,
            transformation_ordinal: selection.transformation_ordinal,
            semantic_comparison_ordinal: selection.comparison_ordinal,
            base_comparison_ordinal,
            subject,
            source_stage: selection.source_stage,
            stage0_encoding: stage0_record.encoding.clone(),
            stage0_record_sha256: stage0_record.record_sha256.clone(),
            stage0_payload_sha256: stage0_record.payload_sha256.clone(),
            candidate_source_encoding: selection.source_encoding.clone(),
            candidate_source_record_sha256: candidate_record.record_sha256.clone(),
            candidate_source_payload_sha256: candidate_record.payload_sha256.clone(),
            candidate_selected_encoding: selection.derived_encoding.clone(),
            candidate_selected_payload_file: selection.derived_payload_file.clone(),
            candidate_selected_payload_bytes: selection.derived_payload_bytes,
            candidate_selected_payload_sha256: selection.derived_payload_sha256.clone(),
            candidate_recovered_payload_sha256: selection.recovered_source_payload_sha256.clone(),
            transform_contract: selection.transform_contract.clone(),
            checkpoint_sha256: selection.checkpoint_sha256.clone(),
            byte_identical: stage0_record.payload_sha256 == selection.derived_payload_sha256,
            reversible: selection.reversible,
            semantically_equivalent: selection.semantically_equivalent,
            equivalent,
        });
    }
    let equivalent_count = comparisons
        .iter()
        .filter(|comparison| comparison.equivalent)
        .count();
    let all_representations_equivalent = equivalent_count == comparisons.len();
    let verdict = if all_representations_equivalent {
        EQUIVALENT_VERDICT
    } else {
        BLOCKED_VERDICT
    };
    let mut report = CompilerComponentRepresentationDifferential {
        protocol: COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_PROTOCOL.to_owned(),
        comparison_contract: COMPILER_COMPONENT_REPRESENTATION_COMPARISON_CONTRACT.to_owned(),
        authority: COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_AUTHORITY.to_owned(),
        component_id: input.base_differential.component_id.clone(),
        base_differential_file: COMPILER_COMPONENT_DIFFERENTIAL_FILE.to_owned(),
        base_differential_report_sha256: input.base_differential.report_sha256.clone(),
        stage0_handoff_bundle_sha256: input.stage0_handoff.bundle_sha256.clone(),
        candidate_handoff_bundle_sha256: input.candidate_handoff.bundle_sha256.clone(),
        candidate_handoff_v2_protocol: input.candidate_handoff_v2.protocol.clone(),
        candidate_handoff_v2_proof_sha256: input.candidate_handoff_v2.proof_sha256.clone(),
        comparison_count: comparisons.len(),
        equivalent_count,
        all_representations_equivalent,
        replacement_authorized: false,
        verdict: verdict.to_owned(),
        report_sha256: String::new(),
        comparisons,
    };
    report.report_sha256 = representation_differential_identity(&report);
    validate_report(&report)?;
    Ok(report)
}

pub fn compare_compiler_component_representation_paths(
    stage0_path: &Path,
    candidate_path: &Path,
) -> Result<
    (
        CompilerComponentDifferential,
        CompilerComponentRepresentationDifferential,
    ),
    ArtifactError,
> {
    let base_differential = compare_compiler_component_paths(stage0_path, candidate_path)?;
    let stage0_component = read_compiler_component_build(stage0_path)?;
    let candidate_component = read_compiler_component_build(candidate_path)?;
    let stage0_root = stage0_path.parent().unwrap_or_else(|| Path::new("."));
    let candidate_root = candidate_path.parent().unwrap_or_else(|| Path::new("."));
    let (stage0_handoff, _) =
        read_compiler_stage_handoff(&stage0_root.join(&stage0_component.stage_handoff_file))?;
    let (candidate_handoff, candidate_payloads) =
        read_compiler_stage_handoff(&candidate_root.join(&candidate_component.stage_handoff_file))?;
    let candidate_handoff_v2 = read_compiler_stage_handoff_v2(
        &candidate_root.join(COMPILER_STAGE_HANDOFF_V2_FILE),
        &candidate_handoff,
        &candidate_payloads,
    )?;
    let representation_differential = build_compiler_component_representation_differential(
        &CompilerComponentRepresentationDifferentialInput {
            base_differential: &base_differential,
            stage0_handoff: &stage0_handoff,
            candidate_handoff: &candidate_handoff,
            candidate_handoff_v2: &candidate_handoff_v2,
        },
    )?;
    Ok((base_differential, representation_differential))
}

pub fn render_compiler_component_representation_differential(
    report: &CompilerComponentRepresentationDifferential,
) -> String {
    let mut out = format!(
        "protocol = \"{}\"\ncomparison_contract = \"{}\"\nauthority = \"{}\"\ncomponent_id = \"{}\"\nbase_differential_file = \"{}\"\nbase_differential_report_sha256 = \"{}\"\nstage0_handoff_bundle_sha256 = \"{}\"\ncandidate_handoff_bundle_sha256 = \"{}\"\ncandidate_handoff_v2_protocol = \"{}\"\ncandidate_handoff_v2_proof_sha256 = \"{}\"\ncomparison_count = {}\nequivalent_count = {}\nall_representations_equivalent = {}\nreplacement_authorized = {}\nverdict = \"{}\"\nreport_sha256 = \"{}\"\n",
        report.protocol,
        report.comparison_contract,
        report.authority,
        escape_toml_string(&report.component_id),
        report.base_differential_file,
        report.base_differential_report_sha256,
        report.stage0_handoff_bundle_sha256,
        report.candidate_handoff_bundle_sha256,
        report.candidate_handoff_v2_protocol,
        report.candidate_handoff_v2_proof_sha256,
        report.comparison_count,
        report.equivalent_count,
        report.all_representations_equivalent,
        report.replacement_authorized,
        report.verdict,
        report.report_sha256,
    );
    for comparison in &report.comparisons {
        out.push_str(&format!(
            "\n[[comparison]]\nordinal = {}\nselection_ordinal = {}\ntransformation_ordinal = {}\nsemantic_comparison_ordinal = {}\nbase_comparison_ordinal = {}\nsubject = \"{}\"\nsource_stage = \"{}\"\nstage0_encoding = \"{}\"\nstage0_record_sha256 = \"{}\"\nstage0_payload_sha256 = \"{}\"\ncandidate_source_encoding = \"{}\"\ncandidate_source_record_sha256 = \"{}\"\ncandidate_source_payload_sha256 = \"{}\"\ncandidate_selected_encoding = \"{}\"\ncandidate_selected_payload_file = \"{}\"\ncandidate_selected_payload_bytes = {}\ncandidate_selected_payload_sha256 = \"{}\"\ncandidate_recovered_payload_sha256 = \"{}\"\ntransform_contract = \"{}\"\ncheckpoint_sha256 = \"{}\"\nbyte_identical = {}\nreversible = {}\nsemantically_equivalent = {}\nequivalent = {}\n",
            comparison.ordinal,
            comparison.selection_ordinal,
            comparison.transformation_ordinal,
            comparison.semantic_comparison_ordinal,
            comparison.base_comparison_ordinal,
            comparison.subject,
            comparison.source_stage.as_str(),
            comparison.stage0_encoding,
            comparison.stage0_record_sha256,
            comparison.stage0_payload_sha256,
            comparison.candidate_source_encoding,
            comparison.candidate_source_record_sha256,
            comparison.candidate_source_payload_sha256,
            comparison.candidate_selected_encoding,
            escape_toml_string(&comparison.candidate_selected_payload_file),
            comparison.candidate_selected_payload_bytes,
            comparison.candidate_selected_payload_sha256,
            comparison.candidate_recovered_payload_sha256,
            comparison.transform_contract,
            comparison.checkpoint_sha256,
            comparison.byte_identical,
            comparison.reversible,
            comparison.semantically_equivalent,
            comparison.equivalent,
        ));
    }
    out
}

pub fn parse_compiler_component_representation_differential(
    path: &Path,
) -> Result<CompilerComponentRepresentationDifferential, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler representation differential `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_component_representation_differential_from_source(&source, path)
}

pub fn parse_compiler_component_representation_differential_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerComponentRepresentationDifferential, ArtifactError> {
    validate_text(source, path)?;
    let report = CompilerComponentRepresentationDifferential {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        comparison_contract: parse_required_toml_string(source, "comparison_contract", path)?,
        authority: parse_required_toml_string(source, "authority", path)?,
        component_id: parse_required_toml_string(source, "component_id", path)?,
        base_differential_file: parse_required_toml_string(source, "base_differential_file", path)?,
        base_differential_report_sha256: parse_required_toml_string(
            source,
            "base_differential_report_sha256",
            path,
        )?,
        stage0_handoff_bundle_sha256: parse_required_toml_string(
            source,
            "stage0_handoff_bundle_sha256",
            path,
        )?,
        candidate_handoff_bundle_sha256: parse_required_toml_string(
            source,
            "candidate_handoff_bundle_sha256",
            path,
        )?,
        candidate_handoff_v2_protocol: parse_required_toml_string(
            source,
            "candidate_handoff_v2_protocol",
            path,
        )?,
        candidate_handoff_v2_proof_sha256: parse_required_toml_string(
            source,
            "candidate_handoff_v2_proof_sha256",
            path,
        )?,
        comparison_count: parse_required_toml_usize(source, "comparison_count", path)?,
        equivalent_count: parse_required_toml_usize(source, "equivalent_count", path)?,
        all_representations_equivalent: parse_required_toml_bool(
            source,
            "all_representations_equivalent",
            path,
        )?,
        replacement_authorized: parse_required_toml_bool(source, "replacement_authorized", path)?,
        verdict: parse_required_toml_string(source, "verdict", path)?,
        report_sha256: parse_required_toml_string(source, "report_sha256", path)?,
        comparisons: parse_comparison_blocks(source, path)?,
    };
    validate_report(&report)?;
    if render_compiler_component_representation_differential(&report) != source {
        return Err(ArtifactError::new(format!(
            "compiler representation differential `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(report)
}

pub fn read_compiler_component_representation_differential(
    path: &Path,
    stage0_path: &Path,
    candidate_path: &Path,
) -> Result<CompilerComponentRepresentationDifferential, ArtifactError> {
    let report = parse_compiler_component_representation_differential(path)?;
    let (_, rebuilt) =
        compare_compiler_component_representation_paths(stage0_path, candidate_path)?;
    if report != rebuilt {
        return Err(ArtifactError::new(
            "compiler representation differential does not match its component evidence",
        ));
    }
    Ok(report)
}

fn validate_input(
    input: &CompilerComponentRepresentationDifferentialInput<'_>,
) -> Result<(), ArtifactError> {
    parse_compiler_component_differential_from_source(
        &render_compiler_component_differential(input.base_differential),
        Path::new(COMPILER_COMPONENT_DIFFERENTIAL_FILE),
    )?;
    parse_compiler_stage_handoff_from_source(
        &render_compiler_stage_handoff(input.stage0_handoff),
        Path::new("stage0-handoff.toml"),
    )?;
    parse_compiler_stage_handoff_from_source(
        &render_compiler_stage_handoff(input.candidate_handoff),
        Path::new("candidate-handoff.toml"),
    )?;
    parse_compiler_stage_handoff_v2_from_source(
        &render_compiler_stage_handoff_v2(input.candidate_handoff_v2),
        Path::new(COMPILER_STAGE_HANDOFF_V2_FILE),
    )?;
    let bundle_comparison = input
        .base_differential
        .comparisons
        .iter()
        .find(|comparison| comparison.subject == "stage-bundle")
        .ok_or_else(|| {
            ArtifactError::new(
                "compiler representation differential base bundle comparison is missing",
            )
        })?;
    if input.base_differential.replacement_authorized
        || input.candidate_handoff_v2.replacement_authorized
        || input.base_differential.stage0_producer_id != input.stage0_handoff.producer_id
        || input.base_differential.candidate_producer_id != input.candidate_handoff.producer_id
        || input.candidate_handoff_v2.producer_id != input.candidate_handoff.producer_id
        || input.candidate_handoff_v2.base_handoff_bundle_sha256
            != input.candidate_handoff.bundle_sha256
        || bundle_comparison.stage0_sha256 != input.stage0_handoff.bundle_sha256
        || bundle_comparison.candidate_sha256 != input.candidate_handoff.bundle_sha256
        || !input.candidate_handoff_v2.all_reversible
        || input.candidate_handoff_v2.selections.is_empty()
    {
        return Err(ArtifactError::new(
            "compiler representation differential evidence lineage is inconsistent",
        ));
    }
    Ok(())
}

fn validate_report(
    report: &CompilerComponentRepresentationDifferential,
) -> Result<(), ArtifactError> {
    if report.protocol != COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_PROTOCOL
        || report.comparison_contract != COMPILER_COMPONENT_REPRESENTATION_COMPARISON_CONTRACT
        || report.authority != COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_AUTHORITY
        || report.base_differential_file != COMPILER_COMPONENT_DIFFERENTIAL_FILE
        || report.candidate_handoff_v2_protocol != COMPILER_STAGE_HANDOFF_V2_PROTOCOL
        || report.replacement_authorized
    {
        return Err(ArtifactError::new(
            "compiler representation differential declares an unsupported or unsafe contract",
        ));
    }
    if report.component_id.is_empty()
        || report.component_id.contains('\r')
        || report.component_id.contains('\0')
    {
        return Err(ArtifactError::new(
            "compiler representation differential component id is invalid",
        ));
    }
    for (label, value) in [
        (
            "base differential",
            report.base_differential_report_sha256.as_str(),
        ),
        (
            "stage0 bundle",
            report.stage0_handoff_bundle_sha256.as_str(),
        ),
        (
            "candidate bundle",
            report.candidate_handoff_bundle_sha256.as_str(),
        ),
        (
            "candidate handoff v2 proof",
            report.candidate_handoff_v2_proof_sha256.as_str(),
        ),
        ("report", report.report_sha256.as_str()),
    ] {
        validate_sha256(value, label)?;
    }
    if report.comparison_count == 0 || report.comparison_count != report.comparisons.len() {
        return Err(ArtifactError::new(
            "compiler representation differential comparison count is invalid",
        ));
    }
    let mut stages = std::collections::BTreeSet::new();
    let mut base_ordinals = std::collections::BTreeSet::new();
    for (ordinal, comparison) in report.comparisons.iter().enumerate() {
        let expected_subject = format!("stage-{}", comparison.source_stage.as_str());
        if comparison.ordinal != ordinal
            || comparison.selection_ordinal != ordinal
            || comparison.transformation_ordinal != comparison.selection_ordinal
            || comparison.semantic_comparison_ordinal != comparison.selection_ordinal
            || comparison.base_comparison_ordinal >= BASE_COMPARISON_COUNT
            || !base_ordinals.insert(comparison.base_comparison_ordinal)
            || comparison.subject != expected_subject
            || !stages.insert(comparison.source_stage.as_str())
            || comparison.stage0_encoding != comparison.source_stage.encoding()
            || comparison.candidate_source_encoding != comparison.source_stage.encoding()
            || comparison.candidate_selected_payload_bytes == 0
            || comparison.byte_identical
            || !comparison.reversible
            || !comparison.semantically_equivalent
            || comparison.candidate_selected_payload_sha256
                == comparison.candidate_source_payload_sha256
            || comparison.candidate_source_payload_sha256
                != comparison.candidate_recovered_payload_sha256
        {
            return Err(ArtifactError::new(format!(
                "compiler representation comparison {ordinal} is invalid"
            )));
        }
        validate_token(&comparison.candidate_selected_encoding, "selected encoding")?;
        validate_token(&comparison.transform_contract, "transform contract")?;
        validate_file_name(
            &comparison.candidate_selected_payload_file,
            "selected payload",
        )?;
        for (label, value) in [
            ("stage0 record", comparison.stage0_record_sha256.as_str()),
            ("stage0 payload", comparison.stage0_payload_sha256.as_str()),
            (
                "candidate source record",
                comparison.candidate_source_record_sha256.as_str(),
            ),
            (
                "candidate source payload",
                comparison.candidate_source_payload_sha256.as_str(),
            ),
            (
                "candidate selected payload",
                comparison.candidate_selected_payload_sha256.as_str(),
            ),
            (
                "candidate recovered payload",
                comparison.candidate_recovered_payload_sha256.as_str(),
            ),
            ("checkpoint", comparison.checkpoint_sha256.as_str()),
        ] {
            validate_sha256(value, label)?;
        }
        let equivalent = comparison.stage0_payload_sha256
            == comparison.candidate_recovered_payload_sha256
            && comparison.reversible
            && comparison.semantically_equivalent;
        if comparison.byte_identical
            != (comparison.stage0_payload_sha256 == comparison.candidate_selected_payload_sha256)
            || comparison.equivalent != equivalent
        {
            return Err(ArtifactError::new(format!(
                "compiler representation comparison {ordinal} verdict mismatch"
            )));
        }
    }
    let equivalent_count = report
        .comparisons
        .iter()
        .filter(|comparison| comparison.equivalent)
        .count();
    let all_equivalent = equivalent_count == report.comparisons.len();
    let verdict = if all_equivalent {
        EQUIVALENT_VERDICT
    } else {
        BLOCKED_VERDICT
    };
    if report.equivalent_count != equivalent_count
        || report.all_representations_equivalent != all_equivalent
        || report.verdict != verdict
    {
        return Err(ArtifactError::new(
            "compiler representation differential aggregate verdict mismatch",
        ));
    }
    if report.report_sha256 != representation_differential_identity(report) {
        return Err(ArtifactError::new(
            "compiler representation differential report identity mismatch",
        ));
    }
    Ok(())
}

fn stage_record<'a>(
    handoff: &'a CompilerStageHandoff,
    stage: CompilerStageKind,
    side: &str,
) -> Result<&'a crate::CompilerStageHandoffRecord, ArtifactError> {
    handoff
        .records
        .iter()
        .find(|record| record.stage == stage)
        .ok_or_else(|| {
            ArtifactError::new(format!(
                "compiler representation differential {side} stage `{}` is missing",
                stage.as_str()
            ))
        })
}

fn parse_comparison_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<CompilerComponentRepresentationComparison>, ArtifactError> {
    let mut blocks = Vec::new();
    let mut current: Option<BTreeMap<String, String>> = None;
    for raw in source.lines() {
        let line = raw.trim();
        if line == "[[comparison]]" {
            if let Some(values) = current.take() {
                blocks.push(parse_comparison(values, path)?);
            }
            current = Some(BTreeMap::new());
            continue;
        }
        if line.is_empty() || line.starts_with('#') || current.is_none() {
            continue;
        }
        let (key, value) = line.split_once('=').ok_or_else(|| {
            ArtifactError::new(format!(
                "`{}` contains malformed representation comparison line `{line}`",
                path.display()
            ))
        })?;
        let values = current.as_mut().expect("comparison block is active");
        if values
            .insert(key.trim().to_owned(), value.trim().to_owned())
            .is_some()
        {
            return Err(ArtifactError::new(format!(
                "`{}` repeats representation comparison key `{}`",
                path.display(),
                key.trim()
            )));
        }
    }
    if let Some(values) = current {
        blocks.push(parse_comparison(values, path)?);
    }
    Ok(blocks)
}

fn parse_comparison(
    values: BTreeMap<String, String>,
    path: &Path,
) -> Result<CompilerComponentRepresentationComparison, ArtifactError> {
    if values.len() != 24 {
        return Err(ArtifactError::new(format!(
            "`{}` representation comparison contains unknown or missing keys",
            path.display()
        )));
    }
    let stage_name = required_map_string(&values, "source_stage", path)?;
    let source_stage = CompilerStageKind::parse(&stage_name).ok_or_else(|| {
        ArtifactError::new(format!(
            "`{}` representation comparison stage `{stage_name}` is unsupported",
            path.display()
        ))
    })?;
    Ok(CompilerComponentRepresentationComparison {
        ordinal: required_map_usize(&values, "ordinal", path)?,
        selection_ordinal: required_map_usize(&values, "selection_ordinal", path)?,
        transformation_ordinal: required_map_usize(&values, "transformation_ordinal", path)?,
        semantic_comparison_ordinal: required_map_usize(
            &values,
            "semantic_comparison_ordinal",
            path,
        )?,
        base_comparison_ordinal: required_map_usize(&values, "base_comparison_ordinal", path)?,
        subject: required_map_string(&values, "subject", path)?,
        source_stage,
        stage0_encoding: required_map_string(&values, "stage0_encoding", path)?,
        stage0_record_sha256: required_map_string(&values, "stage0_record_sha256", path)?,
        stage0_payload_sha256: required_map_string(&values, "stage0_payload_sha256", path)?,
        candidate_source_encoding: required_map_string(&values, "candidate_source_encoding", path)?,
        candidate_source_record_sha256: required_map_string(
            &values,
            "candidate_source_record_sha256",
            path,
        )?,
        candidate_source_payload_sha256: required_map_string(
            &values,
            "candidate_source_payload_sha256",
            path,
        )?,
        candidate_selected_encoding: required_map_string(
            &values,
            "candidate_selected_encoding",
            path,
        )?,
        candidate_selected_payload_file: required_map_string(
            &values,
            "candidate_selected_payload_file",
            path,
        )?,
        candidate_selected_payload_bytes: required_map_usize(
            &values,
            "candidate_selected_payload_bytes",
            path,
        )?,
        candidate_selected_payload_sha256: required_map_string(
            &values,
            "candidate_selected_payload_sha256",
            path,
        )?,
        candidate_recovered_payload_sha256: required_map_string(
            &values,
            "candidate_recovered_payload_sha256",
            path,
        )?,
        transform_contract: required_map_string(&values, "transform_contract", path)?,
        checkpoint_sha256: required_map_string(&values, "checkpoint_sha256", path)?,
        byte_identical: required_map_bool(&values, "byte_identical", path)?,
        reversible: required_map_bool(&values, "reversible", path)?,
        semantically_equivalent: required_map_bool(&values, "semantically_equivalent", path)?,
        equivalent: required_map_bool(&values, "equivalent", path)?,
    })
}

fn required_map_string(
    values: &BTreeMap<String, String>,
    key: &str,
    path: &Path,
) -> Result<String, ArtifactError> {
    parse_required_map_string_in_block(values, key, path, "representation comparison")
}

fn required_map_usize(
    values: &BTreeMap<String, String>,
    key: &str,
    path: &Path,
) -> Result<usize, ArtifactError> {
    parse_optional_map_usize(values, key, path, "representation comparison")?.ok_or_else(|| {
        ArtifactError::new(format!(
            "`{}` representation comparison is missing `{key}`",
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
            "`{}` representation comparison key `{key}` must be a boolean",
            path.display()
        ))),
    }
}

fn validate_sha256(value: &str, label: &str) -> Result<(), ArtifactError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(ArtifactError::new(format!(
            "compiler representation differential {label} must be lowercase SHA-256"
        )))
    }
}

fn validate_token(value: &str, label: &str) -> Result<(), ArtifactError> {
    if !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        Ok(())
    } else {
        Err(ArtifactError::new(format!(
            "compiler representation differential {label} is not a stable token"
        )))
    }
}

fn validate_file_name(value: &str, label: &str) -> Result<(), ArtifactError> {
    let path = Path::new(value);
    if !value.is_empty()
        && !value.contains('/')
        && !value.contains('\\')
        && path.components().count() == 1
        && path.file_name().and_then(|item| item.to_str()) == Some(value)
    {
        Ok(())
    } else {
        Err(ArtifactError::new(format!(
            "compiler representation differential {label} must be a sibling file"
        )))
    }
}

fn validate_text(source: &str, path: &Path) -> Result<(), ArtifactError> {
    if source.is_empty()
        || !source.ends_with('\n')
        || source.contains('\r')
        || source.contains('\0')
    {
        return Err(ArtifactError::new(format!(
            "compiler representation differential `{}` must use canonical UTF-8/LF text",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
#[path = "compiler_component_representation_diff_tests.rs"]
mod tests;
