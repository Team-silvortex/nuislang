use std::path::Path;

use super::*;
use crate::{
    compiler_component_reproducibility::build_from_runs as build_v1_from_runs,
    CompilerComponentReproducibilityRun,
};

fn hash(byte: char) -> String {
    byte.to_string().repeat(64)
}

fn predecessor_run(ordinal: usize, witness: char) -> CompilerComponentReproducibilityRun {
    CompilerComponentReproducibilityRun {
        ordinal,
        run_id: format!("clean-build-{ordinal}"),
        component_id: "projection_relay".to_owned(),
        clean_root_state: "absent-or-empty-before-build".to_owned(),
        clean_root_witness_sha256: hash(witness),
        stage0_record_sha256: hash(if ordinal == 0 { '1' } else { '2' }),
        stage0_reproducible_build_sha256: hash('3'),
        candidate_record_sha256: hash(if ordinal == 0 { '4' } else { '5' }),
        candidate_reproducible_build_sha256: hash('6'),
        candidate_compiler_image_sha256: hash('7'),
        native_output_sha256: hash('8'),
        production_proof_sha256: hash(if ordinal == 0 { '9' } else { 'a' }),
        differential_report_sha256: hash('b'),
        comparison_count: 13,
        equivalent_count: 13,
        deterministic_artifact_equivalent: true,
        differential_verdict: "equivalent-awaiting-authorization".to_owned(),
        replacement_authorized: false,
    }
}

fn predecessor() -> CompilerComponentReproducibility {
    build_v1_from_runs(vec![predecessor_run(0, 'c'), predecessor_run(1, 'd')])
        .expect("build v1 predecessor")
}

fn run(
    predecessor: &CompilerComponentReproducibility,
    ordinal: usize,
) -> CompilerComponentReproducibilityV2Run {
    CompilerComponentReproducibilityV2Run {
        ordinal,
        run_id: format!("clean-build-{ordinal}"),
        clean_root_witness_sha256: predecessor.runs[ordinal].clean_root_witness_sha256.clone(),
        production_proof_sha256: predecessor.runs[ordinal].production_proof_sha256.clone(),
        base_differential_report_sha256: predecessor.runs[ordinal]
            .differential_report_sha256
            .clone(),
        representation_differential_file: COMPILER_COMPONENT_REPRESENTATION_DIFFERENTIAL_FILE
            .to_owned(),
        representation_differential_bytes: 512,
        representation_differential_sha256: hash(if ordinal == 0 { 'e' } else { '0' }),
        representation_report_sha256: hash(if ordinal == 0 { 'f' } else { '1' }),
        representation_comparison_count: 2,
        representation_equivalent_count: 2,
        all_representations_equivalent: true,
        replacement_authorized: false,
    }
}

fn report() -> CompilerComponentReproducibilityV2 {
    let predecessor = predecessor();
    let predecessor_source = render_compiler_component_reproducibility(&predecessor);
    build_from_runs(
        &predecessor,
        &predecessor_source,
        vec![run(&predecessor, 0), run(&predecessor, 1)],
    )
    .expect("build reproducibility v2")
}

#[test]
fn successor_round_trips_without_mutating_v1_authority() {
    let report = report();
    assert_eq!(report.run_count, 2);
    assert_eq!(report.representation_comparison_count, 4);
    assert_eq!(report.equivalent_representation_count, 4);
    assert!(report.sidecars_individually_bound);
    assert!(report.predecessor_signature_compatible);
    assert!(!report.replacement_authorized);

    let source = render_compiler_component_reproducibility_v2(&report);
    let parsed = parse_compiler_component_reproducibility_v2_from_source(
        &source,
        Path::new(COMPILER_COMPONENT_REPRODUCIBILITY_V2_FILE),
    )
    .expect("parse canonical reproducibility v2");
    assert_eq!(parsed, report);
    assert!(!source.contains("/Users/"));
}

#[test]
fn incomplete_or_predecessor_run_drift_fails_closed() {
    let predecessor = predecessor();
    let predecessor_source = render_compiler_component_reproducibility(&predecessor);
    let first = run(&predecessor, 0);
    let mut second = run(&predecessor, 1);
    second.representation_differential_bytes = 0;
    let error = build_from_runs(&predecessor, &predecessor_source, vec![first, second])
        .expect_err("empty sidecar evidence must fail");
    assert!(error.to_string().contains("run contract mismatch"));

    let first = run(&predecessor, 0);
    let mut second = run(&predecessor, 1);
    second.base_differential_report_sha256 = hash('0');
    let error = build_from_runs(&predecessor, &predecessor_source, vec![first, second])
        .expect_err("predecessor run drift must fail");
    assert!(error.to_string().contains("v1 predecessor runs"));
}

#[test]
fn authority_and_aggregate_tampering_fail_closed() {
    let source = render_compiler_component_reproducibility_v2(&report());
    for damaged in [
        source.replacen(
            "replacement_authorized = false",
            "replacement_authorized = true",
            1,
        ),
        source.replacen(
            "predecessor_signature_compatible = true",
            "predecessor_signature_compatible = false",
            1,
        ),
    ] {
        assert!(parse_compiler_component_reproducibility_v2_from_source(
            &damaged,
            Path::new(COMPILER_COMPONENT_REPRODUCIBILITY_V2_FILE),
        )
        .is_err());
    }
}
