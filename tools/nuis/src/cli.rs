use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind {
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
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuis check <input.ns|project-dir|nuis.toml>".to_owned())?,
            ),
        }),
        "build" => Ok(CommandKind::Build {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuis build <input.ns|project-dir|nuis.toml> <output-dir>".to_owned())?,
            ),
            output_dir: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuis build <input.ns|project-dir|nuis.toml> <output-dir>".to_owned())?,
            ),
        }),
        "dump-ast" => Ok(CommandKind::DumpAst {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuis dump-ast <input.ns|project-dir|nuis.toml>".to_owned())?,
            ),
        }),
        "dump-nir" => Ok(CommandKind::DumpNir {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuis dump-nir <input.ns|project-dir|nuis.toml>".to_owned())?,
            ),
        }),
        "dump-yir" => Ok(CommandKind::DumpYir {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuis dump-yir <input.ns|project-dir|nuis.toml>".to_owned())?,
            ),
        }),
        other => Err(format!(
            "unknown nuis command `{other}`; expected `status`, `registry`, `bindings`, `pack-nustar`, `inspect-nustar`, `loader-contract`, `check`, `build`, `dump-ast`, `dump-nir`, or `dump-yir`"
        )),
    }
}
