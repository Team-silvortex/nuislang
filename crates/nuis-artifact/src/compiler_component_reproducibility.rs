use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use crate::{
    compare_compiler_component_paths, parse_build_manifest, parse_compiler_candidate_production,
    parse_compiler_candidate_production_from_source, parse_compiler_component_build_from_source,
    parse_compiler_component_differential, parse_compiler_component_differential_from_source,
    read_compiler_component_build, render_compiler_candidate_production,
    render_compiler_component_build, render_compiler_component_differential,
    toml::{
        escape_toml_string, parse_optional_map_usize, parse_required_map_string_in_block,
        parse_required_toml_bool, parse_required_toml_string, parse_required_toml_usize,
    },
    ArtifactError, CompilerCandidateProduction, CompilerComponentBuild,
    CompilerComponentDifferential, COMPILER_CANDIDATE_PRODUCTION_FILE,
    COMPILER_COMPONENT_BUILD_FILE, COMPILER_COMPONENT_DIFFERENTIAL_FILE,
    COMPILER_COMPONENT_REPLACEMENT_AUTHORITY_CONTRACT, COMPILER_COMPONENT_STAGE0_ROLE,
    COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE,
};

#[path = "compiler_component_reproducibility_identity.rs"]
mod identity;

use identity::reproducibility_identity;

pub const COMPILER_COMPONENT_REPRODUCIBILITY_PROTOCOL: &str =
    "nuis-compiler-component-reproducibility-v1";
pub const COMPILER_COMPONENT_CLEAN_BUILD_CONTRACT: &str = "nuis-compiler-two-clean-build-roots-v1";
pub const COMPILER_COMPONENT_REPRODUCIBILITY_AUTHORITY: &str =
    "local-frontdoor-procedural-witness-no-independent-trust";
pub const COMPILER_COMPONENT_REPRODUCIBILITY_FILE: &str =
    "nuis.compiler-component-reproducibility.toml";

const CLEAN_ROOT_STATE: &str = "absent-or-empty-before-build";
const EXPECTED_RUN_COUNT: usize = 2;
const EXPECTED_COMPARISON_COUNT: usize = 13;
const EQUIVALENT_VERDICT: &str = "equivalent-awaiting-authorization";
const REPRODUCIBLE_VERDICT: &str = "reproducible-equivalent-awaiting-authorization";

#[derive(Debug, Clone, Copy)]
pub struct CompilerComponentReproducibilityRunInput<'a> {
    pub run_id: &'a str,
    pub clean_root_witness_sha256: &'a str,
    pub stage0: &'a CompilerComponentBuild,
    pub candidate: &'a CompilerComponentBuild,
    pub production: &'a CompilerCandidateProduction,
    pub differential: &'a CompilerComponentDifferential,
}

#[derive(Debug, Clone, Copy)]
pub struct CompilerComponentReproducibilityRootInput<'a> {
    pub run_id: &'a str,
    pub clean_root_witness_sha256: &'a str,
    pub root: &'a Path,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentReproducibilityRun {
    pub ordinal: usize,
    pub run_id: String,
    pub component_id: String,
    pub clean_root_state: String,
    pub clean_root_witness_sha256: String,
    pub stage0_record_sha256: String,
    pub stage0_reproducible_build_sha256: String,
    pub candidate_record_sha256: String,
    pub candidate_reproducible_build_sha256: String,
    pub candidate_compiler_image_sha256: String,
    pub native_output_sha256: String,
    pub production_proof_sha256: String,
    pub differential_report_sha256: String,
    pub comparison_count: usize,
    pub equivalent_count: usize,
    pub deterministic_artifact_equivalent: bool,
    pub differential_verdict: String,
    pub replacement_authorized: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentReproducibility {
    pub protocol: String,
    pub clean_build_contract: String,
    pub attestation_authority: String,
    pub replacement_authority_contract: String,
    pub component_id: String,
    pub run_count: usize,
    pub comparison_count: usize,
    pub equivalent_run_count: usize,
    pub stage0_reproducible_build_sha256: String,
    pub candidate_reproducible_build_sha256: String,
    pub candidate_compiler_image_sha256: String,
    pub native_output_sha256: String,
    pub differential_verdict: String,
    pub all_runs_equivalent: bool,
    pub replacement_authorized: bool,
    pub verdict: String,
    pub aggregate_sha256: String,
    pub runs: Vec<CompilerComponentReproducibilityRun>,
}

pub fn build_compiler_component_reproducibility(
    inputs: &[CompilerComponentReproducibilityRunInput<'_>],
) -> Result<CompilerComponentReproducibility, ArtifactError> {
    if inputs.len() != EXPECTED_RUN_COUNT {
        return Err(ArtifactError::new(format!(
            "compiler reproducibility requires exactly {EXPECTED_RUN_COUNT} clean runs"
        )));
    }
    let runs = inputs
        .iter()
        .enumerate()
        .map(|(ordinal, input)| build_run(ordinal, input))
        .collect::<Result<Vec<_>, _>>()?;
    build_from_runs(runs)
}

pub fn build_compiler_component_reproducibility_from_paths(
    inputs: &[CompilerComponentReproducibilityRootInput<'_>],
) -> Result<CompilerComponentReproducibility, ArtifactError> {
    if inputs.len() != EXPECTED_RUN_COUNT {
        return Err(ArtifactError::new(format!(
            "compiler reproducibility requires exactly {EXPECTED_RUN_COUNT} clean roots"
        )));
    }
    validate_distinct_roots(inputs.iter().map(|input| input.root))?;
    let owned = inputs
        .iter()
        .map(|input| read_run_evidence(input.root))
        .collect::<Result<Vec<_>, _>>()?;
    let run_inputs = inputs
        .iter()
        .zip(&owned)
        .map(
            |(input, evidence)| CompilerComponentReproducibilityRunInput {
                run_id: input.run_id,
                clean_root_witness_sha256: input.clean_root_witness_sha256,
                stage0: &evidence.stage0,
                candidate: &evidence.candidate,
                production: &evidence.production,
                differential: &evidence.differential,
            },
        )
        .collect::<Vec<_>>();
    build_compiler_component_reproducibility(&run_inputs)
}

pub fn render_compiler_component_reproducibility(
    report: &CompilerComponentReproducibility,
) -> String {
    let mut out = format!(
        "protocol = \"{}\"\nclean_build_contract = \"{}\"\nattestation_authority = \"{}\"\nreplacement_authority_contract = \"{}\"\ncomponent_id = \"{}\"\nrun_count = {}\ncomparison_count = {}\nequivalent_run_count = {}\nstage0_reproducible_build_sha256 = \"{}\"\ncandidate_reproducible_build_sha256 = \"{}\"\ncandidate_compiler_image_sha256 = \"{}\"\nnative_output_sha256 = \"{}\"\ndifferential_verdict = \"{}\"\nall_runs_equivalent = {}\nreplacement_authorized = {}\nverdict = \"{}\"\naggregate_sha256 = \"{}\"\n",
        report.protocol,
        report.clean_build_contract,
        report.attestation_authority,
        report.replacement_authority_contract,
        escape_toml_string(&report.component_id),
        report.run_count,
        report.comparison_count,
        report.equivalent_run_count,
        report.stage0_reproducible_build_sha256,
        report.candidate_reproducible_build_sha256,
        report.candidate_compiler_image_sha256,
        report.native_output_sha256,
        report.differential_verdict,
        report.all_runs_equivalent,
        report.replacement_authorized,
        report.verdict,
        report.aggregate_sha256,
    );
    for run in &report.runs {
        out.push_str(&format!(
            "\n[[run]]\nordinal = {}\nrun_id = \"{}\"\ncomponent_id = \"{}\"\nclean_root_state = \"{}\"\nclean_root_witness_sha256 = \"{}\"\nstage0_record_sha256 = \"{}\"\nstage0_reproducible_build_sha256 = \"{}\"\ncandidate_record_sha256 = \"{}\"\ncandidate_reproducible_build_sha256 = \"{}\"\ncandidate_compiler_image_sha256 = \"{}\"\nnative_output_sha256 = \"{}\"\nproduction_proof_sha256 = \"{}\"\ndifferential_report_sha256 = \"{}\"\ncomparison_count = {}\nequivalent_count = {}\ndeterministic_artifact_equivalent = {}\ndifferential_verdict = \"{}\"\nreplacement_authorized = {}\n",
            run.ordinal,
            run.run_id,
            escape_toml_string(&run.component_id),
            run.clean_root_state,
            run.clean_root_witness_sha256,
            run.stage0_record_sha256,
            run.stage0_reproducible_build_sha256,
            run.candidate_record_sha256,
            run.candidate_reproducible_build_sha256,
            run.candidate_compiler_image_sha256,
            run.native_output_sha256,
            run.production_proof_sha256,
            run.differential_report_sha256,
            run.comparison_count,
            run.equivalent_count,
            run.deterministic_artifact_equivalent,
            run.differential_verdict,
            run.replacement_authorized,
        ));
    }
    out
}

pub fn parse_compiler_component_reproducibility(
    path: &Path,
) -> Result<CompilerComponentReproducibility, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler reproducibility aggregate `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_component_reproducibility_from_source(&source, path)
}

pub fn parse_compiler_component_reproducibility_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerComponentReproducibility, ArtifactError> {
    validate_text(source, path)?;
    let report = CompilerComponentReproducibility {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        clean_build_contract: parse_required_toml_string(source, "clean_build_contract", path)?,
        attestation_authority: parse_required_toml_string(source, "attestation_authority", path)?,
        replacement_authority_contract: parse_required_toml_string(
            source,
            "replacement_authority_contract",
            path,
        )?,
        component_id: parse_required_toml_string(source, "component_id", path)?,
        run_count: parse_required_toml_usize(source, "run_count", path)?,
        comparison_count: parse_required_toml_usize(source, "comparison_count", path)?,
        equivalent_run_count: parse_required_toml_usize(source, "equivalent_run_count", path)?,
        stage0_reproducible_build_sha256: parse_required_toml_string(
            source,
            "stage0_reproducible_build_sha256",
            path,
        )?,
        candidate_reproducible_build_sha256: parse_required_toml_string(
            source,
            "candidate_reproducible_build_sha256",
            path,
        )?,
        candidate_compiler_image_sha256: parse_required_toml_string(
            source,
            "candidate_compiler_image_sha256",
            path,
        )?,
        native_output_sha256: parse_required_toml_string(source, "native_output_sha256", path)?,
        differential_verdict: parse_required_toml_string(source, "differential_verdict", path)?,
        all_runs_equivalent: parse_required_toml_bool(source, "all_runs_equivalent", path)?,
        replacement_authorized: parse_required_toml_bool(source, "replacement_authorized", path)?,
        verdict: parse_required_toml_string(source, "verdict", path)?,
        aggregate_sha256: parse_required_toml_string(source, "aggregate_sha256", path)?,
        runs: parse_run_blocks(source, path)?,
    };
    validate_report(&report)?;
    if render_compiler_component_reproducibility(&report) != source {
        return Err(ArtifactError::new(format!(
            "compiler reproducibility aggregate `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(report)
}

pub fn read_compiler_component_reproducibility(
    path: &Path,
    run_roots: &[PathBuf],
) -> Result<CompilerComponentReproducibility, ArtifactError> {
    let report = parse_compiler_component_reproducibility(path)?;
    if run_roots.len() != report.runs.len() {
        return Err(ArtifactError::new(
            "compiler reproducibility root count does not match its aggregate",
        ));
    }
    let root_inputs = report
        .runs
        .iter()
        .zip(run_roots)
        .map(|(run, root)| CompilerComponentReproducibilityRootInput {
            run_id: &run.run_id,
            clean_root_witness_sha256: &run.clean_root_witness_sha256,
            root,
        })
        .collect::<Vec<_>>();
    let rebuilt = build_compiler_component_reproducibility_from_paths(&root_inputs)?;
    if rebuilt != report {
        return Err(ArtifactError::new(
            "compiler reproducibility aggregate does not match its bound clean roots",
        ));
    }
    Ok(report)
}

struct OwnedRunEvidence {
    stage0: CompilerComponentBuild,
    candidate: CompilerComponentBuild,
    production: CompilerCandidateProduction,
    differential: CompilerComponentDifferential,
}

fn read_run_evidence(root: &Path) -> Result<OwnedRunEvidence, ArtifactError> {
    let stage0_path = root.join("stage0").join(COMPILER_COMPONENT_BUILD_FILE);
    let candidate_path = root
        .join("stage1-candidate")
        .join(COMPILER_COMPONENT_BUILD_FILE);
    let stage0 = read_compiler_component_build(&stage0_path)?;
    let manifest = parse_build_manifest(&root.join("stage0").join(&stage0.build_manifest_file))?;
    if manifest.compile_cache_status.as_deref() != Some("bypass") {
        return Err(ArtifactError::new(
            "compiler reproducibility requires compile-cache bypass evidence",
        ));
    }
    let candidate = read_compiler_component_build(&candidate_path)?;
    let rebuilt_differential = compare_compiler_component_paths(&stage0_path, &candidate_path)?;
    let differential =
        parse_compiler_component_differential(&root.join(COMPILER_COMPONENT_DIFFERENTIAL_FILE))?;
    if rebuilt_differential != differential {
        return Err(ArtifactError::new(
            "clean build differential does not match its bound component evidence",
        ));
    }
    let production = parse_compiler_candidate_production(
        &root
            .join("stage1-candidate")
            .join(COMPILER_CANDIDATE_PRODUCTION_FILE),
    )?;
    Ok(OwnedRunEvidence {
        stage0,
        candidate,
        production,
        differential,
    })
}

fn validate_distinct_roots<'a>(roots: impl Iterator<Item = &'a Path>) -> Result<(), ArtifactError> {
    let mut canonical_roots = BTreeSet::new();
    for root in roots {
        let canonical = fs::canonicalize(root).map_err(|error| {
            ArtifactError::new(format!(
                "failed to resolve clean build root `{}`: {error}",
                root.display()
            ))
        })?;
        if !canonical_roots.insert(canonical) {
            return Err(ArtifactError::new(
                "compiler reproducibility requires two distinct build roots",
            ));
        }
    }
    Ok(())
}

fn build_run(
    ordinal: usize,
    input: &CompilerComponentReproducibilityRunInput<'_>,
) -> Result<CompilerComponentReproducibilityRun, ArtifactError> {
    let expected_run_id = format!("clean-build-{ordinal}");
    if input.run_id != expected_run_id {
        return Err(ArtifactError::new(format!(
            "compiler reproducibility run {ordinal} must use id `{expected_run_id}`"
        )));
    }
    validate_sha256(input.clean_root_witness_sha256, "clean root witness")?;
    validate_bound_evidence(input)?;
    Ok(CompilerComponentReproducibilityRun {
        ordinal,
        run_id: input.run_id.to_owned(),
        component_id: input.stage0.component_id.clone(),
        clean_root_state: CLEAN_ROOT_STATE.to_owned(),
        clean_root_witness_sha256: input.clean_root_witness_sha256.to_owned(),
        stage0_record_sha256: input.stage0.record_sha256.clone(),
        stage0_reproducible_build_sha256: input.stage0.reproducible_build_sha256.clone(),
        candidate_record_sha256: input.candidate.record_sha256.clone(),
        candidate_reproducible_build_sha256: input.candidate.reproducible_build_sha256.clone(),
        candidate_compiler_image_sha256: input.candidate.compiler_image_sha256.clone(),
        native_output_sha256: input.candidate.native_binary_sha256.clone(),
        production_proof_sha256: input.production.proof_sha256.clone(),
        differential_report_sha256: input.differential.report_sha256.clone(),
        comparison_count: input.differential.comparison_count,
        equivalent_count: input.differential.equivalent_count,
        deterministic_artifact_equivalent: input.differential.deterministic_artifact_equivalent,
        differential_verdict: input.differential.verdict.clone(),
        replacement_authorized: false,
    })
}

fn validate_bound_evidence(
    input: &CompilerComponentReproducibilityRunInput<'_>,
) -> Result<(), ArtifactError> {
    parse_compiler_component_build_from_source(
        &render_compiler_component_build(input.stage0),
        Path::new(COMPILER_COMPONENT_BUILD_FILE),
    )?;
    parse_compiler_component_build_from_source(
        &render_compiler_component_build(input.candidate),
        Path::new(COMPILER_COMPONENT_BUILD_FILE),
    )?;
    parse_compiler_candidate_production_from_source(
        &render_compiler_candidate_production(input.production),
        Path::new(COMPILER_CANDIDATE_PRODUCTION_FILE),
    )?;
    parse_compiler_component_differential_from_source(
        &render_compiler_component_differential(input.differential),
        Path::new(COMPILER_COMPONENT_DIFFERENTIAL_FILE),
    )?;
    if input.stage0.stage_role != COMPILER_COMPONENT_STAGE0_ROLE
        || input.candidate.stage_role != COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE
        || input.stage0.component_id != input.candidate.component_id
    {
        return Err(ArtifactError::new(
            "compiler reproducibility requires one stage0 and matching stage1-candidate",
        ));
    }
    if input.production.stage0_component_sha256 != input.stage0.record_sha256
        || input.production.candidate_component_sha256 != input.candidate.record_sha256
        || input.production.candidate_producer_id != input.candidate.producer_id
        || input.production.candidate_compiler_image_sha256 != input.candidate.compiler_image_sha256
        || input.production.stage_handoff_bundle_sha256
            != input.candidate.stage_handoff_bundle_sha256
        || input.production.replacement_authorized
    {
        return Err(ArtifactError::new(
            "compiler reproducibility production proof does not bind its candidate",
        ));
    }
    if input.differential.component_id != input.stage0.component_id
        || input.differential.stage0_record_sha256 != input.stage0.record_sha256
        || input.differential.candidate_record_sha256 != input.candidate.record_sha256
        || input.differential.stage0_producer_id != input.stage0.producer_id
        || input.differential.candidate_producer_id != input.candidate.producer_id
        || input.differential.comparison_count != EXPECTED_COMPARISON_COUNT
        || input.differential.equivalent_count != EXPECTED_COMPARISON_COUNT
        || !input.differential.deterministic_artifact_equivalent
        || input.differential.verdict != EQUIVALENT_VERDICT
        || input.differential.replacement_authorized
    {
        return Err(ArtifactError::new(
            "compiler reproducibility requires a bound 13/13 non-authoritative differential",
        ));
    }
    Ok(())
}

fn build_from_runs(
    runs: Vec<CompilerComponentReproducibilityRun>,
) -> Result<CompilerComponentReproducibility, ArtifactError> {
    if runs.len() != EXPECTED_RUN_COUNT {
        return Err(ArtifactError::new(
            "compiler reproducibility aggregate requires two runs",
        ));
    }
    if runs[0].clean_root_witness_sha256 == runs[1].clean_root_witness_sha256 {
        return Err(ArtifactError::new(
            "compiler reproducibility clean root witnesses must be distinct",
        ));
    }
    for field in [
        "stage0 reproducible build",
        "candidate reproducible build",
        "candidate compiler image",
        "native output",
        "differential verdict",
    ] {
        let equivalent = match field {
            "stage0 reproducible build" => {
                runs[0].stage0_reproducible_build_sha256 == runs[1].stage0_reproducible_build_sha256
            }
            "candidate reproducible build" => {
                runs[0].candidate_reproducible_build_sha256
                    == runs[1].candidate_reproducible_build_sha256
            }
            "candidate compiler image" => {
                runs[0].candidate_compiler_image_sha256 == runs[1].candidate_compiler_image_sha256
            }
            "native output" => runs[0].native_output_sha256 == runs[1].native_output_sha256,
            _ => runs[0].differential_verdict == runs[1].differential_verdict,
        };
        if !equivalent {
            return Err(ArtifactError::new(format!(
                "compiler reproducibility {field} drifted across clean runs"
            )));
        }
    }
    let equivalent_run_count = runs
        .iter()
        .filter(|run| run.deterministic_artifact_equivalent)
        .count();
    let mut report = CompilerComponentReproducibility {
        protocol: COMPILER_COMPONENT_REPRODUCIBILITY_PROTOCOL.to_owned(),
        clean_build_contract: COMPILER_COMPONENT_CLEAN_BUILD_CONTRACT.to_owned(),
        attestation_authority: COMPILER_COMPONENT_REPRODUCIBILITY_AUTHORITY.to_owned(),
        replacement_authority_contract: COMPILER_COMPONENT_REPLACEMENT_AUTHORITY_CONTRACT
            .to_owned(),
        component_id: runs[0].component_id.clone(),
        run_count: runs.len(),
        comparison_count: EXPECTED_COMPARISON_COUNT,
        equivalent_run_count,
        stage0_reproducible_build_sha256: runs[0].stage0_reproducible_build_sha256.clone(),
        candidate_reproducible_build_sha256: runs[0].candidate_reproducible_build_sha256.clone(),
        candidate_compiler_image_sha256: runs[0].candidate_compiler_image_sha256.clone(),
        native_output_sha256: runs[0].native_output_sha256.clone(),
        differential_verdict: runs[0].differential_verdict.clone(),
        all_runs_equivalent: equivalent_run_count == EXPECTED_RUN_COUNT,
        replacement_authorized: false,
        verdict: REPRODUCIBLE_VERDICT.to_owned(),
        aggregate_sha256: String::new(),
        runs,
    };
    report.aggregate_sha256 = reproducibility_identity(&report);
    validate_report(&report)?;
    Ok(report)
}

fn validate_report(report: &CompilerComponentReproducibility) -> Result<(), ArtifactError> {
    if report.protocol != COMPILER_COMPONENT_REPRODUCIBILITY_PROTOCOL
        || report.clean_build_contract != COMPILER_COMPONENT_CLEAN_BUILD_CONTRACT
        || report.attestation_authority != COMPILER_COMPONENT_REPRODUCIBILITY_AUTHORITY
        || report.replacement_authority_contract
            != COMPILER_COMPONENT_REPLACEMENT_AUTHORITY_CONTRACT
    {
        return Err(ArtifactError::new(
            "compiler reproducibility protocol contract mismatch",
        ));
    }
    if report.run_count != EXPECTED_RUN_COUNT
        || report.runs.len() != EXPECTED_RUN_COUNT
        || report.comparison_count != EXPECTED_COMPARISON_COUNT
        || report.equivalent_run_count != EXPECTED_RUN_COUNT
        || !report.all_runs_equivalent
        || report.replacement_authorized
        || report.differential_verdict != EQUIVALENT_VERDICT
        || report.verdict != REPRODUCIBLE_VERDICT
    {
        return Err(ArtifactError::new(
            "compiler reproducibility aggregate verdict mismatch",
        ));
    }
    let mut witnesses = BTreeSet::new();
    for (ordinal, run) in report.runs.iter().enumerate() {
        if run.ordinal != ordinal
            || run.run_id != format!("clean-build-{ordinal}")
            || run.component_id != report.component_id
            || run.clean_root_state != CLEAN_ROOT_STATE
            || run.comparison_count != EXPECTED_COMPARISON_COUNT
            || run.equivalent_count != EXPECTED_COMPARISON_COUNT
            || !run.deterministic_artifact_equivalent
            || run.differential_verdict != EQUIVALENT_VERDICT
            || run.replacement_authorized
        {
            return Err(ArtifactError::new(
                "compiler reproducibility run verdict or ordering mismatch",
            ));
        }
        for (label, value) in [
            ("clean root witness", &run.clean_root_witness_sha256),
            ("stage0 record", &run.stage0_record_sha256),
            (
                "stage0 reproducible build",
                &run.stage0_reproducible_build_sha256,
            ),
            ("candidate record", &run.candidate_record_sha256),
            (
                "candidate reproducible build",
                &run.candidate_reproducible_build_sha256,
            ),
            (
                "candidate compiler image",
                &run.candidate_compiler_image_sha256,
            ),
            ("native output", &run.native_output_sha256),
            ("production proof", &run.production_proof_sha256),
            ("differential report", &run.differential_report_sha256),
        ] {
            validate_sha256(value, label)?;
        }
        if !witnesses.insert(&run.clean_root_witness_sha256) {
            return Err(ArtifactError::new(
                "compiler reproducibility clean root witnesses must be distinct",
            ));
        }
        if run.stage0_reproducible_build_sha256 != report.stage0_reproducible_build_sha256
            || run.candidate_reproducible_build_sha256 != report.candidate_reproducible_build_sha256
            || run.candidate_compiler_image_sha256 != report.candidate_compiler_image_sha256
            || run.native_output_sha256 != report.native_output_sha256
            || run.differential_verdict != report.differential_verdict
        {
            return Err(ArtifactError::new(
                "compiler reproducibility run does not match stable aggregate identity",
            ));
        }
    }
    for (label, value) in [
        (
            "stage0 reproducible build",
            &report.stage0_reproducible_build_sha256,
        ),
        (
            "candidate reproducible build",
            &report.candidate_reproducible_build_sha256,
        ),
        (
            "candidate compiler image",
            &report.candidate_compiler_image_sha256,
        ),
        ("native output", &report.native_output_sha256),
        ("aggregate", &report.aggregate_sha256),
    ] {
        validate_sha256(value, label)?;
    }
    if report.aggregate_sha256 != reproducibility_identity(report) {
        return Err(ArtifactError::new(
            "compiler reproducibility aggregate identity mismatch",
        ));
    }
    Ok(())
}

fn parse_run_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<CompilerComponentReproducibilityRun>, ArtifactError> {
    let mut runs = Vec::new();
    let mut values = BTreeMap::new();
    let mut in_block = false;
    for raw in source.lines() {
        let line = raw.trim();
        if line == "[[run]]" {
            if in_block {
                runs.push(parse_run(&values, path)?);
                values.clear();
            }
            in_block = true;
            continue;
        }
        if in_block && !line.is_empty() && !line.starts_with('#') {
            let Some((key, value)) = line.split_once('=') else {
                return Err(ArtifactError::new(format!(
                    "`{}` contains malformed reproducibility run line `{line}`",
                    path.display()
                )));
            };
            let key = key.trim().to_owned();
            if values
                .insert(key.clone(), value.trim().to_owned())
                .is_some()
            {
                return Err(ArtifactError::new(format!(
                    "`{}` reproducibility run repeats key `{key}`",
                    path.display()
                )));
            }
        }
    }
    if in_block {
        runs.push(parse_run(&values, path)?);
    }
    Ok(runs)
}

fn parse_run(
    values: &BTreeMap<String, String>,
    path: &Path,
) -> Result<CompilerComponentReproducibilityRun, ArtifactError> {
    let string = |key| parse_required_map_string_in_block(values, key, path, "run");
    let usize_value = |key| {
        parse_optional_map_usize(values, key, path, "run")?.ok_or_else(|| {
            ArtifactError::new(format!("`{}` run is missing `{key}`", path.display()))
        })
    };
    Ok(CompilerComponentReproducibilityRun {
        ordinal: usize_value("ordinal")?,
        run_id: string("run_id")?,
        component_id: string("component_id")?,
        clean_root_state: string("clean_root_state")?,
        clean_root_witness_sha256: string("clean_root_witness_sha256")?,
        stage0_record_sha256: string("stage0_record_sha256")?,
        stage0_reproducible_build_sha256: string("stage0_reproducible_build_sha256")?,
        candidate_record_sha256: string("candidate_record_sha256")?,
        candidate_reproducible_build_sha256: string("candidate_reproducible_build_sha256")?,
        candidate_compiler_image_sha256: string("candidate_compiler_image_sha256")?,
        native_output_sha256: string("native_output_sha256")?,
        production_proof_sha256: string("production_proof_sha256")?,
        differential_report_sha256: string("differential_report_sha256")?,
        comparison_count: usize_value("comparison_count")?,
        equivalent_count: usize_value("equivalent_count")?,
        deterministic_artifact_equivalent: parse_map_bool(
            values,
            "deterministic_artifact_equivalent",
            path,
        )?,
        differential_verdict: string("differential_verdict")?,
        replacement_authorized: parse_map_bool(values, "replacement_authorized", path)?,
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
            "`{}` run key `{key}` must be a boolean",
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
            "compiler reproducibility {label} must be lowercase SHA-256"
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
            "compiler reproducibility aggregate `{}` must be UTF-8/LF text",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
#[path = "compiler_component_reproducibility_tests.rs"]
mod tests;
