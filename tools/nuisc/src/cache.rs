use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::project::LoadedProject;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileCacheKey {
    pub root: PathBuf,
    pub key: String,
    pub input_labels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileCacheEntry {
    pub root: PathBuf,
    pub key: String,
    pub entry_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileCacheStatus {
    pub root: PathBuf,
    pub key: String,
    pub input_labels: Vec<String>,
    pub entry_exists: bool,
    pub entry_dir: PathBuf,
    pub file_count: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileCacheInventoryEntry {
    pub key: String,
    pub entry_dir: PathBuf,
    pub file_count: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileCacheInventory {
    pub root: PathBuf,
    pub entry_count: usize,
    pub total_files: usize,
    pub total_bytes: u64,
    pub entries: Vec<CompileCacheInventoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileCacheInventorySummary {
    pub workspace_root: PathBuf,
    pub roots: Vec<CompileCacheInventory>,
    pub total_entries: usize,
    pub total_files: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanedCompileCache {
    pub root: PathBuf,
    pub removed_entries: usize,
    pub removed_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanedCompileCacheSummary {
    pub workspace_root: PathBuf,
    pub cleaned_roots: Vec<CleanedCompileCache>,
    pub removed_entries: usize,
    pub removed_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrunedCompileCache {
    pub root: PathBuf,
    pub kept_entries: usize,
    pub removed_entries: usize,
    pub removed_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrunedCompileCacheSummary {
    pub workspace_root: PathBuf,
    pub pruned_roots: Vec<PrunedCompileCache>,
    pub kept_entries: usize,
    pub removed_entries: usize,
    pub removed_bytes: u64,
}

pub fn compute_compile_cache_key(
    input: &Path,
    project: Option<&LoadedProject>,
) -> Result<CompileCacheKey, String> {
    let root = cache_root(input, project);
    let mut records = Vec::<(String, Vec<u8>)>::new();

    records.push((
        "toolchain.nuisc.version".to_owned(),
        env!("CARGO_PKG_VERSION").as_bytes().to_vec(),
    ));
    records.push((
        "toolchain.engine.version".to_owned(),
        crate::engine::default_engine().version.as_bytes().to_vec(),
    ));
    records.push((
        "toolchain.engine.profile".to_owned(),
        crate::engine::default_engine().profile.as_bytes().to_vec(),
    ));

    if let Some(project) = project {
        records.push((
            "project.manifest".to_owned(),
            fs::read(&project.manifest_path).map_err(|error| {
                format!(
                    "failed to read `{}` for compile cache: {error}",
                    project.manifest_path.display()
                )
            })?,
        ));
        for module in &project.modules {
            let relative = module
                .path
                .strip_prefix(&project.root)
                .unwrap_or(module.path.as_path())
                .display()
                .to_string();
            records.push((
                format!("project.module:{relative}"),
                fs::read(&module.path).map_err(|error| {
                    format!(
                        "failed to read `{}` for compile cache: {error}",
                        module.path.display()
                    )
                })?,
            ));
        }
        let lock_path = project.root.join("nuis.galaxy.lock");
        if lock_path.exists() {
            records.push((
                "project.galaxy_lock".to_owned(),
                fs::read(&lock_path).map_err(|error| {
                    format!(
                        "failed to read `{}` for compile cache: {error}",
                        lock_path.display()
                    )
                })?,
            ));
        }
    } else {
        records.push((
            format!("source:{}", input.display()),
            fs::read(input).map_err(|error| {
                format!(
                    "failed to read `{}` for compile cache: {error}",
                    input.display()
                )
            })?,
        ));
    }

    for registry_path in collect_registry_manifest_paths(Path::new("nustar-packages"))? {
        let relative = registry_path.display().to_string();
        records.push((
            format!("registry:{relative}"),
            fs::read(&registry_path).map_err(|error| {
                format!(
                    "failed to read `{}` for compile cache: {error}",
                    registry_path.display()
                )
            })?,
        ));
    }

    records.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
    let input_labels = records.iter().map(|(label, _)| label.clone()).collect();
    let key = fingerprint_records(&records);
    Ok(CompileCacheKey {
        root,
        key,
        input_labels,
    })
}

pub fn lookup_compile_cache(key: &CompileCacheKey) -> Result<Option<CompileCacheEntry>, String> {
    fs::create_dir_all(&key.root)
        .map_err(|error| format!("failed to create `{}`: {error}", key.root.display()))?;
    let entry_dir = key.root.join(&key.key);
    if entry_dir.is_dir() {
        Ok(Some(CompileCacheEntry {
            root: key.root.clone(),
            key: key.key.clone(),
            entry_dir,
        }))
    } else {
        Ok(None)
    }
}

pub fn store_compile_cache(
    key: &CompileCacheKey,
    output_dir: &Path,
) -> Result<CompileCacheEntry, String> {
    fs::create_dir_all(&key.root)
        .map_err(|error| format!("failed to create `{}`: {error}", key.root.display()))?;
    let entry_dir = key.root.join(&key.key);
    if entry_dir.is_dir() {
        return Ok(CompileCacheEntry {
            root: key.root.clone(),
            key: key.key.clone(),
            entry_dir,
        });
    }
    let temp_dir = key
        .root
        .join(format!("{}.tmp-{}", key.key, std::process::id()));
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)
            .map_err(|error| format!("failed to reset `{}`: {error}", temp_dir.display()))?;
    }
    copy_directory_recursive(output_dir, &temp_dir)?;
    match fs::rename(&temp_dir, &entry_dir) {
        Ok(()) => {}
        Err(_) if entry_dir.is_dir() => {
            fs::remove_dir_all(&temp_dir).map_err(|error| {
                format!(
                    "failed to clean temporary cache `{}`: {error}",
                    temp_dir.display()
                )
            })?;
        }
        Err(error) => {
            return Err(format!(
                "failed to finalize compile cache `{}`: {error}",
                entry_dir.display()
            ));
        }
    }
    Ok(CompileCacheEntry {
        root: key.root.clone(),
        key: key.key.clone(),
        entry_dir,
    })
}

pub fn restore_compile_cache(entry: &CompileCacheEntry, output_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", output_dir.display()))?;
    copy_directory_recursive(&entry.entry_dir, output_dir)
}

pub fn compile_cache_status(
    input: &Path,
    project: Option<&LoadedProject>,
) -> Result<CompileCacheStatus, String> {
    let key = compute_compile_cache_key(input, project)?;
    let entry_dir = key.root.join(&key.key);
    let (file_count, total_bytes) = if entry_dir.is_dir() {
        summarize_directory(&entry_dir)?
    } else {
        (0, 0)
    };
    Ok(CompileCacheStatus {
        root: key.root,
        key: key.key,
        input_labels: key.input_labels,
        entry_exists: entry_dir.is_dir(),
        entry_dir,
        file_count,
        total_bytes,
    })
}

pub fn clean_compile_cache(
    input: &Path,
    project: Option<&LoadedProject>,
) -> Result<CleanedCompileCache, String> {
    let root = cache_root(input, project);
    clean_compile_cache_root(&root)
}

pub fn clean_compile_cache_summary(
    workspace_root: &Path,
) -> Result<CleanedCompileCacheSummary, String> {
    let cleaned_roots = discover_compile_cache_roots(workspace_root)?
        .into_iter()
        .map(|root| clean_compile_cache_root(&root))
        .collect::<Result<Vec<_>, _>>()?;
    let removed_entries = cleaned_roots
        .iter()
        .map(|cleaned| cleaned.removed_entries)
        .sum();
    let removed_bytes = cleaned_roots
        .iter()
        .map(|cleaned| cleaned.removed_bytes)
        .sum();
    Ok(CleanedCompileCacheSummary {
        workspace_root: workspace_root.to_path_buf(),
        cleaned_roots,
        removed_entries,
        removed_bytes,
    })
}

pub fn prune_compile_cache(
    input: &Path,
    project: Option<&LoadedProject>,
    keep: usize,
) -> Result<PrunedCompileCache, String> {
    let root = cache_root(input, project);
    prune_compile_cache_root(&root, keep)
}

pub fn prune_compile_cache_summary(
    workspace_root: &Path,
    keep: usize,
) -> Result<PrunedCompileCacheSummary, String> {
    let pruned_roots = discover_compile_cache_roots(workspace_root)?
        .into_iter()
        .map(|root| prune_compile_cache_root(&root, keep))
        .collect::<Result<Vec<_>, _>>()?;
    let kept_entries = pruned_roots.iter().map(|pruned| pruned.kept_entries).sum();
    let removed_entries = pruned_roots.iter().map(|pruned| pruned.removed_entries).sum();
    let removed_bytes = pruned_roots.iter().map(|pruned| pruned.removed_bytes).sum();
    Ok(PrunedCompileCacheSummary {
        workspace_root: workspace_root.to_path_buf(),
        pruned_roots,
        kept_entries,
        removed_entries,
        removed_bytes,
    })
}

fn clean_compile_cache_root(root: &Path) -> Result<CleanedCompileCache, String> {
    if !root.exists() {
        return Ok(CleanedCompileCache {
            root: root.to_path_buf(),
            removed_entries: 0,
            removed_bytes: 0,
        });
    }
    let (removed_entries, removed_bytes) = summarize_cache_entries(root)?;
    fs::remove_dir_all(root)
        .map_err(|error| format!("failed to remove `{}`: {error}", root.display()))?;
    Ok(CleanedCompileCache {
        root: root.to_path_buf(),
        removed_entries,
        removed_bytes,
    })
}

fn prune_compile_cache_root(root: &Path, keep: usize) -> Result<PrunedCompileCache, String> {
    if !root.exists() {
        return Ok(PrunedCompileCache {
            root: root.to_path_buf(),
            kept_entries: 0,
            removed_entries: 0,
            removed_bytes: 0,
        });
    }
    let mut entries = collect_cache_entry_stats(root)?;
    entries.sort_by(|lhs, rhs| rhs.modified.cmp(&lhs.modified).then_with(|| lhs.key.cmp(&rhs.key)));
    let kept_entries = entries.len().min(keep);
    let mut removed_entries = 0usize;
    let mut removed_bytes = 0u64;
    for entry in entries.into_iter().skip(keep) {
        fs::remove_dir_all(&entry.entry_dir)
            .map_err(|error| format!("failed to prune `{}`: {error}", entry.entry_dir.display()))?;
        removed_entries += 1;
        removed_bytes += entry.total_bytes;
    }
    if keep == 0 && root.is_dir() && fs::read_dir(root)
        .map_err(|error| format!("failed to read `{}`: {error}", root.display()))?
        .next()
        .is_none()
    {
        fs::remove_dir(root)
            .map_err(|error| format!("failed to remove empty cache root `{}`: {error}", root.display()))?;
    }
    Ok(PrunedCompileCache {
        root: root.to_path_buf(),
        kept_entries,
        removed_entries,
        removed_bytes,
    })
}

pub fn compile_cache_inventory(root: &Path) -> Result<CompileCacheInventory, String> {
    let mut entries = Vec::new();
    if root.is_dir() {
        for entry in fs::read_dir(root)
            .map_err(|error| format!("failed to read `{}`: {error}", root.display()))?
        {
            let entry = entry
                .map_err(|error| format!("failed to enumerate `{}`: {error}", root.display()))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let key = entry.file_name().to_string_lossy().to_string();
            let (file_count, total_bytes) = summarize_directory(&path)?;
            entries.push(CompileCacheInventoryEntry {
                key,
                entry_dir: path,
                file_count,
                total_bytes,
            });
        }
    }
    entries.sort_by(|lhs, rhs| lhs.key.cmp(&rhs.key));
    let entry_count = entries.len();
    let total_files = entries.iter().map(|entry| entry.file_count).sum();
    let total_bytes = entries.iter().map(|entry| entry.total_bytes).sum();
    Ok(CompileCacheInventory {
        root: root.to_path_buf(),
        entry_count,
        total_files,
        total_bytes,
        entries,
    })
}

pub fn compile_cache_inventory_summary(
    workspace_root: &Path,
) -> Result<CompileCacheInventorySummary, String> {
    let roots = discover_compile_cache_roots(workspace_root)?
        .into_iter()
        .map(|root| compile_cache_inventory(&root))
        .collect::<Result<Vec<_>, _>>()?;
    let total_entries = roots.iter().map(|inventory| inventory.entry_count).sum();
    let total_files = roots.iter().map(|inventory| inventory.total_files).sum();
    let total_bytes = roots.iter().map(|inventory| inventory.total_bytes).sum();
    Ok(CompileCacheInventorySummary {
        workspace_root: workspace_root.to_path_buf(),
        roots,
        total_entries,
        total_files,
        total_bytes,
    })
}

fn cache_root(input: &Path, project: Option<&LoadedProject>) -> PathBuf {
    if let Some(project) = project {
        return project.root.join(".nuis").join("cache").join("compile");
    }
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("target")
        .join("nuisc-cache")
        .join(sanitize_path_label(
            input
                .file_stem()
                .or_else(|| input.file_name())
                .and_then(|item| item.to_str())
                .unwrap_or("input"),
        ))
}

fn collect_registry_manifest_paths(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    if !root.exists() {
        return Ok(paths);
    }
    for entry in fs::read_dir(root)
        .map_err(|error| format!("failed to read `{}`: {error}", root.display()))?
    {
        let entry =
            entry.map_err(|error| format!("failed to enumerate `{}`: {error}", root.display()))?;
        let path = entry.path();
        if path.is_dir() {
            paths.extend(collect_registry_manifest_paths(&path)?);
            continue;
        }
        if path.extension().and_then(|item| item.to_str()) == Some("toml") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn fingerprint_records(records: &[(String, Vec<u8>)]) -> String {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for (label, bytes) in records {
        for byte in label.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(PRIME);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(PRIME);
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(PRIME);
        }
        hash ^= 0x00;
        hash = hash.wrapping_mul(PRIME);
    }
    format!("{hash:016x}")
}

fn copy_directory_recursive(source: &Path, target: &Path) -> Result<(), String> {
    fs::create_dir_all(target)
        .map_err(|error| format!("failed to create `{}`: {error}", target.display()))?;
    for entry in fs::read_dir(source)
        .map_err(|error| format!("failed to read `{}`: {error}", source.display()))?
    {
        let entry = entry
            .map_err(|error| format!("failed to enumerate `{}`: {error}", source.display()))?;
        let from = entry.path();
        let to = target.join(entry.file_name());
        if from.is_dir() {
            copy_directory_recursive(&from, &to)?;
        } else {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("failed to create `{}`: {error}", parent.display()))?;
            }
            fs::copy(&from, &to).map_err(|error| {
                format!(
                    "failed to copy `{}` -> `{}`: {error}",
                    from.display(),
                    to.display()
                )
            })?;
        }
    }
    Ok(())
}

fn summarize_cache_entries(root: &Path) -> Result<(usize, u64), String> {
    let mut entries = 0usize;
    let mut bytes = 0u64;
    for entry in fs::read_dir(root)
        .map_err(|error| format!("failed to read `{}`: {error}", root.display()))?
    {
        let entry =
            entry.map_err(|error| format!("failed to enumerate `{}`: {error}", root.display()))?;
        let path = entry.path();
        if path.is_dir() {
            entries += 1;
            let (_, entry_bytes) = summarize_directory(&path)?;
            bytes += entry_bytes;
        }
    }
    Ok((entries, bytes))
}

#[derive(Debug, Clone)]
struct CacheEntryStats {
    key: String,
    entry_dir: PathBuf,
    total_bytes: u64,
    modified: SystemTime,
}

fn collect_cache_entry_stats(root: &Path) -> Result<Vec<CacheEntryStats>, String> {
    let mut entries = Vec::new();
    if !root.is_dir() {
        return Ok(entries);
    }
    for entry in
        fs::read_dir(root).map_err(|error| format!("failed to read `{}`: {error}", root.display()))?
    {
        let entry =
            entry.map_err(|error| format!("failed to enumerate `{}`: {error}", root.display()))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let (_, total_bytes) = summarize_directory(&path)?;
        let modified = fs::metadata(&path)
            .and_then(|metadata| metadata.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        entries.push(CacheEntryStats {
            key: entry.file_name().to_string_lossy().to_string(),
            entry_dir: path,
            total_bytes,
            modified,
        });
    }
    Ok(entries)
}

fn summarize_directory(root: &Path) -> Result<(usize, u64), String> {
    let mut file_count = 0usize;
    let mut total_bytes = 0u64;
    for entry in fs::read_dir(root)
        .map_err(|error| format!("failed to read `{}`: {error}", root.display()))?
    {
        let entry =
            entry.map_err(|error| format!("failed to enumerate `{}`: {error}", root.display()))?;
        let path = entry.path();
        if path.is_dir() {
            let (nested_files, nested_bytes) = summarize_directory(&path)?;
            file_count += nested_files;
            total_bytes += nested_bytes;
        } else {
            let metadata = fs::metadata(&path)
                .map_err(|error| format!("failed to stat `{}`: {error}", path.display()))?;
            file_count += 1;
            total_bytes += metadata.len();
        }
    }
    Ok((file_count, total_bytes))
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

fn discover_compile_cache_roots(workspace_root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut roots = Vec::new();
    let single_file_root = workspace_root.join("target").join("nuisc-cache");
    if single_file_root.is_dir() {
        roots.push(single_file_root);
    }
    discover_project_cache_roots(workspace_root, &mut roots)?;
    roots.sort();
    roots.dedup();
    Ok(roots)
}

fn discover_project_cache_roots(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if !root.is_dir() {
        return Ok(());
    }
    for entry in
        fs::read_dir(root).map_err(|error| format!("failed to read `{}`: {error}", root.display()))?
    {
        let entry =
            entry.map_err(|error| format!("failed to enumerate `{}`: {error}", root.display()))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if entry.file_name() == ".git" {
            continue;
        }
        if entry.file_name() == ".nuis" {
            let compile_root = path.join("cache").join("compile");
            if compile_root.is_dir() {
                out.push(compile_root);
            }
            continue;
        }
        discover_project_cache_roots(&path, out)?;
    }
    Ok(())
}
