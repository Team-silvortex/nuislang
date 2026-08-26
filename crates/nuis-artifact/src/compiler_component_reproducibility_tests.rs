use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use super::*;

fn hash(byte: char) -> String {
    std::iter::repeat_n(byte, 64).collect()
}

fn run(ordinal: usize, witness: char) -> CompilerComponentReproducibilityRun {
    CompilerComponentReproducibilityRun {
        ordinal,
        run_id: format!("clean-build-{ordinal}"),
        component_id: "projection_relay".to_owned(),
        clean_root_state: CLEAN_ROOT_STATE.to_owned(),
        clean_root_witness_sha256: hash(witness),
        stage0_record_sha256: hash(if ordinal == 0 { '1' } else { '2' }),
        stage0_reproducible_build_sha256: hash('3'),
        candidate_record_sha256: hash(if ordinal == 0 { '4' } else { '5' }),
        candidate_reproducible_build_sha256: hash('6'),
        candidate_compiler_image_sha256: hash('7'),
        native_output_sha256: hash('8'),
        production_proof_sha256: hash(if ordinal == 0 { '9' } else { 'a' }),
        differential_report_sha256: hash(if ordinal == 0 { 'b' } else { 'c' }),
        comparison_count: EXPECTED_COMPARISON_COUNT,
        equivalent_count: EXPECTED_COMPARISON_COUNT,
        deterministic_artifact_equivalent: true,
        differential_verdict: EQUIVALENT_VERDICT.to_owned(),
        replacement_authorized: false,
    }
}

fn report() -> CompilerComponentReproducibility {
    build_from_runs(vec![run(0, 'd'), run(1, 'e')]).expect("build reproducibility aggregate")
}

#[test]
fn clean_build_aggregate_round_trips_without_paths_or_authority() {
    let report = report();
    assert_eq!(report.run_count, 2);
    assert_eq!(report.equivalent_run_count, 2);
    assert!(report.all_runs_equivalent);
    assert!(!report.replacement_authorized);
    assert_eq!(report.verdict, REPRODUCIBLE_VERDICT);
    assert_eq!(report.runs[0].stage0_record_sha256, hash('1'));
    assert_eq!(report.runs[1].stage0_record_sha256, hash('2'));
    assert_eq!(
        report.runs[0].candidate_reproducible_build_sha256,
        report.runs[1].candidate_reproducible_build_sha256
    );

    let source = render_compiler_component_reproducibility(&report);
    assert!(!source.contains("/Users/"));
    assert!(!source.contains("output_dir"));
    let parsed = parse_compiler_component_reproducibility_from_source(
        &source,
        Path::new(COMPILER_COMPONENT_REPRODUCIBILITY_FILE),
    )
    .expect("parse aggregate");
    assert_eq!(parsed, report);
    assert_eq!(render_compiler_component_reproducibility(&parsed), source);
}

#[test]
fn duplicate_witness_or_reproducible_identity_drift_fails_closed() {
    let error = build_from_runs(vec![run(0, 'd'), run(1, 'd')])
        .expect_err("duplicate clean witness must fail");
    assert!(error.to_string().contains("witnesses must be distinct"));

    let first = run(0, 'd');
    let mut second = run(1, 'e');
    second.candidate_reproducible_build_sha256 = hash('f');
    let error = build_from_runs(vec![first, second])
        .expect_err("candidate reproducible identity drift must fail");
    assert!(error
        .to_string()
        .contains("candidate reproducible build drifted"));
}

#[test]
fn aggregate_authority_and_identity_tampering_fail_closed() {
    let source = render_compiler_component_reproducibility(&report());
    let authority_tamper = source.replacen(
        "replacement_authorized = false",
        "replacement_authorized = true",
        1,
    );
    let error = parse_compiler_component_reproducibility_from_source(
        &authority_tamper,
        Path::new(COMPILER_COMPONENT_REPRODUCIBILITY_FILE),
    )
    .expect_err("replacement authority must remain false");
    assert!(error.to_string().contains("aggregate verdict mismatch"));

    let identity_tamper = source.replacen(&hash('6'), &hash('f'), 1);
    let error = parse_compiler_component_reproducibility_from_source(
        &identity_tamper,
        Path::new(COMPILER_COMPONENT_REPRODUCIBILITY_FILE),
    )
    .expect_err("stable candidate identity tampering must fail");
    assert!(error.to_string().contains("stable aggregate identity"));
}

#[test]
fn reader_rejects_two_names_for_the_same_build_root() {
    let root = temp_dir();
    let report_path = root.join(COMPILER_COMPONENT_REPRODUCIBILITY_FILE);
    fs::write(
        &report_path,
        render_compiler_component_reproducibility(&report()),
    )
    .expect("write aggregate");
    let roots = vec![root.clone(), root.clone()];
    let error = read_compiler_component_reproducibility(&report_path, &roots)
        .expect_err("aliased roots must fail before evidence loading");
    assert!(error.to_string().contains("two distinct build roots"));
    fs::remove_dir_all(root).expect("remove aggregate test root");
}

fn temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock follows epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("nuis_reproducibility_{nonce}"));
    fs::create_dir_all(&root).expect("create aggregate test root");
    root
}
