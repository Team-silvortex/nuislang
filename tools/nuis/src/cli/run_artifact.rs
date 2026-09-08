use super::{CommandKind, WindowSessionOptions};
use std::path::PathBuf;

pub(super) fn parse(args: &mut impl Iterator<Item = String>) -> Result<CommandKind, String> {
    let mut input = None;
    let mut json = false;
    let mut frame_output = None;
    let mut window_session = None;
    let mut window_events = None;
    let mut window_parent = None;
    let mut cancel_after_events = false;
    let mut drain_provider = false;
    let mut application_arguments = Vec::new();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--application-session" | "--open-args" | "--event-args" | "--close-args" => {
                let value = args
                    .next()
                    .ok_or_else(|| format!("{arg} requires a value"))?;
                application_arguments.extend([arg, value]);
            }
            "--cancel-after-events" => application_arguments.push(arg),
            "--window-cancel-after-events" if !cancel_after_events => cancel_after_events = true,
            "--drain-provider" if !drain_provider => drain_provider = true,
            "--json" if !json => json = true,
            "--export-frame" if frame_output.is_none() => {
                let path = args
                    .next()
                    .filter(|path| !path.is_empty() && !path.starts_with("--"))
                    .ok_or("--export-frame requires a new output PPM path")?;
                frame_output = Some(PathBuf::from(path));
            }
            "--window-session" if window_session.is_none() => {
                window_session = Some(
                    args.next()
                        .filter(|id| !id.is_empty() && !id.starts_with('-'))
                        .ok_or("--window-session requires a registered session ID")?,
                );
            }
            "--window-parent-session" if window_parent.is_none() => {
                window_parent = Some(
                    args.next()
                        .filter(|id| !id.is_empty() && !id.starts_with('-'))
                        .ok_or("--window-parent-session requires a registered parent ID")?,
                );
            }
            "--window-events" if window_events.is_none() => {
                let events = args
                    .next()
                    .ok_or("--window-events requires Unicode codepoints")?;
                if !events.is_empty() {
                    let codes = events.split(',').collect::<Vec<_>>();
                    if codes.len() > 64
                        || codes.iter().any(|value| {
                            value.is_empty()
                                || !value.bytes().all(|byte| byte.is_ascii_digit())
                                || value.parse::<u32>().ok().and_then(char::from_u32).is_none()
                        })
                    {
                        return Err(
                            "--window-events requires at most 64 comma-separated Unicode scalars"
                                .to_owned(),
                        );
                    }
                }
                window_events = Some(events);
            }
            value if value.starts_with('-') => {
                return Err(format!(
                    "unknown or duplicate run-artifact option `{value}`"
                ))
            }
            _ if input.is_none() && !arg.is_empty() => input = Some(PathBuf::from(arg)),
            _ => return Err("run-artifact accepts exactly one artifact input".to_owned()),
        }
    }
    if json && frame_output.is_some() {
        return Err(
            "--json is inspection-only and cannot be combined with --export-frame".to_owned(),
        );
    }
    let application_session = if application_arguments.is_empty() {
        None
    } else {
        if window_session.is_some()
            || window_events.is_some()
            || window_parent.is_some()
            || cancel_after_events
            || json
            || frame_output.is_some()
        {
            return Err(
                "application scripts cannot be combined with window, JSON or frame-export options"
                    .to_owned(),
            );
        }
        if drain_provider {
            application_arguments.push("--drain-provider".to_owned());
        }
        Some(yir_runtime_host::ApplicationScript::from_arguments(
            &application_arguments
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        )?)
    };
    if window_events.is_some() && window_session.is_none() {
        return Err("--window-events requires --window-session".to_owned());
    }
    if window_parent.is_some() && (window_session.is_none() || window_parent == window_session) {
        return Err("--window-parent-session requires a distinct --window-session".to_owned());
    }
    if cancel_after_events && (window_events.is_none() || window_parent.is_some()) {
        return Err(
            "--window-cancel-after-events requires --window-events and no parent session"
                .to_owned(),
        );
    }
    if drain_provider && !cancel_after_events && application_session.is_none() {
        return Err("--drain-provider requires explicit --window-cancel-after-events".to_owned());
    }
    if window_session.is_some() && (json || frame_output.is_some()) {
        return Err("--window-session cannot be combined with --json or --export-frame".to_owned());
    }
    let input = input.ok_or(
        "usage: nuis run-artifact <artifact> [--json | --export-frame PATH | --window-session ID [--window-events CODEPOINTS] [--window-parent-session ID] [--window-cancel-after-events [--drain-provider]] | --application-session ID --open-args I64,... [--event-args I64,...] (--close-args I64,... | --cancel-after-events --drain-provider)]",
    )?;
    Ok(CommandKind::RunArtifact {
        input,
        json,
        frame_output,
        application_session,
        window_session: window_session.map(|id| WindowSessionOptions {
            id,
            events: window_events,
            parent: window_parent,
            cancel_after_events,
            drain_provider,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn application_scripts_use_the_same_scalar_grammar_as_the_packaged_entry() {
        let args = [
            "build",
            "--event-args",
            "-1,2",
            "--application-session",
            "counter",
            "--open-args",
            "",
            "--cancel-after-events",
            "--drain-provider",
        ];
        let CommandKind::RunArtifact {
            application_session: Some(script),
            window_session: None,
            ..
        } = parse(&mut args.into_iter().map(str::to_owned)).unwrap()
        else {
            panic!("expected a non-window application script")
        };
        assert_eq!(script.events, [vec![-1, 2]]);
        assert_eq!(
            script.termination,
            yir_runtime_host::ApplicationScriptTermination::Cancel {
                drain_provider: true
            }
        );
        let close = [
            "build",
            "--application-session",
            "counter",
            "--open-args",
            "",
            "--close-args",
            "",
        ];
        assert!(parse(&mut close.into_iter().map(str::to_owned)).is_ok());
        for extra in [
            vec!["--window-session", "ui"],
            vec!["--window-events", ""],
            vec!["--window-parent-session", "parent"],
            vec!["--window-cancel-after-events"],
            vec!["--json"],
            vec!["--export-frame", "a.ppm"],
            vec!["--cancel-after-events"],
            vec!["--drain-provider"],
            vec!["--open-args", "1"],
            vec!["--event-args", "1,"],
            vec!["--application-session", "other"],
        ] {
            let arguments = [close.as_slice(), &extra].concat();
            assert!(parse(&mut arguments.into_iter().map(str::to_owned)).is_err());
        }
        for args in [
            vec!["build", "--event-args", "1"],
            vec!["build", "--application-session", "counter"],
            vec!["build", "--open-args", ""],
        ] {
            assert!(parse(&mut args.into_iter().map(str::to_owned)).is_err());
        }
    }

    #[test]
    fn provider_drain_is_opt_in_and_requires_explicit_cancellation() {
        let base = ["build", "--window-session", "ui", "--window-events", ""];
        for args in [
            [
                &base[..],
                &["--window-cancel-after-events", "--drain-provider"],
            ]
            .concat(),
            [
                &["--drain-provider"][..],
                &base[..],
                &["--window-cancel-after-events"],
            ]
            .concat(),
        ] {
            let CommandKind::RunArtifact {
                window_session: Some(options),
                ..
            } = parse(&mut args.into_iter().map(str::to_owned)).unwrap()
            else {
                panic!("window options");
            };
            assert!(options.drain_provider && options.cancel_after_events);
        }
        for args in [
            vec!["build", "--drain-provider"],
            [&base[..], &["--drain-provider"]].concat(),
            [
                &base[..],
                &[
                    "--window-cancel-after-events",
                    "--drain-provider",
                    "--drain-provider",
                ],
            ]
            .concat(),
            [
                &base[..],
                &[
                    "--window-cancel-after-events",
                    "--drain-provider",
                    "--window-parent-session",
                    "parent",
                ],
            ]
            .concat(),
            [
                &base[..],
                &["--window-cancel-after-events", "--drain-provider", "--json"],
            ]
            .concat(),
            [
                &base[..],
                &[
                    "--window-cancel-after-events",
                    "--drain-provider",
                    "--export-frame",
                    "frame.ppm",
                ],
            ]
            .concat(),
        ] {
            assert!(parse(&mut args.into_iter().map(str::to_owned)).is_err());
        }
    }

    #[test]
    fn cancellation_requires_an_explicit_script_without_parent_authority() {
        for events in ["", "32,128578"] {
            let parsed = parse(
                &mut [
                    "build",
                    "--window-session",
                    "ui",
                    "--window-cancel-after-events",
                    "--window-events",
                    events,
                ]
                .into_iter()
                .map(str::to_owned),
            )
            .unwrap();
            let CommandKind::RunArtifact {
                window_session: Some(options),
                ..
            } = parsed
            else {
                panic!("window options")
            };
            assert!(options.cancel_after_events);
            assert_eq!(options.events.as_deref(), Some(events));
        }
        for args in [
            vec!["build", "--window-cancel-after-events"],
            vec![
                "build",
                "--window-session",
                "ui",
                "--window-cancel-after-events",
            ],
            vec![
                "build",
                "--window-events",
                "",
                "--window-cancel-after-events",
            ],
            vec![
                "build",
                "--window-session",
                "ui",
                "--window-events",
                "",
                "--window-cancel-after-events",
                "--window-cancel-after-events",
            ],
            vec![
                "build",
                "--window-session",
                "ui",
                "--window-events",
                "",
                "--window-cancel-after-events",
                "--window-parent-session",
                "parent",
            ],
            vec![
                "build",
                "--window-session",
                "ui",
                "--window-events",
                "",
                "--window-cancel-after-events",
                "--json",
            ],
        ] {
            assert!(
                parse(&mut args.iter().map(|arg| (*arg).to_owned())).is_err(),
                "{args:?}"
            );
        }
    }

    #[test]
    fn parent_requires_a_distinct_registered_window_and_no_duplicate_option() {
        let parsed = parse(
            &mut [
                "build",
                "--window-session",
                "ui",
                "--window-parent-session",
                "parent",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap();
        let CommandKind::RunArtifact {
            window_session: Some(options),
            ..
        } = parsed
        else {
            panic!("window options");
        };
        assert_eq!(options.parent.as_deref(), Some("parent"));
        for args in [
            vec!["build", "--window-parent-session", "parent"],
            vec![
                "build",
                "--window-session",
                "ui",
                "--window-parent-session",
                "ui",
            ],
            vec![
                "build",
                "--window-session",
                "ui",
                "--window-parent-session",
                "",
            ],
            vec!["build", "--window-session", "ui", "--window-parent-session"],
            vec![
                "build",
                "--window-session",
                "ui",
                "--window-parent-session",
                "p",
                "--window-parent-session",
                "p",
            ],
        ] {
            assert!(parse(&mut args.into_iter().map(str::to_owned)).is_err());
        }
    }

    #[test]
    fn accepts_registered_window_and_bounded_unicode_event_script() {
        let parsed = parse(
            &mut [
                "build",
                "--window-session",
                "ui",
                "--window-events",
                "32,128578",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap();
        assert_eq!(
            parsed,
            CommandKind::RunArtifact {
                input: "build".into(),
                json: false,
                frame_output: None,
                application_session: None,
                window_session: Some(WindowSessionOptions {
                    id: "ui".to_owned(),
                    events: Some("32,128578".to_owned()),
                    parent: None,
                    cancel_after_events: false,
                    drain_provider: false,
                })
            }
        );
    }

    #[test]
    fn rejects_incompatible_window_modes_and_invalid_scripts() {
        for args in [
            vec!["build", "--window-session"],
            vec!["build", "--window-events", "32"],
            vec!["build", "--window-session", "ui", "--json"],
            vec![
                "build",
                "--window-session",
                "ui",
                "--export-frame",
                "out.ppm",
            ],
            vec![
                "build",
                "--window-session",
                "ui",
                "--window-events",
                "55296",
            ],
            vec![
                "build",
                "--window-session",
                "ui",
                "--window-events",
                "1114112",
            ],
            vec!["build", "--window-session", "ui", "--window-events", "32,"],
            vec!["build", "--window-session", "ui", "--window-events", "+32"],
        ] {
            assert!(parse(&mut args.into_iter().map(str::to_owned)).is_err());
        }
    }

    #[test]
    fn accepts_export_before_or_after_artifact() {
        for args in [
            ["--export-frame", "frames/a b.ppm", "build"],
            ["build", "--export-frame", "frames/a b.ppm"],
        ] {
            assert_eq!(
                parse(&mut args.into_iter().map(str::to_owned)).unwrap(),
                CommandKind::RunArtifact {
                    input: PathBuf::from("build"),
                    json: false,
                    frame_output: Some(PathBuf::from("frames/a b.ppm")),
                    window_session: None,
                    application_session: None,
                }
            );
        }
    }

    #[test]
    fn rejects_ambiguous_or_incomplete_exports() {
        for args in [
            vec!["--export-frame"],
            vec!["--export-frame", ""],
            vec!["--export-frame", "--json", "build"],
            vec!["--export-frame", "a.ppm"],
            vec!["build", "--export-frame", "a.ppm", "--json"],
            vec![
                "build",
                "--export-frame",
                "a.ppm",
                "--export-frame",
                "b.ppm",
            ],
            vec!["build", "--unknown"],
            vec!["build", "another"],
        ] {
            assert!(parse(&mut args.into_iter().map(str::to_owned)).is_err());
        }
    }
}
