use super::*;
use aggregate_carried::{Case, Shape};
use aggregate_loop_probe::{CallProbe, Invocation};

const BASE: Case = Case {
    enabled: true,
    initial: 0,
    limit: 4,
    stride: 1,
    divisor: 3,
    seed: -17,
};

fn source(shape: Shape, count: usize, descending: bool, op: &str, unused: bool) -> String {
    let text = aggregate_carried::source(shape, count, descending, "");
    let body = format!(
        "
        let carry0: i64 = carry0;
        let gate = index < limit;
        let original = gate;
        let gate: bool = gate == false;
        let second: bool = false;
        let scratch: i64 = index;
        if gate {{
            let gate = false;
            let second: bool = original == false;
            let scratch: i64 = scratch + 3;
        }} else {{
            let gate: bool = true;
            let second = original;
            let scratch: i64 = scratch - 2;
        }}
        let joined = gate;
        if gate {{ let gate = false; }}
        if second {{
            let scratch: i64 = scratch + 1;
            let local = false;
            if original {{ let local: bool = true; }}
            let gate = local;
        }}
        let remembered: bool = gate;
        let gate = divisor == 0 || (seed == ({} - 1) && divisor == -1)
            || checked_value(seed, divisor) < index;
        let saved = gate;
        let gate: bool = divisor != 0 && (seed != ({} - 1) || divisor != -1)
            && checked_value(seed, divisor) == index;
        let gate = saved || gate;
        if gate {{ let carry0: i64 = carry0 + index; }}
        else {{ let carry0: i64 = carry0 - index; }}
        if original == joined {{ let carry0: i64 = carry0 + scratch; }}
        else {{ let carry0: i64 = carry0 - 100; }}
        if remembered == joined {{ let carry0: i64 = carry0 + 1; }}
        else {{ let carry0: i64 = carry0 - 1000; }}
        {}
    ",
        i64::MIN + 1,
        i64::MIN + 1,
        if unused {
            "let gate = checked_value(seed, divisor) < index;"
        } else {
            ""
        }
    );
    assert!(text.contains("let carry0: i64 = carry0 + index;"));
    text.replace("let carry0: i64 = carry0 + index;", &body)
        .replace("fn main()", &format!(
            "@noinline fn checked_value(value: i64, divisor: i64) -> i64 {{ return value {op} divisor; }} fn main()"))
}

fn expected(
    case: Case,
    shape: Shape,
    count: usize,
    descending: bool,
    op: &str,
    unused: bool,
    trace: &mut Vec<[i64; 3]>,
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
        .map(|slot| case.seed.wrapping_add(slot as i64))
        .collect::<Vec<_>>();
    let mut carries = seeds.clone();
    let mut index = case.initial;
    let mut indices = Vec::new();
    if case.enabled || matches!(shape, Shape::Prefix) {
        while if descending {
            index > case.limit
        } else {
            index < case.limit
        } {
            if indices.len() == 65536 || case.stride <= 0 {
                return result;
            }
            let Some(next) = (if descending {
                index.checked_sub(case.stride)
            } else {
                index.checked_add(case.stride)
            }) else {
                return result;
            };
            index = next;
            indices.push(index);
        }
    }
    let danger = case.divisor == 0 || (case.seed, case.divisor) == (i64::MIN, -1);
    for &current in &indices {
        result.iterations += 1;
        // An independent value oracle: the two branch flips restore the original
        // predicate; the always-true second predicate adds one to the local word.
        let scratch = current.wrapping_add(if current < case.limit { -1 } else { 4 });
        let gate = if danger {
            true
        } else {
            trace.extend([[case.seed, case.divisor, result.iterations]; 2]);
            let checked = match op {
                "/" => (i128::from(case.seed) / i128::from(case.divisor)) as i64,
                "%" => (i128::from(case.seed) % i128::from(case.divisor)) as i64,
                _ => unreachable!(),
            };
            checked <= current
        };
        carries[0] = if gate {
            carries[0].wrapping_add(current)
        } else {
            carries[0].wrapping_sub(current)
        };
        carries[0] = carries[0].wrapping_add(scratch).wrapping_add(1);
        if unused {
            trace.push([case.seed, case.divisor, result.iterations]);
            if danger {
                result.reference_error = Some(if case.divisor == 0 {
                    "zero"
                } else {
                    "overflow"
                });
                return result;
            }
        }
        for slot in 1..count {
            carries[slot] = if slot % 3 == 2 {
                carries[slot].wrapping_mul(carries[slot - 1])
            } else {
                carries[slot].wrapping_add(carries[slot - 1])
            };
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

fn execute(shape: Shape, count: usize, descending: bool, op: &str, unused: bool, cases: &[Case]) {
    let mut traces = Vec::new();
    let cases = cases
        .iter()
        .map(|&case| {
            let mut trace = Vec::new();
            let observation = expected(case, shape, count, descending, op, unused, &mut trace);
            traces.push(trace);
            observation
        })
        .collect::<Vec<_>>();
    aggregate_loop_probe::execute_probed(
        &source(shape, count, descending, op, unused),
        &cases,
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
fn bool_rebindings_preserve_snapshots_nested_joins_and_lazy_calls() {
    for (shape, count, descending, op) in [
        (Shape::Callee, 1, false, "/"),
        (Shape::Inline, 3, true, "%"),
        (Shape::Prefix, 7, false, "%"),
        (Shape::Suffix, 3, true, "/"),
    ] {
        let mut cases = Vec::new();
        for enabled in [false, true] {
            for trips in [0, 1, 4] {
                for seed in [i64::MIN, -17, 8, i64::MAX] {
                    for divisor in [0, -1, 3] {
                        cases.push(Case {
                            enabled,
                            seed,
                            divisor,
                            initial: if descending { 4 } else { 0 },
                            limit: if descending { 4 - trips } else { trips },
                            ..BASE
                        });
                    }
                }
            }
        }
        execute(shape, count, descending, op, false, &cases);
    }
}

#[test]
fn bool_rebindings_do_not_hide_unused_arithmetic_traps() {
    for op in ["/", "%"] {
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
                3,
                false,
                op,
                true,
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
    }
}

#[test]
fn bool_rebindings_keep_complete_induction_preflight() {
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
    ] {
        execute(
            Shape::Callee,
            3,
            false,
            "/",
            false,
            &[
                Case {
                    enabled: false,
                    ..case
                },
                case,
            ],
        );
    }
}

#[test]
fn bool_branch_transport_rejects_forged_cast_types_and_dependencies() {
    let source = source(Shape::Callee, 3, false, "/", false);
    let project = Project::with_source(&source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    emit_registered(&compiled.yir, "counter").unwrap();
    for (kind, wrong_kind) in [
        ("cast_bool_to_i64", "cast_i64_to_bool"),
        ("cast_i64_to_bool", "cast_bool_to_i64"),
    ] {
        let index = compiled
            .yir
            .nodes
            .iter()
            .position(|n| n.op.instruction == kind)
            .unwrap();
        for mutation in ["type", "missing", "arity"] {
            let mut drift = compiled.yir.clone();
            let node = &mut drift.nodes[index];
            match mutation {
                "type" => node.op.instruction = wrong_kind.to_owned(),
                "missing" => node.op.args[0] = "not_a_value".to_owned(),
                "arity" => node.op.args.push(node.op.args[0].clone()),
                _ => unreachable!(),
            }
            assert!(
                emit_registered(&drift, "counter").is_err(),
                "{kind}: {mutation}"
            );
        }
    }
}
