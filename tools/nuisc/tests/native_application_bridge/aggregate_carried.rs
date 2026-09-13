use super::*;
use aggregate_loop_probe::Invocation;

#[derive(Clone, Copy, Debug)]
pub(super) enum Shape {
    Callee,
    Inline,
    Prefix,
    Suffix,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Case {
    pub(super) enabled: bool,
    pub(super) initial: i64,
    pub(super) limit: i64,
    pub(super) stride: i64,
    pub(super) divisor: i64,
    pub(super) seed: i64,
}

pub(super) fn source(shape: Shape, count: usize, descending: bool, op: &str) -> String {
    let (compare, step) = if descending { (">", "-") } else { ("<", "+") };
    let mut fields = "value: i64, index: i64, ".to_owned();
    let mut seeds = "let index: i64 = initial;".to_owned();
    let mut updates = format!("let index: i64 = index {step} stride;");
    let mut fallback = "value: seed, index: initial, ".to_owned();
    let mut result = format!("value: leaf(carry{}, divisor), index: index, ", count - 1);
    let mut state = "value: result.value, index: result.index, ".to_owned();
    for slot in 0..count {
        fields.push_str(&format!("carry{slot}: i64, "));
        seeds.push_str(&format!("let carry{slot}: i64 = seed + {slot};"));
        let rhs = if slot == 0 {
            "index".into()
        } else {
            format!("carry{}", slot - 1)
        };
        let update = if slot % 3 == 2 { "*" } else { "+" };
        updates.push_str(&format!(
            "let carry{slot}: i64 = carry{slot} {update} {rhs};"
        ));
        fallback.push_str(&format!("carry{slot}: seed + {slot}, "));
        result.push_str(&format!("carry{slot}: carry{slot}, "));
        state.push_str(&format!("carry{slot}: result.carry{slot}, "));
    }
    fields.push_str("seed: i64");
    fallback.push_str("seed: seed");
    result.push_str("seed: seed");
    state.push_str("seed: result.seed");
    let loop_body = format!("while index {compare} limit {{ {updates} }}");
    let calculate = format!("{seeds} {loop_body} return Parts {{ {result} }};");
    let fallback = format!("Parts {{ {fallback} }}");
    let call = "calculate(initial, limit, stride, divisor, seed)";
    let body = match shape {
        Shape::Callee => format!("if enabled {{ return {call}; }} return {fallback};"),
        Shape::Inline => format!("if enabled {{ {calculate} }} return {fallback};"),
        Shape::Prefix => {
            format!("let saved: Parts = {call}; if enabled {{ return saved; }} return {fallback};")
        }
        Shape::Suffix => {
            format!("{seeds} if enabled {{ {loop_body} }} return Parts {{ {result} }};")
        }
    };
    let arithmetic = if op.is_empty() {
        "value".into()
    } else {
        format!("value {op} divisor")
    };
    format!("mod cpu Main {{
      struct State {{ {fields} }}
      struct Parts {{ {fields} }}
      @noinline fn leaf(value: i64, divisor: i64) -> i64 {{ return {arithmetic}; }}
      @noinline fn calculate(initial: i64, limit: i64, stride: i64, divisor: i64, seed: i64) -> Parts {{ {calculate} }}
      @noinline fn compute(enabled: bool, initial: i64, limit: i64, stride: i64, divisor: i64, seed: i64) -> Parts {{ {body} }}
      fn start(enabled: bool, initial: i64, limit: i64, stride: i64, divisor: i64, seed: i64) -> State {{
        let result: Parts = compute(enabled, initial, limit, stride, divisor, seed);
        return State {{ {state} }};
      }}
      fn step(state: State) -> State {{ return state; }}
      fn stop(state: State) -> State {{ return state; }}
      fn main() -> i64 {{ print(999); return 0; }}
    }}")
}

fn expected(case: Case, shape: Shape, count: usize, descending: bool, op: &str) -> Invocation {
    let mut observation = Invocation {
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
            for slot in 0..count {
                let rhs = i128::from(if slot == 0 { index } else { carries[slot - 1] });
                carries[slot] = if slot % 3 == 2 {
                    (i128::from(carries[slot]) * rhs) as i64
                } else {
                    (i128::from(carries[slot]) + rhs) as i64
                };
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

fn execute(shape: Shape, count: usize, descending: bool, op: &str, cases: &[Case]) {
    let observations = cases
        .iter()
        .map(|&case| expected(case, shape, count, descending, op))
        .collect::<Vec<_>>();
    aggregate_loop_probe::execute(
        &source(shape, count, descending, op),
        &observations,
        "loop_while_scalar_chain_body",
        !op.is_empty(),
    );
}

#[test]
fn guarded_carried_values_match_reference_and_ordered_wrapping_oracle() {
    for descending in [false, true] {
        for op in ["/", "%"] {
            for shape in [Shape::Callee, Shape::Inline, Shape::Prefix, Shape::Suffix] {
                let mut cases = Vec::new();
                for enabled in [false, true] {
                    for (initial, limit) in [(-3, 4), (4, -3), (0, 0)] {
                        for stride in [-1, 0, 1, 2] {
                            for seed in [i64::MIN, -7, 0, i64::MAX] {
                                for divisor in [-3, 2] {
                                    let case = Case {
                                        enabled,
                                        initial,
                                        limit,
                                        stride,
                                        divisor,
                                        seed,
                                    };
                                    if expected(case, shape, 3, descending, op).state.is_some() {
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
                        160
                    } else {
                        176
                    }
                );
                execute(shape, 3, descending, op, &cases);
            }
        }
    }
}

#[test]
fn variable_carry_widths_and_loop_only_guards_retain_every_slot() {
    for count in [1, 7] {
        for descending in [false, true] {
            let mut cases = Vec::new();
            for enabled in [false, true] {
                for trips in [0, 1, 7] {
                    for seed in [i64::MIN, 1, i64::MAX] {
                        cases.push(Case {
                            enabled,
                            initial: 0,
                            limit: if descending { -trips } else { trips },
                            stride: 1,
                            divisor: 0,
                            seed,
                        });
                    }
                }
            }
            execute(Shape::Callee, count, descending, "", &cases);
        }
    }
}

#[test]
fn guarded_carries_trap_before_updates_and_never_speculate_skipped_calls() {
    let base = Case {
        enabled: true,
        initial: 0,
        limit: 4,
        stride: 1,
        divisor: 2,
        seed: 1,
    };
    let failures = [
        Case { stride: 0, ..base },
        Case { stride: -1, ..base },
        Case {
            initial: i64::MAX - 1,
            limit: i64::MAX,
            stride: 2,
            ..base
        },
        Case {
            limit: 65537,
            ..base
        },
        Case { divisor: 0, ..base },
        // The third seed wraps to exactly i64::MIN on zero trips.
        Case {
            limit: 0,
            divisor: -1,
            seed: i64::MAX - 1,
            ..base
        },
    ];
    for shape in [Shape::Callee, Shape::Inline] {
        for op in ["/", "%"] {
            let skipped = failures.map(|case| Case {
                enabled: false,
                ..case
            });
            execute(shape, 3, false, op, &skipped);
            for case in failures {
                assert!(
                    expected(case, shape, 3, false, op).state.is_none(),
                    "{case:?}"
                );
                execute(shape, 3, false, op, &[case]);
            }
        }
    }
    execute(
        Shape::Prefix,
        3,
        false,
        "/",
        &[Case {
            enabled: false,
            ..failures[0]
        }],
    );
    execute(Shape::Callee, 3, false, "", &[failures[0]]);
    execute(
        Shape::Inline,
        3,
        false,
        "",
        &[Case {
            limit: 65536,
            divisor: 0,
            ..base
        }],
    );
    // Descending induction underflow must also fail before any carry update.
    execute(
        Shape::Inline,
        3,
        true,
        "",
        &[Case {
            initial: i64::MIN + 1,
            limit: i64::MIN,
            stride: 2,
            ..base
        }],
    );
}

#[test]
fn one_sided_carry_updates_inside_flat_branches_gain_shared_keep_lowering() {
    let source = source(Shape::Callee, 3, false, "/").replace(
        "let carry1: i64 = carry1 + carry0;",
        "if index > 0 { let carry1: i64 = carry1 + carry0; }",
    );
    let project = Project::with_source(&source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    assert!(compiled
        .yir
        .nodes
        .iter()
        .any(|node| node.op.instruction == "loop_while_scalar_cond_chain"
            && node.op.args.contains(&"keep".to_owned())));
    let bridge = emit_registered(&compiled.yir, "counter").unwrap();
    assert!(bridge.llvm_ir.contains("loop_while_scalar_cond_chain_body"));
    assert!(bridge.llvm_ir.contains("native_loop_preflight"));
}

#[test]
fn carried_flat_branch_helpers_compose_with_typed_lifecycle() {
    assert_native_parity(include_str!("aggregate_carried_loops.ns"), true);
}
