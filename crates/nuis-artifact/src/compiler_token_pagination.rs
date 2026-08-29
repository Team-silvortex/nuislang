use crate::{decode_compiler_token_stream, ArtifactError, COMPILER_TOKEN_DECODER_FOLD_MODULUS};

pub const COMPILER_TOKEN_PAGINATION_CONTRACT: &str = "nuis-compiler-token-pagination-v1";
pub const COMPILER_TOKEN_PAGINATION_PAGE_BYTES: usize = 128;
pub const COMPILER_TOKEN_PAGE_HASH_SEED: usize = 313;
pub const COMPILER_TOKEN_PAGE_HASH_RADIX: usize = 257;
pub const COMPILER_TOKEN_PAGE_CHAIN_SEED: usize = 419;
pub const COMPILER_TOKEN_PAGE_CHAIN_RADIX: usize = 65_537;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompilerTokenPaginationPage {
    pub ordinal: usize,
    pub byte_start: usize,
    pub byte_end: usize,
    pub record_end: usize,
    pub complete: bool,
    pub byte_count: usize,
    pub page_hash: usize,
    pub chain_identity: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerTokenPaginationIdentity {
    pub record_count: usize,
    pub byte_count: usize,
    pub page_count: usize,
    pub first_page_hash: usize,
    pub terminal_page_hash: usize,
    pub chain_identity: usize,
    pub pages: Vec<CompilerTokenPaginationPage>,
}

pub fn compiler_token_page_hash_step(state: usize, byte: u8) -> usize {
    (((state as u64 * COMPILER_TOKEN_PAGE_HASH_RADIX as u64) + u64::from(byte) + 1)
        % COMPILER_TOKEN_DECODER_FOLD_MODULUS as u64) as usize
}

pub fn compiler_token_page_chain_fold(
    state: usize,
    ordinal: usize,
    byte_start: usize,
    byte_count: usize,
    record_end: usize,
    page_hash: usize,
) -> usize {
    [ordinal, byte_start, byte_count, record_end, page_hash]
        .into_iter()
        .fold(state as u64, |fold, value| {
            ((fold * COMPILER_TOKEN_PAGE_CHAIN_RADIX as u64) + value as u64 + 1)
                % COMPILER_TOKEN_DECODER_FOLD_MODULUS as u64
        }) as usize
}

pub fn compiler_token_pagination_identity(
    bytes: &[u8],
) -> Result<CompilerTokenPaginationIdentity, ArtifactError> {
    let summary = decode_compiler_token_stream(bytes)?;
    let mut pages = Vec::with_capacity(bytes.len().div_ceil(COMPILER_TOKEN_PAGINATION_PAGE_BYTES));
    let mut chain_identity = COMPILER_TOKEN_PAGE_CHAIN_SEED;
    let mut newline_count = 0usize;

    for (ordinal, page_bytes) in bytes
        .chunks(COMPILER_TOKEN_PAGINATION_PAGE_BYTES)
        .enumerate()
    {
        let byte_start = ordinal * COMPILER_TOKEN_PAGINATION_PAGE_BYTES;
        let byte_end = byte_start + page_bytes.len();
        newline_count += page_bytes.iter().filter(|byte| **byte == b'\n').count();
        let record_end = newline_count.saturating_sub(1);
        let page_hash = page_bytes
            .iter()
            .fold(COMPILER_TOKEN_PAGE_HASH_SEED, |state, byte| {
                compiler_token_page_hash_step(state, *byte)
            });
        chain_identity = compiler_token_page_chain_fold(
            chain_identity,
            ordinal,
            byte_start,
            page_bytes.len(),
            record_end,
            page_hash,
        );
        pages.push(CompilerTokenPaginationPage {
            ordinal,
            byte_start,
            byte_end,
            record_end,
            complete: byte_end == bytes.len(),
            byte_count: page_bytes.len(),
            page_hash,
            chain_identity,
        });
    }
    if pages.is_empty() || pages.last().map(|page| page.record_end) != Some(summary.record_count) {
        return Err(ArtifactError::new(
            "compiler token pagination did not cover the complete decoded stream",
        ));
    }

    let first_page_hash = pages.first().map_or(0, |page| page.page_hash);
    let terminal_page_hash = pages.last().map_or(0, |page| page.page_hash);
    Ok(CompilerTokenPaginationIdentity {
        record_count: summary.record_count,
        byte_count: bytes.len(),
        page_count: pages.len(),
        first_page_hash,
        terminal_page_hash,
        chain_identity,
        pages,
    })
}

#[cfg(test)]
#[path = "compiler_token_pagination_tests.rs"]
mod tests;
