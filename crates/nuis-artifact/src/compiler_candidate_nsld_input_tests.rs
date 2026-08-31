use std::path::Path;

use super::{
    build_compiler_candidate_nsld_input, parse_compiler_candidate_nsld_input_from_source,
    render_compiler_candidate_nsld_input, COMPILER_CANDIDATE_NSLD_ENTRY_SYMBOL,
};
use crate::COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT;

#[test]
fn canonical_fresh_source_builds_candidate_owned_nsld_input() {
    let input = build_compiler_candidate_nsld_input(COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT)
        .expect("build canonical candidate Nsld input");

    assert_eq!(input.entry_symbol, COMPILER_CANDIDATE_NSLD_ENTRY_SYMBOL);
    assert_eq!(input.source_identity, 12_832_741_133);
    assert_eq!(input.yir_identity, 9_279_238_763);
    assert_eq!(input.entry_symbol_identity, 1_040_689_614);
    assert_eq!(input.materialization_fold, 1_403_051_547);
    assert_eq!(input.return_value, 7);
    assert!(input.candidate_owned_yir_materialization);
    assert!(input.equivalent_nsld_input);
    assert!(!input.native_object);
    assert!(!input.stage0_handoff_required);
    assert!(!input.provider_dependency_required);
    assert!(!input.replacement_authorized);
    assert!(!input.selection_authorized);

    let source = render_compiler_candidate_nsld_input(&input);
    let parsed = parse_compiler_candidate_nsld_input_from_source(
        &source,
        Path::new("candidate-nsld-input.toml"),
    )
    .expect("parse canonical candidate Nsld input");
    assert_eq!(parsed, input);
}

#[test]
fn candidate_nsld_input_rejects_semantic_and_authority_drift() {
    let input = build_compiler_candidate_nsld_input(COMPILER_CANDIDATE_FRESH_SOURCE_SNAPSHOT)
        .expect("build canonical candidate Nsld input");
    let source = render_compiler_candidate_nsld_input(&input);
    for damaged in [
        source.replacen("return_value=7", "return_value=8", 1),
        source.replacen(
            "replacement_authorized=false",
            "replacement_authorized=true",
            1,
        ),
        source.replacen(
            "target_selector=registered-native-cpu",
            "target_selector=mach-o-arm64",
            1,
        ),
    ] {
        assert!(parse_compiler_candidate_nsld_input_from_source(
            &damaged,
            Path::new("damaged-candidate-nsld-input.toml"),
        )
        .is_err());
    }
}
