use super::*;
use aggregate_carried::{Case, Shape};
use aggregate_conditional::{Fixture, BASE};

#[derive(Clone, Copy)]
enum Arm {
    OmittedElse,
    EmptyElse,
    EmptyThen,
    Mixed,
}

fn source(fixture: Fixture, arm: Arm) -> String {
    assert!(fixture.keep_first_else);
    let mut source = fixture.source();
    for slot in 0..fixture.count {
        if slot > 0 && slot % 2 == 0 {
            continue;
        }
        let rhs = if slot == 0 {
            "index".into()
        } else {
            format!("carry{}", slot - 1)
        };
        let predicate = |compare| {
            if fixture.reversed {
                format!("pivot {compare} {rhs}")
            } else {
                format!("{rhs} {compare} pivot")
            }
        };
        let op = if slot % 3 == 2 { "*" } else { "+" };
        let update = format!("let carry{slot}: i64 = carry{slot} {op} {rhs};");
        let original = format!(
            "if {} {{ {update} }} else {{ let carry{slot}: i64 = carry{slot}; }}",
            predicate(fixture.compare)
        );
        assert!(source.contains(&original));
        let selected = if matches!(arm, Arm::Mixed) {
            [Arm::OmittedElse, Arm::EmptyElse, Arm::EmptyThen][slot % 3]
        } else {
            arm
        };
        let replacement = match selected {
            Arm::OmittedElse => format!("if {} {{ {update} }}", predicate(fixture.compare)),
            Arm::EmptyElse => format!("if {} {{ {update} }} else {{}}", predicate(fixture.compare)),
            Arm::EmptyThen => {
                let inverse = match fixture.compare {
                    "==" => "!=",
                    "!=" => "==",
                    "<" => ">=",
                    "<=" => ">",
                    ">" => "<=",
                    ">=" => "<",
                    _ => unreachable!(),
                };
                format!("if {} {{}} else {{ {update} }}", predicate(inverse))
            }
            Arm::Mixed => unreachable!(),
        };
        source = source.replace(&original, &replacement);
    }
    source
}

fn execute(fixture: Fixture, arm: Arm, cases: &[(Case, i64)]) {
    aggregate_loop_probe::execute(
        &source(fixture, arm),
        &cases
            .iter()
            .map(|&(case, pivot)| fixture.expected(case, pivot))
            .collect::<Vec<_>>(),
        "loop_while_scalar_cond_chain_body",
        !fixture.op.is_empty(),
    );
}

#[test]
fn one_sided_carries_preserve_own_state_and_source_order_in_real_native_execution() {
    for (position, shape) in [Shape::Callee, Shape::Inline, Shape::Prefix, Shape::Suffix]
        .into_iter()
        .enumerate()
    {
        for descending in [false, true] {
            for op in ["/", "%"] {
                let fixture = Fixture {
                    shape,
                    descending,
                    op,
                    keep_first_else: true,
                    ..Fixture::default()
                };
                let mut cases = Vec::new();
                for enabled in [false, true] {
                    for trips in [0, 1, 7] {
                        for stride in [1, 2] {
                            for seed in [i64::MIN, -7, 0, i64::MAX] {
                                for pivot in [-2, 0, 2] {
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
                assert_eq!(cases.len(), 144);
                execute(
                    fixture,
                    [Arm::OmittedElse, Arm::EmptyElse, Arm::EmptyThen, Arm::Mixed][position],
                    &cases,
                );
            }
        }
    }
}

#[test]
fn one_sided_comparisons_reversed_operands_and_variable_widths_execute() {
    for (position, compare) in ["==", "!=", "<", "<=", ">", ">="].into_iter().enumerate() {
        for reversed in [false, true] {
            let fixture = Fixture {
                count: if position % 2 == 0 { 1 } else { 7 },
                descending: reversed,
                compare,
                reversed,
                op: "",
                keep_first_else: true,
                ..Fixture::default()
            };
            let mut cases = Vec::new();
            for pivot in [i64::MIN, -2, 0, 2, i64::MAX] {
                for seed in [-1, 1, i64::MAX] {
                    cases.push((
                        Case {
                            limit: if reversed { -7 } else { 7 },
                            seed,
                            ..BASE
                        },
                        pivot,
                    ));
                }
            }
            execute(
                fixture,
                if reversed { Arm::EmptyThen } else { Arm::Mixed },
                &cases,
            );
        }
    }
}

#[test]
fn one_sided_preflight_and_checked_leaf_traps_do_not_speculate_skipped_calls() {
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
    for (position, failure) in failures.into_iter().enumerate() {
        let fixture = Fixture {
            shape: if position % 2 == 0 {
                Shape::Callee
            } else {
                Shape::Inline
            },
            op: if position % 2 == 0 { "/" } else { "%" },
            keep_first_else: true,
            ..Fixture::default()
        };
        assert!(fixture.expected(failure, 2).state.is_none());
        execute(
            fixture,
            Arm::Mixed,
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
    // Prefix work still executes before a false guard; suffix arithmetic is
    // unconditional even when its loop body is skipped.
    for (shape, failure) in [(Shape::Prefix, failures[0]), (Shape::Suffix, failures[4])] {
        execute(
            Fixture {
                shape,
                keep_first_else: true,
                ..Fixture::default()
            },
            Arm::EmptyThen,
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
        let fixture = Fixture {
            descending,
            op: "",
            keep_first_else: true,
            ..Fixture::default()
        };
        execute(
            fixture,
            Arm::OmittedElse,
            &[
                (
                    Case {
                        limit: if descending { -65536 } else { 65536 },
                        ..BASE
                    },
                    2,
                ),
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
        Arm::EmptyElse,
        &[(
            Case {
                initial: i64::MIN + 1,
                limit: i64::MIN,
                stride: 2,
                ..BASE
            },
            0,
        )],
    );
}

#[test]
fn one_sided_flat_branch_helpers_compose_with_typed_lifecycle() {
    assert_native_parity(include_str!("aggregate_one_sided_loops.ns"), true);
}

#[test]
fn one_sided_reference_fuel_failure_retains_accepted_state_and_cleanup() {
    aggregate_conditional::assert_reference_fuel_failure(include_str!(
        "aggregate_one_sided_loops.ns"
    ));
}
