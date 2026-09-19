use super::*;
use aggregate_carried::{Case, Shape};
use aggregate_checked::Body;

fn source(shape: Shape, count: usize, descending: bool, op: &str, body: Body) -> String {
    let mut text = aggregate_checked::source(shape, count, descending, op, body);
    for slot in 0..count {
        for (lhs, rhs) in [
            (format!("carry{slot}"), "divisor".to_owned()),
            (format!("(seed + {slot})"), "divisor".to_owned()),
            ("seed".to_owned(), format!("divisor{slot}")),
        ] {
            text = text.replace(
                &format!("{lhs} {op} {rhs}"),
                &format!("relay({lhs}, {rhs})"),
            );
        }
    }
    text = text.replace(&format!("seed {op} divisor"), "relay(seed, divisor)");
    if matches!(body, Body::Unused) {
        text = text.replace("= relay(seed, divisor);", "= ignore(relay(seed, divisor));");
    }
    if matches!(body, Body::ShortCircuit) {
        let mut rewritten = Vec::new();
        for line in text.lines() {
            if let Some((binding, value)) = line.split_once(" = ") {
                if binding.trim().starts_with("let gate") {
                    let name = binding.trim().strip_prefix("let ").unwrap();
                    rewritten.push(format!(
                        "let weight_{name} = select({}, 1, 0); let {name} = weight_{name} == 1;",
                        value.trim_end_matches(';')
                    ));
                    continue;
                }
                if binding.trim().starts_with("let exact") {
                    rewritten.push(format!(
                        "{binding} = boolean({});",
                        value.trim_end_matches(';')
                    ));
                    continue;
                }
            }
            rewritten.push(line.to_owned());
        }
        text = rewritten.join("\n");
    }
    text.replace("fn main()", &format!("
        @noinline fn checked_value(value: i64, divisor: i64) -> i64 {{ return value {op} divisor; }}
        @noinline fn relay(value: i64, divisor: i64) -> i64 {{ return checked_value(value, divisor); }}
        @noinline fn ignore(value: i64) -> i64 {{ return 17; }}
        @noinline fn boolean(value: bool) -> bool {{ return value; }}
        @noinline fn select(flag: bool, yes: i64, no: i64) -> i64 {{ if flag {{ return yes; }} return no; }}
        fn main()"))
}

fn execute(shape: Shape, count: usize, descending: bool, op: &str, body: Body, cases: &[Case]) {
    let mut traces = Vec::new();
    let observations = cases
        .iter()
        .map(|&case| {
            let mut trace = Vec::new();
            let expected = aggregate_checked::expected_with_calls(
                case, shape, count, descending, op, body, &mut trace,
            );
            traces.push(trace);
            expected
        })
        .collect::<Vec<_>>();
    aggregate_loop_probe::execute_probed(
        &source(shape, count, descending, op, body),
        &observations,
        "loop_while_i64_body",
        true,
        None,
        Some(aggregate_loop_probe::CallProbe {
            callee: "checked_value",
            traces: &traces,
        }),
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
fn nested_call_arguments_execute_left_to_right_before_entering_the_callee() {
    let base = source(Shape::Inline, 1, false, "/", Body::Direct);
    let text = base.replace(
        "relay(carry0, divisor)",
        "relay(checked_value(carry0, 1), checked_value(divisor, 1))",
    );
    let mut traces = Vec::new();
    let observations = [BASE, Case { divisor: 0, ..BASE }]
        .into_iter()
        .map(|case| {
            let mut trace = Vec::new();
            let expected = aggregate_checked::expected_with_calls(
                case,
                Shape::Inline,
                1,
                false,
                "/",
                Body::Direct,
                &mut trace,
            );
            traces.push(
                trace
                    .into_iter()
                    .flat_map(|[left, right, trip]| {
                        [[left, 1, trip], [right, 1, trip], [left, right, trip]]
                    })
                    .collect(),
            );
            expected
        })
        .collect::<Vec<_>>();
    aggregate_loop_probe::execute_probed(
        &text,
        &observations,
        "loop_while_i64_body",
        true,
        None,
        Some(aggregate_loop_probe::CallProbe {
            callee: "checked_value",
            traces: &traces,
        }),
    );

    // A failing first argument prevents both the second argument and the outer call.
    let text = base.replace(
        "relay(carry0, divisor)",
        "relay(checked_value(carry0, divisor), checked_value(divisor, 0))",
    );
    for case in [
        Case { divisor: 0, ..BASE },
        Case {
            divisor: -1,
            seed: i64::MIN,
            ..BASE
        },
    ] {
        let mut trace = Vec::new();
        let expected = aggregate_checked::expected_with_calls(
            case,
            Shape::Inline,
            1,
            false,
            "/",
            Body::Direct,
            &mut trace,
        );
        aggregate_loop_probe::execute_probed(
            &text,
            &[expected],
            "loop_while_i64_body",
            true,
            None,
            Some(aggregate_loop_probe::CallProbe {
                callee: "checked_value",
                traces: &[trace],
            }),
        );
    }
}

#[test]
fn iteration_scalar_calls_preserve_argument_values_order_and_snapshots() {
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
                    for seed in [i64::MIN, -17, i64::MAX] {
                        cases.push(Case {
                            enabled,
                            seed,
                            initial: if descending { 4 } else { 0 },
                            limit: if descending { 4 - trips } else { trips },
                            divisor: if descending { -3 } else { 2 },
                            ..BASE
                        });
                    }
                }
            }
            execute(shape, count, descending, op, body, &cases);
        }
    }
}

#[test]
fn logical_call_arguments_and_inferred_bool_results_remain_lazy() {
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
                Case { seed: 3, ..BASE },
                Case { seed: 8, ..BASE },
            ],
        );
    }
}

#[test]
fn unselected_iteration_calls_skip_arguments_but_reached_ignored_arguments_trap() {
    for op in ["/", "%"] {
        execute(
            Shape::Inline,
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
                BASE,
                Case { divisor: 0, ..BASE },
            ],
        );
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
                Body::Unused,
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
        execute(
            Shape::Suffix,
            3,
            false,
            op,
            Body::Late,
            &[Case { divisor: 2, ..BASE }],
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
fn iteration_calls_cannot_bypass_induction_preflight_or_native_closure_limits() {
    for case in [
        Case { stride: 0, ..BASE },
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
        execute(
            Shape::Callee,
            1,
            false,
            "/",
            Body::Direct,
            &[Case { divisor: 0, ..case }],
        );
    }
    let mut text = source(Shape::Callee, 1, false, "/", Body::Direct).replace(
        "return checked_value(value, divisor);",
        "return hop0(value, divisor);",
    );
    for index in (0..40).rev() {
        let next = if index == 39 {
            "checked_value".to_owned()
        } else {
            format!("hop{}", index + 1)
        };
        text = text.replace("fn main()", &format!("@noinline fn hop{index}(value: i64, divisor: i64) -> i64 {{ return {next}(value, divisor); }} fn main()"));
    }
    let project = Project::with_source(&text);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let error = emit_registered(&compiled.yir, "counter").unwrap_err();
    assert!(
        error.contains("depth") || error.contains("functions"),
        "{error}"
    );
}
