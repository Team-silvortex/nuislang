use std::path::{Path, PathBuf};

use crate::model::NsdbPayloadExecutionEventFilter;
use crate::transcript::NsdbReplayControl;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Command {
    Status,
    Inspect {
        input: PathBuf,
        json: bool,
        event_filter: NsdbPayloadExecutionEventFilter,
    },
    Events {
        input: PathBuf,
        json: bool,
        event_filter: NsdbPayloadExecutionEventFilter,
    },
    ReplayPlan {
        input: PathBuf,
        json: bool,
        event_filter: NsdbPayloadExecutionEventFilter,
    },
    Replay {
        input: PathBuf,
        json: bool,
        event_filter: NsdbPayloadExecutionEventFilter,
        replay_control: NsdbReplayControl,
        cursor_input: Option<PathBuf>,
        cursor_output: Option<PathBuf>,
    },
    MaterializeProviderSamples {
        output_dir: PathBuf,
        provider_family: Option<String>,
        json: bool,
    },
    ExecuteProviderSamples {
        output_dir: PathBuf,
        provider_family: Option<String>,
        json: bool,
    },
}

pub(crate) fn parse_args<I>(mut args: I) -> Result<Command, String>
where
    I: Iterator<Item = String>,
{
    let Some(command) = args.next() else {
        return Ok(Command::Status);
    };
    match command.as_str() {
        "status" => Ok(Command::Status),
        "inspect" => {
            let (input, json, event_filter) = parse_input_json_event_filter(args.by_ref())?;
            Ok(Command::Inspect {
                input,
                json,
                event_filter,
            })
        }
        "events" => {
            let (input, json, event_filter) = parse_input_json_event_filter(args.by_ref())?;
            Ok(Command::Events {
                input,
                json,
                event_filter,
            })
        }
        "replay-plan" => {
            let (input, json, event_filter) = parse_input_json_event_filter(args.by_ref())?;
            Ok(Command::ReplayPlan {
                input,
                json,
                event_filter,
            })
        }
        "replay" => {
            let (input, json, event_filter, replay_control, cursor_input, cursor_output) =
                parse_replay_args(args.by_ref())?;
            Ok(Command::Replay {
                input,
                json,
                event_filter,
                replay_control,
                cursor_input,
                cursor_output,
            })
        }
        "materialize-provider-samples" => {
            let (output_dir, provider_family, json) =
                parse_provider_materialize_args(args.by_ref())?;
            Ok(Command::MaterializeProviderSamples {
                output_dir,
                provider_family,
                json,
            })
        }
        "execute-provider-samples" => {
            let (output_dir, provider_family, json) =
                parse_provider_materialize_args(args.by_ref())?;
            Ok(Command::ExecuteProviderSamples {
                output_dir,
                provider_family,
                json,
            })
        }
        "--help" | "-h" | "help" => Err(usage().to_owned()),
        other => Err(format!("unknown nsdb command `{other}`\n{}", usage())),
    }
}

pub(crate) fn resolve_manifest_input(input: &Path) -> Result<PathBuf, String> {
    if input.is_dir() {
        let candidate = input.join("nuis.build.manifest.toml");
        if candidate.exists() {
            return Ok(candidate);
        }
        return Err(format!(
            "directory `{}` does not contain `nuis.build.manifest.toml`",
            input.display()
        ));
    }
    Ok(input.to_path_buf())
}

fn usage() -> &'static str {
    "usage:\n  nsdb status\n  nsdb inspect <nuis.build.manifest.toml|artifact-output-dir> [--json] [--event-status <status>] [--event-phase <phase>] [--trace-id <trace-id>]\n  nsdb events <nuis.build.manifest.toml|artifact-output-dir> [--json] [--event-status <status>] [--event-phase <phase>] [--trace-id <trace-id>]\n  nsdb replay-plan <nuis.build.manifest.toml|artifact-output-dir> [--json] [--event-status <status>] [--event-phase <phase>] [--trace-id <trace-id>]\n  nsdb replay <nuis.build.manifest.toml|artifact-output-dir> [--json] [--event-status <status>] [--event-phase <phase>] [--trace-id <trace-id>] [--frame <index|frame-id> | --break-at <index|frame-id> | [--break-phase <phase>] [--break-entry <symbol>]] [--resume-after <frame-id> --resume-next <frame-id> | --resume-cursor <path>] [--save-cursor <path>]\n  nsdb materialize-provider-samples <artifact-output-dir> [--provider-family <family>] [--json]\n  nsdb execute-provider-samples <artifact-output-dir> [--provider-family <family>] [--json]"
}

fn parse_replay_args<I>(
    args: &mut I,
) -> Result<
    (
        PathBuf,
        bool,
        NsdbPayloadExecutionEventFilter,
        NsdbReplayControl,
        Option<PathBuf>,
        Option<PathBuf>,
    ),
    String,
>
where
    I: Iterator<Item = String>,
{
    let mut json = false;
    let mut input = None;
    let mut event_filter = NsdbPayloadExecutionEventFilter::default();
    let mut replay_control = NsdbReplayControl::default();
    let mut cursor_input = None;
    let mut cursor_output = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--event-status" => event_filter.status = Some(required_value(args, "--event-status")?),
            "--event-phase" => event_filter.phase = Some(required_value(args, "--event-phase")?),
            "--trace-id" => event_filter.trace_id = Some(required_value(args, "--trace-id")?),
            "--frame" => replay_control.frame_selector = Some(required_value(args, "--frame")?),
            "--break-at" => {
                replay_control.breakpoint_selector = Some(required_value(args, "--break-at")?)
            }
            "--break-phase" => {
                replay_control.breakpoint_phase = Some(required_value(args, "--break-phase")?)
            }
            "--break-entry" => {
                replay_control.breakpoint_entry = Some(required_value(args, "--break-entry")?)
            }
            "--resume-after" => {
                replay_control.resume_after_frame_id = Some(required_value(args, "--resume-after")?)
            }
            "--resume-next" => {
                replay_control.resume_next_frame_id = Some(required_value(args, "--resume-next")?)
            }
            "--resume-cursor" => {
                cursor_input = Some(PathBuf::from(required_value(args, "--resume-cursor")?))
            }
            "--save-cursor" => {
                cursor_output = Some(PathBuf::from(required_value(args, "--save-cursor")?))
            }
            _ if input.is_none() => input = Some(PathBuf::from(arg)),
            _ => return Err(format!("unexpected argument `{arg}`")),
        }
    }
    let has_predicate =
        replay_control.breakpoint_phase.is_some() || replay_control.breakpoint_entry.is_some();
    if replay_control.frame_selector.is_some()
        && (replay_control.breakpoint_selector.is_some() || has_predicate)
    {
        return Err("--frame is mutually exclusive with breakpoint controls".to_owned());
    }
    if replay_control.breakpoint_selector.is_some() && has_predicate {
        return Err("--break-at is mutually exclusive with breakpoint predicates".to_owned());
    }
    if replay_control.frame_selector.is_some()
        && (replay_control.resume_after_frame_id.is_some()
            || replay_control.resume_next_frame_id.is_some())
    {
        return Err("--frame is mutually exclusive with replay resume controls".to_owned());
    }
    if replay_control.resume_after_frame_id.is_some()
        != replay_control.resume_next_frame_id.is_some()
    {
        return Err("--resume-after and --resume-next must be provided together".to_owned());
    }
    if cursor_input.is_some()
        && (replay_control.resume_after_frame_id.is_some()
            || replay_control.resume_next_frame_id.is_some())
    {
        return Err("--resume-cursor is mutually exclusive with manual resume controls".to_owned());
    }
    if cursor_input.is_some() && replay_control.frame_selector.is_some() {
        return Err("--resume-cursor is mutually exclusive with --frame".to_owned());
    }
    let input = input.ok_or_else(|| usage().to_owned())?;
    Ok((
        input,
        json,
        event_filter,
        replay_control,
        cursor_input,
        cursor_output,
    ))
}

fn parse_provider_materialize_args<I>(
    args: &mut I,
) -> Result<(PathBuf, Option<String>, bool), String>
where
    I: Iterator<Item = String>,
{
    let mut json = false;
    let mut input = None;
    let mut provider_family = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--provider-family" => {
                provider_family = Some(required_value(args, "--provider-family")?)
            }
            _ if input.is_none() => input = Some(PathBuf::from(arg)),
            _ => return Err(format!("unexpected argument `{arg}`")),
        }
    }
    let input = input.ok_or_else(|| usage().to_owned())?;
    Ok((input, provider_family, json))
}

fn parse_input_json_event_filter<I>(
    args: &mut I,
) -> Result<(PathBuf, bool, NsdbPayloadExecutionEventFilter), String>
where
    I: Iterator<Item = String>,
{
    let mut json = false;
    let mut input = None;
    let mut event_filter = NsdbPayloadExecutionEventFilter::default();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--event-status" => event_filter.status = Some(required_value(args, "--event-status")?),
            "--event-phase" => event_filter.phase = Some(required_value(args, "--event-phase")?),
            "--trace-id" => event_filter.trace_id = Some(required_value(args, "--trace-id")?),
            _ if input.is_none() => input = Some(PathBuf::from(arg)),
            _ => return Err(format!("unexpected argument `{arg}`")),
        }
    }
    let input = input.ok_or_else(|| usage().to_owned())?;
    Ok((input, json, event_filter))
}

fn required_value<I>(args: &mut I, flag: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    args.next()
        .ok_or_else(|| format!("{flag} requires a value\n{}", usage()))
}

#[cfg(test)]
mod tests {
    use super::{parse_args, Command};
    use crate::model::NsdbPayloadExecutionEventFilter;
    use crate::transcript::NsdbReplayControl;
    use std::path::PathBuf;

    #[test]
    fn parses_status_by_default() {
        assert_eq!(
            parse_args(Vec::<String>::new().into_iter()),
            Ok(Command::Status)
        );
    }

    #[test]
    fn parses_inspect_input_and_json_flag() {
        let command = parse_args(
            vec!["inspect".to_owned(), "out".to_owned(), "--json".to_owned()].into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::Inspect {
                input: PathBuf::from("out"),
                json: true,
                event_filter: NsdbPayloadExecutionEventFilter::default(),
            })
        );
    }

    #[test]
    fn parses_inspect_event_filters() {
        let command = parse_args(
            vec![
                "inspect".to_owned(),
                "out".to_owned(),
                "--event-status".to_owned(),
                "blocked".to_owned(),
                "--event-phase".to_owned(),
                "device-dispatch".to_owned(),
                "--trace-id".to_owned(),
                "payload-trace:shader:pixelmagic.blur".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::Inspect {
                input: PathBuf::from("out"),
                json: false,
                event_filter: NsdbPayloadExecutionEventFilter {
                    status: Some("blocked".to_owned()),
                    phase: Some("device-dispatch".to_owned()),
                    trace_id: Some("payload-trace:shader:pixelmagic.blur".to_owned()),
                },
            })
        );
    }

    #[test]
    fn parses_events_command_with_filters() {
        let command = parse_args(
            vec![
                "events".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
                "--event-status".to_owned(),
                "ready".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::Events {
                input: PathBuf::from("out"),
                json: true,
                event_filter: NsdbPayloadExecutionEventFilter {
                    status: Some("ready".to_owned()),
                    phase: None,
                    trace_id: None,
                },
            })
        );
    }

    #[test]
    fn parses_replay_plan_command_with_filters() {
        let command = parse_args(
            vec![
                "replay-plan".to_owned(),
                "out".to_owned(),
                "--event-phase".to_owned(),
                "container-loader-handoff".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::ReplayPlan {
                input: PathBuf::from("out"),
                json: false,
                event_filter: NsdbPayloadExecutionEventFilter {
                    status: None,
                    phase: Some("container-loader-handoff".to_owned()),
                    trace_id: None,
                },
            })
        );
    }

    #[test]
    fn parses_replay_command_with_filters() {
        let command = parse_args(
            vec![
                "replay".to_owned(),
                "out".to_owned(),
                "--json".to_owned(),
                "--trace-id".to_owned(),
                "trace-1".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::Replay {
                input: PathBuf::from("out"),
                json: true,
                event_filter: NsdbPayloadExecutionEventFilter {
                    status: None,
                    phase: None,
                    trace_id: Some("trace-1".to_owned()),
                },
                replay_control: NsdbReplayControl::default(),
                cursor_input: None,
                cursor_output: None,
            })
        );
    }

    #[test]
    fn parses_replay_frame_and_breakpoint_controls() {
        let frame = parse_args(
            vec![
                "replay".to_owned(),
                "out".to_owned(),
                "--frame".to_owned(),
                "3".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            frame,
            Ok(Command::Replay {
                input: PathBuf::from("out"),
                json: false,
                event_filter: NsdbPayloadExecutionEventFilter::default(),
                replay_control: NsdbReplayControl {
                    frame_selector: Some("3".to_owned()),
                    breakpoint_selector: None,
                    breakpoint_phase: None,
                    breakpoint_entry: None,
                    resume_after_frame_id: None,
                    resume_next_frame_id: None,
                },
                cursor_input: None,
                cursor_output: None,
            })
        );

        let conflicting = parse_args(
            vec![
                "replay".to_owned(),
                "out".to_owned(),
                "--frame".to_owned(),
                "3".to_owned(),
                "--break-at".to_owned(),
                "frame:4".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap_err();
        assert!(conflicting.contains("mutually exclusive"));
    }

    #[test]
    fn parses_typed_replay_breakpoint_predicate() {
        let command = parse_args(
            vec![
                "replay".to_owned(),
                "out".to_owned(),
                "--break-phase".to_owned(),
                "device-dispatch".to_owned(),
                "--break-entry".to_owned(),
                "pixelmagic.blur".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::Replay {
                input: PathBuf::from("out"),
                json: false,
                event_filter: NsdbPayloadExecutionEventFilter::default(),
                replay_control: NsdbReplayControl {
                    frame_selector: None,
                    breakpoint_selector: None,
                    breakpoint_phase: Some("device-dispatch".to_owned()),
                    breakpoint_entry: Some("pixelmagic.blur".to_owned()),
                    resume_after_frame_id: None,
                    resume_next_frame_id: None,
                },
                cursor_input: None,
                cursor_output: None,
            })
        );
    }

    #[test]
    fn parses_replay_resume_cursor_pair() {
        let command = parse_args(
            vec![
                "replay".to_owned(),
                "out".to_owned(),
                "--resume-after".to_owned(),
                "frame-0".to_owned(),
                "--resume-next".to_owned(),
                "frame-1".to_owned(),
                "--save-cursor".to_owned(),
                "cursor.toml".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::Replay {
                input: PathBuf::from("out"),
                json: false,
                event_filter: NsdbPayloadExecutionEventFilter::default(),
                replay_control: NsdbReplayControl {
                    frame_selector: None,
                    breakpoint_selector: None,
                    breakpoint_phase: None,
                    breakpoint_entry: None,
                    resume_after_frame_id: Some("frame-0".to_owned()),
                    resume_next_frame_id: Some("frame-1".to_owned()),
                },
                cursor_input: None,
                cursor_output: Some(PathBuf::from("cursor.toml")),
            })
        );

        let incomplete = parse_args(
            vec![
                "replay".to_owned(),
                "out".to_owned(),
                "--resume-after".to_owned(),
                "frame-0".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap_err();
        assert!(incomplete.contains("must be provided together"));
    }

    #[test]
    fn parses_persisted_replay_cursor_input() {
        let command = parse_args(
            vec![
                "replay".to_owned(),
                "out".to_owned(),
                "--resume-cursor".to_owned(),
                "cursor.toml".to_owned(),
                "--break-at".to_owned(),
                "frame-2".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::Replay {
                input: PathBuf::from("out"),
                json: false,
                event_filter: NsdbPayloadExecutionEventFilter::default(),
                replay_control: NsdbReplayControl {
                    breakpoint_selector: Some("frame-2".to_owned()),
                    ..NsdbReplayControl::default()
                },
                cursor_input: Some(PathBuf::from("cursor.toml")),
                cursor_output: None,
            })
        );
    }

    #[test]
    fn parses_materialize_provider_samples_command() {
        let command = parse_args(
            vec![
                "materialize-provider-samples".to_owned(),
                "out".to_owned(),
                "--provider-family".to_owned(),
                "metal:apple-silicon-gpu".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::MaterializeProviderSamples {
                output_dir: PathBuf::from("out"),
                provider_family: Some("metal:apple-silicon-gpu".to_owned()),
                json: true,
            })
        );
    }

    #[test]
    fn parses_execute_provider_samples_command() {
        let command = parse_args(
            vec![
                "execute-provider-samples".to_owned(),
                "out".to_owned(),
                "--provider-family".to_owned(),
                "metal:apple-silicon-gpu".to_owned(),
                "--json".to_owned(),
            ]
            .into_iter(),
        );
        assert_eq!(
            command,
            Ok(Command::ExecuteProviderSamples {
                output_dir: PathBuf::from("out"),
                provider_family: Some("metal:apple-silicon-gpu".to_owned()),
                json: true,
            })
        );
    }
}
