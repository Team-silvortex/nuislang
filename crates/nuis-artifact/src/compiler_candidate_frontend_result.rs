use std::{fs, path::Path};

use sha2::{Digest, Sha256};

use crate::{
    compiler_candidate_bundle_fold, render_compiler_stage_transformations, ArtifactError,
    CompilerCandidateProduction, CompilerProjectionKind, CompilerStageKind,
    CompilerStageTransformations, COMPILER_STAGE_CHECKPOINT_PAGE_COUNT,
    COMPILER_STAGE_CHECKPOINT_WORD_COUNT,
};

pub const COMPILER_CANDIDATE_FRONTEND_RESULT_PROTOCOL: &str =
    "nuis-bootstrap-candidate-scalar-output-v9";
pub const COMPILER_CANDIDATE_FRONTEND_RESULT_FILE: &str =
    "nuis.compiler-candidate-front-end-result";
const EXPECTED_STAGE_COUNT: usize = 5;
const EXPECTED_LINE_COUNT: usize = 53;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerCandidateFrontendResult {
    pub protocol: String,
    pub stage_folds: Vec<usize>,
    pub bundle_fold: usize,
    pub token_record_count: usize,
    pub token_semantic_fold: usize,
    pub token_page_identity: usize,
    pub token_page_count: usize,
    pub token_terminal_page_hash: usize,
    pub token_page_chain_identity: usize,
    pub ast_checkpoint_words: Vec<usize>,
    pub nir_checkpoint_words: Vec<usize>,
}

pub fn build_compiler_candidate_frontend_result(
    production: &CompilerCandidateProduction,
    transformations: &CompilerStageTransformations,
) -> Result<CompilerCandidateFrontendResult, ArtifactError> {
    verify_transformation_binding(production, transformations)?;
    let stage_folds = production
        .records
        .iter()
        .enumerate()
        .map(|(ordinal, record)| {
            if record.ordinal != ordinal {
                return Err(ArtifactError::new(
                    "compiler candidate production records are not in canonical order",
                ));
            }
            Ok(record.fold)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let result = CompilerCandidateFrontendResult {
        protocol: COMPILER_CANDIDATE_FRONTEND_RESULT_PROTOCOL.to_owned(),
        stage_folds,
        bundle_fold: production.bundle_fold,
        token_record_count: production.token_record_count,
        token_semantic_fold: production.token_semantic_fold,
        token_page_identity: production.token_page_identity,
        token_page_count: production.token_page_count,
        token_terminal_page_hash: production.token_terminal_page_hash,
        token_page_chain_identity: production.token_page_chain_identity,
        ast_checkpoint_words: checkpoint_words(transformations, CompilerStageKind::Ast)?,
        nir_checkpoint_words: checkpoint_words(transformations, CompilerStageKind::Nir)?,
    };
    validate_result(&result)?;
    verify_production_summary(&result, production)?;
    Ok(result)
}

pub fn render_compiler_candidate_frontend_result(
    result: &CompilerCandidateFrontendResult,
) -> String {
    let mut out = format!("protocol={}\n", result.protocol);
    for (ordinal, fold) in result.stage_folds.iter().enumerate() {
        out.push_str(&format!("stage.{ordinal}={fold}\n"));
    }
    out.push_str(&format!(
        "bundle={}\ntokens.record_count={}\ntokens.semantic_fold={}\ntokens.page_identity={}\ntokens.page_count={}\ntokens.terminal_page_hash={}\ntokens.page_chain_identity={}\n",
        result.bundle_fold,
        result.token_record_count,
        result.token_semantic_fold,
        result.token_page_identity,
        result.token_page_count,
        result.token_terminal_page_hash,
        result.token_page_chain_identity,
    ));
    render_checkpoint(&mut out, "ast", &result.ast_checkpoint_words);
    render_checkpoint(&mut out, "nir", &result.nir_checkpoint_words);
    out
}

pub fn parse_compiler_candidate_frontend_result(
    path: &Path,
) -> Result<CompilerCandidateFrontendResult, ArtifactError> {
    let bytes = fs::read(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler candidate front-end result `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_candidate_frontend_result_bytes(&bytes, path)
}

pub fn parse_compiler_candidate_frontend_result_bytes(
    bytes: &[u8],
    path: &Path,
) -> Result<CompilerCandidateFrontendResult, ArtifactError> {
    let source = std::str::from_utf8(bytes).map_err(|error| {
        ArtifactError::new(format!(
            "compiler candidate front-end result `{}` is not UTF-8: {error}",
            path.display()
        ))
    })?;
    parse_compiler_candidate_frontend_result_from_source(source, path)
}

pub fn parse_compiler_candidate_frontend_result_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerCandidateFrontendResult, ArtifactError> {
    if source.is_empty()
        || source.contains('\r')
        || source.contains('\0')
        || !source.ends_with('\n')
    {
        return Err(ArtifactError::new(format!(
            "compiler candidate front-end result `{}` must use canonical UTF-8/LF text",
            path.display()
        )));
    }
    let lines = source.lines().collect::<Vec<_>>();
    if lines.len() != EXPECTED_LINE_COUNT {
        return Err(ArtifactError::new(format!(
            "compiler candidate front-end result `{}` must contain {EXPECTED_LINE_COUNT} lines",
            path.display()
        )));
    }
    let protocol = parse_text(lines[0], "protocol", path)?;
    let stage_folds = (0..EXPECTED_STAGE_COUNT)
        .map(|ordinal| parse_usize(lines[ordinal + 1], &format!("stage.{ordinal}"), path))
        .collect::<Result<Vec<_>, _>>()?;
    let mut ast = vec![0; COMPILER_STAGE_CHECKPOINT_WORD_COUNT];
    let mut nir = vec![0; COMPILER_STAGE_CHECKPOINT_WORD_COUNT];
    parse_checkpoint(&lines, 13, "ast", &mut ast, path)?;
    parse_checkpoint(&lines, 33, "nir", &mut nir, path)?;
    let result = CompilerCandidateFrontendResult {
        protocol,
        stage_folds,
        bundle_fold: parse_usize(lines[6], "bundle", path)?,
        token_record_count: parse_usize(lines[7], "tokens.record_count", path)?,
        token_semantic_fold: parse_usize(lines[8], "tokens.semantic_fold", path)?,
        token_page_identity: parse_usize(lines[9], "tokens.page_identity", path)?,
        token_page_count: parse_usize(lines[10], "tokens.page_count", path)?,
        token_terminal_page_hash: parse_usize(lines[11], "tokens.terminal_page_hash", path)?,
        token_page_chain_identity: parse_usize(lines[12], "tokens.page_chain_identity", path)?,
        ast_checkpoint_words: ast,
        nir_checkpoint_words: nir,
    };
    validate_result(&result)?;
    if render_compiler_candidate_frontend_result(&result) != source {
        return Err(ArtifactError::new(format!(
            "compiler candidate front-end result `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(result)
}

fn verify_transformation_binding(
    production: &CompilerCandidateProduction,
    transformations: &CompilerStageTransformations,
) -> Result<(), ArtifactError> {
    let source = render_compiler_stage_transformations(transformations);
    if source.len() != production.stage_transformations_bytes
        || sha256_hex(source.as_bytes()) != production.stage_transformations_sha256
    {
        return Err(ArtifactError::new(
            "compiler candidate front-end result transformation binding mismatch",
        ));
    }
    if transformations.producer_id != production.candidate_producer_id
        || transformations.stage_handoff_bundle_sha256 != production.stage_handoff_bundle_sha256
        || transformations.replacement_authorized
    {
        return Err(ArtifactError::new(
            "compiler candidate front-end result transformation lineage mismatch",
        ));
    }
    Ok(())
}

fn checkpoint_words(
    transformations: &CompilerStageTransformations,
    stage: CompilerStageKind,
) -> Result<Vec<usize>, ArtifactError> {
    let record = transformations
        .records
        .iter()
        .find(|record| record.source_stage == stage)
        .ok_or_else(|| ArtifactError::new("compiler candidate checkpoint stage is missing"))?;
    if record.output_words.len() != COMPILER_STAGE_CHECKPOINT_WORD_COUNT {
        return Err(ArtifactError::new(
            "compiler candidate checkpoint has a non-canonical word count",
        ));
    }
    Ok(record.output_words.clone())
}

fn verify_production_summary(
    result: &CompilerCandidateFrontendResult,
    production: &CompilerCandidateProduction,
) -> Result<(), ArtifactError> {
    let ast = &result.ast_checkpoint_words;
    let nir = &result.nir_checkpoint_words;
    if result.stage_folds.len() != production.records.len()
        || result.bundle_fold != production.bundle_fold
        || result.token_record_count != production.token_record_count
        || result.token_semantic_fold != production.token_semantic_fold
        || result.token_page_identity != production.token_page_identity
        || result.token_page_count != production.token_page_count
        || result.token_terminal_page_hash != production.token_terminal_page_hash
        || result.token_page_chain_identity != production.token_page_chain_identity
        || ast[2] != production.ast_page_identity
        || ast[3] != production.ast_page_cursor_identity
        || ast[12] != production.ast_continuation_page_identity
        || ast[13] != production.ast_continuation_cursor_identity
        || nir[2] != production.nir_page_identity
        || nir[3] != production.nir_page_cursor_identity
        || nir[12] != production.nir_continuation_page_identity
        || nir[13] != production.nir_continuation_cursor_identity
    {
        return Err(ArtifactError::new(
            "compiler candidate front-end result changed its production summary",
        ));
    }
    Ok(())
}

fn validate_result(result: &CompilerCandidateFrontendResult) -> Result<(), ArtifactError> {
    if result.protocol != COMPILER_CANDIDATE_FRONTEND_RESULT_PROTOCOL
        || result.stage_folds.len() != EXPECTED_STAGE_COUNT
        || result.stage_folds.contains(&0)
        || result.bundle_fold != compiler_candidate_bundle_fold(&result.stage_folds)
        || result.token_record_count == 0
        || result.token_semantic_fold == 0
        || result.token_page_identity == 0
        || result.token_page_count == 0
        || result.token_terminal_page_hash == 0
        || result.token_page_chain_identity == 0
    {
        return Err(ArtifactError::new(
            "compiler candidate front-end result summary is invalid",
        ));
    }
    validate_checkpoint(&result.ast_checkpoint_words, CompilerProjectionKind::Ast)?;
    validate_checkpoint(&result.nir_checkpoint_words, CompilerProjectionKind::Nir)
}

fn validate_checkpoint(words: &[usize], kind: CompilerProjectionKind) -> Result<(), ArtifactError> {
    let kind_tag = match kind {
        CompilerProjectionKind::Ast => 1,
        CompilerProjectionKind::Nir => 2,
    };
    if words.len() != COMPILER_STAGE_CHECKPOINT_WORD_COUNT
        || words[0] != kind_tag
        || words[1] != COMPILER_STAGE_CHECKPOINT_PAGE_COUNT
        || [words[2], words[3], words[12], words[13]].contains(&0)
    {
        return Err(ArtifactError::new(
            "compiler candidate front-end result checkpoint is invalid",
        ));
    }
    Ok(())
}

fn render_checkpoint(out: &mut String, prefix: &str, words: &[usize]) {
    out.push_str(&format!("{prefix}.page_identity={}\n", words[2]));
    out.push_str(&format!("{prefix}.page_cursor_identity={}\n", words[3]));
    out.push_str(&format!(
        "{prefix}.continuation_page_identity={}\n",
        words[12]
    ));
    out.push_str(&format!(
        "{prefix}.continuation_cursor_identity={}\n",
        words[13]
    ));
    for (index, value) in words[4..12].iter().enumerate() {
        out.push_str(&format!("{prefix}.first_cursor_lane.{index}={value}\n"));
    }
    for (index, value) in words[14..22].iter().enumerate() {
        out.push_str(&format!(
            "{prefix}.continuation_cursor_lane.{index}={value}\n"
        ));
    }
}

fn parse_checkpoint(
    lines: &[&str],
    start: usize,
    prefix: &str,
    words: &mut [usize],
    path: &Path,
) -> Result<(), ArtifactError> {
    words[0] = if prefix == "ast" { 1 } else { 2 };
    words[1] = COMPILER_STAGE_CHECKPOINT_PAGE_COUNT;
    words[2] = parse_usize(lines[start], &format!("{prefix}.page_identity"), path)?;
    words[3] = parse_usize(
        lines[start + 1],
        &format!("{prefix}.page_cursor_identity"),
        path,
    )?;
    words[12] = parse_usize(
        lines[start + 2],
        &format!("{prefix}.continuation_page_identity"),
        path,
    )?;
    words[13] = parse_usize(
        lines[start + 3],
        &format!("{prefix}.continuation_cursor_identity"),
        path,
    )?;
    for index in 0..8 {
        words[index + 4] = parse_usize(
            lines[start + 4 + index],
            &format!("{prefix}.first_cursor_lane.{index}"),
            path,
        )?;
        words[index + 14] = parse_usize(
            lines[start + 12 + index],
            &format!("{prefix}.continuation_cursor_lane.{index}"),
            path,
        )?;
    }
    Ok(())
}

fn parse_text(line: &str, key: &str, path: &Path) -> Result<String, ArtifactError> {
    let (actual, value) = split_line(line, path)?;
    if actual != key || value.is_empty() || value.chars().any(char::is_control) {
        return Err(ArtifactError::new(format!(
            "compiler candidate front-end result `{}` expected `{key}=<text>`",
            path.display()
        )));
    }
    Ok(value.to_owned())
}

fn parse_usize(line: &str, key: &str, path: &Path) -> Result<usize, ArtifactError> {
    let (actual, value) = split_line(line, path)?;
    if actual != key || value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ArtifactError::new(format!(
            "compiler candidate front-end result `{}` expected `{key}=<integer>`",
            path.display()
        )));
    }
    value.parse::<usize>().map_err(|error| {
        ArtifactError::new(format!(
            "compiler candidate front-end result `{}` key `{key}` is invalid: {error}",
            path.display()
        ))
    })
}

fn split_line<'a>(line: &'a str, path: &Path) -> Result<(&'a str, &'a str), ArtifactError> {
    line.split_once('=').ok_or_else(|| {
        ArtifactError::new(format!(
            "compiler candidate front-end result `{}` contains malformed line `{line}`",
            path.display()
        ))
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[cfg(test)]
#[path = "compiler_candidate_frontend_result_tests.rs"]
mod tests;
