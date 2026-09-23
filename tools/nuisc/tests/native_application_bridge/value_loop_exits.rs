use super::*;
use aggregate_loop_probe::{CallProbe, Invocation};

#[path = "trailing_value_loops.rs"]
mod trailing_value_loops;

#[path = "counted_returns.rs"]
mod counted_returns;

#[derive(Clone, Copy)]
struct Case {
    enabled: bool,
    initial: i64,
    limit: i64,
    stride: i64,
    child_initial: i64,
    child_limit: i64,
    child_stride: i64,
    stop: i64,
    skip: i64,
    outer_stop: i64,
    outer_skip: i64,
    divisor: i64,
    seed: i64,
}

const BASE: Case = Case {
    enabled: true,
    initial: 0,
    limit: 3,
    stride: 1,
    child_initial: 0,
    child_limit: 4,
    child_stride: 1,
    stop: 2,
    skip: 1,
    outer_stop: 99,
    outer_skip: 99,
    divisor: 3,
    seed: -17,
};

fn source(descending: bool, late: bool, overwritten: bool) -> String {
    let (compare, step, distance) = if descending {
        (">", "-", "initial - index")
    } else {
        ("<", "+", "index - initial")
    };
    let stride = if late {
        format!("child_stride + 2 - ({distance})")
    } else {
        "child_stride".to_owned()
    };
    let overwrite = if overwritten { "let total = seed;" } else { "" };
    format!("mod cpu Main {{
        struct Cell {{ sum: i64, stamp: i64 }}
        struct State {{ value: i64, index: i64, child: i64, sum: i64, stamp: i64, flag: i64, saved: i64 }}
        fn word(value: bool) -> i64 {{ if value {{ return 1; }} return 0; }}
        @noinline fn checked_value(value: i64, divisor: i64) -> i64 {{ return value / divisor; }}
        @noinline fn leaf(value: i64, divisor: i64) -> i64 {{ return value; }}
        fn start(enabled: bool, initial: i64, limit: i64, stride: i64,
            child_initial: i64, child_limit: i64, child_stride: i64, stop: i64, skip: i64,
            outer_stop: i64, outer_skip: i64, divisor: i64, seed: i64) -> State {{
            let index = initial;
            let child = child_initial;
            let total = seed;
            let flag = seed < 0;
            let saved = flag;
            let cell = Cell {{ sum: seed, stamp: 0 }};
            while index {compare} limit {{
                let index = index {step} stride;
                let child = child_initial;
                let effective_stride = {stride};
                if enabled {{
                    while child {compare} child_limit {{
                        let child = child {step} effective_stride;
                        let flag = flag == false;
                        let cell = Cell {{ sum: cell.sum + word(flag), stamp: cell.stamp + 1 }};
                        if child == stop {{ let total = total + word(flag); break; }}
                        if child == skip {{ let total = total + child; continue; }}
                        let before = cell;
                        let total = checked_value(total, divisor) + before.sum;
                        {overwrite}
                    }}
                }}
                let total = total + child;
                if index == outer_stop {{ break; }}
                if index == outer_skip {{ continue; }}
                let total = total + index;
            }}
            return State {{ value: leaf(total, divisor), index: index, child: child, sum: cell.sum,
                stamp: cell.stamp, flag: word(flag), saved: word(saved) }};
        }}
        fn step(state: State) -> State {{ return state; }}
        fn stop(state: State) -> State {{ return state; }}
        fn main() -> i64 {{ return 0; }}
    }}")
}

enum Fault {
    Preflight,
    Arithmetic(&'static str),
}

fn arguments(c: Case) -> Vec<Value> {
    let mut values = vec![Value::Bool(c.enabled)];
    values.extend(
        [
            c.initial,
            c.limit,
            c.stride,
            c.child_initial,
            c.child_limit,
            c.child_stride,
            c.stop,
            c.skip,
            c.outer_stop,
            c.outer_skip,
            c.divisor,
            c.seed,
        ]
        .into_iter()
        .map(Value::Int),
    );
    values
}

fn induction(mut index: i64, limit: i64, stride: i64, descending: bool) -> Result<Vec<i64>, Fault> {
    let mut values = Vec::new();
    while if descending {
        index > limit
    } else {
        index < limit
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

fn expected(
    c: Case,
    descending: bool,
    late: bool,
    overwritten: bool,
) -> (Invocation, Vec<[i64; 3]>) {
    let (mut index, mut child, mut total, mut sum, mut stamp, mut flag) =
        (c.initial, c.child_initial, c.seed, c.seed, 0i64, c.seed < 0);
    let mut iterations = 0;
    let mut calls = Vec::new();
    let execution = (|| {
        for current in induction(c.initial, c.limit, c.stride, descending)? {
            index = current;
            iterations += 1;
            child = c.child_initial;
            let distance = if descending {
                c.initial.wrapping_sub(index)
            } else {
                index.wrapping_sub(c.initial)
            };
            let stride = if late {
                c.child_stride.wrapping_add(2).wrapping_sub(distance)
            } else {
                c.child_stride
            };
            if c.enabled {
                for current in induction(child, c.child_limit, stride, descending)? {
                    child = current;
                    iterations += 1;
                    flag = !flag;
                    sum = sum.wrapping_add(i64::from(flag));
                    stamp += 1;
                    if child == c.stop {
                        total = total.wrapping_add(i64::from(flag));
                        break;
                    }
                    if child == c.skip {
                        total = total.wrapping_add(child);
                        continue;
                    }
                    calls.push([total, c.divisor, iterations]);
                    if c.divisor == 0 {
                        return Err(Fault::Arithmetic("zero"));
                    }
                    if (total, c.divisor) == (i64::MIN, -1) {
                        return Err(Fault::Arithmetic("overflow"));
                    }
                    total = (i128::from(total) / i128::from(c.divisor) + i128::from(sum)) as i64;
                    if overwritten {
                        total = c.seed;
                    }
                }
            }
            total = total.wrapping_add(child);
            if index == c.outer_stop {
                break;
            }
            if index == c.outer_skip {
                continue;
            }
            total = total.wrapping_add(index);
        }
        Ok(())
    })();
    let success = execution.is_ok();
    (
        Invocation {
            arguments: arguments(c),
            state: success.then(|| {
                vec![
                    total,
                    index,
                    child,
                    sum,
                    stamp,
                    i64::from(flag),
                    i64::from(c.seed < 0),
                ]
            }),
            leaf: success.then_some([total, c.divisor]),
            iterations,
            reference_error: match execution {
                Err(Fault::Arithmetic(error)) => Some(error),
                _ => None,
            },
        },
        calls,
    )
}

fn execute(descending: bool, late: bool, overwritten: bool, cases: &[Case]) {
    let (observations, traces): (Vec<_>, Vec<_>) = cases
        .iter()
        .map(|&c| expected(c, descending, late, overwritten))
        .unzip();
    aggregate_loop_probe::execute_probed(
        &source(descending, late, overwritten),
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

fn direction(mut c: Case, descending: bool) -> Case {
    if descending {
        std::mem::swap(&mut c.initial, &mut c.limit);
        std::mem::swap(&mut c.child_initial, &mut c.child_limit);
    }
    c
}

#[test]
fn value_loop_exits_keep_nested_scope_advanced_indices_and_typed_snapshots() {
    for descending in [false, true] {
        let mut cases = Vec::new();
        for limit in [0, 1, 3] {
            for stop in [1, 2, 99] {
                for skip in [1, 2, 99] {
                    for seed in [i64::MIN, -17, 0, i64::MAX] {
                        cases.push(direction(
                            Case {
                                limit,
                                stop,
                                skip,
                                seed,
                                ..BASE
                            },
                            descending,
                        ));
                    }
                }
            }
        }
        cases.extend([
            direction(
                Case {
                    outer_stop: 2,
                    ..BASE
                },
                descending,
            ),
            direction(
                Case {
                    outer_skip: 2,
                    ..BASE
                },
                descending,
            ),
        ]);
        execute(descending, false, false, &cases);
    }
}

#[test]
fn value_loop_exits_skip_only_their_suffix_and_preserve_selected_failures() {
    for descending in [false, true] {
        early_parent_exits_skip_child_preflight(descending);
        let base = direction(BASE, descending);
        let first = if descending {
            base.child_initial - 1
        } else {
            base.child_initial + 1
        };
        let skipped = [
            Case {
                enabled: false,
                child_stride: 0,
                divisor: 0,
                ..base
            },
            Case {
                child_limit: base.child_initial,
                child_stride: 0,
                divisor: 0,
                ..base
            },
            Case {
                stop: first,
                divisor: 0,
                ..base
            },
            Case {
                child_limit: first,
                skip: first,
                divisor: 0,
                ..base
            },
        ];
        for overwritten in [false, true] {
            for bad in [
                Case {
                    stop: 99,
                    skip: 99,
                    divisor: 0,
                    ..base
                },
                Case {
                    stop: 99,
                    skip: 99,
                    seed: i64::MIN,
                    divisor: -1,
                    ..base
                },
            ] {
                let mut cases = skipped.to_vec();
                cases.push(bad);
                execute(descending, false, overwritten, &cases);
            }
        }
    }
}

fn early_parent_exits_skip_child_preflight(descending: bool) {
    let guards =
        "if index == outer_stop { break; }\n                if index == outer_skip { continue; }";
    let step = if descending { "-" } else { "+" };
    let induction = format!("let index = index {step} stride;");
    let source = source(descending, false, false)
        .replace(guards, "")
        .replace(
            &induction,
            &format!("{induction} let total = total + index; {guards}"),
        );
    let base = direction(
        Case {
            child_stride: 0,
            divisor: 0,
            ..BASE
        },
        descending,
    );
    let first = if descending {
        base.initial - 1
    } else {
        base.initial + 1
    };
    let mut cases = [
        Case {
            outer_stop: first,
            ..base
        },
        Case {
            limit: first,
            outer_skip: first,
            ..base
        },
    ]
    .into_iter()
    .map(|c| Invocation {
        arguments: arguments(c),
        state: Some(vec![
            c.seed + first,
            first,
            c.child_initial,
            c.seed,
            0,
            i64::from(c.seed < 0),
            i64::from(c.seed < 0),
        ]),
        leaf: Some([c.seed + first, c.divisor]),
        iterations: 1,
        reference_error: None,
    })
    .collect::<Vec<_>>();
    // Continue skips the invalid child once, not forever: the second parent
    // iteration reaches its preflight before any child body or checked call.
    cases.push(Invocation {
        arguments: arguments(Case {
            outer_skip: first,
            ..base
        }),
        state: None,
        leaf: None,
        iterations: 2,
        reference_error: None,
    });
    let traces = vec![Vec::new(); cases.len()];
    aggregate_loop_probe::execute_probed(
        &source,
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
fn value_loop_exits_do_not_bypass_full_or_late_induction_preflight() {
    for descending in [false, true] {
        let base = direction(BASE, descending);
        for bad in [
            Case { stride: 0, ..base },
            Case {
                child_stride: 0,
                ..base
            },
            direction(
                Case {
                    limit: 65537,
                    outer_stop: 1,
                    ..BASE
                },
                descending,
            ),
            direction(
                Case {
                    child_limit: 65537,
                    stop: 1,
                    ..BASE
                },
                descending,
            ),
        ] {
            execute(descending, false, false, &[bad]);
        }
        execute(descending, true, false, &[base]);
    }
    execute(
        false,
        false,
        false,
        &[Case {
            limit: 65536,
            outer_stop: 1,
            ..BASE
        }],
    );
    execute(
        false,
        false,
        false,
        &[Case {
            initial: i64::MAX - 1,
            limit: i64::MAX,
            stride: 2,
            outer_stop: i64::MAX,
            ..BASE
        }],
    );
}
