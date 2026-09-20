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

fn source(shape: Shape, width: usize, descending: bool, op: &str, overwritten: bool) -> String {
    let fields = (0..width)
        .map(|n| format!("field{n}: i64"))
        .collect::<Vec<_>>()
        .join(", ");
    let literal = |value: &dyn Fn(usize) -> String| {
        let fields = (0..width)
            .rev()
            .map(|n| format!("field{n}: {}", value(n)))
            .collect::<Vec<_>>()
            .join(", ");
        format!("Packet {{ {fields} }}")
    };
    let seed = literal(&|n| format!("carry0 + index + {n}"));
    let rewrite = literal(&|n| format!("value.field{n} + {}", n + 1));
    let rotate = literal(&|n| format!("packet.field{}", (n + 1) % width));
    let checked = literal(&|n| {
        if n == 0 {
            "checked_value(seed, divisor)".into()
        } else {
            format!("value.field{n}")
        }
    });
    let sum = (0..width)
        .map(|n| format!("value.field{n} * {}", n + 1))
        .collect::<Vec<_>>()
        .join(" + ");
    let body = format!(
        "
        let carry0: i64 = carry0;
        let packet = {seed};
        let saved: Packet = packet;
        let packet: Packet = rewrite(packet);
        let updated = packet;
        let flag = index < limit;
        let marker = Marker {{ mark: index }};
        if flag {{
            let packet = rewrite(packet);
            let marker = Marker {{ mark: marker.mark + 3 }};
            let flag = false;
        }} else {{
            let packet = saved;
            let marker = Marker {{ mark: marker.mark - 2 }};
            let flag = true;
        }}
        let joined = packet;
        if flag {{ let packet = rewrite(saved); }}
        if index != limit {{
            let local = packet;
            if marker.mark != 0 {{ let local: Packet = rewrite(local); }}
            let packet = local;
        }}
        let packet = packet;
        let packet: Packet = {rotate};
        let rotated = packet;
        if divisor != 0 && (seed != ({} - 1) || divisor != -1) {{
            let packet = checked_packet(packet, seed, divisor);
        }}
        let carry0: i64 = carry0 + read(saved) + read(updated) + read(joined)
            + read(rotated) + read(packet) + marker.mark;
        if flag {{ let carry0: i64 = carry0 + 1; }}
        else {{ let carry0: i64 = carry0 - 1; }}
        {}
    ",
        i64::MIN + 1,
        if overwritten {
            "let packet = checked_packet(packet, seed, divisor); let packet = saved;"
        } else {
            ""
        }
    );
    aggregate_carried::source(shape, 1, descending, "")
        .replace("let carry0: i64 = carry0 + index;", &body)
        .replace("fn main()", &format!("
            struct Packet {{ {fields} }}
            struct Marker {{ mark: i64 }}
            @noinline fn rewrite(value: Packet) -> Packet {{ return {rewrite}; }}
            @noinline fn read(value: Packet) -> i64 {{ return {sum}; }}
            @noinline fn checked_value(value: i64, divisor: i64) -> i64 {{ return value {op} divisor; }}
            @noinline fn checked_packet(value: Packet, seed: i64, divisor: i64) -> Packet {{ return {checked}; }}
            fn main()"))
}

fn expected(
    case: Case,
    shape: Shape,
    width: usize,
    descending: bool,
    op: &str,
    overwritten: bool,
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
    let mut carry = case.seed;
    let read = |values: &[i64]| {
        values.iter().enumerate().fold(0i64, |sum, (n, value)| {
            sum.wrapping_add(value.wrapping_mul(n as i64 + 1))
        })
    };
    for current in indices {
        result.iterations += 1;
        let saved = (0..width)
            .map(|n| carry.wrapping_add(current).wrapping_add(n as i64))
            .collect::<Vec<_>>();
        let updated = saved
            .iter()
            .enumerate()
            .map(|(n, value)| value.wrapping_add(n as i64 + 1))
            .collect::<Vec<_>>();
        let before_end = current < case.limit;
        let marker = current.wrapping_add(if before_end { 3 } else { -2 });
        let joined = if before_end {
            updated
                .iter()
                .enumerate()
                .map(|(n, value)| value.wrapping_add(n as i64 + 1))
                .collect::<Vec<_>>()
        } else {
            saved.clone()
        };
        let mut packet = if before_end {
            joined.clone()
        } else {
            updated.clone()
        };
        if current != case.limit && marker != 0 {
            for (n, value) in packet.iter_mut().enumerate() {
                *value = value.wrapping_add(n as i64 + 1);
            }
        }
        packet.rotate_left(1);
        let rotated = packet.clone();
        if !danger {
            trace.push([case.seed, case.divisor, result.iterations]);
            packet[0] = match op {
                "/" => (i128::from(case.seed) / i128::from(case.divisor)) as i64,
                "%" => (i128::from(case.seed) % i128::from(case.divisor)) as i64,
                _ => unreachable!(),
            };
        }
        for values in [&saved, &updated, &joined, &rotated, &packet] {
            carry = carry.wrapping_add(read(values));
        }
        carry = carry
            .wrapping_add(marker)
            .wrapping_add(if before_end { -1 } else { 1 });
        if overwritten {
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
    }
    if case.enabled || matches!(shape, Shape::Prefix | Shape::Suffix) {
        result.leaf = Some([carry, case.divisor]);
    }
    result.state = Some(if !case.enabled && !matches!(shape, Shape::Suffix) {
        vec![case.seed, case.initial, case.seed, case.seed]
    } else {
        vec![carry, index, carry, case.seed]
    });
    result
}

fn execute(
    shape: Shape,
    width: usize,
    descending: bool,
    op: &str,
    overwritten: bool,
    cases: &[Case],
) {
    let mut traces = Vec::new();
    let cases = cases
        .iter()
        .map(|&case| {
            let mut trace = Vec::new();
            let result = expected(case, shape, width, descending, op, overwritten, &mut trace);
            traces.push(trace);
            result
        })
        .collect::<Vec<_>>();
    aggregate_loop_probe::execute_probed(
        &source(shape, width, descending, op, overwritten),
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
fn flat_rebindings_preserve_snapshots_nominal_joins_and_balanced_cleanup() {
    for (shape, width, descending, op) in [
        (Shape::Callee, 1, false, "/"),
        (Shape::Inline, 3, true, "%"),
        (Shape::Prefix, 7, false, "/"),
        (Shape::Suffix, 3, true, "%"),
    ] {
        let mut cases = Vec::new();
        for enabled in [false, true] {
            for trips in [0, 1, 4] {
                for seed in [i64::MIN, -17, i64::MAX] {
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
        execute(shape, width, descending, op, false, &cases);
    }
}

#[test]
fn overwritten_flat_rebindings_keep_checked_failures_and_zero_trip_guards() {
    for op in ["/", "%"] {
        for failing in [
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
                    BASE,
                    Case {
                        enabled: false,
                        ..failing
                    },
                    Case {
                        limit: 0,
                        ..failing
                    },
                    failing,
                ],
            );
        }
    }
}

#[test]
fn flat_rebindings_stay_behind_complete_induction_preflight() {
    for failing in [
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
        execute(Shape::Callee, 3, false, "/", false, &[failing]);
    }
}

#[test]
fn flat_rebinding_fields_keep_source_order_before_failure_or_overwrite() {
    let source = "mod cpu Main {
        struct Packet { left: i64, right: i64, extra: i64 }
        struct State { value: i64 }
        @noinline fn checked_value(value: i64, divisor: i64) -> i64 { return value / divisor; }
        @noinline fn leaf(value: i64, divisor: i64) -> i64 { return value; }
        @noinline fn work(limit: i64, seed: i64, divisor: i64) -> i64 {
            let index: i64 = 0;
            let total: i64 = 0;
            while index < limit {
                let index: i64 = index + 1;
                let packet = Packet { left: 1, right: 2, extra: 3 };
                let saved = packet;
                if index == 1 {
                    let packet = Packet { extra: checked_value(30, 1),
                        left: checked_value(seed, divisor), right: checked_value(20, 1) };
                }
                let changed = packet;
                let packet = saved;
                let total: i64 = total + packet.left + changed.left + changed.right + changed.extra;
            }
            return leaf(total, divisor);
        }
        fn start(limit: i64, seed: i64, divisor: i64) -> State { return State { value: work(limit, seed, divisor) }; }
        fn step(state: State) -> State { return state; }
        fn stop(state: State) -> State { return state; }
        fn main() -> i64 { return 0; }
    }";
    // The same source-order guarantee must hold when the record survives the backedge.
    let outer = source
        .replace(
            "let packet = Packet { left: 1, right: 2, extra: 3 };",
            "let packet = packet;",
        )
        .replace(
            "let index: i64 = 0;",
            "let packet = Packet { left: 1, right: 2, extra: 3 }; let index: i64 = 0;",
        );
    for source in [source, outer.as_str()] {
        for (seed, divisor, error) in [(10, 0, "zero"), (i64::MIN, -1, "overflow")] {
            let cases = [
                Invocation {
                    arguments: vec![Value::Int(2), Value::Int(10), Value::Int(3)],
                    state: Some(vec![61]),
                    leaf: Some([61, 3]),
                    iterations: 2,
                    reference_error: None,
                },
                Invocation {
                    arguments: vec![Value::Int(0), Value::Int(seed), Value::Int(divisor)],
                    state: Some(vec![0]),
                    leaf: Some([0, divisor]),
                    iterations: 0,
                    reference_error: None,
                },
                Invocation {
                    arguments: vec![Value::Int(2), Value::Int(seed), Value::Int(divisor)],
                    state: None,
                    leaf: None,
                    iterations: 1,
                    reference_error: Some(error),
                },
            ];
            let traces = vec![
                vec![[30, 1, 1], [10, 3, 1], [20, 1, 1]],
                vec![],
                vec![[30, 1, 1], [seed, divisor, 1]],
            ];
            aggregate_loop_probe::execute_probed(
                source,
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
    }
}

#[test]
fn flat_rebinding_private_transport_rejects_layout_and_projection_drift() {
    let source = source(Shape::Callee, 3, false, "/", false);
    let project = Project::with_source(&source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    emit_registered(&compiled.yir, "counter").unwrap();
    let call = compiled
        .yir
        .nodes
        .iter()
        .position(|node| {
            node.op.instruction == "call_owned_struct"
                && node.op.args[0].starts_with("__nuis_buffer_branch_")
        })
        .unwrap();
    for change in ["nominal", "width", "resource"] {
        let mut drift = compiled.yir.clone();
        let encoded = &mut drift.nodes[call].op.args[1];
        *encoded = match change {
            "nominal" => encoded.replacen("__nuis_scalar_carries_", "wrong_carries_", 1),
            "width" => format!("{}{{carry0:i64}}", encoded.split('{').next().unwrap()),
            "resource" => encoded.replacen(":i64", ":Bytes", 1),
            _ => unreachable!(),
        };
        assert!(emit_registered(&drift, "counter").is_err(), "{change}");
    }
    let projection = compiled
        .yir
        .nodes
        .iter()
        .position(|node| {
            node.op.instruction == "field"
                && node
                    .op
                    .args
                    .get(1)
                    .is_some_and(|field| field.starts_with("carry"))
        })
        .unwrap();
    let mut drift = compiled.yir.clone();
    drift.nodes[projection].op.args[1] = "missing_carry_field".into();
    assert!(emit_registered(&drift, "counter").is_err());
}
