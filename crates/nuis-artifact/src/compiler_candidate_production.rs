use std::{fs, path::Path};

#[path = "compiler_candidate_production_identity.rs"]
mod identity;
#[path = "compiler_candidate_production_support.rs"]
mod support;

use identity::production_identity;
use support::{
    parse_record_blocks, sha256_hex, validate_file_name, validate_sha256, validate_text,
    validate_token,
};

use crate::{
    compiler_projection_first_page_identity, compiler_token_first_page_identity,
    decode_compiler_token_stream,
    toml::{
        escape_toml_string, parse_required_toml_bool, parse_required_toml_string,
        parse_required_toml_usize,
    },
    ArtifactError, CompilerCandidateExecution, CompilerComponentBuild, CompilerProjectionKind,
    CompilerProjectionPageIdentity, CompilerStageHandoff, CompilerStageKind,
    CompilerTokenDecodeSummary, CompilerTokenPageIdentity, VerifiedCompilerStagePayload,
    COMPILER_COMPONENT_STAGE0_ROLE, COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE,
    COMPILER_PROJECTION_PAGE_BYTES, COMPILER_PROJECTION_PAGE_CONTRACT,
    COMPILER_PROJECTION_PAGE_HASH_MODULUS, COMPILER_PROJECTION_PAGE_IDENTITY_RADIX,
    COMPILER_TOKEN_DECODER_CONTRACT, COMPILER_TOKEN_DECODER_FOLD_MODULUS,
    COMPILER_TOKEN_DECODER_MAX_RECORDS, COMPILER_TOKEN_PAGE_CANONICAL_BYTES,
    COMPILER_TOKEN_PAGE_IDENTITY_RADIX, COMPILER_TOKEN_PAGE_PAYLOAD_BYTES,
    COMPILER_TOKEN_PAGE_RECORDS,
};

pub const COMPILER_CANDIDATE_PRODUCTION_PROTOCOL: &str = "nuis-compiler-candidate-production-v5";
pub const COMPILER_CANDIDATE_PRODUCER_CONTRACT: &str =
    "nuis-stage1-materialized-token-ast-nir-page-producer-v5";
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
    pub ast_page: &'a CompilerProjectionPageIdentity,
    pub nir_page: &'a CompilerProjectionPageIdentity,
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
    pub ast_page_contract: String,
    pub ast_page_record_count: usize,
    pub ast_page_bytes: usize,
    pub ast_page_projection_hash: usize,
    pub ast_page_continuation_indentation: usize,
    pub ast_page_continuation_body_bytes: usize,
    pub ast_page_continuation_body_hash: usize,
    pub ast_page_state_hash: usize,
    pub ast_page_identity: usize,
    pub nir_page_contract: String,
    pub nir_page_record_count: usize,
    pub nir_page_bytes: usize,
    pub nir_page_projection_hash: usize,
    pub nir_page_continuation_indentation: usize,
    pub nir_page_continuation_body_bytes: usize,
    pub nir_page_continuation_body_hash: usize,
    pub nir_page_state_hash: usize,
    pub nir_page_identity: usize,
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
        ast_page_contract: COMPILER_PROJECTION_PAGE_CONTRACT.to_owned(),
        ast_page_record_count: input.ast_page.record_count,
        ast_page_bytes: input.ast_page.page_bytes,
        ast_page_projection_hash: input.ast_page.projection_hash,
        ast_page_continuation_indentation: input.ast_page.continuation_indentation,
        ast_page_continuation_body_bytes: input.ast_page.continuation_body_bytes,
        ast_page_continuation_body_hash: input.ast_page.continuation_body_hash,
        ast_page_state_hash: input.ast_page.state_hash,
        ast_page_identity: input.ast_page.identity,
        nir_page_contract: COMPILER_PROJECTION_PAGE_CONTRACT.to_owned(),
        nir_page_record_count: input.nir_page.record_count,
        nir_page_bytes: input.nir_page.page_bytes,
        nir_page_projection_hash: input.nir_page.projection_hash,
        nir_page_continuation_indentation: input.nir_page.continuation_indentation,
        nir_page_continuation_body_bytes: input.nir_page.continuation_body_bytes,
        nir_page_continuation_body_hash: input.nir_page.continuation_body_hash,
        nir_page_state_hash: input.nir_page.state_hash,
        nir_page_identity: input.nir_page.identity,
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
        "protocol = \"{}\"\nproducer_contract = \"{}\"\nauthority = \"{}\"\nstage0_component_sha256 = \"{}\"\nstage0_execution_sha256 = \"{}\"\ncandidate_component_sha256 = \"{}\"\ncandidate_producer_id = \"{}\"\ncandidate_compiler_image_sha256 = \"{}\"\nstage_handoff_bundle_sha256 = \"{}\"\nadapter_file = \"{}\"\nadapter_bytes = {}\nadapter_sha256 = \"{}\"\nrecord_count = {}\nbundle_fold = {}\ntoken_decoder_contract = \"{}\"\ntoken_record_count = {}\ntoken_semantic_fold = {}\ntoken_page_record_count = {}\ntoken_page_payload_bytes = {}\ntoken_page_canonical_bytes = {}\ntoken_page_canonical_hash = {}\ntoken_page_identity = {}\nast_page_contract = \"{}\"\nast_page_record_count = {}\nast_page_bytes = {}\nast_page_projection_hash = {}\nast_page_continuation_indentation = {}\nast_page_continuation_body_bytes = {}\nast_page_continuation_body_hash = {}\nast_page_state_hash = {}\nast_page_identity = {}\nnir_page_contract = \"{}\"\nnir_page_record_count = {}\nnir_page_bytes = {}\nnir_page_projection_hash = {}\nnir_page_continuation_indentation = {}\nnir_page_continuation_body_bytes = {}\nnir_page_continuation_body_hash = {}\nnir_page_state_hash = {}\nnir_page_identity = {}\nreplacement_authorized = {}\nproof_sha256 = \"{}\"\n",
        proof.protocol,
        proof.producer_contract,
        proof.authority,
        proof.stage0_component_sha256,
        proof.stage0_execution_sha256,
        proof.candidate_component_sha256,
        escape_toml_string(&proof.candidate_producer_id),
        proof.candidate_compiler_image_sha256,
        proof.stage_handoff_bundle_sha256,
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
        proof.ast_page_contract,
        proof.ast_page_record_count,
        proof.ast_page_bytes,
        proof.ast_page_projection_hash,
        proof.ast_page_continuation_indentation,
        proof.ast_page_continuation_body_bytes,
        proof.ast_page_continuation_body_hash,
        proof.ast_page_state_hash,
        proof.ast_page_identity,
        proof.nir_page_contract,
        proof.nir_page_record_count,
        proof.nir_page_bytes,
        proof.nir_page_projection_hash,
        proof.nir_page_continuation_indentation,
        proof.nir_page_continuation_body_bytes,
        proof.nir_page_continuation_body_hash,
        proof.nir_page_state_hash,
        proof.nir_page_identity,
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
    let ast_page = CompilerProjectionPageIdentity {
        record_count: proof.ast_page_record_count,
        page_bytes: proof.ast_page_bytes,
        projection_hash: proof.ast_page_projection_hash,
        continuation_indentation: proof.ast_page_continuation_indentation,
        continuation_body_bytes: proof.ast_page_continuation_body_bytes,
        continuation_body_hash: proof.ast_page_continuation_body_hash,
        state_hash: proof.ast_page_state_hash,
        identity: proof.ast_page_identity,
    };
    let nir_page = CompilerProjectionPageIdentity {
        record_count: proof.nir_page_record_count,
        page_bytes: proof.nir_page_bytes,
        projection_hash: proof.nir_page_projection_hash,
        continuation_indentation: proof.nir_page_continuation_indentation,
        continuation_body_bytes: proof.nir_page_continuation_body_bytes,
        continuation_body_hash: proof.nir_page_continuation_body_hash,
        state_hash: proof.nir_page_state_hash,
        identity: proof.nir_page_identity,
    };
    validate_evidence(&CompilerCandidateProductionInput {
        stage0,
        execution,
        candidate,
        handoff,
        payloads,
        stage_folds: &stage_folds,
        bundle_fold: proof.bundle_fold,
        token_decode: &token_decode,
        token_page: &token_page,
        ast_page: &ast_page,
        nir_page: &nir_page,
        adapter_file: &proof.adapter_file,
        adapter: &adapter,
    })?;
    let rebuilt = build_compiler_candidate_production(&CompilerCandidateProductionInput {
        stage0,
        execution,
        candidate,
        handoff,
        payloads,
        stage_folds: &stage_folds,
        bundle_fold: proof.bundle_fold,
        token_decode: &token_decode,
        token_page: &token_page,
        ast_page: &ast_page,
        nir_page: &nir_page,
        adapter_file: &proof.adapter_file,
        adapter: &adapter,
    })?;
    if rebuilt != proof {
        return Err(ArtifactError::new(
            "compiler candidate production does not match its bound evidence",
        ));
    }
    Ok(proof)
}

fn validate_evidence(input: &CompilerCandidateProductionInput<'_>) -> Result<(), ArtifactError> {
    if input.stage0.stage_role != COMPILER_COMPONENT_STAGE0_ROLE
        || input.candidate.stage_role != COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE
        || input.stage0.producer_id == input.candidate.producer_id
    {
        return Err(ArtifactError::new(
            "compiler candidate production requires distinct stage0 and stage1-candidate producers",
        ));
    }
    if input.execution.component_record_sha256 != input.stage0.record_sha256
        || input.execution.component_reproducible_build_sha256
            != input.stage0.reproducible_build_sha256
        || input.execution.candidate_binary_sha256 != input.stage0.native_binary_sha256
        || input.execution.exit_code != 0
    {
        return Err(ArtifactError::new(
            "compiler candidate production execution does not bind the stage0 candidate image",
        ));
    }
    if input.candidate.component_id != input.stage0.component_id
        || input.candidate.component_domain != input.stage0.component_domain
        || input.candidate.component_unit != input.stage0.component_unit
        || input.candidate.bootstrap_subset_protocol != input.stage0.bootstrap_subset_protocol
        || input.candidate.compiler_image_sha256 != input.stage0.native_binary_sha256
        || input.candidate.native_binary_sha256 != input.stage0.native_binary_sha256
        || input.candidate.dependency_closure_sha256 != input.stage0.dependency_closure_sha256
    {
        return Err(ArtifactError::new(
            "compiler candidate production changed a protected source component identity",
        ));
    }
    if input.handoff.producer_id != input.candidate.producer_id
        || input.handoff.module_domain != input.candidate.component_domain
        || input.handoff.module_unit != input.candidate.component_unit
        || input.handoff.bundle_sha256 != input.candidate.stage_handoff_bundle_sha256
        || input.handoff.bundle_sha256 != input.stage0.stage_handoff_bundle_sha256
    {
        return Err(ArtifactError::new(
            "compiler candidate production handoff does not match its candidate component",
        ));
    }
    if input.handoff.records.len() != EXPECTED_STAGE_COUNT
        || input.payloads.len() != EXPECTED_STAGE_COUNT
        || input.stage_folds.len() != EXPECTED_STAGE_COUNT
    {
        return Err(ArtifactError::new(
            "compiler candidate production requires exactly five stage records",
        ));
    }
    for (ordinal, ((record, payload), fold)) in input
        .handoff
        .records
        .iter()
        .zip(input.payloads)
        .zip(input.stage_folds)
        .enumerate()
    {
        if record.ordinal != ordinal
            || record.stage != payload.stage
            || record.payload_bytes != payload.bytes.len()
            || record.payload_sha256 != sha256_hex(&payload.bytes)
            || *fold != compiler_candidate_stage_fold(ordinal, &payload.bytes)
        {
            return Err(ArtifactError::new(format!(
                "compiler candidate production stage {ordinal} fold or payload identity mismatch"
            )));
        }
    }
    if input.bundle_fold != compiler_candidate_bundle_fold(input.stage_folds) {
        return Err(ArtifactError::new(
            "compiler candidate production bundle fold mismatch",
        ));
    }
    let token_payload = input
        .payloads
        .iter()
        .find(|payload| payload.stage == CompilerStageKind::Tokens)
        .ok_or_else(|| ArtifactError::new("compiler candidate token payload is missing"))?;
    let expected_token_decode = decode_compiler_token_stream(&token_payload.bytes)?;
    if *input.token_decode != expected_token_decode {
        return Err(ArtifactError::new(
            "compiler candidate production token decode summary mismatch",
        ));
    }
    let expected_token_page = compiler_token_first_page_identity(&token_payload.bytes)?;
    if *input.token_page != expected_token_page {
        return Err(ArtifactError::new(
            "compiler candidate production canonical token page identity mismatch",
        ));
    }
    let ast_payload = input
        .payloads
        .iter()
        .find(|payload| payload.stage == CompilerStageKind::Ast)
        .ok_or_else(|| ArtifactError::new("compiler candidate AST payload is missing"))?;
    let expected_ast_page =
        compiler_projection_first_page_identity(CompilerProjectionKind::Ast, &ast_payload.bytes)?;
    if *input.ast_page != expected_ast_page {
        return Err(ArtifactError::new(
            "compiler candidate production AST structural page identity mismatch",
        ));
    }
    let nir_payload = input
        .payloads
        .iter()
        .find(|payload| payload.stage == CompilerStageKind::Nir)
        .ok_or_else(|| ArtifactError::new("compiler candidate NIR payload is missing"))?;
    let expected_nir_page =
        compiler_projection_first_page_identity(CompilerProjectionKind::Nir, &nir_payload.bytes)?;
    if *input.nir_page != expected_nir_page {
        return Err(ArtifactError::new(
            "compiler candidate production NIR structural page identity mismatch",
        ));
    }
    validate_file_name(input.adapter_file, "candidate adapter")?;
    if input.adapter.is_empty() {
        return Err(ArtifactError::new(
            "compiler candidate production adapter cannot be empty",
        ));
    }
    Ok(())
}

fn validate_proof(proof: &CompilerCandidateProduction) -> Result<(), ArtifactError> {
    if proof.protocol != COMPILER_CANDIDATE_PRODUCTION_PROTOCOL
        || proof.producer_contract != COMPILER_CANDIDATE_PRODUCER_CONTRACT
        || proof.authority != COMPILER_CANDIDATE_PRODUCTION_AUTHORITY
        || proof.token_decoder_contract != COMPILER_TOKEN_DECODER_CONTRACT
        || proof.replacement_authorized
    {
        return Err(ArtifactError::new(
            "compiler candidate production declares an unsupported authority contract",
        ));
    }
    if proof.record_count != EXPECTED_STAGE_COUNT || proof.records.len() != EXPECTED_STAGE_COUNT {
        return Err(ArtifactError::new(
            "compiler candidate production record count must be five",
        ));
    }
    for (label, value) in [
        ("stage0 component", proof.stage0_component_sha256.as_str()),
        ("stage0 execution", proof.stage0_execution_sha256.as_str()),
        (
            "candidate component",
            proof.candidate_component_sha256.as_str(),
        ),
        (
            "candidate compiler image",
            proof.candidate_compiler_image_sha256.as_str(),
        ),
        (
            "stage handoff bundle",
            proof.stage_handoff_bundle_sha256.as_str(),
        ),
        ("candidate adapter", proof.adapter_sha256.as_str()),
        ("production proof", proof.proof_sha256.as_str()),
    ] {
        validate_sha256(value, label)?;
    }
    validate_token(&proof.candidate_producer_id, "candidate producer id")?;
    validate_file_name(&proof.adapter_file, "candidate adapter")?;
    if proof.adapter_bytes == 0 || proof.bundle_fold >= FOLD_MODULUS as usize {
        return Err(ArtifactError::new(
            "compiler candidate production adapter length or bundle fold is invalid",
        ));
    }
    if proof.token_record_count > COMPILER_TOKEN_DECODER_MAX_RECORDS
        || proof.token_semantic_fold >= COMPILER_TOKEN_DECODER_FOLD_MODULUS
    {
        return Err(ArtifactError::new(
            "compiler candidate production token decode summary is invalid",
        ));
    }
    if proof.token_page_record_count != COMPILER_TOKEN_PAGE_RECORDS
        || proof.token_page_payload_bytes > COMPILER_TOKEN_PAGE_PAYLOAD_BYTES
        || proof.token_page_canonical_bytes == 0
        || proof.token_page_canonical_bytes > COMPILER_TOKEN_PAGE_CANONICAL_BYTES
        || proof.token_page_canonical_hash >= COMPILER_TOKEN_DECODER_FOLD_MODULUS
        || proof.token_page_identity
            != proof.token_page_canonical_hash * COMPILER_TOKEN_PAGE_IDENTITY_RADIX
                + proof.token_page_canonical_bytes
    {
        return Err(ArtifactError::new(
            "compiler candidate production canonical token page summary is invalid",
        ));
    }
    validate_projection_page_summary(
        "AST",
        &proof.ast_page_contract,
        CompilerProjectionPageIdentity {
            record_count: proof.ast_page_record_count,
            page_bytes: proof.ast_page_bytes,
            projection_hash: proof.ast_page_projection_hash,
            continuation_indentation: proof.ast_page_continuation_indentation,
            continuation_body_bytes: proof.ast_page_continuation_body_bytes,
            continuation_body_hash: proof.ast_page_continuation_body_hash,
            state_hash: proof.ast_page_state_hash,
            identity: proof.ast_page_identity,
        },
    )?;
    validate_projection_page_summary(
        "NIR",
        &proof.nir_page_contract,
        CompilerProjectionPageIdentity {
            record_count: proof.nir_page_record_count,
            page_bytes: proof.nir_page_bytes,
            projection_hash: proof.nir_page_projection_hash,
            continuation_indentation: proof.nir_page_continuation_indentation,
            continuation_body_bytes: proof.nir_page_continuation_body_bytes,
            continuation_body_hash: proof.nir_page_continuation_body_hash,
            state_hash: proof.nir_page_state_hash,
            identity: proof.nir_page_identity,
        },
    )?;
    for (ordinal, record) in proof.records.iter().enumerate() {
        let expected_stage = ["source", "tokens", "ast", "nir", "yir"][ordinal];
        if record.ordinal != ordinal
            || record.stage != expected_stage
            || record.payload_bytes == 0
            || record.fold >= FOLD_MODULUS as usize
        {
            return Err(ArtifactError::new(format!(
                "compiler candidate production record {ordinal} is invalid"
            )));
        }
        validate_sha256(&record.payload_sha256, "candidate stage payload")?;
    }
    if proof.proof_sha256 != production_identity(proof) {
        return Err(ArtifactError::new(
            "compiler candidate production proof identity mismatch",
        ));
    }
    Ok(())
}

fn validate_projection_page_summary(
    label: &str,
    contract: &str,
    page: CompilerProjectionPageIdentity,
) -> Result<(), ArtifactError> {
    if contract != COMPILER_PROJECTION_PAGE_CONTRACT
        || page.record_count == 0
        || page.record_count > page.page_bytes
        || page.page_bytes == 0
        || page.page_bytes > COMPILER_PROJECTION_PAGE_BYTES
        || page.projection_hash >= COMPILER_PROJECTION_PAGE_HASH_MODULUS
        || page.continuation_indentation > page.page_bytes
        || page.continuation_body_bytes > page.page_bytes
        || page.continuation_indentation + page.continuation_body_bytes > page.page_bytes
        || page.continuation_body_hash >= COMPILER_PROJECTION_PAGE_HASH_MODULUS
        || page.state_hash >= COMPILER_PROJECTION_PAGE_HASH_MODULUS
        || page.identity
            != page.state_hash * COMPILER_PROJECTION_PAGE_IDENTITY_RADIX + page.page_bytes
    {
        return Err(ArtifactError::new(format!(
            "compiler candidate production {label} structural page summary is invalid"
        )));
    }
    Ok(())
}

#[cfg(test)]
#[path = "compiler_candidate_production_tests.rs"]
mod tests;
