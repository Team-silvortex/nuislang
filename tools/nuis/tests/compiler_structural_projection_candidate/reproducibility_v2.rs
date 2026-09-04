use std::{
    fs,
    path::{Path, PathBuf},
};

use nuis_artifact::{
    parse_compiler_candidate_structural_pagination_from_source,
    parse_compiler_candidate_structural_pagination_result_bytes,
    read_compiler_component_representation_differential, read_compiler_component_reproducibility,
    read_compiler_component_reproducibility_v2, COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_FILE,
    COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_FILE, COMPILER_COMPONENT_BUILD_FILE,
    COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_FILE,
    COMPILER_COMPONENT_REPRODUCIBILITY_V2_FILE,
};

pub(super) fn assert_bound_selected_representations(
    output_dir: &Path,
    output_dir_text: &str,
    predecessor: &Path,
    roots: &[PathBuf; 2],
) -> PathBuf {
    let successor = output_dir.join(COMPILER_COMPONENT_REPRODUCIBILITY_V2_FILE);
    let report = read_compiler_component_reproducibility_v2(&successor, predecessor, roots)
        .expect("verify selected-representation reproducibility successor");
    assert_eq!(report.run_count, 2);
    assert_eq!(report.representation_comparison_count, 4);
    assert_eq!(report.equivalent_representation_count, 4);
    assert!(report.all_selected_representations_equivalent);
    assert!(report.sidecars_individually_bound);
    assert!(report.predecessor_signature_compatible);
    assert!(!report.replacement_authorized);
    assert_ne!(
        report.runs[0].representation_differential_sha256,
        report.runs[1].representation_differential_sha256
    );
    assert_ne!(
        report.runs[0].representation_report_sha256,
        report.runs[1].representation_report_sha256
    );

    for (root, run) in roots.iter().zip(&report.runs) {
        let sidecar_path = root.join(COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_FILE);
        let sidecar = read_compiler_component_representation_differential(
            &sidecar_path,
            &root.join("stage0").join(COMPILER_COMPONENT_BUILD_FILE),
            &root
                .join("stage1-candidate")
                .join(COMPILER_COMPONENT_BUILD_FILE),
        )
        .expect("verify root representation sidecar");
        assert_eq!(run.representation_report_sha256, sidecar.report_sha256);
        assert!(!fs::read_to_string(sidecar_path)
            .expect("read representation sidecar")
            .contains(output_dir_text));
    }
    let source = fs::read_to_string(&successor).expect("read reproducibility v2 source");
    assert!(!source.contains(output_dir_text));
    assert_structural_pagination_is_reproducible(roots, output_dir_text);
    successor
}

fn assert_structural_pagination_is_reproducible(roots: &[PathBuf; 2], output_dir_text: &str) {
    let mut proofs = Vec::new();
    let mut proof_sources = Vec::new();
    let mut result_sources = Vec::new();
    for root in roots {
        let candidate = root.join("stage1-candidate");
        let proof_path = candidate.join(COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_FILE);
        let proof_source = fs::read_to_string(&proof_path).expect("read pagination proof");
        let proof =
            parse_compiler_candidate_structural_pagination_from_source(&proof_source, &proof_path)
                .expect("parse canonical pagination proof");
        assert_eq!(proof.page_count, 3);
        assert_eq!(proof.projections.len(), 2);
        assert!(proof.candidate_owned_pagination);
        assert!(proof.host_recomputed);
        assert!(proof.predecessor_unchanged);
        assert!(!proof.stage0_provider_dependency);
        assert!(!proof.replacement_authorized);
        assert!(!proof.selection_authorized);
        assert!(proof
            .projections
            .iter()
            .all(|projection| projection.third_page_identity > 0));
        assert!(!proof_source.contains(output_dir_text));

        let result_path = candidate.join(COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_FILE);
        let result_source = fs::read(&result_path).expect("read pagination result");
        let result = parse_compiler_candidate_structural_pagination_result_bytes(
            &result_source,
            &result_path,
        )
        .expect("parse canonical pagination result");
        assert_eq!(result.page_count, 3);
        assert_eq!(result.ast_pages.len(), 3);
        assert_eq!(result.nir_pages.len(), 3);
        assert!(!String::from_utf8_lossy(&result_source).contains(output_dir_text));
        proofs.push(proof);
        proof_sources.push(proof_source);
        result_sources.push(result_source);
    }
    assert_ne!(proof_sources[0], proof_sources[1]);
    assert_ne!(
        proofs[0].candidate_component_sha256,
        proofs[1].candidate_component_sha256
    );
    assert_ne!(
        proofs[0].predecessor_proof_sha256,
        proofs[1].predecessor_proof_sha256
    );
    assert_eq!(
        proofs[0].stage_handoff_bundle_sha256,
        proofs[1].stage_handoff_bundle_sha256
    );
    assert_eq!(proofs[0].adapter_sha256, proofs[1].adapter_sha256);
    assert_eq!(proofs[0].result_sha256, proofs[1].result_sha256);
    assert_eq!(proofs[0].projections, proofs[1].projections);
    assert_eq!(result_sources[0], result_sources[1]);
}

pub(super) fn assert_sidecar_tampering_fails(
    successor: &Path,
    predecessor: &Path,
    roots: &[PathBuf; 2],
) {
    let sidecar = roots[1].join(COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_FILE);
    let original = fs::read(&sidecar).expect("read second representation sidecar");
    let mut tampered = original.clone();
    tampered.push(b'\n');
    fs::write(&sidecar, tampered).expect("tamper second representation sidecar");
    let error = read_compiler_component_reproducibility_v2(successor, predecessor, roots)
        .expect_err("sidecar tampering must invalidate reproducibility v2");
    assert!(error.to_string().contains("not canonically encoded"));
    fs::write(&sidecar, original).expect("restore second representation sidecar");
    read_compiler_component_reproducibility(predecessor, roots)
        .expect("v1 predecessor remains verifiable after v2 sidecar rejection");
}
