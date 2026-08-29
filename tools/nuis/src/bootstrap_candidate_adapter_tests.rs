use super::*;

#[test]
fn adapter_output_parser_requires_exact_order_and_utf8_lf() {
    let source = b"protocol=nuis-bootstrap-candidate-scalar-output-v8\nstage.0=1\nstage.1=2\nstage.2=3\nstage.3=4\nstage.4=5\nbundle=6\ntokens.record_count=7\ntokens.semantic_fold=8\ntokens.page_identity=9\ntokens.page_count=10\ntokens.terminal_page_hash=11\ntokens.page_chain_identity=12\nast.page_identity=13\nast.page_cursor_identity=14\nast.continuation_page_identity=15\nast.continuation_cursor_identity=16\nnir.page_identity=17\nnir.page_cursor_identity=18\nnir.continuation_page_identity=19\nnir.continuation_cursor_identity=20\nnir.first_cursor_lane.0=21\nnir.first_cursor_lane.1=22\nnir.first_cursor_lane.2=23\nnir.first_cursor_lane.3=24\nnir.first_cursor_lane.4=25\nnir.first_cursor_lane.5=26\nnir.first_cursor_lane.6=27\nnir.first_cursor_lane.7=28\nnir.continuation_cursor_lane.0=29\nnir.continuation_cursor_lane.1=30\nnir.continuation_cursor_lane.2=31\nnir.continuation_cursor_lane.3=32\nnir.continuation_cursor_lane.4=33\nnir.continuation_cursor_lane.5=34\nnir.continuation_cursor_lane.6=35\nnir.continuation_cursor_lane.7=36\n";
    assert_eq!(
        parse_adapter_output(source, ADAPTER_OUTPUT_PROTOCOL).unwrap(),
        (
            vec![1, 2, 3, 4, 5],
            6,
            CompilerTokenDecodeSummary {
                record_count: 7,
                semantic_fold: 8,
            },
            AdapterTokenPaginationOutput {
                first_page_identity: 9,
                page_count: 10,
                terminal_page_hash: 11,
                chain_identity: 12,
            },
            AdapterProjectionOutput {
                first_page_identity: 13,
                first_cursor_identity: 14,
                continuation_page_identity: 15,
                continuation_cursor_identity: 16,
            },
            AdapterNirOutput {
                projection: AdapterProjectionOutput {
                    first_page_identity: 17,
                    first_cursor_identity: 18,
                    continuation_page_identity: 19,
                    continuation_cursor_identity: 20,
                },
                first_cursor_lanes: [21, 22, 23, 24, 25, 26, 27, 28],
                continuation_cursor_lanes: [29, 30, 31, 32, 33, 34, 35, 36],
            },
        )
    );

    let reordered = std::str::from_utf8(source)
        .unwrap()
        .replacen("stage.0=1", "stage.1=1", 1);
    assert!(parse_adapter_output(reordered.as_bytes(), ADAPTER_OUTPUT_PROTOCOL).is_err());
    assert!(parse_adapter_output(&source[..source.len() - 1], ADAPTER_OUTPUT_PROTOCOL).is_err());
}
