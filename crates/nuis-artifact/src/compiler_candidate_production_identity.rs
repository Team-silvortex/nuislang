use sha2::{Digest, Sha256};

use super::CompilerCandidateProduction;

pub(super) fn production_identity(proof: &CompilerCandidateProduction) -> String {
    let mut hash = Sha256::new();
    for value in [
        proof.protocol.as_bytes(),
        proof.producer_contract.as_bytes(),
        proof.authority.as_bytes(),
        proof.stage0_component_sha256.as_bytes(),
        proof.stage0_execution_sha256.as_bytes(),
        proof.candidate_component_sha256.as_bytes(),
        proof.candidate_producer_id.as_bytes(),
        proof.candidate_compiler_image_sha256.as_bytes(),
        proof.stage_handoff_bundle_sha256.as_bytes(),
        proof.stage_transformations_file.as_bytes(),
        proof.stage_transformations_sha256.as_bytes(),
        proof.stage_semantic_differential_file.as_bytes(),
        proof.stage_semantic_differential_sha256.as_bytes(),
        proof.stage_semantic_differential_proof_sha256.as_bytes(),
        proof.adapter_file.as_bytes(),
        proof.adapter_sha256.as_bytes(),
        proof.token_decoder_contract.as_bytes(),
        proof.token_pagination_contract.as_bytes(),
        proof.projection_cursor_contract.as_bytes(),
        proof.ast_page_contract.as_bytes(),
        proof.nir_page_contract.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [
        proof.adapter_bytes,
        proof.stage_transformations_bytes,
        proof.stage_semantic_differential_bytes,
        proof.record_count,
        proof.bundle_fold,
        proof.token_record_count,
        proof.token_semantic_fold,
        proof.token_page_record_count,
        proof.token_page_payload_bytes,
        proof.token_page_canonical_bytes,
        proof.token_page_canonical_hash,
        proof.token_page_identity,
        proof.token_page_count,
        proof.token_terminal_page_hash,
        proof.token_page_chain_identity,
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
        usize::from(proof.replacement_authorized),
    ] {
        hash_field(&mut hash, &(value as u64).to_le_bytes());
    }
    for record in &proof.records {
        hash_field(&mut hash, &(record.ordinal as u64).to_le_bytes());
        hash_field(&mut hash, record.stage.as_bytes());
        hash_field(&mut hash, &(record.payload_bytes as u64).to_le_bytes());
        hash_field(&mut hash, record.payload_sha256.as_bytes());
        hash_field(&mut hash, &(record.fold as u64).to_le_bytes());
    }
    format!("{:x}", hash.finalize())
}

fn hash_field(hash: &mut Sha256, value: &[u8]) {
    hash.update((value.len() as u64).to_le_bytes());
    hash.update(value);
}
