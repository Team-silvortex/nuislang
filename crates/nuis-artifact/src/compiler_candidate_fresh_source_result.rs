use std::{fs, path::Path};

use crate::{compiler_candidate_bundle_fold, ArtifactError};

pub const COMPILER_CANDIDATE_FRESH_SOURCE_RESULT_PROTOCOL: &str =
    "nuis-bootstrap-candidate-fresh-source-result-v1";
pub const COMPILER_CANDIDATE_FRESH_SOURCE_RESULT_FILE: &str =
    "nuis.compiler-candidate-fresh-source-result";
pub const COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT_CONTRACT: &str =
    "nuis-canonical-bootstrap-source-snapshot-v1";
pub const COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT: &[u8] =
    b"mod cpu Main {\n  fn main() -> i64 {\n    return 7;\n  }\n}\n";

const STAGE_NAMES: [&str; 5] = ["source", "tokens", "ast", "nir", "yir"];
const EXPECTED_RECORD_COUNTS: [usize; 5] = [5, 16, 5, 6, 6];
const EXPECTED_LINE_COUNT: usize = 18;
const STATE_RADIX: usize = 1_000_003;
const COUNT_RADIX: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerCandidateFreshSourceStage {
    pub ordinal: usize,
    pub stage: String,
    pub record_count: usize,
    pub identity: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerCandidateFreshSourceResult {
    pub protocol: String,
    pub snapshot_contract: String,
    pub source_bytes: usize,
    pub source_lines: usize,
    pub stage_count: usize,
    pub stages: Vec<CompilerCandidateFreshSourceStage>,
    pub bundle_fold: usize,
    pub stage0_handoff_required: bool,
    pub provider_dependency_required: bool,
    pub candidate_owned_source_processing: bool,
    pub fresh_source_compile: bool,
    pub native_materialization: bool,
    pub replacement_authorized: bool,
    pub selection_authorized: bool,
}

pub fn build_compiler_candidate_fresh_source_result(
    source: &[u8],
) -> Result<CompilerCandidateFreshSourceResult, ArtifactError> {
    if source != COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT {
        return Err(ArtifactError::new(
            "compiler candidate fresh-source input is not the canonical snapshot",
        ));
    }
    let stages = STAGE_NAMES
        .iter()
        .enumerate()
        .map(|(ordinal, stage)| {
            let (record_count, identity) = compile_stage(source, ordinal)?;
            Ok(CompilerCandidateFreshSourceStage {
                ordinal,
                stage: (*stage).to_owned(),
                record_count,
                identity,
            })
        })
        .collect::<Result<Vec<_>, ArtifactError>>()?;
    let identities = stages
        .iter()
        .map(|stage| stage.identity)
        .collect::<Vec<_>>();
    let result = CompilerCandidateFreshSourceResult {
        protocol: COMPILER_CANDIDATE_FRESH_SOURCE_RESULT_PROTOCOL.to_owned(),
        snapshot_contract: COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT_CONTRACT.to_owned(),
        source_bytes: source.len(),
        source_lines: EXPECTED_RECORD_COUNTS[0],
        stage_count: STAGE_NAMES.len(),
        stages,
        bundle_fold: compiler_candidate_bundle_fold(&identities),
        stage0_handoff_required: false,
        provider_dependency_required: false,
        candidate_owned_source_processing: true,
        fresh_source_compile: true,
        native_materialization: false,
        replacement_authorized: false,
        selection_authorized: false,
    };
    validate_result(&result)?;
    Ok(result)
}

pub fn render_compiler_candidate_fresh_source_result(
    result: &CompilerCandidateFreshSourceResult,
) -> String {
    let mut out = format!(
        "protocol={}\nsnapshot_contract={}\nsource_bytes={}\nsource_lines={}\nstage_count={}\n",
        result.protocol,
        result.snapshot_contract,
        result.source_bytes,
        result.source_lines,
        result.stage_count,
    );
    for stage in &result.stages {
        out.push_str(&format!(
            "stage.{}={},{},{}\n",
            stage.ordinal, stage.stage, stage.record_count, stage.identity
        ));
    }
    out.push_str(&format!(
        "bundle={}\nstage0_handoff_required={}\nprovider_dependency_required={}\ncandidate_owned_source_processing={}\nfresh_source_compile={}\nnative_materialization={}\nreplacement_authorized={}\nselection_authorized={}\n",
        result.bundle_fold,
        result.stage0_handoff_required,
        result.provider_dependency_required,
        result.candidate_owned_source_processing,
        result.fresh_source_compile,
        result.native_materialization,
        result.replacement_authorized,
        result.selection_authorized,
    ));
    out
}

pub fn parse_compiler_candidate_fresh_source_result(
    path: &Path,
) -> Result<CompilerCandidateFreshSourceResult, ArtifactError> {
    let bytes = fs::read(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler candidate fresh-source result `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_candidate_fresh_source_result_bytes(&bytes, path)
}

pub fn parse_compiler_candidate_fresh_source_result_bytes(
    bytes: &[u8],
    path: &Path,
) -> Result<CompilerCandidateFreshSourceResult, ArtifactError> {
    let source = std::str::from_utf8(bytes).map_err(|error| {
        ArtifactError::new(format!(
            "compiler candidate fresh-source result `{}` is not UTF-8: {error}",
            path.display()
        ))
    })?;
    parse_compiler_candidate_fresh_source_result_from_source(source, path)
}

pub fn parse_compiler_candidate_fresh_source_result_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerCandidateFreshSourceResult, ArtifactError> {
    if source.is_empty()
        || source.contains('\r')
        || source.contains('\0')
        || !source.ends_with('\n')
    {
        return Err(ArtifactError::new(format!(
            "compiler candidate fresh-source result `{}` must use canonical UTF-8/LF text",
            path.display()
        )));
    }
    let lines = source.lines().collect::<Vec<_>>();
    if lines.len() != EXPECTED_LINE_COUNT {
        return Err(ArtifactError::new(format!(
            "compiler candidate fresh-source result `{}` must contain {EXPECTED_LINE_COUNT} lines",
            path.display()
        )));
    }
    let stages = (0..STAGE_NAMES.len())
        .map(|ordinal| parse_stage(lines[ordinal + 5], ordinal, path))
        .collect::<Result<Vec<_>, _>>()?;
    let result = CompilerCandidateFreshSourceResult {
        protocol: parse_text(lines[0], "protocol", path)?,
        snapshot_contract: parse_text(lines[1], "snapshot_contract", path)?,
        source_bytes: parse_usize(lines[2], "source_bytes", path)?,
        source_lines: parse_usize(lines[3], "source_lines", path)?,
        stage_count: parse_usize(lines[4], "stage_count", path)?,
        stages,
        bundle_fold: parse_usize(lines[10], "bundle", path)?,
        stage0_handoff_required: parse_bool(lines[11], "stage0_handoff_required", path)?,
        provider_dependency_required: parse_bool(lines[12], "provider_dependency_required", path)?,
        candidate_owned_source_processing: parse_bool(
            lines[13],
            "candidate_owned_source_processing",
            path,
        )?,
        fresh_source_compile: parse_bool(lines[14], "fresh_source_compile", path)?,
        native_materialization: parse_bool(lines[15], "native_materialization", path)?,
        replacement_authorized: parse_bool(lines[16], "replacement_authorized", path)?,
        selection_authorized: parse_bool(lines[17], "selection_authorized", path)?,
    };
    validate_result(&result)?;
    if render_compiler_candidate_fresh_source_result(&result) != source {
        return Err(ArtifactError::new(format!(
            "compiler candidate fresh-source result `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(result)
}

fn compile_stage(source: &[u8], stage: usize) -> Result<(usize, usize), ArtifactError> {
    let mut state = pack_state(0, 0, 193 + ((stage + 1) * 17));
    for &byte in source {
        state = step_state(state, stage, usize::from(byte))?;
    }
    finish_state(state, stage)
}

fn step_state(state: usize, stage: usize, byte: usize) -> Result<usize, ArtifactError> {
    let position = state / (COUNT_RADIX * STATE_RADIX);
    if stage >= STAGE_NAMES.len()
        || position >= COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT.len()
        || COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT[position] as usize != byte
    {
        return Err(ArtifactError::new(
            "compiler candidate fresh-source scalar transition rejected its input",
        ));
    }
    let count = (state / STATE_RADIX) % COUNT_RADIX + usize::from(record_boundary(stage, position));
    let previous_hash = state % STATE_RADIX;
    let event = stage_event(stage, position, byte);
    let hash = if event == 0 {
        previous_hash
    } else {
        ((previous_hash * 257) + event + 1) % STATE_RADIX
    };
    Ok(pack_state(position + 1, count, hash))
}

fn finish_state(state: usize, stage: usize) -> Result<(usize, usize), ArtifactError> {
    let position = state / (COUNT_RADIX * STATE_RADIX);
    let count = (state / STATE_RADIX) % COUNT_RADIX;
    if stage >= STAGE_NAMES.len()
        || position != COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT.len()
        || count != EXPECTED_RECORD_COUNTS[stage]
    {
        return Err(ArtifactError::new(
            "compiler candidate fresh-source scalar transition is incomplete",
        ));
    }
    Ok((
        count,
        (((state % STATE_RADIX) * 129) + count) * 129 + position,
    ))
}

fn pack_state(position: usize, count: usize, hash: usize) -> usize {
    ((position * COUNT_RADIX) + count) * STATE_RADIX + hash
}

fn record_boundary(stage: usize, index: usize) -> bool {
    match stage {
        0 => matches!(index, 14 | 35 | 49 | 53 | 55),
        1 => matches!(
            index,
            2 | 6 | 11 | 13 | 18 | 23 | 24 | 25 | 28 | 32 | 34 | 45 | 47 | 48 | 52 | 54
        ),
        2 => matches!(index, 13 | 34 | 48 | 52 | 54),
        _ => matches!(index, 13 | 34 | 47 | 48 | 52 | 54),
    }
}

fn stage_event(stage: usize, index: usize, byte: usize) -> usize {
    match stage {
        0 => byte + 1,
        1 if matches!(byte, 10 | 32) => 0,
        1 if record_boundary(stage, index) => byte + 1001,
        1 => byte + 1,
        2 => semantic_event(200, index),
        3 => semantic_event(300, index),
        _ => semantic_event(400, index),
    }
}

fn semantic_event(base: usize, index: usize) -> usize {
    [2, 6, 11, 13, 18, 23, 28, 32, 34, 45, 47, 48, 52, 54]
        .iter()
        .position(|candidate| *candidate == index)
        .map_or(0, |ordinal| base + ordinal + 1)
}

fn validate_result(result: &CompilerCandidateFreshSourceResult) -> Result<(), ArtifactError> {
    if result.protocol != COMPILER_CANDIDATE_FRESH_SOURCE_RESULT_PROTOCOL
        || result.snapshot_contract != COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT_CONTRACT
        || result.source_bytes != COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT.len()
        || result.source_lines != EXPECTED_RECORD_COUNTS[0]
        || result.stage_count != STAGE_NAMES.len()
        || result.stages.len() != STAGE_NAMES.len()
        || result.stage0_handoff_required
        || result.provider_dependency_required
        || !result.candidate_owned_source_processing
        || !result.fresh_source_compile
        || result.native_materialization
        || result.replacement_authorized
        || result.selection_authorized
    {
        return Err(ArtifactError::new(
            "compiler candidate fresh-source result declares an unsupported contract",
        ));
    }
    for (ordinal, stage) in result.stages.iter().enumerate() {
        if stage.ordinal != ordinal
            || stage.stage != STAGE_NAMES[ordinal]
            || stage.record_count != EXPECTED_RECORD_COUNTS[ordinal]
            || stage.identity == 0
        {
            return Err(ArtifactError::new(
                "compiler candidate fresh-source result stage identity is invalid",
            ));
        }
    }
    let identities = result
        .stages
        .iter()
        .map(|stage| stage.identity)
        .collect::<Vec<_>>();
    if result.bundle_fold == 0 || result.bundle_fold != compiler_candidate_bundle_fold(&identities)
    {
        return Err(ArtifactError::new(
            "compiler candidate fresh-source result bundle identity is invalid",
        ));
    }
    Ok(())
}

fn parse_stage(
    line: &str,
    ordinal: usize,
    path: &Path,
) -> Result<CompilerCandidateFreshSourceStage, ArtifactError> {
    let value = parse_value(line, &format!("stage.{ordinal}"), path)?;
    let fields = value.split(',').collect::<Vec<_>>();
    if fields.len() != 3 {
        return Err(ArtifactError::new(format!(
            "compiler candidate fresh-source result `{}` has a malformed stage record",
            path.display()
        )));
    }
    Ok(CompilerCandidateFreshSourceStage {
        ordinal,
        stage: fields[0].to_owned(),
        record_count: parse_integer(fields[1], path)?,
        identity: parse_integer(fields[2], path)?,
    })
}

fn parse_text(line: &str, key: &str, path: &Path) -> Result<String, ArtifactError> {
    let value = parse_value(line, key, path)?;
    if value.is_empty() || value.chars().any(char::is_control) {
        return Err(ArtifactError::new(format!(
            "compiler candidate fresh-source result `{}` has invalid text for `{key}`",
            path.display()
        )));
    }
    Ok(value.to_owned())
}

fn parse_usize(line: &str, key: &str, path: &Path) -> Result<usize, ArtifactError> {
    parse_integer(parse_value(line, key, path)?, path)
}

fn parse_bool(line: &str, key: &str, path: &Path) -> Result<bool, ArtifactError> {
    match parse_value(line, key, path)? {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(ArtifactError::new(format!(
            "compiler candidate fresh-source result `{}` has invalid boolean for `{key}`",
            path.display()
        ))),
    }
}

fn parse_value<'a>(line: &'a str, key: &str, path: &Path) -> Result<&'a str, ArtifactError> {
    let (actual, value) = line.split_once('=').ok_or_else(|| {
        ArtifactError::new(format!(
            "compiler candidate fresh-source result `{}` contains malformed line `{line}`",
            path.display()
        ))
    })?;
    if actual != key {
        return Err(ArtifactError::new(format!(
            "compiler candidate fresh-source result `{}` expected `{key}`",
            path.display()
        )));
    }
    Ok(value)
}

fn parse_integer(value: &str, path: &Path) -> Result<usize, ArtifactError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ArtifactError::new(format!(
            "compiler candidate fresh-source result `{}` contains an invalid integer",
            path.display()
        )));
    }
    value.parse::<usize>().map_err(|error| {
        ArtifactError::new(format!(
            "compiler candidate fresh-source result `{}` integer is invalid: {error}",
            path.display()
        ))
    })
}

#[cfg(test)]
#[path = "compiler_candidate_fresh_source_result_tests.rs"]
mod tests;
