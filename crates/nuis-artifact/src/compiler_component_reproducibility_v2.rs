use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use crate::{
    build_compiler_component_reproducibility_from_paths,
    parse_compiler_component_reproducibility_from_source,
    read_compiler_component_representation_differential, render_compiler_component_reproducibility,
    toml::{
        escape_toml_string, parse_optional_map_usize, parse_required_map_string_in_block,
        parse_required_toml_bool, parse_required_toml_string, parse_required_toml_usize,
    },
    ArtifactError, CompilerComponentReproducibility, CompilerComponentReproducibilityRootInput,
    COMPILER_COMPONENT_BUILD_FILE, COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_FILE,
    COMPILER_COMPONENT_REPRODUCIBILITY_FILE, COMPILER_COMPONENT_REPRODUCIBILITY_PROTOCOL,
};

pub const COMPILER_COMPONENT_REPRODUCIBILITY_V2_PROTOCOL: &str =
    "nuis-compiler-component-reproducibility-v2";
pub const COMPILER_COMPONENT_REPRODUCIBILITY_V2_FILE: &str =
    "nuis.compiler-component-reproducibility-v2.toml";
pub const COMPILER_COMPONENT_REPRODUCIBILITY_V2_BINDING_CONTRACT: &str =
    "nuis-two-clean-roots-selected-representation-sidecar-binding-v1";
pub const COMPILER_COMPONENT_REPRODUCIBILITY_V2_AUTHORITY: &str =
    "local-successor-evidence-no-independent-trust-or-replacement";
pub const COMPILER_COMPONENT_REPRODUCIBILITY_V2_VERDICT: &str =
    "reproducible-selected-representations-bound-awaiting-authorization";

const EXPECTED_RUN_COUNT: usize = 2;
const EXPECTED_REPRESENTATION_COMPARISONS_PER_RUN: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentReproducibilityV2Run {
    pub ordinal: usize,
    pub run_id: String,
    pub clean_root_witness_sha256: String,
    pub production_proof_sha256: String,
    pub base_differential_report_sha256: String,
    pub representation_differential_file: String,
    pub representation_differential_bytes: usize,
    pub representation_differential_sha256: String,
    pub representation_report_sha256: String,
    pub representation_comparison_count: usize,
    pub representation_equivalent_count: usize,
    pub all_representations_equivalent: bool,
    pub replacement_authorized: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerComponentReproducibilityV2 {
    pub protocol: String,
    pub binding_contract: String,
    pub authority: String,
    pub predecessor_protocol: String,
    pub predecessor_file: String,
    pub predecessor_bytes: usize,
    pub predecessor_sha256: String,
    pub predecessor_aggregate_sha256: String,
    pub component_id: String,
    pub run_count: usize,
    pub representation_comparison_count: usize,
    pub equivalent_representation_count: usize,
    pub all_selected_representations_equivalent: bool,
    pub sidecars_individually_bound: bool,
    pub predecessor_signature_compatible: bool,
    pub replacement_authorized: bool,
    pub verdict: String,
    pub aggregate_sha256: String,
    pub runs: Vec<CompilerComponentReproducibilityV2Run>,
}

pub fn build_compiler_component_reproducibility_v2_from_paths(
    predecessor: &CompilerComponentReproducibility,
    predecessor_source: &str,
    roots: &[PathBuf],
) -> Result<CompilerComponentReproducibilityV2, ArtifactError> {
    validate_predecessor(predecessor, predecessor_source, roots)?;
    let runs = roots
        .iter()
        .zip(&predecessor.runs)
        .enumerate()
        .map(|(ordinal, (root, predecessor_run))| {
            let stage0 = root.join("stage0").join(COMPILER_COMPONENT_BUILD_FILE);
            let candidate = root
                .join("stage1-candidate")
                .join(COMPILER_COMPONENT_BUILD_FILE);
            let sidecar_path = root.join(COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_FILE);
            let sidecar_source = fs::read_to_string(&sidecar_path).map_err(|error| {
                ArtifactError::new(format!(
                    "failed to read compiler representation sidecar `{}`: {error}",
                    sidecar_path.display()
                ))
            })?;
            let sidecar = read_compiler_component_representation_differential(
                &sidecar_path,
                &stage0,
                &candidate,
            )?;
            if sidecar.component_id != predecessor.component_id
                || sidecar.base_differential_report_sha256
                    != predecessor_run.differential_report_sha256
                || sidecar.comparison_count != EXPECTED_REPRESENTATION_COMPARISONS_PER_RUN
                || sidecar.equivalent_count != EXPECTED_REPRESENTATION_COMPARISONS_PER_RUN
                || !sidecar.all_representations_equivalent
                || sidecar.replacement_authorized
            {
                return Err(ArtifactError::new(format!(
                    "compiler reproducibility v2 run {ordinal} representation lineage is inconsistent"
                )));
            }
            Ok(CompilerComponentReproducibilityV2Run {
                ordinal,
                run_id: predecessor_run.run_id.clone(),
                clean_root_witness_sha256: predecessor_run.clean_root_witness_sha256.clone(),
                production_proof_sha256: predecessor_run.production_proof_sha256.clone(),
                base_differential_report_sha256: predecessor_run
                    .differential_report_sha256
                    .clone(),
                representation_differential_file:
                    COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_FILE.to_owned(),
                representation_differential_bytes: sidecar_source.len(),
                representation_differential_sha256: sha256_hex(sidecar_source.as_bytes()),
                representation_report_sha256: sidecar.report_sha256,
                representation_comparison_count: sidecar.comparison_count,
                representation_equivalent_count: sidecar.equivalent_count,
                all_representations_equivalent: sidecar.all_representations_equivalent,
                replacement_authorized: false,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    build_from_runs(predecessor, predecessor_source, runs)
}

fn validate_predecessor(
    predecessor: &CompilerComponentReproducibility,
    predecessor_source: &str,
    roots: &[PathBuf],
) -> Result<(), ArtifactError> {
    let parsed = parse_compiler_component_reproducibility_from_source(
        predecessor_source,
        Path::new(COMPILER_COMPONENT_REPRODUCIBILITY_FILE),
    )?;
    if &parsed != predecessor
        || render_compiler_component_reproducibility(predecessor) != predecessor_source
        || roots.len() != predecessor.runs.len()
    {
        return Err(ArtifactError::new(
            "compiler reproducibility v2 predecessor is not canonical or complete",
        ));
    }
    let root_inputs = predecessor
        .runs
        .iter()
        .zip(roots)
        .map(|(run, root)| CompilerComponentReproducibilityRootInput {
            run_id: &run.run_id,
            clean_root_witness_sha256: &run.clean_root_witness_sha256,
            root,
        })
        .collect::<Vec<_>>();
    if build_compiler_component_reproducibility_from_paths(&root_inputs)? != *predecessor {
        return Err(ArtifactError::new(
            "compiler reproducibility v2 predecessor changed its bound clean roots",
        ));
    }
    Ok(())
}

fn build_from_runs(
    predecessor: &CompilerComponentReproducibility,
    predecessor_source: &str,
    runs: Vec<CompilerComponentReproducibilityV2Run>,
) -> Result<CompilerComponentReproducibilityV2, ArtifactError> {
    if runs.len() != predecessor.runs.len()
        || runs.iter().zip(&predecessor.runs).any(|(run, prior)| {
            run.ordinal != prior.ordinal
                || run.run_id != prior.run_id
                || run.clean_root_witness_sha256 != prior.clean_root_witness_sha256
                || run.production_proof_sha256 != prior.production_proof_sha256
                || run.base_differential_report_sha256 != prior.differential_report_sha256
        })
    {
        return Err(ArtifactError::new(
            "compiler reproducibility v2 runs do not bind their v1 predecessor runs",
        ));
    }
    let equivalent_representation_count = runs
        .iter()
        .map(|run| run.representation_equivalent_count)
        .sum();
    let mut report = CompilerComponentReproducibilityV2 {
        protocol: COMPILER_COMPONENT_REPRODUCIBILITY_V2_PROTOCOL.to_owned(),
        binding_contract: COMPILER_COMPONENT_REPRODUCIBILITY_V2_BINDING_CONTRACT.to_owned(),
        authority: COMPILER_COMPONENT_REPRODUCIBILITY_V2_AUTHORITY.to_owned(),
        predecessor_protocol: predecessor.protocol.clone(),
        predecessor_file: COMPILER_COMPONENT_REPRODUCIBILITY_FILE.to_owned(),
        predecessor_bytes: predecessor_source.len(),
        predecessor_sha256: sha256_hex(predecessor_source.as_bytes()),
        predecessor_aggregate_sha256: predecessor.aggregate_sha256.clone(),
        component_id: predecessor.component_id.clone(),
        run_count: runs.len(),
        representation_comparison_count: runs
            .iter()
            .map(|run| run.representation_comparison_count)
            .sum(),
        equivalent_representation_count,
        all_selected_representations_equivalent: runs
            .iter()
            .all(|run| run.all_representations_equivalent),
        sidecars_individually_bound: runs.len() == EXPECTED_RUN_COUNT,
        predecessor_signature_compatible: true,
        replacement_authorized: false,
        verdict: COMPILER_COMPONENT_REPRODUCIBILITY_V2_VERDICT.to_owned(),
        aggregate_sha256: String::new(),
        runs,
    };
    report.aggregate_sha256 = reproducibility_v2_identity(&report);
    validate_report(&report)?;
    Ok(report)
}

pub fn render_compiler_component_reproducibility_v2(
    report: &CompilerComponentReproducibilityV2,
) -> String {
    let mut out = format!(
        "protocol = \"{}\"\nbinding_contract = \"{}\"\nauthority = \"{}\"\npredecessor_protocol = \"{}\"\npredecessor_file = \"{}\"\npredecessor_bytes = {}\npredecessor_sha256 = \"{}\"\npredecessor_aggregate_sha256 = \"{}\"\ncomponent_id = \"{}\"\nrun_count = {}\nrepresentation_comparison_count = {}\nequivalent_representation_count = {}\nall_selected_representations_equivalent = {}\nsidecars_individually_bound = {}\npredecessor_signature_compatible = {}\nreplacement_authorized = {}\nverdict = \"{}\"\naggregate_sha256 = \"{}\"\n",
        report.protocol,
        report.binding_contract,
        report.authority,
        report.predecessor_protocol,
        report.predecessor_file,
        report.predecessor_bytes,
        report.predecessor_sha256,
        report.predecessor_aggregate_sha256,
        escape_toml_string(&report.component_id),
        report.run_count,
        report.representation_comparison_count,
        report.equivalent_representation_count,
        report.all_selected_representations_equivalent,
        report.sidecars_individually_bound,
        report.predecessor_signature_compatible,
        report.replacement_authorized,
        report.verdict,
        report.aggregate_sha256,
    );
    for run in &report.runs {
        out.push_str(&format!(
            "\n[[run]]\nordinal = {}\nrun_id = \"{}\"\nclean_root_witness_sha256 = \"{}\"\nproduction_proof_sha256 = \"{}\"\nbase_differential_report_sha256 = \"{}\"\nrepresentation_differential_file = \"{}\"\nrepresentation_differential_bytes = {}\nrepresentation_differential_sha256 = \"{}\"\nrepresentation_report_sha256 = \"{}\"\nrepresentation_comparison_count = {}\nrepresentation_equivalent_count = {}\nall_representations_equivalent = {}\nreplacement_authorized = {}\n",
            run.ordinal,
            escape_toml_string(&run.run_id),
            run.clean_root_witness_sha256,
            run.production_proof_sha256,
            run.base_differential_report_sha256,
            run.representation_differential_file,
            run.representation_differential_bytes,
            run.representation_differential_sha256,
            run.representation_report_sha256,
            run.representation_comparison_count,
            run.representation_equivalent_count,
            run.all_representations_equivalent,
            run.replacement_authorized,
        ));
    }
    out
}

pub fn parse_compiler_component_reproducibility_v2(
    path: &Path,
) -> Result<CompilerComponentReproducibilityV2, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler reproducibility v2 `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_component_reproducibility_v2_from_source(&source, path)
}

pub fn parse_compiler_component_reproducibility_v2_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerComponentReproducibilityV2, ArtifactError> {
    validate_text(source, path)?;
    let report = CompilerComponentReproducibilityV2 {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        binding_contract: parse_required_toml_string(source, "binding_contract", path)?,
        authority: parse_required_toml_string(source, "authority", path)?,
        predecessor_protocol: parse_required_toml_string(source, "predecessor_protocol", path)?,
        predecessor_file: parse_required_toml_string(source, "predecessor_file", path)?,
        predecessor_bytes: parse_required_toml_usize(source, "predecessor_bytes", path)?,
        predecessor_sha256: parse_required_toml_string(source, "predecessor_sha256", path)?,
        predecessor_aggregate_sha256: parse_required_toml_string(
            source,
            "predecessor_aggregate_sha256",
            path,
        )?,
        component_id: parse_required_toml_string(source, "component_id", path)?,
        run_count: parse_required_toml_usize(source, "run_count", path)?,
        representation_comparison_count: parse_required_toml_usize(
            source,
            "representation_comparison_count",
            path,
        )?,
        equivalent_representation_count: parse_required_toml_usize(
            source,
            "equivalent_representation_count",
            path,
        )?,
        all_selected_representations_equivalent: parse_required_toml_bool(
            source,
            "all_selected_representations_equivalent",
            path,
        )?,
        sidecars_individually_bound: parse_required_toml_bool(
            source,
            "sidecars_individually_bound",
            path,
        )?,
        predecessor_signature_compatible: parse_required_toml_bool(
            source,
            "predecessor_signature_compatible",
            path,
        )?,
        replacement_authorized: parse_required_toml_bool(source, "replacement_authorized", path)?,
        verdict: parse_required_toml_string(source, "verdict", path)?,
        aggregate_sha256: parse_required_toml_string(source, "aggregate_sha256", path)?,
        runs: parse_run_blocks(source, path)?,
    };
    validate_report(&report)?;
    if render_compiler_component_reproducibility_v2(&report) != source {
        return Err(ArtifactError::new(format!(
            "compiler reproducibility v2 `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(report)
}

pub fn read_compiler_component_reproducibility_v2(
    path: &Path,
    predecessor_path: &Path,
    roots: &[PathBuf],
) -> Result<CompilerComponentReproducibilityV2, ArtifactError> {
    let report = parse_compiler_component_reproducibility_v2(path)?;
    let predecessor_source = fs::read_to_string(predecessor_path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler reproducibility predecessor `{}`: {error}",
            predecessor_path.display()
        ))
    })?;
    let predecessor = parse_compiler_component_reproducibility_from_source(
        &predecessor_source,
        predecessor_path,
    )?;
    let rebuilt = build_compiler_component_reproducibility_v2_from_paths(
        &predecessor,
        &predecessor_source,
        roots,
    )?;
    if rebuilt != report {
        return Err(ArtifactError::new(
            "compiler reproducibility v2 does not match its predecessor and clean roots",
        ));
    }
    Ok(report)
}

fn validate_report(report: &CompilerComponentReproducibilityV2) -> Result<(), ArtifactError> {
    let expected_total = EXPECTED_RUN_COUNT * EXPECTED_REPRESENTATION_COMPARISONS_PER_RUN;
    if report.protocol != COMPILER_COMPONENT_REPRODUCIBILITY_V2_PROTOCOL
        || report.binding_contract != COMPILER_COMPONENT_REPRODUCIBILITY_V2_BINDING_CONTRACT
        || report.authority != COMPILER_COMPONENT_REPRODUCIBILITY_V2_AUTHORITY
        || report.predecessor_protocol != COMPILER_COMPONENT_REPRODUCIBILITY_PROTOCOL
        || report.predecessor_file != COMPILER_COMPONENT_REPRODUCIBILITY_FILE
        || report.predecessor_bytes == 0
        || report.component_id.is_empty()
        || report.run_count != EXPECTED_RUN_COUNT
        || report.runs.len() != EXPECTED_RUN_COUNT
        || report.representation_comparison_count != expected_total
        || report.equivalent_representation_count != expected_total
        || !report.all_selected_representations_equivalent
        || !report.sidecars_individually_bound
        || !report.predecessor_signature_compatible
        || report.replacement_authorized
        || report.verdict != COMPILER_COMPONENT_REPRODUCIBILITY_V2_VERDICT
    {
        return Err(ArtifactError::new(
            "compiler reproducibility v2 aggregate contract mismatch",
        ));
    }
    for (label, hash) in [
        ("predecessor source", &report.predecessor_sha256),
        (
            "predecessor aggregate",
            &report.predecessor_aggregate_sha256,
        ),
        ("aggregate", &report.aggregate_sha256),
    ] {
        validate_sha256(hash, label)?;
    }
    let mut witnesses = BTreeSet::new();
    for (ordinal, run) in report.runs.iter().enumerate() {
        if run.ordinal != ordinal
            || run.run_id != format!("clean-build-{ordinal}")
            || run.representation_differential_file
                != COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_FILE
            || run.representation_differential_bytes == 0
            || run.representation_comparison_count != EXPECTED_REPRESENTATION_COMPARISONS_PER_RUN
            || run.representation_equivalent_count != EXPECTED_REPRESENTATION_COMPARISONS_PER_RUN
            || !run.all_representations_equivalent
            || run.replacement_authorized
        {
            return Err(ArtifactError::new(
                "compiler reproducibility v2 run contract mismatch",
            ));
        }
        for (label, hash) in [
            ("clean root witness", &run.clean_root_witness_sha256),
            ("production proof", &run.production_proof_sha256),
            (
                "base differential report",
                &run.base_differential_report_sha256,
            ),
            (
                "representation sidecar",
                &run.representation_differential_sha256,
            ),
            ("representation report", &run.representation_report_sha256),
        ] {
            validate_sha256(hash, label)?;
        }
        if !witnesses.insert(&run.clean_root_witness_sha256) {
            return Err(ArtifactError::new(
                "compiler reproducibility v2 clean root witnesses must be distinct",
            ));
        }
    }
    if report.aggregate_sha256 != reproducibility_v2_identity(report) {
        return Err(ArtifactError::new(
            "compiler reproducibility v2 aggregate identity drifted",
        ));
    }
    Ok(())
}

fn parse_run_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<CompilerComponentReproducibilityV2Run>, ArtifactError> {
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
        } else if in_block && !line.is_empty() && !line.starts_with('#') {
            let Some((key, value)) = line.split_once('=') else {
                return Err(ArtifactError::new(format!(
                    "`{}` contains malformed reproducibility v2 run line `{line}`",
                    path.display()
                )));
            };
            if values
                .insert(key.trim().to_owned(), value.trim().to_owned())
                .is_some()
            {
                return Err(ArtifactError::new(format!(
                    "`{}` reproducibility v2 run repeats a key",
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
) -> Result<CompilerComponentReproducibilityV2Run, ArtifactError> {
    let string = |key| parse_required_map_string_in_block(values, key, path, "run");
    let number = |key| {
        parse_optional_map_usize(values, key, path, "run")?.ok_or_else(|| {
            ArtifactError::new(format!("`{}` run is missing `{key}`", path.display()))
        })
    };
    Ok(CompilerComponentReproducibilityV2Run {
        ordinal: number("ordinal")?,
        run_id: string("run_id")?,
        clean_root_witness_sha256: string("clean_root_witness_sha256")?,
        production_proof_sha256: string("production_proof_sha256")?,
        base_differential_report_sha256: string("base_differential_report_sha256")?,
        representation_differential_file: string("representation_differential_file")?,
        representation_differential_bytes: number("representation_differential_bytes")?,
        representation_differential_sha256: string("representation_differential_sha256")?,
        representation_report_sha256: string("representation_report_sha256")?,
        representation_comparison_count: number("representation_comparison_count")?,
        representation_equivalent_count: number("representation_equivalent_count")?,
        all_representations_equivalent: parse_map_bool(
            values,
            "all_representations_equivalent",
            path,
        )?,
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

fn reproducibility_v2_identity(report: &CompilerComponentReproducibilityV2) -> String {
    let mut identity_record = report.clone();
    identity_record.aggregate_sha256.clear();
    sha256_hex(render_compiler_component_reproducibility_v2(&identity_record).as_bytes())
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
            "compiler reproducibility v2 {label} must be lowercase SHA-256"
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
            "compiler reproducibility v2 `{}` must be UTF-8/LF text",
            path.display()
        )));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
#[path = "compiler_component_reproducibility_v2_tests.rs"]
mod tests;
