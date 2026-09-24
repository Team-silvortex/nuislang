use super::*;

#[test]
fn native_alias_snapshots_survive_origin_and_alias_rebindings() {
    let status = compile_and_run("alias_snapshots", include_str!("alias_snapshots.ns"));
    assert_eq!(status.code(), Some(187));
}

#[test]
fn native_snapshot_projection_retains_discarded_checked_rebindings() {
    let (prefix, _) = include_str!("alias_snapshots.ns")
        .split_once("    fn main()")
        .unwrap();
    let source = format!(
        "{prefix}
        fn main() -> i64 {{
            let state = State {{ left: 12, right: 7, unused: 0 }};
            return checked_snapshot(state, 0, false) + checked_snapshot(state, 2, true);
        }}
    }}"
    );
    let status = compile_and_run("discarded_snapshot_valid", &source);
    assert_eq!(status.code(), Some(24));
    let source = source.replace(
        "checked_snapshot(state, 2, true)",
        "checked_snapshot(state, 0, true)",
    );
    let status = compile_and_run("discarded_snapshot_trap", &source);
    assert!(!status.success(), "{status:?}");
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert!(matches!(status.signal(), Some(4 | 5)), "{status:?}");
    }
}
