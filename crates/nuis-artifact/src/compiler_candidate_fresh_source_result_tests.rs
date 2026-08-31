use super::*;

#[test]
fn canonical_snapshot_reaches_five_candidate_owned_stage_identities() {
    let result =
        build_compiler_candidate_fresh_source_result(COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT)
            .expect("compile canonical fresh-source snapshot");
    assert_eq!(result.stages.len(), 5);
    assert_eq!(result.stages[1].record_count, 16);
    assert!(result
        .stages
        .windows(2)
        .all(|pair| pair[0].identity != pair[1].identity));
    assert!(result.candidate_owned_source_processing);
    assert!(result.fresh_source_compile);
    assert!(!result.stage0_handoff_required);

    let source = render_compiler_candidate_fresh_source_result(&result);
    let parsed = parse_compiler_candidate_fresh_source_result_from_source(
        &source,
        Path::new(COMPILER_CANDIDATE_FRESH_SOURCE_RESULT_FILE),
    )
    .expect("parse canonical fresh-source result");
    assert_eq!(parsed, result);
}

#[test]
fn source_or_result_drift_fails_closed() {
    let mut source = COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT.to_vec();
    source[0] = b'x';
    assert!(build_compiler_candidate_fresh_source_result(&source).is_err());

    let result =
        build_compiler_candidate_fresh_source_result(COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT)
            .expect("compile canonical fresh-source snapshot");
    let rendered = render_compiler_candidate_fresh_source_result(&result);
    for tampered in [
        rendered.replacen("fresh_source_compile=true", "fresh_source_compile=false", 1),
        rendered.replacen("stage.3=nir,6,", "stage.3=nir,5,", 1),
        rendered.trim_end().to_owned(),
    ] {
        assert!(parse_compiler_candidate_fresh_source_result_from_source(
            &tampered,
            Path::new(COMPILER_CANDIDATE_FRESH_SOURCE_RESULT_FILE),
        )
        .is_err());
    }
}
