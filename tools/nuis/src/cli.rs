use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind {
    Help,
    Status,
    Registry,
    Fmt {
        input: PathBuf,
    },
    Bindings {
        input: PathBuf,
    },
    PackNustar {
        package_id: String,
        output: PathBuf,
    },
    InspectNustar {
        input: PathBuf,
    },
    LoaderContract {
        package_id: String,
    },
    VerifyBuildManifest {
        manifest: PathBuf,
    },
    CacheStatus {
        input: Option<PathBuf>,
        all: bool,
        verbose_cache: bool,
        json: bool,
    },
    CleanCache {
        input: Option<PathBuf>,
        all: bool,
        json: bool,
    },
    PruneCache {
        input: Option<PathBuf>,
        all: bool,
        keep: usize,
        json: bool,
    },
    ReleaseCheck {
        input: PathBuf,
        output_dir: PathBuf,
        cpu_abi: Option<String>,
        target: Option<String>,
    },
    Check {
        input: PathBuf,
    },
    Test {
        input: PathBuf,
        list: bool,
        ignored_only: bool,
        include_ignored: bool,
        exact: bool,
        filter: Option<String>,
    },
    Build {
        input: PathBuf,
        output_dir: PathBuf,
        verbose_cache: bool,
        cpu_abi: Option<String>,
        target: Option<String>,
    },
    DumpAst {
        input: PathBuf,
    },
    DumpNir {
        input: PathBuf,
    },
    DumpYir {
        input: PathBuf,
    },
    Workflow {
        input: PathBuf,
        json: bool,
    },
    SchedulerView {
        input: PathBuf,
        json: bool,
    },
    Rc {
        args: Vec<String>,
    },
    ProjectStatus {
        input: PathBuf,
        json: bool,
    },
    ProjectDoctor {
        input: PathBuf,
        json: bool,
    },
    ProjectLockAbi {
        input: PathBuf,
    },
    Galaxy(GalaxyCommand),
}

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

pub fn parse_args<I>(mut args: I) -> Result<CommandKind, String>
where
    I: Iterator<Item = String>,
{
    let command = args.next().unwrap_or_else(|| "status".to_owned());
    match command.as_str() {
        "help" | "--help" | "-h" => Ok(CommandKind::Help),
        "status" => Ok(CommandKind::Status),
        "registry" => Ok(CommandKind::Registry),
        "fmt" => Ok(CommandKind::Fmt {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        }),
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
        "verify-build-manifest" => Ok(CommandKind::VerifyBuildManifest {
            manifest: PathBuf::from(args.next().ok_or_else(|| {
                "usage: nuis verify-build-manifest <nuis.build.manifest.toml>".to_owned()
            })?),
        }),
        "cache-status" => {
            let mut verbose_cache = false;
            let mut all = false;
            let mut json = false;
            let mut input = None;
            for arg in args.by_ref() {
                if arg == "--verbose-cache" {
                    verbose_cache = true;
                } else if arg == "--all" {
                    all = true;
                } else if arg == "--json" {
                    json = true;
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(
                        "usage: nuis cache-status [--all] [--verbose-cache] [--json] [input.ns|project-dir|nuis.toml]"
                            .to_owned(),
                    );
                }
            }
            Ok(CommandKind::CacheStatus {
                input: if all {
                    input
                } else {
                    Some(input.unwrap_or_else(|| PathBuf::from(".")))
                },
                all,
                verbose_cache,
                json,
            })
        }
        "clean-cache" => {
            let mut all = false;
            let mut json = false;
            let mut input = None;
            for arg in args.by_ref() {
                if arg == "--all" {
                    all = true;
                } else if arg == "--json" {
                    json = true;
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(
                        "usage: nuis clean-cache [--all] [--json] [input.ns|project-dir|nuis.toml]"
                            .to_owned(),
                    );
                }
            }
            Ok(CommandKind::CleanCache {
                input: if all {
                    input
                } else {
                    Some(input.unwrap_or_else(|| PathBuf::from(".")))
                },
                all,
                json,
            })
        }
        "cache-prune" => {
            let mut all = false;
            let mut input = None;
            let mut keep = 4usize;
            let mut json = false;
            while let Some(arg) = args.next() {
                if arg == "--all" {
                    all = true;
                } else if arg == "--json" {
                    json = true;
                } else if arg == "--keep" {
                    let raw = args.next().ok_or_else(|| {
                        "usage: nuis cache-prune [--all] [--keep N] [--json] [input.ns|project-dir|nuis.toml]"
                            .to_owned()
                    })?;
                    keep = raw.parse::<usize>().map_err(|_| {
                        format!("invalid value for `--keep`: `{raw}`; expected non-negative integer")
                    })?;
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(
                        "usage: nuis cache-prune [--all] [--keep N] [--json] [input.ns|project-dir|nuis.toml]"
                            .to_owned(),
                    );
                }
            }
            Ok(CommandKind::PruneCache {
                input: if all {
                    input
                } else {
                    Some(input.unwrap_or_else(|| PathBuf::from(".")))
                },
                all,
                keep,
                json,
            })
        }
        "release-check" => {
            let mut cpu_abi = None;
            let mut target = None;
            let mut positional = Vec::new();
            while let Some(arg) = args.next() {
                if arg == "--cpu-abi" {
                    cpu_abi = Some(args.next().ok_or_else(|| {
                        "usage: nuis release-check [--cpu-abi ABI] [--target TRIPLE] [input.ns|project-dir|nuis.toml] [output-dir]"
                            .to_owned()
                    })?);
                } else if arg == "--target" {
                    target = Some(args.next().ok_or_else(|| {
                        "usage: nuis release-check [--cpu-abi ABI] [--target TRIPLE] [input.ns|project-dir|nuis.toml] [output-dir]"
                            .to_owned()
                    })?);
                } else {
                    positional.push(arg);
                }
            }
            let input = PathBuf::from(positional.first().cloned().unwrap_or_else(|| ".".to_owned()));
            let output_dir = PathBuf::from(positional.get(1).cloned().unwrap_or_else(|| {
                format!(
                    "target/nuis-release-check/{}",
                    sanitize_path_label(
                        input
                            .file_stem()
                            .or_else(|| input.file_name())
                            .and_then(|item| item.to_str())
                            .unwrap_or("input")
                    )
                )
            }));
            if positional.len() > 2 {
                return Err(
                    "usage: nuis release-check [--cpu-abi ABI] [--target TRIPLE] [input.ns|project-dir|nuis.toml] [output-dir]"
                        .to_owned(),
                );
            }
            Ok(CommandKind::ReleaseCheck {
                input,
                output_dir,
                cpu_abi,
                target,
            })
        }
        "check" => Ok(CommandKind::Check {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        }),
        "test" => {
            let mut list = false;
            let mut ignored_only = false;
            let mut include_ignored = false;
            let mut exact = false;
            let mut positional = Vec::new();
            while let Some(arg) = args.next() {
                if arg == "--list" {
                    list = true;
                } else if arg == "--ignored" {
                    ignored_only = true;
                } else if arg == "--include-ignored" {
                    include_ignored = true;
                } else if arg == "--exact" {
                    exact = true;
                } else {
                    positional.push(arg);
                }
            }
            if ignored_only && include_ignored {
                return Err(
                    "usage: nuis test [--list] [--ignored|--include-ignored] [--exact] [input.ns|project-dir|nuis.toml] [filter]"
                        .to_owned(),
                );
            }
            if positional.len() > 2 {
                return Err(
                    "usage: nuis test [--list] [--ignored|--include-ignored] [--exact] [input.ns|project-dir|nuis.toml] [filter]"
                        .to_owned(),
                );
            }
            Ok(CommandKind::Test {
                input: PathBuf::from(positional.first().cloned().unwrap_or_else(|| ".".to_owned())),
                list,
                ignored_only,
                include_ignored,
                exact,
                filter: positional.get(1).cloned(),
            })
        }
        "build" => {
            let mut verbose_cache = false;
            let mut cpu_abi = None;
            let mut target = None;
            let mut positional = Vec::new();
            while let Some(arg) = args.next() {
                if arg == "--verbose-cache" {
                    verbose_cache = true;
                } else if arg == "--cpu-abi" {
                    cpu_abi = Some(args.next().ok_or_else(|| {
                        "usage: nuis build [--verbose-cache] [--cpu-abi ABI] [--target TRIPLE] [input.ns|project-dir|nuis.toml] <output-dir>"
                            .to_owned()
                    })?);
                } else if arg == "--target" {
                    target = Some(args.next().ok_or_else(|| {
                        "usage: nuis build [--verbose-cache] [--cpu-abi ABI] [--target TRIPLE] [input.ns|project-dir|nuis.toml] <output-dir>"
                            .to_owned()
                    })?);
                } else {
                    positional.push(arg);
                }
            }
            let (input, output_dir) = match positional.len() {
                1 => (PathBuf::from("."), PathBuf::from(&positional[0])),
                2 => (PathBuf::from(&positional[0]), PathBuf::from(&positional[1])),
                _ => {
                    return Err(
                        "usage: nuis build [--verbose-cache] [--cpu-abi ABI] [--target TRIPLE] [input.ns|project-dir|nuis.toml] <output-dir>"
                            .to_owned(),
                    )
                }
            };
            Ok(CommandKind::Build {
                input,
                output_dir,
                verbose_cache,
                cpu_abi,
                target,
            })
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
        "workflow" => {
            let mut json = false;
            let mut input = None;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(
                        "usage: nuis workflow [--json] [input.ns|project-dir|nuis.toml]"
                            .to_owned(),
                    );
                }
            }
            Ok(CommandKind::Workflow {
                input: input.unwrap_or_else(|| PathBuf::from(".")),
                json,
            })
        }
        "scheduler-view" => {
            let mut json = false;
            let mut input = None;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(
                        "usage: nuis scheduler-view [--json] [input.ns|project-dir|nuis.toml]"
                            .to_owned(),
                    );
                }
            }
            Ok(CommandKind::SchedulerView {
                input: input.unwrap_or_else(|| PathBuf::from(".")),
                json,
            })
        }
        "rc" => Ok(CommandKind::Rc {
            args: args.collect::<Vec<_>>(),
        }),
        "project-status" => {
            let mut json = false;
            let mut input = None;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(
                        "usage: nuis project-status [--json] [project-dir|nuis.toml]"
                            .to_owned(),
                    );
                }
            }
            Ok(CommandKind::ProjectStatus {
                input: input.unwrap_or_else(|| PathBuf::from(".")),
                json,
            })
        }
        "project-doctor" => {
            let mut json = false;
            let mut input = None;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(
                        "usage: nuis project-doctor [--json] [project-dir|nuis.toml]"
                            .to_owned(),
                    );
                }
            }
            Ok(CommandKind::ProjectDoctor {
                input: input.unwrap_or_else(|| PathBuf::from(".")),
                json,
            })
        }
        "project-lock-abi" => Ok(CommandKind::ProjectLockAbi {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        }),
        "galaxy" => parse_galaxy_args(args),
        other => Err(format!(
            "unknown nuis command `{other}`; expected `help`, `status`, `registry`, `fmt`, `bindings`, `pack-nustar`, `inspect-nustar`, `loader-contract`, `verify-build-manifest`, `cache-status`, `clean-cache`, `cache-prune`, `release-check`, `check`, `test`, `build`, `dump-ast`, `dump-nir`, `dump-yir`, `workflow`, `scheduler-view`, `rc`, `project-status`, `project-doctor`, `project-lock-abi`, or `galaxy`"
        )),
    }
}

fn parse_galaxy_args<I>(mut args: I) -> Result<CommandKind, String>
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
                } else if input == PathBuf::from(".") {
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
            "unknown nuis galaxy command `{other}`; expected `init`, `check`, `pack`, `inspect`, `publish-local`, `list`, `install-local`, `install-deps`, `doctor`, `sync-deps`, `lock-deps`, `verify-lock`, `inspect-local`, `verify-local`, or `remove-local`"
        )),
    }
}

fn sanitize_path_label(raw: &str) -> String {
    let mut out = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "input".to_owned()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_args, CommandKind};
    use std::path::PathBuf;

    #[test]
    fn parses_workflow_with_default_input() {
        let command = parse_args(["workflow".to_owned()].into_iter()).expect("workflow parses");
        assert_eq!(
            command,
            CommandKind::Workflow {
                input: PathBuf::from("."),
                json: false,
            }
        );
    }

    #[test]
    fn parses_workflow_json_with_explicit_input() {
        let command = parse_args(
            [
                "workflow".to_owned(),
                "--json".to_owned(),
                "examples/demo.ns".to_owned(),
            ]
            .into_iter(),
        )
        .expect("workflow json parses");
        assert_eq!(
            command,
            CommandKind::Workflow {
                input: PathBuf::from("examples/demo.ns"),
                json: true,
            }
        );
    }
}
