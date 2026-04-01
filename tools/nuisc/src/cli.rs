use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind {
    Status,
    Registry,
    Bindings { input: PathBuf },
    PackNustar { package_id: String, output: PathBuf },
    InspectNustar { input: PathBuf },
    LoaderContract { package_id: String },
    DumpNir { input: PathBuf },
    DumpYir { input: PathBuf },
    Check { input: PathBuf },
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
        "bindings" => Ok(CommandKind::Bindings {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuisc bindings <input.ns>".to_owned())?,
            ),
        }),
        "pack-nustar" => Ok(CommandKind::PackNustar {
            package_id: args
                .next()
                .ok_or_else(|| "usage: nuisc pack-nustar <package-id> <output.nustar>".to_owned())?,
            output: PathBuf::from(
                args.next().ok_or_else(|| {
                    "usage: nuisc pack-nustar <package-id> <output.nustar>".to_owned()
                })?,
            ),
        }),
        "inspect-nustar" => Ok(CommandKind::InspectNustar {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuisc inspect-nustar <input.nustar>".to_owned())?,
            ),
        }),
        "loader-contract" => Ok(CommandKind::LoaderContract {
            package_id: args
                .next()
                .ok_or_else(|| "usage: nuisc loader-contract <package-id>".to_owned())?,
        }),
        "dump-nir" => Ok(CommandKind::DumpNir {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuisc dump-nir <input.ns>".to_owned())?,
            ),
        }),
        "dump-yir" => Ok(CommandKind::DumpYir {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuisc dump-yir <input.ns>".to_owned())?,
            ),
        }),
        "check" => Ok(CommandKind::Check {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuisc check <input.ns>".to_owned())?,
            ),
        }),
        "compile" => Ok(CommandKind::Compile {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuisc compile <input.ns> <output-dir>".to_owned())?,
            ),
            output_dir: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuisc compile <input.ns> <output-dir>".to_owned())?,
            ),
        }),
        other => Err(format!(
            "unknown nuisc command `{other}`; expected `status`, `registry`, `bindings`, `pack-nustar`, `inspect-nustar`, `loader-contract`, `dump-nir`, `dump-yir`, `check`, or `compile`"
        )),
    }
}
