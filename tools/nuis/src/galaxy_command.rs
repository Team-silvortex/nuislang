use crate::{cli, galaxy, yes_no};

pub(crate) fn handle_galaxy(command: cli::GalaxyCommand) -> Result<(), String> {
    match command {
        cli::GalaxyCommand::Init { input, framework } => {
            let manifest_path = galaxy::init(&input, framework.as_deref())?;
            println!("initialized galaxy package");
            println!("  manifest: {}", manifest_path.display());
            if let Some(framework) = framework {
                println!("  framework: {}", framework);
            }
            println!("  local_index: {}", galaxy::local_index_root().display());
        }
        cli::GalaxyCommand::Check { input } => {
            let checked = galaxy::check(&input)?;
            println!("checked galaxy package: {}", checked.manifest.name);
            println!("  root: {}", checked.root.display());
            println!("  manifest: {}", checked.manifest_path.display());
            println!("  project_plan: {}", checked.project_plan_summary);
            println!("  version: {}", checked.manifest.version);
            println!("  package_kind: {}", checked.manifest.package_kind);
            if let Some(framework) = &checked.manifest.framework {
                println!("  framework: {}", framework);
            }
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
            if let Some(framework) = &inspected.manifest.framework {
                println!("  framework: {}", framework);
            }
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
        cli::GalaxyCommand::InstallDeps { input } => {
            let installed = galaxy::install_project_deps(&input)?;
            if installed.installed.is_empty() {
                println!("project has no galaxy dependencies");
                println!("  project_root: {}", installed.project_root.display());
                println!("  project_plan: {}", installed.project_plan_summary);
                println!("  lock: {}", installed.lock.path.display());
            } else {
                println!("installed galaxy dependencies");
                println!("  project_root: {}", installed.project_root.display());
                println!("  project_plan: {}", installed.project_plan_summary);
                for item in installed.installed {
                    println!("  dep: {}={}", item.name, item.version);
                    println!("  output: {}", item.output.display());
                    println!("  project: {}", item.project.display());
                    println!("  bundle: {}", item.bundle.display());
                    println!("  bundle_fnv1a64: {}", item.bundle_fnv1a64);
                }
                println!("  lock: {}", installed.lock.path.display());
            }
        }
        cli::GalaxyCommand::Doctor { input } => {
            let report = galaxy::doctor_project(&input)?;
            println!("galaxy doctor");
            println!("  project_root: {}", report.project_root.display());
            println!("  project_plan: {}", report.project_plan_summary);
            println!("  deps_root: {}", report.deps_root.display());
            println!(
                "  local_registry_root: {}",
                report.local_registry_root.display()
            );
            println!("  lock_path: {}", report.lock_path.display());
            println!("  lock_status: {}", report.lock_status);
            if let Some(error) = report.lock_error {
                println!("  lock_error: {}", error);
            }
            println!("  dependencies: {}", report.dependencies.len());
            for item in report.dependencies {
                println!(
                    "  dep: {}={} local={} lock={} installed={}",
                    item.name,
                    item.version,
                    yes_no(item.local_available),
                    yes_no(item.locked),
                    yes_no(item.installed)
                );
            }
        }
        cli::GalaxyCommand::SyncDeps { input } => {
            let synced = galaxy::sync_project_deps(&input)?;
            if synced.entries.is_empty() {
                println!("galaxy lock has no dependencies");
                println!("  project_root: {}", synced.project_root.display());
                println!("  project_plan: {}", synced.project_plan_summary);
                println!("  root: {}", synced.root.display());
            } else {
                println!("synced galaxy dependencies");
                println!("  project_root: {}", synced.project_root.display());
                println!("  project_plan: {}", synced.project_plan_summary);
                println!("  root: {}", synced.root.display());
                println!("  dependencies: {}", synced.entries.len());
                for entry in synced.entries {
                    println!("  dep: {}={}", entry.name, entry.version);
                    println!("  bundle: {}", entry.bundle.display());
                    println!("  bundle_fnv1a64: {}", entry.bundle_fnv1a64);
                }
            }
        }
        cli::GalaxyCommand::LockDeps { input } => {
            let lock = galaxy::lock_project_deps(&input)?;
            println!("locked galaxy dependencies");
            println!("  project_root: {}", lock.project_root.display());
            println!("  project_plan: {}", lock.project_plan_summary);
            println!("  lock: {}", lock.path.display());
            println!("  dependencies: {}", lock.entries.len());
            for entry in lock.entries {
                println!("  dep: {}={}", entry.name, entry.version);
                println!("  bundle: {}", entry.bundle.display());
                println!("  bundle_fnv1a64: {}", entry.bundle_fnv1a64);
            }
        }
        cli::GalaxyCommand::VerifyLock { input } => {
            let lock = galaxy::verify_project_lock(&input)?;
            println!("verified galaxy lock");
            println!("  project_root: {}", lock.project_root.display());
            println!("  project_plan: {}", lock.project_plan_summary);
            println!("  lock: {}", lock.path.display());
            println!("  dependencies: {}", lock.entries.len());
            for entry in lock.entries {
                println!("  dep: {}={}", entry.name, entry.version);
                println!("  bundle: {}", entry.bundle.display());
                println!("  bundle_fnv1a64: {}", entry.bundle_fnv1a64);
            }
        }
        cli::GalaxyCommand::InspectLocal { name, version } => {
            let inspected = galaxy::inspect_local(&name, version.as_deref())?;
            println!("inspected local galaxy package");
            println!("  name: {}", inspected.manifest.name);
            println!("  version: {}", inspected.manifest.version);
            println!("  package_kind: {}", inspected.manifest.package_kind);
            if let Some(framework) = &inspected.manifest.framework {
                println!("  framework: {}", framework);
            }
            println!("  project: {}", inspected.manifest.project);
            println!("  summary: {}", inspected.manifest.summary);
            println!("  entries: {}", inspected.entries.len());
            for entry in inspected.entries {
                println!("  file: {} ({} bytes)", entry.path, entry.bytes);
            }
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
        cli::GalaxyCommand::RemoveLocal { name, version } => {
            let removed = galaxy::remove_local(&name, version.as_deref())?;
            println!("removed local galaxy package");
            println!("  name: {}", removed.name);
            println!("  version: {}", removed.version);
            println!("  bundle: {}", removed.package.display());
            println!("  index_entry: {}", removed.index_entry.display());
        }
    }
    Ok(())
}
