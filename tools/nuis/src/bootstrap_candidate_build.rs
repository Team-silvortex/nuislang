use std::{
    fs,
    path::{Path, PathBuf},
};

use nuis_artifact::{
    build_compiler_candidate_production, build_compiler_diagnostic_report,
    build_compiler_stage_handoff, build_compiler_stage_semantic_differential,
    build_compiler_stage_transformations, materialize_compiler_stage_transformation_payloads,
    parse_compiler_component_differential, promote_compiler_component_candidate,
    read_compiler_candidate_execution, read_compiler_candidate_production,
    read_compiler_component_build, read_compiler_diagnostic_report, read_compiler_stage_handoff,
    read_compiler_stage_semantic_differential, read_compiler_stage_transformations,
    render_compiler_candidate_production, render_compiler_component_build,
    render_compiler_diagnostic_report, render_compiler_stage_handoff,
    render_compiler_stage_semantic_differential, render_compiler_stage_transformations,
    CompilerCandidateProductionInput, CompilerComponentCandidatePromotionInput,
    CompilerDiagnosticReportInput, CompilerStageKind, CompilerStagePayloadInput,
    CompilerStageSemanticDifferentialInput, CompilerStageTransformationRecordInput,
    CompilerStageTransformationsInput, COMPILER_CANDIDATE_EXECUTION_FILE,
    COMPILER_CANDIDATE_PRODUCTION_FILE, COMPILER_COMPONENT_BUILD_FILE,
    COMPILER_COMPONENT_DIFFERENTIAL_FILE, COMPILER_DIAGNOSTIC_REPORT_FILE,
    COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_FILE, COMPILER_STAGE_STRUCTURAL_CHECKPOINT_CONTRACT,
    COMPILER_STAGE_TRANSFORMATION_FILE, COMPILER_STAGE_TRANSFORMATION_OUTPUT_ENCODING,
};

use crate::{
    bootstrap_candidate_adapter::run_candidate_adapter,
    bootstrap_candidate_probe::{
        handle_bootstrap_candidate_probe, handle_bootstrap_clean_candidate_probe,
    },
};

const STAGE0_DIR: &str = "stage0";
const CANDIDATE_DIR: &str = "stage1-candidate";
const STAGE_HANDOFF_FILE: &str = "nuis.compiler-stage-handoff.toml";
const CANDIDATE_PRODUCER_ID: &str = "nuis-stage1-paginated-token-derived-nir-producer-v9";

pub(crate) fn handle_bootstrap_candidate_build(
    input: PathBuf,
    output_dir: PathBuf,
) -> Result<(), String> {
    handle_bootstrap_candidate_build_with_cache(input, output_dir, true)
}

pub(crate) fn handle_bootstrap_clean_candidate_build(
    input: PathBuf,
    output_dir: PathBuf,
) -> Result<(), String> {
    handle_bootstrap_candidate_build_with_cache(input, output_dir, false)
}

fn handle_bootstrap_candidate_build_with_cache(
    input: PathBuf,
    output_dir: PathBuf,
    reuse_compile_cache: bool,
) -> Result<(), String> {
    let stage0_dir = output_dir.join(STAGE0_DIR);
    let candidate_dir = output_dir.join(CANDIDATE_DIR);
    fs::create_dir_all(&output_dir).map_err(|error| {
        format!(
            "failed to create bootstrap candidate root `{}`: {error}",
            output_dir.display()
        )
    })?;
    if reuse_compile_cache {
        handle_bootstrap_candidate_probe(input, stage0_dir.clone())?;
    } else {
        handle_bootstrap_clean_candidate_probe(input, stage0_dir.clone())?;
    }

    let stage0_path = stage0_dir.join(COMPILER_COMPONENT_BUILD_FILE);
    let stage0 = read_compiler_component_build(&stage0_path)
        .map_err(|error| format!("failed to verify stage0 candidate source: {error}"))?;
    let execution_path = stage0_dir.join(COMPILER_CANDIDATE_EXECUTION_FILE);
    let execution = read_compiler_candidate_execution(&execution_path)
        .map_err(|error| format!("failed to verify candidate execution: {error}"))?;
    let (stage0_handoff, stage0_payloads) =
        read_compiler_stage_handoff(&stage0_dir.join(&stage0.stage_handoff_file))
            .map_err(|error| format!("failed to verify stage0 handoff: {error}"))?;

    let adapter = run_candidate_adapter(&stage0_dir, &candidate_dir, &stage0_handoff)?;
    let candidate_handoff =
        materialize_candidate_handoff(&candidate_dir, &stage0_handoff, &stage0_payloads)?;
    copy_component_payloads(&stage0_dir, &candidate_dir, &stage0)?;
    let candidate_image =
        fs::read(stage0_dir.join(&stage0.native_binary_file)).map_err(|error| {
            format!(
                "failed to read executed Nuis candidate image `{}`: {error}",
                stage0.native_binary_file
            )
        })?;
    let candidate =
        promote_compiler_component_candidate(&CompilerComponentCandidatePromotionInput {
            stage0: &stage0,
            producer_id: CANDIDATE_PRODUCER_ID,
            compiler_image: &candidate_image,
            stage_handoff_bundle_sha256: &candidate_handoff.bundle_sha256,
        })
        .map_err(|error| format!("failed to promote Nuis candidate component: {error}"))?;
    let candidate_path = candidate_dir.join(COMPILER_COMPONENT_BUILD_FILE);
    fs::write(&candidate_path, render_compiler_component_build(&candidate)).map_err(|error| {
        format!(
            "failed to write candidate component `{}`: {error}",
            candidate_path.display()
        )
    })?;
    let verified_candidate = read_compiler_component_build(&candidate_path)
        .map_err(|error| format!("failed to verify candidate component: {error}"))?;
    let diagnostics = build_compiler_diagnostic_report(&CompilerDiagnosticReportInput {
        producer_id: &verified_candidate.producer_id,
        component_record_sha256: &verified_candidate.record_sha256,
        bootstrap_subset_protocol: &verified_candidate.bootstrap_subset_protocol,
        accepted: true,
        semantic_pipeline: "checked",
        semantic_error: None,
        diagnostics: &[],
    })
    .map_err(|error| format!("failed to build candidate diagnostics: {error}"))?;
    let diagnostics_path = candidate_dir.join(COMPILER_DIAGNOSTIC_REPORT_FILE);
    fs::write(
        &diagnostics_path,
        render_compiler_diagnostic_report(&diagnostics),
    )
    .map_err(|error| {
        format!(
            "failed to write candidate diagnostics `{}`: {error}",
            diagnostics_path.display()
        )
    })?;
    read_compiler_diagnostic_report(
        &diagnostics_path,
        &verified_candidate.record_sha256,
        &verified_candidate.producer_id,
    )
    .map_err(|error| format!("failed to verify candidate diagnostics: {error}"))?;

    let (verified_handoff, verified_payloads) =
        read_compiler_stage_handoff(&candidate_dir.join(STAGE_HANDOFF_FILE))
            .map_err(|error| format!("failed to verify candidate handoff: {error}"))?;
    let transformation_records = [CompilerStageTransformationRecordInput {
        source_stage: CompilerStageKind::Nir,
        transform_contract: COMPILER_STAGE_STRUCTURAL_CHECKPOINT_CONTRACT,
        output_encoding: COMPILER_STAGE_TRANSFORMATION_OUTPUT_ENCODING,
        output_words: &adapter.nir_transformation_words,
    }];
    let stage_transformations =
        build_compiler_stage_transformations(&CompilerStageTransformationsInput {
            producer_id: &verified_candidate.producer_id,
            handoff: &verified_handoff,
            payloads: &verified_payloads,
            records: &transformation_records,
        })
        .map_err(|error| format!("failed to attest candidate stage transformation: {error}"))?;
    let stage_transformations_path = candidate_dir.join(COMPILER_STAGE_TRANSFORMATION_FILE);
    fs::write(
        &stage_transformations_path,
        render_compiler_stage_transformations(&stage_transformations),
    )
    .map_err(|error| {
        format!(
            "failed to write candidate stage transformations `{}`: {error}",
            stage_transformations_path.display()
        )
    })?;
    materialize_compiler_stage_transformation_payloads(
        &candidate_dir,
        &stage_transformations,
        &verified_handoff,
        &verified_payloads,
    )
    .map_err(|error| format!("failed to materialize candidate stage payload: {error}"))?;
    let verified_stage_transformations = read_compiler_stage_transformations(
        &stage_transformations_path,
        &verified_handoff,
        &verified_payloads,
    )
    .map_err(|error| format!("failed to verify candidate stage transformations: {error}"))?;
    let stage_semantic_input = CompilerStageSemanticDifferentialInput {
        producer_id: &verified_candidate.producer_id,
        handoff: &verified_handoff,
        payloads: &verified_payloads,
        transformations: &verified_stage_transformations,
    };
    let stage_semantic_differential =
        build_compiler_stage_semantic_differential(&stage_semantic_input)
            .map_err(|error| format!("failed to compare candidate stage semantics: {error}"))?;
    let stage_semantic_differential_path =
        candidate_dir.join(COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_FILE);
    fs::write(
        &stage_semantic_differential_path,
        render_compiler_stage_semantic_differential(&stage_semantic_differential),
    )
    .map_err(|error| {
        format!(
            "failed to write candidate stage semantic differential `{}`: {error}",
            stage_semantic_differential_path.display()
        )
    })?;
    let verified_stage_semantic_differential = read_compiler_stage_semantic_differential(
        &stage_semantic_differential_path,
        &stage_semantic_input,
    )
    .map_err(|error| format!("failed to verify candidate stage semantics: {error}"))?;
    let production = build_compiler_candidate_production(&CompilerCandidateProductionInput {
        stage0: &stage0,
        execution: &execution,
        candidate: &verified_candidate,
        handoff: &verified_handoff,
        payloads: &verified_payloads,
        stage_folds: &adapter.stage_folds,
        bundle_fold: adapter.bundle_fold,
        token_decode: &adapter.token_decode,
        token_page: &adapter.token_page,
        token_pagination: &adapter.token_pagination,
        ast_pages: &adapter.ast_pages,
        nir_pages: &adapter.nir_pages,
        stage_transformations_file: COMPILER_STAGE_TRANSFORMATION_FILE,
        stage_transformations: &verified_stage_transformations,
        stage_semantic_differential_file: COMPILER_STAGE_SEMANTIC_DIFFERENTIAL_FILE,
        stage_semantic_differential: &verified_stage_semantic_differential,
        adapter_file: adapter.adapter_file,
        adapter: &adapter.adapter,
    })
    .map_err(|error| format!("failed to attest candidate production: {error}"))?;
    let production_path = candidate_dir.join(COMPILER_CANDIDATE_PRODUCTION_FILE);
    fs::write(
        &production_path,
        render_compiler_candidate_production(&production),
    )
    .map_err(|error| {
        format!(
            "failed to write candidate production proof `{}`: {error}",
            production_path.display()
        )
    })?;
    let verified_production = read_compiler_candidate_production(
        &production_path,
        &stage0,
        &execution,
        &verified_candidate,
        &verified_handoff,
        &verified_payloads,
    )
    .map_err(|error| format!("failed to verify candidate production proof: {error}"))?;

    let report_path = output_dir.join(COMPILER_COMPONENT_DIFFERENTIAL_FILE);
    nuisc::run(nuisc::CommandKind::BootstrapDiff {
        stage0_record: stage0_path,
        candidate_record: candidate_path,
        report: report_path.clone(),
    })?;
    let differential = parse_compiler_component_differential(&report_path)
        .map_err(|error| format!("failed to verify candidate differential: {error}"))?;
    if !differential.deterministic_artifact_equivalent || differential.replacement_authorized {
        return Err(
            "candidate production did not reach equivalent-awaiting-authorization".to_owned(),
        );
    }

    println!("bootstrap candidate component: produced");
    println!("  producer: {}", verified_candidate.producer_id);
    println!("  stage_role: {}", verified_candidate.stage_role);
    println!("  stage_records: {}", verified_production.record_count);
    println!("  bundle_fold: {}", verified_production.bundle_fold);
    println!(
        "  token_records: {}",
        verified_production.token_record_count
    );
    println!(
        "  token_semantic_fold: {}",
        verified_production.token_semantic_fold
    );
    println!(
        "  token_page_identity: {}",
        verified_production.token_page_identity
    );
    println!(
        "  token_page_count: {}",
        verified_production.token_page_count
    );
    println!(
        "  token_terminal_page_hash: {}",
        verified_production.token_terminal_page_hash
    );
    println!(
        "  token_page_chain_identity: {}",
        verified_production.token_page_chain_identity
    );
    println!(
        "  ast_page_identity: {}",
        verified_production.ast_page_identity
    );
    println!(
        "  ast_continuation_page_identity: {}",
        verified_production.ast_continuation_page_identity
    );
    println!(
        "  nir_page_identity: {}",
        verified_production.nir_page_identity
    );
    println!(
        "  nir_continuation_page_identity: {}",
        verified_production.nir_continuation_page_identity
    );
    println!(
        "  stage_transformation_sha256: {}",
        verified_production.stage_transformations_sha256
    );
    println!(
        "  stage_semantic_differential_sha256: {}",
        verified_production.stage_semantic_differential_proof_sha256
    );
    println!(
        "  derived_stage_payload: {}",
        verified_stage_transformations.records[0].output_payload_file
    );
    println!("  production_sha256: {}", verified_production.proof_sha256);
    println!("  differential: {}", differential.verdict);
    println!("  replacement_authorized: false");
    println!(
        "  candidate_record: {}",
        candidate_dir.join(COMPILER_COMPONENT_BUILD_FILE).display()
    );
    println!("  production_record: {}", production_path.display());
    println!("  differential_record: {}", report_path.display());
    Ok(())
}

fn materialize_candidate_handoff(
    candidate_dir: &Path,
    stage0_handoff: &nuis_artifact::CompilerStageHandoff,
    payloads: &[nuis_artifact::VerifiedCompilerStagePayload],
) -> Result<nuis_artifact::CompilerStageHandoff, String> {
    if stage0_handoff.records.len() != payloads.len() {
        return Err("stage0 handoff payload count changed before candidate production".to_owned());
    }
    for (record, payload) in stage0_handoff.records.iter().zip(payloads) {
        fs::write(candidate_dir.join(&record.payload_file), &payload.bytes).map_err(|error| {
            format!(
                "failed to materialize candidate stage payload `{}`: {error}",
                record.payload_file
            )
        })?;
    }
    let inputs = stage0_handoff
        .records
        .iter()
        .zip(payloads)
        .map(|(record, payload)| CompilerStagePayloadInput {
            stage: record.stage,
            payload_file: &record.payload_file,
            bytes: &payload.bytes,
        })
        .collect::<Vec<_>>();
    let handoff = build_compiler_stage_handoff(
        CANDIDATE_PRODUCER_ID,
        &stage0_handoff.module_domain,
        &stage0_handoff.module_unit,
        &inputs,
    )
    .map_err(|error| format!("failed to build candidate handoff: {error}"))?;
    fs::write(
        candidate_dir.join(STAGE_HANDOFF_FILE),
        render_compiler_stage_handoff(&handoff),
    )
    .map_err(|error| format!("failed to write candidate handoff: {error}"))?;
    Ok(handoff)
}

fn copy_component_payloads(
    stage0_dir: &Path,
    candidate_dir: &Path,
    stage0: &nuis_artifact::CompilerComponentBuild,
) -> Result<(), String> {
    for file in [
        &stage0.build_manifest_file,
        &stage0.compiled_artifact_file,
        &stage0.native_binary_file,
    ] {
        fs::copy(stage0_dir.join(file), candidate_dir.join(file)).map_err(|error| {
            format!("failed to copy candidate component payload `{file}`: {error}")
        })?;
    }
    Ok(())
}
