use super::*;

#[derive(Clone, Copy, Debug)]
enum Shape {
    Callee,
    Inline,
    Prefix,
    Suffix,
}

#[derive(Clone, Copy, Debug)]
struct Case {
    enabled: bool,
    initial: i64,
    limit: i64,
    stride: i64,
    divisor: i64,
}

fn source(shape: Shape, compare: &str, step: &str, op: &str) -> String {
    let loop_body =
        format!("while index {compare} limit {{ let index: i64 = index {step} stride; }}");
    let arithmetic = if op.is_empty() {
        "value".into()
    } else {
        format!("value {op} divisor")
    };
    let calculate = format!(
        "let index: i64 = initial; {loop_body}
        return Parts {{ value: leaf(index, divisor), seed: seed }};"
    );
    let fallback = "Parts { value: seed, seed: seed }";
    let call = "calculate(initial, limit, stride, divisor, seed)";
    let body = match shape {
        Shape::Callee => format!("if enabled {{ return {call}; }} return {fallback};"),
        Shape::Inline => format!("if enabled {{ {calculate} }} return {fallback};"),
        Shape::Prefix => {
            format!("let saved: Parts = {call}; if enabled {{ return saved; }} return {fallback};")
        }
        Shape::Suffix => format!(
            "let index: i64 = initial;
            if enabled {{ {loop_body} }}
            return Parts {{ value: leaf(index, divisor), seed: seed }};"
        ),
    };
    format!("mod cpu Main {{
      struct State {{ value: i64, seed: i64 }}
      struct Parts {{ value: i64, seed: i64 }}
      @noinline fn leaf(value: i64, divisor: i64) -> i64 {{ return {arithmetic}; }}
      @noinline fn calculate(initial: i64, limit: i64, stride: i64, divisor: i64, seed: i64) -> Parts {{ {calculate} }}
      @noinline fn compute(enabled: bool, initial: i64, limit: i64, stride: i64, divisor: i64, seed: i64) -> Parts {{ {body} }}
      fn start(enabled: bool, initial: i64, limit: i64, stride: i64, divisor: i64, seed: i64) -> State {{
        let result: Parts = compute(enabled, initial, limit, stride, divisor, seed);
        return State {{ value: result.value, seed: result.seed }};
      }}
      fn step(state: State) -> State {{ return state; }}
      fn stop(state: State) -> State {{ return state; }}
      fn main() -> i64 {{ print(999); return 0; }}
    }}")
}

// Independent checked-step oracle, deliberately not the preflight's closed form.
fn induction(case: Case, compare: &str, step: &str) -> Option<(i64, i64)> {
    let mut index = case.initial;
    for trips in 0..=65536 {
        let active = match compare {
            "<" => index < case.limit,
            "<=" => index <= case.limit,
            ">" => index > case.limit,
            _ => unreachable!(),
        };
        if !active {
            return Some((index, trips));
        }
        index = if step == "+" {
            index.checked_add(case.stride)?
        } else {
            index.checked_sub(case.stride)?
        };
    }
    None
}

// Err(None) is preflight rejection; Err(Some(...)) reached the arithmetic leaf.
fn expected(
    case: Case,
    shape: Shape,
    compare: &str,
    step: &str,
    op: &str,
) -> Result<(i64, i64, Option<i64>), Option<(i64, i64)>> {
    let (index, trips) = if case.enabled || matches!(shape, Shape::Prefix) {
        induction(case, compare, step).ok_or(None)?
    } else {
        (case.initial, 0)
    };
    if !case.enabled && !matches!(shape, Shape::Prefix | Shape::Suffix) {
        return Ok((41, trips, None));
    }
    if !op.is_empty() && (case.divisor == 0 || (index, case.divisor) == (i64::MIN, -1)) {
        return Err(Some((index, trips)));
    }
    let value = match op {
        "/" => (i128::from(index) / i128::from(case.divisor)) as i64,
        "%" => (i128::from(index) % i128::from(case.divisor)) as i64,
        "" => index,
        _ => unreachable!(),
    };
    Ok((
        if !case.enabled && matches!(shape, Shape::Prefix) {
            41
        } else {
            value
        },
        trips,
        Some(index),
    ))
}

fn execute(shape: Shape, compare: &str, step: &str, op: &str, cases: &[Case], trap: bool) {
    use aggregate_loop_probe::Invocation;
    let cases = cases
        .iter()
        .map(|&case| {
            let (state, leaf, iterations, reference_error) =
                match expected(case, shape, compare, step, op) {
                    Ok((value, trips, leaf)) => {
                        assert!(!trap);
                        (
                            Some(vec![value, 41]),
                            leaf.map(|index| [index, case.divisor]),
                            trips,
                            None,
                        )
                    }
                    Err(leaf) => {
                        assert!(trap);
                        (
                            None,
                            leaf.map(|(index, _)| [index, case.divisor]),
                            leaf.map_or(0, |(_, trips)| trips),
                            leaf.map(|_| {
                                if case.divisor == 0 {
                                    "zero"
                                } else {
                                    "overflow"
                                }
                            }),
                        )
                    }
                };
            Invocation {
                arguments: vec![
                    Value::Bool(case.enabled),
                    Value::Int(case.initial),
                    Value::Int(case.limit),
                    Value::Int(case.stride),
                    Value::Int(case.divisor),
                    Value::Int(41),
                ],
                state,
                leaf,
                iterations,
                reference_error,
            }
        })
        .collect::<Vec<_>>();
    aggregate_loop_probe::execute(
        &source(shape, compare, step, op),
        &cases,
        "loop_while_i64_body",
        !op.is_empty(),
    );
}

#[test]
fn guarded_counted_values_match_reference_and_independent_oracle() {
    for (compare, step) in [("<", "+"), (">", "-")] {
        for op in ["/", "%"] {
            for shape in [Shape::Callee, Shape::Inline, Shape::Prefix, Shape::Suffix] {
                let mut cases = Vec::new();
                for enabled in [false, true] {
                    for initial in [-3, 0, 3] {
                        for limit in [-3, 0, 3] {
                            for stride in [-2, 0, 1, 2] {
                                for divisor in [-3, 2] {
                                    let case = Case {
                                        enabled,
                                        initial,
                                        limit,
                                        stride,
                                        divisor,
                                    };
                                    if expected(case, shape, compare, step, op).is_ok() {
                                        cases.push(case);
                                    }
                                }
                            }
                        }
                    }
                }
                assert_eq!(
                    cases.len(),
                    if matches!(shape, Shape::Prefix) {
                        120
                    } else {
                        132
                    }
                );
                execute(shape, compare, step, op, &cases, false);
            }
        }
    }
}

#[test]
fn guarded_counted_preflight_and_arithmetic_trap_only_when_selected() {
    let failures = [
        Case {
            enabled: true,
            initial: 0,
            limit: 4,
            stride: 0,
            divisor: 2,
        },
        Case {
            enabled: true,
            initial: 0,
            limit: 4,
            stride: -1,
            divisor: 2,
        },
        Case {
            enabled: true,
            initial: i64::MAX - 1,
            limit: i64::MAX,
            stride: 2,
            divisor: 2,
        },
        Case {
            enabled: true,
            initial: 0,
            limit: 65537,
            stride: 1,
            divisor: 2,
        },
        Case {
            enabled: true,
            initial: 0,
            limit: 4,
            stride: 1,
            divisor: 0,
        },
        Case {
            enabled: true,
            initial: i64::MIN,
            limit: i64::MIN,
            stride: 1,
            divisor: -1,
        },
    ];
    for shape in [Shape::Callee, Shape::Inline] {
        for op in ["/", "%"] {
            let skipped = failures
                .iter()
                .map(|case| Case {
                    enabled: false,
                    ..*case
                })
                .collect::<Vec<_>>();
            execute(shape, "<", "+", op, &skipped, false);
            for case in failures {
                execute(shape, "<", "+", op, &[case], true);
            }
        }
    }
    // A source prefix must still run even when the later return selects the fallback.
    execute(
        Shape::Prefix,
        "<",
        "+",
        "/",
        &[Case {
            enabled: false,
            ..failures[0]
        }],
        true,
    );
}

#[test]
fn loop_only_branches_preserve_preflight_without_checked_arithmetic() {
    let invalid = Case {
        enabled: true,
        initial: i64::MAX,
        limit: i64::MAX,
        stride: 1,
        divisor: 0,
    };
    execute(
        Shape::Callee,
        "<=",
        "+",
        "",
        &[Case {
            enabled: false,
            ..invalid
        }],
        false,
    );
    execute(Shape::Callee, "<=", "+", "", &[invalid], true);
    execute(
        Shape::Inline,
        "<",
        "+",
        "",
        &[Case {
            enabled: true,
            initial: 0,
            limit: 65536,
            stride: 1,
            divisor: 0,
        }],
        false,
    );
}

#[test]
fn loop_carried_updates_inside_flat_branches_gain_guarded_lowering() {
    let source = source(Shape::Callee, "<", "+", "/")
        .replace(
            "let index: i64 = initial;",
            "let index: i64 = initial; let total: i64 = 0;",
        )
        .replace(
            "let index: i64 = index + stride;",
            "let index: i64 = index + stride; let total: i64 = total + index;",
        )
        .replace("leaf(index, divisor)", "leaf(total + index, divisor)");
    let project = Project::with_source(&source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let bridge = emit_registered(&compiled.yir, "counter").unwrap();
    assert!(bridge.llvm_ir.contains("loop_while_scalar_chain_body"));
    assert!(bridge.llvm_ir.contains("native_loop_preflight"));
    assert!(bridge.llvm_ir.contains("integer_divisor_invalid"));
}

#[test]
fn counted_flat_branch_helpers_compose_with_typed_lifecycle() {
    assert_native_parity(include_str!("aggregate_counted_loops.ns"), true);
}
