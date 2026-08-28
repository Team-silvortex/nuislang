use super::*;

#[test]
fn adapter_output_parser_requires_exact_order_and_utf8_lf() {
    let source = b"protocol=nuis-bootstrap-candidate-scalar-output-v7\nstage.0=1\nstage.1=2\nstage.2=3\nstage.3=4\nstage.4=5\nbundle=6\ntokens.record_count=7\ntokens.semantic_fold=8\ntokens.page_identity=9\nast.page_identity=10\nast.page_cursor_identity=11\nast.continuation_page_identity=12\nast.continuation_cursor_identity=13\nnir.page_identity=14\nnir.page_cursor_identity=15\nnir.continuation_page_identity=16\nnir.continuation_cursor_identity=17\nnir.first_cursor_lane.0=18\nnir.first_cursor_lane.1=19\nnir.first_cursor_lane.2=20\nnir.first_cursor_lane.3=21\nnir.first_cursor_lane.4=22\nnir.first_cursor_lane.5=23\nnir.first_cursor_lane.6=24\nnir.first_cursor_lane.7=25\nnir.continuation_cursor_lane.0=26\nnir.continuation_cursor_lane.1=27\nnir.continuation_cursor_lane.2=28\nnir.continuation_cursor_lane.3=29\nnir.continuation_cursor_lane.4=30\nnir.continuation_cursor_lane.5=31\nnir.continuation_cursor_lane.6=32\nnir.continuation_cursor_lane.7=33\n";
    assert_eq!(
        parse_adapter_output(source).unwrap(),
        (
            vec![1, 2, 3, 4, 5],
            6,
            CompilerTokenDecodeSummary {
                record_count: 7,
                semantic_fold: 8,
            },
            9,
            AdapterProjectionOutput {
                first_page_identity: 10,
                first_cursor_identity: 11,
                continuation_page_identity: 12,
                continuation_cursor_identity: 13,
            },
            AdapterNirOutput {
                projection: AdapterProjectionOutput {
                    first_page_identity: 14,
                    first_cursor_identity: 15,
                    continuation_page_identity: 16,
                    continuation_cursor_identity: 17,
                },
                first_cursor_lanes: [18, 19, 20, 21, 22, 23, 24, 25],
                continuation_cursor_lanes: [26, 27, 28, 29, 30, 31, 32, 33],
            },
        )
    );

    let reordered = std::str::from_utf8(source)
        .unwrap()
        .replacen("stage.0=1", "stage.1=1", 1);
    assert!(parse_adapter_output(reordered.as_bytes()).is_err());
    assert!(parse_adapter_output(&source[..source.len() - 1]).is_err());
}
