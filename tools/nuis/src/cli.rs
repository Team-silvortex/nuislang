use std::path::PathBuf;

mod galaxy;
mod support;

use galaxy::parse_galaxy_args;
pub use galaxy::GalaxyCommand;
use support::{
    parse_bench_args, parse_build_args, parse_cache_status_args, parse_clean_cache_args,
    parse_optional_json_input, parse_prune_cache_args, parse_release_check_args,
    parse_required_json_input, parse_required_json_input_output, parse_test_args,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind {
    Help,
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
    InspectArtifact {
        input: PathBuf,
        json: bool,
    },
    VerifyArtifact {
        input: PathBuf,
        json: bool,
    },
    UnpackArtifactSupport {
        input: PathBuf,
        output_dir: PathBuf,
        json: bool,
    },
    MaterializeArtifact {
        input: PathBuf,
        output_dir: PathBuf,
        json: bool,
    },
    ArtifactDoctor {
        input: PathBuf,
        json: bool,
    },
    BuildReport {
        input: PathBuf,
        json: bool,
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
    Bench {
        input: PathBuf,
        list: bool,
        json: bool,
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
    RunArtifact {
        input: PathBuf,
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
    ProjectImports {
        input: PathBuf,
        json: bool,
        apply_suggested: bool,
    },
    ProjectLockAbi {
        input: PathBuf,
    },
    Galaxy(GalaxyCommand),
}

pub fn parse_args<I>(mut args: I) -> Result<CommandKind, String>
where
    I: Iterator<Item = String>,
{
    let command = args.next().unwrap_or_else(|| "status".to_owned());
    match command.as_str() {
        "help" | "--help" | "-h" => Ok(CommandKind::Help),
        "status" => Ok(CommandKind::Status),
        "registry" => {
            let mut json = false;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else {
                    return Err("usage: nuis registry [--json]".to_owned());
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
        "inspect-artifact" => {
            let (input, json) = parse_required_json_input(
                &mut args,
                "usage: nuis inspect-artifact [--json] <output-dir|nuis.compiled.artifact|nuis.build.manifest.toml>",
            )?;
            Ok(CommandKind::InspectArtifact { input, json })
        }
        "verify-artifact" => {
            let (input, json) = parse_required_json_input(
                &mut args,
                "usage: nuis verify-artifact [--json] <output-dir|nuis.compiled.artifact>",
            )?;
            Ok(CommandKind::VerifyArtifact { input, json })
        }
        "unpack-artifact-support" => {
            let (input, output_dir, json) = parse_required_json_input_output(
                &mut args,
                "usage: nuis unpack-artifact-support [--json] <output-dir|nuis.compiled.artifact|nuis.build.manifest.toml> <output-dir>",
            )?;
            Ok(CommandKind::UnpackArtifactSupport {
                input,
                output_dir,
                json,
            })
        }
        "materialize-artifact" => {
            let (input, output_dir, json) = parse_required_json_input_output(
                &mut args,
                "usage: nuis materialize-artifact [--json] <output-dir|nuis.compiled.artifact|nuis.build.manifest.toml> <output-dir>",
            )?;
            Ok(CommandKind::MaterializeArtifact {
                input,
                output_dir,
                json,
            })
        }
        "artifact-doctor" => {
            let (input, json) = parse_required_json_input(
                &mut args,
                "usage: nuis artifact-doctor [--json] <output-dir|binary-path|nuis.compiled.artifact|nuis.build.manifest.toml>",
            )?;
            Ok(CommandKind::ArtifactDoctor { input, json })
        }
        "build-report" => {
            let (input, json) = parse_required_json_input(
                &mut args,
                "usage: nuis build-report [--json] <output-dir|binary-path|nuis.compiled.artifact|nuis.build.manifest.toml>",
            )?;
            Ok(CommandKind::BuildReport { input, json })
        }
        "verify-build-manifest" => Ok(CommandKind::VerifyBuildManifest {
            manifest: PathBuf::from(args.next().ok_or_else(|| {
                "usage: nuis verify-build-manifest <output-dir|nuis.build.manifest.toml>"
                    .to_owned()
            })?),
        }),
        "cache-status" => {
            let parsed = parse_cache_status_args(&mut args)?;
            Ok(CommandKind::CacheStatus {
                input: parsed.input,
                all: parsed.all,
                verbose_cache: parsed.verbose_cache,
                json: parsed.json,
            })
        }
        "clean-cache" => {
            let parsed = parse_clean_cache_args(&mut args)?;
            Ok(CommandKind::CleanCache {
                input: parsed.input,
                all: parsed.all,
                json: parsed.json,
            })
        }
        "cache-prune" => {
            let parsed = parse_prune_cache_args(&mut args)?;
            Ok(CommandKind::PruneCache {
                input: parsed.input,
                all: parsed.all,
                keep: parsed.keep,
                json: parsed.json,
            })
        }
        "release-check" => {
            let parsed = parse_release_check_args(&mut args)?;
            Ok(CommandKind::ReleaseCheck {
                input: parsed.input,
                output_dir: parsed.output_dir,
                cpu_abi: parsed.cpu_abi,
                target: parsed.target,
            })
        }
        "check" => Ok(CommandKind::Check {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        }),
        "test" => {
            let parsed = parse_test_args(&mut args)?;
            Ok(CommandKind::Test {
                input: parsed.input,
                list: parsed.list,
                ignored_only: parsed.ignored_only,
                include_ignored: parsed.include_ignored,
                exact: parsed.exact,
                filter: parsed.filter,
            })
        }
        "bench" => {
            let parsed = parse_bench_args(&mut args)?;
            Ok(CommandKind::Bench {
                input: parsed.input,
                list: parsed.list,
                json: parsed.json,
                exact: parsed.exact,
                filter: parsed.filter,
            })
        }
        "build" => {
            let parsed = parse_build_args(&mut args)?;
            Ok(CommandKind::Build {
                input: parsed.input,
                output_dir: parsed.output_dir,
                verbose_cache: parsed.verbose_cache,
                cpu_abi: parsed.cpu_abi,
                target: parsed.target,
            })
        }
        "run-artifact" => {
            let (input, json) = parse_required_json_input(
                &mut args,
                "usage: nuis run-artifact [--json] <output-dir|binary-path|nuis.compiled.artifact|nuis.build.manifest.toml>",
            )?;
            Ok(CommandKind::RunArtifact { input, json })
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
            let (input, json) = parse_optional_json_input(&mut args, "usage: nuis workflow [--json] [input.ns|project-dir|nuis.toml]")?;
            Ok(CommandKind::Workflow { input, json })
        }
        "scheduler-view" => {
            let (input, json) = parse_optional_json_input(&mut args, "usage: nuis scheduler-view [--json] [input.ns|project-dir|nuis.toml]")?;
            Ok(CommandKind::SchedulerView { input, json })
        }
        "rc" => Ok(CommandKind::Rc {
            args: args.collect::<Vec<_>>(),
        }),
        "project-status" => {
            let (input, json) = parse_optional_json_input(&mut args, "usage: nuis project-status [--json] [project-dir|nuis.toml]")?;
            Ok(CommandKind::ProjectStatus { input, json })
        }
        "project-doctor" => {
            let (input, json) = parse_optional_json_input(&mut args, "usage: nuis project-doctor [--json] [project-dir|nuis.toml]")?;
            Ok(CommandKind::ProjectDoctor { input, json })
        }
        "project-imports" => {
            let mut json = false;
            let mut apply_suggested = false;
            let mut input = None;
            for arg in args.by_ref() {
                if arg == "--json" {
                    json = true;
                } else if arg == "--apply-suggested" {
                    apply_suggested = true;
                } else if input.is_none() {
                    input = Some(PathBuf::from(arg));
                } else {
                    return Err(
                        "usage: nuis project-imports [--json] [--apply-suggested] [project-dir|nuis.toml]"
                            .to_owned(),
                    );
                }
            }
            Ok(CommandKind::ProjectImports {
                input: input.unwrap_or_else(|| PathBuf::from(".")),
                json,
                apply_suggested,
            })
        }
        "project-lock-abi" => Ok(CommandKind::ProjectLockAbi {
            input: PathBuf::from(args.next().unwrap_or_else(|| ".".to_owned())),
        }),
        "galaxy" => parse_galaxy_args(args),
        other => Err(format!(
            "unknown nuis command `{other}`; expected `help`, `status`, `registry`, `fmt`, `bindings`, `pack-nustar`, `inspect-nustar`, `loader-contract`, `inspect-artifact`, `verify-artifact`, `unpack-artifact-support`, `materialize-artifact`, `artifact-doctor`, `build-report`, `verify-build-manifest`, `cache-status`, `clean-cache`, `cache-prune`, `release-check`, `check`, `test`, `build`, `run-artifact`, `dump-ast`, `dump-nir`, `dump-yir`, `workflow`, `scheduler-view`, `rc`, `project-status`, `project-doctor`, `project-imports`, `project-lock-abi`, or `galaxy`"
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
#[path = "cli_tests.rs"]
mod tests;
