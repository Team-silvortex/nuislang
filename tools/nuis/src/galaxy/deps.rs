use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

struct VerifiedProjectLockContext {
    project: nuisc::project::LoadedProject,
    project_plan_summary: String,
    path: PathBuf,
    source: String,
    entries: Vec<GalaxyLockEntry>,
    summary: nuisc::project::ProjectGalaxyResolutionLockSummary,
}

pub fn install_project_deps(input: &Path) -> Result<InstalledProjectDeps, String> {
    let lock = lock_project_deps(input)?;
    let synced = sync_project_deps(input)?;
    let installed = synced
        .entries
        .iter()
        .map(|entry| {
            let output = synced.root.join(&entry.name).join(&entry.version);
            InstalledGalaxyDependency {
                name: entry.name.clone(),
                version: entry.version.clone(),
                package_id: entry.package_id.clone(),
                direct: entry.direct,
                project: output.join("module.toml"),
                output,
                manifest_sha256: entry.manifest_sha256.clone(),
            }
        })
        .collect();
    Ok(InstalledProjectDeps {
        project_root: synced.project_root,
        project_plan_summary: synced.project_plan_summary,
        installed,
        lock,
    })
}

pub fn lock_project_deps(input: &Path) -> Result<WroteGalaxyLock, String> {
    let project = nuisc::project::load_project(input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let path = nuisc::project::committed_project_galaxy_resolution_lock_path(&project);
    let summary = nuisc::project::write_project_galaxy_resolution_lock(&path, &project)?;
    Ok(WroteGalaxyLock {
        project_root: project.root.clone(),
        project_plan_summary: nuisc::project::describe_project_compilation_plan(&plan),
        path,
        entries: resolution_entries(&project),
        summary,
    })
}

pub fn verify_project_lock(input: &Path) -> Result<VerifiedGalaxyLock, String> {
    let context = load_verified_project_lock(input)?;
    Ok(VerifiedGalaxyLock {
        project_root: context.project.root,
        project_plan_summary: context.project_plan_summary,
        path: context.path,
        entries: context.entries,
        summary: context.summary,
    })
}

pub fn sync_project_deps(input: &Path) -> Result<SyncedProjectDeps, String> {
    let context = load_verified_project_lock(input)?;
    let deps_parent = context.project.root.join(".nuis").join("deps");
    fs::create_dir_all(&deps_parent)
        .map_err(|error| format!("failed to create `{}`: {error}", deps_parent.display()))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock is before Unix epoch: {error}"))?
        .as_nanos();
    let suffix = format!("{}-{nonce}", std::process::id());
    let stage = deps_parent.join(format!(".galaxy.syncing-{suffix}"));
    let backup = deps_parent.join(format!(".galaxy.previous-{suffix}"));
    let deps_root = deps_parent.join("galaxy");
    fs::create_dir(&stage)
        .map_err(|error| format!("failed to create sync stage `{}`: {error}", stage.display()))?;

    let stage_root = nuisc::project::project_galaxy_cache_root_under(&stage, &context.summary)?;
    if let Err(error) = nuisc::project::materialize_project_galaxy_cache(
        &context.project,
        &context.source,
        &context.path,
        &stage_root,
    ) {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }
    replace_materialized_tree(&stage, &deps_root, &backup)?;
    let cache_root = nuisc::project::project_galaxy_cache_root_under(&deps_root, &context.summary)?;

    Ok(SyncedProjectDeps {
        project_root: context.project.root,
        project_plan_summary: context.project_plan_summary,
        root: cache_root,
        entries: context.entries,
        summary: context.summary,
    })
}

pub fn doctor_project(input: &Path) -> Result<GalaxyDoctorReport, String> {
    let project = nuisc::project::load_project(input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let deps_root = project.root.join(".nuis").join("deps").join("galaxy");
    let lock_path = project
        .root
        .join(nuisc::project::PROJECT_GALAXY_RESOLUTION_LOCK_FILE);
    let available = project
        .resolved_galaxies
        .iter()
        .filter(|entry| entry.direct)
        .map(|entry| format!("{}={}", entry.name, entry.version))
        .collect::<BTreeSet<_>>();
    let (lock_status, lock_error, locked, installed) = match verify_project_lock(input) {
        Ok(lock) => {
            let cache_root =
                nuisc::project::project_galaxy_cache_root(&project.root, &lock.summary)?;
            let installed = collect_installed_project_deps(&cache_root)?;
            (
                "ok".to_owned(),
                None,
                lock.entries
                    .into_iter()
                    .filter(|entry| entry.direct)
                    .map(|entry| format!("{}={}", entry.name, entry.version))
                    .collect::<BTreeSet<_>>(),
                installed,
            )
        }
        Err(error) if lock_path.exists() => (
            "invalid".to_owned(),
            Some(error),
            BTreeSet::new(),
            BTreeSet::new(),
        ),
        Err(_) => ("missing".to_owned(), None, BTreeSet::new(), BTreeSet::new()),
    };

    let dependencies = project
        .manifest
        .galaxy_dependencies
        .iter()
        .map(|item| {
            let key = format!("{}={}", item.name, item.version);
            GalaxyDoctorDependency {
                name: item.name.clone(),
                version: item.version.clone(),
                source_available: available.contains(&key),
                locked: locked.contains(&key),
                installed: installed.contains(&key),
            }
        })
        .collect::<Vec<_>>();

    Ok(GalaxyDoctorReport {
        project_root: project.root,
        project_plan_summary: nuisc::project::describe_project_compilation_plan(&plan),
        deps_root,
        local_registry_root: local_root(),
        lock_path,
        lock_status,
        lock_error,
        dependencies,
    })
}

fn load_verified_project_lock(input: &Path) -> Result<VerifiedProjectLockContext, String> {
    let project = nuisc::project::load_project(input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let path = nuisc::project::committed_project_galaxy_resolution_lock_path(&project);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    let summary = nuisc::project::verify_project_galaxy_resolution_lock(&project, &source, &path)?;
    Ok(VerifiedProjectLockContext {
        entries: resolution_entries(&project),
        project,
        project_plan_summary: nuisc::project::describe_project_compilation_plan(&plan),
        path,
        source,
        summary,
    })
}

fn resolution_entries(project: &nuisc::project::LoadedProject) -> Vec<GalaxyLockEntry> {
    let mut entries = project
        .resolved_galaxies
        .iter()
        .map(|dependency| GalaxyLockEntry {
            name: dependency.name.clone(),
            version: dependency.version.clone(),
            package_id: dependency.package_id.clone(),
            direct: dependency.direct,
            manifest_sha256: dependency.manifest_content_identity.sha256.clone(),
            source_modules: dependency.source_modules.len(),
            library_modules: dependency.library_modules.len(),
        })
        .collect::<Vec<_>>();
    entries.sort_by(|lhs, rhs| {
        lhs.name
            .cmp(&rhs.name)
            .then(lhs.version.cmp(&rhs.version))
            .then(lhs.package_id.cmp(&rhs.package_id))
    });
    entries
}

fn replace_materialized_tree(
    stage: &Path,
    destination: &Path,
    backup: &Path,
) -> Result<(), String> {
    if !destination.exists() {
        return fs::rename(stage, destination).map_err(|error| {
            format!(
                "failed to activate Galaxy dependency tree `{}`: {error}",
                destination.display()
            )
        });
    }
    fs::rename(destination, backup).map_err(|error| {
        format!(
            "failed to preserve previous Galaxy dependency tree `{}`: {error}",
            destination.display()
        )
    })?;
    if let Err(error) = fs::rename(stage, destination) {
        let restore = fs::rename(backup, destination);
        return Err(match restore {
            Ok(()) => format!(
                "failed to activate Galaxy dependency tree `{}`; previous tree restored: {error}",
                destination.display()
            ),
            Err(restore_error) => format!(
                "failed to activate Galaxy dependency tree `{}` ({error}) and failed to restore `{}` ({restore_error})",
                destination.display(),
                backup.display()
            ),
        });
    }
    fs::remove_dir_all(backup).map_err(|error| {
        format!(
            "activated Galaxy dependency tree but failed to remove previous tree `{}`: {error}",
            backup.display()
        )
    })
}

pub(super) fn collect_installed_project_deps(root: &Path) -> Result<BTreeSet<String>, String> {
    let mut installed = BTreeSet::new();
    if !root.exists() {
        return Ok(installed);
    }
    for package_dir in fs::read_dir(root)
        .map_err(|error| format!("failed to read `{}`: {error}", root.display()))?
    {
        let package_dir = package_dir
            .map_err(|error| format!("failed to enumerate `{}`: {error}", root.display()))?;
        let package_path = package_dir.path();
        if !package_path.is_dir() {
            continue;
        }
        let Some(name) = package_path.file_name().and_then(|item| item.to_str()) else {
            continue;
        };
        for version_dir in fs::read_dir(&package_path)
            .map_err(|error| format!("failed to read `{}`: {error}", package_path.display()))?
        {
            let version_dir = version_dir.map_err(|error| {
                format!("failed to enumerate `{}`: {error}", package_path.display())
            })?;
            let version_path = version_dir.path();
            if !version_path.is_dir() {
                continue;
            }
            let Some(version) = version_path.file_name().and_then(|item| item.to_str()) else {
                continue;
            };
            if version_path.join("module.toml").exists() {
                installed.insert(format!("{name}={version}"));
            }
        }
    }
    Ok(installed)
}

pub(super) fn remove_dir_if_empty(path: Option<&Path>) -> Result<(), String> {
    let Some(path) = path else {
        return Ok(());
    };
    if !path.exists() {
        return Ok(());
    }
    let mut items = fs::read_dir(path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    if items.next().is_none() {
        fs::remove_dir(path)
            .map_err(|error| format!("failed to remove `{}`: {error}", path.display()))?;
    }
    Ok(())
}
