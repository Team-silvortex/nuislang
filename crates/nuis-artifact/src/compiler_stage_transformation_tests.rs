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
        "nuis-stage1-nir-checkpoint-materializer-v7",
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
        transform_contract: COMPILER_STAGE_STRUCTURAL_CHECKPOINT_CONTRACT,
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
fn stage_transformation_builder_rejects_non_nuis_or_reordered_output() {
    let fixture = fixture();
    let mut drifted = fixture.nir_words;
    drifted[5] += 1;
    let records = [CompilerStageTransformationRecordInput {
        source_stage: CompilerStageKind::Nir,
        transform_contract: COMPILER_STAGE_STRUCTURAL_CHECKPOINT_CONTRACT,
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
    let verified = read_compiler_stage_transformations(&path, &fixture.handoff, &fixture.payloads)
        .expect("verify transformation manifest");
    assert_eq!(verified, manifest);

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
