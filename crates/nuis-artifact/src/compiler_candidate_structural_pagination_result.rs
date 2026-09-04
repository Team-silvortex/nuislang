use std::path::Path;

use crate::{ArtifactError, CompilerProjectionKind, COMPILER_PROJECTION_CURSOR_LANES};

pub const COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_PROTOCOL: &str =
    "nuis-bootstrap-candidate-structural-pagination-v1";
pub const COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_FILE: &str =
    "nuis.compiler-candidate-structural-pagination-result";
pub const COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_PAGE_COUNT: usize = 3;

const PROJECTION_COUNT: usize = 2;
const PAGE_LINE_COUNT: usize = 2 + COMPILER_PROJECTION_CURSOR_LANES;
const EXPECTED_LINE_COUNT: usize =
    2 + PROJECTION_COUNT * COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_PAGE_COUNT * PAGE_LINE_COUNT;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerCandidateStructuralPaginationPage {
    pub ordinal: usize,
    pub identity: usize,
    pub cursor_identity: usize,
    pub cursor_lanes: [usize; COMPILER_PROJECTION_CURSOR_LANES],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerCandidateStructuralPaginationResult {
    pub protocol: String,
    pub page_count: usize,
    pub ast_pages: Vec<CompilerCandidateStructuralPaginationPage>,
    pub nir_pages: Vec<CompilerCandidateStructuralPaginationPage>,
}

pub fn render_compiler_candidate_structural_pagination_result(
    result: &CompilerCandidateStructuralPaginationResult,
) -> String {
    let mut out = format!(
        "protocol={}\npage_count={}\n",
        result.protocol, result.page_count
    );
    render_pages(&mut out, "ast", &result.ast_pages);
    render_pages(&mut out, "nir", &result.nir_pages);
    out
}

pub fn parse_compiler_candidate_structural_pagination_result_bytes(
    bytes: &[u8],
    path: &Path,
) -> Result<CompilerCandidateStructuralPaginationResult, ArtifactError> {
    let source = std::str::from_utf8(bytes).map_err(|error| {
        ArtifactError::new(format!(
            "compiler candidate structural pagination result `{}` is not UTF-8: {error}",
            path.display()
        ))
    })?;
    parse_compiler_candidate_structural_pagination_result_from_source(source, path)
}

pub fn parse_compiler_candidate_structural_pagination_result_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerCandidateStructuralPaginationResult, ArtifactError> {
    if source.is_empty()
        || source.contains('\r')
        || source.contains('\0')
        || !source.ends_with('\n')
    {
        return Err(ArtifactError::new(format!(
            "compiler candidate structural pagination result `{}` must use canonical UTF-8/LF text",
            path.display()
        )));
    }
    let lines = source.lines().collect::<Vec<_>>();
    if lines.len() != EXPECTED_LINE_COUNT {
        return Err(ArtifactError::new(format!(
            "compiler candidate structural pagination result `{}` must contain {EXPECTED_LINE_COUNT} lines",
            path.display()
        )));
    }
    let result = CompilerCandidateStructuralPaginationResult {
        protocol: parse_text(lines[0], "protocol", path)?,
        page_count: parse_usize(lines[1], "page_count", path)?,
        ast_pages: parse_pages(&lines, 2, "ast", path)?,
        nir_pages: parse_pages(
            &lines,
            2 + COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_PAGE_COUNT * PAGE_LINE_COUNT,
            "nir",
            path,
        )?,
    };
    validate_result(&result)?;
    if render_compiler_candidate_structural_pagination_result(&result) != source {
        return Err(ArtifactError::new(format!(
            "compiler candidate structural pagination result `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(result)
}

fn render_pages(
    out: &mut String,
    prefix: &str,
    pages: &[CompilerCandidateStructuralPaginationPage],
) {
    for page in pages {
        out.push_str(&format!(
            "{prefix}.page.{}.identity={}\n{prefix}.page.{}.cursor_identity={}\n",
            page.ordinal, page.identity, page.ordinal, page.cursor_identity
        ));
        for (lane, value) in page.cursor_lanes.iter().enumerate() {
            out.push_str(&format!(
                "{prefix}.page.{}.cursor_lane.{lane}={value}\n",
                page.ordinal
            ));
        }
    }
}

fn parse_pages(
    lines: &[&str],
    start: usize,
    prefix: &str,
    path: &Path,
) -> Result<Vec<CompilerCandidateStructuralPaginationPage>, ArtifactError> {
    (0..COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_PAGE_COUNT)
        .map(|ordinal| {
            let offset = start + ordinal * PAGE_LINE_COUNT;
            let mut cursor_lanes = [0; COMPILER_PROJECTION_CURSOR_LANES];
            for (lane, value) in cursor_lanes.iter_mut().enumerate() {
                *value = parse_usize(
                    lines[offset + 2 + lane],
                    &format!("{prefix}.page.{ordinal}.cursor_lane.{lane}"),
                    path,
                )?;
            }
            Ok(CompilerCandidateStructuralPaginationPage {
                ordinal,
                identity: parse_usize(
                    lines[offset],
                    &format!("{prefix}.page.{ordinal}.identity"),
                    path,
                )?,
                cursor_identity: parse_usize(
                    lines[offset + 1],
                    &format!("{prefix}.page.{ordinal}.cursor_identity"),
                    path,
                )?,
                cursor_lanes,
            })
        })
        .collect()
}

fn validate_result(
    result: &CompilerCandidateStructuralPaginationResult,
) -> Result<(), ArtifactError> {
    if result.protocol != COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_PROTOCOL
        || result.page_count != COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_PAGE_COUNT
        || result.ast_pages.len() != result.page_count
        || result.nir_pages.len() != result.page_count
    {
        return Err(ArtifactError::new(
            "compiler candidate structural pagination result contract mismatch",
        ));
    }
    for (kind, pages) in [
        (CompilerProjectionKind::Ast, &result.ast_pages),
        (CompilerProjectionKind::Nir, &result.nir_pages),
    ] {
        for (ordinal, page) in pages.iter().enumerate() {
            if page.ordinal != ordinal || page.identity == 0 || page.cursor_identity == 0 {
                return Err(ArtifactError::new(format!(
                    "compiler candidate {} structural pagination page is invalid",
                    kind.as_str()
                )));
            }
        }
    }
    Ok(())
}

fn parse_text(line: &str, key: &str, path: &Path) -> Result<String, ArtifactError> {
    let (actual, value) = split_line(line, path)?;
    if actual != key || value.is_empty() || value.chars().any(char::is_control) {
        return Err(expected_value_error(path, key, "text"));
    }
    Ok(value.to_owned())
}

fn parse_usize(line: &str, key: &str, path: &Path) -> Result<usize, ArtifactError> {
    let (actual, value) = split_line(line, path)?;
    if actual != key || value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(expected_value_error(path, key, "integer"));
    }
    value.parse::<usize>().map_err(|error| {
        ArtifactError::new(format!(
            "compiler candidate structural pagination result `{}` key `{key}` is invalid: {error}",
            path.display()
        ))
    })
}

fn split_line<'a>(line: &'a str, path: &Path) -> Result<(&'a str, &'a str), ArtifactError> {
    line.split_once('=').ok_or_else(|| {
        ArtifactError::new(format!(
            "compiler candidate structural pagination result `{}` has malformed line `{line}`",
            path.display()
        ))
    })
}

fn expected_value_error(path: &Path, key: &str, kind: &str) -> ArtifactError {
    ArtifactError::new(format!(
        "compiler candidate structural pagination result `{}` expected `{key}=<{kind}>`",
        path.display()
    ))
}

#[cfg(test)]
#[path = "compiler_candidate_structural_pagination_result_tests.rs"]
mod tests;
