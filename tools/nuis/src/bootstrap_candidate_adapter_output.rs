use nuis_artifact::{
    compiler_projection_checkpoint_kind_tag, CompilerProjectionKind,
    CompilerProjectionTwoPageIdentity, CompilerTokenDecodeSummary, CompilerTokenPaginationIdentity,
    COMPILER_STAGE_CHECKPOINT_PAGE_COUNT,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AdapterTokenPaginationOutput {
    pub(super) first_page_identity: usize,
    pub(super) page_count: usize,
    pub(super) terminal_page_hash: usize,
    pub(super) chain_identity: usize,
}

impl AdapterTokenPaginationOutput {
    pub(super) fn from_evidence(
        first_page_identity: usize,
        pagination: &CompilerTokenPaginationIdentity,
    ) -> Self {
        Self {
            first_page_identity,
            page_count: pagination.page_count,
            terminal_page_hash: pagination.terminal_page_hash,
            chain_identity: pagination.chain_identity,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AdapterProjectionCheckpointOutput {
    pub(super) projection: AdapterProjectionOutput,
    pub(super) first_cursor_lanes: [usize; 8],
    pub(super) continuation_cursor_lanes: [usize; 8],
}

impl AdapterProjectionCheckpointOutput {
    pub(super) fn from_pages(pages: CompilerProjectionTwoPageIdentity) -> Self {
        Self {
            projection: AdapterProjectionOutput::from_pages(pages),
            first_cursor_lanes: pages.first.cursor.lanes(),
            continuation_cursor_lanes: pages.second.cursor.lanes(),
        }
    }

    pub(super) fn checkpoint_words(self, kind: CompilerProjectionKind) -> Vec<usize> {
        let mut words = vec![
            compiler_projection_checkpoint_kind_tag(kind),
            COMPILER_STAGE_CHECKPOINT_PAGE_COUNT,
            self.projection.first_page_identity,
            self.projection.first_cursor_identity,
        ];
        words.extend(self.first_cursor_lanes);
        words.push(self.projection.continuation_page_identity);
        words.push(self.projection.continuation_cursor_identity);
        words.extend(self.continuation_cursor_lanes);
        words
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AdapterProjectionOutput {
    pub(super) first_page_identity: usize,
    pub(super) first_cursor_identity: usize,
    pub(super) continuation_page_identity: usize,
    pub(super) continuation_cursor_identity: usize,
}

impl AdapterProjectionOutput {
    pub(super) fn from_pages(pages: CompilerProjectionTwoPageIdentity) -> Self {
        Self {
            first_page_identity: pages.first.page.identity,
            first_cursor_identity: pages.first.cursor_identity,
            continuation_page_identity: pages.second.page.identity,
            continuation_cursor_identity: pages.second.cursor_identity,
        }
    }
}

pub(super) fn parse_adapter_output(
    bytes: &[u8],
    protocol: &str,
) -> Result<
    (
        Vec<usize>,
        usize,
        CompilerTokenDecodeSummary,
        AdapterTokenPaginationOutput,
        AdapterProjectionCheckpointOutput,
        AdapterProjectionCheckpointOutput,
    ),
    String,
> {
    let source = std::str::from_utf8(bytes)
        .map_err(|error| format!("candidate scalar output is not UTF-8: {error}"))?;
    if source.contains('\r') || source.contains('\0') || !source.ends_with('\n') {
        return Err("candidate scalar output must use canonical UTF-8/LF text".to_owned());
    }
    let lines = source.lines().collect::<Vec<_>>();
    if lines.len() != 53 || lines[0] != format!("protocol={protocol}") {
        return Err("candidate scalar output has an invalid protocol or line count".to_owned());
    }
    let stage_folds = (0..5)
        .map(|ordinal| parse_output_usize(lines[ordinal + 1], &format!("stage.{ordinal}")))
        .collect::<Result<Vec<_>, _>>()?;
    let bundle_fold = parse_output_usize(lines[6], "bundle")?;
    let token_decode = CompilerTokenDecodeSummary {
        record_count: parse_output_usize(lines[7], "tokens.record_count")?,
        semantic_fold: parse_output_usize(lines[8], "tokens.semantic_fold")?,
    };
    let token_pagination = AdapterTokenPaginationOutput {
        first_page_identity: parse_output_usize(lines[9], "tokens.page_identity")?,
        page_count: parse_output_usize(lines[10], "tokens.page_count")?,
        terminal_page_hash: parse_output_usize(lines[11], "tokens.terminal_page_hash")?,
        chain_identity: parse_output_usize(lines[12], "tokens.page_chain_identity")?,
    };
    let ast_output = AdapterProjectionCheckpointOutput {
        projection: AdapterProjectionOutput {
            first_page_identity: parse_output_usize(lines[13], "ast.page_identity")?,
            first_cursor_identity: parse_output_usize(lines[14], "ast.page_cursor_identity")?,
            continuation_page_identity: parse_output_usize(
                lines[15],
                "ast.continuation_page_identity",
            )?,
            continuation_cursor_identity: parse_output_usize(
                lines[16],
                "ast.continuation_cursor_identity",
            )?,
        },
        first_cursor_lanes: parse_output_lanes(&lines, 17, "ast.first_cursor_lane")?,
        continuation_cursor_lanes: parse_output_lanes(&lines, 25, "ast.continuation_cursor_lane")?,
    };
    let nir_output = AdapterProjectionCheckpointOutput {
        projection: AdapterProjectionOutput {
            first_page_identity: parse_output_usize(lines[33], "nir.page_identity")?,
            first_cursor_identity: parse_output_usize(lines[34], "nir.page_cursor_identity")?,
            continuation_page_identity: parse_output_usize(
                lines[35],
                "nir.continuation_page_identity",
            )?,
            continuation_cursor_identity: parse_output_usize(
                lines[36],
                "nir.continuation_cursor_identity",
            )?,
        },
        first_cursor_lanes: parse_output_lanes(&lines, 37, "nir.first_cursor_lane")?,
        continuation_cursor_lanes: parse_output_lanes(&lines, 45, "nir.continuation_cursor_lane")?,
    };
    Ok((
        stage_folds,
        bundle_fold,
        token_decode,
        token_pagination,
        ast_output,
        nir_output,
    ))
}

fn parse_output_lanes(
    lines: &[&str],
    start: usize,
    key_prefix: &str,
) -> Result<[usize; 8], String> {
    let mut lanes = [0; 8];
    for (index, lane) in lanes.iter_mut().enumerate() {
        *lane = parse_output_usize(lines[start + index], &format!("{key_prefix}.{index}"))?;
    }
    Ok(lanes)
}

fn parse_output_usize(line: &str, expected_key: &str) -> Result<usize, String> {
    let (key, value) = line
        .split_once('=')
        .ok_or_else(|| format!("candidate scalar output line `{line}` is malformed"))?;
    if key != expected_key || value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!(
            "candidate scalar output expected `{expected_key}=<integer>`, found `{line}`"
        ));
    }
    value
        .parse::<usize>()
        .map_err(|error| format!("candidate scalar output `{expected_key}` is invalid: {error}"))
}
