use super::*;

#[test]
fn decoder_covers_all_canonical_token_record_kinds() {
    let source = b"nuis-token-stream-v1\nword\t6d6f64\ninteger\t-12\nfloat\t312e35\nsymbol\t123\narrow\nstring\t\ndoc-comment\t6869\n";
    let summary = decode_compiler_token_stream(source).expect("decode canonical token stream");
    assert_eq!(summary.record_count, 7);
    assert_eq!(summary.semantic_fold, 1_001_345_261);
}

#[test]
fn decoder_rejects_noncanonical_or_malformed_payloads() {
    for source in [
        b"wrong-token-stream\n".as_slice(),
        b"nuis-token-stream-v1\r\n".as_slice(),
        b"nuis-token-stream-v1\nword\t\n".as_slice(),
        b"nuis-token-stream-v1\nword\t6D\n".as_slice(),
        b"nuis-token-stream-v1\nword\t6\n".as_slice(),
        b"nuis-token-stream-v1\ninteger\t-\n".as_slice(),
        b"nuis-token-stream-v1\nfloat\t\n".as_slice(),
        b"nuis-token-stream-v1\nsymbol\t\n".as_slice(),
        b"nuis-token-stream-v1\narrow\t\n".as_slice(),
        b"nuis-token-stream-v1\nstring\tf\n".as_slice(),
        b"nuis-token-stream-v1\ndoc-comment\t0G\n".as_slice(),
        b"nuis-token-stream-v1\nsymbol\t12x\n".as_slice(),
    ] {
        assert!(decode_compiler_token_stream(source).is_err(), "{source:?}");
    }
    assert!(decode_compiler_token_stream(b"nuis-token-stream-v1").is_err());
    assert!(decode_compiler_token_stream(&[0xff]).is_err());
}

#[test]
fn decoder_accepts_an_empty_record_sequence_but_not_an_empty_payload() {
    let summary = decode_compiler_token_stream(b"nuis-token-stream-v1\n")
        .expect("header-only stream is canonical");
    assert_eq!(summary.record_count, 0);
    assert_eq!(summary.semantic_fold, COMPILER_TOKEN_DECODER_SEMANTIC_SEED);
    assert!(decode_compiler_token_stream(b"").is_err());
}

#[test]
fn decoder_enforces_byte_and_record_bounds_without_an_off_by_one() {
    let mut source = String::from("nuis-token-stream-v1\n");
    source.push_str(&"arrow\n".repeat(COMPILER_TOKEN_DECODER_MAX_RECORDS));
    let summary = decode_compiler_token_stream(source.as_bytes())
        .expect("the maximum record count remains admissible");
    assert_eq!(summary.record_count, COMPILER_TOKEN_DECODER_MAX_RECORDS);

    source.push_str("arrow\n");
    assert!(decode_compiler_token_stream(source.as_bytes()).is_err());
    assert!(
        decode_compiler_token_stream(&vec![b'x'; COMPILER_TOKEN_DECODER_MAX_BYTES + 1]).is_err()
    );
}
