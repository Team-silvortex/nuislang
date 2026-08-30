use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    build_compiler_stage_handoff, build_compiler_stage_semantic_differential,
    build_compiler_stage_transformations, compiler_projection_two_page_identity,
    compiler_stage_structural_checkpoint_words, materialize_compiler_stage_transformation_payloads,
    render_compiler_stage_semantic_differential, render_compiler_stage_transformations,
    CompilerProjectionKind, CompilerStagePayloadInput, CompilerStageSemanticDifferentialInput,
    CompilerStageTransformationRecordInput, CompilerStageTransformationsInput,
    COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_FILE, COMPILER_STAGE_STRUCTURED_RECORD_CONTRACT,
    COMPILER_STAGE_TRANSFORMATION_FILE, COMPILER_STAGE_TRANSFORMATION_OUTPUT_ENCODING,
};

use super::*;

struct Fixture {
    handoff: CompilerStageHandoff,
    payloads: Vec<VerifiedCompilerStagePayload>,
    transformations: CompilerStageTransformations,
    semantic_differential: CompilerStageSemanticDifferential,
}

fn fixture() -> Fixture {
    let source = b"mod cpu Main { fn main() -> i64 { return 42; } }\n";
    let tokens = b"nuis-token-stream-v1\nword\t6d6f64\nword\t637075\nword\t4d61696e\nsymbol\t123\n";
    let ast = concat!(
        "use cpu StdLanguageCore\n",
        "use cpu StdCompilerData\n",
        "ast mod cpu unit Main\n",
        "  fn main() -> i64\n",
        "    let value = 40\n",
        "    return value + 2\n",
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
    .expect("build handoff v2 fixture");
    let payloads = stages
        .into_iter()
        .zip(bytes)
        .map(|(stage, bytes)| VerifiedCompilerStagePayload {
            stage,
            bytes: bytes.to_vec(),
        })
        .collect::<Vec<_>>();
    let ast_pages = compiler_projection_two_page_identity(CompilerProjectionKind::Ast, ast)
        .expect("project fixture AST");
    let nir_pages = compiler_projection_two_page_identity(CompilerProjectionKind::Nir, nir)
        .expect("project fixture NIR");
    let ast_words =
        compiler_stage_structural_checkpoint_words(CompilerProjectionKind::Ast, ast_pages);
    let nir_words =
        compiler_stage_structural_checkpoint_words(CompilerProjectionKind::Nir, nir_pages);
    let records = [
        CompilerStageTransformationRecordInput {
            source_stage: CompilerStageKind::Ast,
            transform_contract: COMPILER_STAGE_STRUCTURED_RECORD_CONTRACT,
            output_encoding: COMPILER_STAGE_TRANSFORMATION_OUTPUT_ENCODING,
            output_words: &ast_words,
        },
        CompilerStageTransformationRecordInput {
            source_stage: CompilerStageKind::Nir,
            transform_contract: COMPILER_STAGE_STRUCTURED_RECORD_CONTRACT,
            output_encoding: COMPILER_STAGE_TRANSFORMATION_OUTPUT_ENCODING,
            output_words: &nir_words,
        },
    ];
    let transformations =
        build_compiler_stage_transformations(&CompilerStageTransformationsInput {
            producer_id: &handoff.producer_id,
            handoff: &handoff,
            payloads: &payloads,
            records: &records,
        })
        .expect("build registered fixture transformations");
    let semantic_differential =
        build_compiler_stage_semantic_differential(&CompilerStageSemanticDifferentialInput {
            producer_id: &handoff.producer_id,
            handoff: &handoff,
            payloads: &payloads,
            transformations: &transformations,
        })
        .expect("build fixture semantic differential");
    Fixture {
        handoff,
        payloads,
        transformations,
        semantic_differential,
    }
}

fn input(fixture: &Fixture) -> CompilerStageHandoffV2Input<'_> {
    CompilerStageHandoffV2Input {
        handoff: &fixture.handoff,
        payloads: &fixture.payloads,
        transformations: &fixture.transformations,
        semantic_differential: &fixture.semantic_differential,
    }
}

#[test]
fn selects_every_registered_stage_without_stage_specific_protocol_logic() {
    let fixture = fixture();
    let handoff =
        build_compiler_stage_handoff_v2(&input(&fixture)).expect("build compiler stage handoff v2");
    assert_eq!(handoff.selection_count, 2);
    assert!(handoff.all_reversible);
    assert!(!handoff.replacement_authorized);
    assert_eq!(handoff.selections[0].source_stage, CompilerStageKind::Ast);
    assert_eq!(handoff.selections[1].source_stage, CompilerStageKind::Nir);
    for selection in &handoff.selections {
        let source_record = fixture
            .handoff
            .records
            .iter()
            .find(|record| record.stage == selection.source_stage)
            .expect("selected source record");
        assert_eq!(selection.source_record_sha256, source_record.record_sha256);
        assert_eq!(
            selection.source_payload_sha256,
            selection.recovered_source_payload_sha256
        );
        assert!(selection.reversible);
        assert!(selection.semantically_equivalent);
    }
    assert_eq!(
        handoff
            .selection_for_stage(CompilerStageKind::Nir)
            .expect("NIR selection")
            .ordinal,
        1
    );

    let source = render_compiler_stage_handoff_v2(&handoff);
    let parsed = parse_compiler_stage_handoff_v2_from_source(
        &source,
        Path::new(COMPILER_STAGE_HANDOFF_V2_FILE),
    )
    .expect("parse compiler stage handoff v2");
    assert_eq!(parsed, handoff);
    assert_eq!(render_compiler_stage_handoff_v2(&parsed), source);
}

#[test]
fn reader_replays_all_sibling_evidence_and_rejects_tampering() {
    let fixture = fixture();
    let handoff =
        build_compiler_stage_handoff_v2(&input(&fixture)).expect("build compiler stage handoff v2");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock follows epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("nuis_stage_handoff_v2_{nonce}"));
    fs::create_dir_all(&root).expect("create handoff v2 root");
    fs::write(
        root.join(COMPILER_STAGE_TRANSFORMATION_FILE),
        render_compiler_stage_transformations(&fixture.transformations),
    )
    .expect("write fixture transformations");
    materialize_compiler_stage_transformation_payloads(
        &root,
        &fixture.transformations,
        &fixture.handoff,
        &fixture.payloads,
    )
    .expect("materialize fixture derived payloads");
    fs::write(
        root.join(COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_FILE),
        render_compiler_stage_semantic_differential(&fixture.semantic_differential),
    )
    .expect("write fixture semantic differential");
    let path = root.join(COMPILER_STAGE_HANDOFF_V2_FILE);
    let source = render_compiler_stage_handoff_v2(&handoff);
    fs::write(&path, &source).expect("write compiler stage handoff v2");
    let verified = read_compiler_stage_handoff_v2(&path, &fixture.handoff, &fixture.payloads)
        .expect("read compiler stage handoff v2");
    assert_eq!(verified, handoff);

    let tampered = source.replacen(
        "semantically_equivalent = true",
        "semantically_equivalent = false",
        1,
    );
    fs::write(&path, tampered).expect("tamper handoff v2 selection");
    let error = read_compiler_stage_handoff_v2(&path, &fixture.handoff, &fixture.payloads)
        .expect_err("selection tampering must fail");
    assert!(error.to_string().contains("selection 0 is invalid"));

    fs::write(&path, source).expect("restore compiler stage handoff v2");
    let transformations_path = root.join(COMPILER_STAGE_TRANSFORMATION_FILE);
    let transformations_source =
        fs::read_to_string(&transformations_path).expect("read fixture transformations");
    fs::write(
        &transformations_path,
        transformations_source.replacen(
            "replacement_authorized = false",
            "replacement_authorized = true",
            1,
        ),
    )
    .expect("tamper fixture transformations");
    let error = read_compiler_stage_handoff_v2(&path, &fixture.handoff, &fixture.payloads)
        .expect_err("sibling transformation tampering must fail");
    assert!(error
        .to_string()
        .contains("stage transformations length or SHA-256 mismatch"));
    fs::remove_dir_all(root).expect("remove handoff v2 root");
}
