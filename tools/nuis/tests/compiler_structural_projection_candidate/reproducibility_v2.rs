use std::{
    fs,
    path::{Path, PathBuf},
};

use nuis_artifact::{
    read_compiler_component_representation_differential, read_compiler_component_reproducibility,
    read_compiler_component_reproducibility_v2, COMPILER_COMPONENT_BUILD_FILE,
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
    successor
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
