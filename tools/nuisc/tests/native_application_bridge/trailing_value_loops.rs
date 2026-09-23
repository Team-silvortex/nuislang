use super::*;

fn source(
    descending: bool,
    outer_leading: bool,
    child_leading: bool,
    late: bool,
    overwritten: bool,
) -> String {
    let mut text = super::source(descending, late, overwritten);
    let op = if descending { "-" } else { "+" };
    if !child_leading {
        let step = format!("let child = child {op} effective_stride;");
        text = text.replace(&step, "").replace(
            "let total = total + child; continue;",
            &format!("let total = total + child; {step} continue;"),
        ).replace(
            "                    }\n                }\n                let total = total + child;",
            &format!("                        {step}\n                    }}\n                }}\n                let total = total + child;"),
        );
    }
    if !outer_leading {
        let step = format!("let index = index {op} stride;");
        text = text
            .replace(&step, "")
            .replace(
                "if index == outer_skip { continue; }",
                &format!("if index == outer_skip {{ {step} continue; }}"),
            )
            .replace(
                "let total = total + index;\n            }",
                &format!("let total = total + index;\n                {step}\n            }}"),
            );
    }
    text
}

fn expected(
    c: Case,
    descending: bool,
    outer_leading: bool,
    child_leading: bool,
    late: bool,
    overwritten: bool,
) -> (Invocation, Vec<[i64; 3]>) {
    let (mut index, mut child, mut total, mut sum, mut stamp, mut flag) =
        (c.initial, c.child_initial, c.seed, c.seed, 0i64, c.seed < 0);
    let mut iterations = 0;
    let mut calls = Vec::new();
    let execution = (|| {
        for next in induction(index, c.limit, c.stride, descending)? {
            if outer_leading {
                index = next;
            }
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
                for child_next in induction(child, c.child_limit, stride, descending)? {
                    if child_leading {
                        child = child_next;
                    }
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
                        child = child_next;
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
                    child = child_next;
                }
            }
            total = total.wrapping_add(child);
            if index == c.outer_stop {
                break;
            }
            if index == c.outer_skip {
                index = next;
                continue;
            }
            total = total.wrapping_add(index);
            index = next;
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

fn execute(
    descending: bool,
    outer_leading: bool,
    child_leading: bool,
    late: bool,
    overwritten: bool,
    cases: &[Case],
) {
    let (observations, traces): (Vec<_>, Vec<_>) = cases
        .iter()
        .map(|&c| {
            expected(
                c,
                descending,
                outer_leading,
                child_leading,
                late,
                overwritten,
            )
        })
        .unzip();
    aggregate_loop_probe::execute_probed(
        &source(descending, outer_leading, child_leading, late, overwritten),
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
fn trailing_value_loops_keep_pre_step_indices_mixed_child_scope_and_typed_snapshots() {
    for descending in [false, true] {
        for (outer_leading, child_leading) in [(false, false), (false, true), (true, false)] {
            let mut cases = Vec::new();
            for limit in [0, 1, 3] {
                for stop in [0, 2, 99] {
                    for skip in [0, 1, 99] {
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
                    ..BASE
                },
                descending,
            ));
            let mut stepped = direction(
                Case {
                    limit: 7,
                    stride: 2,
                    child_limit: 9,
                    child_stride: 3,
                    ..BASE
                },
                descending,
            );
            stepped.skip = stepped.child_initial;
            stepped.stop = stepped.child_initial + if descending { -3 } else { 3 };
            cases.push(stepped);
            execute(
                descending,
                outer_leading,
                child_leading,
                false,
                false,
                &cases,
            );
        }
    }
}

#[test]
fn trailing_value_exits_skip_only_unselected_fallible_effects() {
    for descending in [false, true] {
        early_parent_exits_skip_child_preflight(descending);
        let base = direction(BASE, descending);
        let first = base.child_initial;
        let next = first + if descending { -1 } else { 1 };
        let skipped = [
            Case {
                enabled: false,
                child_stride: 0,
                divisor: 0,
                ..base
            },
            Case {
                child_limit: first,
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
                child_limit: next,
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
                execute(descending, false, false, false, overwritten, &cases);
            }
        }
    }
}

fn early_parent_exits_skip_child_preflight(descending: bool) {
    let op = if descending { "-" } else { "+" };
    let guards = format!("if index == outer_stop {{ break; }}\n                if index == outer_skip {{ let index = index {op} stride; continue; }}");
    let text = source(descending, false, false, false, false);
    assert_eq!(text.matches(&guards).count(), 1);
    let text = text.replace(&guards, "").replace(
        "                let child = child_initial;",
        &format!("                let total = total + index;\n                {guards}\n                let child = child_initial;"),
    );
    let base = direction(BASE, descending);
    let next = base.initial + if descending { -1 } else { 1 };
    let breaks = Case {
        outer_stop: base.initial,
        child_stride: 0,
        divisor: 0,
        ..base
    };
    let continues = Case {
        limit: next,
        outer_skip: base.initial,
        child_stride: 0,
        divisor: 0,
        ..base
    };
    let later = Case {
        limit: base.limit,
        ..continues
    };
    let mut observations = Vec::new();
    for (case, index) in [(breaks, base.initial), (continues, next)] {
        let total = case.seed.wrapping_add(case.initial);
        observations.push(Invocation {
            arguments: arguments(case),
            state: Some(vec![
                total,
                index,
                case.child_initial,
                case.seed,
                0,
                i64::from(case.seed < 0),
                i64::from(case.seed < 0),
            ]),
            leaf: Some([total, 0]),
            iterations: 1,
            reference_error: None,
        });
    }
    observations.push(Invocation {
        arguments: arguments(later),
        state: None,
        leaf: None,
        iterations: 2,
        reference_error: None,
    });
    aggregate_loop_probe::execute_probed(
        &text,
        &observations,
        "loop_while_i64_body",
        true,
        None,
        Some(CallProbe {
            callee: "checked_value",
            traces: &[vec![], vec![], vec![]],
        }),
    );
}

#[test]
fn trailing_value_exits_preserve_full_and_late_induction_preflight() {
    for descending in [false, true] {
        let base = direction(BASE, descending);
        for bad in [
            Case {
                stride: 0,
                outer_stop: base.initial,
                ..base
            },
            Case {
                child_stride: 0,
                stop: base.child_initial,
                ..base
            },
            Case {
                limit: base.initial + if descending { -65537 } else { 65537 },
                outer_stop: base.initial,
                ..base
            },
            Case {
                initial: if descending {
                    i64::MIN + 1
                } else {
                    i64::MAX - 1
                },
                limit: if descending { i64::MIN } else { i64::MAX },
                stride: 2,
                ..base
            },
        ] {
            execute(
                descending,
                false,
                false,
                false,
                false,
                &[
                    Case {
                        limit: base.initial,
                        stride: 0,
                        child_stride: 0,
                        divisor: 0,
                        ..base
                    },
                    bad,
                ],
            );
        }
        let late = direction(
            Case {
                limit: 5,
                child_stride: 1,
                stop: 99,
                skip: 99,
                ..BASE
            },
            descending,
        );
        execute(
            descending,
            false,
            false,
            true,
            false,
            &[
                Case {
                    enabled: false,
                    divisor: 0,
                    ..late
                },
                late,
            ],
        );
    }
}
