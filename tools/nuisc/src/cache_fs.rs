use super::*;

pub(super) fn copy_directory_recursive(source: &Path, target: &Path) -> Result<(), String> {
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

pub(super) fn summarize_cache_entries(root: &Path) -> Result<(usize, u64), String> {
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
pub(super) struct CacheEntryStats {
    pub(super) key: String,
    pub(super) entry_dir: PathBuf,
    pub(super) total_bytes: u64,
    pub(super) modified: SystemTime,
}

pub(super) fn collect_cache_entry_stats(root: &Path) -> Result<Vec<CacheEntryStats>, String> {
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

pub(super) fn summarize_directory(root: &Path) -> Result<(usize, u64), String> {
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

pub(super) fn sanitize_path_label(raw: &str) -> String {
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

pub(super) fn discover_compile_cache_roots(workspace_root: &Path) -> Result<Vec<PathBuf>, String> {
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

pub(super) fn discover_project_cache_roots(
    root: &Path,
    out: &mut Vec<PathBuf>,
) -> Result<(), String> {
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
