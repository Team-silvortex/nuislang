use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind {
    Status,
    Registry,
    DumpNir { input: PathBuf },
    DumpYir { input: PathBuf },
    Compile { input: PathBuf, output_dir: PathBuf },
}

pub fn parse_args<I>(mut args: I) -> Result<CommandKind, String>
where
    I: Iterator<Item = String>,
{
    let command = args.next().unwrap_or_else(|| "status".to_owned());
    match command.as_str() {
        "status" => Ok(CommandKind::Status),
        "registry" => Ok(CommandKind::Registry),
        "dump-nir" => Ok(CommandKind::DumpNir {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: cargo run -p nuisc -- dump-nir <input.ns>".to_owned())?,
            ),
        }),
        "dump-yir" => Ok(CommandKind::DumpYir {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: cargo run -p nuisc -- dump-yir <input.ns>".to_owned())?,
            ),
        }),
        "compile" => Ok(CommandKind::Compile {
            input: PathBuf::from(args.next().ok_or_else(|| {
                "usage: cargo run -p nuisc -- compile <input.ns> <output-dir>".to_owned()
            })?),
            output_dir: PathBuf::from(args.next().ok_or_else(|| {
                "usage: cargo run -p nuisc -- compile <input.ns> <output-dir>".to_owned()
            })?),
        }),
        other => Err(format!(
            "unknown nuisc command `{other}`; expected `status`, `registry`, `dump-nir`, `dump-yir`, or `compile`"
        )),
    }
}
