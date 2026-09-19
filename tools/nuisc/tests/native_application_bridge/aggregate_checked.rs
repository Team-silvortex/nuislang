use super::*;
use aggregate_carried::{Case, Shape};
use aggregate_loop_probe::Invocation;

#[derive(Clone, Copy, Debug)]
pub(super) enum Body {
    Direct,
    Temporary,
    Branch,
    ShortCircuit,
    Late,
    Unused,
}

pub(super) fn source(shape: Shape, count: usize, descending: bool, op: &str, body: Body) -> String {
    let mut text = aggregate_carried::source(shape, count, descending, "");
    for slot in 0..count {
        let carry = format!("carry{slot}");
        let rhs = if slot == 0 {
            "index".into()
        } else {
            format!("carry{}", slot - 1)
        };
        let update = if slot % 3 == 2 { "*" } else { "+" };
        let old = format!("let {carry}: i64 = {carry} {update} {rhs};");
        let replacement = match body {
            Body::Direct => format!("let {carry}: i64 = {carry} {op} divisor;"),
            Body::Temporary => format!("
                let {carry}: i64 = {carry};
                let local{slot} = {carry} {op} divisor;
                let saved{slot}: i64 = local{slot};
                let local{slot}: i64 = local{slot} + 1;
                let {carry}: i64 = saved{slot} + {rhs};"),
            Body::Branch => format!("
                if index < 2 {{
                    let local{slot}: i64 = (seed + {slot}) {op} divisor;
                    let {carry}: i64 = {carry} + local{slot};
                }} else {{ let {carry}: i64 = {carry} + index; }}"),
            Body::ShortCircuit => format!("
                let gate{slot} = divisor == 0 || (seed == ({} - 1) && divisor == -1) || seed {op} divisor < index;
                let exact{slot}: bool = divisor != 0 && (seed != ({} - 1) || divisor != -1) && seed {op} divisor == index;
                if gate{slot} || exact{slot} {{ let {carry}: i64 = {carry} + index; }}
                else {{ let {carry}: i64 = {carry} - index; }}", i64::MIN + 1, i64::MIN + 1),
            Body::Late => format!("
                let divisor{slot} = divisor - index;
                let local{slot} = seed {op} divisor{slot};
                let {carry}: i64 = {carry} + local{slot};"),
            Body::Unused => format!("
                let unused{slot} = seed {op} divisor;
                let {carry}: i64 = {carry} + {rhs};"),
        };
        assert!(text.contains(&old));
        text = text.replace(&old, &replacement);
    }
    text
}

fn checked(lhs: i64, rhs: i64, op: &str) -> Result<i64, &'static str> {
    if rhs == 0 {
        return Err("zero");
    }
    // Nuis rejects MIN % -1 as well as MIN / -1, even though i128 can represent both.
    if (lhs, rhs) == (i64::MIN, -1) {
        return Err("overflow");
    }
    Ok(match op {
        "/" => (i128::from(lhs) / i128::from(rhs)) as i64,
        "%" => (i128::from(lhs) % i128::from(rhs)) as i64,
        _ => unreachable!(),
    })
}

fn expected(
    case: Case,
    shape: Shape,
    count: usize,
    descending: bool,
    op: &str,
    body: Body,
) -> Invocation {
    expected_with_calls(case, shape, count, descending, op, body, &mut Vec::new())
}

pub(super) fn expected_with_calls(
    case: Case,
    shape: Shape,
    count: usize,
    descending: bool,
    op: &str,
    body: Body,
    calls: &mut Vec<[i64; 3]>,
) -> Invocation {
    let mut result = Invocation {
        arguments: vec![
            Value::Bool(case.enabled),
            Value::Int(case.initial),
            Value::Int(case.limit),
            Value::Int(case.stride),
            Value::Int(case.divisor),
            Value::Int(case.seed),
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
    let mut indices = Vec::new();
    let mut index = case.initial;
    if case.enabled || matches!(shape, Shape::Prefix) {
        // Validate the entire induction independently before evaluating any body.
        // A later step overflow or excess trip count must trap at iteration zero.
        while if descending {
            index > case.limit
        } else {
            index < case.limit
        } {
            if indices.len() == 65536 || case.stride <= 0 {
                return result;
            }
            let next = if descending {
                index.checked_sub(case.stride)
            } else {
                index.checked_add(case.stride)
            };
            let Some(next) = next else {
                return result;
            };
            index = next;
            indices.push(index);
        }
    }
    for &index in &indices {
        result.iterations += 1;
        for slot in 0..count {
            let prior = carries[slot];
            let rhs = if slot == 0 { index } else { carries[slot - 1] };
            let add = |value: i64| (i128::from(prior) + i128::from(value)) as i64;
            let mut check = |left, right| {
                calls.push([left, right, result.iterations]);
                checked(left, right, op)
            };
            let value = match body {
                Body::Direct => check(prior, case.divisor),
                Body::Temporary => check(prior, case.divisor)
                    .map(|saved| (i128::from(saved) + i128::from(rhs)) as i64),
                Body::Branch if index < 2 => check(seeds[slot], case.divisor).map(add),
                Body::Branch => Ok(add(index)),
                Body::ShortCircuit => {
                    let danger = case.divisor == 0 || (case.seed, case.divisor) == (i64::MIN, -1);
                    let gate = danger || check(case.seed, case.divisor).unwrap() < index;
                    let exact = !danger && check(case.seed, case.divisor).unwrap() == index;
                    Ok(if gate || exact {
                        add(index)
                    } else {
                        (i128::from(prior) - i128::from(index)) as i64
                    })
                }
                Body::Late => check(
                    case.seed,
                    (i128::from(case.divisor) - i128::from(index)) as i64,
                )
                .map(add),
                Body::Unused => check(case.seed, case.divisor).map(|_| add(rhs)),
            };
            match value {
                Ok(value) => carries[slot] = value,
                Err(error) => {
                    result.reference_error = Some(error);
                    return result;
                }
            }
        }
    }
    if case.enabled || matches!(shape, Shape::Prefix | Shape::Suffix) {
        result.leaf = Some([carries[count - 1], case.divisor]);
    }
    let mut state = if !case.enabled && !matches!(shape, Shape::Suffix) {
        let mut state = vec![case.seed, case.initial];
        state.extend(seeds);
        state
    } else {
        let mut state = vec![carries[count - 1], index];
        state.extend(carries);
        state
    };
    state.push(case.seed);
    result.state = Some(state);
    result
}

fn execute(shape: Shape, count: usize, descending: bool, op: &str, body: Body, cases: &[Case]) {
    let observations = cases
        .iter()
        .map(|&case| expected(case, shape, count, descending, op, body))
        .collect::<Vec<_>>();
    aggregate_loop_probe::execute(
        &source(shape, count, descending, op, body),
        &observations,
        "loop_while_i64_body",
        true,
    );
}

const BASE: Case = Case {
    enabled: true,
    initial: 0,
    limit: 4,
    stride: 1,
    divisor: 3,
    seed: -17,
};

#[test]
fn checked_iteration_arithmetic_matches_independent_oracle_across_call_shapes() {
    for (shape, body, count) in [
        (Shape::Callee, Body::Direct, 1),
        (Shape::Inline, Body::Temporary, 3),
        (Shape::Prefix, Body::Direct, 7),
        (Shape::Suffix, Body::Temporary, 3),
    ] {
        for (op, descending) in [("/", false), ("%", true)] {
            let mut cases = Vec::new();
            for enabled in [false, true] {
                for trips in [0, 1, 4] {
                    for seed in [i64::MIN, -17, 0, i64::MAX] {
                        for divisor in [-3, 2] {
                            cases.push(Case {
                                enabled,
                                initial: if descending { 4 } else { 0 },
                                limit: if descending { 4 - trips } else { trips },
                                seed,
                                divisor,
                                ..BASE
                            });
                        }
                    }
                }
            }
            execute(shape, count, descending, op, body, &cases);
        }
    }
}

#[test]
fn checked_iteration_short_circuit_skips_zero_and_signed_overflow() {
    for op in ["/", "%"] {
        execute(
            Shape::Callee,
            1,
            false,
            op,
            Body::ShortCircuit,
            &[
                BASE,
                Case { divisor: 0, ..BASE },
                Case {
                    divisor: -1,
                    seed: i64::MIN,
                    ..BASE
                },
                Case {
                    divisor: -1,
                    seed: i64::MIN + 1,
                    ..BASE
                },
                Case {
                    seed: 3,
                    divisor: 3,
                    ..BASE
                },
                Case {
                    seed: 8,
                    divisor: 3,
                    ..BASE
                },
                Case {
                    seed: 11,
                    divisor: -2,
                    ..BASE
                },
            ],
        );
    }
}

#[test]
fn checked_iteration_unselected_arms_and_zero_trips_do_not_evaluate_arithmetic() {
    for op in ["/", "%"] {
        for shape in [Shape::Callee, Shape::Inline, Shape::Prefix, Shape::Suffix] {
            execute(
                shape,
                1,
                false,
                op,
                Body::Branch,
                &[
                    Case {
                        initial: 2,
                        divisor: 0,
                        ..BASE
                    },
                    Case {
                        initial: 2,
                        divisor: -1,
                        seed: i64::MIN,
                        ..BASE
                    },
                    Case {
                        initial: 4,
                        stride: 0,
                        divisor: 0,
                        ..BASE
                    },
                    BASE,
                ],
            );
        }
    }
}

#[test]
fn checked_iteration_faults_trap_at_the_reached_iteration_even_when_unused() {
    for op in ["/", "%"] {
        for body in [Body::Direct, Body::Branch, Body::Unused] {
            for case in [
                Case { divisor: 0, ..BASE },
                Case {
                    divisor: -1,
                    seed: i64::MIN,
                    ..BASE
                },
            ] {
                execute(
                    Shape::Callee,
                    1,
                    false,
                    op,
                    body,
                    &[
                        Case {
                            enabled: false,
                            ..case
                        },
                        Case {
                            limit: 0,
                            stride: 0,
                            ..case
                        },
                        BASE,
                        case,
                    ],
                );
            }
        }
        execute(
            Shape::Suffix,
            3,
            false,
            op,
            Body::Late,
            &[Case { divisor: 2, ..BASE }],
        );
    }
}

#[test]
fn checked_iteration_preflight_rejects_whole_bound_before_any_division() {
    for case in [
        Case { stride: 0, ..BASE },
        Case { stride: -1, ..BASE },
        Case {
            limit: 65537,
            ..BASE
        },
        Case {
            initial: i64::MAX - 3,
            limit: i64::MAX,
            stride: 2,
            ..BASE
        },
    ] {
        let case = Case { divisor: 0, ..case };
        execute(
            Shape::Callee,
            1,
            false,
            "/",
            Body::Direct,
            &[
                Case {
                    enabled: false,
                    ..case
                },
                case,
            ],
        );
    }
    execute(
        Shape::Callee,
        1,
        true,
        "%",
        Body::Direct,
        &[Case {
            initial: i64::MIN + 3,
            limit: i64::MIN,
            stride: 2,
            divisor: 0,
            ..BASE
        }],
    );
}

#[test]
fn checked_iteration_call_guards_preserve_prefix_and_suffix_order() {
    for op in ["/", "%"] {
        execute(
            Shape::Suffix,
            1,
            false,
            op,
            Body::Direct,
            &[
                Case {
                    enabled: false,
                    divisor: 0,
                    ..BASE
                },
                Case {
                    enabled: false,
                    divisor: -1,
                    seed: i64::MIN,
                    ..BASE
                },
                BASE,
                Case { divisor: 0, ..BASE },
            ],
        );
        execute(
            Shape::Prefix,
            1,
            false,
            op,
            Body::Direct,
            &[Case {
                enabled: false,
                divisor: 0,
                ..BASE
            }],
        );
    }
}

#[test]
fn reference_checked_iteration_failure_preserves_accepted_state_and_close() {
    for op in ["/", "%"] {
        let text = source(Shape::Callee, 1, false, op, Body::Late).replace(
            "fn step(state: State) -> State { return state; }",
            "fn step(state: State, divisor: i64) -> State {
                let result: Parts = compute(true, 0, 4, 1, divisor, state.seed);
                return State { value: result.value, index: result.index,
                    carry0: result.carry0, seed: result.seed };
            }",
        );
        let project = Project::with_source(&text);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        emit_registered(&compiled.yir, "counter").unwrap();
        let registry = yir_verify::default_registry();
        let (mut session, _) = ApplicationSession::open_registered(
            &compiled.yir,
            &registry,
            "counter",
            vec![
                Value::Bool(true),
                Value::Int(0),
                Value::Int(4),
                Value::Int(1),
                Value::Int(6),
                Value::Int(17),
            ],
        )
        .unwrap();
        session.event(vec![Value::Int(5)]).unwrap();
        let accepted = session.state().clone();
        let error = session.event(vec![Value::Int(2)]).unwrap_err();
        assert!(error.contains("zero"), "{error}");
        assert_eq!(session.state(), &accepted);
        session.close(vec![]).unwrap();
        assert!(session.completion_status().is_err());
    }
}
