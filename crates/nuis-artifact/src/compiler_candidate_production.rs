use std::{fs, path::Path};

#[path = "compiler_candidate_production_identity.rs"]
mod identity;
#[path = "compiler_candidate_production_support.rs"]
mod support;
#[path = "compiler_candidate_production_validation.rs"]
mod validation;

use identity::production_identity;
use support::{parse_record_blocks, sha256_hex, validate_text};
use validation::{validate_evidence, validate_proof};

use crate::{
    compiler_projection_two_page_identity, read_compiler_stage_semantic_differential,
    read_compiler_stage_transformations, render_compiler_stage_semantic_differential,
    render_compiler_stage_transformations,
    toml::{
        escape_toml_string, parse_required_toml_bool, parse_required_toml_string,
        parse_required_toml_usize,
    },
    ArtifactError, CompilerCandidateExecution, CompilerComponentBuild, CompilerProjectionKind,
    CompilerProjectionTwoPageIdentity, CompilerStageHandoff, CompilerStageKind,
    CompilerStageSemanticDifferential, CompilerStageSemanticDifferentialInput,
    CompilerStageTransformations, CompilerTokenDecodeSummary, CompilerTokenPageIdentity,
    CompilerTokenPaginationIdentity, VerifiedCompilerStagePayload,
    COMPILER_PROJECTION_CURSOR_CONTRACT, COMPILER_PROJECTION_PAGE_CONTRACT,
    COMPILER_TOKEN_DECODER_CONTRACT, COMPILER_TOKEN_PAGINATION_CONTRACT,
};

pub const COMPILER_CANDIDATE_PRODUCTION_PROTOCOL: &str = "nuis-compiler-candidate-production-v10";
pub const COMPILER_CANDIDATE_PRODUCER_CONTRACT: &str =
    "nuis-stage1-compact-structured-nir-producer-v10";
pub const COMPILER_CANDIDATE_PRODUCTION_AUTHORITY: &str =
    "stage1-candidate-component-production-no-replacement";
pub const COMPILER_CANDIDATE_PRODUCTION_FILE: &str = "nuis.compiler-candidate-production.toml";
pub const COMPILER_CANDIDATE_ADAPTER_FILE: &str = "nuis.compiler-candidate-adapter";

const FOLD_MODULUS: u64 = 2_147_483_629;
const EXPECTED_STAGE_COUNT: usize = 5;

#[derive(Debug, Clone, Copy)]
pub struct CompilerCandidateProductionInput<'a> {
    pub stage0: &'a CompilerComponentBuild,
    pub execution: &'a CompilerCandidateExecution,
    pub candidate: &'a CompilerComponentBuild,
    pub handoff: &'a CompilerStageHandoff,
    pub payloads: &'a [VerifiedCompilerStagePayload],
    pub stage_folds: &'a [usize],
    pub bundle_fold: usize,
    pub token_decode: &'a CompilerTokenDecodeSummary,
    pub token_page: &'a CompilerTokenPageIdentity,
    pub token_pagination: &'a CompilerTokenPaginationIdentity,
    pub ast_pages: &'a CompilerProjectionTwoPageIdentity,
    pub nir_pages: &'a CompilerProjectionTwoPageIdentity,
    pub stage_transformations_file: &'a str,
    pub stage_transformations: &'a CompilerStageTransformations,
    pub stage_semantic_differential_file: &'a str,
    pub stage_semantic_differential: &'a CompilerStageSemanticDifferential,
    pub adapter_file: &'a str,
    pub adapter: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerCandidateProductionRecord {
    pub ordinal: usize,
    pub stage: String,
    pub payload_bytes: usize,
    pub payload_sha256: String,
    pub fold: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerCandidateProduction {
    pub protocol: String,
    pub producer_contract: String,
    pub authority: String,
    pub stage0_component_sha256: String,
    pub stage0_execution_sha256: String,
    pub candidate_component_sha256: String,
    pub candidate_producer_id: String,
    pub candidate_compiler_image_sha256: String,
    pub stage_handoff_bundle_sha256: String,
    pub stage_transformations_file: String,
    pub stage_transformations_bytes: usize,
    pub stage_transformations_sha256: String,
    pub stage_semantic_differential_file: String,
    pub stage_semantic_differential_bytes: usize,
    pub stage_semantic_differential_sha256: String,
    pub stage_semantic_differential_proof_sha256: String,
    pub adapter_file: String,
    pub adapter_bytes: usize,
    pub adapter_sha256: String,
    pub record_count: usize,
    pub bundle_fold: usize,
    pub token_decoder_contract: String,
    pub token_record_count: usize,
    pub token_semantic_fold: usize,
    pub token_page_record_count: usize,
    pub token_page_payload_bytes: usize,
    pub token_page_canonical_bytes: usize,
    pub token_page_canonical_hash: usize,
    pub token_page_identity: usize,
    pub token_pagination_contract: String,
    pub token_page_count: usize,
    pub token_terminal_page_hash: usize,
    pub token_page_chain_identity: usize,
    pub projection_cursor_contract: String,
    pub ast_page_contract: String,
    pub ast_page_record_count: usize,
    pub ast_page_bytes: usize,
    pub ast_page_projection_hash: usize,
    pub ast_page_continuation_indentation: usize,
    pub ast_page_continuation_body_bytes: usize,
    pub ast_page_continuation_body_hash: usize,
    pub ast_page_state_hash: usize,
    pub ast_page_identity: usize,
    pub ast_page_cursor_identity: usize,
    pub ast_continuation_page_identity: usize,
    pub ast_continuation_cursor_identity: usize,
    pub nir_page_contract: String,
    pub nir_page_record_count: usize,
    pub nir_page_bytes: usize,
    pub nir_page_projection_hash: usize,
    pub nir_page_continuation_indentation: usize,
    pub nir_page_continuation_body_bytes: usize,
    pub nir_page_continuation_body_hash: usize,
    pub nir_page_state_hash: usize,
    pub nir_page_identity: usize,
    pub nir_page_cursor_identity: usize,
    pub nir_continuation_page_identity: usize,
    pub nir_continuation_cursor_identity: usize,
    pub replacement_authorized: bool,
    pub proof_sha256: String,
    pub records: Vec<CompilerCandidateProductionRecord>,
}

pub fn compiler_candidate_stage_fold(ordinal: usize, bytes: &[u8]) -> usize {
    let mut state = 97_u64 + ((ordinal as u64 + 1) * 17);
    for byte in bytes {
        state = ((state * 257) + u64::from(*byte) + ordinal as u64 + 1) % FOLD_MODULUS;
    }
    state as usize
}

pub fn compiler_candidate_bundle_fold(stage_folds: &[usize]) -> usize {
    let mut state = 193_u64;
    for (ordinal, fold) in stage_folds.iter().copied().enumerate() {
        state = ((state * 65_537) + fold as u64 + ordinal as u64 + 1) % FOLD_MODULUS;
    }
    state as usize
}

pub fn build_compiler_candidate_production(
    input: &CompilerCandidateProductionInput<'_>,
) -> Result<CompilerCandidateProduction, ArtifactError> {
    validate_evidence(input)?;
    let records = input
        .handoff
        .records
        .iter()
        .zip(input.payloads)
        .zip(input.stage_folds)
        .map(
            |((record, payload), fold)| CompilerCandidateProductionRecord {
                ordinal: record.ordinal,
                stage: record.stage.as_str().to_owned(),
                payload_bytes: payload.bytes.len(),
                payload_sha256: record.payload_sha256.clone(),
                fold: *fold,
            },
        )
        .collect::<Vec<_>>();
    let stage_transformations_source =
        render_compiler_stage_transformations(input.stage_transformations);
    let stage_semantic_differential_source =
        render_compiler_stage_semantic_differential(input.stage_semantic_differential);
    let mut proof = CompilerCandidateProduction {
        protocol: COMPILER_CANDIDATE_PRODUCTION_PROTOCOL.to_owned(),
        producer_contract: COMPILER_CANDIDATE_PRODUCER_CONTRACT.to_owned(),
        authority: COMPILER_CANDIDATE_PRODUCTION_AUTHORITY.to_owned(),
        stage0_component_sha256: input.stage0.record_sha256.clone(),
        stage0_execution_sha256: input.execution.execution_sha256.clone(),
        candidate_component_sha256: input.candidate.record_sha256.clone(),
        candidate_producer_id: input.candidate.producer_id.clone(),
        candidate_compiler_image_sha256: input.candidate.compiler_image_sha256.clone(),
        stage_handoff_bundle_sha256: input.handoff.bundle_sha256.clone(),
        stage_transformations_file: input.stage_transformations_file.to_owned(),
        stage_transformations_bytes: stage_transformations_source.len(),
        stage_transformations_sha256: sha256_hex(stage_transformations_source.as_bytes()),
        stage_semantic_differential_file: input.stage_semantic_differential_file.to_owned(),
        stage_semantic_differential_bytes: stage_semantic_differential_source.len(),
        stage_semantic_differential_sha256: sha256_hex(
            stage_semantic_differential_source.as_bytes(),
        ),
        stage_semantic_differential_proof_sha256: input
            .stage_semantic_differential
            .proof_sha256
            .clone(),
        adapter_file: input.adapter_file.to_owned(),
        adapter_bytes: input.adapter.len(),
        adapter_sha256: sha256_hex(input.adapter),
        record_count: records.len(),
        bundle_fold: input.bundle_fold,
        token_decoder_contract: COMPILER_TOKEN_DECODER_CONTRACT.to_owned(),
        token_record_count: input.token_decode.record_count,
        token_semantic_fold: input.token_decode.semantic_fold,
        token_page_record_count: input.token_page.record_count,
        token_page_payload_bytes: input.token_page.payload_bytes,
        token_page_canonical_bytes: input.token_page.canonical_bytes,
        token_page_canonical_hash: input.token_page.canonical_hash,
        token_page_identity: input.token_page.identity,
        token_pagination_contract: COMPILER_TOKEN_PAGINATION_CONTRACT.to_owned(),
        token_page_count: input.token_pagination.page_count,
        token_terminal_page_hash: input.token_pagination.terminal_page_hash,
        token_page_chain_identity: input.token_pagination.chain_identity,
        projection_cursor_contract: COMPILER_PROJECTION_CURSOR_CONTRACT.to_owned(),
        ast_page_contract: COMPILER_PROJECTION_PAGE_CONTRACT.to_owned(),
        ast_page_record_count: input.ast_pages.first.page.record_count,
        ast_page_bytes: input.ast_pages.first.page.page_bytes,
        ast_page_projection_hash: input.ast_pages.first.page.projection_hash,
        ast_page_continuation_indentation: input.ast_pages.first.page.continuation_indentation,
        ast_page_continuation_body_bytes: input.ast_pages.first.page.continuation_body_bytes,
        ast_page_continuation_body_hash: input.ast_pages.first.page.continuation_body_hash,
        ast_page_state_hash: input.ast_pages.first.page.state_hash,
        ast_page_identity: input.ast_pages.first.page.identity,
        ast_page_cursor_identity: input.ast_pages.first.cursor_identity,
        ast_continuation_page_identity: input.ast_pages.second.page.identity,
        ast_continuation_cursor_identity: input.ast_pages.second.cursor_identity,
        nir_page_contract: COMPILER_PROJECTION_PAGE_CONTRACT.to_owned(),
        nir_page_record_count: input.nir_pages.first.page.record_count,
        nir_page_bytes: input.nir_pages.first.page.page_bytes,
        nir_page_projection_hash: input.nir_pages.first.page.projection_hash,
        nir_page_continuation_indentation: input.nir_pages.first.page.continuation_indentation,
        nir_page_continuation_body_bytes: input.nir_pages.first.page.continuation_body_bytes,
        nir_page_continuation_body_hash: input.nir_pages.first.page.continuation_body_hash,
        nir_page_state_hash: input.nir_pages.first.page.state_hash,
        nir_page_identity: input.nir_pages.first.page.identity,
        nir_page_cursor_identity: input.nir_pages.first.cursor_identity,
        nir_continuation_page_identity: input.nir_pages.second.page.identity,
        nir_continuation_cursor_identity: input.nir_pages.second.cursor_identity,
        replacement_authorized: false,
        proof_sha256: String::new(),
        records,
    };
    proof.proof_sha256 = production_identity(&proof);
    validate_proof(&proof)?;
    Ok(proof)
}

pub fn render_compiler_candidate_production(proof: &CompilerCandidateProduction) -> String {
    let mut out = format!(
        "protocol = \"{}\"\nproducer_contract = \"{}\"\nauthority = \"{}\"\nstage0_component_sha256 = \"{}\"\nstage0_execution_sha256 = \"{}\"\ncandidate_component_sha256 = \"{}\"\ncandidate_producer_id = \"{}\"\ncandidate_compiler_image_sha256 = \"{}\"\nstage_handoff_bundle_sha256 = \"{}\"\nstage_transformations_file = \"{}\"\nstage_transformations_bytes = {}\nstage_transformations_sha256 = \"{}\"\nstage_semantic_differential_file = \"{}\"\nstage_semantic_differential_bytes = {}\nstage_semantic_differential_sha256 = \"{}\"\nstage_semantic_differential_proof_sha256 = \"{}\"\nadapter_file = \"{}\"\nadapter_bytes = {}\nadapter_sha256 = \"{}\"\nrecord_count = {}\nbundle_fold = {}\ntoken_decoder_contract = \"{}\"\ntoken_record_count = {}\ntoken_semantic_fold = {}\ntoken_page_record_count = {}\ntoken_page_payload_bytes = {}\ntoken_page_canonical_bytes = {}\ntoken_page_canonical_hash = {}\ntoken_page_identity = {}\ntoken_pagination_contract = \"{}\"\ntoken_page_count = {}\ntoken_terminal_page_hash = {}\ntoken_page_chain_identity = {}\nprojection_cursor_contract = \"{}\"\nast_page_contract = \"{}\"\nast_page_record_count = {}\nast_page_bytes = {}\nast_page_projection_hash = {}\nast_page_continuation_indentation = {}\nast_page_continuation_body_bytes = {}\nast_page_continuation_body_hash = {}\nast_page_state_hash = {}\nast_page_identity = {}\nast_page_cursor_identity = {}\nast_continuation_page_identity = {}\nast_continuation_cursor_identity = {}\nnir_page_contract = \"{}\"\nnir_page_record_count = {}\nnir_page_bytes = {}\nnir_page_projection_hash = {}\nnir_page_continuation_indentation = {}\nnir_page_continuation_body_bytes = {}\nnir_page_continuation_body_hash = {}\nnir_page_state_hash = {}\nnir_page_identity = {}\nnir_page_cursor_identity = {}\nnir_continuation_page_identity = {}\nnir_continuation_cursor_identity = {}\nreplacement_authorized = {}\nproof_sha256 = \"{}\"\n",
        proof.protocol,
        proof.producer_contract,
        proof.authority,
        proof.stage0_component_sha256,
        proof.stage0_execution_sha256,
        proof.candidate_component_sha256,
        escape_toml_string(&proof.candidate_producer_id),
        proof.candidate_compiler_image_sha256,
        proof.stage_handoff_bundle_sha256,
        escape_toml_string(&proof.stage_transformations_file),
        proof.stage_transformations_bytes,
        proof.stage_transformations_sha256,
        escape_toml_string(&proof.stage_semantic_differential_file),
        proof.stage_semantic_differential_bytes,
        proof.stage_semantic_differential_sha256,
        proof.stage_semantic_differential_proof_sha256,
        escape_toml_string(&proof.adapter_file),
        proof.adapter_bytes,
        proof.adapter_sha256,
        proof.record_count,
        proof.bundle_fold,
        proof.token_decoder_contract,
        proof.token_record_count,
        proof.token_semantic_fold,
        proof.token_page_record_count,
        proof.token_page_payload_bytes,
        proof.token_page_canonical_bytes,
        proof.token_page_canonical_hash,
        proof.token_page_identity,
        proof.token_pagination_contract,
        proof.token_page_count,
        proof.token_terminal_page_hash,
        proof.token_page_chain_identity,
        proof.projection_cursor_contract,
        proof.ast_page_contract,
        proof.ast_page_record_count,
        proof.ast_page_bytes,
        proof.ast_page_projection_hash,
        proof.ast_page_continuation_indentation,
        proof.ast_page_continuation_body_bytes,
        proof.ast_page_continuation_body_hash,
        proof.ast_page_state_hash,
        proof.ast_page_identity,
        proof.ast_page_cursor_identity,
        proof.ast_continuation_page_identity,
        proof.ast_continuation_cursor_identity,
        proof.nir_page_contract,
        proof.nir_page_record_count,
        proof.nir_page_bytes,
        proof.nir_page_projection_hash,
        proof.nir_page_continuation_indentation,
        proof.nir_page_continuation_body_bytes,
        proof.nir_page_continuation_body_hash,
        proof.nir_page_state_hash,
        proof.nir_page_identity,
        proof.nir_page_cursor_identity,
        proof.nir_continuation_page_identity,
        proof.nir_continuation_cursor_identity,
        proof.replacement_authorized,
        proof.proof_sha256,
    );
    for record in &proof.records {
        out.push_str(&format!(
            "\n[[record]]\nordinal = {}\nstage = \"{}\"\npayload_bytes = {}\npayload_sha256 = \"{}\"\nfold = {}\n",
            record.ordinal,
            record.stage,
            record.payload_bytes,
            record.payload_sha256,
            record.fold,
        ));
    }
    out
}

pub fn parse_compiler_candidate_production(
    path: &Path,
) -> Result<CompilerCandidateProduction, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler candidate production `{}`: {error}",
            path.display()
        ))
    })?;
    parse_compiler_candidate_production_from_source(&source, path)
}

pub fn parse_compiler_candidate_production_from_source(
    source: &str,
    path: &Path,
) -> Result<CompilerCandidateProduction, ArtifactError> {
    validate_text(source, path)?;
    let proof = CompilerCandidateProduction {
        protocol: parse_required_toml_string(source, "protocol", path)?,
        producer_contract: parse_required_toml_string(source, "producer_contract", path)?,
        authority: parse_required_toml_string(source, "authority", path)?,
        stage0_component_sha256: parse_required_toml_string(
            source,
            "stage0_component_sha256",
            path,
        )?,
        stage0_execution_sha256: parse_required_toml_string(
            source,
            "stage0_execution_sha256",
            path,
        )?,
        candidate_component_sha256: parse_required_toml_string(
            source,
            "candidate_component_sha256",
            path,
        )?,
        candidate_producer_id: parse_required_toml_string(source, "candidate_producer_id", path)?,
        candidate_compiler_image_sha256: parse_required_toml_string(
            source,
            "candidate_compiler_image_sha256",
            path,
        )?,
        stage_handoff_bundle_sha256: parse_required_toml_string(
            source,
            "stage_handoff_bundle_sha256",
            path,
        )?,
        stage_transformations_file: parse_required_toml_string(
            source,
            "stage_transformations_file",
            path,
        )?,
        stage_transformations_bytes: parse_required_toml_usize(
            source,
            "stage_transformations_bytes",
            path,
        )?,
        stage_transformations_sha256: parse_required_toml_string(
            source,
            "stage_transformations_sha256",
            path,
        )?,
        stage_semantic_differential_file: parse_required_toml_string(
            source,
            "stage_semantic_differential_file",
            path,
        )?,
        stage_semantic_differential_bytes: parse_required_toml_usize(
            source,
            "stage_semantic_differential_bytes",
            path,
        )?,
        stage_semantic_differential_sha256: parse_required_toml_string(
            source,
            "stage_semantic_differential_sha256",
            path,
        )?,
        stage_semantic_differential_proof_sha256: parse_required_toml_string(
            source,
            "stage_semantic_differential_proof_sha256",
            path,
        )?,
        adapter_file: parse_required_toml_string(source, "adapter_file", path)?,
        adapter_bytes: parse_required_toml_usize(source, "adapter_bytes", path)?,
        adapter_sha256: parse_required_toml_string(source, "adapter_sha256", path)?,
        record_count: parse_required_toml_usize(source, "record_count", path)?,
        bundle_fold: parse_required_toml_usize(source, "bundle_fold", path)?,
        token_decoder_contract: parse_required_toml_string(source, "token_decoder_contract", path)?,
        token_record_count: parse_required_toml_usize(source, "token_record_count", path)?,
        token_semantic_fold: parse_required_toml_usize(source, "token_semantic_fold", path)?,
        token_page_record_count: parse_required_toml_usize(
            source,
            "token_page_record_count",
            path,
        )?,
        token_page_payload_bytes: parse_required_toml_usize(
            source,
            "token_page_payload_bytes",
            path,
        )?,
        token_page_canonical_bytes: parse_required_toml_usize(
            source,
            "token_page_canonical_bytes",
            path,
        )?,
        token_page_canonical_hash: parse_required_toml_usize(
            source,
            "token_page_canonical_hash",
            path,
        )?,
        token_page_identity: parse_required_toml_usize(source, "token_page_identity", path)?,
        token_pagination_contract: parse_required_toml_string(
            source,
            "token_pagination_contract",
            path,
        )?,
        token_page_count: parse_required_toml_usize(source, "token_page_count", path)?,
        token_terminal_page_hash: parse_required_toml_usize(
            source,
            "token_terminal_page_hash",
            path,
        )?,
        token_page_chain_identity: parse_required_toml_usize(
            source,
            "token_page_chain_identity",
            path,
        )?,
        projection_cursor_contract: parse_required_toml_string(
            source,
            "projection_cursor_contract",
            path,
        )?,
        ast_page_contract: parse_required_toml_string(source, "ast_page_contract", path)?,
        ast_page_record_count: parse_required_toml_usize(source, "ast_page_record_count", path)?,
        ast_page_bytes: parse_required_toml_usize(source, "ast_page_bytes", path)?,
        ast_page_projection_hash: parse_required_toml_usize(
            source,
            "ast_page_projection_hash",
            path,
        )?,
        ast_page_continuation_indentation: parse_required_toml_usize(
            source,
            "ast_page_continuation_indentation",
            path,
        )?,
        ast_page_continuation_body_bytes: parse_required_toml_usize(
            source,
            "ast_page_continuation_body_bytes",
            path,
        )?,
        ast_page_continuation_body_hash: parse_required_toml_usize(
            source,
            "ast_page_continuation_body_hash",
            path,
        )?,
        ast_page_state_hash: parse_required_toml_usize(source, "ast_page_state_hash", path)?,
        ast_page_identity: parse_required_toml_usize(source, "ast_page_identity", path)?,
        ast_page_cursor_identity: parse_required_toml_usize(
            source,
            "ast_page_cursor_identity",
            path,
        )?,
        ast_continuation_page_identity: parse_required_toml_usize(
            source,
            "ast_continuation_page_identity",
            path,
        )?,
        ast_continuation_cursor_identity: parse_required_toml_usize(
            source,
            "ast_continuation_cursor_identity",
            path,
        )?,
        nir_page_contract: parse_required_toml_string(source, "nir_page_contract", path)?,
        nir_page_record_count: parse_required_toml_usize(source, "nir_page_record_count", path)?,
        nir_page_bytes: parse_required_toml_usize(source, "nir_page_bytes", path)?,
        nir_page_projection_hash: parse_required_toml_usize(
            source,
            "nir_page_projection_hash",
            path,
        )?,
        nir_page_continuation_indentation: parse_required_toml_usize(
            source,
            "nir_page_continuation_indentation",
            path,
        )?,
        nir_page_continuation_body_bytes: parse_required_toml_usize(
            source,
            "nir_page_continuation_body_bytes",
            path,
        )?,
        nir_page_continuation_body_hash: parse_required_toml_usize(
            source,
            "nir_page_continuation_body_hash",
            path,
        )?,
        nir_page_state_hash: parse_required_toml_usize(source, "nir_page_state_hash", path)?,
        nir_page_identity: parse_required_toml_usize(source, "nir_page_identity", path)?,
        nir_page_cursor_identity: parse_required_toml_usize(
            source,
            "nir_page_cursor_identity",
            path,
        )?,
        nir_continuation_page_identity: parse_required_toml_usize(
            source,
            "nir_continuation_page_identity",
            path,
        )?,
        nir_continuation_cursor_identity: parse_required_toml_usize(
            source,
            "nir_continuation_cursor_identity",
            path,
        )?,
        replacement_authorized: parse_required_toml_bool(source, "replacement_authorized", path)?,
        proof_sha256: parse_required_toml_string(source, "proof_sha256", path)?,
        records: parse_record_blocks(source, path)?,
    };
    validate_proof(&proof)?;
    if render_compiler_candidate_production(&proof) != source {
        return Err(ArtifactError::new(format!(
            "compiler candidate production `{}` is not canonically encoded",
            path.display()
        )));
    }
    Ok(proof)
}

pub fn read_compiler_candidate_production(
    path: &Path,
    stage0: &CompilerComponentBuild,
    execution: &CompilerCandidateExecution,
    candidate: &CompilerComponentBuild,
    handoff: &CompilerStageHandoff,
    payloads: &[VerifiedCompilerStagePayload],
) -> Result<CompilerCandidateProduction, ArtifactError> {
    let proof = parse_compiler_candidate_production(path)?;
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    let stage_transformations_path = root.join(&proof.stage_transformations_file);
    let stage_transformations_bytes = fs::read(&stage_transformations_path).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler stage transformations `{}`: {error}",
            proof.stage_transformations_file
        ))
    })?;
    if stage_transformations_bytes.len() != proof.stage_transformations_bytes
        || sha256_hex(&stage_transformations_bytes) != proof.stage_transformations_sha256
    {
        return Err(ArtifactError::new(
            "compiler stage transformations length or SHA-256 mismatch",
        ));
    }
    let stage_transformations =
        read_compiler_stage_transformations(&stage_transformations_path, handoff, payloads)?;
    let stage_semantic_differential_path = root.join(&proof.stage_semantic_differential_file);
    let stage_semantic_differential_bytes =
        fs::read(&stage_semantic_differential_path).map_err(|error| {
            ArtifactError::new(format!(
                "failed to read compiler stage semantic differential `{}`: {error}",
                proof.stage_semantic_differential_file
            ))
        })?;
    if stage_semantic_differential_bytes.len() != proof.stage_semantic_differential_bytes
        || sha256_hex(&stage_semantic_differential_bytes)
            != proof.stage_semantic_differential_sha256
    {
        return Err(ArtifactError::new(
            "compiler stage semantic differential length or SHA-256 mismatch",
        ));
    }
    let stage_semantic_input = CompilerStageSemanticDifferentialInput {
        producer_id: &candidate.producer_id,
        handoff,
        payloads,
        transformations: &stage_transformations,
    };
    let stage_semantic_differential = read_compiler_stage_semantic_differential(
        &stage_semantic_differential_path,
        &stage_semantic_input,
    )?;
    let adapter = fs::read(root.join(&proof.adapter_file)).map_err(|error| {
        ArtifactError::new(format!(
            "failed to read compiler candidate adapter `{}`: {error}",
            proof.adapter_file
        ))
    })?;
    if adapter.len() != proof.adapter_bytes || sha256_hex(&adapter) != proof.adapter_sha256 {
        return Err(ArtifactError::new(
            "compiler candidate adapter length or SHA-256 mismatch",
        ));
    }
    let stage_folds = proof
        .records
        .iter()
        .map(|record| record.fold)
        .collect::<Vec<_>>();
    let token_decode = CompilerTokenDecodeSummary {
        record_count: proof.token_record_count,
        semantic_fold: proof.token_semantic_fold,
    };
    let token_page = CompilerTokenPageIdentity {
        record_count: proof.token_page_record_count,
        payload_bytes: proof.token_page_payload_bytes,
        canonical_bytes: proof.token_page_canonical_bytes,
        canonical_hash: proof.token_page_canonical_hash,
        identity: proof.token_page_identity,
    };
    let token_payload = payloads
        .iter()
        .find(|payload| payload.stage == CompilerStageKind::Tokens)
        .ok_or_else(|| ArtifactError::new("compiler candidate token payload is missing"))?;
    let token_pagination = crate::compiler_token_pagination_identity(&token_payload.bytes)?;
    let ast_payload = payloads
        .iter()
        .find(|payload| payload.stage == CompilerStageKind::Ast)
        .ok_or_else(|| ArtifactError::new("compiler candidate AST payload is missing"))?;
    let ast_pages =
        compiler_projection_two_page_identity(CompilerProjectionKind::Ast, &ast_payload.bytes)?;
    let nir_payload = payloads
        .iter()
        .find(|payload| payload.stage == CompilerStageKind::Nir)
        .ok_or_else(|| ArtifactError::new("compiler candidate NIR payload is missing"))?;
    let nir_pages =
        compiler_projection_two_page_identity(CompilerProjectionKind::Nir, &nir_payload.bytes)?;
    let input = CompilerCandidateProductionInput {
        stage0,
        execution,
        candidate,
        handoff,
        payloads,
        stage_folds: &stage_folds,
        bundle_fold: proof.bundle_fold,
        token_decode: &token_decode,
        token_page: &token_page,
        token_pagination: &token_pagination,
        ast_pages: &ast_pages,
        nir_pages: &nir_pages,
        stage_transformations_file: &proof.stage_transformations_file,
        stage_transformations: &stage_transformations,
        stage_semantic_differential_file: &proof.stage_semantic_differential_file,
        stage_semantic_differential: &stage_semantic_differential,
        adapter_file: &proof.adapter_file,
        adapter: &adapter,
    };
    validate_evidence(&input)?;
    let rebuilt = build_compiler_candidate_production(&input)?;
    if rebuilt != proof {
        return Err(ArtifactError::new(
            "compiler candidate production does not match its bound evidence",
        ));
    }
    Ok(proof)
}

#[cfg(test)]
#[path = "compiler_candidate_production_tests.rs"]
mod tests;
