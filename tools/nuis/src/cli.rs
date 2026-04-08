use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind {
    Help,
    Status,
    Registry,
    Fmt { input: PathBuf },
    Bindings { input: PathBuf },
    PackNustar { package_id: String, output: PathBuf },
    InspectNustar { input: PathBuf },
    LoaderContract { package_id: String },
    VerifyBuildManifest { manifest: PathBuf },
    ReleaseCheck { input: PathBuf, output_dir: PathBuf },
    Check { input: PathBuf },
    Build { input: PathBuf, output_dir: PathBuf },
    DumpAst { input: PathBuf },
    DumpNir { input: PathBuf },
    DumpYir { input: PathBuf },
    Rc { args: Vec<String> },
    ProjectStatus { input: PathBuf },
    ProjectLockAbi { input: PathBuf },
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
        "release-check" => {
            let input = PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned()));
            let output_dir = PathBuf::from(args.next().unwrap_or_else(|| {
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
            Ok(CommandKind::ReleaseCheck { input, output_dir })
        }
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
        "project-status" => Ok(CommandKind::ProjectStatus {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        }),
        "project-lock-abi" => Ok(CommandKind::ProjectLockAbi {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        }),
        other => Err(format!(
            "unknown nuis command `{other}`; expected `help`, `status`, `registry`, `fmt`, `bindings`, `pack-nustar`, `inspect-nustar`, `loader-contract`, `verify-build-manifest`, `release-check`, `check`, `build`, `dump-ast`, `dump-nir`, `dump-yir`, `rc`, `project-status`, or `project-lock-abi`"
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
