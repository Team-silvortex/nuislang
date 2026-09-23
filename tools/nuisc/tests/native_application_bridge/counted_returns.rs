use super::*;

fn source(descending: bool, outer_leading: bool, child_leading: bool) -> String {
    let (cmp, op) = if descending { (">", "-") } else { ("<", "+") };
    let outer_step = format!("let index = index {op} stride;");
    let child_step = format!("let child = child {op} child_stride;");
    let (outer_head, outer_tail) = if outer_leading {
        (outer_step.as_str(), "")
    } else {
        ("", outer_step.as_str())
    };
    let (child_head, child_tail) = if child_leading {
        (child_step.as_str(), "")
    } else {
        ("", child_step.as_str())
    };
    format!("mod cpu Main {{
        struct Cell {{ sum: i64, stamp: i64 }}
        struct State {{ value: i64, index: i64, child: i64, sum: i64, stamp: i64, flag: i64, saved: i64 }}
        fn word(flag: bool) -> i64 {{ if flag {{ return 1; }} return 0; }}
        @noinline fn checked_value(value: i64, divisor: i64) -> i64 {{ return value / divisor; }}
        @noinline fn leaf(value: i64, divisor: i64) -> i64 {{ return value; }}
        fn finish(total: i64, index: i64, child: i64, cell: Cell, flag: bool, saved: bool, divisor: i64) -> State {{
            return State {{ value: leaf(checked_value(total, divisor), divisor), index: index, child: child,
                sum: cell.sum, stamp: cell.stamp, flag: word(flag), saved: word(saved) }};
        }}
        fn start(enabled: bool, initial: i64, limit: i64, stride: i64,
            child_initial: i64, child_limit: i64, child_stride: i64, stop: i64, skip: i64,
            outer_stop: i64, outer_skip: i64, divisor: i64, seed: i64) -> State {{
            let index = initial;
            let child = child_initial;
            let total = seed;
            let cell = Cell {{ sum: seed, stamp: 0 }};
            let flag = seed < 0;
            let saved = flag;
            while index {cmp} limit {{
                {outer_head}
                let child = child_initial;
                let total = total + index;
                let cell = cell;
                let flag = flag;
                if index == outer_stop {{ return finish(total, index, child, cell, flag, saved, divisor); }}
                if enabled {{
                    while child {cmp} child_limit {{
                        {child_head}
                        let flag = flag == false;
                        let cell = Cell {{ sum: cell.sum + word(flag), stamp: cell.stamp + 1 }};
                        let total = total + child;
                        if child == stop {{ return finish(total, index, child, cell, flag, saved, divisor); }}
                        if child == skip {{ {child_tail} continue; }}
                        if child == 2 {{ break; }}
                        let total = total + checked_value(child, divisor);
                        {child_tail}
                    }}
                }}
                if index == outer_skip {{ {outer_tail} continue; }}
                let total = total + 100;
                {outer_tail}
            }}
            return finish(total, index, child, cell, flag, saved, divisor);
        }}
        fn step(state: State) -> State {{ return state; }}
        fn stop(state: State) -> State {{ return state; }}
        fn main() -> i64 {{ return 0; }}
    }}")
}

fn expected(
    c: Case,
    descending: bool,
    outer_leading: bool,
    child_leading: bool,
) -> (Invocation, Vec<[i64; 3]>) {
    let (mut index, mut child, mut total, mut sum, mut stamp, mut flag) =
        (c.initial, c.child_initial, c.seed, c.seed, 0i64, c.seed < 0);
    let mut trips = 0;
    let mut calls = Vec::new();
    let mut divide = |value: i64, trips| {
        calls.push([value, c.divisor, trips]);
        if c.divisor == 0 {
            return Err(Fault::Arithmetic("zero"));
        }
        value
            .checked_div(c.divisor)
            .ok_or(Fault::Arithmetic("overflow"))
    };
    let execution = (|| {
        'outer: for next in induction(index, c.limit, c.stride, descending)? {
            trips += 1;
            if outer_leading {
                index = next;
            }
            child = c.child_initial;
            total = total.wrapping_add(index);
            if index == c.outer_stop {
                break;
            }
            if c.enabled {
                for next_child in induction(child, c.child_limit, c.child_stride, descending)? {
                    trips += 1;
                    if child_leading {
                        child = next_child;
                    }
                    flag = !flag;
                    sum = sum.wrapping_add(i64::from(flag));
                    stamp += 1;
                    total = total.wrapping_add(child);
                    if child == c.stop {
                        break 'outer;
                    }
                    if child == c.skip {
                        child = next_child;
                        continue;
                    }
                    if child == 2 {
                        break;
                    }
                    total = total.wrapping_add(divide(child, trips)?);
                    child = next_child;
                }
            }
            if index == c.outer_skip {
                index = next;
                continue;
            }
            total = total.wrapping_add(100);
            index = next;
        }
        total = divide(total, trips)?;
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
            iterations: trips,
            reference_error: match execution {
                Err(Fault::Arithmetic(e)) => Some(e),
                _ => None,
            },
        },
        calls,
    )
}

fn execute(descending: bool, outer_leading: bool, child_leading: bool, cases: &[Case]) {
    let (cases, traces): (Vec<_>, Vec<_>) = cases
        .iter()
        .map(|&c| expected(c, descending, outer_leading, child_leading))
        .unzip();
    aggregate_loop_probe::execute_probed(
        &source(descending, outer_leading, child_leading),
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
fn counted_returns_preserve_typed_snapshots_induction_and_parent_child_scope() {
    for descending in [false, true] {
        for outer_leading in [false, true] {
            for child_leading in [false, true] {
                let mut cases = Vec::new();
                for limit in [0, 1, 3] {
                    for stop in [0, 1, 2, 99] {
                        for seed in [i64::MIN, -17, 0, i64::MAX] {
                            cases.push(direction(
                                Case {
                                    limit,
                                    stop,
                                    seed,
                                    ..BASE
                                },
                                descending,
                            ));
                        }
                    }
                }
                for exit in [0, 1, 2, 3] {
                    cases.push(direction(
                        Case {
                            outer_stop: exit,
                            ..BASE
                        },
                        descending,
                    ));
                    cases.push(direction(
                        Case {
                            outer_skip: exit,
                            ..BASE
                        },
                        descending,
                    ));
                }
                cases.push(direction(
                    Case {
                        limit: 7,
                        stride: 2,
                        child_limit: 9,
                        child_stride: 3,
                        stop: 3,
                        ..BASE
                    },
                    descending,
                ));
                cases.push(direction(
                    Case {
                        enabled: false,
                        child_stride: 0,
                        ..BASE
                    },
                    descending,
                ));
                execute(descending, outer_leading, child_leading, &cases);
            }
        }
    }
}

#[test]
fn counted_returns_skip_child_preflight_but_never_waive_entered_loop_bounds() {
    for leading in [false, true] {
        let stop = i64::from(leading);
        execute(
            false,
            leading,
            !leading,
            &[
                Case {
                    outer_stop: stop,
                    child_stride: 0,
                    ..BASE
                },
                Case {
                    outer_stop: stop,
                    child_limit: 65537,
                    ..BASE
                },
                Case {
                    limit: 0,
                    stride: 0,
                    child_stride: 0,
                    ..BASE
                },
                Case {
                    outer_stop: stop,
                    stride: 0,
                    ..BASE
                },
            ],
        );
        execute(
            false,
            leading,
            !leading,
            &[Case {
                stop: 0,
                child_stride: 0,
                ..BASE
            }],
        );
        execute(
            false,
            leading,
            !leading,
            &[Case {
                stop: stop,
                child_limit: 65537,
                ..BASE
            }],
        );
        execute(
            false,
            leading,
            !leading,
            &[Case {
                outer_stop: stop,
                limit: 65537,
                ..BASE
            }],
        );
        execute(
            false,
            leading,
            !leading,
            &[Case {
                initial: i64::MAX - 1,
                limit: i64::MAX,
                stride: 2,
                outer_stop: i64::MAX - 1,
                ..BASE
            }],
        );
    }
}

#[test]
fn counted_returns_evaluate_selected_fallible_payload_before_publication() {
    for leading in [false, true] {
        execute(
            false,
            leading,
            !leading,
            &[Case {
                outer_stop: i64::from(leading),
                divisor: 0,
                child_stride: 0,
                ..BASE
            }],
        );
        execute(
            false,
            false,
            leading,
            &[Case {
                stop: i64::from(leading),
                divisor: 0,
                ..BASE
            }],
        );
        execute(
            false,
            false,
            false,
            &[Case {
                stop: 0,
                seed: i64::MIN,
                divisor: -1,
                ..BASE
            }],
        );
    }
}
