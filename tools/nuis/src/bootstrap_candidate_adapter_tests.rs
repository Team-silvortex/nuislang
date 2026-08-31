use super::*;

#[test]
fn adapter_output_parser_requires_exact_order_and_utf8_lf() {
    let source = b"protocol=nuis-bootstrap-candidate-scalar-output-v9\nstage.0=1\nstage.1=2\nstage.2=3\nstage.3=4\nstage.4=5\nbundle=1693152009\ntokens.record_count=7\ntokens.semantic_fold=8\ntokens.page_identity=9\ntokens.page_count=10\ntokens.terminal_page_hash=11\ntokens.page_chain_identity=12\nast.page_identity=13\nast.page_cursor_identity=14\nast.continuation_page_identity=15\nast.continuation_cursor_identity=16\nast.first_cursor_lane.0=17\nast.first_cursor_lane.1=18\nast.first_cursor_lane.2=19\nast.first_cursor_lane.3=20\nast.first_cursor_lane.4=21\nast.first_cursor_lane.5=22\nast.first_cursor_lane.6=23\nast.first_cursor_lane.7=24\nast.continuation_cursor_lane.0=25\nast.continuation_cursor_lane.1=26\nast.continuation_cursor_lane.2=27\nast.continuation_cursor_lane.3=28\nast.continuation_cursor_lane.4=29\nast.continuation_cursor_lane.5=30\nast.continuation_cursor_lane.6=31\nast.continuation_cursor_lane.7=32\nnir.page_identity=33\nnir.page_cursor_identity=34\nnir.continuation_page_identity=35\nnir.continuation_cursor_identity=36\nnir.first_cursor_lane.0=37\nnir.first_cursor_lane.1=38\nnir.first_cursor_lane.2=39\nnir.first_cursor_lane.3=40\nnir.first_cursor_lane.4=41\nnir.first_cursor_lane.5=42\nnir.first_cursor_lane.6=43\nnir.first_cursor_lane.7=44\nnir.continuation_cursor_lane.0=45\nnir.continuation_cursor_lane.1=46\nnir.continuation_cursor_lane.2=47\nnir.continuation_cursor_lane.3=48\nnir.continuation_cursor_lane.4=49\nnir.continuation_cursor_lane.5=50\nnir.continuation_cursor_lane.6=51\nnir.continuation_cursor_lane.7=52\n";
    let result = parse_compiler_candidate_frontend_result_bytes(
        source,
        Path::new(COMPILER_CANDIDATE_FRONTEND_RESULT_FILE),
    )
    .unwrap();
    assert_eq!(result.stage_folds, [1, 2, 3, 4, 5]);
    assert_eq!(result.bundle_fold, 1_693_152_009);
    assert_eq!(result.token_record_count, 7);
    assert_eq!(
        result.ast_checkpoint_words[2..14],
        [13, 14, 17, 18, 19, 20, 21, 22, 23, 24, 15, 16]
    );
    assert_eq!(
        result.nir_checkpoint_words[14..22],
        [45, 46, 47, 48, 49, 50, 51, 52]
    );

    let reordered = std::str::from_utf8(source)
        .unwrap()
        .replacen("stage.0=1", "stage.1=1", 1);
    assert!(parse_compiler_candidate_frontend_result_bytes(
        reordered.as_bytes(),
        Path::new(COMPILER_CANDIDATE_FRONTEND_RESULT_FILE),
    )
    .is_err());
    assert!(parse_compiler_candidate_frontend_result_bytes(
        &source[..source.len() - 1],
        Path::new(COMPILER_CANDIDATE_FRONTEND_RESULT_FILE),
    )
    .is_err());
}

#[test]
fn adapter_compile_route_is_nuis_folded_and_never_uses_a_shell() {
    let source = render_adapter_source();
    for required in [
        "NUIS_BOOTSTRAP_STAGE0_PROVIDER_V1",
        "if (argc == 4) return run_compile_request(argv);",
        "nuis_bootstrap_candidate_stage_fold_v1(",
        "nuis_bootstrap_candidate_bundle_fold_v1(",
        "execl(provider, provider, NUIS_COMPILE_COMMAND",
        "candidate_compile_admission=nuis-owned-stage-fold-v1",
    ] {
        assert!(source.contains(required), "missing `{required}`");
    }
    assert!(!source.contains("sh -c"));
    assert!(!source.contains("system("));
}
