use super::*;
use aggregate_carried::{Case as Outer, Shape};
use aggregate_loop_probe::{CallProbe, Invocation};

#[derive(Clone, Copy)]
enum Use {
    Direct,
    Local,
    Branch,
    Logical,
    Late,
    Unused,
    Ignored,
}

#[derive(Clone, Copy)]
struct Case {
    outer: Outer,
    child_limit: i64,
    child_step: i64,
}

const BASE: Case = Case {
    outer: Outer {
        enabled: true,
        initial: 0,
        limit: 3,
        stride: 1,
        divisor: 2,
        seed: -17,
    },
    child_limit: 2,
    child_step: 1,
};

fn source(
    shape: Shape,
    count: usize,
    descending: bool,
    op: &str,
    usage: Use,
    layers: usize,
) -> String {
    let mut text = aggregate_carried::source(shape, count, descending, "")
        .replace(
            "divisor: i64, seed: i64",
            "divisor: i64, seed: i64, child_limit: i64, child_step: i64",
        )
        .replace(
            "stride, divisor, seed)",
            "stride, divisor, seed, child_limit, child_step)",
        );
    for slot in 0..count {
        let rhs = if slot == 0 {
            "index".to_owned()
        } else {
            format!("carry{}", slot - 1)
        };
        let update = if slot % 3 == 2 { "*" } else { "+" };
        let old = format!("let carry{slot}: i64 = carry{slot} {update} {rhs};");
        let call = format!("scalar(carry{slot}, child_limit, child_step, divisor)");
        let body = match usage {
            Use::Direct => format!("let carry{slot}: i64 = {call} + index;"),
            Use::Local => format!("let carry{slot}: i64 = carry{slot}; let record{slot} = relay(carry{slot}, child_limit, child_step, divisor); let saved{slot}: Packet = record{slot}; let carry{slot}: i64 = carry{slot} + 100; let carry{slot}: i64 = read(saved{slot}) + index;"),
            Use::Branch => format!("if index < 2 {{ let carry{slot}: i64 = {call} + index; }} else {{ let carry{slot}: i64 = carry{slot} + index; }}"),
            Use::Logical => format!("let selected{slot} = divisor == 0 || predicate(seed, child_limit, child_step, divisor); if selected{slot} {{ let carry{slot}: i64 = carry{slot} + index; }} else {{ let carry{slot}: i64 = carry{slot} - index; }}"),
            Use::Late => format!("let carry{slot}: i64 = scalar(carry{slot}, child_limit, child_step + 2 - index, divisor) + index;"),
            Use::Unused | Use::Ignored => {
                let call = "relay(seed, child_limit, child_step, divisor)";
                let value = if matches!(usage, Use::Ignored) { format!("ignore({call})") } else { call.to_owned() };
                format!("let unused{slot} = {value}; let carry{slot}: i64 = carry{slot} + index;")
            }
        };
        assert!(text.contains(&old));
        text = text.replace(&old, &body);
    }
    let (initial, compare, bound, step) = if descending {
        ("limit", ">", "0", "-")
    } else {
        ("0", "<", "limit", "+")
    };
    let mut helpers = format!("
        struct Packet {{ value: i64, visits: i64 }}
        @noinline fn checked_value(value: i64, divisor: i64) -> i64 {{ return value {op} divisor; }}
        @noinline fn read(value: Packet) -> i64 {{ return value.value + value.visits; }}
        @noinline fn ignore(value: Packet) -> i64 {{ return 0; }}
        @noinline fn relay(value: i64, limit: i64, stride: i64, divisor: i64) -> Packet {{ return loop{layers}(value, limit, stride, divisor); }}
        @noinline fn scalar(value: i64, limit: i64, stride: i64, divisor: i64) -> i64 {{ return read(relay(value, limit, stride, divisor)); }}
        @noinline fn predicate(value: i64, limit: i64, stride: i64, divisor: i64) -> bool {{ return relay(value, limit, stride, divisor).visits > 0; }}
    ");
    for layer in 0..=layers {
        let body = if layer == 0 {
            "let total: i64 = checked_value(total, divisor) + index; let visits: i64 = visits + 1;"
                .to_owned()
        } else {
            format!("let total: i64 = total; let inner = loop{}(total, limit, stride, divisor); let total: i64 = inner.value + index; let visits: i64 = visits + inner.visits + 1;", layer - 1)
        };
        helpers.push_str(&format!("@noinline fn loop{layer}(value: i64, limit: i64, stride: i64, divisor: i64) -> Packet {{
            let index: i64 = {initial}; let total: i64 = value; let visits: i64 = 0;
            while index {compare} {bound} {{ let index: i64 = index {step} stride; {body} }}
            return Packet {{ visits: visits, value: total }};
        }}"));
    }
    text.replace("fn main()", &format!("{helpers} fn main()"))
}

enum Fault {
    Preflight,
    Arithmetic(&'static str),
}

fn induction(mut index: i64, bound: i64, stride: i64, descending: bool) -> Result<Vec<i64>, Fault> {
    let mut values = Vec::new();
    while if descending {
        index > bound
    } else {
        index < bound
    } {
        if stride <= 0 || values.len() == 65536 {
            return Err(Fault::Preflight);
        }
        index = if descending {
            index.checked_sub(stride)
        } else {
            index.checked_add(stride)
        }
        .ok_or(Fault::Preflight)?;
        values.push(index);
    }
    Ok(values)
}

#[derive(Default)]
struct Execution {
    iterations: i64,
    calls: Vec<[i64; 3]>,
}

impl Execution {
    fn child(
        &mut self,
        layer: usize,
        mut value: i64,
        limit: i64,
        stride: i64,
        divisor: i64,
        descending: bool,
        op: &str,
    ) -> Result<(i64, i64), Fault> {
        // Each invocation proves its complete induction before any body work.
        let indices = induction(
            if descending { limit } else { 0 },
            if descending { 0 } else { limit },
            stride,
            descending,
        )?;
        let mut visits = 0i64;
        for index in indices {
            self.iterations += 1;
            if layer == 0 {
                self.calls.push([value, divisor, self.iterations]);
                if divisor == 0 {
                    return Err(Fault::Arithmetic("zero"));
                }
                if (value, divisor) == (i64::MIN, -1) {
                    return Err(Fault::Arithmetic("overflow"));
                }
                let result = if op == "/" {
                    i128::from(value) / i128::from(divisor)
                } else {
                    i128::from(value) % i128::from(divisor)
                };
                value = (result + i128::from(index)) as i64;
                visits = visits.wrapping_add(1);
            } else {
                let (inner, count) =
                    self.child(layer - 1, value, limit, stride, divisor, descending, op)?;
                value = inner.wrapping_add(index);
                visits = visits.wrapping_add(count).wrapping_add(1);
            }
        }
        Ok((value, visits))
    }
}

fn expected(
    case: Case,
    shape: Shape,
    count: usize,
    descending: bool,
    op: &str,
    usage: Use,
    layers: usize,
) -> (Invocation, Vec<[i64; 3]>) {
    let c = case.outer;
    let mut result = Invocation {
        arguments: vec![
            Value::Bool(c.enabled),
            Value::Int(c.initial),
            Value::Int(c.limit),
            Value::Int(c.stride),
            Value::Int(c.divisor),
            Value::Int(c.seed),
            Value::Int(case.child_limit),
            Value::Int(case.child_step),
        ],
        state: None,
        leaf: None,
        iterations: 0,
        reference_error: None,
    };
    let seeds = (0..count)
        .map(|slot| c.seed.wrapping_add(slot as i64))
        .collect::<Vec<_>>();
    let mut carries = seeds.clone();
    let mut execution = Execution::default();
    let mut index = c.initial;
    let operation = (|| {
        if c.enabled || matches!(shape, Shape::Prefix) {
            for current in induction(c.initial, c.limit, c.stride, descending)? {
                index = current;
                execution.iterations += 1;
                for carry in &mut carries {
                    match usage {
                        Use::Branch if index >= 2 => *carry = carry.wrapping_add(index),
                        Use::Logical => {
                            let selected = c.divisor == 0
                                || execution
                                    .child(
                                        layers,
                                        c.seed,
                                        case.child_limit,
                                        case.child_step,
                                        c.divisor,
                                        descending,
                                        op,
                                    )?
                                    .1
                                    > 0;
                            *carry = if selected {
                                carry.wrapping_add(index)
                            } else {
                                carry.wrapping_sub(index)
                            };
                        }
                        Use::Unused | Use::Ignored => {
                            execution.child(
                                layers,
                                c.seed,
                                case.child_limit,
                                case.child_step,
                                c.divisor,
                                descending,
                                op,
                            )?;
                            *carry = carry.wrapping_add(index);
                        }
                        _ => {
                            let stride = if matches!(usage, Use::Late) {
                                case.child_step.wrapping_add(2).wrapping_sub(index)
                            } else {
                                case.child_step
                            };
                            let (value, visits) = execution.child(
                                layers,
                                *carry,
                                case.child_limit,
                                stride,
                                c.divisor,
                                descending,
                                op,
                            )?;
                            *carry = value.wrapping_add(visits).wrapping_add(index);
                        }
                    }
                }
            }
        }
        Ok::<(), Fault>(())
    })();
    result.iterations = execution.iterations;
    if let Err(fault) = operation {
        if let Fault::Arithmetic(message) = fault {
            result.reference_error = Some(message);
        }
        return (result, execution.calls);
    }
    if c.enabled || matches!(shape, Shape::Prefix | Shape::Suffix) {
        result.leaf = Some([carries[count - 1], c.divisor]);
    }
    let mut state = if !c.enabled && !matches!(shape, Shape::Suffix) {
        let mut state = vec![c.seed, c.initial];
        state.extend(seeds);
        state
    } else {
        let mut state = vec![carries[count - 1], index];
        state.extend(carries);
        state
    };
    state.push(c.seed);
    result.state = Some(state);
    (result, execution.calls)
}

fn execute(
    shape: Shape,
    count: usize,
    descending: bool,
    op: &str,
    usage: Use,
    layers: usize,
    cases: &[Case],
) {
    let (observations, traces): (Vec<_>, Vec<_>) = cases
        .iter()
        .map(|&case| expected(case, shape, count, descending, op, usage, layers))
        .unzip();
    aggregate_loop_probe::execute_probed(
        &source(shape, count, descending, op, usage, layers),
        &observations,
        "loop_while_i64_body",
        true,
        None,
        Some(CallProbe {
            callee: "checked_value",
            traces: &traces,
        }),
    );
}

#[test]
fn loop_helpers_execute_scalar_and_flat_results_at_actual_iteration_positions() {
    for (shape, usage, count, descending, op) in [
        (Shape::Callee, Use::Direct, 1, false, "/"),
        (Shape::Inline, Use::Local, 3, true, "%"),
        (Shape::Prefix, Use::Local, 1, false, "/"),
        (Shape::Suffix, Use::Direct, 1, true, "%"),
    ] {
        let mut cases = Vec::new();
        for enabled in [false, true] {
            for trips in [0, 1, 3] {
                for seed in [i64::MIN, -17, i64::MAX] {
                    cases.push(Case {
                        outer: Outer {
                            enabled,
                            seed,
                            initial: if descending { 3 } else { 0 },
                            limit: if descending { 3 - trips } else { trips },
                            ..BASE.outer
                        },
                        ..BASE
                    });
                }
            }
        }
        execute(shape, count, descending, op, usage, 0, &cases);
    }
}

#[test]
fn loop_helpers_cross_multiple_validated_loop_call_boundaries() {
    for layers in [1, 2] {
        execute(
            Shape::Inline,
            1,
            false,
            "/",
            Use::Local,
            layers,
            &[
                BASE,
                Case {
                    outer: Outer {
                        seed: i64::MAX,
                        ..BASE.outer
                    },
                    ..BASE
                },
            ],
        );
    }
}

#[test]
fn loop_helper_lazy_and_unselected_calls_skip_child_preflight() {
    execute(
        Shape::Inline,
        1,
        false,
        "/",
        Use::Logical,
        0,
        &[
            BASE,
            Case {
                child_limit: 0,
                ..BASE
            },
            Case {
                outer: Outer {
                    divisor: 0,
                    ..BASE.outer
                },
                child_step: 0,
                ..BASE
            },
        ],
    );
    execute(
        Shape::Callee,
        1,
        false,
        "/",
        Use::Branch,
        0,
        &[
            Case {
                outer: Outer {
                    initial: 2,
                    ..BASE.outer
                },
                child_step: 0,
                ..BASE
            },
            Case {
                outer: Outer {
                    enabled: false,
                    ..BASE.outer
                },
                child_limit: 65537,
                ..BASE
            },
            Case {
                outer: Outer {
                    limit: 0,
                    ..BASE.outer
                },
                child_step: 0,
                ..BASE
            },
        ],
    );
}

#[test]
fn loop_helper_preflight_remains_per_selected_invocation() {
    for case in [
        Case {
            child_step: 0,
            ..BASE
        },
        Case {
            child_limit: 65537,
            ..BASE
        },
        Case {
            child_limit: i64::MAX,
            child_step: i64::MAX - 1,
            ..BASE
        },
        Case {
            outer: Outer {
                limit: 65537,
                ..BASE.outer
            },
            ..BASE
        },
        Case {
            outer: Outer {
                initial: i64::MAX - 3,
                limit: i64::MAX,
                stride: 2,
                ..BASE.outer
            },
            ..BASE
        },
    ] {
        execute(Shape::Inline, 1, false, "/", Use::Direct, 0, &[case]);
    }
    execute(
        Shape::Callee,
        1,
        false,
        "/",
        Use::Late,
        0,
        &[Case {
            child_step: 0,
            ..BASE
        }],
    );
}

#[test]
fn loop_helper_discarded_results_and_arguments_keep_arithmetic_traps() {
    for (usage, op) in [(Use::Unused, "/"), (Use::Ignored, "%")] {
        for outer in [
            Outer {
                divisor: 0,
                ..BASE.outer
            },
            Outer {
                divisor: -1,
                seed: i64::MIN,
                ..BASE.outer
            },
        ] {
            execute(
                Shape::Inline,
                1,
                false,
                op,
                usage,
                0,
                &[
                    Case {
                        outer: Outer {
                            enabled: false,
                            ..outer
                        },
                        ..BASE
                    },
                    Case {
                        outer: Outer { limit: 0, ..outer },
                        ..BASE
                    },
                    Case {
                        outer,
                        child_limit: 0,
                        ..BASE
                    },
                    Case { outer, ..BASE },
                ],
            );
        }
    }
}

#[test]
fn loop_helper_arguments_run_before_the_callees_preflight() {
    let text = source(Shape::Inline, 1, false, "/", Use::Direct, 0).replace(
        "scalar(carry0, child_limit, child_step, divisor)",
        "scalar(checked_value(carry0, divisor), child_limit, child_step, divisor)",
    );
    let case = Case {
        outer: Outer {
            divisor: 0,
            ..BASE.outer
        },
        child_step: 0,
        ..BASE
    };
    let (mut observation, _) = expected(case, Shape::Inline, 1, false, "/", Use::Direct, 0);
    observation.reference_error = Some("zero");
    assert_eq!(observation.iterations, 1);
    aggregate_loop_probe::execute_probed(
        &text,
        &[observation],
        "loop_while_i64_body",
        true,
        None,
        Some(CallProbe {
            callee: "checked_value",
            traces: &[vec![[case.outer.seed, 0, 1]]],
        }),
    );
}

#[test]
fn loop_helper_dependency_order_does_not_bypass_native_closure_budgets() {
    let mut text = source(Shape::Callee, 1, false, "/", Use::Direct, 0).replace(
        "return loop0(value, limit, stride, divisor);",
        "return hop0(value, limit, stride, divisor);",
    );
    for index in (0..40).rev() {
        let next = if index == 39 {
            "loop0".to_owned()
        } else {
            format!("hop{}", index + 1)
        };
        text = text.replace("fn main()", &format!("@noinline fn hop{index}(value: i64, limit: i64, stride: i64, divisor: i64) -> Packet {{ return {next}(value, limit, stride, divisor); }} fn main()"));
    }
    let project = Project::with_source(&text);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let error = emit_registered(&compiled.yir, "counter").unwrap_err();
    assert!(
        error.contains("depth") || error.contains("functions"),
        "{error}"
    );
}
