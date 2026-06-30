use super::*;

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

pub(super) fn clean_compile_cache_root(root: &Path) -> Result<CleanedCompileCache, String> {
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

pub(super) fn prune_compile_cache_root(
    root: &Path,
    keep: usize,
) -> Result<PrunedCompileCache, String> {
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
