use std::path::Path;

use crate::{
    build_compiler_component_build, build_compiler_diagnostic_report, build_compiler_stage_handoff,
    CompilerComponentBuildInput, CompilerComponentDependencyInput, CompilerDiagnosticReportInput,
    CompilerStageKind, CompilerStagePayloadInput,
};

use super::*;

struct OwnedEvidence {
    component: CompilerComponentBuild,
    handoff: CompilerStageHandoff,
    diagnostics: CompilerDiagnosticReport,
}

impl OwnedEvidence {
    fn evidence(&self) -> CompilerComponentEvidence<'_> {
        CompilerComponentEvidence {
            component: &self.component,
            handoff: &self.handoff,
            diagnostics: &self.diagnostics,
        }
    }
}

fn build_evidence(
    role: &str,
    producer: &str,
    compiler_image: &[u8],
    yir: &[u8],
    native: &[u8],
    dependency_source: &[u8],
) -> OwnedEvidence {
    let source = b"mod cpu Main { fn main() -> i64 { return 7; } }\n";
    let tokens = b"nuis-token-stream-v1\nword\t6d6f64\n";
    let ast = b"ast mod cpu unit Main\n";
    let nir = b"nir mod cpu unit Main\n";
    let payloads = [
        CompilerStagePayloadInput {
            stage: CompilerStageKind::Source,
            payload_file: "component.source.ns",
            bytes: source,
        },
        CompilerStagePayloadInput {
            stage: CompilerStageKind::Tokens,
            payload_file: "component.tokens.txt",
            bytes: tokens,
        },
        CompilerStagePayloadInput {
            stage: CompilerStageKind::Ast,
            payload_file: "component.ast.txt",
            bytes: ast,
        },
        CompilerStagePayloadInput {
            stage: CompilerStageKind::Nir,
            payload_file: "component.nir.txt",
            bytes: nir,
        },
        CompilerStagePayloadInput {
            stage: CompilerStageKind::Yir,
            payload_file: "component.yir",
            bytes: yir,
        },
    ];
    let handoff = build_compiler_stage_handoff(producer, "cpu", "Main", &payloads)
        .expect("build compiler handoff");
    let dependencies = [CompilerComponentDependencyInput {
        kind: "component-source",
        identity: "main.ns",
        bytes: dependency_source,
    }];
    let component = build_compiler_component_build(&CompilerComponentBuildInput {
        stage_role: role,
        bootstrap_subset_protocol: "nuis-bootstrap-language-subset-v1",
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
        native_binary: native,
        dependencies: &dependencies,
    })
    .expect("build compiler component");
    let diagnostics = build_compiler_diagnostic_report(&CompilerDiagnosticReportInput {
        producer_id: producer,
        component_record_sha256: &component.record_sha256,
        bootstrap_subset_protocol: "nuis-bootstrap-language-subset-v1",
        accepted: true,
        semantic_pipeline: "checked",
        semantic_error: None,
        diagnostics: &[],
    })
    .expect("build compiler diagnostic report");
    OwnedEvidence {
        component,
        handoff,
        diagnostics,
    }
}

fn equivalent_pair() -> (OwnedEvidence, OwnedEvidence) {
    (
        build_evidence(
            COMPILER_COMPONENT_STAGE0_ROLE,
            "nuisc-stage0-reference",
            b"stage0 compiler",
            b"yir 0.1\n",
            b"native output",
            b"mod cpu Main {}\n",
        ),
        build_evidence(
            COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE,
            "nuis-stage1-candidate",
            b"stage1 compiler",
            b"yir 0.1\n",
            b"native output",
            b"mod cpu Main {}\n",
        ),
    )
}

#[test]
fn equivalent_producers_remain_unauthorized_and_round_trip() {
    let (stage0, candidate) = equivalent_pair();
    let report = build_compiler_component_differential(stage0.evidence(), candidate.evidence())
        .expect("compare equivalent compiler components");
    assert!(report.deterministic_artifact_equivalent);
    assert_eq!(report.equivalent_count, report.comparison_count);
    assert_eq!(report.verdict, "equivalent-awaiting-authorization");
    assert!(!report.replacement_authorized);
    assert_ne!(
        report.stage0_compiler_image_sha256,
        report.candidate_compiler_image_sha256
    );

    let source = render_compiler_component_differential(&report);
    let parsed = parse_compiler_component_differential_from_source(
        &source,
        Path::new("nuis.compiler-component-diff.toml"),
    )
    .expect("parse differential report");
    assert_eq!(parsed, report);
    assert_eq!(render_compiler_component_differential(&parsed), source);
}

#[test]
fn semantic_drift_blocks_the_candidate() {
    let stage0 = equivalent_pair().0;
    let candidate = build_evidence(
        COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE,
        "nuis-stage1-candidate",
        b"stage1 compiler",
        b"yir 0.1\nfunc @drift\n",
        b"native output",
        b"mod cpu Main {}\n",
    );
    let report = build_compiler_component_differential(stage0.evidence(), candidate.evidence())
        .expect("report semantic drift");
    assert!(!report.stage_equivalent);
    assert!(!report.deterministic_artifact_equivalent);
    assert_eq!(report.verdict, "blocked-drift");
    assert!(!report.comparisons[8].equivalent);
    assert!(!report.comparisons[9].equivalent);
}

#[test]
fn dependency_and_native_drift_are_independently_visible() {
    let stage0 = equivalent_pair().0;
    let candidate = build_evidence(
        COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE,
        "nuis-stage1-candidate",
        b"stage1 compiler",
        b"yir 0.1\n",
        b"different native output",
        b"mod cpu Main { fn changed() {} }\n",
    );
    let report = build_compiler_component_differential(stage0.evidence(), candidate.evidence())
        .expect("report dependency and native drift");
    assert!(report.stage_equivalent);
    assert!(report.diagnostics_equivalent);
    assert!(!report.dependency_closure_equivalent);
    assert!(!report.native_output_equivalent);
    assert_eq!(report.verdict, "blocked-drift");
}

#[test]
fn same_producer_or_wrong_role_cannot_enter_the_gate() {
    let (stage0, _) = equivalent_pair();
    let same_producer = build_evidence(
        COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE,
        "nuisc-stage0-reference",
        b"candidate compiler",
        b"yir 0.1\n",
        b"native output",
        b"mod cpu Main {}\n",
    );
    let error = build_compiler_component_differential(stage0.evidence(), same_producer.evidence())
        .expect_err("same producer must fail");
    assert!(error.to_string().contains("separately identified"));

    let wrong_role = build_evidence(
        COMPILER_COMPONENT_STAGE0_ROLE,
        "other-stage0",
        b"other compiler",
        b"yir 0.1\n",
        b"native output",
        b"mod cpu Main {}\n",
    );
    let error = build_compiler_component_differential(stage0.evidence(), wrong_role.evidence())
        .expect_err("candidate role must be explicit");
    assert!(error.to_string().contains("stage1-candidate"));
}

#[test]
fn rendered_verdict_and_comparison_tampering_fail_closed() {
    let (stage0, candidate) = equivalent_pair();
    let report = build_compiler_component_differential(stage0.evidence(), candidate.evidence())
        .expect("compare components");
    let source = render_compiler_component_differential(&report);
    let verdict_tamper = source.replacen("equivalent-awaiting-authorization", "blocked-drift", 1);
    let error = parse_compiler_component_differential_from_source(
        &verdict_tamper,
        Path::new("nuis.compiler-component-diff.toml"),
    )
    .expect_err("verdict tamper must fail");
    assert!(error.to_string().contains("verdict mismatch"));

    let comparison_tamper = source.replacen("\nequivalent = true\n", "\nequivalent = false\n", 1);
    let error = parse_compiler_component_differential_from_source(
        &comparison_tamper,
        Path::new("nuis.compiler-component-diff.toml"),
    )
    .expect_err("comparison tamper must fail");
    assert!(error.to_string().contains("comparison verdict mismatch"));
}
