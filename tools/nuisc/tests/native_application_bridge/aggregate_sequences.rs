use super::*;
use aggregate_carried::{Case, Shape};
use aggregate_conditional::BASE;
use aggregate_loop_probe::Invocation;

pub(super) fn source(shape: Shape, count: usize, descending: bool, op: &str) -> String {
    let source = aggregate_carried::source(shape, count, descending, op)
        .replace("seed: i64) ->", "seed: i64, pivot: i64) ->")
        .replace("divisor, seed)", "divisor, seed, pivot)");
    let step = if descending { "-" } else { "+" };
    let mut original = format!("let index: i64 = index {step} stride;");
    for slot in 0..count {
        let rhs = if slot == 0 {
            "index".into()
        } else {
            format!("carry{}", slot - 1)
        };
        let op = if slot % 3 == 2 { "*" } else { "+" };
        original.push_str(&format!("let carry{slot}: i64 = carry{slot} {op} {rhs};"));
    }
    let mut body = format!(
        "let index: i64 = index {step} stride;
        let carry0: i64 = carry0 + index;
        if index < pivot || (index == pivot && index < limit) {{
            let carry0: i64 = carry0 * 2;
            let carry0: i64 = carry0 + index;"
    );
    for slot in 1..count {
        body.push_str(&format!(
            "let carry{slot}: i64 = carry{slot} + carry{};",
            slot - 1
        ));
    }
    body.push_str(
        "} else { let carry0: i64 = carry0 - index; }
        if carry0 < pivot && (index > initial || carry0 == seed) {
            let carry0: i64 = carry0 + 3;",
    );
    for slot in 1..count {
        body.push_str(&format!(
            "let carry{slot}: i64 = carry{slot} * carry{};",
            slot - 1
        ));
    }
    body.push_str("}");
    for slot in 0..count {
        let rhs = if slot == 0 {
            "index".into()
        } else {
            format!("carry{}", slot - 1)
        };
        body.push_str(&format!(
            "let carry{slot}: i64 = carry{slot} + {rhs}; let carry{slot}: i64 = carry{slot} + 1;"
        ));
    }
    assert!(source.contains(&original));
    source.replace(&original, &body)
}

pub(super) fn expected(
    case: Case,
    shape: Shape,
    count: usize,
    descending: bool,
    op: &str,
    pivot: i64,
    predicate_count: &mut i64,
) -> Invocation {
    let mut observation = Invocation {
        arguments: vec![
            Value::Bool(case.enabled),
            Value::Int(case.initial),
            Value::Int(case.limit),
            Value::Int(case.stride),
            Value::Int(case.divisor),
            Value::Int(case.seed),
            Value::Int(pivot),
        ],
        state: None,
        leaf: None,
        iterations: 0,
        reference_error: None,
    };
    let seeds = (0..count)
        .map(|slot| (i128::from(case.seed) + slot as i128) as i64)
        .collect::<Vec<_>>();
    let mut carries = seeds.clone();
    let mut index = case.initial;
    let mut trips = 0;
    if case.enabled || matches!(shape, Shape::Prefix) {
        // Independent checked-step oracle, not the native closed-form preflight.
        while if descending {
            index > case.limit
        } else {
            index < case.limit
        } {
            if trips == 65536 || case.stride <= 0 {
                return observation;
            }
            let next = if descending {
                index.checked_sub(case.stride)
            } else {
                index.checked_add(case.stride)
            };
            let Some(next) = next else {
                return observation;
            };
            index = next;
            trips += 1;
            // Independent sequential interpretation. Every arithmetic leaf wraps
            // through i128, and comparisons are counted only when evaluated.
            carries[0] = (i128::from(carries[0]) + i128::from(index)) as i64;
            let mut compare = |lhs: i64, op: &str, rhs: i64| {
                *predicate_count += 1;
                match op {
                    "<" => lhs < rhs,
                    ">" => lhs > rhs,
                    "==" => lhs == rhs,
                    _ => unreachable!(),
                }
            };
            if compare(index, "<", pivot)
                || (compare(index, "==", pivot) && compare(index, "<", case.limit))
            {
                carries[0] = (i128::from(carries[0]) * 2) as i64;
                carries[0] = (i128::from(carries[0]) + i128::from(index)) as i64;
                for slot in 1..count {
                    carries[slot] =
                        (i128::from(carries[slot]) + i128::from(carries[slot - 1])) as i64;
                }
            } else {
                carries[0] = (i128::from(carries[0]) - i128::from(index)) as i64;
            }
            if compare(carries[0], "<", pivot)
                && (compare(index, ">", case.initial) || compare(carries[0], "==", case.seed))
            {
                carries[0] = (i128::from(carries[0]) + 3) as i64;
                for slot in 1..count {
                    carries[slot] =
                        (i128::from(carries[slot]) * i128::from(carries[slot - 1])) as i64;
                }
            }
            for slot in 0..count {
                let rhs = if slot == 0 { index } else { carries[slot - 1] };
                carries[slot] = (i128::from(carries[slot]) + i128::from(rhs)) as i64;
                carries[slot] = (i128::from(carries[slot]) + 1) as i64;
            }
        }
    }
    observation.iterations = trips;
    let value = if case.enabled || matches!(shape, Shape::Prefix | Shape::Suffix) {
        let value = carries[count - 1];
        observation.leaf = Some([value, case.divisor]);
        if !op.is_empty() && (case.divisor == 0 || (value, case.divisor) == (i64::MIN, -1)) {
            observation.reference_error = Some(if case.divisor == 0 {
                "zero"
            } else {
                "overflow"
            });
            return observation;
        }
        match op {
            "/" => (i128::from(value) / i128::from(case.divisor)) as i64,
            "%" => (i128::from(value) % i128::from(case.divisor)) as i64,
            "" => value,
            _ => unreachable!(),
        }
    } else {
        case.seed
    };
    let mut state = if !case.enabled && !matches!(shape, Shape::Suffix) {
        let mut state = vec![case.seed, case.initial];
        state.extend(seeds);
        state
    } else {
        let mut state = vec![value, index];
        state.extend(carries);
        state
    };
    state.push(case.seed);
    observation.state = Some(state);
    observation
}

fn execute(shape: Shape, count: usize, descending: bool, op: &str, cases: &[(Case, i64)]) {
    let source = source(shape, count, descending, op);
    let mut counts = Vec::new();
    let observations = cases
        .iter()
        .map(|&(case, pivot)| {
            let mut predicates = 0;
            let observation = expected(case, shape, count, descending, op, pivot, &mut predicates);
            if observation.state.is_none() && observation.reference_error.is_none() {
                predicates = 0;
            }
            counts.push(predicates);
            observation
        })
        .collect::<Vec<_>>();
    aggregate_loop_probe::execute_with_predicates(
        &source,
        &observations,
        "loop_while_i64_body",
        !op.is_empty(),
        Some(&counts),
    );
}

#[test]
fn sequence_native_smoke_preserves_repeated_writes_and_branch_join() {
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
fn sequence_statements_match_native_reference_wrapping_and_comparison_counts() {
    for shape in [Shape::Callee, Shape::Inline, Shape::Prefix, Shape::Suffix] {
        for count in [1, 3, 7] {
            for descending in [false, true] {
                let mut cases = Vec::new();
                for enabled in [false, true] {
                    for trips in [0, 1, 7] {
                        for seed in [i64::MIN, -7, 0, i64::MAX] {
                            for pivot in [-2, 0, 2] {
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
fn sequence_preflight_and_arithmetic_traps_preserve_skipped_path() {
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
        assert!(expected(case, Shape::Callee, 3, false, "/", 2, &mut 0)
            .state
            .is_none());
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
                (BASE, 2),
                (case, 2),
            ],
        );
    }
}

#[test]
fn sequence_fixture_preserves_typed_lifecycle_and_declaration_order() {
    let source = include_str!("aggregate_sequences_loops.ns");
    assert_native_parity(source, false);
    assert_native_parity(source, true);
}
