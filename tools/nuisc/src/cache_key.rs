use super::*;

pub fn compute_compile_cache_key(
    input: &Path,
    project: Option<&LoadedProject>,
) -> Result<CompileCacheKey, String> {
    compute_compile_cache_key_with_plan_and_identity(input, project, None, None)
}

pub fn compute_compile_cache_key_with_identity(
    input: &Path,
    project: Option<&LoadedProject>,
    identity: &str,
) -> Result<CompileCacheKey, String> {
    compute_compile_cache_key_with_plan_and_identity(input, project, None, Some(identity))
}

pub fn compute_compile_cache_key_with_plan(
    input: &Path,
    project: Option<&LoadedProject>,
    plan: Option<&ProjectCompilationPlan>,
) -> Result<CompileCacheKey, String> {
    compute_compile_cache_key_with_plan_and_identity(input, project, plan, None)
}

fn compute_compile_cache_key_with_plan_and_identity(
    input: &Path,
    project: Option<&LoadedProject>,
    plan: Option<&ProjectCompilationPlan>,
    identity: Option<&str>,
) -> Result<CompileCacheKey, String> {
    let root = cache_root(input, project);
    let mut records = vec![
        CacheFingerprintRecord::inline_bytes(
            "toolchain.nuisc.version",
            env!("CARGO_PKG_VERSION").as_bytes().to_vec(),
        ),
        CacheFingerprintRecord::inline_bytes(
            "toolchain.nuisc.cache_epoch",
            COMPILE_CACHE_EPOCH.as_bytes().to_vec(),
        ),
        CacheFingerprintRecord::inline_bytes(
            "toolchain.engine.version",
            crate::engine::default_engine().version.as_bytes().to_vec(),
        ),
        CacheFingerprintRecord::inline_bytes(
            "toolchain.engine.profile",
            crate::engine::default_engine().profile.as_bytes().to_vec(),
        ),
    ];
    if let Some(identity) = identity {
        if identity.is_empty() {
            return Err("compile cache identity cannot be empty".to_owned());
        }
        records.push(CacheFingerprintRecord::inline_bytes(
            "compile.cache.identity",
            identity.as_bytes().to_vec(),
        ));
    }

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

pub(super) fn cache_root(input: &Path, project: Option<&LoadedProject>) -> PathBuf {
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

pub(super) fn collect_registry_manifest_paths(root: &Path) -> Result<Vec<PathBuf>, String> {
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
