use sha2::{Digest, Sha256};

use super::CompilerComponentRepresentationDifferential;

pub(super) fn representation_differential_identity(
    report: &CompilerComponentRepresentationDifferential,
) -> String {
    let mut hash = Sha256::new();
    for value in [
        report.protocol.as_bytes(),
        report.comparison_contract.as_bytes(),
        report.authority.as_bytes(),
        report.component_id.as_bytes(),
        report.base_differential_file.as_bytes(),
        report.base_differential_report_sha256.as_bytes(),
        report.stage0_handoff_bundle_sha256.as_bytes(),
        report.candidate_handoff_bundle_sha256.as_bytes(),
        report.candidate_handoff_v2_protocol.as_bytes(),
        report.candidate_handoff_v2_proof_sha256.as_bytes(),
        report.verdict.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [
        report.comparison_count,
        report.equivalent_count,
        usize::from(report.all_representations_equivalent),
        usize::from(report.replacement_authorized),
    ] {
        hash_usize(&mut hash, value);
    }
    for comparison in &report.comparisons {
        for value in [
            comparison.ordinal,
            comparison.selection_ordinal,
            comparison.transformation_ordinal,
            comparison.semantic_comparison_ordinal,
            comparison.base_comparison_ordinal,
            comparison.candidate_selected_payload_bytes,
            usize::from(comparison.byte_identical),
            usize::from(comparison.reversible),
            usize::from(comparison.semantically_equivalent),
            usize::from(comparison.equivalent),
        ] {
            hash_usize(&mut hash, value);
        }
        for value in [
            comparison.subject.as_bytes(),
            comparison.source_stage.as_str().as_bytes(),
            comparison.stage0_encoding.as_bytes(),
            comparison.stage0_record_sha256.as_bytes(),
            comparison.stage0_payload_sha256.as_bytes(),
            comparison.candidate_source_encoding.as_bytes(),
            comparison.candidate_source_record_sha256.as_bytes(),
            comparison.candidate_source_payload_sha256.as_bytes(),
            comparison.candidate_selected_encoding.as_bytes(),
            comparison.candidate_selected_payload_file.as_bytes(),
            comparison.candidate_selected_payload_sha256.as_bytes(),
            comparison.candidate_recovered_payload_sha256.as_bytes(),
            comparison.transform_contract.as_bytes(),
            comparison.checkpoint_sha256.as_bytes(),
        ] {
            hash_field(&mut hash, value);
        }
    }
    format!("{:x}", hash.finalize())
}

fn hash_usize(hash: &mut Sha256, value: usize) {
    hash_field(hash, &(value as u64).to_le_bytes());
}

fn hash_field(hash: &mut Sha256, value: &[u8]) {
    hash.update((value.len() as u64).to_le_bytes());
    hash.update(value);
}
