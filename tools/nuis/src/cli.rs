use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind {
    Status,
    Registry,
    Bindings { input: PathBuf },
    Check { input: PathBuf },
    Build { input: PathBuf, output_dir: PathBuf },
    DumpNir { input: PathBuf },
    DumpYir { input: PathBuf },
}

pub fn parse_args<I>(mut args: I) -> Result<CommandKind, String>
where
    I: Iterator<Item = String>,
{
    let command = args.next().unwrap_or_else(|| "status".to_owned());
    match command.as_str() {
        "status" => Ok(CommandKind::Status),
        "registry" => Ok(CommandKind::Registry),
        "bindings" => Ok(CommandKind::Bindings {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuis bindings <input.ns>".to_owned())?,
            ),
        }),
        "check" => Ok(CommandKind::Check {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuis check <input.ns>".to_owned())?,
            ),
        }),
        "build" => Ok(CommandKind::Build {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuis build <input.ns> <output-dir>".to_owned())?,
            ),
            output_dir: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuis build <input.ns> <output-dir>".to_owned())?,
            ),
        }),
        "dump-nir" => Ok(CommandKind::DumpNir {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuis dump-nir <input.ns>".to_owned())?,
            ),
        }),
        "dump-yir" => Ok(CommandKind::DumpYir {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuis dump-yir <input.ns>".to_owned())?,
            ),
        }),
        other => Err(format!(
            "unknown nuis command `{other}`; expected `status`, `registry`, `bindings`, `check`, `build`, `dump-nir`, or `dump-yir`"
        )),
    }
}
