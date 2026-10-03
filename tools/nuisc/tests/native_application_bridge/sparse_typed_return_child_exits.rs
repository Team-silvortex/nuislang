use super::*;

#[test]
fn typed_nested_preheader_snapshots_keep_explicit_private_iteration_arities() {
    let base = fixture::child_return_source(64);
    let observed = base.replace("tag: i32_from_i64(9)", "tag: i32_from_i64(9 + k)");
    let extra = (9..64)
        .map(|i| format!(", extra{i}: carry.extra{i}"))
        .collect::<String>();
    let mut cases = vec![
        ("full", fixture::return_source(64), vec![5, 11]),
        ("narrow", fixture::return_source(9), vec![9, 11]),
        (
            "changed-tag",
            fixture::return_source(64).replace(
                "tag: selected.tag, enabled: !enabled",
                "tag: i32_from_i64(17), enabled: !enabled",
            ),
            vec![4, 11],
        ),
        (
            "joined-preheader",
            fixture::joined_return_source(64),
            vec![4, 11],
        ),
        (
            "parent-entry",
            fixture::parent_return_source(64),
            vec![6, 12],
        ),
        (
            "post-loop",
            fixture::post_loop_return_source(64, ""),
            vec![5, 6, 12],
        ),
        (
            "post-loop-break",
            fixture::post_loop_return_source(64, "break;"),
            vec![6, 6, 12],
        ),
        (
            "post-loop-continue",
            fixture::post_loop_return_source(64, "continue;"),
            vec![5, 6, 12],
        ),
        ("unobserved", base, vec![2, 5, 11]),
    ];
    for continuing in [false, true] {
        let exit = if continuing { "continue" } else { "break" };
        let source = observed.replace("if k == 1 { break; }", &format!("if k == 1 {{ {exit}; }}"));
        cases.push((
            "observed",
            source.clone(),
            vec![if continuing { 1 } else { 3 }, 5, 11],
        ));
        cases.push(("partial", source.replace(&format!("if k == 1 {{ {exit}; }}"),
            &format!("let snapshot = carry; let checked_snapshot = 10 / snapshot.count; if k == 1 {{ {exit}; }}")),
            vec![if continuing { 2 } else { 4 }, 6, 11]));
        cases.push(("opaque", source.replace(&format!("if k == 1 {{ {exit}; }}"),
            &format!("let snapshot = relay(carry); let checked_snapshot = 10 / snapshot.count; if k == 1 {{ {exit}; }}")),
            vec![if continuing { 4 } else { 6 }, 6, 11]));
        cases.push(("checked", source.replace(&format!("if k == 1 {{ {exit}; }}"), &format!(
            "let snapshot = State {{ left: carry.left, right: carry.right, count: carry.count + 1{extra} }}; let checked_snapshot = 10 / (snapshot.count - carry.count); if k == 1 {{ {exit}; }}")),
            vec![if continuing { 2 } else { 4 }, 6, 11]));
    }
    let mut actual = Vec::new();
    let mut expected = Vec::new();
    for (name, source, arities) in cases {
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let bridge = emit_registered(&compiled.yir, "counter").unwrap_or_else(|error| {
            let wide = compiled
                .yir
                .functions
                .iter()
                .filter(|f| f.parameters.len() > 64 || f.body_nodes.len() > 4096)
                .map(|f| (&f.name, f.parameters.len(), f.body_nodes.len()))
                .collect::<Vec<_>>();
            panic!("{name}: {error}; wide helpers: {wide:?}");
        });
        let mut widths = bridge
            .llvm_ir
            .lines()
            .filter(|line| {
                line.starts_with("define ") && line.contains("@nuis_fn___nuis_scalar_iteration_")
            })
            .map(|line| line.matches(" %arg").count())
            .collect::<Vec<_>>();
        widths.sort_unstable();
        actual.push((name, widths));
        expected.push((name, arities));
    }
    assert_eq!(actual, expected);
}

#[test]
fn typed_nested_returns_cross_ordinary_child_exits_with_64_word_state() {
    for width in [9, 63, 64] {
        for continuing in [false, true] {
            check(width, continuing, false, false);
        }
    }
}

#[test]
fn typed_nested_returns_preserve_observed_child_indices_at_full_width() {
    for width in [9, 63, 64] {
        for continuing in [false, true] {
            check(width, continuing, true, false);
        }
    }
}

#[test]
fn typed_nested_child_exits_do_not_infer_invariance_through_opaque_calls() {
    let source = fixture::child_return_source(64)
        .replace(
            "let k = k + 1;\n                let carry = carry;",
            "let k = k + 1;\n                let carry = relay(carry);",
        )
        .replace("tag: i32_from_i64(9)", "tag: i32_from_i64(9 + k)");
    let compiled = nuisc::pipeline::compile_source(&source).unwrap();
    assert!(carried_widths(&compiled.yir)
        .iter()
        .any(|width| *width > 64));
    let project = Project::with_source(&source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let error = emit_registered(&compiled.yir, "counter").unwrap_err();
    assert!(
        error.contains("carried words exceed the 64-word native limit"),
        "{error}"
    );
}

#[test]
fn typed_nested_child_exits_project_partly_observed_reconstructions() {
    for width in [9, 63, 64] {
        for continuing in [false, true] {
            check(width, continuing, true, true);
        }
    }
}

#[test]
fn typed_nested_child_breaks_preserve_opaque_snapshot_calls_with_readonly_record_inputs() {
    for width in [9, 63, 64] {
        for observed in [false, true] {
            let extra = (9..width)
                .map(|i| format!(", extra{i}: value.extra{i}"))
                .collect::<String>();
            let mut source = fixture::child_return_source(width).replace("fn step(state:", &format!(
                "@noinline fn inspect(value: State) -> State {{ return State {{ left: value.left, right: value.right, count: value.count + 1{extra} }}; }} fn step(state:"
            )).replace(
                "if k == 1 { break; }",
                "let snapshot = inspect(carry); let checked_snapshot = 10 / (snapshot.count - carry.count); if k == 1 { break; }",
            );
            if observed {
                source = source.replace("tag: i32_from_i64(9)", "tag: i32_from_i64(9 + k)");
            }
            check_source(&source, width, false, observed);
        }
    }
}

#[test]
fn typed_nested_opaque_continue_snapshots_preserve_readonly_inputs() {
    for width in [9, 63, 64] {
        for observed in [false, true] {
            let extra = (9..width)
                .map(|i| format!(", extra{i}: value.extra{i}"))
                .collect::<String>();
            let mut source = fixture::child_return_source(width).replace("fn step(state:", &format!(
                "@noinline fn inspect(value: State) -> State {{ return State {{ left: value.left, right: value.right, count: value.count + 1{extra} }}; }} fn step(state:"
            )).replace(
                "if k == 1 { break; }",
                "let snapshot = inspect(carry); let checked_snapshot = 10 / (snapshot.count - carry.count); if k == 1 { continue; }",
            );
            if observed {
                source = source.replace("tag: i32_from_i64(9)", "tag: i32_from_i64(9 + k)");
            }
            check_source(&source, width, true, observed);
        }
    }
}

#[test]
fn typed_nested_opaque_child_snapshots_preserve_traps_and_entry_reservations() {
    for width in [9, 64] {
        let source = fixture::child_return_source(width).replace("fn step(state:",
            "@noinline fn inspect(value: State) -> State { let checked = 10 / (value.count - value.count); return value; } fn step(state:")
            .replace("if k == 1 { break; }", "let snapshot = inspect(carry); let observed = snapshot.right.tag; if k == 1 { break; }");
        let cases = [
            probe::Case {
                limit: 0,
                loops: 0,
                entries: 1,
                status: Some(0),
                remaining: [0, 0],
                corrupt: None,
                alias: false,
                semantic_trap: false,
            },
            // The opaque call consumes one additional helper entry, at the
            // selected trip rather than while assembling invariant inputs.
            probe::Case {
                limit: 2,
                loops: 64,
                entries: 64,
                status: None,
                remaining: [59, 52],
                corrupt: None,
                alias: false,
                semantic_trap: true,
            },
        ];
        probe::check(&source, width, &with_aliases(&cases));
        let source = source.replace("10 / (value.count - value.count)", "10 / value.count");
        let case = probe::Case {
            limit: 2,
            loops: 64,
            entries: 11,
            status: None,
            remaining: [59, 0],
            corrupt: None,
            alias: false,
            semantic_trap: false,
        };
        probe::check(&source, width, &with_aliases(&[case]));
    }
}

#[test]
fn typed_nested_opaque_continue_snapshot_traps_keep_iteration_timing() {
    for observed in [false, true] {
        for late in [false, true] {
            let call = "let snapshot = inspect(carry); let observed_snapshot = snapshot.right.tag;";
            let body = if late {
                format!("if k == 1 {{ continue; }} {call}")
            } else {
                format!("{call} if k == 1 {{ continue; }}")
            };
            let mut source = fixture::child_return_source(64).replace("fn step(state:",
                "@noinline fn inspect(value: State) -> State { let checked = 10 / (value.count - value.count); return value; } fn step(state:")
                .replace("if k == 1 { break; }", &body);
            if observed {
                source = source.replace("tag: i32_from_i64(9)", "tag: i32_from_i64(9 + k)");
            }
            let cases = [
                probe::Case {
                    limit: 0,
                    loops: 0,
                    entries: 1,
                    status: Some(0),
                    remaining: [0, 0],
                    corrupt: None,
                    alias: false,
                    semantic_trap: false,
                },
                probe::Case {
                    limit: 2,
                    loops: 64,
                    entries: 64,
                    status: None,
                    // The late path also enters the guarded continuation on trip two.
                    remaining: [59, if late { 47 } else { 52 }],
                    corrupt: None,
                    alias: false,
                    semantic_trap: true,
                },
            ];
            probe::check(&source, 64, &with_aliases(&cases));
            let source = source.replace("10 / (value.count - value.count)", "10 / value.count");
            let case = probe::Case {
                limit: 2,
                loops: 64,
                entries: if late { 16 } else { 11 },
                status: None,
                remaining: [59, 0],
                corrupt: None,
                alias: false,
                semantic_trap: false,
            };
            probe::check(&source, 64, &with_aliases(&[case]));
        }
    }
}

#[test]
fn typed_nested_checked_child_snapshots_preserve_evaluated_constructors() {
    for width in [9, 63, 64] {
        for continuing in [false, true] {
            for observed in [false, true] {
                let mut source = checked_snapshot_source(width, "carry.count + 1").replace(
                    "let observed = snapshot.right.tag;",
                    "let checked_snapshot = 10 / (snapshot.count - carry.count);",
                );
                if continuing {
                    source = source.replace("if k == 1 { break; }", "if k == 1 { continue; }");
                }
                if observed {
                    source = source.replace("tag: i32_from_i64(9)", "tag: i32_from_i64(9 + k)");
                }
                check_source(&source, width, continuing, observed);
            }
        }
    }
}

fn checked_snapshot_source(width: usize, count: &str) -> String {
    let extra = (9..width)
        .map(|i| format!(", extra{i}: carry.extra{i}"))
        .collect::<String>();
    fixture::child_return_source(width).replace(
        "if k == 1 { break; }",
        &format!("let snapshot = State {{ left: carry.left, right: carry.right, count: {count}{extra} }}; let observed = snapshot.right.tag; if k == 1 {{ break; }}"),
    )
}

#[test]
fn typed_nested_checked_child_snapshots_preserve_unused_traps_and_field_order() {
    for width in [9, 64] {
        for call_first in [false, true] {
            let checked = "10 / (carry.count - carry.count)";
            let mut source = checked_snapshot_source(width, checked).replace(
                "fn step(state:",
                "@noinline fn inspect_leaf(value: Leaf) -> Leaf { return value; } fn step(state:",
            );
            let original = format!("left: carry.left, right: carry.right, count: {checked}");
            let fields = if call_first {
                format!("left: inspect_leaf(carry.left), right: carry.right, count: {checked}")
            } else {
                format!("count: {checked}, right: carry.right, left: inspect_leaf(carry.left)")
            };
            source = source.replace(&original, &fields);
            let cases = [
                probe::Case {
                    limit: 0,
                    loops: 0,
                    entries: 1,
                    status: Some(0),
                    remaining: [0, 0],
                    corrupt: None,
                    alias: false,
                    semantic_trap: false,
                },
                probe::Case {
                    limit: 2,
                    loops: 64,
                    entries: 64,
                    status: None,
                    remaining: [59, if call_first { 52 } else { 53 }],
                    corrupt: None,
                    alias: false,
                    semantic_trap: true,
                },
            ];
            probe::check(&source, width, &with_aliases(&cases));
        }
    }
}

#[test]
fn typed_nested_checked_continue_snapshots_preserve_selected_trip() {
    for late in [false, true] {
        let mut source = checked_snapshot_source(64, "10 / (carry.count - carry.count)")
            .replace("tag: i32_from_i64(9)", "tag: i32_from_i64(9 + k)");
        if late {
            source = source
                .replace(
                    "let snapshot = State",
                    "if k == 1 { continue; } let snapshot = State",
                )
                .replace("if k == 1 { break; }", "");
        } else {
            source = source.replace("if k == 1 { break; }", "if k == 1 { continue; }");
        }
        let cases = [
            probe::Case {
                limit: 0,
                loops: 0,
                entries: 1,
                status: Some(0),
                remaining: [0, 0],
                corrupt: None,
                alias: false,
                semantic_trap: false,
            },
            probe::Case {
                limit: 2,
                loops: 64,
                entries: 64,
                status: None,
                remaining: [59, if late { 48 } else { 53 }],
                corrupt: None,
                alias: false,
                semantic_trap: true,
            },
        ];
        probe::check(&source, 64, &with_aliases(&cases));
    }
}

#[test]
fn typed_nested_child_exit_traps_preserve_atomic_callback_publication() {
    for width in [9, 64] {
        let source = fixture::child_return_source(width).replace(
            "if k == 1 { break; }",
            "let selected_trap = 1 / (k - k); if k == 1 { break; }",
        );
        let case = probe::Case {
            limit: 2,
            loops: 64,
            entries: 64,
            status: None,
            // Reserve two outer trips, one return-owning inner trip, and
            // two ordinary child trips before reaching the selected trap.
            remaining: [59, 53],
            corrupt: None,
            alias: false,
            semantic_trap: true,
        };
        probe::check(&source, width, &with_aliases(&[case]));
    }
}

#[test]
fn typed_nested_child_exit_snapshot_traps_preserve_atomic_callback_publication() {
    for width in [9, 64] {
        let source = fixture::child_return_source(width).replace(
            "if k == 1 { break; }",
            "let snapshot = carry; let checked_snapshot = 10 / (snapshot.count - snapshot.count); if k == 1 { break; }",
        );
        let cases = [
            probe::Case {
                limit: 0,
                loops: 0,
                entries: 1,
                status: Some(0),
                remaining: [0, 0],
                corrupt: None,
                alias: false,
                semantic_trap: false,
            },
            probe::Case {
                limit: 2,
                loops: 64,
                entries: 64,
                status: None,
                remaining: [59, 53],
                corrupt: None,
                alias: false,
                semantic_trap: true,
            },
        ];
        probe::check(&source, width, &with_aliases(&cases));
    }
}

fn check(width: usize, continuing: bool, observed: bool, snapshot: bool) {
    let mut source = fixture::child_return_source(width);
    if snapshot {
        source = source.replace(
            "if k == 1 { break; }",
            "let snapshot = carry; let checked_snapshot = 10 / snapshot.count; if k == 1 { break; }",
        );
    }
    if continuing {
        source = source.replace("if k == 1 { break; }", "if k == 1 { continue; }");
    }
    if observed {
        source = source.replace("tag: i32_from_i64(9)", "tag: i32_from_i64(9 + k)");
    }
    check_source(&source, width, continuing, observed);
}

fn check_source(source: &str, width: usize, continuing: bool, observed: bool) {
    let project = Project::with_source(source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap_or_else(|error| {
        panic!("width={width}, continuing={continuing}, observed={observed}: {error}")
    });
    let widths = carried_widths(&compiled.yir);
    let expected = if continuing {
        vec![width - 4, width - 1]
    } else {
        vec![if observed { 2 } else { 1 }, width - 4, width - 1]
    };
    assert_eq!(widths, expected);
    let cases = (0..=4)
        .map(|limit| {
            let initial = initial(width, limit);
            let mut event = event(&initial, limit);
            if observed && limit > 0 {
                let index = if continuing {
                    limit % 3
                } else {
                    (limit % 3).min(1)
                };
                event[6] += index;
            }
            (
                vec![initial[0], initial[1], initial[2], limit],
                vec![initial, event.clone(), event],
            )
        })
        .collect();
    if width == 9 {
        typed_record_inputs::check_flattened(source, cases);
    } else {
        typed_record_inputs::check_compact(source, cases, width - 7);
    }
}
