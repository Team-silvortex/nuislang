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

fn source(shape: Shape, count: usize, descending: bool, unused: bool) -> String {
    let (compare, step) = if descending { (">", "-") } else { ("<", "+") };
    let mut fields = "value: i64, index: i64, total: i64, sum: i64, stamp: i64, ".to_owned();
    let mut seeds =
        "let index: i64 = initial; let total: i64 = seed; let cell = Cell { sum: seed, stamp: 0 };"
            .to_owned();
    let mut result = "value: leaf(total, divisor), index: index, total: total, sum: cell.sum, stamp: cell.stamp, ".to_owned();
    let mut fallback = "value: seed, index: initial, total: seed, sum: seed, stamp: 0, ".to_owned();
    let mut state = "value: result.value, index: result.index, total: result.total, sum: result.sum, stamp: result.stamp, ".to_owned();
    for slot in 0..count {
        fields.push_str(&format!("flag{slot}: i64, saved{slot}: i64, "));
        seeds.push_str(&format!(
            "let flag{slot} = seed < {slot}; let saved{slot} = flag{slot};"
        ));
        result.push_str(&format!(
            "flag{slot}: word(flag{slot}), saved{slot}: word(saved{slot}), "
        ));
        fallback.push_str(&format!(
            "flag{slot}: word(seed < {slot}), saved{slot}: word(seed < {slot}), "
        ));
        state.push_str(&format!(
            "flag{slot}: result.flag{slot}, saved{slot}: result.saved{slot}, "
        ));
    }
    fields.push_str("seed: i64");
    fallback.push_str("seed: seed");
    result.push_str("seed: seed");
    state.push_str("seed: result.seed");
    let mut updates = format!(
        "let index: i64 = index {step} stride;
        let flag0 = flag0 == false;
        let before = flag0;
        if flag0 {{ let flag0 = danger || checked_value(seed, divisor) < index; }}
        else {{ let flag0 = safe && checked_value(seed, divisor) == index; }}"
    );
    if unused {
        updates.push_str("let flag0 = checked_value(seed, divisor) > 0; let flag0 = false;");
    }
    for slot in 1..count {
        updates.push_str(&format!("let flag{slot} = flag{slot} != flag{};", slot - 1));
    }
    updates.push_str(&format!(
        "let cell = Cell {{ stamp: cell.stamp + 1, sum: cell.sum + word(flag{}) }};
        let total: i64 = total + index * word(flag0) + word(before) + cell.sum;",
        count - 1
    ));
    let loop_body = format!("while index {compare} limit {{ {updates} }}");
    let calculate = format!("{seeds} {loop_body} return Parts {{ {result} }};");
    let call = "calculate(initial, limit, stride, divisor, seed, danger, safe)";
    let body = match shape {
        Shape::Callee => format!("if enabled {{ return {call}; }} return Parts {{ {fallback} }};"),
        Shape::Inline => format!("if enabled {{ {calculate} }} return Parts {{ {fallback} }};"),
        Shape::Prefix => format!(
            "let saved = {call}; if enabled {{ return saved; }} return Parts {{ {fallback} }};"
        ),
        Shape::Suffix => {
            format!("{seeds} if enabled {{ {loop_body} }} return Parts {{ {result} }};")
        }
    };
    format!("mod cpu Main {{
        struct Cell {{ sum: i64, stamp: i64 }}
        struct State {{ {fields} }} struct Parts {{ {fields} }}
        fn word(value: bool) -> i64 {{ if value {{ return 1; }} return 0; }}
        @noinline fn checked_value(value: i64, divisor: i64) -> i64 {{ return value / divisor; }}
        @noinline fn leaf(value: i64, divisor: i64) -> i64 {{ return value; }}
        @noinline fn calculate(initial: i64, limit: i64, stride: i64, divisor: i64, seed: i64, danger: bool, safe: bool) -> Parts {{ {calculate} }}
        @noinline fn compute(enabled: bool, initial: i64, limit: i64, stride: i64, divisor: i64, seed: i64, danger: bool, safe: bool) -> Parts {{ {body} }}
        fn start(enabled: bool, initial: i64, limit: i64, stride: i64, divisor: i64, seed: i64) -> State {{
            let danger = divisor == 0 || (seed == ({} - 1) && divisor == -1);
            let safe = divisor != 0 && (seed != ({} - 1) || divisor != -1);
            let result = compute(enabled, initial, limit, stride, divisor, seed, danger, safe);
            return State {{ {state} }};
        }}
        fn step(state: State) -> State {{ return state; }}
        fn stop(state: State) -> State {{ return state; }}
        fn main() -> i64 {{ print(999); return 0; }}
    }}", i64::MIN + 1, i64::MIN + 1)
}

fn expected(
    case: Case,
    shape: Shape,
    count: usize,
    descending: bool,
    unused: bool,
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
    let saved = (0..count)
        .map(|slot| case.seed < slot as i64)
        .collect::<Vec<_>>();
    let mut flags = saved.clone();
    let mut total = case.seed;
    let mut sum = case.seed;
    let danger = case.divisor == 0 || (case.seed, case.divisor) == (i64::MIN, -1);
    for &current in &indices {
        observation.iterations += 1;
        let before = !flags[0];
        flags[0] = if danger {
            before
        } else {
            trace.push([case.seed, case.divisor, observation.iterations]);
            let quotient = (i128::from(case.seed) / i128::from(case.divisor)) as i64;
            if before {
                quotient < current
            } else {
                quotient == current
            }
        };
        if unused {
            trace.push([case.seed, case.divisor, observation.iterations]);
            if danger {
                observation.reference_error = Some(if case.divisor == 0 {
                    "zero"
                } else {
                    "overflow"
                });
                return observation;
            }
            flags[0] = false;
        }
        for slot in 1..count {
            flags[slot] ^= flags[slot - 1];
        }
        sum = sum.wrapping_add(i64::from(flags[count - 1]));
        total = total
            .wrapping_add(current.wrapping_mul(i64::from(flags[0])))
            .wrapping_add(i64::from(before))
            .wrapping_add(sum);
    }
    if case.enabled || matches!(shape, Shape::Prefix | Shape::Suffix) {
        observation.leaf = Some([total, case.divisor]);
    }
    let fallback = !case.enabled && !matches!(shape, Shape::Suffix);
    let mut state = if fallback {
        vec![case.seed, case.initial, case.seed, case.seed, 0]
    } else {
        vec![total, index, total, sum, observation.iterations]
    };
    for slot in 0..count {
        state.extend([
            i64::from(if fallback { saved[slot] } else { flags[slot] }),
            i64::from(saved[slot]),
        ]);
    }
    state.push(case.seed);
    observation.state = Some(state);
    observation
}

fn execute(shape: Shape, count: usize, descending: bool, unused: bool, cases: &[Case]) {
    let mut traces = Vec::new();
    let cases = cases
        .iter()
        .map(|&case| {
            let mut trace = Vec::new();
            let result = expected(case, shape, count, descending, unused, &mut trace);
            traces.push(trace);
            result
        })
        .collect::<Vec<_>>();
    aggregate_loop_probe::execute_probed(
        &source(shape, count, descending, unused),
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
fn outer_bool_carries_match_seed_snapshot_order_and_lazy_call_oracle() {
    for (shape, count, descending) in [
        (Shape::Callee, 1, false),
        (Shape::Inline, 3, true),
        (Shape::Prefix, 7, false),
        (Shape::Suffix, 3, true),
    ] {
        let mut cases = Vec::new();
        for enabled in [false, true] {
            for trips in [0, 1, 4] {
                for seed in [i64::MIN, -17, 0, i64::MAX] {
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
        execute(shape, count, descending, false, &cases);
    }
}

#[test]
fn outer_bool_carries_do_not_hide_overwritten_checked_failures() {
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
            true,
            &[
                Case {
                    enabled: false,
                    ..case
                },
                Case { limit: 0, ..case },
                BASE,
                case,
            ],
        );
    }
}

#[test]
fn outer_bool_carries_preserve_full_preflight_before_any_iteration() {
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
fn singleton_bool_carry_uses_canonical_seed_and_backedge_words() {
    let source = "mod cpu Main {
        struct State { value: i64, saved: bool }
        fn word(value: bool) -> i64 { if value { return 1; } return 0; }
        @noinline fn leaf(value: i64, divisor: i64) -> i64 { return value; }
        @noinline fn calculate(seed: bool, limit: i64) -> i64 {
            let flag = seed; let index: i64 = 0;
            while index < limit { let index: i64 = index + 1; let flag = flag == false; }
            return leaf(word(flag), limit);
        }
        fn start(seed: bool, limit: i64) -> State {
            return State { value: calculate(seed, limit), saved: seed };
        }
        fn step(state: State) -> State { return state; }
        fn stop(state: State) -> State { return state; }
        fn main() -> i64 { return 0; }
    }";
    let cases = [false, true]
        .into_iter()
        .flat_map(|seed| {
            [0, 1, 4].into_iter().map(move |trips| {
                let value = i64::from(seed ^ (trips % 2 != 0));
                Invocation {
                    arguments: vec![Value::Bool(seed), Value::Int(trips)],
                    state: Some(vec![value, i64::from(seed)]),
                    leaf: Some([value, trips]),
                    iterations: trips,
                    reference_error: None,
                }
            })
        })
        .collect::<Vec<_>>();
    aggregate_loop_probe::execute(source, &cases, "loop_while_i64_body", false);
}

#[test]
fn outer_bool_carries_reject_raw_boolean_seeds_and_forged_cast_types() {
    let project = Project::with_source(&source(Shape::Callee, 3, false, false));
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    emit_registered(&compiled.yir, "counter").unwrap();
    let (loop_index, operand, slot, seed) = compiled
        .yir
        .nodes
        .iter()
        .enumerate()
        .find_map(|(index, node)| {
            if node.op.args.get(6).map(String::as_str) != Some("scoped_call_i64_carries") {
                return None;
            }
            node.op
                .args
                .iter()
                .enumerate()
                .skip(10)
                .find_map(|(operand, arg)| {
                    let (slot, seed) = yir_core::parse_loop_owned_struct_carry(arg).ok()??;
                    let seed = compiled
                        .yir
                        .nodes
                        .iter()
                        .find(|n| n.name == seed && n.op.instruction == "cast_bool_to_i64")?;
                    Some((index, operand, slot, seed))
                })
        })
        .expect("explicit bool seed word");
    for change in ["raw_bool", "missing", "bool_layout"] {
        let mut drift = compiled.yir.clone();
        let args = &mut drift.nodes[loop_index].op.args;
        match change {
            "raw_bool" => {
                args[operand] = yir_core::encode_loop_owned_struct_carry(slot, &seed.op.args[0])
            }
            "missing" => {
                args[operand] = yir_core::encode_loop_owned_struct_carry(slot, "missing_seed")
            }
            "bool_layout" => args[9] = args[9].replacen(":i64", ":bool", 1),
            _ => unreachable!(),
        }
        assert!(emit_registered(&drift, "counter").is_err(), "{change}");
    }
    for (kind, wrong) in [
        ("cast_bool_to_i64", "cast_i64_to_bool"),
        ("cast_i64_to_bool", "cast_bool_to_i64"),
    ] {
        let index = compiled
            .yir
            .nodes
            .iter()
            .position(|n| n.op.instruction == kind)
            .unwrap();
        for change in ["type", "missing", "arity"] {
            let mut drift = compiled.yir.clone();
            let op = &mut drift.nodes[index].op;
            match change {
                "type" => op.instruction = wrong.into(),
                "missing" => op.args[0] = "missing_value".into(),
                "arity" => op.args.push(op.args[0].clone()),
                _ => unreachable!(),
            }
            assert!(
                emit_registered(&drift, "counter").is_err(),
                "{kind}: {change}"
            );
        }
    }
}
