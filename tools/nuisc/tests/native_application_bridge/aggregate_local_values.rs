use super::*;
use aggregate_carried::{Case, Shape};
use aggregate_checked::Body;

const BASE: Case = Case {
    enabled: true,
    initial: 0,
    limit: 4,
    stride: 1,
    divisor: 3,
    seed: -17,
};

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
                &format!("packet({lhs}, {rhs}).first"),
            );
        }
        if matches!(body, Body::Temporary) {
            text = text.replace(&format!("let local{slot} = packet(carry{slot}, divisor).first;"),
                &format!("let record{slot} = packet(carry{slot}, divisor); let copy{slot}: Packet = record{slot}; let local{slot} = read(copy{slot});"));
        }
    }
    text = text.replace(&format!("seed {op} divisor"), "packet(seed, divisor).first");
    if matches!(body, Body::Unused) {
        text = text.replace("= packet(seed, divisor).first;", "= packet(seed, divisor);");
    }
    if matches!(body, Body::ShortCircuit) {
        let mut lines = Vec::new();
        for line in text.lines() {
            if let Some((binding, value)) = line.split_once(" = ") {
                if let Some(name) = binding.trim().strip_prefix("let gate") {
                    lines.push(format!("let selected{name} = select({}, index); let gate{name} = selected{name}.first == 1;", value.trim_end_matches(';')));
                    continue;
                }
            }
            lines.push(line.to_owned());
        }
        text = lines.join("\n");
    }
    text.replace(
        "fn main()",
        &format!(
            "
        struct Packet {{ first: i64, second: i64, third: i64 }}
        @noinline fn checked_value(value: i64, divisor: i64) -> i64 {{ return value {op} divisor; }}
        @noinline fn packet(value: i64, divisor: i64) -> Packet {{
            let result: i64 = checked_value(value, divisor);
            if value < 0 {{ return Packet {{ third: value, second: divisor, first: result }}; }}
            return Packet {{ first: result, second: divisor, third: value }};
        }}
        @noinline fn read(value: Packet) -> i64 {{ return value.first; }}
        @noinline fn select(flag: bool, value: i64) -> Packet {{
            if flag {{ return Packet {{ third: value, first: 1, second: 0 }}; }}
            return Packet {{ first: 0, second: 0, third: value }};
        }}
        fn main()"
        ),
    )
}

fn execute(shape: Shape, count: usize, descending: bool, op: &str, body: Body, cases: &[Case]) {
    let mut traces = Vec::new();
    let observations = cases
        .iter()
        .map(|&case| {
            let mut trace = Vec::new();
            let result = aggregate_checked::expected_with_calls(
                case, shape, count, descending, op, body, &mut trace,
            );
            traces.push(trace);
            result
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

#[test]
fn flat_iteration_results_project_copy_and_pass_exact_values() {
    for (shape, body, count, op, descending) in [
        (Shape::Callee, Body::Direct, 1, "/", false),
        (Shape::Inline, Body::Temporary, 3, "%", true),
        (Shape::Prefix, Body::Temporary, 7, "/", false),
        (Shape::Suffix, Body::Direct, 3, "%", true),
    ] {
        let mut cases = Vec::new();
        for enabled in [false, true] {
            for trips in [0, 1, 4] {
                for seed in [i64::MIN, -17, i64::MAX] {
                    cases.push(Case {
                        enabled,
                        seed,
                        initial: if descending { 4 } else { 0 },
                        limit: if descending { 4 - trips } else { trips },
                        ..BASE
                    });
                }
            }
        }
        execute(shape, count, descending, op, body, &cases);
    }
}

#[test]
fn flat_iteration_arguments_keep_short_circuit_and_ignored_result_checks() {
    for op in ["/", "%"] {
        execute(
            Shape::Inline,
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
                Case { seed: 8, ..BASE },
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
                    Case { limit: 0, ..case },
                    case,
                ],
            );
        }
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
                BASE,
                Case { divisor: 0, ..BASE },
            ],
        );
        execute(
            Shape::Inline,
            1,
            false,
            op,
            Body::Late,
            &[Case { divisor: 2, ..BASE }],
        );
    }
}

#[test]
fn flat_iteration_snapshots_survive_mutation_and_branch_captures() {
    let mut text = source(Shape::Inline, 1, false, "/", Body::Unused);
    text = text.replace(
        "third: value, second: divisor",
        "third: divisor, second: value % divisor",
    );
    text = text.replace(
        "second: divisor, third: value",
        "second: value % divisor, third: divisor",
    );
    text = text.replace(
        "return value.first;",
        "return value.first * value.third + value.second;",
    );
    let old =
        "let unused0 = packet(seed, divisor);\n                let carry0: i64 = carry0 + index;";
    assert!(text.contains(old));
    text = text.replace(
        old,
        "
        let carry0: i64 = carry0;
        let record = packet(carry0, divisor);
        let copy: Packet = record;
        let carry0: i64 = carry0 + 100;
        if index < limit && copy.third != 0 {
            let carry0: i64 = copy.first * copy.third + copy.second + index;
        } else { let carry0: i64 = read(copy) + index; }",
    );
    let mut traces = Vec::new();
    let observations = [
        BASE,
        Case {
            seed: i64::MAX,
            ..BASE
        },
        Case {
            seed: i64::MIN,
            ..BASE
        },
        Case { limit: 0, ..BASE },
        Case {
            enabled: false,
            ..BASE
        },
    ]
    .into_iter()
    .map(|case| {
        let result = aggregate_checked::expected_with_calls(
            case,
            Shape::Inline,
            1,
            false,
            "/",
            Body::Unused,
            &mut Vec::new(),
        );
        let mut trace = Vec::new();
        let mut carry = case.seed;
        for trip in 1..=result.iterations {
            trace.push([carry, case.divisor, trip]);
            carry = (i128::from(carry) + i128::from(case.initial + trip * case.stride)) as i64;
        }
        traces.push(trace);
        result
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
}

#[test]
fn flat_iteration_calls_stay_behind_complete_induction_preflight() {
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
        execute(Shape::Callee, 1, false, "/", Body::Direct, &[case]);
    }
}

#[test]
fn flat_iteration_literal_fields_evaluate_in_source_order_before_the_call() {
    let text = source(Shape::Inline, 1, false, "/", Body::Direct).replace(
        "packet(carry0, divisor).first",
        "read(Packet { third: checked_value(30, 1), first: checked_value(carry0, divisor), second: checked_value(20, 1) })",
    );
    for last in [
        Case { divisor: 0, ..BASE },
        Case {
            divisor: -1,
            seed: i64::MIN,
            ..BASE
        },
    ] {
        let mut traces = Vec::new();
        let observations = [BASE, last]
            .into_iter()
            .map(|case| {
                let mut trace = Vec::new();
                let result = aggregate_checked::expected_with_calls(
                    case,
                    Shape::Inline,
                    1,
                    false,
                    "/",
                    Body::Direct,
                    &mut trace,
                );
                let length = trace.len();
                traces.push(
                    trace
                        .into_iter()
                        .enumerate()
                        .flat_map(|(position, [left, right, trip])| {
                            let mut fields = vec![[30, 1, trip], [left, right, trip]];
                            if result.state.is_some() || position + 1 < length {
                                fields.push([20, 1, trip]);
                            }
                            fields
                        })
                        .collect(),
                );
                result
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
    }
}

#[test]
fn flat_iteration_ignored_aggregate_arguments_still_execute_and_trap() {
    let text = source(Shape::Inline, 1, false, "/", Body::Unused)
        .replace(
            "let unused0 = packet(seed, divisor);",
            "let unused0 = ignore_packet(packet(seed, divisor));",
        )
        .replace(
            "fn main()",
            "@noinline fn ignore_packet(value: Packet) -> i64 { return 0; } fn main()",
        );
    let mut traces = Vec::new();
    let observations = [BASE, Case { divisor: 0, ..BASE }]
        .into_iter()
        .map(|case| {
            let mut trace = Vec::new();
            let result = aggregate_checked::expected_with_calls(
                case,
                Shape::Inline,
                1,
                false,
                "/",
                Body::Unused,
                &mut trace,
            );
            traces.push(trace);
            result
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
}
