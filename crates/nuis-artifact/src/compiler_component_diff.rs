use std::{collections::BTreeMap, fs, path::Path};

use crate::{
    parse_compiler_component_build_from_source, parse_compiler_diagnostic_report_from_source,
    parse_compiler_stage_handoff_from_source, read_compiler_component_build,
    read_compiler_diagnostic_report, read_compiler_stage_handoff, render_compiler_component_build,
    render_compiler_diagnostic_report, render_compiler_stage_handoff,
    toml::{
        escape_toml_string, parse_optional_map_usize, parse_required_map_string_in_block,
        parse_required_toml_bool, parse_required_toml_string, parse_required_toml_usize,
    },
    ArtifactError, CompilerComponentBuild, CompilerDiagnosticReport, CompilerStageHandoff,
    COMPILER_COMPONENT_STAGE0_ROLE, COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE,
    COMPILER_DIAGNOSTIC_REPORT_FILE,
};

#[path = "compiler_component_diff_identity.rs"]
mod identity;

use identity::{differential_report_identity, value_identity};

pub const COMPILER_COMPONENT_DIFFERENTIAL_PROTOCOL: &str =
    "nuis-compiler-component-differential-v1";
pub const COMPILER_COMPONENT_DIFFERENTIAL_GATE_CONTRACT: &str =
    "nuis-compiler-component-differential-gate-v1";
pub const COMPILER_COMPONENT_REPLACEMENT_AUTHORITY_CONTRACT: &str =
    "nuis-compiler-replacement-authorization-separate-v1";
pub const COMPILER_COMPONENT_DIFFERENTIAL_FILE: &str = "nuis.compiler-component-diff.toml";

const SUBJECTS: [&str; 13] = [
    "component-id",
    "component-domain",
    "component-unit",
    "bootstrap-subset",
    "stage-source",
    "stage-tokens",
    "stage-ast",
    "stage-nir",
    "stage-yir",
    "stage-bundle",
    "diagnostics",
    "dependency-closure",
    "native-output",
];

#[derive(Debug, Clone, Copy)]
pub struct CompilerComponentEvidence<'a> {
    pub component: &'a CompilerComponentBuild,
    pub handoff: &'a CompilerStageHandoff,
    pub diagnostics: &'a CompilerDiagnosticReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentComparison {
    pub ordinal: usize,
    pub subject: String,
    pub stage0_sha256: String,
    pub candidate_sha256: String,
    pub equivalent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentDifferential {
    pub protocol: String,
    pub gate_contract: String,
    pub replacement_authority_contract: String,
    pub component_id: String,
    pub stage0_producer_id: String,
    pub candidate_producer_id: String,
    pub stage0_record_sha256: String,
    pub candidate_record_sha256: String,
    pub stage0_compiler_image_sha256: String,
    pub candidate_compiler_image_sha256: String,
    pub comparison_count: usize,
    pub equivalent_count: usize,
    pub stage_equivalent: bool,
    pub diagnostics_equivalent: bool,
    pub dependency_closure_equivalent: bool,
    pub native_output_equivalent: bool,
    pub deterministic_artifact_equivalent: bool,
    pub replacement_authorized: bool,
    pub verdict: String,
    pub report_sha256: String,
    pub comparisons: Vec<CompilerComponentComparison>,
}

pub fn build_compiler_component_differential(
    stage0: CompilerComponentEvidence<'_>,
    candidate: CompilerComponentEvidence<'_>,
) -> Result<CompilerComponentDifferential, ArtifactError> {
    validate_evidence(stage0, COMPILER_COMPONENT_STAGE0_ROLE)?;
    validate_evidence(candidate, COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE)?;
    if stage0.component.producer_id == candidate.component.producer_id {
        return Err(ArtifactError::new(
            "compiler differential requires two separately identified producers",
        ));
    }

    let stage0_records = &stage0.handoff.records;
    let candidate_records = &candidate.handoff.records;
    let pairs = [
        comparison_values(
            "component-id",
            stage0.component.component_id.as_bytes(),
            candidate.component.component_id.as_bytes(),
        ),
        comparison_values(
            "component-domain",
            stage0.component.component_domain.as_bytes(),
            candidate.component.component_domain.as_bytes(),
        ),
        comparison_values(
            "component-unit",
            stage0.component.component_unit.as_bytes(),
            candidate.component.component_unit.as_bytes(),
        ),
        comparison_values(
            "bootstrap-subset",
            stage0.component.bootstrap_subset_protocol.as_bytes(),
            candidate.component.bootstrap_subset_protocol.as_bytes(),
        ),
        comparison_hashes(
            "stage-source",
            &stage0_records[0].payload_sha256,
            &candidate_records[0].payload_sha256,
        ),
        comparison_hashes(
            "stage-tokens",
            &stage0_records[1].payload_sha256,
            &candidate_records[1].payload_sha256,
        ),
        comparison_hashes(
            "stage-ast",
            &stage0_records[2].payload_sha256,
            &candidate_records[2].payload_sha256,
        ),
        comparison_hashes(
            "stage-nir",
            &stage0_records[3].payload_sha256,
            &candidate_records[3].payload_sha256,
        ),
        comparison_hashes(
            "stage-yir",
            &stage0_records[4].payload_sha256,
            &candidate_records[4].payload_sha256,
        ),
        comparison_hashes(
            "stage-bundle",
            &stage0.handoff.bundle_sha256,
            &candidate.handoff.bundle_sha256,
        ),
        comparison_hashes(
            "diagnostics",
            &stage0.diagnostics.diagnostics_sha256,
            &candidate.diagnostics.diagnostics_sha256,
        ),
        comparison_hashes(
            "dependency-closure",
            &stage0.component.dependency_closure_sha256,
            &candidate.component.dependency_closure_sha256,
        ),
        comparison_hashes(
            "native-output",
            &stage0.component.native_binary_sha256,
            &candidate.component.native_binary_sha256,
        ),
    ];
    let comparisons = pairs
        .into_iter()
        .enumerate()
        .map(
            |(ordinal, (subject, stage0_sha256, candidate_sha256))| CompilerComponentComparison {
                ordinal,
                subject: subject.to_owned(),
                equivalent: stage0_sha256 == candidate_sha256,
                stage0_sha256,
                candidate_sha256,
            },
        )
        .collect::<Vec<_>>();
    let equivalent_count = comparisons
        .iter()
        .filter(|comparison| comparison.equivalent)
        .count();
    let stage_equivalent = comparisons[4..=9]
        .iter()
        .all(|comparison| comparison.equivalent);
    let diagnostics_equivalent = comparisons[10].equivalent;
    let dependency_closure_equivalent = comparisons[11].equivalent;
    let native_output_equivalent = comparisons[12].equivalent;
    let deterministic_artifact_equivalent = equivalent_count == comparisons.len();
    let verdict = if deterministic_artifact_equivalent {
        "equivalent-awaiting-authorization"
    } else {
        "blocked-drift"
    };
    let mut report = CompilerComponentDifferential {
        protocol: COMPILER_COMPONENT_DIFFERENTIAL_PROTOCOL.to_owned(),
        gate_contract: COMPILER_COMPONENT_DIFFERENTIAL_GATE_CONTRACT.to_owned(),
        replacement_authority_contract: COMPILER_COMPONENT_REPLACEMENT_AUTHORITY_CONTRACT
            .to_owned(),
        component_id: stage0.component.component_id.clone(),
        stage0_producer_id: stage0.component.producer_id.clone(),
        candidate_producer_id: candidate.component.producer_id.clone(),
        stage0_record_sha256: stage0.component.record_sha256.clone(),
        candidate_record_sha256: candidate.component.record_sha256.clone(),
        stage0_compiler_image_sha256: stage0.component.compiler_image_sha256.clone(),
        candidate_compiler_image_sha256: candidate.component.compiler_image_sha256.clone(),
        comparison_count: comparisons.len(),
        equivalent_count,
        stage_equivalent,
        diagnostics_equivalent,
        dependency_closure_equivalent,
        native_output_equivalent,
        deterministic_artifact_equivalent,
        replacement_authorized: false,
        verdict: verdict.to_owned(),
        report_sha256: String::new(),
        comparisons,
    };
    report.report_sha256 = differential_report_identity(&report);
    validate_compiler_component_differential(&report)?;
    Ok(report)
}

pub fn compare_compiler_component_paths(
    stage0_path: &Path,
    candidate_path: &Path,
) -> Result<CompilerComponentDifferential, ArtifactError> {
    let stage0 = read_evidence(stage0_path)?;
    let candidate = read_evidence(candidate_path)?;
    build_compiler_component_differential(
        CompilerComponentEvidence {
            component: &stage0.0,
            handoff: &stage0.1,
            diagnostics: &stage0.2,
        },
        CompilerComponentEvidence {
            component: &candidate.0,
            handoff: &candidate.1,
            diagnostics: &candidate.2,
        },
    )
}

pub fn render_compiler_component_differential(report: &CompilerComponentDifferential) -> String {
    let mut out = format!(
        "protocol = \"{}\"\ngate_contract = \"{}\"\nreplacement_authority_contract = \"{}\"\ncomponent_id = \"{}\"\nstage0_producer_id = \"{}\"\ncandidate_producer_id = \"{}\"\nstage0_record_sha256 = \"{}\"\ncandidate_record_sha256 = \"{}\"\nstage0_compiler_image_sha256 = \"{}\"\ncandidate_compiler_image_sha256 = \"{}\"\ncomparison_count = {}\nequivalent_count = {}\nstage_equivalent = {}\ndiagnostics_equivalent = {}\ndependency_closure_equivalent = {}\nnative_output_equivalent = {}\ndeterministic_artifact_equivalent = {}\nreplacement_authorized = {}\nverdict = \"{}\"\nreport_sha256 = \"{}\"\n",
        report.protocol,
        report.gate_contract,
        report.replacement_authority_contract,
        escape_toml_string(&report.component_id),
        escape_toml_string(&report.stage0_producer_id),
        escape_toml_string(&report.candidate_producer_id),
        report.stage0_record_sha256,
        report.candidate_record_sha256,
        report.stage0_compiler_image_sha256,
        report.candidate_compiler_image_sha256,
        report.comparison_count,
        report.equivalent_count,
        report.stage_equivalent,
        report.diagnostics_equivalent,
        report.dependency_closure_equivalent,
        report.native_output_equivalent,
        report.deterministic_artifact_equivalent,
        report.replacement_authorized,
        report.verdict,
        report.report_sha256,
    );
    for comparison in &report.comparisons {
        out.push_str(&format!(
            "\n[[comparison]]\nordinal = {}\nsubject = \"{}\"\nstage0_sha256 = \"{}\"\ncandidate_sha256 = \"{}\"\nequivalent = {}\n",
            comparison.ordinal,
            comparison.subject,
            comparison.stage0_sha256,
            comparison.candidate_sha256,
            comparison.equivalent,
        ));
    }
    out
}

pub fn parse_compiler_component_differential(
    path: &Path,
) -> Result<CompilerComponentDifferential, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler component differential `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_component_differential_from_source(&source, path)
}

pub fn parse_compiler_component_differential_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerComponentDifferential, ArtifactError> {
    validate_text(source, path)?;
    let report = CompilerComponentDifferential {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        gate_contract: parse_required_toml_string(source, "gate_contract", path)?,
        replacement_authority_contract: parse_required_toml_string(
            source,
            "replacement_authority_contract",
            path,
        )?,
        component_id: parse_required_toml_string(source, "component_id", path)?,
        stage0_producer_id: parse_required_toml_string(source, "stage0_producer_id", path)?,
        candidate_producer_id: parse_required_toml_string(source, "candidate_producer_id", path)?,
        stage0_record_sha256: parse_required_toml_string(source, "stage0_record_sha256", path)?,
        candidate_record_sha256: parse_required_toml_string(
            source,
            "candidate_record_sha256",
            path,
        )?,
        stage0_compiler_image_sha256: parse_required_toml_string(
            source,
            "stage0_compiler_image_sha256",
            path,
        )?,
        candidate_compiler_image_sha256: parse_required_toml_string(
            source,
            "candidate_compiler_image_sha256",
            path,
        )?,
        comparison_count: parse_required_toml_usize(source, "comparison_count", path)?,
        equivalent_count: parse_required_toml_usize(source, "equivalent_count", path)?,
        stage_equivalent: parse_required_toml_bool(source, "stage_equivalent", path)?,
        diagnostics_equivalent: parse_required_toml_bool(source, "diagnostics_equivalent", path)?,
        dependency_closure_equivalent: parse_required_toml_bool(
            source,
            "dependency_closure_equivalent",
            path,
        )?,
        native_output_equivalent: parse_required_toml_bool(
            source,
            "native_output_equivalent",
            path,
        )?,
        deterministic_artifact_equivalent: parse_required_toml_bool(
            source,
            "deterministic_artifact_equivalent",
            path,
        )?,
        replacement_authorized: parse_required_toml_bool(source, "replacement_authorized", path)?,
        verdict: parse_required_toml_string(source, "verdict", path)?,
        report_sha256: parse_required_toml_string(source, "report_sha256", path)?,
        comparisons: parse_comparison_blocks(source, path)?,
    };
    validate_compiler_component_differential(&report)?;
    if render_compiler_component_differential(&report) != source {
        return Err(ArtifactError::new(format!(
            "compiler component differential `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(report)
}

fn read_evidence(
    path: &Path,
) -> Result<
    (
        CompilerComponentBuild,
        CompilerStageHandoff,
        CompilerDiagnosticReport,
    ),
    ArtifactError,
> {
    let component = read_compiler_component_build(path)?;
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    let (handoff, _) = read_compiler_stage_handoff(&root.join(&component.stage_handoff_file))?;
    let diagnostics = read_compiler_diagnostic_report(
        &root.join(COMPILER_DIAGNOSTIC_REPORT_FILE),
        &component.record_sha256,
        &component.producer_id,
    )?;
    Ok((component, handoff, diagnostics))
}

fn validate_evidence(
    evidence: CompilerComponentEvidence<'_>,
    expected_role: &str,
) -> Result<(), ArtifactError> {
    parse_compiler_component_build_from_source(
        &render_compiler_component_build(evidence.component),
        Path::new("nuis.compiler-component-build.toml"),
    )?;
    parse_compiler_stage_handoff_from_source(
        &render_compiler_stage_handoff(evidence.handoff),
        Path::new("nuis.compiler-stage-handoff.toml"),
    )?;
    parse_compiler_diagnostic_report_from_source(
        &render_compiler_diagnostic_report(evidence.diagnostics),
        Path::new(COMPILER_DIAGNOSTIC_REPORT_FILE),
    )?;
    if evidence.component.stage_role != expected_role {
        return Err(ArtifactError::new(format!(
            "compiler differential expected component role `{expected_role}`, found `{}`",
            evidence.component.stage_role
        )));
    }
    if evidence.handoff.records.len() != 5
        || evidence.handoff.producer_id != evidence.component.producer_id
        || evidence.handoff.module_domain != evidence.component.component_domain
        || evidence.handoff.module_unit != evidence.component.component_unit
        || evidence.handoff.bundle_sha256 != evidence.component.stage_handoff_bundle_sha256
    {
        return Err(ArtifactError::new(
            "compiler differential stage evidence does not match its component",
        ));
    }
    if evidence.diagnostics.producer_id != evidence.component.producer_id
        || evidence.diagnostics.component_record_sha256 != evidence.component.record_sha256
        || evidence.diagnostics.bootstrap_subset_protocol
            != evidence.component.bootstrap_subset_protocol
        || !evidence.diagnostics.accepted
    {
        return Err(ArtifactError::new(
            "compiler differential diagnostic evidence does not match its accepted component",
        ));
    }
    Ok(())
}

fn parse_comparison_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<CompilerComponentComparison>, ArtifactError> {
    let mut records = Vec::new();
    let mut values = BTreeMap::new();
    let mut in_block = false;
    for raw in source.lines() {
        let line = raw.trim();
        if line == "[[comparison]]" {
            if in_block {
                records.push(parse_comparison(&values, path)?);
                values.clear();
            }
            in_block = true;
            continue;
        }
        if in_block && !line.is_empty() && !line.starts_with('#') {
            let Some((key, value)) = line.split_once('=') else {
                return Err(ArtifactError::new(format!(
                    "`{}` contains malformed compiler comparison line `{line}`",
                    path.display()
                )));
            };
            let key = key.trim().to_owned();
            if values
                .insert(key.clone(), value.trim().to_owned())
                .is_some()
            {
                return Err(ArtifactError::new(format!(
                    "`{}` compiler comparison repeats key `{key}`",
                    path.display()
                )));
            }
        }
    }
    if in_block {
        records.push(parse_comparison(&values, path)?);
    }
    Ok(records)
}

fn parse_comparison(
    values: &BTreeMap<String, String>,
    path: &Path,
) -> Result<CompilerComponentComparison, ArtifactError> {
    Ok(CompilerComponentComparison {
        ordinal: parse_optional_map_usize(values, "ordinal", path, "comparison")?.ok_or_else(
            || {
                ArtifactError::new(format!(
                    "`{}` comparison is missing `ordinal`",
                    path.display()
                ))
            },
        )?,
        subject: parse_required_map_string_in_block(values, "subject", path, "comparison")?,
        stage0_sha256: parse_required_map_string_in_block(
            values,
            "stage0_sha256",
            path,
            "comparison",
        )?,
        candidate_sha256: parse_required_map_string_in_block(
            values,
            "candidate_sha256",
            path,
            "comparison",
        )?,
        equivalent: parse_map_bool(values, "equivalent", path)?,
    })
}

fn parse_map_bool(
    values: &BTreeMap<String, String>,
    key: &str,
    path: &Path,
) -> Result<bool, ArtifactError> {
    match values.get(key).map(String::as_str) {
        Some("true") => Ok(true),
        Some("false") => Ok(false),
        _ => Err(ArtifactError::new(format!(
            "`{}` comparison key `{key}` must be a boolean",
            path.display()
        ))),
    }
}

fn validate_compiler_component_differential(
    report: &CompilerComponentDifferential,
) -> Result<(), ArtifactError> {
    if report.protocol != COMPILER_COMPONENT_DIFFERENTIAL_PROTOCOL
        || report.gate_contract != COMPILER_COMPONENT_DIFFERENTIAL_GATE_CONTRACT
        || report.replacement_authority_contract
            != COMPILER_COMPONENT_REPLACEMENT_AUTHORITY_CONTRACT
        || report.replacement_authorized
    {
        return Err(ArtifactError::new(
            "compiler component differential declares an unsupported or unsafe contract",
        ));
    }
    for (label, value) in [
        ("component id", report.component_id.as_str()),
        ("stage0 producer", report.stage0_producer_id.as_str()),
        ("candidate producer", report.candidate_producer_id.as_str()),
    ] {
        if value.is_empty() || value.contains('\r') || value.contains('\0') {
            return Err(ArtifactError::new(format!(
                "compiler component differential {label} is invalid"
            )));
        }
    }
    if report.stage0_producer_id == report.candidate_producer_id {
        return Err(ArtifactError::new(
            "compiler component differential producers must be distinct",
        ));
    }
    for (label, value) in [
        ("stage0 record", report.stage0_record_sha256.as_str()),
        ("candidate record", report.candidate_record_sha256.as_str()),
        (
            "stage0 compiler image",
            report.stage0_compiler_image_sha256.as_str(),
        ),
        (
            "candidate compiler image",
            report.candidate_compiler_image_sha256.as_str(),
        ),
        ("report", report.report_sha256.as_str()),
    ] {
        validate_sha256(value, label)?;
    }
    if report.comparison_count != SUBJECTS.len() || report.comparisons.len() != SUBJECTS.len() {
        return Err(ArtifactError::new(
            "compiler component differential has an incomplete comparison set",
        ));
    }
    for (ordinal, (comparison, expected_subject)) in
        report.comparisons.iter().zip(SUBJECTS).enumerate()
    {
        if comparison.ordinal != ordinal || comparison.subject != expected_subject {
            return Err(ArtifactError::new(
                "compiler component differential comparisons are not canonically ordered",
            ));
        }
        validate_sha256(&comparison.stage0_sha256, "stage0 comparison")?;
        validate_sha256(&comparison.candidate_sha256, "candidate comparison")?;
        if comparison.equivalent != (comparison.stage0_sha256 == comparison.candidate_sha256) {
            return Err(ArtifactError::new(
                "compiler component differential comparison verdict mismatch",
            ));
        }
    }
    let equivalent_count = report
        .comparisons
        .iter()
        .filter(|comparison| comparison.equivalent)
        .count();
    let stage_equivalent = report.comparisons[4..=9]
        .iter()
        .all(|comparison| comparison.equivalent);
    let all_equivalent = equivalent_count == report.comparisons.len();
    if report.equivalent_count != equivalent_count
        || report.stage_equivalent != stage_equivalent
        || report.diagnostics_equivalent != report.comparisons[10].equivalent
        || report.dependency_closure_equivalent != report.comparisons[11].equivalent
        || report.native_output_equivalent != report.comparisons[12].equivalent
        || report.deterministic_artifact_equivalent != all_equivalent
    {
        return Err(ArtifactError::new(
            "compiler component differential aggregate verdict mismatch",
        ));
    }
    let expected_verdict = if all_equivalent {
        "equivalent-awaiting-authorization"
    } else {
        "blocked-drift"
    };
    if report.verdict != expected_verdict {
        return Err(ArtifactError::new(
            "compiler component differential top-level verdict mismatch",
        ));
    }
    if report.report_sha256 != differential_report_identity(report) {
        return Err(ArtifactError::new(
            "compiler component differential report identity mismatch",
        ));
    }
    Ok(())
}

fn comparison_values(
    subject: &'static str,
    stage0: &[u8],
    candidate: &[u8],
) -> (&'static str, String, String) {
    (
        subject,
        value_identity(subject, stage0),
        value_identity(subject, candidate),
    )
}

fn comparison_hashes(
    subject: &'static str,
    stage0: &str,
    candidate: &str,
) -> (&'static str, String, String) {
    (subject, stage0.to_owned(), candidate.to_owned())
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
            "compiler component differential {label} must be lowercase SHA-256"
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
            "compiler component differential `{}` must be UTF-8/LF text",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
#[path = "compiler_component_diff_tests.rs"]
mod tests;
