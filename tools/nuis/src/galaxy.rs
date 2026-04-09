use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};

const GALAXY_MAGIC: &[u8; 8] = b"GALAXY01";
const GALAXY_BUNDLE_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalaxyManifest {
    pub manifest_schema: String,
    pub name: String,
    pub version: String,
    pub package_kind: String,
    pub project: String,
    pub summary: String,
    pub license: String,
    pub repository: String,
    pub authors: Vec<String>,
    pub include: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedGalaxy {
    pub root: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest: GalaxyManifest,
    pub project: nuisc::project::LoadedProject,
    pub include_files: Vec<PathBuf>,
    pub abi_entries: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalaxyBundleEntry {
    pub path: String,
    pub bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectedGalaxyBundle {
    pub manifest: GalaxyManifest,
    pub entries: Vec<GalaxyBundleEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalGalaxyIndexEntry {
    pub name: String,
    pub version: String,
    pub package: String,
    pub project: String,
    pub abi: Vec<String>,
    pub bundle_bytes: Option<u64>,
    pub bundle_fnv1a64: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedLocalGalaxy {
    pub name: String,
    pub version: String,
    pub package: PathBuf,
    pub bundle_bytes: u64,
    pub bundle_fnv1a64: String,
    pub entries: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovedLocalGalaxy {
    pub name: String,
    pub version: String,
    pub package: PathBuf,
    pub index_entry: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledGalaxyDependency {
    pub name: String,
    pub version: String,
    pub output: PathBuf,
    pub project: PathBuf,
    pub bundle: PathBuf,
    pub bundle_bytes: u64,
    pub bundle_fnv1a64: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalaxyLockEntry {
    pub name: String,
    pub version: String,
    pub bundle: PathBuf,
    pub bundle_bytes: u64,
    pub bundle_fnv1a64: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WroteGalaxyLock {
    pub path: PathBuf,
    pub entries: Vec<GalaxyLockEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledProjectDeps {
    pub installed: Vec<InstalledGalaxyDependency>,
    pub lock: WroteGalaxyLock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedGalaxyLock {
    pub path: PathBuf,
    pub entries: Vec<GalaxyLockEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncedProjectDeps {
    pub root: PathBuf,
    pub entries: Vec<GalaxyLockEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalaxyDoctorDependency {
    pub name: String,
    pub version: String,
    pub local_available: bool,
    pub locked: bool,
    pub installed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalaxyDoctorReport {
    pub project_root: PathBuf,
    pub deps_root: PathBuf,
    pub local_registry_root: PathBuf,
    pub lock_path: PathBuf,
    pub lock_status: String,
    pub lock_error: Option<String>,
    pub dependencies: Vec<GalaxyDoctorDependency>,
}

pub fn init(input: &Path) -> Result<PathBuf, String> {
    ensure_local_layout()?;
    let project = nuisc::project::load_project(input)?;
    let manifest_path = project.root.join("galaxy.toml");
    if manifest_path.exists() {
        return Err(format!(
            "galaxy manifest already exists at `{}`",
            manifest_path.display()
        ));
    }

    let manifest = default_manifest(&project)?;
    fs::write(&manifest_path, render_manifest(&manifest))
        .map_err(|error| format!("failed to write `{}`: {error}", manifest_path.display()))?;
    Ok(manifest_path)
}

pub fn check(input: &Path) -> Result<CheckedGalaxy, String> {
    ensure_local_layout()?;
    let (root, manifest_path) = resolve_galaxy_manifest(input)?;
    let source = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("failed to read `{}`: {error}", manifest_path.display()))?;
    let manifest = parse_manifest(&source, &manifest_path)?;
    let project_path = root.join(&manifest.project);
    let project = nuisc::project::load_project(&project_path)?;
    let abi = nuisc::project::resolve_project_abi(&project)?;

    let include_files = manifest
        .include
        .iter()
        .map(|item| root.join(item))
        .collect::<Vec<_>>();
    for path in &include_files {
        if !path.exists() {
            return Err(format!(
                "galaxy manifest `{}` references missing include `{}`",
                manifest_path.display(),
                path.display()
            ));
        }
    }

    let mut required = BTreeSet::new();
    required.insert(project.manifest_path.clone());
    for module in &project.modules {
        required.insert(module.path.clone());
    }
    for path in &required {
        if !include_files.iter().any(|item| item == path) {
            return Err(format!(
                "galaxy manifest `{}` is missing required project file `{}` in `include`",
                manifest_path.display(),
                path.display()
            ));
        }
    }

    Ok(CheckedGalaxy {
        root,
        manifest_path,
        manifest,
        project,
        include_files,
        abi_entries: abi
            .requirements
            .into_iter()
            .map(|item| (item.domain, item.abi))
            .collect(),
    })
}

pub fn pack(input: &Path, output: &Path) -> Result<PathBuf, String> {
    let checked = check(input)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create `{}`: {error}", parent.display()))?;
    }

    let manifest_source = fs::read_to_string(&checked.manifest_path).map_err(|error| {
        format!(
            "failed to read `{}` for pack: {error}",
            checked.manifest_path.display()
        )
    })?;

    let mut bytes = Vec::new();
    bytes.extend_from_slice(GALAXY_MAGIC);
    bytes.extend_from_slice(&GALAXY_BUNDLE_VERSION.to_le_bytes());
    bytes.extend_from_slice(&(manifest_source.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(checked.include_files.len() as u32).to_le_bytes());
    bytes.extend_from_slice(manifest_source.as_bytes());

    for path in &checked.include_files {
        let relative = path
            .strip_prefix(&checked.root)
            .unwrap_or(path)
            .display()
            .to_string();
        let content = fs::read(path)
            .map_err(|error| format!("failed to read `{}` for pack: {error}", path.display()))?;
        bytes.extend_from_slice(&(relative.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&(content.len() as u64).to_le_bytes());
        bytes.extend_from_slice(relative.as_bytes());
        bytes.extend_from_slice(&content);
    }

    fs::write(output, bytes)
        .map_err(|error| format!("failed to write `{}`: {error}", output.display()))?;

    write_local_index_entry(&checked, output)?;
    Ok(output.to_path_buf())
}

pub fn inspect_bundle(input: &Path) -> Result<InspectedGalaxyBundle, String> {
    let bytes = fs::read(input)
        .map_err(|error| format!("failed to read `{}`: {error}", input.display()))?;
    decode_bundle(&bytes, input)
}

pub fn publish_local(input: &Path, output: Option<&Path>) -> Result<PathBuf, String> {
    ensure_local_layout()?;
    let checked = check(input)?;
    let bundle_path = output.map(PathBuf::from).unwrap_or_else(|| {
        local_packages_root()
            .join(&checked.manifest.name)
            .join(&checked.manifest.version)
            .join(format!(
                "{}-{}.galaxy",
                checked.manifest.name, checked.manifest.version
            ))
    });
    let packed = pack(input, &bundle_path)?;
    Ok(packed)
}

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
    let chosen = select_local_entry(name, version)?;

    let bundle = inspect_bundle(Path::new(&chosen.package))?;
    fs::create_dir_all(output)
        .map_err(|error| format!("failed to create `{}`: {error}", output.display()))?;
    let bytes = fs::read(&chosen.package)
        .map_err(|error| format!("failed to read bundle `{}`: {error}", chosen.package))?;
    extract_bundle(&bytes, Path::new(&chosen.package), output)?;
    Ok(output.join(&bundle.manifest.project))
}

pub fn verify_local(name: &str, version: Option<&str>) -> Result<VerifiedLocalGalaxy, String> {
    ensure_local_layout()?;
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
    let chosen = select_local_entry(name, version)?;
    inspect_bundle(Path::new(&chosen.package))
}

pub fn remove_local(name: &str, version: Option<&str>) -> Result<RemovedLocalGalaxy, String> {
    ensure_local_layout()?;
    let chosen = select_local_entry(name, version)?;
    let package_path = PathBuf::from(&chosen.package);
    let index_entry = local_index_root()
        .join(&chosen.name)
        .join(format!("{}.toml", chosen.version));

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

pub fn install_project_deps(input: &Path) -> Result<InstalledProjectDeps, String> {
    ensure_local_layout()?;
    let project = nuisc::project::load_project(input)?;
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
    let lock = write_project_lock_from_installed(&project.root, &installed)?;
    Ok(InstalledProjectDeps { installed, lock })
}

pub fn lock_project_deps(input: &Path) -> Result<WroteGalaxyLock, String> {
    ensure_local_layout()?;
    let project = nuisc::project::load_project(input)?;
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
    write_project_lock(&project.root, &entries)
}

pub fn verify_project_lock(input: &Path) -> Result<VerifiedGalaxyLock, String> {
    ensure_local_layout()?;
    let project = nuisc::project::load_project(input)?;
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
    Ok(VerifiedGalaxyLock { path, entries })
}

pub fn sync_project_deps(input: &Path) -> Result<SyncedProjectDeps, String> {
    ensure_local_layout()?;
    let project = nuisc::project::load_project(input)?;
    let verified = verify_project_lock(input)?;
    let deps_root = project.root.join(".nuis").join("deps").join("galaxy");

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
        root: deps_root,
        entries: verified.entries,
    })
}

pub fn doctor_project(input: &Path) -> Result<GalaxyDoctorReport, String> {
    ensure_local_layout()?;
    let project = nuisc::project::load_project(input)?;
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
        deps_root,
        local_registry_root: local_root(),
        lock_path,
        lock_status,
        lock_error,
        dependencies,
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

fn ensure_local_layout() -> Result<(), String> {
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

fn default_manifest(project: &nuisc::project::LoadedProject) -> Result<GalaxyManifest, String> {
    let mut include = vec!["nuis.toml".to_owned()];
    for module in &project.modules {
        let relative = module
            .path
            .strip_prefix(&project.root)
            .unwrap_or(module.path.as_path())
            .display()
            .to_string();
        if !include.iter().any(|item| item == &relative) {
            include.push(relative);
        }
    }
    include.sort();

    Ok(GalaxyManifest {
        manifest_schema: "galaxy-manifest-v1".to_owned(),
        name: project.manifest.name.clone(),
        version: "0.1.0".to_owned(),
        package_kind: "nuis-project".to_owned(),
        project: "nuis.toml".to_owned(),
        summary: format!(
            "Galaxy package for nuis project `{}`",
            project.manifest.name
        ),
        license: "UNLICENSED".to_owned(),
        repository: String::new(),
        authors: Vec::new(),
        include,
    })
}

fn resolve_galaxy_manifest(input: &Path) -> Result<(PathBuf, PathBuf), String> {
    if input.is_dir() {
        let manifest_path = input.join("galaxy.toml");
        return Ok((input.to_path_buf(), manifest_path));
    }
    if input.file_name().and_then(|item| item.to_str()) == Some("galaxy.toml") {
        let root = input.parent().ok_or_else(|| {
            format!(
                "galaxy manifest `{}` has no parent directory",
                input.display()
            )
        })?;
        return Ok((root.to_path_buf(), input.to_path_buf()));
    }
    let root = input
        .parent()
        .ok_or_else(|| format!("input `{}` has no parent directory", input.display()))?;
    Ok((root.to_path_buf(), root.join("galaxy.toml")))
}

fn write_local_index_entry(checked: &CheckedGalaxy, output: &Path) -> Result<(), String> {
    let package_dir = local_index_root().join(&checked.manifest.name);
    fs::create_dir_all(&package_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", package_dir.display()))?;
    let entry_path = package_dir.join(format!("{}.toml", checked.manifest.version));
    let mut abi_entries = checked
        .abi_entries
        .iter()
        .map(|(domain, abi)| format!("{domain}={abi}"))
        .collect::<Vec<_>>();
    abi_entries.sort();
    let bundle_bytes = fs::read(output).map_err(|error| {
        format!(
            "failed to read `{}` for local index: {error}",
            output.display()
        )
    })?;
    let bundle_len = bundle_bytes.len() as u64;
    let bundle_hash = fnv1a64_hex(&bundle_bytes);
    let source = format!(
        "name = \"{}\"\nversion = \"{}\"\npackage = \"{}\"\nproject = \"{}\"\nabi = {}\nbundle_bytes = {}\nbundle_fnv1a64 = \"{}\"\n",
        checked.manifest.name,
        checked.manifest.version,
        output.display(),
        checked.manifest.project,
        render_string_array(&abi_entries),
        bundle_len,
        bundle_hash
    );
    fs::write(&entry_path, source)
        .map_err(|error| format!("failed to write `{}`: {error}", entry_path.display()))?;
    Ok(())
}

fn decode_bundle(bytes: &[u8], path: &Path) -> Result<InspectedGalaxyBundle, String> {
    if bytes.len() < 18 {
        return Err(format!(
            "`{}` is too short to be a galaxy bundle",
            path.display()
        ));
    }
    if &bytes[..8] != GALAXY_MAGIC {
        return Err(format!(
            "`{}` does not start with the galaxy bundle magic",
            path.display()
        ));
    }
    let version = u16::from_le_bytes(bytes[8..10].try_into().unwrap());
    if version != GALAXY_BUNDLE_VERSION {
        return Err(format!(
            "`{}` has unsupported galaxy bundle version {}; expected {}",
            path.display(),
            version,
            GALAXY_BUNDLE_VERSION
        ));
    }
    let manifest_len = u32::from_le_bytes(bytes[10..14].try_into().unwrap()) as usize;
    let entry_count = u32::from_le_bytes(bytes[14..18].try_into().unwrap()) as usize;
    let mut offset = 18usize;
    if bytes.len() < offset + manifest_len {
        return Err(format!(
            "`{}` is truncated before manifest payload",
            path.display()
        ));
    }
    let manifest_source =
        std::str::from_utf8(&bytes[offset..offset + manifest_len]).map_err(|e| {
            format!(
                "`{}` has invalid utf-8 in galaxy manifest payload: {e}",
                path.display()
            )
        })?;
    offset += manifest_len;
    let manifest = parse_manifest(manifest_source, path)?;

    let mut entries = Vec::new();
    for _ in 0..entry_count {
        if bytes.len() < offset + 12 {
            return Err(format!(
                "`{}` is truncated while decoding bundle entries",
                path.display()
            ));
        }
        let path_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let content_len =
            u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap()) as usize;
        offset += 8;
        if bytes.len() < offset + path_len + content_len {
            return Err(format!(
                "`{}` is truncated while decoding entry payload",
                path.display()
            ));
        }
        let entry_path = std::str::from_utf8(&bytes[offset..offset + path_len]).map_err(|e| {
            format!(
                "`{}` has invalid utf-8 in bundle entry path: {e}",
                path.display()
            )
        })?;
        offset += path_len + content_len;
        entries.push(GalaxyBundleEntry {
            path: entry_path.to_owned(),
            bytes: content_len,
        });
    }

    if offset != bytes.len() {
        return Err(format!(
            "`{}` has trailing bytes after decoding galaxy bundle",
            path.display()
        ));
    }

    Ok(InspectedGalaxyBundle { manifest, entries })
}

fn extract_bundle(bytes: &[u8], path: &Path, output: &Path) -> Result<(), String> {
    if bytes.len() < 18 || &bytes[..8] != GALAXY_MAGIC {
        return Err(format!("`{}` is not a valid galaxy bundle", path.display()));
    }
    let manifest_len = u32::from_le_bytes(bytes[10..14].try_into().unwrap()) as usize;
    let entry_count = u32::from_le_bytes(bytes[14..18].try_into().unwrap()) as usize;
    let mut offset = 18usize + manifest_len;
    for _ in 0..entry_count {
        let path_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let content_len =
            u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap()) as usize;
        offset += 8;
        let relative = std::str::from_utf8(&bytes[offset..offset + path_len]).map_err(|e| {
            format!(
                "`{}` has invalid utf-8 in install entry path: {e}",
                path.display()
            )
        })?;
        offset += path_len;
        let content = &bytes[offset..offset + content_len];
        offset += content_len;
        let target = output.join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create `{}`: {error}", parent.display()))?;
        }
        fs::write(&target, content)
            .map_err(|error| format!("failed to write `{}`: {error}", target.display()))?;
    }
    Ok(())
}

fn render_manifest(manifest: &GalaxyManifest) -> String {
    format!(
        "manifest_schema = \"{}\"\nname = \"{}\"\nversion = \"{}\"\npackage_kind = \"{}\"\nproject = \"{}\"\nsummary = \"{}\"\nlicense = \"{}\"\nrepository = \"{}\"\nauthors = {}\ninclude = {}\n",
        manifest.manifest_schema,
        escape(&manifest.name),
        escape(&manifest.version),
        escape(&manifest.package_kind),
        escape(&manifest.project),
        escape(&manifest.summary),
        escape(&manifest.license),
        escape(&manifest.repository),
        render_string_array(&manifest.authors),
        render_string_array(&manifest.include),
    )
}

fn parse_manifest(source: &str, path: &Path) -> Result<GalaxyManifest, String> {
    Ok(GalaxyManifest {
        manifest_schema: parse_required_string(source, "manifest_schema", path)?,
        name: parse_required_string(source, "name", path)?,
        version: parse_required_string(source, "version", path)?,
        package_kind: parse_required_string(source, "package_kind", path)?,
        project: parse_required_string(source, "project", path)?,
        summary: parse_optional_string(source, "summary").unwrap_or_default(),
        license: parse_optional_string(source, "license").unwrap_or_default(),
        repository: parse_optional_string(source, "repository").unwrap_or_default(),
        authors: parse_optional_string_array(source, "authors").unwrap_or_default(),
        include: parse_optional_string_array(source, "include").unwrap_or_default(),
    })
}

fn parse_required_string(source: &str, key: &str, path: &Path) -> Result<String, String> {
    parse_optional_string(source, key).ok_or_else(|| {
        format!(
            "galaxy manifest `{}` is missing required key `{key}`",
            path.display()
        )
    })
}

fn parse_optional_string(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    for raw in source.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let value = rest.trim();
            if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                return Some(value[1..value.len() - 1].to_owned());
            }
            return None;
        }
    }
    None
}

fn parse_optional_string_array(source: &str, key: &str) -> Option<Vec<String>> {
    let prefix = format!("{key} = ");
    for raw in source.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let value = rest.trim();
            if !(value.starts_with('[') && value.ends_with(']')) {
                return None;
            }
            let inner = &value[1..value.len() - 1];
            if inner.trim().is_empty() {
                return Some(Vec::new());
            }
            let mut items = Vec::new();
            for item in inner.split(',') {
                let item = item.trim();
                if !(item.starts_with('"') && item.ends_with('"') && item.len() >= 2) {
                    return None;
                }
                items.push(item[1..item.len() - 1].to_owned());
            }
            return Some(items);
        }
    }
    None
}

fn parse_local_index_entry(source: &str, path: &Path) -> Result<LocalGalaxyIndexEntry, String> {
    Ok(LocalGalaxyIndexEntry {
        name: parse_required_string(source, "name", path)?,
        version: parse_required_string(source, "version", path)?,
        package: parse_required_string(source, "package", path)?,
        project: parse_required_string(source, "project", path)?,
        abi: parse_optional_string_array(source, "abi").unwrap_or_default(),
        bundle_bytes: parse_optional_u64(source, "bundle_bytes"),
        bundle_fnv1a64: parse_optional_string(source, "bundle_fnv1a64"),
    })
}

fn select_local_entry(name: &str, version: Option<&str>) -> Result<LocalGalaxyIndexEntry, String> {
    let entries = list_local()?;
    let mut matches = entries
        .into_iter()
        .filter(|entry| entry.name == name)
        .collect::<Vec<_>>();
    if matches.is_empty() {
        return Err(format!("no local galaxy package named `{name}`"));
    }
    matches.sort_by(|lhs, rhs| compare_version(&lhs.version, &rhs.version));
    let chosen = if let Some(version) = version {
        matches
            .into_iter()
            .find(|entry| entry.version == version)
            .ok_or_else(|| format!("no local galaxy package `{name}` with version `{version}`"))?
    } else {
        matches.pop().unwrap()
    };
    Ok(chosen)
}

fn compare_version(lhs: &str, rhs: &str) -> std::cmp::Ordering {
    let lhs_parts = parse_version_parts(lhs);
    let rhs_parts = parse_version_parts(rhs);
    if lhs_parts.is_empty() || rhs_parts.is_empty() {
        return lhs.cmp(rhs);
    }
    let width = lhs_parts.len().max(rhs_parts.len());
    for index in 0..width {
        let lhs_part = lhs_parts.get(index).copied().unwrap_or(0);
        let rhs_part = rhs_parts.get(index).copied().unwrap_or(0);
        match lhs_part.cmp(&rhs_part) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    lhs.cmp(rhs)
}

fn parse_version_parts(value: &str) -> Vec<u64> {
    value
        .split('.')
        .filter_map(|part| part.parse::<u64>().ok())
        .collect()
}

fn render_string_array(values: &[String]) -> String {
    let items = values
        .iter()
        .map(|value| format!("\"{}\"", escape(value)))
        .collect::<Vec<_>>();
    format!("[{}]", items.join(", "))
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn parse_optional_u64(source: &str, key: &str) -> Option<u64> {
    let prefix = format!("{key} = ");
    for raw in source.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return rest.trim().parse::<u64>().ok();
        }
    }
    None
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    format!("{hash:016x}")
}

fn write_project_lock_from_installed(
    project_root: &Path,
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
    write_project_lock(project_root, &entries)
}

fn write_project_lock(
    project_root: &Path,
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
        path,
        entries: sorted,
    })
}

fn parse_lock_entries(source: &str, path: &Path) -> Result<Vec<GalaxyLockEntry>, String> {
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

fn collect_installed_project_deps(root: &Path) -> Result<BTreeSet<String>, String> {
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

fn remove_dir_if_empty(path: Option<&Path>) -> Result<(), String> {
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
