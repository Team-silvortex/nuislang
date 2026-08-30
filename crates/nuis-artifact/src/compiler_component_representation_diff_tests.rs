use std::path::Path;

use crate::{
    build_compiler_component_build, build_compiler_component_differential,
    build_compiler_diagnostic_report, build_compiler_stage_handoff,
    build_compiler_stage_handoff_v2, build_compiler_stage_semantic_differential,
    build_compiler_stage_transformations, compiler_projection_two_page_identity,
    compiler_stage_structural_checkpoint_words, CompilerComponentBuild,
    CompilerComponentBuildInput, CompilerComponentDependencyInput, CompilerComponentEvidence,
    CompilerDiagnosticReport, CompilerDiagnosticReportInput, CompilerProjectionKind,
    CompilerStageHandoffV2Input, CompilerStagePayloadInput, CompilerStageSemanticDifferentialInput,
    CompilerStageTransformationRecordInput, CompilerStageTransformationsInput,
    VerifiedCompilerStagePayload, COMPILER_COMPONENT_STAGE0_ROLE,
    COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE, COMPILER_STAGE_STRUCTURED_RECORD_CONTRACT,
    COMPILER_STAGE_TRANSFORMATION_OUTPUT_ENCODING,
};

use super::*;

const AST: &[u8] = concat!(
    "use cpu StdLanguageCore\n",
    "use cpu StdCompilerData\n",
    "use cpu StdCompilerProjection\n",
    "ast mod cpu unit Main\n",
    "  fn main() -> i64\n",
    "    let value = 40\n",
    "    return value + 2\n",
)
.as_bytes();

const CANDIDATE_NIR: &[u8] = concat!(
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

struct OwnedComponentEvidence {
    component: CompilerComponentBuild,
    handoff: CompilerStageHandoff,
    diagnostics: CompilerDiagnosticReport,
}

impl OwnedComponentEvidence {
    fn evidence(&self) -> CompilerComponentEvidence<'_> {
        CompilerComponentEvidence {
            component: &self.component,
            handoff: &self.handoff,
            diagnostics: &self.diagnostics,
        }
    }
}

struct Fixture {
    stage0: OwnedComponentEvidence,
    candidate: OwnedComponentEvidence,
    base_differential: CompilerComponentDifferential,
    candidate_handoff_v2: CompilerStageHandoffV2,
}

fn fixture(stage0_nir: &'static [u8]) -> Fixture {
    let source = b"mod cpu Main { fn main() -> i64 { return 42; } }\n";
    let tokens = b"nuis-token-stream-v1\nword\t6d6f64\nword\t637075\nword\t4d61696e\n";
    let yir = b"yir 0.1\nmodule cpu Main\n";
    let stage0 = build_component_evidence(
        COMPILER_COMPONENT_STAGE0_ROLE,
        "nuisc-stage0-reference",
        b"stage0 compiler",
        [source.as_slice(), tokens, AST, stage0_nir, yir],
    );
    let candidate = build_component_evidence(
        COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE,
        "nuis-stage1-candidate",
        b"stage1 compiler",
        [source.as_slice(), tokens, AST, CANDIDATE_NIR, yir],
    );
    let candidate_payloads = verified_payloads([source, tokens, AST, CANDIDATE_NIR, yir]);
    let ast_pages = compiler_projection_two_page_identity(CompilerProjectionKind::Ast, AST)
        .expect("project AST fixture");
    let nir_pages =
        compiler_projection_two_page_identity(CompilerProjectionKind::Nir, CANDIDATE_NIR)
            .expect("project NIR fixture");
    let ast_words =
        compiler_stage_structural_checkpoint_words(CompilerProjectionKind::Ast, ast_pages);
    let nir_words =
        compiler_stage_structural_checkpoint_words(CompilerProjectionKind::Nir, nir_pages);
    let transformation_records = [
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
            producer_id: &candidate.handoff.producer_id,
            handoff: &candidate.handoff,
            payloads: &candidate_payloads,
            records: &transformation_records,
        })
        .expect("build representation transformations");
    let semantic_differential =
        build_compiler_stage_semantic_differential(&CompilerStageSemanticDifferentialInput {
            producer_id: &candidate.handoff.producer_id,
            handoff: &candidate.handoff,
            payloads: &candidate_payloads,
            transformations: &transformations,
        })
        .expect("build representation semantic differential");
    let candidate_handoff_v2 = build_compiler_stage_handoff_v2(&CompilerStageHandoffV2Input {
        handoff: &candidate.handoff,
        payloads: &candidate_payloads,
        transformations: &transformations,
        semantic_differential: &semantic_differential,
    })
    .expect("build representation handoff v2");
    let base_differential =
        build_compiler_component_differential(stage0.evidence(), candidate.evidence())
            .expect("build base component differential");
    Fixture {
        stage0,
        candidate,
        base_differential,
        candidate_handoff_v2,
    }
}

fn build_component_evidence(
    role: &str,
    producer: &str,
    compiler_image: &[u8],
    bytes: [&[u8]; 5],
) -> OwnedComponentEvidence {
    let stages = [
        CompilerStageKind::Source,
        CompilerStageKind::Tokens,
        CompilerStageKind::Ast,
        CompilerStageKind::Nir,
        CompilerStageKind::Yir,
    ];
    let files = [
        "component.source.ns",
        "component.tokens.txt",
        "component.ast.txt",
        "component.nir.txt",
        "component.yir",
    ];
    let payloads = stages
        .into_iter()
        .zip(files)
        .zip(bytes)
        .map(|((stage, payload_file), bytes)| CompilerStagePayloadInput {
            stage,
            payload_file,
            bytes,
        })
        .collect::<Vec<_>>();
    let handoff = build_compiler_stage_handoff(producer, "cpu", "Main", &payloads)
        .expect("build component handoff");
    let dependencies = [CompilerComponentDependencyInput {
        kind: "component-source",
        identity: "main.ns",
        bytes: b"mod cpu Main {}\n",
    }];
    let component = build_compiler_component_build(&CompilerComponentBuildInput {
        stage_role: role,
        bootstrap_subset_protocol: "nuis-bootstrap-language-subset-v8",
        component_id: "compiler_scanner",
        component_domain: "cpu",
        component_unit: "Main",
        producer_id: producer,
        compiler_image,
        stage_handoff_file: "nuis.compiler-stage-handoff.toml",
        stage_handoff_bundle_sha256: &handoff.bundle_sha256,
        build_manifest_file: "nuis.build.manifest.toml",
        build_manifest: b"build manifest",
        compiled_artifact_file: "nuis.compiled.artifact",
        compiled_artifact: b"compiled artifact",
        native_binary_file: "compiler_scanner",
        native_binary: b"native output",
        dependencies: &dependencies,
    })
    .expect("build component record");
    let diagnostics = build_compiler_diagnostic_report(&CompilerDiagnosticReportInput {
        producer_id: producer,
        component_record_sha256: &component.record_sha256,
        bootstrap_subset_protocol: "nuis-bootstrap-language-subset-v8",
        accepted: true,
        semantic_pipeline: "checked",
        semantic_error: None,
        diagnostics: &[],
    })
    .expect("build component diagnostics");
    OwnedComponentEvidence {
        component,
        handoff,
        diagnostics,
    }
}

fn verified_payloads(bytes: [&[u8]; 5]) -> Vec<VerifiedCompilerStagePayload> {
    [
        CompilerStageKind::Source,
        CompilerStageKind::Tokens,
        CompilerStageKind::Ast,
        CompilerStageKind::Nir,
        CompilerStageKind::Yir,
    ]
    .into_iter()
    .zip(bytes)
    .map(|(stage, bytes)| VerifiedCompilerStagePayload {
        stage,
        bytes: bytes.to_vec(),
    })
    .collect()
}

fn input(fixture: &Fixture) -> CompilerComponentRepresentationDifferentialInput<'_> {
    CompilerComponentRepresentationDifferentialInput {
        base_differential: &fixture.base_differential,
        stage0_handoff: &fixture.stage0.handoff,
        candidate_handoff: &fixture.candidate.handoff,
        candidate_handoff_v2: &fixture.candidate_handoff_v2,
    }
}

#[test]
fn registered_representations_are_compared_without_stage_specific_branches() {
    let fixture = fixture(CANDIDATE_NIR);
    let report = build_compiler_component_representation_differential(&input(&fixture))
        .expect("build representation differential");
    assert_eq!(report.comparison_count, 2);
    assert_eq!(report.equivalent_count, 2);
    assert!(report.all_representations_equivalent);
    assert!(!report.replacement_authorized);
    assert_eq!(report.comparisons[0].source_stage, CompilerStageKind::Ast);
    assert_eq!(report.comparisons[1].source_stage, CompilerStageKind::Nir);
    for comparison in &report.comparisons {
        assert!(!comparison.byte_identical);
        assert!(comparison.reversible);
        assert!(comparison.semantically_equivalent);
        assert!(comparison.equivalent);
        assert_eq!(
            comparison.stage0_payload_sha256,
            comparison.candidate_recovered_payload_sha256
        );
        assert_ne!(
            comparison.stage0_payload_sha256,
            comparison.candidate_selected_payload_sha256
        );
        assert_eq!(
            fixture.base_differential.comparisons[comparison.base_comparison_ordinal].subject,
            comparison.subject
        );
    }

    let source = render_compiler_component_representation_differential(&report);
    let parsed = parse_compiler_component_representation_differential_from_source(
        &source,
        Path::new(COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_FILE),
    )
    .expect("parse representation differential");
    assert_eq!(parsed, report);
    assert_eq!(
        render_compiler_component_representation_differential(&parsed),
        source
    );
}

#[test]
fn canonical_drift_remains_blocked_even_when_candidate_representation_is_reversible() {
    let fixture = fixture(b"nir mod cpu unit Main\n  fn main() -> i64\n    return 41\n");
    assert!(!fixture.base_differential.deterministic_artifact_equivalent);
    let report = build_compiler_component_representation_differential(&input(&fixture))
        .expect("build blocked representation differential");
    assert!(!report.all_representations_equivalent);
    assert_eq!(report.equivalent_count, 1);
    assert_eq!(report.verdict, "blocked-representation-drift");
    let nir = report
        .comparisons
        .iter()
        .find(|comparison| comparison.source_stage == CompilerStageKind::Nir)
        .expect("NIR representation comparison");
    assert!(nir.reversible);
    assert!(nir.semantically_equivalent);
    assert!(!nir.equivalent);
}

#[test]
fn representation_report_tampering_fails_closed() {
    let fixture = fixture(CANDIDATE_NIR);
    let report = build_compiler_component_representation_differential(&input(&fixture))
        .expect("build representation differential");
    let source = render_compiler_component_representation_differential(&report);
    let authorized = source.replacen(
        "replacement_authorized = false",
        "replacement_authorized = true",
        1,
    );
    let error = parse_compiler_component_representation_differential_from_source(
        &authorized,
        Path::new(COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_FILE),
    )
    .expect_err("representation report cannot grant replacement authority");
    assert!(error.to_string().contains("unsafe contract"));

    let drifted = source.replacen(
        "semantically_equivalent = true",
        "semantically_equivalent = false",
        1,
    );
    let error = parse_compiler_component_representation_differential_from_source(
        &drifted,
        Path::new(COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_FILE),
    )
    .expect_err("semantic verdict tampering must fail");
    assert!(error.to_string().contains("comparison 0 is invalid"));
}
