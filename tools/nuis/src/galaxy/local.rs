use super::bundle::{decode_bundle, extract_bundle};
use super::*;
use std::env;

pub fn list_local() -> Result<Vec<LocalGalaxyIndexEntry>, String> {
    ensure_local_layout()?;
    let mut entries = Vec::new();
    let root = local_index_root();
    if !root.exists() {
        return Ok(entries);
    }
    for package_dir in fs::read_dir(&root)
        .map_err(|error| format!("failed to read `{}`: {error}", root.display()))?
    {
        let package_dir = package_dir
            .map_err(|error| format!("failed to enumerate `{}`: {error}", root.display()))?;
        let package_path = package_dir.path();
        if !package_path.is_dir() {
            continue;
        }
        for file in fs::read_dir(&package_path)
            .map_err(|error| format!("failed to read `{}`: {error}", package_path.display()))?
        {
            let file = file.map_err(|error| {
                format!("failed to enumerate `{}`: {error}", package_path.display())
            })?;
            let path = file.path();
            if path.extension().and_then(|item| item.to_str()) != Some("toml") {
                continue;
            }
            let source = fs::read_to_string(&path)
                .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
            entries.push(parse_local_index_entry(&source, &path)?);
        }
    }
    entries.sort_by(|lhs, rhs| {
        lhs.name
            .cmp(&rhs.name)
            .then(compare_version(&lhs.version, &rhs.version))
    });
    Ok(entries)
}

pub fn install_local(name: &str, version: Option<&str>, output: &Path) -> Result<PathBuf, String> {
    ensure_local_layout()?;
    validate_galaxy_token("name", name, Path::new("<galaxy install-local>"))?;
    if let Some(version) = version {
        validate_galaxy_token("version", version, Path::new("<galaxy install-local>"))?;
    }
    let chosen = select_local_entry(name, version)?;

    let bundle = super::inspect_bundle(Path::new(&chosen.package))?;
    fs::create_dir_all(output)
        .map_err(|error| format!("failed to create `{}`: {error}", output.display()))?;
    let bytes = fs::read(&chosen.package)
        .map_err(|error| format!("failed to read bundle `{}`: {error}", chosen.package))?;
    extract_bundle(&bytes, Path::new(&chosen.package), output)?;
    Ok(output.join(&bundle.manifest.project))
}

pub fn verify_local(name: &str, version: Option<&str>) -> Result<VerifiedLocalGalaxy, String> {
    ensure_local_layout()?;
    validate_galaxy_token("name", name, Path::new("<galaxy verify-local>"))?;
    if let Some(version) = version {
        validate_galaxy_token("version", version, Path::new("<galaxy verify-local>"))?;
    }
    let chosen = select_local_entry(name, version)?;
    let bundle_path = PathBuf::from(&chosen.package);
    let bytes = fs::read(&bundle_path)
        .map_err(|error| format!("failed to read bundle `{}`: {error}", bundle_path.display()))?;
    let inspected = decode_bundle(&bytes, &bundle_path)?;
    let actual_bytes = bytes.len() as u64;
    let actual_hash = fnv1a64_hex(&bytes);

    if let Some(expected_bytes) = chosen.bundle_bytes {
        if expected_bytes != actual_bytes {
            return Err(format!(
                "local galaxy package `{}` version `{}` byte size mismatch: index={}, actual={}",
                chosen.name, chosen.version, expected_bytes, actual_bytes
            ));
        }
    }
    if let Some(expected_hash) = &chosen.bundle_fnv1a64 {
        if expected_hash != &actual_hash {
            return Err(format!(
                "local galaxy package `{}` version `{}` hash mismatch: index={}, actual={}",
                chosen.name, chosen.version, expected_hash, actual_hash
            ));
        }
    }

    Ok(VerifiedLocalGalaxy {
        name: chosen.name,
        version: chosen.version,
        package: bundle_path,
        bundle_bytes: actual_bytes,
        bundle_fnv1a64: actual_hash,
        entries: inspected.entries.len(),
    })
}

pub fn inspect_local(name: &str, version: Option<&str>) -> Result<InspectedGalaxyBundle, String> {
    ensure_local_layout()?;
    validate_galaxy_token("name", name, Path::new("<galaxy inspect-local>"))?;
    if let Some(version) = version {
        validate_galaxy_token("version", version, Path::new("<galaxy inspect-local>"))?;
    }
    let chosen = select_local_entry(name, version)?;
    super::inspect_bundle(Path::new(&chosen.package))
}

pub fn remove_local(name: &str, version: Option<&str>) -> Result<RemovedLocalGalaxy, String> {
    ensure_local_layout()?;
    validate_galaxy_token("name", name, Path::new("<galaxy remove-local>"))?;
    if let Some(version) = version {
        validate_galaxy_token("version", version, Path::new("<galaxy remove-local>"))?;
    }
    let chosen = select_local_entry(name, version)?;
    let package_path = PathBuf::from(&chosen.package);
    let index_entry = local_index_root()
        .join(&chosen.name)
        .join(format!("{}.toml", chosen.version));

    if package_path.exists() {
        validate_path_under_root(
            "local package removal",
            &package_path,
            &local_packages_root(),
            Path::new("<galaxy remove-local>"),
        )?;
    }
    if index_entry.exists() {
        fs::remove_file(&index_entry).map_err(|error| {
            format!(
                "failed to remove local galaxy index `{}`: {error}",
                index_entry.display()
            )
        })?;
    }
    if package_path.exists() {
        fs::remove_file(&package_path).map_err(|error| {
            format!(
                "failed to remove local galaxy bundle `{}`: {error}",
                package_path.display()
            )
        })?;
    }

    remove_dir_if_empty(index_entry.parent())?;
    remove_dir_if_empty(index_entry.parent().and_then(Path::parent))?;
    remove_dir_if_empty(package_path.parent())?;
    remove_dir_if_empty(package_path.parent().and_then(Path::parent))?;
    remove_dir_if_empty(
        package_path
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent),
    )?;

    Ok(RemovedLocalGalaxy {
        name: chosen.name,
        version: chosen.version,
        package: package_path,
        index_entry,
    })
}

pub fn local_root() -> PathBuf {
    nuis_home_root().join("galaxy")
}

pub fn local_index_root() -> PathBuf {
    local_root().join("index")
}

pub fn local_packages_root() -> PathBuf {
    local_root().join("packages")
}

pub fn local_cache_root() -> PathBuf {
    local_root().join("cache")
}

pub(super) fn ensure_local_layout() -> Result<(), String> {
    for path in [
        local_root(),
        local_index_root(),
        local_packages_root(),
        local_cache_root(),
    ] {
        fs::create_dir_all(&path)
            .map_err(|error| format!("failed to create `{}`: {error}", path.display()))?;
    }
    Ok(())
}

fn nuis_home_root() -> PathBuf {
    if let Ok(root) = env::var("NUIS_HOME") {
        return PathBuf::from(root);
    }
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_owned());
    PathBuf::from(home).join(".nuis")
}
