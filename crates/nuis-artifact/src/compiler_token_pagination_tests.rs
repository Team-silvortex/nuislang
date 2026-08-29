use super::*;

fn long_token_stream() -> Vec<u8> {
    format!(
        "nuis-token-stream-v1\nword\t757365\nword\t637075\nword\t5374644c616e6775616765436f7265\nsymbol\t59\ndoc-comment\t{}\narrow\n",
        "61".repeat(180)
    )
    .into_bytes()
}

#[test]
fn pagination_covers_long_records_and_a_partial_tail() {
    let source = long_token_stream();
    let pagination = compiler_token_pagination_identity(&source).expect("paginate token stream");
    assert_eq!(pagination.record_count, 6);
    assert_eq!(
        pagination.page_count,
        source.len().div_ceil(COMPILER_TOKEN_PAGINATION_PAGE_BYTES)
    );
    assert!(pagination.page_count >= 4);
    assert_eq!(pagination.byte_count, source.len());
    assert_eq!(pagination.pages[0].byte_start, 0);
    assert_eq!(
        pagination.pages[0].byte_end,
        COMPILER_TOKEN_PAGINATION_PAGE_BYTES
    );
    assert_eq!(
        pagination.pages.last().expect("terminal page").record_end,
        pagination.record_count
    );
    assert!(pagination.pages.last().expect("terminal page").complete);
    assert!(pagination
        .pages
        .windows(2)
        .any(|pages| pages[0].record_end == pages[1].record_end));
    assert_eq!(
        pagination.terminal_page_hash,
        pagination.pages.last().expect("terminal page").page_hash
    );
}

#[test]
fn pagination_chain_is_replayable_and_binds_byte_ranges() {
    let source = long_token_stream();
    let pagination = compiler_token_pagination_identity(&source).expect("paginate token stream");
    let mut chain = COMPILER_TOKEN_PAGE_CHAIN_SEED;
    for page in &pagination.pages {
        chain = compiler_token_page_chain_fold(
            chain,
            page.ordinal,
            page.byte_start,
            page.byte_count,
            page.record_end,
            page.page_hash,
        );
        assert_eq!(chain, page.chain_identity);
    }
    assert_eq!(chain, pagination.chain_identity);

    let first = pagination.pages[0];
    let changed = compiler_token_page_chain_fold(
        COMPILER_TOKEN_PAGE_CHAIN_SEED,
        first.ordinal,
        first.byte_start + 1,
        first.byte_count,
        first.record_end,
        first.page_hash,
    );
    assert_ne!(changed, first.chain_identity);
}

#[test]
fn pagination_keeps_a_header_only_stream_as_one_complete_page() {
    let source = b"nuis-token-stream-v1\n";
    let pagination = compiler_token_pagination_identity(source).expect("paginate empty stream");
    assert_eq!(pagination.record_count, 0);
    assert_eq!(pagination.byte_count, source.len());
    assert_eq!(pagination.page_count, 1);
    assert_eq!(pagination.pages[0].record_end, 0);
    assert!(pagination.pages[0].complete);
    assert_ne!(pagination.chain_identity, COMPILER_TOKEN_PAGE_CHAIN_SEED);
}

#[test]
fn page_hash_is_independently_replayable() {
    let source = long_token_stream();
    let pagination = compiler_token_pagination_identity(&source).expect("paginate token stream");
    let expected = source[..COMPILER_TOKEN_PAGINATION_PAGE_BYTES]
        .iter()
        .fold(COMPILER_TOKEN_PAGE_HASH_SEED, |state, byte| {
            compiler_token_page_hash_step(state, *byte)
        });
    assert_eq!(pagination.first_page_hash, expected);
}
