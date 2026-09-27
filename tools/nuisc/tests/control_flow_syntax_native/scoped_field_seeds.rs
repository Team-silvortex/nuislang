use super::*;

const SOURCE: &str = include_str!("scoped_field_seeds.ns");

#[path = "../native_application_bridge/scoped_record_fixture.rs"]
mod record_fixture;

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
