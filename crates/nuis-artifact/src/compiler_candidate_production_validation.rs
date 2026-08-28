use super::{
    compiler_candidate_bundle_fold, compiler_candidate_stage_fold,
    identity::production_identity,
    support::{
        sha256_hex, validate_file_name, validate_projection_chain_summary,
        validate_projection_page_summary, validate_sha256, validate_token,
    },
    CompilerCandidateProduction, CompilerCandidateProductionInput,
    COMPILER_CANDIDATE_PRODUCER_CONTRACT, COMPILER_CANDIDATE_PRODUCTION_AUTHORITY,
    COMPILER_CANDIDATE_PRODUCTION_PROTOCOL, EXPECTED_STAGE_COUNT, FOLD_MODULUS,
};
use crate::{
    compiler_projection_two_page_identity, compiler_token_first_page_identity,
    decode_compiler_token_stream, verify_compiler_stage_transformations, ArtifactError,
    CompilerProjectionKind, CompilerProjectionPageIdentity, CompilerStageKind,
    COMPILER_COMPONENT_STAGE0_ROLE, COMPILER_COMPONENT_STAGE1_CANDIDATE_ROLE,
    COMPILER_PROJECTION_CURSOR_CONTRACT, COMPILER_STAGE_TRANSFORMATION_FILE,
    COMPILER_TOKEN_DECODER_CONTRACT, COMPILER_TOKEN_DECODER_FOLD_MODULUS,
    COMPILER_TOKEN_DECODER_MAX_RECORDS, COMPILER_TOKEN_PAGE_CANONICAL_BYTES,
    COMPILER_TOKEN_PAGE_IDENTITY_RADIX, COMPILER_TOKEN_PAGE_PAYLOAD_BYTES,
    COMPILER_TOKEN_PAGE_RECORDS,
};

pub(super) fn validate_evidence(
    input: &CompilerCandidateProductionInput<'_>,
) -> Result<(), ArtifactError> {
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
    if *input.token_decode != decode_compiler_token_stream(&token_payload.bytes)? {
        return Err(ArtifactError::new(
            "compiler candidate production token decode summary mismatch",
        ));
    }
    if *input.token_page != compiler_token_first_page_identity(&token_payload.bytes)? {
        return Err(ArtifactError::new(
            "compiler candidate production canonical token page identity mismatch",
        ));
    }
    validate_projection_evidence(input, CompilerStageKind::Ast, CompilerProjectionKind::Ast)?;
    validate_projection_evidence(input, CompilerStageKind::Nir, CompilerProjectionKind::Nir)?;
    validate_file_name(input.stage_transformations_file, "stage transformations")?;
    if input.stage_transformations_file != COMPILER_STAGE_TRANSFORMATION_FILE {
        return Err(ArtifactError::new(
            "compiler candidate production requires the canonical stage transformations file",
        ));
    }
    verify_compiler_stage_transformations(
        input.stage_transformations,
        input.handoff,
        input.payloads,
    )?;
    if input.stage_transformations.producer_id != input.candidate.producer_id {
        return Err(ArtifactError::new(
            "compiler candidate production stage transformations do not bind its candidate producer",
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

fn validate_projection_evidence(
    input: &CompilerCandidateProductionInput<'_>,
    stage: CompilerStageKind,
    kind: CompilerProjectionKind,
) -> Result<(), ArtifactError> {
    let payload = input
        .payloads
        .iter()
        .find(|payload| payload.stage == stage)
        .ok_or_else(|| {
            ArtifactError::new(format!(
                "compiler candidate {} payload is missing",
                stage.as_str().to_ascii_uppercase()
            ))
        })?;
    let expected = compiler_projection_two_page_identity(kind, &payload.bytes)?;
    let actual = match kind {
        CompilerProjectionKind::Ast => input.ast_pages,
        CompilerProjectionKind::Nir => input.nir_pages,
    };
    if *actual != expected {
        return Err(ArtifactError::new(format!(
            "compiler candidate production {} structural page chain mismatch",
            stage.as_str().to_ascii_uppercase()
        )));
    }
    Ok(())
}

pub(super) fn validate_proof(proof: &CompilerCandidateProduction) -> Result<(), ArtifactError> {
    if proof.protocol != COMPILER_CANDIDATE_PRODUCTION_PROTOCOL
        || proof.producer_contract != COMPILER_CANDIDATE_PRODUCER_CONTRACT
        || proof.authority != COMPILER_CANDIDATE_PRODUCTION_AUTHORITY
        || proof.token_decoder_contract != COMPILER_TOKEN_DECODER_CONTRACT
        || proof.projection_cursor_contract != COMPILER_PROJECTION_CURSOR_CONTRACT
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
        (
            "stage transformations",
            proof.stage_transformations_sha256.as_str(),
        ),
        ("candidate adapter", proof.adapter_sha256.as_str()),
        ("production proof", proof.proof_sha256.as_str()),
    ] {
        validate_sha256(value, label)?;
    }
    validate_token(&proof.candidate_producer_id, "candidate producer id")?;
    validate_file_name(&proof.stage_transformations_file, "stage transformations")?;
    validate_file_name(&proof.adapter_file, "candidate adapter")?;
    if proof.stage_transformations_file != COMPILER_STAGE_TRANSFORMATION_FILE
        || proof.stage_transformations_bytes == 0
        || proof.adapter_bytes == 0
        || proof.bundle_fold >= FOLD_MODULUS as usize
    {
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
    validate_projection_summaries(proof)?;
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

fn validate_projection_summaries(proof: &CompilerCandidateProduction) -> Result<(), ArtifactError> {
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
    validate_projection_chain_summary(
        "AST",
        &proof.projection_cursor_contract,
        proof.ast_page_cursor_identity,
        proof.ast_continuation_page_identity,
        proof.ast_continuation_cursor_identity,
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
    validate_projection_chain_summary(
        "NIR",
        &proof.projection_cursor_contract,
        proof.nir_page_cursor_identity,
        proof.nir_continuation_page_identity,
        proof.nir_continuation_cursor_identity,
    )
}
