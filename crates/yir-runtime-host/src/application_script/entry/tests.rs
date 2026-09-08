use super::*;

#[test]
fn scalar_script_requires_explicit_and_exclusive_termination() {
    let base = ["--application-session", "counter", "--open-args", "10"];
    assert!(parse(&base).is_err());
    let mut args = base.to_vec();
    args.extend(["--event-args", "-2,9223372036854775807", "--close-args", ""]);
    let script = parse(&args).unwrap();
    assert_eq!(script.open, [10]);
    assert_eq!(script.events, [vec![-2, i64::MAX]]);
    assert_eq!(
        script.termination,
        ApplicationScriptTermination::Close(vec![])
    );
    args.push("--cancel-after-events");
    assert!(parse(&args)
        .unwrap_err()
        .contains("cannot also request close"));
    for terminal in [
        vec!["--cancel-after-events"],
        vec!["--cancel-after-events", "--drain-provider"],
    ] {
        let mut args = base.to_vec();
        args.extend(&terminal);
        assert_eq!(
            parse(&args).unwrap().termination,
            ApplicationScriptTermination::Cancel {
                drain_provider: terminal.len() == 2
            }
        );
    }
}

#[test]
fn scalar_script_rejects_malformed_duplicate_and_window_options() {
    let base = [
        "--application-session",
        "counter",
        "--open-args",
        "",
        "--cancel-after-events",
    ];
    for extra in [
        vec!["--drain-provider", "--drain-provider"],
        vec!["--cancel-after-events"],
        vec!["--application-session", "other"],
        vec!["--open-args", "1"],
        vec!["--event-args"],
        vec!["--window-session", "window"],
        vec!["--event-args", "1,"],
        vec!["--event-args", "1.0"],
        vec!["--event-args", "9223372036854775808"],
        vec!["--event-args", "1, 2"],
    ] {
        let mut args = base.to_vec();
        args.extend(extra);
        assert!(parse(&args).is_err(), "{args:?}");
    }
    assert!(parse(&[
        "--application-session",
        "counter",
        "--open-args",
        "",
        "--close-args",
        "",
        "--drain-provider"
    ])
    .is_err());
}

#[test]
fn scalar_script_limits_are_checked_before_any_session_is_started() {
    let mut args = vec![
        "--application-session",
        "counter",
        "--open-args",
        "",
        "--close-args",
        "",
    ];
    for _ in 0..MAX_SCRIPT_EVENTS {
        args.extend(["--event-args", ""]);
    }
    assert!(parse(&args).is_ok());
    args.extend(["--event-args", ""]);
    assert!(parse(&args).is_err());
    let values = vec!["1"; MAX_SCRIPT_ARGUMENTS].join(",");
    assert!(parse_values(&values).is_ok());
    assert!(parse_values(&(values + ",1")).is_err());
    assert!(parse(&[&"x".repeat(MAX_ARGUMENT_BYTES + 1)]).is_err());
}

#[test]
fn scalar_script_never_selects_an_implicit_or_ambiguous_provider() {
    let path = std::ffi::OsString::from("provider-path");
    assert!(provider_paths(None, None).is_err());
    assert!(provider_paths(Some(path.clone()), Some(path.clone())).is_err());
    assert!(provider_paths(Some("".into()), None).is_err());
    assert!(provider_paths(None, Some("".into())).is_err());
    assert_eq!(
        provider_paths(Some(path.clone()), None).unwrap(),
        (Some(PathBuf::from(&path)), None)
    );
    assert_eq!(
        provider_paths(None, Some(path.clone())).unwrap(),
        (None, Some(PathBuf::from(path)))
    );
}

#[test]
fn scalar_script_ffi_rejects_invalid_utf8_nulls_and_counts() {
    let source = b"yir 0.1\n";
    let argv = [c"headless".as_ptr(), std::ptr::null()];
    unsafe {
        assert_eq!(
            nuis_application_script_main(source.as_ptr(), source.len(), 2, argv.as_ptr()),
            1
        );
        assert_eq!(
            nuis_application_script_main(std::ptr::null(), 0, 1, argv.as_ptr()),
            1
        );
        assert_eq!(
            nuis_application_script_main(source.as_ptr(), source.len(), 0, argv.as_ptr()),
            1
        );
        assert_eq!(
            nuis_application_script_main(source.as_ptr(), usize::MAX, 1, argv.as_ptr()),
            1
        );
        assert_eq!(
            nuis_application_script_main(b"\xff".as_ptr(), 1, 1, argv.as_ptr()),
            1
        );
    }
}
