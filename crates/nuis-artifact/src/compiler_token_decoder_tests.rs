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

#[test]
fn first_page_identity_canonicalizes_a_real_candidate_prefix() {
    let source = b"nuis-token-stream-v1\nword\t757365\nword\t637075\nword\t5374644c616e6775616765436f7265\nsymbol\t59\narrow\n";
    let page = compiler_token_first_page_identity(source).expect("materialize first token page");
    assert_eq!(
        page,
        CompilerTokenPageIdentity {
            record_count: 4,
            payload_bytes: 21,
            canonical_bytes: 91,
            canonical_hash: 1_277_127_995,
            identity: 164_749_511_446,
        }
    );
}

#[test]
fn first_page_identity_reemits_numeric_payloads_and_enforces_capacity() {
    let source = b"nuis-token-stream-v1\ninteger\t00012\nsymbol\t00059\narrow\nstring\t\n";
    let page = compiler_token_first_page_identity(source).expect("canonicalize numeric page");
    let canonical = b"nuis-token-stream-v1\ninteger\t12\nsymbol\t59\narrow\nstring\t\n";
    let expected_hash = canonical
        .iter()
        .fold(COMPILER_TOKEN_DECODER_SEMANTIC_SEED, |state, byte| {
            fold_unit(state, usize::from(*byte) + 1)
        });
    assert_eq!(page.canonical_bytes, canonical.len());
    assert_eq!(page.canonical_hash, expected_hash);
    assert_eq!(page.payload_bytes, 0);

    assert!(compiler_token_first_page_identity(
        b"nuis-token-stream-v1\nword\t61\nword\t62\nword\t63\n"
    )
    .is_err());
    let oversized = format!(
        "nuis-token-stream-v1\nword\t{}\narrow\narrow\narrow\n",
        "61".repeat(COMPILER_TOKEN_PAGE_PAYLOAD_BYTES + 1)
    );
    assert!(compiler_token_first_page_identity(oversized.as_bytes()).is_err());
}
