use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind {
    Help,
    Status,
    Registry,
    Bindings { input: PathBuf },
    PackNustar { package_id: String, output: PathBuf },
    InspectNustar { input: PathBuf },
    LoaderContract { package_id: String },
    Check { input: PathBuf },
    Build { input: PathBuf, output_dir: PathBuf },
    DumpAst { input: PathBuf },
    DumpNir { input: PathBuf },
    DumpYir { input: PathBuf },
    Rc { args: Vec<String> },
}

pub fn parse_args<I>(mut args: I) -> Result<CommandKind, String>
where
    I: Iterator<Item = String>,
{
    let command = args.next().unwrap_or_else(|| "status".to_owned());
    match command.as_str() {
        "help" | "--help" | "-h" => Ok(CommandKind::Help),
        "status" => Ok(CommandKind::Status),
        "registry" => Ok(CommandKind::Registry),
        "bindings" => Ok(CommandKind::Bindings {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuis bindings <input.ns|project-dir|nuis.toml>".to_owned())?,
            ),
        }),
        "pack-nustar" => Ok(CommandKind::PackNustar {
            package_id: args
                .next()
                .ok_or_else(|| "usage: nuis pack-nustar <package-id> <output.nustar>".to_owned())?,
            output: PathBuf::from(
                args.next().ok_or_else(|| {
                    "usage: nuis pack-nustar <package-id> <output.nustar>".to_owned()
                })?,
            ),
        }),
        "inspect-nustar" => Ok(CommandKind::InspectNustar {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuis inspect-nustar <input.nustar>".to_owned())?,
            ),
        }),
        "loader-contract" => Ok(CommandKind::LoaderContract {
            package_id: args
                .next()
                .ok_or_else(|| "usage: nuis loader-contract <package-id>".to_owned())?,
        }),
        "check" => Ok(CommandKind::Check {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        }),
        "build" => {
            let first = args.next().ok_or_else(|| {
                "usage: nuis build [input.ns|project-dir|nuis.toml] <output-dir>".to_owned()
            })?;
            let second = args.next();
            let (input, output_dir) = if let Some(output_dir) = second {
                (PathBuf::from(first), PathBuf::from(output_dir))
            } else {
                (PathBuf::from("."), PathBuf::from(first))
            };
            Ok(CommandKind::Build { input, output_dir })
        }
        "dump-ast" => Ok(CommandKind::DumpAst {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        }),
        "dump-nir" => Ok(CommandKind::DumpNir {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        }),
        "dump-yir" => Ok(CommandKind::DumpYir {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        }),
        "rc" => Ok(CommandKind::Rc {
            args: args.collect::<Vec<_>>(),
        }),
        other => Err(format!(
            "unknown nuis command `{other}`; expected `help`, `status`, `registry`, `bindings`, `pack-nustar`, `inspect-nustar`, `loader-contract`, `check`, `build`, `dump-ast`, `dump-nir`, `dump-yir`, or `rc`"
        )),
    }
}
