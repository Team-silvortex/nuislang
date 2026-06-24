use std::{
    fmt, fs,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::project::{LoadedProject, ProjectCompilationPlan};

const COMPILE_CACHE_EPOCH: &str = "2026-06-22-aot-streaming-cache-fingerprint-v1";

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
    compute_compile_cache_key_with_plan(input, project, None)
}

pub fn compute_compile_cache_key_with_plan(
    input: &Path,
    project: Option<&LoadedProject>,
    plan: Option<&ProjectCompilationPlan>,
) -> Result<CompileCacheKey, String> {
    let root = cache_root(input, project);
    let mut records = Vec::<CacheFingerprintRecord<'_>>::new();

    records.push(CacheFingerprintRecord::inline_bytes(
        "toolchain.nuisc.version",
        env!("CARGO_PKG_VERSION").as_bytes().to_vec(),
    ));
    records.push(CacheFingerprintRecord::inline_bytes(
        "toolchain.nuisc.cache_epoch",
        COMPILE_CACHE_EPOCH.as_bytes().to_vec(),
    ));
    records.push(CacheFingerprintRecord::inline_bytes(
        "toolchain.engine.version",
        crate::engine::default_engine().version.as_bytes().to_vec(),
    ));
    records.push(CacheFingerprintRecord::inline_bytes(
        "toolchain.engine.profile",
        crate::engine::default_engine().profile.as_bytes().to_vec(),
    ));

    if let Some(project) = project {
        if let Some(plan) = plan {
            records.push(CacheFingerprintRecord::project_plan("project.plan", plan));
        }
        records.push(CacheFingerprintRecord::file_path(
            "project.manifest",
            project.manifest_path.clone(),
        ));
        for module in &project.modules {
            let relative = module
                .path
                .strip_prefix(&project.root)
                .unwrap_or(module.path.as_path())
                .display()
                .to_string();
            records.push(CacheFingerprintRecord::file_path(
                format!("project.module:{relative}"),
                module.path.clone(),
            ));
        }
        let lock_path = project.root.join("nuis.galaxy.lock");
        if lock_path.exists() {
            records.push(CacheFingerprintRecord::file_path(
                "project.galaxy_lock",
                lock_path,
            ));
        }
    } else {
        records.push(CacheFingerprintRecord::file_path(
            format!("source:{}", input.display()),
            input.to_path_buf(),
        ));
    }

    for registry_path in collect_registry_manifest_paths(Path::new("nustar-packages"))? {
        let relative = registry_path.display().to_string();
        records.push(CacheFingerprintRecord::file_path(
            format!("registry:{relative}"),
            registry_path,
        ));
    }

    records.sort_by(|lhs, rhs| lhs.label.cmp(&rhs.label));
    let input_labels = records.iter().map(|record| record.label.clone()).collect();
    let key = fingerprint_records(&records)?;
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
    if entry_dir.exists() {
        if entry_dir.is_dir() {
            fs::remove_dir_all(&entry_dir).map_err(|error| {
                format!(
                    "failed to refresh compile cache entry `{}`: {error}",
                    entry_dir.display()
                )
            })?;
        } else {
            fs::remove_file(&entry_dir).map_err(|error| {
                format!(
                    "failed to reset compile cache entry `{}`: {error}",
                    entry_dir.display()
                )
            })?;
        }
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
    compile_cache_status_with_plan(input, project, None)
}

pub fn compile_cache_status_with_plan(
    input: &Path,
    project: Option<&LoadedProject>,
    plan: Option<&ProjectCompilationPlan>,
) -> Result<CompileCacheStatus, String> {
    let key = compute_compile_cache_key_with_plan(input, project, plan)?;
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
    clean_compile_cache_with_plan(input, project, None)
}

pub fn clean_compile_cache_with_plan(
    input: &Path,
    project: Option<&LoadedProject>,
    _plan: Option<&ProjectCompilationPlan>,
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
    prune_compile_cache_with_plan(input, project, None, keep)
}

pub fn prune_compile_cache_with_plan(
    input: &Path,
    project: Option<&LoadedProject>,
    _plan: Option<&ProjectCompilationPlan>,
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
    let removed_entries = pruned_roots
        .iter()
        .map(|pruned| pruned.removed_entries)
        .sum();
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
    entries.sort_by(|lhs, rhs| {
        rhs.modified
            .cmp(&lhs.modified)
            .then_with(|| lhs.key.cmp(&rhs.key))
    });
    let kept_entries = entries.len().min(keep);
    let mut removed_entries = 0usize;
    let mut removed_bytes = 0u64;
    for entry in entries.into_iter().skip(keep) {
        fs::remove_dir_all(&entry.entry_dir)
            .map_err(|error| format!("failed to prune `{}`: {error}", entry.entry_dir.display()))?;
        removed_entries += 1;
        removed_bytes += entry.total_bytes;
    }
    if keep == 0
        && root.is_dir()
        && fs::read_dir(root)
            .map_err(|error| format!("failed to read `{}`: {error}", root.display()))?
            .next()
            .is_none()
    {
        fs::remove_dir(root).map_err(|error| {
            format!(
                "failed to remove empty cache root `{}`: {error}",
                root.display()
            )
        })?;
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct CacheFingerprintRecord<'a> {
    label: String,
    source: CacheFingerprintSource<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CacheFingerprintSource<'a> {
    Inline(Vec<u8>),
    File(PathBuf),
    ProjectPlan(&'a ProjectCompilationPlan),
}

impl<'a> CacheFingerprintRecord<'a> {
    fn inline_bytes(label: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            label: label.into(),
            source: CacheFingerprintSource::Inline(bytes),
        }
    }

    fn file_path(label: impl Into<String>, path: PathBuf) -> Self {
        Self {
            label: label.into(),
            source: CacheFingerprintSource::File(path),
        }
    }

    fn project_plan(label: impl Into<String>, plan: &'a ProjectCompilationPlan) -> Self {
        Self {
            label: label.into(),
            source: CacheFingerprintSource::ProjectPlan(plan),
        }
    }
}

struct FingerprintState {
    hash: u64,
}

impl FingerprintState {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    fn new() -> Self {
        Self { hash: Self::OFFSET }
    }

    fn update_byte(&mut self, byte: u8) {
        self.hash ^= u64::from(byte);
        self.hash = self.hash.wrapping_mul(Self::PRIME);
    }

    fn update_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.update_byte(*byte);
        }
    }

    fn update_record_boundary(&mut self, byte: u8) {
        self.update_byte(byte);
    }

    fn finish(self) -> String {
        format!("{:016x}", self.hash)
    }
}

fn fingerprint_records(records: &[CacheFingerprintRecord<'_>]) -> Result<String, String> {
    let mut state = FingerprintState::new();
    for record in records {
        state.update_bytes(record.label.as_bytes());
        state.update_record_boundary(0xff);
        match &record.source {
            CacheFingerprintSource::Inline(bytes) => state.update_bytes(bytes),
            CacheFingerprintSource::File(path) => fingerprint_file(path, &mut state)?,
            CacheFingerprintSource::ProjectPlan(plan) => {
                fingerprint_project_plan(plan, &mut state)?
            }
        }
        state.update_record_boundary(0x00);
    }
    Ok(state.finish())
}

fn fingerprint_file(path: &Path, state: &mut FingerprintState) -> Result<(), String> {
    let mut file = File::open(path).map_err(|error| {
        format!(
            "failed to read `{}` for compile cache: {error}",
            path.display()
        )
    })?;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| {
            format!(
                "failed to read `{}` for compile cache: {error}",
                path.display()
            )
        })?;
        if read == 0 {
            break;
        }
        state.update_bytes(&buffer[..read]);
    }
    Ok(())
}

struct FingerprintFmtWriter<'a> {
    state: &'a mut FingerprintState,
}

impl fmt::Write for FingerprintFmtWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.state.update_bytes(s.as_bytes());
        Ok(())
    }
}

fn fingerprint_project_plan(
    plan: &ProjectCompilationPlan,
    state: &mut FingerprintState,
) -> Result<(), String> {
    let mut writer = FingerprintFmtWriter { state };
    crate::project::write_project_compilation_plan_index(&mut writer, plan)
        .map_err(|error| format!("failed to fingerprint project plan index: {error}"))
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
    for entry in fs::read_dir(root)
        .map_err(|error| format!("failed to read `{}`: {error}", root.display()))?
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
    for entry in fs::read_dir(root)
        .map_err(|error| format!("failed to read `{}`: {error}", root.display()))?
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("nuisc_cache_{label}_{nonce}"))
    }

    #[test]
    fn fingerprint_records_is_stable_across_record_order_after_sorting() {
        let temp_dir = temp_path("fingerprint_order");
        fs::create_dir_all(&temp_dir).unwrap();
        let alpha = temp_dir.join("alpha.txt");
        let beta = temp_dir.join("beta.txt");
        fs::write(&alpha, "alpha-file").unwrap();
        fs::write(&beta, "beta-file").unwrap();

        let mut left = vec![
            CacheFingerprintRecord::file_path("b.file", beta.clone()),
            CacheFingerprintRecord::inline_bytes("a.inline", b"inline".to_vec()),
            CacheFingerprintRecord::file_path("c.file", alpha.clone()),
        ];
        let mut right = vec![
            CacheFingerprintRecord::file_path("c.file", alpha),
            CacheFingerprintRecord::file_path("b.file", beta),
            CacheFingerprintRecord::inline_bytes("a.inline", b"inline".to_vec()),
        ];
        left.sort_by(|lhs, rhs| lhs.label.cmp(&rhs.label));
        right.sort_by(|lhs, rhs| lhs.label.cmp(&rhs.label));

        let left_hash = fingerprint_records(&left).unwrap();
        let right_hash = fingerprint_records(&right).unwrap();
        assert_eq!(left_hash, right_hash);

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn store_compile_cache_refreshes_existing_entry_contents() {
        let temp_dir = temp_path("store_refresh");
        let cache_root = temp_dir.join("cache");
        let output_dir = temp_dir.join("out");
        fs::create_dir_all(&output_dir).unwrap();
        fs::write(output_dir.join("marker.txt"), "first").unwrap();

        let key = CompileCacheKey {
            root: cache_root,
            key: "demo-key".to_owned(),
            input_labels: vec!["demo".to_owned()],
        };

        let entry = store_compile_cache(&key, &output_dir).unwrap();
        assert_eq!(
            fs::read_to_string(entry.entry_dir.join("marker.txt")).unwrap(),
            "first"
        );

        fs::write(output_dir.join("marker.txt"), "second").unwrap();
        let entry = store_compile_cache(&key, &output_dir).unwrap();
        assert_eq!(
            fs::read_to_string(entry.entry_dir.join("marker.txt")).unwrap(),
            "second"
        );

        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
