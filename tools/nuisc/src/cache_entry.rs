use super::*;

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
