use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    build_compiler_stage_handoff, CompilerStagePayloadInput, VerifiedCompilerStagePayload,
};

use super::*;

struct Fixture {
    handoff: CompilerStageHandoff,
    payloads: Vec<VerifiedCompilerStagePayload>,
    nir_words: [usize; COMPILER_STAGE_CHECKPOINT_WORD_COUNT],
}

fn fixture() -> Fixture {
    let source = b"mod cpu Main { fn main() -> i64 { return 0; } }\n";
    let tokens = b"nuis-token-stream-v1\nword\t757365\nword\t637075\nword\t5374644c616e6775616765436f7265\nsymbol\t59\narrow\n";
    let ast = concat!(
        "/// Transformation fixture with a continuation page.\n",
        "use cpu StdLanguageCore\n",
        "use cpu StdCompilerData\n",
        "ast mod cpu unit Main\n",
        "  fn main() -> i64\n",
        "    let value = 40\n",
        "    let delta = 2\n",
        "    return value + delta\n",
    )
    .as_bytes();
    let nir = concat!(
        "use cpu StdLanguageCore\n",
        "use cpu StdCompilerData\n",
        "use cpu StdCompilerTokenEmit\n",
        "use cpu StdCompilerTokens\n",
        "use cpu StdCompilerProjection\n",
        "nir mod cpu unit Main\n",
        "  fn main() -> i64\n",
        "    let value = 40\n",
        "    return value + 2\n",
    )
    .as_bytes();
    let yir = b"yir 0.1\nmodule cpu Main\n";
    let stages = [
        CompilerStageKind::Source,
        CompilerStageKind::Tokens,
        CompilerStageKind::Ast,
        CompilerStageKind::Nir,
        CompilerStageKind::Yir,
    ];
    let bytes = [source.as_slice(), tokens, ast, nir, yir];
    let files = [
        "main.source.ns",
        "main.tokens.txt",
        "main.ast.txt",
        "main.nir.txt",
        "main.yir",
    ];
    let inputs = stages
        .into_iter()
        .zip(files)
        .zip(bytes)
        .map(|((stage, payload_file), bytes)| CompilerStagePayloadInput {
            stage,
            payload_file,
            bytes,
        })
        .collect::<Vec<_>>();
    let handoff = build_compiler_stage_handoff(
        "nuis-stage1-compact-structured-nir-producer-v10",
        "cpu",
        "Main",
        &inputs,
    )
    .expect("build fixture handoff");
    let payloads = stages
        .into_iter()
        .zip(bytes)
        .map(|(stage, bytes)| VerifiedCompilerStagePayload {
            stage,
            bytes: bytes.to_vec(),
        })
        .collect::<Vec<_>>();
    let pages = compiler_projection_two_page_identity(CompilerProjectionKind::Nir, nir)
        .expect("materialize fixture NIR pages");
    Fixture {
        handoff,
        payloads,
        nir_words: compiler_stage_structural_checkpoint_words(CompilerProjectionKind::Nir, pages),
    }
}

fn build(fixture: &Fixture) -> CompilerStageTransformations {
    let records = [CompilerStageTransformationRecordInput {
        source_stage: CompilerStageKind::Nir,
        transform_contract: COMPILER_STAGE_STRUCTURED_RECORD_CONTRACT,
        output_encoding: COMPILER_STAGE_TRANSFORMATION_OUTPUT_ENCODING,
        output_words: &fixture.nir_words,
    }];
    build_compiler_stage_transformations(&CompilerStageTransformationsInput {
        producer_id: &fixture.handoff.producer_id,
        handoff: &fixture.handoff,
        payloads: &fixture.payloads,
        records: &records,
    })
    .expect("build stage transformations")
}

#[test]
fn structural_checkpoint_round_trips_and_preserves_ordered_cursor_words() {
    let fixture = fixture();
    let manifest = build(&fixture);
    assert_eq!(manifest.record_count, 1);
    assert!(!manifest.replacement_authorized);
    assert_eq!(manifest.records[0].source_stage, CompilerStageKind::Nir);
    assert_eq!(manifest.records[0].output_words, fixture.nir_words);
    assert_eq!(manifest.records[0].output_words[0], 2);
    assert_eq!(manifest.records[0].output_words[1], 2);
    assert_eq!(manifest.records[0].output_words.len(), 22);
    assert_eq!(
        manifest.records[0].output_payload_file,
        compiler_stage_transformation_payload_file(0)
    );
    let legacy_v2_bytes =
        8 + 3 * size_of::<u64>() + 22 * size_of::<u64>() + fixture.payloads[3].bytes.len();
    assert!(manifest.records[0].output_payload_bytes < legacy_v2_bytes);

    let derived = encode_compiler_stage_transformation_payload(
        CompilerStageKind::Nir,
        &fixture.payloads[3].bytes,
        &fixture.nir_words,
    )
    .expect("encode derived NIR payload");
    assert_ne!(derived, fixture.payloads[3].bytes);
    assert!(!derived
        .windows(fixture.payloads[3].bytes.len())
        .any(|window| window == fixture.payloads[3].bytes));
    let (checkpoint, recovered) =
        decode_compiler_stage_transformation_payload(CompilerStageKind::Nir, &derived)
            .expect("decode derived NIR payload");
    assert_eq!(checkpoint, fixture.nir_words);
    assert_eq!(recovered, fixture.payloads[3].bytes);

    let source = render_compiler_stage_transformations(&manifest);
    let parsed = parse_compiler_stage_transformations_from_source(
        &source,
        Path::new(COMPILER_STAGE_TRANSFORMATION_FILE),
    )
    .expect("parse stage transformations");
    assert_eq!(parsed, manifest);
    assert_eq!(render_compiler_stage_transformations(&parsed), source);
}

#[test]
fn compact_record_payload_rejects_noncanonical_varints_and_metadata_drift() {
    let fixture = fixture();
    let derived = encode_compiler_stage_transformation_payload(
        CompilerStageKind::Nir,
        &fixture.payloads[3].bytes,
        &fixture.nir_words,
    )
    .expect("encode compact records");

    let mut noncanonical = Vec::with_capacity(derived.len() + 1);
    noncanonical.extend_from_slice(&derived[..8]);
    noncanonical.extend_from_slice(&[0x82, 0x00]);
    noncanonical.extend_from_slice(&derived[9..]);
    let error = decode_compiler_stage_transformation_payload(CompilerStageKind::Nir, &noncanonical)
        .expect_err("noncanonical projection kind must fail");
    assert!(error.to_string().contains("not canonically encoded"));

    let mut impossible_count = derived.clone();
    let mut record_count_offset = 8;
    for _ in 0..3 {
        loop {
            let byte = impossible_count[record_count_offset];
            record_count_offset += 1;
            if byte & 0x80 == 0 {
                break;
            }
        }
    }
    assert_eq!(impossible_count[record_count_offset] & 0x80, 0);
    impossible_count[record_count_offset] = 0x7f;
    let error =
        decode_compiler_stage_transformation_payload(CompilerStageKind::Nir, &impossible_count)
            .expect_err("impossible record count must fail before allocation");
    assert!(error.to_string().contains("record count is invalid"));

    let mut trailing = derived;
    trailing.push(0);
    let error = decode_compiler_stage_transformation_payload(CompilerStageKind::Nir, &trailing)
        .expect_err("trailing payload bytes must fail");
    assert!(error.to_string().contains("length mismatch"));

    let mut metadata_drift = encode_compiler_stage_transformation_payload(
        CompilerStageKind::Nir,
        &fixture.payloads[3].bytes,
        &fixture.nir_words,
    )
    .expect("encode compact records for metadata drift");
    let mut cursor = 8;
    for _ in 0..(4 + COMPILER_STAGE_CHECKPOINT_WORD_COUNT) {
        loop {
            let byte = metadata_drift[cursor];
            cursor += 1;
            if byte & 0x80 == 0 {
                break;
            }
        }
    }
    assert_eq!(metadata_drift[cursor] & 0x0f, 1);
    metadata_drift[cursor] = (metadata_drift[cursor] & 0xf0) | 3;
    let error =
        decode_compiler_stage_transformation_payload(CompilerStageKind::Nir, &metadata_drift)
            .expect_err("valid but incorrect record kind must fail");
    assert!(error.to_string().contains("record metadata mismatch"));
}

#[test]
fn stage_transformation_builder_rejects_non_nuis_or_reordered_output() {
    let fixture = fixture();
    let mut drifted = fixture.nir_words;
    drifted[5] += 1;
    let records = [CompilerStageTransformationRecordInput {
        source_stage: CompilerStageKind::Nir,
        transform_contract: COMPILER_STAGE_STRUCTURED_RECORD_CONTRACT,
        output_encoding: COMPILER_STAGE_TRANSFORMATION_OUTPUT_ENCODING,
        output_words: &drifted,
    }];
    let error = build_compiler_stage_transformations(&CompilerStageTransformationsInput {
        producer_id: &fixture.handoff.producer_id,
        handoff: &fixture.handoff,
        payloads: &fixture.payloads,
        records: &records,
    })
    .expect_err("cursor lane drift must fail independent replay");
    assert!(error.to_string().contains("independent structural replay"));
}

#[test]
fn stage_transformation_reader_binds_payload_and_rejects_word_tampering() {
    let fixture = fixture();
    let manifest = build(&fixture);
    let source = render_compiler_stage_transformations(&manifest);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock follows epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("nuis_stage_transformations_{nonce}"));
    fs::create_dir_all(&root).expect("create transformation root");
    let path = root.join(COMPILER_STAGE_TRANSFORMATION_FILE);
    fs::write(&path, &source).expect("write transformation manifest");
    materialize_compiler_stage_transformation_payloads(
        &root,
        &manifest,
        &fixture.handoff,
        &fixture.payloads,
    )
    .expect("materialize derived stage payload");
    let verified = read_compiler_stage_transformations(&path, &fixture.handoff, &fixture.payloads)
        .expect("verify transformation manifest");
    assert_eq!(verified, manifest);

    let payload_path = root.join(&manifest.records[0].output_payload_file);
    let mut payload_bytes = fs::read(&payload_path).expect("read derived payload");
    payload_bytes[0] ^= 0xff;
    fs::write(&payload_path, payload_bytes).expect("tamper derived payload");
    let error = read_compiler_stage_transformations(&path, &fixture.handoff, &fixture.payloads)
        .expect_err("derived payload tampering must fail");
    assert!(error.to_string().contains("length or SHA-256 mismatch"));
    materialize_compiler_stage_transformation_payloads(
        &root,
        &manifest,
        &fixture.handoff,
        &fixture.payloads,
    )
    .expect("restore derived stage payload");

    let needle = format!("output_word_21 = {}", fixture.nir_words[21]);
    let tampered = source.replacen(
        &needle,
        &format!("output_word_21 = {}", fixture.nir_words[21] + 1),
        1,
    );
    fs::write(&path, tampered).expect("tamper transformation word");
    let error = read_compiler_stage_transformations(&path, &fixture.handoff, &fixture.payloads)
        .expect_err("transformation word tampering must fail");
    assert!(error.to_string().contains("record identity"));
    fs::remove_dir_all(root).expect("remove transformation root");
}
