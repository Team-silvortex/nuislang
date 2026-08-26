use sha2::{Digest, Sha256};

use super::CompilerComponentReproducibility;

pub(super) fn reproducibility_identity(report: &CompilerComponentReproducibility) -> String {
    let mut hash = Sha256::new();
    for value in [
        report.protocol.as_bytes(),
        report.clean_build_contract.as_bytes(),
        report.attestation_authority.as_bytes(),
        report.replacement_authority_contract.as_bytes(),
        report.component_id.as_bytes(),
        report.stage0_reproducible_build_sha256.as_bytes(),
        report.candidate_reproducible_build_sha256.as_bytes(),
        report.candidate_compiler_image_sha256.as_bytes(),
        report.native_output_sha256.as_bytes(),
        report.differential_verdict.as_bytes(),
        report.verdict.as_bytes(),
    ] {
        hash_field(&mut hash, value);
    }
    for value in [
        report.run_count,
        report.comparison_count,
        report.equivalent_run_count,
        usize::from(report.all_runs_equivalent),
        usize::from(report.replacement_authorized),
    ] {
        hash_field(&mut hash, &(value as u64).to_le_bytes());
    }
    for run in &report.runs {
        for value in [
            run.run_id.as_bytes(),
            run.component_id.as_bytes(),
            run.clean_root_state.as_bytes(),
            run.clean_root_witness_sha256.as_bytes(),
            run.stage0_record_sha256.as_bytes(),
            run.stage0_reproducible_build_sha256.as_bytes(),
            run.candidate_record_sha256.as_bytes(),
            run.candidate_reproducible_build_sha256.as_bytes(),
            run.candidate_compiler_image_sha256.as_bytes(),
            run.native_output_sha256.as_bytes(),
            run.production_proof_sha256.as_bytes(),
            run.differential_report_sha256.as_bytes(),
            run.differential_verdict.as_bytes(),
        ] {
            hash_field(&mut hash, value);
        }
        for value in [
            run.ordinal,
            run.comparison_count,
            run.equivalent_count,
            usize::from(run.deterministic_artifact_equivalent),
            usize::from(run.replacement_authorized),
        ] {
            hash_field(&mut hash, &(value as u64).to_le_bytes());
        }
    }
    let digest = hash.finalize();
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn hash_field(hash: &mut Sha256, value: &[u8]) {
    hash.update((value.len() as u64).to_le_bytes());
    hash.update(value);
}
