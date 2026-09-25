use super::*;

const SOURCE: &str = include_str!("scoped_field_seeds.ns");

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
