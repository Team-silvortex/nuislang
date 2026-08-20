use super::{sanitize_path_label, CommandKind};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GalaxyCommand {
    Init {
        input: PathBuf,
        framework: Option<String>,
    },
    Check {
        input: PathBuf,
    },
    Pack {
        input: PathBuf,
        output: PathBuf,
    },
    Inspect {
        input: PathBuf,
    },
    PublishLocal {
        input: PathBuf,
        output: Option<PathBuf>,
    },
    List,
    InstallLocal {
        name: String,
        version: Option<String>,
        output: PathBuf,
    },
    InstallDeps {
        input: PathBuf,
    },
    Doctor {
        input: PathBuf,
    },
    ResolveDeps {
        input: PathBuf,
        provider_root: PathBuf,
        provider_id: String,
        provider_kind: String,
    },
    SyncDeps {
        input: PathBuf,
    },
    LockDeps {
        input: PathBuf,
    },
    VerifyLock {
        input: PathBuf,
    },
    InspectLocal {
        name: String,
        version: Option<String>,
    },
    VerifyLocal {
        name: String,
        version: Option<String>,
    },
    RemoveLocal {
        name: String,
        version: Option<String>,
    },
}

pub(super) fn parse_galaxy_args<I>(mut args: I) -> Result<CommandKind, String>
where
    I: Iterator<Item = String>,
{
    let subcommand = args.next().unwrap_or_else(|| "check".to_owned());
    match subcommand.as_str() {
        "init" => {
            let mut input = PathBuf::from(".".to_owned());
            let mut framework = None;
            while let Some(arg) = args.next() {
                if arg == "--framework" {
                    framework = Some(args.next().ok_or_else(|| {
                        "usage: nuis galaxy init [project-dir] [--framework <name>]".to_owned()
                    })?);
                } else if input == Path::new(".") {
                    input = PathBuf::from(arg);
                } else {
                    return Err(format!(
                        "unknown nuis galaxy init argument `{arg}`; expected `[project-dir] [--framework <name>]`"
                    ));
                }
            }
            Ok(CommandKind::Galaxy(GalaxyCommand::Init { input, framework }))
        }
        "check" => Ok(CommandKind::Galaxy(GalaxyCommand::Check {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        })),
        "pack" => {
            let input = PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned()));
            let output = PathBuf::from(args.next().unwrap_or_else(|| {
                format!(
                    "target/galaxy/{}.galaxy",
                    sanitize_path_label(
                        input
                            .file_stem()
                            .or_else(|| input.file_name())
                            .and_then(|item| item.to_str())
                            .unwrap_or("package")
                    )
                )
            }));
            Ok(CommandKind::Galaxy(GalaxyCommand::Pack { input, output }))
        }
        "inspect" => Ok(CommandKind::Galaxy(GalaxyCommand::Inspect {
            input: PathBuf::from(args.next().ok_or_else(|| {
                "usage: nuis galaxy inspect <input.galaxy>".to_owned()
            })?),
        })),
        "publish-local" => Ok(CommandKind::Galaxy(GalaxyCommand::PublishLocal {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
            output: args.next().map(PathBuf::from),
        })),
        "list" => Ok(CommandKind::Galaxy(GalaxyCommand::List)),
        "install-local" => Ok(CommandKind::Galaxy(GalaxyCommand::InstallLocal {
            name: args.next().ok_or_else(|| {
                "usage: nuis galaxy install-local <name> [version] [output-dir]".to_owned()
            })?,
            version: args.next(),
            output: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        })),
        "install-deps" => Ok(CommandKind::Galaxy(GalaxyCommand::InstallDeps {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        })),
        "doctor" => Ok(CommandKind::Galaxy(GalaxyCommand::Doctor {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        })),
        "resolve-deps" => {
            let mut input = None;
            let mut provider_root = None;
            let mut provider_id = "offline.layout".to_owned();
            let mut provider_kind = "offline-layout".to_owned();
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--provider-root" => {
                        provider_root = Some(PathBuf::from(args.next().ok_or_else(|| {
                            "usage: nuis galaxy resolve-deps [project-dir|nuis.toml] --provider-root <dir> [--provider-id <id>] [--provider-kind <kind>]".to_owned()
                        })?));
                    }
                    "--provider-id" => {
                        provider_id = args.next().ok_or_else(|| {
                            "usage: nuis galaxy resolve-deps [project-dir|nuis.toml] --provider-root <dir> [--provider-id <id>] [--provider-kind <kind>]".to_owned()
                        })?;
                    }
                    "--provider-kind" => {
                        provider_kind = args.next().ok_or_else(|| {
                            "usage: nuis galaxy resolve-deps [project-dir|nuis.toml] --provider-root <dir> [--provider-id <id>] [--provider-kind <kind>]".to_owned()
                        })?;
                    }
                    _ if input.is_none() => input = Some(PathBuf::from(arg)),
                    _ => {
                        return Err(format!(
                            "unknown nuis galaxy resolve-deps argument `{arg}`"
                        ))
                    }
                }
            }
            Ok(CommandKind::Galaxy(GalaxyCommand::ResolveDeps {
                input: input.unwrap_or_else(|| PathBuf::from(".")),
                provider_root: provider_root.ok_or_else(|| {
                    "usage: nuis galaxy resolve-deps [project-dir|nuis.toml] --provider-root <dir> [--provider-id <id>] [--provider-kind <kind>]".to_owned()
                })?,
                provider_id,
                provider_kind,
            }))
        }
        "sync-deps" => Ok(CommandKind::Galaxy(GalaxyCommand::SyncDeps {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        })),
        "lock-deps" => Ok(CommandKind::Galaxy(GalaxyCommand::LockDeps {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        })),
        "verify-lock" => Ok(CommandKind::Galaxy(GalaxyCommand::VerifyLock {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        })),
        "inspect-local" => Ok(CommandKind::Galaxy(GalaxyCommand::InspectLocal {
            name: args.next().ok_or_else(|| {
                "usage: nuis galaxy inspect-local <name> [version]".to_owned()
            })?,
            version: args.next(),
        })),
        "verify-local" => Ok(CommandKind::Galaxy(GalaxyCommand::VerifyLocal {
            name: args.next().ok_or_else(|| {
                "usage: nuis galaxy verify-local <name> [version]".to_owned()
            })?,
            version: args.next(),
        })),
        "remove-local" => Ok(CommandKind::Galaxy(GalaxyCommand::RemoveLocal {
            name: args.next().ok_or_else(|| {
                "usage: nuis galaxy remove-local <name> [version]".to_owned()
            })?,
            version: args.next(),
        })),
        other => Err(format!(
            "unknown nuis galaxy command `{other}`; expected `init`, `check`, `pack`, `inspect`, `publish-local`, `list`, `install-local`, `install-deps`, `doctor`, `resolve-deps`, `sync-deps`, `lock-deps`, `verify-lock`, `inspect-local`, `verify-local`, or `remove-local`"
        )),
    }
}
