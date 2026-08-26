use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    build_compiler_candidate_execution, build_compiler_component_build,
    build_compiler_stage_handoff, promote_compiler_component_candidate,
    CompilerCandidateExecutionInput, CompilerComponentBuildInput,
    CompilerComponentCandidatePromotionInput, CompilerComponentDependencyInput, CompilerStageKind,
    CompilerStagePayloadInput, VerifiedCompilerStagePayload, COMPILER_COMPONENT_STAGE0_ROLE,
};

use super::*;

struct Evidence {
    stage0: CompilerComponentBuild,
    execution: CompilerCandidateExecution,
    candidate: CompilerComponentBuild,
    handoff: CompilerStageHandoff,
    payloads: Vec<VerifiedCompilerStagePayload>,
    folds: Vec<usize>,
    token_decode: CompilerTokenDecodeSummary,
}

fn evidence() -> Evidence {
    let source = b"mod cpu Main { fn main() -> i64 { return 0; } }\n";
    let tokens = b"nuis-token-stream-v1\nword\t6d6f64\n";
    let ast = b"ast mod cpu unit Main\n";
    let nir = b"nir mod cpu unit Main\n";
    let yir = b"yir 0.1\nmodule cpu Main\n";
    let payload_bytes = [source.as_slice(), tokens, ast, nir, yir];
    let payload_files = [
        "main.source.ns",
        "main.tokens.txt",
        "main.ast.txt",
        "main.nir.txt",
        "main.yir",
    ];
    let stages = [
        CompilerStageKind::Source,
        CompilerStageKind::Tokens,
        CompilerStageKind::Ast,
        CompilerStageKind::Nir,
        CompilerStageKind::Yir,
    ];
    let stage0_inputs = stages
        .into_iter()
        .zip(payload_files)
        .zip(payload_bytes)
        .map(|((stage, payload_file), bytes)| CompilerStagePayloadInput {
            stage,
            payload_file,
            bytes,
        })
        .collect::<Vec<_>>();
    let stage0_handoff =
        build_compiler_stage_handoff("nuisc-stage0-reference", "cpu", "Main", &stage0_inputs)
            .expect("build stage0 handoff");
    let dependencies = [CompilerComponentDependencyInput {
        kind: "component-source",
        identity: "main.ns",
        bytes: source,
    }];
    let stage0 = build_compiler_component_build(&CompilerComponentBuildInput {
        stage_role: COMPILER_COMPONENT_STAGE0_ROLE,
        bootstrap_subset_protocol: "nuis-bootstrap-language-subset-v2",
        component_id: "projection_relay",
        component_domain: "cpu",
        component_unit: "Main",
        producer_id: "nuisc-stage0-reference",
        compiler_image: b"stage0-host-image",
        stage_handoff_file: "nuis.compiler-stage-handoff.toml",
        stage_handoff_bundle_sha256: &stage0_handoff.bundle_sha256,
        build_manifest_file: "nuis.build.manifest.toml",
        build_manifest: b"build-manifest",
        compiled_artifact_file: "nuis.compiled.artifact",
        compiled_artifact: b"compiled-artifact",
        native_binary_file: "projection_relay",
        native_binary: b"nuis-candidate-image",
        dependencies: &dependencies,
    })
    .expect("build stage0 component");
    let execution = build_compiler_candidate_execution(&CompilerCandidateExecutionInput {
        component: &stage0,
        exit_code: 0,
        stdout: &[],
        stderr: &[],
    })
    .expect("build execution proof");
    let candidate_handoff = build_compiler_stage_handoff(
        "nuis-stage1-token-decoder-v2",
        "cpu",
        "Main",
        &stage0_inputs,
    )
    .expect("build candidate handoff");
    let candidate =
        promote_compiler_component_candidate(&CompilerComponentCandidatePromotionInput {
            stage0: &stage0,
            producer_id: "nuis-stage1-token-decoder-v2",
            compiler_image: b"nuis-candidate-image",
            stage_handoff_bundle_sha256: &candidate_handoff.bundle_sha256,
        })
        .expect("promote candidate component");
    let payloads = stages
        .into_iter()
        .zip(payload_bytes)
        .map(|(stage, bytes)| VerifiedCompilerStagePayload {
            stage,
            bytes: bytes.to_vec(),
        })
        .collect::<Vec<_>>();
    let folds = payloads
        .iter()
        .enumerate()
        .map(|(ordinal, payload)| compiler_candidate_stage_fold(ordinal, &payload.bytes))
        .collect();
    let token_decode = decode_compiler_token_stream(tokens).expect("decode fixture tokens");
    Evidence {
        stage0,
        execution,
        candidate,
        handoff: candidate_handoff,
        payloads,
        folds,
        token_decode,
    }
}

fn build(evidence: &Evidence) -> CompilerCandidateProduction {
    build_compiler_candidate_production(&CompilerCandidateProductionInput {
        stage0: &evidence.stage0,
        execution: &evidence.execution,
        candidate: &evidence.candidate,
        handoff: &evidence.handoff,
        payloads: &evidence.payloads,
        stage_folds: &evidence.folds,
        bundle_fold: compiler_candidate_bundle_fold(&evidence.folds),
        token_decode: &evidence.token_decode,
        adapter_file: COMPILER_CANDIDATE_ADAPTER_FILE,
        adapter: b"adapter-image",
    })
    .expect("build candidate production proof")
}

#[test]
fn candidate_production_round_trips_and_never_authorizes_replacement() {
    let evidence = evidence();
    let proof = build(&evidence);
    assert_eq!(proof.record_count, 5);
    assert_eq!(proof.token_record_count, 1);
    assert_eq!(
        proof.token_semantic_fold,
        evidence.token_decode.semantic_fold
    );
    assert!(!proof.replacement_authorized);
    assert_eq!(proof.authority, COMPILER_CANDIDATE_PRODUCTION_AUTHORITY);
    assert_eq!(
        proof.candidate_compiler_image_sha256,
        evidence.stage0.native_binary_sha256
    );

    let source = render_compiler_candidate_production(&proof);
    let parsed = parse_compiler_candidate_production_from_source(
        &source,
        Path::new(COMPILER_CANDIDATE_PRODUCTION_FILE),
    )
    .expect("parse production proof");
    assert_eq!(parsed, proof);
    assert_eq!(render_compiler_candidate_production(&parsed), source);
}

#[test]
fn candidate_production_rejects_fold_and_authority_tampering() {
    let evidence = evidence();
    let mut folds = evidence.folds.clone();
    folds[2] += 1;
    let error = build_compiler_candidate_production(&CompilerCandidateProductionInput {
        stage0: &evidence.stage0,
        execution: &evidence.execution,
        candidate: &evidence.candidate,
        handoff: &evidence.handoff,
        payloads: &evidence.payloads,
        stage_folds: &folds,
        bundle_fold: compiler_candidate_bundle_fold(&folds),
        token_decode: &evidence.token_decode,
        adapter_file: COMPILER_CANDIDATE_ADAPTER_FILE,
        adapter: b"adapter-image",
    })
    .expect_err("Nuis fold drift must fail");
    assert!(error.to_string().contains("stage 2 fold"));

    let mut token_decode = evidence.token_decode;
    token_decode.semantic_fold += 1;
    let error = build_compiler_candidate_production(&CompilerCandidateProductionInput {
        stage0: &evidence.stage0,
        execution: &evidence.execution,
        candidate: &evidence.candidate,
        handoff: &evidence.handoff,
        payloads: &evidence.payloads,
        stage_folds: &evidence.folds,
        bundle_fold: compiler_candidate_bundle_fold(&evidence.folds),
        token_decode: &token_decode,
        adapter_file: COMPILER_CANDIDATE_ADAPTER_FILE,
        adapter: b"adapter-image",
    })
    .expect_err("Nuis token decode drift must fail");
    assert!(error.to_string().contains("token decode summary"));

    let source = render_compiler_candidate_production(&build(&evidence));
    let tampered = source.replacen(
        "replacement_authorized = false",
        "replacement_authorized = true",
        1,
    );
    let error = parse_compiler_candidate_production_from_source(
        &tampered,
        Path::new(COMPILER_CANDIDATE_PRODUCTION_FILE),
    )
    .expect_err("replacement authority mutation must fail");
    assert!(error.to_string().contains("unsupported authority"));
}

#[test]
fn candidate_production_reader_binds_adapter_bytes_and_all_evidence() {
    let evidence = evidence();
    let proof = build(&evidence);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock follows epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("nuis_candidate_production_{nonce}"));
    fs::create_dir_all(&root).expect("create proof directory");
    let path = root.join(COMPILER_CANDIDATE_PRODUCTION_FILE);
    fs::write(&path, render_compiler_candidate_production(&proof)).expect("write proof");
    fs::write(root.join(COMPILER_CANDIDATE_ADAPTER_FILE), b"adapter-image").expect("write adapter");

    let verified = read_compiler_candidate_production(
        &path,
        &evidence.stage0,
        &evidence.execution,
        &evidence.candidate,
        &evidence.handoff,
        &evidence.payloads,
    )
    .expect("verify production proof");
    assert_eq!(verified, proof);

    fs::write(
        root.join(COMPILER_CANDIDATE_ADAPTER_FILE),
        b"tampered-adapter",
    )
    .expect("tamper adapter");
    let error = read_compiler_candidate_production(
        &path,
        &evidence.stage0,
        &evidence.execution,
        &evidence.candidate,
        &evidence.handoff,
        &evidence.payloads,
    )
    .expect_err("adapter tampering must fail");
    assert!(error.to_string().contains("adapter length or SHA-256"));
    fs::remove_dir_all(root).expect("remove proof directory");
}
