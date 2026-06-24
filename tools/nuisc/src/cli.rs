use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind {
    Status,
    Registry {
        json: bool,
    },
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
    PackEnvelope {
        input: PathBuf,
        output: PathBuf,
    },
    UnpackEnvelope {
        input: PathBuf,
        output: PathBuf,
    },
    InspectEnvelope {
        input: PathBuf,
    },
    InspectArtifact {
        input: PathBuf,
        json: bool,
    },
    InspectExecution {
        input: PathBuf,
        json: bool,
    },
    ArtifactReport {
        input: PathBuf,
        json: bool,
        summary: bool,
    },
    VerifyArtifact {
        input: PathBuf,
        json: bool,
    },
    UnpackArtifact {
        input: PathBuf,
        output_dir: PathBuf,
    },
    VerifyBuildManifest {
        manifest: PathBuf,
        json: bool,
    },
    InspectBenchmarks {
        input: PathBuf,
        json: bool,
    },
    InspectDocs {
        input: PathBuf,
        json: bool,
        output: Option<PathBuf>,
    },
    InspectGalaxyDocs {
        galaxy: String,
        json: bool,
    },
    InspectStdlibDocs {
        json: bool,
    },
    InspectProjectMetadata {
        input: PathBuf,
        json: bool,
        summary: bool,
        paths_only: bool,
    },
    RepairProjectMetadata {
        input: PathBuf,
        dry_run: bool,
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
    DumpAst {
        input: PathBuf,
    },
    DumpNir {
        input: PathBuf,
    },
    DumpYir {
        input: PathBuf,
    },
    Check {
        input: PathBuf,
    },
    Compile {
        input: PathBuf,
        output_dir: PathBuf,
        verbose_cache: bool,
        cpu_abi: Option<String>,
        target: Option<String>,
    },
}

pub fn parse_args<I>(mut args: I) -> Result<CommandKind, String>
where
    I: Iterator<Item = String>,
{
    let command = args.next().unwrap_or_else(|| "status".to_owned());
    match command.as_str() {
        "status" => Ok(CommandKind::Status),
        "registry" => {
            let mut json = false;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else {
                    return Err("usage: nuisc registry [--json]".to_owned());
                }
            }
            Ok(CommandKind::Registry { json })
        }
        "fmt" => Ok(CommandKind::Fmt {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        }),
        "bindings" => Ok(CommandKind::Bindings {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuisc bindings <input.ns|project-dir|nuis.toml>".to_owned())?,
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
        "pack-envelope" => Ok(CommandKind::PackEnvelope {
            input: PathBuf::from(args.next().ok_or_else(|| {
                "usage: nuisc pack-envelope <nuis.executable.envelope.toml|nuis.build.manifest.toml> <output.nenv>"
                    .to_owned()
            })?),
            output: PathBuf::from(args.next().ok_or_else(|| {
                "usage: nuisc pack-envelope <nuis.executable.envelope.toml|nuis.build.manifest.toml> <output.nenv>"
                    .to_owned()
            })?),
        }),
        "unpack-envelope" => Ok(CommandKind::UnpackEnvelope {
            input: PathBuf::from(args.next().ok_or_else(|| {
                "usage: nuisc unpack-envelope <input.nenv> <output.toml>".to_owned()
            })?),
            output: PathBuf::from(args.next().ok_or_else(|| {
                "usage: nuisc unpack-envelope <input.nenv> <output.toml>".to_owned()
            })?),
        }),
        "inspect-envelope" => Ok(CommandKind::InspectEnvelope {
            input: PathBuf::from(args.next().ok_or_else(|| {
                "usage: nuisc inspect-envelope <nuis.executable.envelope.toml|nuis.build.manifest.toml|envelope.bin>".to_owned()
            })?),
        }),
        "inspect-artifact" => {
            let mut json = false;
            let mut input = None;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(
                        "usage: nuisc inspect-artifact [--json] <output-dir|nuis.compiled.artifact|nuis.build.manifest.toml>"
                            .to_owned(),
                    );
                }
            }
            Ok(CommandKind::InspectArtifact {
                input: input.ok_or_else(|| {
                    "usage: nuisc inspect-artifact [--json] <output-dir|nuis.compiled.artifact|nuis.build.manifest.toml>"
                        .to_owned()
                })?,
                json,
            })
        }
        "inspect-execution" => {
            let mut json = false;
            let mut input = None;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(
                        "usage: nuisc inspect-execution [--json] <output-dir|nuis.compiled.artifact|nuis.build.manifest.toml>"
                            .to_owned(),
                    );
                }
            }
            Ok(CommandKind::InspectExecution {
                input: input.ok_or_else(|| {
                    "usage: nuisc inspect-execution [--json] <output-dir|nuis.compiled.artifact|nuis.build.manifest.toml>"
                        .to_owned()
                })?,
                json,
            })
        }
        "artifact-report" => {
            let mut json = false;
            let mut summary = false;
            let mut input = None;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else if arg == "--summary" {
                    summary = true;
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(
                        "usage: nuisc artifact-report [--json|--summary] <output-dir|nuis.compiled.artifact|nuis.build.manifest.toml>"
                            .to_owned(),
                    );
                }
            }
            if json && summary {
                return Err(
                    "usage: nuisc artifact-report [--json|--summary] <output-dir|nuis.compiled.artifact|nuis.build.manifest.toml>"
                        .to_owned(),
                );
            }
            Ok(CommandKind::ArtifactReport {
                input: input.ok_or_else(|| {
                    "usage: nuisc artifact-report [--json|--summary] <output-dir|nuis.compiled.artifact|nuis.build.manifest.toml>"
                        .to_owned()
                })?,
                json,
                summary,
            })
        }
        "verify-artifact" => {
            let mut json = false;
            let mut input = None;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(
                        "usage: nuisc verify-artifact [--json] <output-dir|nuis.compiled.artifact>"
                            .to_owned(),
                    );
                }
            }
            Ok(CommandKind::VerifyArtifact {
                input: input.ok_or_else(|| {
                    "usage: nuisc verify-artifact [--json] <output-dir|nuis.compiled.artifact>"
                        .to_owned()
                })?,
                json,
            })
        }
        "unpack-artifact" => Ok(CommandKind::UnpackArtifact {
            input: PathBuf::from(args.next().ok_or_else(|| {
                "usage: nuisc unpack-artifact <nuis.compiled.artifact> <output-dir>".to_owned()
            })?),
            output_dir: PathBuf::from(args.next().ok_or_else(|| {
                "usage: nuisc unpack-artifact <nuis.compiled.artifact> <output-dir>".to_owned()
            })?),
        }),
        "verify-build-manifest" => {
            let mut json = false;
            let mut manifest = None;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else if manifest.is_none() {
                    manifest = Some(PathBuf::from(arg));
                } else {
                    return Err(
                        "usage: nuisc verify-build-manifest [--json] <output-dir|nuis.build.manifest.toml>"
                            .to_owned(),
                    );
                }
            }
            Ok(CommandKind::VerifyBuildManifest {
                manifest: manifest.ok_or_else(|| {
                    "usage: nuisc verify-build-manifest [--json] <output-dir|nuis.build.manifest.toml>"
                        .to_owned()
                })?,
                json,
            })
        }
        "inspect-benchmarks" => {
            let mut json = false;
            let mut input = None;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(
                        "usage: nuisc inspect-benchmarks [--json] <input.ns|project-dir|nuis.toml>"
                            .to_owned(),
                    );
                }
            }
            Ok(CommandKind::InspectBenchmarks {
                input: input.ok_or_else(|| {
                    "usage: nuisc inspect-benchmarks [--json] <input.ns|project-dir|nuis.toml>"
                        .to_owned()
                })?,
                json,
            })
        }
        "inspect-docs" => {
            let mut json = false;
            let mut input = None;
            let mut output = None;
            while let Some(arg) = args.next() {
                if arg == "--json" {
                    json = true;
                } else if arg == "--output" {
                    output = Some(PathBuf::from(args.next().ok_or_else(|| {
                        "usage: nuisc inspect-docs [--json] [--output <doc-index.json>] <input.ns|project-dir|nuis.toml>"
                            .to_owned()
                    })?));
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(
                        "usage: nuisc inspect-docs [--json] [--output <doc-index.json>] <input.ns|project-dir|nuis.toml>"
                            .to_owned(),
                    );
                }
            }
            if output.is_some() && !json {
                return Err(
                    "usage: nuisc inspect-docs [--json] [--output <doc-index.json>] <input.ns|project-dir|nuis.toml>"
                        .to_owned(),
                );
            }
            Ok(CommandKind::InspectDocs {
                input: input.ok_or_else(|| {
                    "usage: nuisc inspect-docs [--json] [--output <doc-index.json>] <input.ns|project-dir|nuis.toml>"
                        .to_owned()
                })?,
                json,
                output,
            })
        }
        "inspect-galaxy-docs" => {
            let mut json = false;
            let mut galaxy = None;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else if galaxy.is_none() {
                    galaxy = Some(arg);
                } else {
                    return Err(
                        "usage: nuisc inspect-galaxy-docs [--json] <galaxy-name>".to_owned(),
                    );
                }
            }
            Ok(CommandKind::InspectGalaxyDocs {
                galaxy: galaxy.ok_or_else(|| {
                    "usage: nuisc inspect-galaxy-docs [--json] <galaxy-name>".to_owned()
                })?,
                json,
            })
        }
        "inspect-stdlib-docs" => {
            let mut json = false;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else {
                    return Err("usage: nuisc inspect-stdlib-docs [--json]".to_owned());
                }
            }
            Ok(CommandKind::InspectStdlibDocs { json })
        }
        "inspect-project-metadata" => {
            let mut json = false;
            let mut summary = false;
            let mut paths_only = false;
            let mut input = None;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else if arg == "--summary" {
                    summary = true;
                } else if arg == "--paths-only" {
                    paths_only = true;
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(
                        "usage: nuisc inspect-project-metadata [--json|--summary|--paths-only] <project-dir|nuis.toml|nuis.build.manifest.toml|nuis.compiled.artifact>"
                            .to_owned(),
                    );
                }
            }
            if json && (summary || paths_only) {
                return Err(
                    "usage: nuisc inspect-project-metadata [--json|--summary|--paths-only] <project-dir|nuis.toml|nuis.build.manifest.toml|nuis.compiled.artifact>"
                        .to_owned(),
                );
            }
            if summary && paths_only {
                return Err(
                    "usage: nuisc inspect-project-metadata [--json|--summary|--paths-only] <project-dir|nuis.toml|nuis.build.manifest.toml|nuis.compiled.artifact>"
                        .to_owned(),
                );
            }
            Ok(CommandKind::InspectProjectMetadata {
                input: input.ok_or_else(|| {
                    "usage: nuisc inspect-project-metadata [--json|--summary|--paths-only] <project-dir|nuis.toml|nuis.build.manifest.toml|nuis.compiled.artifact>"
                        .to_owned()
                })?,
                json,
                summary,
                paths_only,
            })
        }
        "repair-project-metadata" => {
            let mut dry_run = false;
            let mut input = None;
            for arg in args.by_ref() {
                if arg == "--dry-run" {
                    dry_run = true;
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(
                        "usage: nuisc repair-project-metadata [--dry-run] <output-dir|nuis.build.manifest.toml|nuis.compiled.artifact>"
                            .to_owned(),
                    );
                }
            }
            Ok(CommandKind::RepairProjectMetadata {
                input: input.ok_or_else(|| {
                    "usage: nuisc repair-project-metadata [--dry-run] <output-dir|nuis.build.manifest.toml|nuis.compiled.artifact>"
                        .to_owned()
                })?,
                dry_run,
            })
        }
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
                        "usage: nuisc cache-status [--all] [--verbose-cache] [--json] [input.ns|project-dir|nuis.toml]"
                            .to_owned(),
                    );
                }
            }
            Ok(CommandKind::CacheStatus {
                input: if all {
                    input
                } else {
                    Some(input.ok_or_else(|| {
                        "usage: nuisc cache-status [--all] [--verbose-cache] [--json] [input.ns|project-dir|nuis.toml]"
                            .to_owned()
                    })?)
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
                        "usage: nuisc clean-cache [--all] [--json] [input.ns|project-dir|nuis.toml]"
                            .to_owned(),
                    );
                }
            }
            Ok(CommandKind::CleanCache {
                input: if all {
                    input
                } else {
                    Some(input.ok_or_else(|| {
                        "usage: nuisc clean-cache [--all] [--json] [input.ns|project-dir|nuis.toml]"
                            .to_owned()
                    })?)
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
                        "usage: nuisc cache-prune [--all] [--keep N] [--json] [input.ns|project-dir|nuis.toml]"
                            .to_owned()
                    })?;
                    keep = raw.parse::<usize>().map_err(|_| {
                        format!("invalid value for `--keep`: `{raw}`; expected non-negative integer")
                    })?;
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(
                        "usage: nuisc cache-prune [--all] [--keep N] [--json] [input.ns|project-dir|nuis.toml]"
                            .to_owned(),
                    );
                }
            }
            Ok(CommandKind::PruneCache {
                input: if all {
                    input
                } else {
                    Some(input.ok_or_else(|| {
                        "usage: nuisc cache-prune [--all] [--keep N] [--json] [input.ns|project-dir|nuis.toml]"
                            .to_owned()
                    })?)
                },
                all,
                keep,
                json,
            })
        }
        "dump-ast" => Ok(CommandKind::DumpAst {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuisc dump-ast <input.ns|project-dir|nuis.toml>".to_owned())?,
            ),
        }),
        "dump-nir" => Ok(CommandKind::DumpNir {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuisc dump-nir <input.ns|project-dir|nuis.toml>".to_owned())?,
            ),
        }),
        "dump-yir" => Ok(CommandKind::DumpYir {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuisc dump-yir <input.ns|project-dir|nuis.toml>".to_owned())?,
            ),
        }),
        "check" => Ok(CommandKind::Check {
            input: PathBuf::from(
                args.next()
                    .ok_or_else(|| "usage: nuisc check <input.ns|project-dir|nuis.toml>".to_owned())?,
            ),
        }),
        "compile" => {
            let mut verbose_cache = false;
            let mut cpu_abi = None;
            let mut target = None;
            let mut positional = Vec::new();
            while let Some(arg) = args.next() {
                if arg == "--verbose-cache" {
                    verbose_cache = true;
                } else if arg == "--cpu-abi" {
                    cpu_abi = Some(args.next().ok_or_else(|| {
                        "usage: nuisc compile [--verbose-cache] [--cpu-abi ABI] [--target TRIPLE] <input.ns|project-dir|nuis.toml> <output-dir>"
                            .to_owned()
                    })?);
                } else if arg == "--target" {
                    target = Some(args.next().ok_or_else(|| {
                        "usage: nuisc compile [--verbose-cache] [--cpu-abi ABI] [--target TRIPLE] <input.ns|project-dir|nuis.toml> <output-dir>"
                            .to_owned()
                    })?);
                } else {
                    positional.push(arg);
                }
            }
            if positional.len() != 2 {
                return Err(
                    "usage: nuisc compile [--verbose-cache] [--cpu-abi ABI] [--target TRIPLE] <input.ns|project-dir|nuis.toml> <output-dir>"
                        .to_owned(),
                );
            }
            Ok(CommandKind::Compile {
                input: PathBuf::from(&positional[0]),
                output_dir: PathBuf::from(&positional[1]),
                verbose_cache,
                cpu_abi,
                target,
            })
        }
        other => Err(format!(
            "unknown nuisc command `{other}`; expected `status`, `registry`, `fmt`, `bindings`, `pack-nustar`, `inspect-nustar`, `loader-contract`, `pack-envelope`, `unpack-envelope`, `inspect-envelope`, `inspect-artifact`, `inspect-execution`, `artifact-report`, `verify-artifact`, `unpack-artifact`, `verify-build-manifest`, `inspect-benchmarks`, `inspect-docs`, `inspect-galaxy-docs`, `inspect-stdlib-docs`, `inspect-project-metadata`, `repair-project-metadata`, `cache-status`, `clean-cache`, `cache-prune`, `dump-ast`, `dump-nir`, `dump-yir`, `check`, or `compile`"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_args, CommandKind};
    use std::path::PathBuf;

    #[test]
    fn parse_pack_envelope_command() {
        let command = parse_args(
            vec![
                "pack-envelope".to_owned(),
                "nuis.build.manifest.toml".to_owned(),
                "out.nenv".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::PackEnvelope {
                input: PathBuf::from("nuis.build.manifest.toml"),
                output: PathBuf::from("out.nenv"),
            }
        );
    }

    #[test]
    fn parse_unpack_envelope_command() {
        let command = parse_args(
            vec![
                "unpack-envelope".to_owned(),
                "artifact.nenv".to_owned(),
                "out.toml".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::UnpackEnvelope {
                input: PathBuf::from("artifact.nenv"),
                output: PathBuf::from("out.toml"),
            }
        );
    }

    #[test]
    fn parse_inspect_artifact_command() {
        let command = parse_args(
            vec![
                "inspect-artifact".to_owned(),
                "nuis.compiled.artifact".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::InspectArtifact {
                input: PathBuf::from("nuis.compiled.artifact"),
                json: false,
            }
        );
    }

    #[test]
    fn parse_verify_artifact_command() {
        let command = parse_args(
            vec![
                "verify-artifact".to_owned(),
                "nuis.compiled.artifact".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::VerifyArtifact {
                input: PathBuf::from("nuis.compiled.artifact"),
                json: false,
            }
        );
    }

    #[test]
    fn parse_inspect_execution_command() {
        let command = parse_args(
            vec![
                "inspect-execution".to_owned(),
                "nuis.compiled.artifact".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::InspectExecution {
                input: PathBuf::from("nuis.compiled.artifact"),
                json: false,
            }
        );
    }

    #[test]
    fn parse_inspect_execution_json_command() {
        let command = parse_args(
            vec![
                "inspect-execution".to_owned(),
                "--json".to_owned(),
                "nuis.compiled.artifact".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::InspectExecution {
                input: PathBuf::from("nuis.compiled.artifact"),
                json: true,
            }
        );
    }

    #[test]
    fn parse_artifact_report_json_command() {
        let command = parse_args(
            vec![
                "artifact-report".to_owned(),
                "--json".to_owned(),
                "nuis.compiled.artifact".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::ArtifactReport {
                input: PathBuf::from("nuis.compiled.artifact"),
                json: true,
                summary: false,
            }
        );
    }

    #[test]
    fn parse_artifact_report_summary_command() {
        let command = parse_args(
            vec![
                "artifact-report".to_owned(),
                "--summary".to_owned(),
                "nuis.compiled.artifact".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::ArtifactReport {
                input: PathBuf::from("nuis.compiled.artifact"),
                json: false,
                summary: true,
            }
        );
    }

    #[test]
    fn parse_inspect_artifact_json_command() {
        let command = parse_args(
            vec![
                "inspect-artifact".to_owned(),
                "--json".to_owned(),
                "nuis.compiled.artifact".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::InspectArtifact {
                input: PathBuf::from("nuis.compiled.artifact"),
                json: true,
            }
        );
    }

    #[test]
    fn parse_verify_artifact_json_command() {
        let command = parse_args(
            vec![
                "verify-artifact".to_owned(),
                "--json".to_owned(),
                "nuis.compiled.artifact".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::VerifyArtifact {
                input: PathBuf::from("nuis.compiled.artifact"),
                json: true,
            }
        );
    }

    #[test]
    fn parse_verify_build_manifest_json_command() {
        let command = parse_args(
            vec![
                "verify-build-manifest".to_owned(),
                "--json".to_owned(),
                "nuis.build.manifest.toml".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::VerifyBuildManifest {
                manifest: PathBuf::from("nuis.build.manifest.toml"),
                json: true,
            }
        );
    }

    #[test]
    fn parse_inspect_benchmarks_command() {
        let command =
            parse_args(vec!["inspect-benchmarks".to_owned(), "main.ns".to_owned()].into_iter())
                .unwrap();
        assert_eq!(
            command,
            CommandKind::InspectBenchmarks {
                input: PathBuf::from("main.ns"),
                json: false,
            }
        );
    }

    #[test]
    fn parse_inspect_benchmarks_json_command() {
        let command = parse_args(
            vec![
                "inspect-benchmarks".to_owned(),
                "--json".to_owned(),
                "main.ns".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::InspectBenchmarks {
                input: PathBuf::from("main.ns"),
                json: true,
            }
        );
    }

    #[test]
    fn parse_inspect_docs_command() {
        let command =
            parse_args(vec!["inspect-docs".to_owned(), "main.ns".to_owned()].into_iter()).unwrap();
        assert_eq!(
            command,
            CommandKind::InspectDocs {
                input: PathBuf::from("main.ns"),
                json: false,
                output: None,
            }
        );
    }

    #[test]
    fn parse_inspect_docs_json_command() {
        let command = parse_args(
            vec![
                "inspect-docs".to_owned(),
                "--json".to_owned(),
                "nuis.toml".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::InspectDocs {
                input: PathBuf::from("nuis.toml"),
                json: true,
                output: None,
            }
        );
    }

    #[test]
    fn parse_inspect_docs_json_output_command() {
        let command = parse_args(
            vec![
                "inspect-docs".to_owned(),
                "--json".to_owned(),
                "--output".to_owned(),
                "docs.json".to_owned(),
                "nuis.toml".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::InspectDocs {
                input: PathBuf::from("nuis.toml"),
                json: true,
                output: Some(PathBuf::from("docs.json")),
            }
        );
    }

    #[test]
    fn parse_inspect_galaxy_docs_command() {
        let command =
            parse_args(vec!["inspect-galaxy-docs".to_owned(), "pixelmagic".to_owned()].into_iter())
                .unwrap();
        assert_eq!(
            command,
            CommandKind::InspectGalaxyDocs {
                galaxy: "pixelmagic".to_owned(),
                json: false,
            }
        );
    }

    #[test]
    fn parse_inspect_galaxy_docs_json_command() {
        let command = parse_args(
            vec![
                "inspect-galaxy-docs".to_owned(),
                "--json".to_owned(),
                "pixelmagic".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::InspectGalaxyDocs {
                galaxy: "pixelmagic".to_owned(),
                json: true,
            }
        );
    }

    #[test]
    fn parse_inspect_stdlib_docs_command() {
        let command = parse_args(vec!["inspect-stdlib-docs".to_owned()].into_iter()).unwrap();
        assert_eq!(command, CommandKind::InspectStdlibDocs { json: false });
    }

    #[test]
    fn parse_inspect_stdlib_docs_json_command() {
        let command =
            parse_args(vec!["inspect-stdlib-docs".to_owned(), "--json".to_owned()].into_iter())
                .unwrap();
        assert_eq!(command, CommandKind::InspectStdlibDocs { json: true });
    }

    #[test]
    fn parse_inspect_project_metadata_command() {
        let command = parse_args(
            vec![
                "inspect-project-metadata".to_owned(),
                "examples/projects/tooling/benchmark_report_file_demo".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::InspectProjectMetadata {
                input: PathBuf::from("examples/projects/tooling/benchmark_report_file_demo"),
                json: false,
                summary: false,
                paths_only: false,
            }
        );
    }

    #[test]
    fn parse_inspect_project_metadata_json_command() {
        let command = parse_args(
            vec![
                "inspect-project-metadata".to_owned(),
                "--json".to_owned(),
                "build/nuis.build.manifest.toml".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::InspectProjectMetadata {
                input: PathBuf::from("build/nuis.build.manifest.toml"),
                json: true,
                summary: false,
                paths_only: false,
            }
        );
    }

    #[test]
    fn parse_inspect_project_metadata_summary_command() {
        let command = parse_args(
            vec![
                "inspect-project-metadata".to_owned(),
                "--summary".to_owned(),
                "examples/projects/tooling/benchmark_report_file_demo".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::InspectProjectMetadata {
                input: PathBuf::from("examples/projects/tooling/benchmark_report_file_demo"),
                json: false,
                summary: true,
                paths_only: false,
            }
        );
    }

    #[test]
    fn parse_inspect_project_metadata_paths_only_command() {
        let command = parse_args(
            vec![
                "inspect-project-metadata".to_owned(),
                "--paths-only".to_owned(),
                "build/nuis.build.manifest.toml".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::InspectProjectMetadata {
                input: PathBuf::from("build/nuis.build.manifest.toml"),
                json: false,
                summary: false,
                paths_only: true,
            }
        );
    }

    #[test]
    fn parse_repair_project_metadata_command() {
        let command = parse_args(
            vec![
                "repair-project-metadata".to_owned(),
                "build/nuis.build.manifest.toml".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::RepairProjectMetadata {
                input: PathBuf::from("build/nuis.build.manifest.toml"),
                dry_run: false,
            }
        );
    }

    #[test]
    fn parse_repair_project_metadata_dry_run_command() {
        let command = parse_args(
            vec![
                "repair-project-metadata".to_owned(),
                "--dry-run".to_owned(),
                "build/nuis.build.manifest.toml".to_owned(),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(
            command,
            CommandKind::RepairProjectMetadata {
                input: PathBuf::from("build/nuis.build.manifest.toml"),
                dry_run: true,
            }
        );
    }
}
