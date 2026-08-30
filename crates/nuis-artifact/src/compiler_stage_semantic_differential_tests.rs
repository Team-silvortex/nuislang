use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    build_compiler_stage_handoff, build_compiler_stage_transformations,
    compiler_projection_two_page_identity, compiler_stage_structural_checkpoint_words,
    materialize_compiler_stage_transformation_payloads, CompilerProjectionKind,
    CompilerStagePayloadInput, CompilerStageTransformationRecordInput,
    CompilerStageTransformationsInput, COMPILER_STAGE_STRUCTURED_RECORD_CONTRACT,
    COMPILER_STAGE_TRANSFORMATION_OUTPUT_ENCODING,
};

use super::*;

struct Fixture {
    handoff: CompilerStageHandoff,
    payloads: Vec<VerifiedCompilerStagePayload>,
    transformations: CompilerStageTransformations,
}

fn fixture() -> Fixture {
    let source = b"mod cpu Main { fn main() -> i64 { return 0; } }\n";
    let tokens = b"nuis-token-stream-v1\nword\t757365\nword\t637075\nsymbol\t59\narrow\n";
    let ast = b"ast mod cpu unit Main\n  fn main() -> i64\n    return 0\n";
    let mut nir = String::from(concat!(
        "use cpu StdLanguageCore\n",
        "use cpu StdCompilerData\n",
        "use cpu StdCompilerTokenEmit\n",
        "use cpu StdCompilerTokens\n",
        "use cpu StdCompilerProjection\n",
        "nir mod cpu unit Main\n",
        "  fn main() -> i64\n",
        "    let value = 40\n",
    ));
    for _ in 0..80 {
        nir.push_str("    let padded = value\n");
    }
    nir.push_str("    return value + 2\n");
    let yir = b"yir 0.1\nmodule cpu Main\n";
    let stages = [
        CompilerStageKind::Source,
        CompilerStageKind::Tokens,
        CompilerStageKind::Ast,
        CompilerStageKind::Nir,
        CompilerStageKind::Yir,
    ];
    let bytes = [source.as_slice(), tokens, ast, nir.as_bytes(), yir];
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
    .expect("build semantic fixture handoff");
    let payloads = stages
        .into_iter()
        .zip(bytes)
        .map(|(stage, bytes)| VerifiedCompilerStagePayload {
            stage,
            bytes: bytes.to_vec(),
        })
        .collect::<Vec<_>>();
    let nir_pages =
        compiler_projection_two_page_identity(CompilerProjectionKind::Nir, nir.as_bytes())
            .expect("materialize fixture NIR pages");
    let nir_words =
        compiler_stage_structural_checkpoint_words(CompilerProjectionKind::Nir, nir_pages);
    let records = [CompilerStageTransformationRecordInput {
        source_stage: CompilerStageKind::Nir,
        transform_contract: COMPILER_STAGE_STRUCTURED_RECORD_CONTRACT,
        output_encoding: COMPILER_STAGE_TRANSFORMATION_OUTPUT_ENCODING,
        output_words: &nir_words,
    }];
    let transformations =
        build_compiler_stage_transformations(&CompilerStageTransformationsInput {
            producer_id: &handoff.producer_id,
            handoff: &handoff,
            payloads: &payloads,
            records: &records,
        })
        .expect("build semantic fixture transformations");
    Fixture {
        handoff,
        payloads,
        transformations,
    }
}

fn input<'a>(fixture: &'a Fixture) -> CompilerStageSemanticDifferentialInput<'a> {
    CompilerStageSemanticDifferentialInput {
        producer_id: &fixture.handoff.producer_id,
        handoff: &fixture.handoff,
        payloads: &fixture.payloads,
        transformations: &fixture.transformations,
    }
}

#[test]
fn lossless_derived_stage_semantics_round_trip_without_replacement_authority() {
    let fixture = fixture();
    let differential = build_compiler_stage_semantic_differential(&input(&fixture))
        .expect("build semantic differential");
    assert_eq!(differential.comparison_count, 1);
    assert_eq!(differential.equivalent_count, 1);
    assert!(differential.deterministic_semantic_equivalent);
    assert!(!differential.replacement_authorized);
    assert_eq!(
        differential.verdict,
        COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_VERDICT
    );
    let comparison = &differential.comparisons[0];
    assert_eq!(comparison.source_stage, CompilerStageKind::Nir);
    assert!(!comparison.byte_identical);
    assert!(comparison.semantically_equivalent);
    assert_eq!(
        comparison.source_payload_sha256,
        comparison.recovered_source_payload_sha256
    );
    assert_eq!(
        comparison.derived_payload_sha256,
        fixture.transformations.records[0].output_payload_sha256
    );
    assert!(comparison.derived_payload_bytes < comparison.source_payload_bytes);

    let source = render_compiler_stage_semantic_differential(&differential);
    let parsed = parse_compiler_stage_semantic_differential_from_source(
        &source,
        Path::new(COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_FILE),
    )
    .expect("parse semantic differential");
    assert_eq!(parsed, differential);
    assert_eq!(render_compiler_stage_semantic_differential(&parsed), source);
}

#[test]
fn semantic_schema_defers_registered_encoding_identity_to_the_bound_transform() {
    let fixture = fixture();
    let mut detached = build_compiler_stage_semantic_differential(&input(&fixture))
        .expect("build semantic differential");
    detached.comparisons[0].derived_encoding = "nuis-test-registered-encoding-v1".to_owned();
    detached.proof_sha256 = differential_identity(&detached);
    validate_differential(&detached).expect("schema accepts a canonical registered encoding token");

    let error = verify_compiler_stage_semantic_differential(&detached, &input(&fixture))
        .expect_err("bound transformation must reject a different encoding identity");
    assert!(error.to_string().contains("bound evidence"));
}

#[test]
fn semantic_differential_reader_binds_transform_and_rejects_equivalence_tampering() {
    let fixture = fixture();
    let differential = build_compiler_stage_semantic_differential(&input(&fixture))
        .expect("build semantic differential");
    let source = render_compiler_stage_semantic_differential(&differential);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock follows epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("nuis_stage_semantic_diff_{nonce}"));
    fs::create_dir_all(&root).expect("create semantic differential root");
    let path = root.join(COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_FILE);
    fs::write(&path, &source).expect("write semantic differential");
    materialize_compiler_stage_transformation_payloads(
        &root,
        &fixture.transformations,
        &fixture.handoff,
        &fixture.payloads,
    )
    .expect("materialize semantic fixture derived payload");
    let verified = read_compiler_stage_semantic_differential(&path, &input(&fixture))
        .expect("verify semantic differential");
    assert_eq!(verified, differential);

    let tampered = source.replacen("byte_identical = false", "byte_identical = true", 1);
    fs::write(&path, tampered).expect("tamper semantic differential");
    let error = read_compiler_stage_semantic_differential(&path, &input(&fixture))
        .expect_err("byte identity tampering must fail");
    assert!(error
        .to_string()
        .contains("semantic comparison 0 is invalid"));
    fs::remove_dir_all(root).expect("remove semantic differential root");
}
