use super::bundle::{decode_bundle, extract_bundle};
use super::local::ensure_local_layout;
use super::*;

pub fn install_project_deps(input: &Path) -> Result<InstalledProjectDeps, String> {
    ensure_local_layout()?;
    let project = nuisc::project::load_project(input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let project_plan_summary = nuisc::project::describe_project_compilation_plan(&plan);
    let deps_root = project.root.join(".nuis").join("deps").join("galaxy");
    fs::create_dir_all(&deps_root)
        .map_err(|error| format!("failed to create `{}`: {error}", deps_root.display()))?;
    let mut installed = Vec::new();
    for dependency in &project.manifest.galaxy_dependencies {
        let resolved = select_local_entry(&dependency.name, Some(&dependency.version))?;
        let output = deps_root.join(&dependency.name).join(&dependency.version);
        fs::create_dir_all(&output)
            .map_err(|error| format!("failed to create `{}`: {error}", output.display()))?;
        let project_path = install_local(&dependency.name, Some(&dependency.version), &output)?;
        installed.push(InstalledGalaxyDependency {
            name: dependency.name.clone(),
            version: dependency.version.clone(),
            output,
            project: project_path,
            bundle: PathBuf::from(&resolved.package),
            bundle_bytes: resolved.bundle_bytes.unwrap_or(0),
            bundle_fnv1a64: resolved.bundle_fnv1a64.unwrap_or_default(),
        });
    }
    let lock = write_project_lock_from_installed(&project.root, &project_plan_summary, &installed)?;
    Ok(InstalledProjectDeps {
        project_root: project.root,
        project_plan_summary,
        installed,
        lock,
    })
}

pub fn lock_project_deps(input: &Path) -> Result<WroteGalaxyLock, String> {
    ensure_local_layout()?;
    let project = nuisc::project::load_project(input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let mut entries = Vec::new();
    for dependency in &project.manifest.galaxy_dependencies {
        let resolved = select_local_entry(&dependency.name, Some(&dependency.version))?;
        let verified = verify_local(&dependency.name, Some(&dependency.version))?;
        entries.push(GalaxyLockEntry {
            name: resolved.name,
            version: resolved.version,
            bundle: verified.package,
            bundle_bytes: verified.bundle_bytes,
            bundle_fnv1a64: verified.bundle_fnv1a64,
        });
    }
    write_project_lock(
        &project.root,
        &nuisc::project::describe_project_compilation_plan(&plan),
        &entries,
    )
}

pub fn verify_project_lock(input: &Path) -> Result<VerifiedGalaxyLock, String> {
    ensure_local_layout()?;
    let project = nuisc::project::load_project(input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let path = project.root.join("nuis.galaxy.lock");
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    let entries = parse_lock_entries(&source, &path)?;
    for entry in &entries {
        let bytes = fs::read(&entry.bundle).map_err(|error| {
            format!(
                "failed to read locked galaxy bundle `{}`: {error}",
                entry.bundle.display()
            )
        })?;
        let actual_bytes = bytes.len() as u64;
        if actual_bytes != entry.bundle_bytes {
            return Err(format!(
                "locked galaxy dependency `{}` version `{}` byte size mismatch: lock={}, actual={}",
                entry.name, entry.version, entry.bundle_bytes, actual_bytes
            ));
        }
        let actual_hash = fnv1a64_hex(&bytes);
        if actual_hash != entry.bundle_fnv1a64 {
            return Err(format!(
                "locked galaxy dependency `{}` version `{}` hash mismatch: lock={}, actual={}",
                entry.name, entry.version, entry.bundle_fnv1a64, actual_hash
            ));
        }
        let inspected = decode_bundle(&bytes, &entry.bundle)?;
        if inspected.manifest.name != entry.name {
            return Err(format!(
                "locked galaxy dependency bundle `{}` resolved to package `{}`, expected `{}`",
                entry.bundle.display(),
                inspected.manifest.name,
                entry.name
            ));
        }
        if inspected.manifest.version != entry.version {
            return Err(format!(
                "locked galaxy dependency bundle `{}` resolved to version `{}`, expected `{}`",
                entry.bundle.display(),
                inspected.manifest.version,
                entry.version
            ));
        }
    }
    Ok(VerifiedGalaxyLock {
        project_root: project.root,
        project_plan_summary: nuisc::project::describe_project_compilation_plan(&plan),
        path,
        entries,
    })
}

pub fn sync_project_deps(input: &Path) -> Result<SyncedProjectDeps, String> {
    ensure_local_layout()?;
    let verified = verify_project_lock(input)?;
    let deps_root = verified
        .project_root
        .join(".nuis")
        .join("deps")
        .join("galaxy");

    if deps_root.exists() {
        fs::remove_dir_all(&deps_root)
            .map_err(|error| format!("failed to reset `{}`: {error}", deps_root.display()))?;
    }
    fs::create_dir_all(&deps_root)
        .map_err(|error| format!("failed to create `{}`: {error}", deps_root.display()))?;

    for entry in &verified.entries {
        let output = deps_root.join(&entry.name).join(&entry.version);
        fs::create_dir_all(&output)
            .map_err(|error| format!("failed to create `{}`: {error}", output.display()))?;
        let bytes = fs::read(&entry.bundle).map_err(|error| {
            format!(
                "failed to read locked galaxy bundle `{}`: {error}",
                entry.bundle.display()
            )
        })?;
        extract_bundle(&bytes, &entry.bundle, &output)?;
    }

    Ok(SyncedProjectDeps {
        project_root: verified.project_root,
        project_plan_summary: verified.project_plan_summary,
        root: deps_root,
        entries: verified.entries,
    })
}

pub fn doctor_project(input: &Path) -> Result<GalaxyDoctorReport, String> {
    ensure_local_layout()?;
    let project = nuisc::project::load_project(input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let deps_root = project.root.join(".nuis").join("deps").join("galaxy");
    let lock_path = project.root.join("nuis.galaxy.lock");
    let local_entries = list_local()?;
    let available = local_entries
        .into_iter()
        .map(|entry| format!("{}={}", entry.name, entry.version))
        .collect::<BTreeSet<_>>();
    let installed = collect_installed_project_deps(&deps_root)?;

    let (lock_status, lock_error, locked) = match verify_project_lock(input) {
        Ok(lock) => (
            "ok".to_owned(),
            None,
            lock.entries
                .into_iter()
                .map(|entry| format!("{}={}", entry.name, entry.version))
                .collect::<BTreeSet<_>>(),
        ),
        Err(error) if lock_path.exists() => ("invalid".to_owned(), Some(error), BTreeSet::new()),
        Err(_) => ("missing".to_owned(), None, BTreeSet::new()),
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
                local_available: available.contains(&key),
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

pub(super) fn write_project_lock_from_installed(
    project_root: &Path,
    project_plan_summary: &str,
    installed: &[InstalledGalaxyDependency],
) -> Result<WroteGalaxyLock, String> {
    let entries = installed
        .iter()
        .map(|item| GalaxyLockEntry {
            name: item.name.clone(),
            version: item.version.clone(),
            bundle: item.bundle.clone(),
            bundle_bytes: item.bundle_bytes,
            bundle_fnv1a64: item.bundle_fnv1a64.clone(),
        })
        .collect::<Vec<_>>();
    write_project_lock(project_root, project_plan_summary, &entries)
}

pub(super) fn write_project_lock(
    project_root: &Path,
    project_plan_summary: &str,
    entries: &[GalaxyLockEntry],
) -> Result<WroteGalaxyLock, String> {
    let path = project_root.join("nuis.galaxy.lock");
    let mut sorted = entries.to_vec();
    sorted.sort_by(|lhs, rhs| {
        lhs.name
            .cmp(&rhs.name)
            .then(compare_version(&lhs.version, &rhs.version))
    });
    let mut source = String::new();
    source.push_str("lock_schema = \"nuis-galaxy-lock-v1\"\n");
    for entry in &sorted {
        source.push_str("\n[[dependency]]\n");
        source.push_str(&format!("name = \"{}\"\n", escape(&entry.name)));
        source.push_str(&format!("version = \"{}\"\n", escape(&entry.version)));
        source.push_str(&format!(
            "bundle = \"{}\"\n",
            escape(&entry.bundle.display().to_string())
        ));
        source.push_str(&format!("bundle_bytes = {}\n", entry.bundle_bytes));
        source.push_str(&format!(
            "bundle_fnv1a64 = \"{}\"\n",
            escape(&entry.bundle_fnv1a64)
        ));
    }
    fs::write(&path, source)
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
    Ok(WroteGalaxyLock {
        project_root: project_root.to_path_buf(),
        project_plan_summary: project_plan_summary.to_owned(),
        path,
        entries: sorted,
    })
}

pub(super) fn parse_lock_entries(
    source: &str,
    path: &Path,
) -> Result<Vec<GalaxyLockEntry>, String> {
    let schema = parse_optional_string(source, "lock_schema").ok_or_else(|| {
        format!(
            "galaxy lock `{}` is missing required key `lock_schema`",
            path.display()
        )
    })?;
    if schema != "nuis-galaxy-lock-v1" {
        return Err(format!(
            "galaxy lock `{}` has unsupported schema `{}`",
            path.display(),
            schema
        ));
    }

    let mut rows = Vec::<Vec<String>>::new();
    let mut current = Vec::<String>::new();
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line == "[[dependency]]" {
            if !current.is_empty() {
                rows.push(current);
                current = Vec::new();
            }
            continue;
        }
        if line.is_empty() {
            continue;
        }
        current.push(line.to_owned());
    }
    if !current.is_empty() {
        rows.push(current);
    }

    let mut entries = Vec::new();
    for row in rows {
        if row.iter().any(|line| line.starts_with("lock_schema = ")) {
            continue;
        }
        let block = row.join("\n");
        let name = parse_required_string(&block, "name", path)?;
        let version = parse_required_string(&block, "version", path)?;
        let bundle = PathBuf::from(parse_required_string(&block, "bundle", path)?);
        validate_galaxy_token("dependency name", &name, path)?;
        validate_galaxy_token("dependency version", &version, path)?;
        let bundle_bytes = parse_optional_u64(&block, "bundle_bytes").ok_or_else(|| {
            format!(
                "galaxy lock `{}` dependency `{}` is missing required key `bundle_bytes`",
                path.display(),
                name
            )
        })?;
        let bundle_fnv1a64 = parse_required_string(&block, "bundle_fnv1a64", path)?;
        entries.push(GalaxyLockEntry {
            name,
            version,
            bundle,
            bundle_bytes,
            bundle_fnv1a64,
        });
    }
    Ok(entries)
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
            if version_path.join("nuis.toml").exists() {
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
