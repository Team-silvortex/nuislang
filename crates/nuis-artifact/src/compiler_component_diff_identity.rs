use sha2::{Digest, Sha256};

use super::CompilerComponentDifferential;

const VALUE_IDENTITY_CONTRACT: &str = "nuis-compiler-differential-value-v1";

pub(super) fn value_identity(subject: &str, value: &[u8]) -> String {
    sha256_fields(&[
        VALUE_IDENTITY_CONTRACT.as_bytes(),
        subject.as_bytes(),
        value,
    ])
}

pub(super) fn differential_report_identity(report: &CompilerComponentDifferential) -> String {
    let mut hash = Sha256::new();
    for value in [
        report.protocol.as_bytes(),
        report.gate_contract.as_bytes(),
        report.replacement_authority_contract.as_bytes(),
        report.component_id.as_bytes(),
        report.stage0_producer_id.as_bytes(),
        report.candidate_producer_id.as_bytes(),
        report.stage0_record_sha256.as_bytes(),
        report.candidate_record_sha256.as_bytes(),
        report.stage0_compiler_image_sha256.as_bytes(),
        report.candidate_compiler_image_sha256.as_bytes(),
        report.verdict.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [
        report.comparison_count,
        report.equivalent_count,
        usize::from(report.stage_equivalent),
        usize::from(report.diagnostics_equivalent),
        usize::from(report.dependency_closure_equivalent),
        usize::from(report.native_output_equivalent),
        usize::from(report.deterministic_artifact_equivalent),
        usize::from(report.replacement_authorized),
    ] {
        hash_field(&mut hash, &(value as u64).to_le_bytes());
    }
    for comparison in &report.comparisons {
        hash_field(&mut hash, &(comparison.ordinal as u64).to_le_bytes());
        hash_field(&mut hash, comparison.subject.as_bytes());
        hash_field(&mut hash, comparison.stage0_sha256.as_bytes());
        hash_field(&mut hash, comparison.candidate_sha256.as_bytes());
        hash_field(&mut hash, &[u8::from(comparison.equivalent)]);
    }
    finish_hash(hash)
}

fn sha256_fields(fields: &[&[u8]]) -> String {
    let mut hash = Sha256::new();
    for field in fields {
        hash_field(&mut hash, field);
    }
    finish_hash(hash)
}

fn hash_field(hash: &mut Sha256, value: &[u8]) {
    hash.update((value.len() as u64).to_le_bytes());
    hash.update(value);
}

fn finish_hash(hash: Sha256) -> String {
    let digest = hash.finalize();
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}
