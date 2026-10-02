use super::*;

const SOURCE: &str = include_str!("scoped_field_seeds.ns");

#[path = "scoped_return_join_cases.rs"]
mod join_cases;

#[test]
fn scoped_return_joins_execute_correlated_arms_old_versions_and_selected_checks() {
    for case in join_cases::CASES {
        for choose in [false, true] {
            for limit in [0, 1, 3] {
                let source = join_cases::source(case, choose, limit);
                let status = compile_and_run(&format!("joined_{case}_{choose}_{limit}"), &source);
                assert_eq!(
                    status.code(),
                    join_cases::expected(case, choose, limit).map(|v| v as i32),
                    "{source}"
                );
                if join_cases::expected(case, choose, limit).is_none() {
                    use std::os::unix::process::ExitStatusExt;
                    assert!(
                        matches!(status.signal(), Some(4 | 5)),
                        "selected preheader check must still trap: {status:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn scoped_return_preheaders_preserve_old_versions_and_later_parent_trips() {
    for limit in [0, 1, 3] {
        let delayed = format!("mod cpu Main {{
            struct State {{ value: i64, tag: i64 }}
            fn step(seed: State, other: State, limit: i64) -> State {{
                let carry = seed; let delayed = seed; let i = 0;
                while i < limit {{
                    let i = i + 1; let carry = carry; let delayed = delayed;
                    let carry = delayed; let delayed = other;
                    if i == 2 {{ return carry; }}
                }}
                return carry;
            }}
            fn main() -> i64 {{
                let result = step(State {{ value: 37, tag: 5 }}, State {{ value: 99, tag: 7 }}, {limit});
                return result.value + result.tag;
            }}
        }}");
        assert_eq!(
            compile_and_run(&format!("delayed_snapshot_{limit}"), &delayed).code(),
            Some(if limit < 2 { 42 } else { 106 })
        );
        let parent = format!(
            "mod cpu Main {{
            struct State {{ value: i64, tag: i64 }}
            fn step(seed: State, limit: i64) -> State {{
                let carry = seed; let tag = carry.tag; let i = 0;
                while i < limit {{
                    let i = i + 1; let carry = carry; let j = 0;
                    while j < 2 {{
                        let j = j + 1; let carry = carry;
                        let carry = State {{ value: carry.value + 1, tag: tag }};
                    }}
                    let carry = State {{ value: carry.value, tag: carry.tag + i }};
                    if i == 2 {{ return carry; }}
                }}
                return carry;
            }}
            fn main() -> i64 {{
                let result = step(State {{ value: 10, tag: 7 }}, {limit});
                return result.value + result.tag;
            }}
        }}"
        );
        assert_eq!(
            compile_and_run(&format!("parent_snapshot_{limit}"), &parent).code(),
            Some(match limit {
                0 => 17,
                1 => 20,
                _ => 23,
            })
        );
        let opaque = format!(
            "mod cpu Main {{
            struct State {{ value: i64, tag: i64 }}
            @noinline fn relay(value: State) -> State {{
                return State {{ value: value.value, tag: value.tag + 1 }};
            }}
            fn step(seed: State, limit: i64) -> State {{
                let carry = relay(seed); let tag = carry.tag; let carry = relay(carry);
                let i = 0;
                while i < limit {{
                    let i = i + 1; let carry = carry;
                    let carry = State {{ value: carry.value + 1, tag: tag }};
                    if i == 2 {{ return carry; }}
                }}
                return carry;
            }}
            fn main() -> i64 {{
                let result = step(State {{ value: 10, tag: 7 }}, {limit});
                return result.value + result.tag;
            }}
        }}"
        );
        assert_eq!(
            compile_and_run(&format!("opaque_snapshot_{limit}"), &opaque).code(),
            Some(match limit {
                0 => 19,
                1 => 19,
                _ => 20,
            })
        );
    }
}

#[path = "../native_application_bridge/scoped_record_fixture.rs"]
mod record_fixture;

#[path = "scoped_index_recovery_cases.rs"]
mod index_recovery;

#[path = "scoped_return_signal_cases.rs"]
mod return_signal;

#[test]
fn scoped_readonly_record_single_carry_runs_with_both_induction_orders() {
    let fields = (0..63)
        .map(|i| format!("f{i}: i64"))
        .collect::<Vec<_>>()
        .join(", ");
    let values = (0..63)
        .map(|i| format!("f{i}: 7"))
        .collect::<Vec<_>>()
        .join(", ");
    for leading in [false, true] {
        for limit in [0, 3] {
            let step = "let i = i + 1;";
            let source = format!(
                "mod cpu Main {{
                struct Wide {{ {fields} }}
                @noinline fn inspect(input: Wide) -> Wide {{ return input; }}
                @noinline fn walk(input: Wide, limit: i64) -> i64 {{
                    let i = 0; let total = 1;
                    while i < limit {{ {} let snapshot = inspect(input);
                        let total = total + snapshot.f0 + i; {} }}
                    return total + i * 10;
                }}
                fn main() -> i64 {{ return walk(Wide {{ {values} }}, {limit}); }}
            }}",
                if leading { step } else { "" },
                if leading { "" } else { step }
            );
            let compiled = nuisc::pipeline::compile_source(&source).unwrap();
            assert!(compiled.yir.nodes.iter().any(|node| {
                node.op
                    .args
                    .get(6)
                    .is_some_and(|action| action == "scoped_call_i64_carry")
                    && node
                        .op
                        .args
                        .iter()
                        .any(|arg| arg.starts_with("$value_record:"))
            }));
            assert_eq!(
                compile_and_run(&format!("readonly_single_{leading}_{limit}"), &source).code(),
                Some(if limit == 0 {
                    1
                } else if leading {
                    58
                } else {
                    55
                })
            );
        }
    }
}

#[test]
fn scoped_return_signal_preserves_native_break_continue_and_parent_propagation() {
    for (case, _, _) in return_signal::CASES {
        for leading in [false, true] {
            for (limit, stop) in [(0, 0), (4, 1), (4, 3)] {
                let source = return_signal::source(case, leading, limit, stop);
                assert_eq!(
                    compile_and_run(
                        &format!("return_signal_{case}_{leading}_{limit}_{stop}"),
                        &source
                    )
                    .code(),
                    Some(return_signal::expected(case, leading, limit, stop) as i32),
                    "{case} leading={leading} limit={limit} stop={stop}",
                );
            }
        }
    }
}

#[test]
fn scoped_index_recovery_preserves_native_exits_and_generated_backedges() {
    for case in index_recovery::cases() {
        assert!(case.recoveries <= 1);
        for limit in [0, 3] {
            let source = index_recovery::source(&case.body, limit);
            assert_eq!(
                compile_and_run(&format!("index_recovery_{}_{limit}", case.name), &source).code(),
                Some(case.expected[limit] as i32),
                "{} limit={limit}",
                case.name,
            );
        }
    }
}

#[test]
fn scoped_index_recovery_keeps_selected_work_and_skips_unreachable_traps() {
    let case = index_recovery::cases().remove(0);
    let body = case.body.replace(
        "let total = total + index;",
        "let unused = 10 / (limit - index); let total = total + index;",
    );
    for limit in [0, 3] {
        assert_eq!(
            compile_and_run(
                &format!("index_recovery_lazy_{limit}"),
                &index_recovery::source(&body, limit)
            )
            .code(),
            Some(case.expected[limit] as i32),
        );
    }
    let status = compile_and_run(
        "index_recovery_selected_trap",
        &index_recovery::source(&body, 1),
    );
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
}

#[test]
fn materialized_typed_record_nested_returns_execute_and_keep_payload_traps_lazy() {
    let source = include_str!("scoped_return_typed_record_carries.ns");
    assert_eq!(
        compile_and_run("materialized_nested_return", source).code(),
        Some(19)
    );
    let trapped = source.replace("return State { left: selected,",
        "return State { left: Leaf { value: selected.value, gain: selected.gain, tag: i32_from_i64(10 / (2 - j)), enabled: selected.enabled },");
    let status = compile_and_run("materialized_nested_return_trap", &trapped);
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
    for limit in [0, 1] {
        let lazy = trapped.replace("i32_from_i64(7), 3", &format!("i32_from_i64(7), {limit}"));
        assert_eq!(
            compile_and_run(&format!("materialized_nested_return_lazy_{limit}"), &lazy).code(),
            Some(201)
        );
    }
}

#[test]
fn materialized_typed_record_loops_execute_and_keep_nested_traps_lazy() {
    let source = include_str!("scoped_loop_typed_record_carries.ns");
    assert_eq!(
        compile_and_run("materialized_record_loop", source).code(),
        Some(19)
    );
    let trapped = source.replace("tag: previous.tag", "tag: i32_from_i64(10 / (2 - j))");
    let status = compile_and_run("materialized_record_loop_trap", &trapped);
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
    for limit in [0, 1] {
        let lazy = trapped.replace("i32_from_i64(7), 3", &format!("i32_from_i64(7), {limit}"));
        assert_eq!(
            compile_and_run(&format!("materialized_loop_lazy_{limit}"), &lazy).code(),
            Some(201)
        );
    }
    for exit in ["break", "continue"] {
        let lazy = trapped.replace(
            "let previous = selected;",
            &format!("let previous = selected; if j == 2 {{ {exit}; }}"),
        );
        assert_eq!(
            compile_and_run(&format!("materialized_loop_lazy_{exit}"), &lazy).code(),
            Some(201)
        );
    }
}

#[test]
fn materialized_typed_record_joins_execute_and_keep_selected_traps() {
    let source = include_str!("scoped_join_typed_record_carries.ns");
    assert_eq!(
        compile_and_run("materialized_record_join", source).code(),
        Some(19)
    );
    let trapped = source.replacen(
        "tag: previous.tag",
        "tag: i32_from_i64(10 / (limit - i))",
        1,
    );
    let status = compile_and_run("materialized_record_join_trap", &trapped);
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
    for limit in [0, 2] {
        let lazy = trapped.replace("i32_from_i64(7), 3", &format!("i32_from_i64(7), {limit}"));
        assert_eq!(
            compile_and_run(&format!("materialized_record_join_lazy_{limit}"), &lazy).code(),
            Some(201)
        );
    }
}

#[test]
fn sparse_branch_typed_snapshots_execute_and_preserve_branch_local_traps() {
    let source = include_str!("scoped_branch_typed_record_carries.ns");
    assert_eq!(
        compile_and_run("sparse_branch_snapshots", source).code(),
        Some(19)
    );
    let trapped = source.replace(
        "let leaf = local.left;",
        "let leaf = local.left;\n\
        let checked = Leaf { value: 0.0, gain: 0.0,\n\
            tag: i32_from_i64(10 / (limit - i)), enabled: false };",
    );
    let status = compile_and_run("sparse_branch_snapshot_trap", &trapped);
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
    for limit in [0, 2] {
        let lazy = trapped.replace("i32_from_i64(7), 3", &format!("i32_from_i64(7), {limit}"));
        assert_eq!(
            compile_and_run(&format!("sparse_branch_lazy_{limit}"), &lazy).code(),
            Some(201)
        );
    }
}

#[test]
fn sparse_typed_record_carries_execute_and_keep_unused_checked_work() {
    let source = include_str!("scoped_sparse_typed_record_carries.ns");
    assert_eq!(
        compile_and_run("sparse_typed_record_words", source).code(),
        Some(19)
    );
    let trapped = source.replace("tag: i32_from_i64(9)", "tag: i32_from_i64(9 / (limit - i))");
    assert!(!compile_and_run("sparse_typed_unused_field_trap", &trapped).success());
    let zero = trapped
        .replace("i32_from_i64(7), 3", "i32_from_i64(7), 0")
        .replace(
            "i32_from_i64(9) { return 202; }",
            "i32_from_i64(7) { return 202; }",
        );
    assert_eq!(
        compile_and_run("sparse_typed_zero_trip", &zero).code(),
        Some(203)
    );
}

#[test]
fn nested_record_carries_preserve_native_paths_snapshots_breaks_and_lazy_traps() {
    let source = include_str!("scoped_nested_record_carries.ns");
    assert_eq!(
        compile_and_run("nested_record_words", source).code(),
        Some(37)
    );
    let trapped = source.replace("walk(5, 2)", "walk(5, 0)");
    let status = compile_and_run("nested_record_words_trap", &trapped);
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
}

#[test]
fn f64_record_carries_preserve_native_values_scalar_slots_and_lazy_traps() {
    let source = include_str!("scoped_f64_record_carries.ns");
    assert_eq!(compile_and_run("f64_record_words", source).code(), Some(37));
    let trapped = source.replace("walk(5, 2)", "walk(5, 0)");
    let status = compile_and_run("f64_record_words_trap", &trapped);
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
}

#[test]
fn f32_record_carries_preserve_native_values_scalar_slots_and_lazy_traps() {
    let source = include_str!("scoped_f32_record_carries.ns");
    assert_eq!(compile_and_run("f32_record_words", source).code(), Some(37));
    let trapped = source.replace("walk(5, 2)", "walk(5, 0)");
    let status = compile_and_run("f32_record_words_trap", &trapped);
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
}

#[test]
fn i32_record_carries_preserve_native_signed_wraps_comparisons_and_lazy_traps() {
    let source = include_str!("scoped_i32_record_carries.ns");
    assert_eq!(compile_and_run("i32_record_words", source).code(), Some(37));
    let trapped = source.replace("walk(5, 2)", "walk(5, 0)");
    let status = compile_and_run("i32_record_words_trap", &trapped);
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
}

#[test]
fn mixed_record_word_carries_preserve_ordinary_native_snapshots_and_lazy_traps() {
    let source = include_str!("scoped_mixed_record_carries.ns");
    assert_eq!(
        compile_and_run("mixed_record_words", source).code(),
        Some(37)
    );
    let trapped = source.replace("walk(5, 2)", "walk(5, 0)");
    let status = compile_and_run("mixed_record_words_trap", &trapped);
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
}

#[test]
fn generated_branch_record_inputs_preserve_ordinary_native_execution_and_traps() {
    let source = record_fixture::source(64, true)
        .replace("if i == 3 { break; }", "")
        .replace(
            "fn main() -> i64 { return 0; }",
            "fn main() -> i64 { let result = step(start(5)); return result.f0 + result.f63; }",
        );
    assert_eq!(
        compile_and_run("branch_record_inputs", &source).code(),
        Some(93)
    );
    assert_eq!(
        compile_and_run(
            "branch_record_zero_trip",
            &source.replace("start(5)", "start(-3)")
        )
        .code(),
        Some(57)
    );
    let lazy = source
        .replace("start(5)", "start(1)")
        .replace("if i <= 2", "if limit < 3")
        .replace("carry.f0 + 2", "10 / (carry.f0 - 2)");
    assert_eq!(
        compile_and_run("branch_record_lazy", &lazy).code(),
        Some(69)
    );
    let trapped = source
        .replace("start(5)", "start(2)")
        .replace("value.f0 + 1", "10 / (value.f0 - 2)");
    let status = compile_and_run("branch_record_trap", &trapped);
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
}

#[test]
fn scoped_nested_return_child_exits_execute_and_keep_selected_traps_lazy() {
    let source = include_str!("scoped_return_child_exits.ns");
    for exit in ["break", "continue"] {
        let source = source.replace("if k == 1 { break; }", &format!("if k == 1 {{ {exit}; }}"));
        for limit in [0, 1, 2, 3, 4] {
            let case = source.replace("i32_from_i64(7), 3)", &format!("i32_from_i64(7), {limit})"));
            assert_eq!(
                compile_and_run(&format!("return_child_{exit}_{limit}"), &case).code(),
                Some(if limit < 2 { 201 } else { 19 })
            );
        }
    }
    let source = source.replace(
        "if k == 1 { break; }",
        "let selected_trap = 1 / (k - k); if k == 1 { break; }",
    );
    // Three outer trips imply a zero-trip ordinary child; the return path must
    // still skip its own suffix. Two outer trips select the new child trap.
    assert_eq!(
        compile_and_run("return_child_zero_trap", &source).code(),
        Some(19)
    );
    let trapped = source.replace("i32_from_i64(7), 3)", "i32_from_i64(7), 2)");
    let status = compile_and_run("return_child_selected_trap", &trapped);
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
}

#[test]
fn scoped_field_seed_backedges_execute_in_the_ordinary_native_entry() {
    let status = compile_and_run("scoped_field_seeds", SOURCE);
    assert_eq!(status.code(), Some(171));
}

#[test]
fn scoped_field_seed_backedges_preserve_selected_arithmetic_failure() {
    let source = SOURCE.replace("walk(2, 1)", "walk(2, 0)");
    let status = compile_and_run("scoped_field_seed_trap", &source);
    assert!(!status.success(), "{status:?}");
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
}

#[test]
fn scoped_partial_field_seeds_preserve_zero_trips_and_selected_traps() {
    let source = include_str!("scoped_partial_field_seeds.ns");
    assert_eq!(
        compile_and_run("scoped_partial_field_seeds", source).code(),
        Some(96)
    );
    let status = compile_and_run(
        "scoped_partial_field_trap",
        &source.replace("walk(2, 1)", "walk(2, 0)"),
    );
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
    let source = source
        .replace("right: 33", "right: 33 / divisor")
        .replace("walk(2, 1) + walk(0, 0)", "walk(0, 0)");
    let status = compile_and_run("scoped_partial_initial_trap", &source);
    assert!(!status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
}

#[test]
fn generated_record_capture_projection_preserves_real_native_results_and_traps() {
    let source = include_str!("scoped_projected_record_carries.ns");
    assert_eq!(
        compile_and_run("scoped_projected_record_carries", source).code(),
        Some(154)
    );
    for (name, source) in [
        ("iteration", source.replace("walk(2, 1)", "walk(2, 0)")),
        (
            "initializer",
            source
                .replace("unused: 7", "unused: 7 / divisor")
                .replace("walk(2, 1) + walk(0, 0)", "walk(0, 0)"),
        ),
        (
            "second_trip",
            source.replace("unused: 9", "unused: 9 / (divisor - i + 1)"),
        ),
    ] {
        let status = compile_and_run(&format!("scoped_projected_record_{name}"), &source);
        assert!(!status.success(), "{status:?}");
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
        }
    }
}

#[test]
fn generated_unread_record_elision_preserves_native_state_and_traps() {
    let source = include_str!("scoped_unread_record_carries.ns");
    assert_eq!(
        compile_and_run("scoped_unread_record_carries", source).code(),
        Some(132)
    );
    for (name, source) in [
        ("iteration", source.replace("walk(2, 1)", "walk(2, 0)")),
        (
            "initializer",
            source
                .replace("unused: 7", "unused: 7 / divisor")
                .replace("walk(2, 1) + walk(0, 0)", "walk(0, 0)"),
        ),
        (
            "second_trip",
            source.replace("unused: 9", "unused: 9 / (divisor - i + 1)"),
        ),
    ] {
        let status = compile_and_run(&format!("scoped_unread_record_{name}"), &source);
        assert!(!status.success(), "{status:?}");
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
        }
    }
}
