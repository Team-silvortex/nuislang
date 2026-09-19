use super::*;
use aggregate_carried::{Case, Shape};
use aggregate_conditional::{Fixture, BASE};
use aggregate_nested_tree::{Gate, Tree};

fn execute(
    fixture: Fixture,
    tree: &Tree,
    empty_keep: bool,
    omit_else: bool,
    cases: &[(Case, i64)],
) {
    let mut counts = Vec::new();
    let observations = cases
        .iter()
        .map(|&(case, pivot)| {
            let mut count = 0;
            let result = fixture.expected_with_predicate(case, pivot, &mut |state| {
                tree.evaluate(state, case, pivot, &mut count)
            });
            if result.state.is_none() && result.reference_error.is_none() {
                count = 0;
            }
            counts.push(count);
            result
        })
        .collect::<Vec<_>>();
    aggregate_loop_probe::execute_with_predicates(
        &tree.source(fixture, empty_keep, omit_else),
        &observations,
        "loop_while_scalar_cond_chain_body",
        !fixture.op.is_empty(),
        Some(&counts),
    );
}

#[test]
fn nested_carry_paths_match_original_decision_oracle_and_actual_comparison_counts() {
    let trees = [
        Tree::and(Gate::Above, Tree::test(Gate::Below)),
        Tree::or(Gate::Above, Tree::test(Gate::Below)),
        Tree::and(Gate::Any, Tree::or(Gate::Below, Tree::test(Gate::Unequal))),
        Tree::or(Gate::Above, Tree::and(Gate::All, Tree::test(Gate::Equal))),
        Tree::branch(
            Gate::Above,
            Tree::Leaf(false),
            Tree::branch(Gate::Below, Tree::Leaf(false), Tree::Leaf(true)),
        ),
        Tree::chain(5),
    ];
    for (shape_index, shape) in [Shape::Callee, Shape::Inline, Shape::Prefix, Shape::Suffix]
        .into_iter()
        .enumerate()
    {
        for (tree_index, tree) in trees.iter().enumerate() {
            let descending = tree_index % 2 == 1;
            let fixture = Fixture {
                shape,
                descending,
                op: if shape_index % 2 == 0 { "/" } else { "%" },
                keep_first_else: tree_index % 2 == 0,
                ..Fixture::default()
            };
            let mut cases = Vec::new();
            for enabled in [false, true] {
                for trips in [0, 1, 7] {
                    for stride in [1, 2] {
                        for seed in [-7, 0, i64::MAX] {
                            for pivot in [-2, 2] {
                                cases.push((
                                    Case {
                                        enabled,
                                        limit: if descending {
                                            -trips * stride
                                        } else {
                                            trips * stride
                                        },
                                        stride,
                                        seed,
                                        ..BASE
                                    },
                                    pivot,
                                ));
                            }
                        }
                    }
                }
            }
            assert_eq!(cases.len(), 72);
            execute(
                fixture,
                tree,
                shape_index != 0,
                shape_index % 2 == 0,
                &cases,
            );
        }
    }
}

#[test]
fn nested_updates_preserve_variable_widths_and_dependent_carry_values() {
    for count in [1, 7] {
        for descending in [false, true] {
            let fixture = Fixture {
                count,
                descending,
                op: "",
                keep_first_else: true,
                ..Fixture::default()
            };
            let mut cases = Vec::new();
            for seed in [-1, 1, i64::MAX] {
                for pivot in [i64::MIN, -2, 2, i64::MAX] {
                    cases.push((
                        Case {
                            seed,
                            limit: if descending { -7 } else { 7 },
                            ..BASE
                        },
                        pivot,
                    ));
                }
            }
            execute(fixture, &Tree::chain(8), true, descending, &cases);
        }
    }
}

#[test]
fn nested_updates_retain_preflight_selected_traps_and_exact_trip_limit() {
    let failures = [
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
        Case {
            limit: 0,
            divisor: -1,
            seed: i64::MAX - 1,
            ..BASE
        },
    ];
    let tree = Tree::and(Gate::Any, Tree::or(Gate::Below, Tree::test(Gate::Unequal)));
    for (index, failure) in failures.into_iter().enumerate() {
        let fixture = Fixture {
            shape: if index % 2 == 0 {
                Shape::Callee
            } else {
                Shape::Inline
            },
            op: if index % 2 == 0 { "/" } else { "%" },
            keep_first_else: true,
            ..Fixture::default()
        };
        execute(
            fixture,
            &tree,
            true,
            true,
            &[
                (
                    Case {
                        enabled: false,
                        ..failure
                    },
                    2,
                ),
                (BASE, 2),
                (failure, 2),
            ],
        );
    }
    for (shape, failure) in [(Shape::Prefix, failures[0]), (Shape::Suffix, failures[4])] {
        execute(
            Fixture {
                shape,
                keep_first_else: true,
                ..Fixture::default()
            },
            &tree,
            true,
            false,
            &[(
                Case {
                    enabled: false,
                    ..failure
                },
                2,
            )],
        );
    }
    for descending in [false, true] {
        execute(
            Fixture {
                descending,
                op: "",
                keep_first_else: true,
                ..Fixture::default()
            },
            &Tree::chain(16),
            true,
            true,
            &[
                (
                    Case {
                        limit: if descending { -131072 } else { 131072 },
                        stride: 2,
                        ..BASE
                    },
                    2,
                ),
                (
                    Case {
                        limit: 0,
                        stride: 0,
                        ..BASE
                    },
                    2,
                ),
            ],
        );
    }
    execute(
        Fixture {
            descending: true,
            op: "",
            keep_first_else: true,
            ..Fixture::default()
        },
        &tree,
        true,
        true,
        &[(
            Case {
                initial: i64::MIN + 1,
                limit: i64::MIN,
                stride: 2,
                ..BASE
            },
            2,
        )],
    );
}

#[test]
fn nested_tree_admission_retains_backend_owned_condition_budgets() {
    let fixture = Fixture {
        count: 1,
        op: "",
        keep_first_else: true,
        ..Fixture::default()
    };
    let tree = Tree::chain(32);
    let mut count = 0;
    assert!(tree.evaluate(1, BASE, 0, &mut count));
    assert_eq!(count, 32);
    execute(
        fixture,
        &tree,
        true,
        true,
        &[
            (BASE, 0),
            (
                Case {
                    enabled: false,
                    ..BASE
                },
                2,
            ),
        ],
    );
    for depth in [33, 40] {
        let project = Project::with_source(&Tree::chain(depth).source(fixture, true, true));
        let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
        let error = emit_registered(&module, "counter").unwrap_err();
        assert!(
            error.contains("conditional-carry shape/bound"),
            "{depth}: {error}"
        );
    }
}

#[test]
fn nested_aggregate_updates_compose_with_typed_lifecycle() {
    assert_native_parity(include_str!("aggregate_nested_loops.ns"), true);
}

#[test]
fn nested_aggregate_reference_fuel_failure_retains_state_and_cleanup() {
    aggregate_conditional::assert_reference_fuel_failure(include_str!("aggregate_nested_loops.ns"));
}
