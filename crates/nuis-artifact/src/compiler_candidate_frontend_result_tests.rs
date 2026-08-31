use super::*;

fn fixture() -> CompilerCandidateFrontendResult {
    let stage_folds = vec![11, 12, 13, 14, 15];
    let mut ast = (0..COMPILER_STAGE_CHECKPOINT_WORD_COUNT)
        .map(|index| index + 20)
        .collect::<Vec<_>>();
    let mut nir = (0..COMPILER_STAGE_CHECKPOINT_WORD_COUNT)
        .map(|index| index + 50)
        .collect::<Vec<_>>();
    ast[0] = 1;
    ast[1] = COMPILER_STAGE_CHECKPOINT_PAGE_COUNT;
    nir[0] = 2;
    nir[1] = COMPILER_STAGE_CHECKPOINT_PAGE_COUNT;
    CompilerCandidateFrontendResult {
        protocol: COMPILER_CANDIDATE_FRONTEND_RESULT_PROTOCOL.to_owned(),
        bundle_fold: compiler_candidate_bundle_fold(&stage_folds),
        stage_folds,
        token_record_count: 4,
        token_semantic_fold: 101,
        token_page_identity: 102,
        token_page_count: 2,
        token_terminal_page_hash: 103,
        token_page_chain_identity: 104,
        ast_checkpoint_words: ast,
        nir_checkpoint_words: nir,
    }
}

#[test]
fn candidate_frontend_result_roundtrips_canonically() {
    let result = fixture();
    validate_result(&result).expect("valid front-end result");
    let source = render_compiler_candidate_frontend_result(&result);
    let parsed = parse_compiler_candidate_frontend_result_from_source(
        &source,
        Path::new(COMPILER_CANDIDATE_FRONTEND_RESULT_FILE),
    )
    .expect("parse canonical front-end result");
    assert_eq!(parsed, result);
    assert_eq!(source.lines().count(), EXPECTED_LINE_COUNT);
}

#[test]
fn candidate_frontend_result_tampering_fails_closed() {
    let source = render_compiler_candidate_frontend_result(&fixture());
    for tampered in [
        source.replacen("protocol=", "protocol=unsupported-", 1),
        source.replacen("ast.page_identity=22", "ast.page_identity=0", 1),
        source.replacen("stage.0=11", "stage.0=-11", 1),
        source.trim_end().to_owned(),
    ] {
        assert!(parse_compiler_candidate_frontend_result_from_source(
            &tampered,
            Path::new(COMPILER_CANDIDATE_FRONTEND_RESULT_FILE),
        )
        .is_err());
    }
}
