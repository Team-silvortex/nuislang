use super::*;
use aggregate_carried::{Case, Shape};
use aggregate_conditional::BASE;

// Preserve the independently interpreted sequence program's output, but retain
// real i64/bool local values across later writes. Re-substitution is incorrect.
fn source(shape: Shape, count: usize, descending: bool, op: &str) -> String {
    let mut text = aggregate_sequences::source(shape, count, descending, op);
    let old = "let carry0: i64 = carry0 + index;\n        if index < pivot || (index == pivot && index < limit) {";
    let new = "let doubled = index * 2;
        let snapshot = doubled;
        let doubled: i64 = doubled + 1;
        let delta = (snapshot + 1) - doubled + index;
        let carry0: i64 = carry0 + delta;
        let saved = carry0;
        let carry0: i64 = carry0 + 5;
        let carry0: i64 = saved;
        let gate = index < pivot || (index == pivot && index < limit);
        let suffix_delta = index;
        if gate {
            let suffix_delta: i64 = suffix_delta + 1;";
    assert!(text.contains(old));
    text = text.replace(old, new);
    let old = "let carry0: i64 = carry0 + index;";
    // Every occurrence is scope-local and reinitialized, including the suffix.
    text = text.replace(old, "let local = index; let carry0: i64 = carry0 + local;");
    let old = "} else { let carry0: i64 = carry0 - index; }";
    assert!(text.contains(old));
    text = text.replace(
        old,
        "} else { let local: i64 = index; let carry0: i64 = carry0 - local; let suffix_delta: i64 = suffix_delta - 1; }
        if gate == true { let suffix_delta: i64 = suffix_delta - 1; } else { let suffix_delta: i64 = suffix_delta + 1; }",
    );
    let old = "if carry0 < pivot && (index > initial || carry0 == seed) {\n            let carry0: i64 = carry0 + 3;";
    assert!(text.contains(old));
    text = text.replace(
        old,
        "let chosen: bool = carry0 < pivot && (index > initial || carry0 == seed);
        let carry0: i64 = carry0 + 1;
        if chosen {
            let carry0: i64 = carry0 + 2;",
    );
    // The sequence's second IF is followed by its ordered suffix. Restore the
    // unselected path only; a re-evaluated bool would pick the wrong arm.
    let old = "}let carry0: i64 = carry0 + local;";
    // The suffix replacement above declares local first.
    let suffix = "}let local = index; let carry0: i64 = carry0 + local;";
    assert!(!text.contains(old));
    assert!(text.contains(suffix));
    text.replace(suffix, "} else { let carry0: i64 = carry0 - 1; }let local = suffix_delta; let carry0: i64 = carry0 + local;")
}

fn execute(shape: Shape, count: usize, descending: bool, op: &str, cases: &[(Case, i64)]) {
    let mut counts = Vec::new();
    let observations = cases
        .iter()
        .map(|&(case, pivot)| {
            let mut predicates = 0;
            let observation = aggregate_sequences::expected(
                case,
                shape,
                count,
                descending,
                op,
                pivot,
                &mut predicates,
            );
            if observation.state.is_none() && observation.reference_error.is_none() {
                predicates = 0;
            }
            counts.push(predicates);
            observation
        })
        .collect::<Vec<_>>();
    aggregate_loop_probe::execute_with_predicates(
        &source(shape, count, descending, op),
        &observations,
        "loop_while_i64_body",
        !op.is_empty(),
        Some(&counts),
    );
}

#[test]
fn temporary_snapshots_preserve_values_and_short_circuit_counts() {
    execute(
        Shape::Callee,
        3,
        false,
        "/",
        &[
            (BASE, 2),
            (
                Case {
                    enabled: false,
                    stride: 0,
                    ..BASE
                },
                0,
            ),
        ],
    );
}

#[test]
fn typed_and_inferred_temporaries_match_reference_across_shapes_and_extremes() {
    for shape in [Shape::Callee, Shape::Inline, Shape::Prefix, Shape::Suffix] {
        for count in [1, 3, 7] {
            for descending in [false, true] {
                let mut cases = Vec::new();
                for enabled in [false, true] {
                    for trips in [0, 1, 4] {
                        for seed in [i64::MIN, -3, i64::MAX] {
                            for pivot in [0, 2] {
                                cases.push((
                                    Case {
                                        enabled,
                                        initial: if descending { 3 } else { -3 },
                                        limit: if descending {
                                            3 - trips * 2
                                        } else {
                                            -3 + trips * 2
                                        },
                                        stride: 2,
                                        seed,
                                        ..BASE
                                    },
                                    pivot,
                                ));
                            }
                        }
                    }
                }
                execute(
                    shape,
                    count,
                    descending,
                    if descending { "%" } else { "/" },
                    &cases,
                );
            }
        }
    }
}

#[test]
fn temporary_paths_keep_native_preflight_traps_and_skipped_branches() {
    for case in [
        Case { stride: 0, ..BASE },
        Case { stride: -1, ..BASE },
        Case {
            initial: i64::MAX - 1,
            limit: i64::MAX,
            stride: 2,
            ..BASE
        },
        Case {
            limit: 65537,
            ..BASE
        },
        Case { divisor: 0, ..BASE },
    ] {
        execute(
            Shape::Callee,
            3,
            false,
            "/",
            &[
                (
                    Case {
                        enabled: false,
                        ..case
                    },
                    2,
                ),
                (case, 2),
            ],
        );
    }
}
