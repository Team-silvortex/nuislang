use std::path::{Path, PathBuf};

use crate::model::NsdbPayloadExecutionEventFilter;

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
    MaterializeProviderSamples {
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
        "materialize-provider-samples" => {
            let (output_dir, provider_family, json) =
                parse_provider_materialize_args(args.by_ref())?;
            Ok(Command::MaterializeProviderSamples {
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
    "usage:\n  nsdb status\n  nsdb inspect <nuis.build.manifest.toml|artifact-output-dir> [--json] [--event-status <status>] [--event-phase <phase>] [--trace-id <trace-id>]\n  nsdb events <nuis.build.manifest.toml|artifact-output-dir> [--json] [--event-status <status>] [--event-phase <phase>] [--trace-id <trace-id>]\n  nsdb replay-plan <nuis.build.manifest.toml|artifact-output-dir> [--json] [--event-status <status>] [--event-phase <phase>] [--trace-id <trace-id>]\n  nsdb materialize-provider-samples <artifact-output-dir> [--provider-family <family>] [--json]"
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
}
