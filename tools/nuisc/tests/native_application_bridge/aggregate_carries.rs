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
    let seeds = literal(&|n| format!("seed + {n}"));
    let rotated = literal(&|n| format!("packet.field{} + index", (n + 1) % width));
    let rewritten = literal(&|n| format!("value.field{n} + {}", n + 1));
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
    let declarations = format!(
        "let index: i64 = initial; let packet = {seeds}; let saved = packet;
        let marker = Marker {{ stamp: seed - 3 }}; let total: i64 = seed;"
    );
    let (compare, step) = if descending { (">", "-") } else { ("<", "+") };
    let body = format!(
        "while index {compare} limit {{
        let index: i64 = index {step} stride;
        let packet = packet;
        let before = packet;
        let packet = {rotated};
        if index < limit {{ let packet = rewrite(packet); }}
        else {{ let packet = before; }}
        if index != limit {{ if index != 0 {{ let packet = rewrite(packet); }} }}
        let marker = Marker {{ stamp: marker.stamp + packet.field0 }};
        let total: i64 = total + read(packet) + read(saved) + marker.stamp;
        if divisor != 0 && (seed != ({} - 1) || divisor != -1) {{
            let packet = checked_packet(packet, seed, divisor);
        }}
        {}
    }}",
        i64::MIN + 1,
        if overwritten {
            "let packet = checked_packet(packet, seed, divisor); let packet = before;"
        } else {
            ""
        }
    );
    let mut state_fields = "value: i64, index: i64, total: i64, marker: i64".to_owned();
    let mut result =
        "value: leaf(total, divisor), index: index, total: total, marker: marker.stamp".to_owned();
    let mut fallback = "value: seed, index: initial, total: seed, marker: seed - 3".to_owned();
    for prefix in ["packet", "saved"] {
        for n in 0..width {
            state_fields.push_str(&format!(", {prefix}{n}: i64"));
            result.push_str(&format!(", {prefix}{n}: {prefix}.field{n}"));
            fallback.push_str(&format!(", {prefix}{n}: seed + {n}"));
        }
    }
    let returned = format!("return State {{ {result} }};");
    let fallback = format!("State {{ {fallback} }}");
    let calculate = format!("{declarations} {body} {returned}");
    let call = "calculate(initial, limit, stride, divisor, seed)";
    let selected = match shape {
        Shape::Callee => format!("if enabled {{ return {call}; }} return {fallback};"),
        Shape::Inline => format!("if enabled {{ {calculate} }} return {fallback};"),
        Shape::Prefix => {
            format!("let result = {call}; if enabled {{ return result; }} return {fallback};")
        }
        Shape::Suffix => format!("{declarations} if enabled {{ {body} }} {returned}"),
    };
    format!("mod cpu Main {{
        struct Packet {{ {fields} }}
        struct Marker {{ stamp: i64 }}
        struct State {{ {state_fields} }}
        @noinline fn leaf(value: i64, divisor: i64) -> i64 {{ return value; }}
        @noinline fn checked_value(value: i64, divisor: i64) -> i64 {{ return value {op} divisor; }}
        @noinline fn rewrite(value: Packet) -> Packet {{ return {rewritten}; }}
        @noinline fn read(value: Packet) -> i64 {{ return {sum}; }}
        @noinline fn checked_packet(value: Packet, seed: i64, divisor: i64) -> Packet {{ return {checked}; }}
        @noinline fn calculate(initial: i64, limit: i64, stride: i64, divisor: i64, seed: i64) -> State {{ {calculate} }}
        @noinline fn compute(enabled: bool, initial: i64, limit: i64, stride: i64, divisor: i64, seed: i64) -> State {{ {selected} }}
        fn start(enabled: bool, initial: i64, limit: i64, stride: i64, divisor: i64, seed: i64) -> State {{
            return compute(enabled, initial, limit, stride, divisor, seed);
        }}
        fn step(state: State) -> State {{ return state; }}
        fn stop(state: State) -> State {{ return state; }}
        fn main() -> i64 {{ return 0; }}
    }}")
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
    let mut index = case.initial;
    let mut indices = Vec::new();
    if case.enabled || matches!(shape, Shape::Prefix) {
        while if descending {
            index > case.limit
        } else {
            index < case.limit
        } {
            if indices.len() == 65536 || case.stride <= 0 {
                return observation;
            }
            let Some(next) = (if descending {
                index.checked_sub(case.stride)
            } else {
                index.checked_add(case.stride)
            }) else {
                return observation;
            };
            index = next;
            indices.push(index);
        }
    }
    let saved = (0..width)
        .map(|n| case.seed.wrapping_add(n as i64))
        .collect::<Vec<_>>();
    let mut packet = saved.clone();
    let mut marker = case.seed.wrapping_sub(3);
    let mut total = case.seed;
    let read = |value: &[i64]| {
        value.iter().enumerate().fold(0i64, |sum, (n, value)| {
            sum.wrapping_add(value.wrapping_mul(n as i64 + 1))
        })
    };
    let rewrite = |value: &mut Vec<i64>| {
        for (n, value) in value.iter_mut().enumerate() {
            *value = value.wrapping_add(n as i64 + 1);
        }
    };
    let danger = case.divisor == 0 || (case.seed, case.divisor) == (i64::MIN, -1);
    for current in indices {
        observation.iterations += 1;
        let before = packet.clone();
        packet.rotate_left(1);
        for value in &mut packet {
            *value = value.wrapping_add(current);
        }
        if current < case.limit {
            rewrite(&mut packet);
        } else {
            packet = before.clone();
        }
        if current != case.limit && current != 0 {
            rewrite(&mut packet);
        }
        marker = marker.wrapping_add(packet[0]);
        total = total
            .wrapping_add(read(&packet))
            .wrapping_add(read(&saved))
            .wrapping_add(marker);
        if !danger {
            trace.push([case.seed, case.divisor, observation.iterations]);
            packet[0] = match op {
                "/" => (i128::from(case.seed) / i128::from(case.divisor)) as i64,
                "%" => (i128::from(case.seed) % i128::from(case.divisor)) as i64,
                _ => unreachable!(),
            };
        }
        if overwritten {
            trace.push([case.seed, case.divisor, observation.iterations]);
            if danger {
                observation.reference_error = Some(if case.divisor == 0 {
                    "zero"
                } else {
                    "overflow"
                });
                return observation;
            }
            packet = before;
        }
    }
    if case.enabled || matches!(shape, Shape::Prefix | Shape::Suffix) {
        observation.leaf = Some([total, case.divisor]);
    }
    let mut state = if !case.enabled && !matches!(shape, Shape::Suffix) {
        packet = saved.clone();
        vec![
            case.seed,
            case.initial,
            case.seed,
            case.seed.wrapping_sub(3),
        ]
    } else {
        vec![total, index, total, marker]
    };
    state.extend(packet);
    state.extend(saved);
    observation.state = Some(state);
    observation
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
            let observation = expected(case, shape, width, descending, op, overwritten, &mut trace);
            traces.push(trace);
            observation
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
fn outer_flat_carries_match_zero_trip_seeds_snapshots_and_ordered_oracle() {
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
fn outer_flat_carries_keep_overwritten_checked_failures_and_skipped_guards() {
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
fn outer_flat_carries_stay_behind_complete_induction_preflight() {
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
fn outer_flat_carries_reject_transport_layout_slot_and_seed_drift() {
    let project = Project::with_source(&source(Shape::Callee, 3, false, "/", false));
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    emit_registered(&compiled.yir, "counter").unwrap();
    let slot = compiled
        .yir
        .nodes
        .iter()
        .position(|node| {
            node.op
                .args
                .get(6)
                .is_some_and(|action| action == "scoped_call_i64_carries")
        })
        .unwrap();
    for change in [
        "nominal",
        "width",
        "resource",
        "duplicate",
        "missing",
        "seed",
    ] {
        let mut drift = compiled.yir.clone();
        let args = &mut drift.nodes[slot].op.args;
        match change {
            "nominal" => args[9] = args[9].replacen("__nuis_scalar_carries_", "wrong_carries_", 1),
            "width" => args[9] = format!("{}{{carry0:i64}}", args[9].split('{').next().unwrap()),
            "resource" => args[9] = args[9].replacen(":i64", ":Bytes", 1),
            _ => {
                let positions = (10..args.len())
                    .filter(|&n| {
                        yir_core::parse_loop_owned_struct_carry(&args[n])
                            .unwrap()
                            .is_some()
                    })
                    .collect::<Vec<_>>();
                match change {
                    "duplicate" => args[positions[0]] = args[positions[1]].clone(),
                    "missing" => args[positions[0]] = "$current".into(),
                    "seed" => {
                        args[positions[0]] =
                            yir_core::encode_loop_owned_struct_carry(0, "missing_seed")
                    }
                    _ => unreachable!(),
                }
            }
        }
        assert!(emit_registered(&drift, "counter").is_err(), "{change}");
    }
}

#[test]
fn singleton_flat_carry_uses_value_transport_and_retains_immutable_scalar_captures() {
    let source = "mod cpu Main {
        struct Marker { value: i64 }
        struct State { value: i64 }
        @noinline fn leaf(value: i64, divisor: i64) -> i64 { return value; }
        @noinline fn work(limit: i64, seed: i64) -> i64 {
            let marker = Marker { value: seed };
            let saved = marker;
            let index: i64 = 0;
            while index < limit { let index: i64 = index + 1; let marker = marker; }
            return leaf(marker.value + saved.value + index, 1);
        }
        fn start(limit: i64, seed: i64) -> State { return State { value: work(limit, seed) }; }
        fn step(state: State) -> State { return state; }
        fn stop(state: State) -> State { return state; }
        fn main() -> i64 { return 0; }
    }";
    let observations = |scalar: bool| {
        [0, 3]
            .into_iter()
            .map(|limit| {
                let value = if scalar {
                    7 * limit + 14 + limit
                } else {
                    14 + limit
                };
                Invocation {
                    arguments: vec![Value::Int(limit), Value::Int(7)],
                    state: Some(vec![value]),
                    leaf: Some([value, 1]),
                    iterations: limit,
                    reference_error: None,
                }
            })
            .collect::<Vec<_>>()
    };
    aggregate_loop_probe::execute(source, &observations(false), "loop_while_i64_body", false);
    let scalar = source
        .replace(
            "let index: i64 = 0;",
            "let total: i64 = 0; let index: i64 = 0;",
        )
        .replace(
            "let marker = marker;",
            "let total: i64 = total + saved.value;",
        )
        .replace(
            "marker.value + saved.value + index",
            "total + marker.value + saved.value + index",
        );
    aggregate_loop_probe::execute(&scalar, &observations(true), "loop_while_i64_body", false);
}
