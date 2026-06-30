use std::path::PathBuf;

use super::CommandKind;

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
