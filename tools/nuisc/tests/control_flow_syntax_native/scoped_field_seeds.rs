use super::*;

const SOURCE: &str = include_str!("scoped_field_seeds.ns");

#[path = "../native_application_bridge/scoped_record_fixture.rs"]
mod record_fixture;

#[path = "scoped_index_recovery_cases.rs"]
mod index_recovery;

#[path = "scoped_return_signal_cases.rs"]
mod return_signal;

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
