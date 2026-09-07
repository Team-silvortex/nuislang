use super::{CommandKind, WindowSessionOptions};
use std::path::PathBuf;

pub(super) fn parse(args: &mut impl Iterator<Item = String>) -> Result<CommandKind, String> {
    let mut input = None;
    let mut json = false;
    let mut frame_output = None;
    let mut window_session = None;
    let mut window_events = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
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
    if window_events.is_some() && window_session.is_none() {
        return Err("--window-events requires --window-session".to_owned());
    }
    if window_session.is_some() && (json || frame_output.is_some()) {
        return Err("--window-session cannot be combined with --json or --export-frame".to_owned());
    }
    let input = input.ok_or(
        "usage: nuis run-artifact [--json | --export-frame PATH | --window-session ID [--window-events CODEPOINTS]] <artifact>",
    )?;
    Ok(CommandKind::RunArtifact {
        input,
        json,
        frame_output,
        window_session: window_session.map(|id| WindowSessionOptions {
            id,
            events: window_events,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
                window_session: Some(WindowSessionOptions {
                    id: "ui".to_owned(),
                    events: Some("32,128578".to_owned())
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
