use std::{fs, path::Path};

use crate::{
    build_compiler_candidate_fresh_source_result, ArtifactError,
    COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT, COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT_CONTRACT,
};

pub const COMPILER_CANDIDATE_NSLD_INPUT_PROTOCOL: &str = "nuis-compiler-candidate-nsld-input-v1";
pub const COMPILER_CANDIDATE_NSLD_INPUT_FILE: &str = "nuis.compiler-candidate-nsld-input.toml";
pub const COMPILER_CANDIDATE_NSLD_INPUT_CONTRACT: &str =
    "nuis-stage1-yir-to-nsld-materialization-input-v1";
pub const COMPILER_CANDIDATE_NSLD_TARGET_CONTRACT: &str = "nuis-registered-native-object-target-v1";
pub const COMPILER_CANDIDATE_NSLD_TARGET_SELECTOR: &str = "registered-native-cpu";
pub const COMPILER_CANDIDATE_NSLD_ENTRY_SYMBOL: &str = "Main.main";
pub const COMPILER_CANDIDATE_NSLD_FUNCTION_CONTRACT: &str = "nuis-yir-scalar-function-v1";
pub const COMPILER_CANDIDATE_NSLD_OPERATION_CONTRACT: &str = "nuis-yir-return-i64-v1";
pub const COMPILER_CANDIDATE_NSLD_TIME_CONTRACT: &str = "timestamped-partial-order";
pub const COMPILER_CANDIDATE_NSLD_GLM_CONTRACT: &str = "candidate-snapshot-no-owned-resource-v1";

const FOLD_MODULUS: usize = 2_147_483_629;
const EXPECTED_LINE_COUNT: usize = 31;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerCandidateNsldInput {
    pub protocol: String,
    pub input_contract: String,
    pub source_snapshot_contract: String,
    pub target_contract: String,
    pub target_selector: String,
    pub entry_symbol: String,
    pub function_contract: String,
    pub operation_contract: String,
    pub return_type: String,
    pub time_contract: String,
    pub glm_contract: String,
    pub source_bytes: usize,
    pub source_identity: usize,
    pub yir_identity: usize,
    pub unit_count: usize,
    pub function_count: usize,
    pub operation_count: usize,
    pub return_value: usize,
    pub dependency_count: usize,
    pub relocation_count: usize,
    pub time_ordinal: usize,
    pub glm_resource_count: usize,
    pub entry_symbol_identity: usize,
    pub materialization_fold: usize,
    pub candidate_owned_yir_materialization: bool,
    pub equivalent_nsld_input: bool,
    pub native_object: bool,
    pub stage0_handoff_required: bool,
    pub provider_dependency_required: bool,
    pub replacement_authorized: bool,
    pub selection_authorized: bool,
}

pub fn build_compiler_candidate_nsld_input(
    source: &[u8],
) -> Result<CompilerCandidateNsldInput, ArtifactError> {
    let fresh = build_compiler_candidate_fresh_source_result(source)?;
    let source_identity = fresh.stages[0].identity;
    let yir_identity = fresh.stages[4].identity;
    let entry_symbol_identity = fold_bytes(431, COMPILER_CANDIDATE_NSLD_ENTRY_SYMBOL.as_bytes());
    let values = [
        1,
        source.len(),
        source_identity % FOLD_MODULUS,
        yir_identity % FOLD_MODULUS,
        1,
        1,
        1,
        7,
        0,
        0,
        0,
        0,
        entry_symbol_identity,
    ];
    Ok(CompilerCandidateNsldInput {
        protocol: COMPILER_CANDIDATE_NSLD_INPUT_PROTOCOL.to_owned(),
        input_contract: COMPILER_CANDIDATE_NSLD_INPUT_CONTRACT.to_owned(),
        source_snapshot_contract: COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT_CONTRACT.to_owned(),
        target_contract: COMPILER_CANDIDATE_NSLD_TARGET_CONTRACT.to_owned(),
        target_selector: COMPILER_CANDIDATE_NSLD_TARGET_SELECTOR.to_owned(),
        entry_symbol: COMPILER_CANDIDATE_NSLD_ENTRY_SYMBOL.to_owned(),
        function_contract: COMPILER_CANDIDATE_NSLD_FUNCTION_CONTRACT.to_owned(),
        operation_contract: COMPILER_CANDIDATE_NSLD_OPERATION_CONTRACT.to_owned(),
        return_type: "i64".to_owned(),
        time_contract: COMPILER_CANDIDATE_NSLD_TIME_CONTRACT.to_owned(),
        glm_contract: COMPILER_CANDIDATE_NSLD_GLM_CONTRACT.to_owned(),
        source_bytes: source.len(),
        source_identity,
        yir_identity,
        unit_count: 1,
        function_count: 1,
        operation_count: 1,
        return_value: 7,
        dependency_count: 0,
        relocation_count: 0,
        time_ordinal: 0,
        glm_resource_count: 0,
        entry_symbol_identity,
        materialization_fold: fold_values(541, &values),
        candidate_owned_yir_materialization: true,
        equivalent_nsld_input: true,
        native_object: false,
        stage0_handoff_required: false,
        provider_dependency_required: false,
        replacement_authorized: false,
        selection_authorized: false,
    })
}

pub fn render_compiler_candidate_nsld_input(input: &CompilerCandidateNsldInput) -> String {
    format!(
        "protocol={}\ninput_contract={}\nsource_snapshot_contract={}\ntarget_contract={}\ntarget_selector={}\nentry_symbol={}\nfunction_contract={}\noperation_contract={}\nreturn_type={}\ntime_contract={}\nglm_contract={}\nsource_bytes={}\nsource_identity={}\nyir_identity={}\nunit_count={}\nfunction_count={}\noperation_count={}\nreturn_value={}\ndependency_count={}\nrelocation_count={}\ntime_ordinal={}\nglm_resource_count={}\nentry_symbol_identity={}\nmaterialization_fold={}\ncandidate_owned_yir_materialization={}\nequivalent_nsld_input={}\nnative_object={}\nstage0_handoff_required={}\nprovider_dependency_required={}\nreplacement_authorized={}\nselection_authorized={}\n",
        input.protocol,
        input.input_contract,
        input.source_snapshot_contract,
        input.target_contract,
        input.target_selector,
        input.entry_symbol,
        input.function_contract,
        input.operation_contract,
        input.return_type,
        input.time_contract,
        input.glm_contract,
        input.source_bytes,
        input.source_identity,
        input.yir_identity,
        input.unit_count,
        input.function_count,
        input.operation_count,
        input.return_value,
        input.dependency_count,
        input.relocation_count,
        input.time_ordinal,
        input.glm_resource_count,
        input.entry_symbol_identity,
        input.materialization_fold,
        input.candidate_owned_yir_materialization,
        input.equivalent_nsld_input,
        input.native_object,
        input.stage0_handoff_required,
        input.provider_dependency_required,
        input.replacement_authorized,
        input.selection_authorized,
    )
}

pub fn parse_compiler_candidate_nsld_input(
    path: &Path,
) -> Result<CompilerCandidateNsldInput, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler candidate Nsld input `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_candidate_nsld_input_from_source(&source, path)
}

pub fn parse_compiler_candidate_nsld_input_bytes(
    bytes: &[u8],
    path: &Path,
) -> Result<CompilerCandidateNsldInput, ArtifactError> {
    let source = std::str::from_utf8(bytes).map_err(|error| {
        ArtifactError::new(format!(
            "compiler candidate Nsld input `{}` is not UTF-8: {error}",
            path.display()
        ))
    })?;
    parse_compiler_candidate_nsld_input_from_source(source, path)
}

pub fn parse_compiler_candidate_nsld_input_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerCandidateNsldInput, ArtifactError> {
    if source.is_empty()
        || source.contains('\r')
        || source.contains('\0')
        || !source.ends_with('\n')
    {
        return Err(ArtifactError::new(format!(
            "compiler candidate Nsld input `{}` must use canonical UTF-8/LF text",
            path.display()
        )));
    }
    let lines = source.lines().collect::<Vec<_>>();
    if lines.len() != EXPECTED_LINE_COUNT {
        return Err(ArtifactError::new(format!(
            "compiler candidate Nsld input `{}` must contain {EXPECTED_LINE_COUNT} lines",
            path.display()
        )));
    }
    let mut line = 0;
    let mut text = |key: &str| {
        let value = parse_value(lines[line], key, path)?;
        line += 1;
        if value.is_empty() || value.chars().any(char::is_control) {
            return Err(ArtifactError::new(format!(
                "compiler candidate Nsld input `{}` has invalid text for `{key}`",
                path.display()
            )));
        }
        Ok(value.to_owned())
    };
    let protocol = text("protocol")?;
    let input_contract = text("input_contract")?;
    let source_snapshot_contract = text("source_snapshot_contract")?;
    let target_contract = text("target_contract")?;
    let target_selector = text("target_selector")?;
    let entry_symbol = text("entry_symbol")?;
    let function_contract = text("function_contract")?;
    let operation_contract = text("operation_contract")?;
    let return_type = text("return_type")?;
    let time_contract = text("time_contract")?;
    let glm_contract = text("glm_contract")?;
    drop(text);
    let mut number = |key: &str| {
        let value = parse_integer(parse_value(lines[line], key, path)?, path)?;
        line += 1;
        Ok(value)
    };
    let source_bytes = number("source_bytes")?;
    let source_identity = number("source_identity")?;
    let yir_identity = number("yir_identity")?;
    let unit_count = number("unit_count")?;
    let function_count = number("function_count")?;
    let operation_count = number("operation_count")?;
    let return_value = number("return_value")?;
    let dependency_count = number("dependency_count")?;
    let relocation_count = number("relocation_count")?;
    let time_ordinal = number("time_ordinal")?;
    let glm_resource_count = number("glm_resource_count")?;
    let entry_symbol_identity = number("entry_symbol_identity")?;
    let materialization_fold = number("materialization_fold")?;
    drop(number);
    let mut boolean = |key: &str| {
        let value = parse_bool(parse_value(lines[line], key, path)?, path)?;
        line += 1;
        Ok(value)
    };
    let input = CompilerCandidateNsldInput {
        protocol,
        input_contract,
        source_snapshot_contract,
        target_contract,
        target_selector,
        entry_symbol,
        function_contract,
        operation_contract,
        return_type,
        time_contract,
        glm_contract,
        source_bytes,
        source_identity,
        yir_identity,
        unit_count,
        function_count,
        operation_count,
        return_value,
        dependency_count,
        relocation_count,
        time_ordinal,
        glm_resource_count,
        entry_symbol_identity,
        materialization_fold,
        candidate_owned_yir_materialization: boolean("candidate_owned_yir_materialization")?,
        equivalent_nsld_input: boolean("equivalent_nsld_input")?,
        native_object: boolean("native_object")?,
        stage0_handoff_required: boolean("stage0_handoff_required")?,
        provider_dependency_required: boolean("provider_dependency_required")?,
        replacement_authorized: boolean("replacement_authorized")?,
        selection_authorized: boolean("selection_authorized")?,
    };
    let expected = build_compiler_candidate_nsld_input(COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT)?;
    if input != expected || render_compiler_candidate_nsld_input(&input) != source {
        return Err(ArtifactError::new(format!(
            "compiler candidate Nsld input `{}` changed its canonical materialization contract",
            path.display()
        )));
    }
    Ok(input)
}

fn fold_bytes(seed: usize, bytes: &[u8]) -> usize {
    bytes.iter().fold(seed, |state, byte| {
        ((state * 257) + usize::from(*byte) + 1) % FOLD_MODULUS
    })
}

fn fold_values(seed: usize, values: &[usize]) -> usize {
    values.iter().fold(seed, |state, value| {
        ((state * 65_537) + value + 1) % FOLD_MODULUS
    })
}

fn parse_value<'a>(line: &'a str, key: &str, path: &Path) -> Result<&'a str, ArtifactError> {
    let (actual, value) = line.split_once('=').ok_or_else(|| {
        ArtifactError::new(format!(
            "compiler candidate Nsld input `{}` contains malformed line `{line}`",
            path.display()
        ))
    })?;
    if actual != key {
        return Err(ArtifactError::new(format!(
            "compiler candidate Nsld input `{}` expected `{key}`",
            path.display()
        )));
    }
    Ok(value)
}

fn parse_integer(value: &str, path: &Path) -> Result<usize, ArtifactError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ArtifactError::new(format!(
            "compiler candidate Nsld input `{}` contains an invalid integer",
            path.display()
        )));
    }
    value.parse::<usize>().map_err(|error| {
        ArtifactError::new(format!(
            "compiler candidate Nsld input `{}` integer is invalid: {error}",
            path.display()
        ))
    })
}

fn parse_bool(value: &str, path: &Path) -> Result<bool, ArtifactError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(ArtifactError::new(format!(
            "compiler candidate Nsld input `{}` contains an invalid boolean",
            path.display()
        ))),
    }
}

#[cfg(test)]
#[path = "compiler_candidate_nsld_input_tests.rs"]
mod tests;
