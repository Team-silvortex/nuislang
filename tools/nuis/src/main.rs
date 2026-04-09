mod cli;
mod galaxy;

use std::{collections::BTreeSet, fs};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    match cli::parse_args(std::env::args().skip(1))? {
        cli::CommandKind::Help => {
            print_help();
        }
        cli::CommandKind::Status => {
            let index = nuisc::registry::load_index(std::path::Path::new("nustar-packages"))?;
            let engine = nuisc::engine::default_engine();
            println!("nuis toolchain frontdoor");
            println!("  tool: nuis");
            println!("  compiler_core: nuisc");
            println!("  resident_control: nuis-rc");
            println!("  profile: {}", engine.profile);
            println!("  yir: {}", engine.version);
            println!("  indexed_nustar: {}", index.len());
            println!("  nustar_loading: lazy");
            println!("  external_projects: yalivia, vulpoya");
        }
        cli::CommandKind::Registry => {
            nuisc::run(nuisc::CommandKind::Registry)?;
        }
        cli::CommandKind::Fmt { input } => {
            nuisc::run(nuisc::CommandKind::Fmt { input })?;
        }
        cli::CommandKind::Bindings { input } => {
            nuisc::run(nuisc::CommandKind::Bindings { input })?;
        }
        cli::CommandKind::PackNustar { package_id, output } => {
            nuisc::run(nuisc::CommandKind::PackNustar { package_id, output })?;
        }
        cli::CommandKind::InspectNustar { input } => {
            nuisc::run(nuisc::CommandKind::InspectNustar { input })?;
        }
        cli::CommandKind::LoaderContract { package_id } => {
            nuisc::run(nuisc::CommandKind::LoaderContract { package_id })?;
        }
        cli::CommandKind::VerifyBuildManifest { manifest } => {
            nuisc::run(nuisc::CommandKind::VerifyBuildManifest { manifest })?;
        }
        cli::CommandKind::ReleaseCheck { input, output_dir } => {
            println!("release-check: check");
            nuisc::run(nuisc::CommandKind::Check {
                input: input.clone(),
            })?;
            println!("release-check: build");
            nuisc::run(nuisc::CommandKind::Compile {
                input: input.clone(),
                output_dir: output_dir.clone(),
            })?;
            println!("release-check: verify-build-manifest");
            let manifest = output_dir.join("nuis.build.manifest.toml");
            nuisc::run(nuisc::CommandKind::VerifyBuildManifest {
                manifest: manifest.clone(),
            })?;
            println!("release-check: ok");
            println!("  output_dir: {}", output_dir.display());
            println!("  manifest: {}", manifest.display());
        }
        cli::CommandKind::Check { input } => {
            nuisc::run(nuisc::CommandKind::Check { input })?;
        }
        cli::CommandKind::Build { input, output_dir } => {
            nuisc::run(nuisc::CommandKind::Compile { input, output_dir })?;
        }
        cli::CommandKind::DumpAst { input } => {
            nuisc::run(nuisc::CommandKind::DumpAst { input })?;
        }
        cli::CommandKind::DumpNir { input } => {
            nuisc::run(nuisc::CommandKind::DumpNir { input })?;
        }
        cli::CommandKind::DumpYir { input } => {
            nuisc::run(nuisc::CommandKind::DumpYir { input })?;
        }
        cli::CommandKind::Rc { args } => {
            run_nuis_rc(&args)?;
        }
        cli::CommandKind::ProjectStatus { input } => {
            let project = nuisc::project::load_project(&input)?;
            let resolution = nuisc::project::resolve_project_abi(&project)?;
            let mut domains = project
                .modules
                .iter()
                .map(|module| module.ast.domain.clone())
                .collect::<BTreeSet<_>>();
            for link in &project.manifest.links {
                if let Some((domain, _)) = link.from.split_once('.') {
                    domains.insert(domain.to_owned());
                }
                if let Some((domain, _)) = link.to.split_once('.') {
                    domains.insert(domain.to_owned());
                }
                if let Some(via) = &link.via {
                    if let Some((domain, _)) = via.split_once('.') {
                        domains.insert(domain.to_owned());
                    }
                }
            }
            println!("project status: {}", project.manifest.name);
            println!("  root: {}", project.root.display());
            println!("  manifest: {}", project.manifest_path.display());
            println!("  entry: {}", project.manifest.entry);
            println!("  modules: {}", project.modules.len());
            println!("  links: {}", project.manifest.links.len());
            println!(
                "  domains: {}",
                domains.into_iter().collect::<Vec<_>>().join(", ")
            );
            println!(
                "  abi_mode: {}",
                if resolution.explicit {
                    "explicit"
                } else {
                    "auto-recommended"
                }
            );
            for item in resolution.requirements {
                println!("  abi: {}={}", item.domain, item.abi);
            }
        }
        cli::CommandKind::ProjectLockAbi { input } => {
            let project = nuisc::project::load_project(&input)?;
            let resolution = nuisc::project::resolve_project_abi(&project)?;
            let manifest_source = fs::read_to_string(&project.manifest_path).map_err(|error| {
                format!(
                    "failed to read `{}`: {error}",
                    project.manifest_path.display()
                )
            })?;
            let updated = upsert_abi_block(&manifest_source, &resolution.requirements);
            if updated == manifest_source {
                println!(
                    "project abi already locked: {}",
                    project.manifest_path.display()
                );
            } else {
                fs::write(&project.manifest_path, updated).map_err(|error| {
                    format!(
                        "failed to write `{}`: {error}",
                        project.manifest_path.display()
                    )
                })?;
                println!("locked project abi: {}", project.manifest_path.display());
            }
            println!(
                "  mode: {}",
                if resolution.explicit {
                    "explicit (normalized)"
                } else {
                    "auto -> explicit"
                }
            );
            for item in resolution.requirements {
                println!("  abi: {}={}", item.domain, item.abi);
            }
        }
        cli::CommandKind::Galaxy(command) => match command {
            cli::GalaxyCommand::Init { input } => {
                let manifest_path = galaxy::init(&input)?;
                println!("initialized galaxy package");
                println!("  manifest: {}", manifest_path.display());
                println!("  local_index: {}", galaxy::local_index_root().display());
            }
            cli::GalaxyCommand::Check { input } => {
                let checked = galaxy::check(&input)?;
                println!("checked galaxy package: {}", checked.manifest.name);
                println!("  root: {}", checked.root.display());
                println!("  manifest: {}", checked.manifest_path.display());
                println!("  version: {}", checked.manifest.version);
                println!("  package_kind: {}", checked.manifest.package_kind);
                println!("  project: {}", checked.manifest.project);
                println!("  include_files: {}", checked.include_files.len());
                println!("  local_index: {}", galaxy::local_index_root().display());
                for (domain, abi) in checked.abi_entries {
                    println!("  abi: {}={}", domain, abi);
                }
            }
            cli::GalaxyCommand::Pack { input, output } => {
                let bundle = galaxy::pack(&input, &output)?;
                println!("packed galaxy bundle");
                println!("  bundle: {}", bundle.display());
                println!("  local_index: {}", galaxy::local_index_root().display());
                println!(
                    "  local_packages: {}",
                    galaxy::local_packages_root().display()
                );
            }
            cli::GalaxyCommand::Inspect { input } => {
                let inspected = galaxy::inspect_bundle(&input)?;
                println!("inspected galaxy bundle: {}", input.display());
                println!("  name: {}", inspected.manifest.name);
                println!("  version: {}", inspected.manifest.version);
                println!("  package_kind: {}", inspected.manifest.package_kind);
                println!("  project: {}", inspected.manifest.project);
                println!("  summary: {}", inspected.manifest.summary);
                println!("  entries: {}", inspected.entries.len());
                for entry in inspected.entries {
                    println!("  file: {} ({} bytes)", entry.path, entry.bytes);
                }
            }
            cli::GalaxyCommand::PublishLocal { input, output } => {
                let bundle = galaxy::publish_local(&input, output.as_deref())?;
                println!("published galaxy bundle locally");
                println!("  bundle: {}", bundle.display());
                println!("  local_index: {}", galaxy::local_index_root().display());
                println!(
                    "  local_packages: {}",
                    galaxy::local_packages_root().display()
                );
            }
            cli::GalaxyCommand::List => {
                let entries = galaxy::list_local()?;
                if entries.is_empty() {
                    println!("no local galaxy packages");
                } else {
                    for entry in entries {
                        println!("package: {}", entry.name);
                        println!("  version: {}", entry.version);
                        println!("  bundle: {}", entry.package);
                        println!("  project: {}", entry.project);
                        if let Some(bytes) = entry.bundle_bytes {
                            println!("  bundle_bytes: {}", bytes);
                        }
                        if let Some(hash) = &entry.bundle_fnv1a64 {
                            println!("  bundle_fnv1a64: {}", hash);
                        }
                        if !entry.abi.is_empty() {
                            println!("  abi: {}", entry.abi.join(", "));
                        }
                    }
                }
            }
            cli::GalaxyCommand::InstallLocal {
                name,
                version,
                output,
            } => {
                let project_path = galaxy::install_local(&name, version.as_deref(), &output)?;
                println!("installed local galaxy package");
                println!("  name: {}", name);
                if let Some(version) = version {
                    println!("  version: {}", version);
                }
                println!("  output: {}", output.display());
                println!("  project: {}", project_path.display());
            }
            cli::GalaxyCommand::VerifyLocal { name, version } => {
                let verified = galaxy::verify_local(&name, version.as_deref())?;
                println!("verified local galaxy package");
                println!("  name: {}", verified.name);
                println!("  version: {}", verified.version);
                println!("  bundle: {}", verified.package.display());
                println!("  bundle_bytes: {}", verified.bundle_bytes);
                println!("  bundle_fnv1a64: {}", verified.bundle_fnv1a64);
                println!("  entries: {}", verified.entries);
            }
        },
    }

    Ok(())
}

fn print_help() {
    println!("nuis toolchain frontdoor");
    println!("usage:");
    println!("  nuis status");
    println!("  nuis registry");
    println!("  nuis fmt [input.ns|project-dir|nuis.toml]");
    println!("  nuis bindings <input.ns|project-dir|nuis.toml>");
    println!("  nuis check [input.ns|project-dir|nuis.toml]");
    println!("  nuis build [input.ns|project-dir|nuis.toml] <output-dir>");
    println!("  nuis dump-ast [input.ns|project-dir|nuis.toml]");
    println!("  nuis dump-nir [input.ns|project-dir|nuis.toml]");
    println!("  nuis dump-yir [input.ns|project-dir|nuis.toml]");
    println!("  nuis pack-nustar <package-id> <output.nustar>");
    println!("  nuis inspect-nustar <input.nustar>");
    println!("  nuis loader-contract <package-id>");
    println!("  nuis verify-build-manifest <nuis.build.manifest.toml>");
    println!("  nuis release-check [input.ns|project-dir|nuis.toml] [output-dir]");
    println!("  nuis rc <status|start|stop|track|projects|versions> [...]");
    println!("  nuis project-status [project-dir|nuis.toml]");
    println!("  nuis project-lock-abi [project-dir|nuis.toml]");
    println!("  nuis galaxy init [project-dir]");
    println!("  nuis galaxy check [project-dir|galaxy.toml]");
    println!("  nuis galaxy pack [project-dir|galaxy.toml] [output.galaxy]");
    println!("  nuis galaxy inspect <input.galaxy>");
    println!("  nuis galaxy publish-local [project-dir|galaxy.toml] [output.galaxy]");
    println!("  nuis galaxy list");
    println!("  nuis galaxy install-local <name> [version] [output-dir]");
    println!("  nuis galaxy verify-local <name> [version]");
}

fn run_nuis_rc(args: &[String]) -> Result<(), String> {
    let status = std::process::Command::new("nuis-rc").args(args).status();
    match status {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(format!(
                    "nuis-rc exited with status {}",
                    status
                        .code()
                        .map(|code| code.to_string())
                        .unwrap_or_else(|| "signal".to_owned())
                ))
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let fallback = std::process::Command::new("cargo")
                .args(["run", "-q", "-p", "nuis-rc", "--"])
                .args(args)
                .status();
            match fallback {
                Ok(status) if status.success() => Ok(()),
                Ok(status) => Err(format!(
                    "failed to run nuis-rc via PATH and cargo fallback exited with status {}",
                    status
                        .code()
                        .map(|code| code.to_string())
                        .unwrap_or_else(|| "signal".to_owned())
                )),
                Err(fallback_error) => Err(format!(
                    "failed to run nuis-rc via PATH ({error}) and cargo fallback ({fallback_error})"
                )),
            }
        }
        Err(error) => Err(format!("failed to run nuis-rc: {error}")),
    }
}

fn upsert_abi_block(
    source: &str,
    requirements: &[nuisc::project::ProjectAbiRequirement],
) -> String {
    let mut entries = requirements
        .iter()
        .map(|item| (item.domain.clone(), item.abi.clone()))
        .collect::<Vec<_>>();
    entries.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
    let block = render_abi_block(&entries);

    if let Some((start, end)) = find_abi_block_span(source) {
        let mut out = String::new();
        out.push_str(&source[..start]);
        out.push_str(&block);
        out.push_str(&source[end..]);
        out
    } else if source.ends_with('\n') {
        format!("{source}\n{block}")
    } else {
        format!("{source}\n\n{block}")
    }
}

fn render_abi_block(entries: &[(String, String)]) -> String {
    let mut out = String::new();
    out.push_str("abi = [\n");
    for (domain, abi) in entries {
        out.push_str(&format!("  \"{}={}\",\n", domain, abi));
    }
    out.push_str("]\n");
    out
}

fn find_abi_block_span(source: &str) -> Option<(usize, usize)> {
    let mut offset = 0usize;
    let mut start = None::<usize>;
    let mut depth = 0i32;
    let mut seen_open = false;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if start.is_none() && trimmed.starts_with("abi") && trimmed.contains('=') {
            start = Some(offset);
            depth += line.matches('[').count() as i32;
            depth -= line.matches(']').count() as i32;
            seen_open = line.contains('[');
            if seen_open && depth <= 0 {
                return Some((start?, offset + line.len()));
            }
        } else if start.is_some() {
            depth += line.matches('[').count() as i32;
            depth -= line.matches(']').count() as i32;
            if line.contains('[') {
                seen_open = true;
            }
            if seen_open && depth <= 0 {
                return Some((start?, offset + line.len()));
            }
        }
        offset += line.len();
    }
    start.map(|s| (s, source.len()))
}
