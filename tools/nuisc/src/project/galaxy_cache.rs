use std::fmt::Write as _;
use std::fs;
use std::path::{Component, Path, PathBuf};

use super::{
    verify_project_galaxy_resolution_lock, verify_project_galaxy_resolution_lock_source,
    LoadedProject, ProjectGalaxyResolutionLockSummary,
};

pub const PROJECT_GALAXY_CACHE_SCHEMA: &str = "nuis-galaxy-resolution-cache-v1";
pub const PROJECT_GALAXY_CACHE_MANIFEST_FILE: &str = "nuis.galaxy.cache.toml";
pub const PROJECT_GALAXY_CACHE_DIGEST_DIR: &str = "sha256";

pub fn project_galaxy_cache_base(project_root: &Path) -> PathBuf {
    project_root.join(".nuis").join("deps").join("galaxy")
}

pub fn project_galaxy_cache_root(
    project_root: &Path,
    summary: &ProjectGalaxyResolutionLockSummary,
) -> Result<PathBuf, String> {
    project_galaxy_cache_root_under(&project_galaxy_cache_base(project_root), summary)
}

pub fn project_galaxy_cache_root_under(
    cache_base: &Path,
    summary: &ProjectGalaxyResolutionLockSummary,
) -> Result<PathBuf, String> {
    let digest = resolution_digest_hex(summary)?;
    Ok(cache_base
        .join(PROJECT_GALAXY_CACHE_DIGEST_DIR)
        .join(digest))
}

pub fn materialize_project_galaxy_cache(
    project: &LoadedProject,
    lock_source: &str,
    lock_path: &Path,
    cache_root: &Path,
) -> Result<ProjectGalaxyResolutionLockSummary, String> {
    let summary = verify_project_galaxy_resolution_lock(project, lock_source, lock_path)?;
    let expected_tail =
        Path::new(PROJECT_GALAXY_CACHE_DIGEST_DIR).join(resolution_digest_hex(&summary)?);
    if !cache_root.ends_with(&expected_tail) {
        return Err(format!(
            "Galaxy resolution cache `{}` is not addressed by `{}`",
            cache_root.display(),
            summary.resolution_sha256
        ));
    }
    fs::create_dir_all(cache_root).map_err(|error| {
        format!(
            "failed to create Galaxy resolution cache `{}`: {error}",
            cache_root.display()
        )
    })?;
    fs::write(
        cache_root.join(PROJECT_GALAXY_CACHE_MANIFEST_FILE),
        render_cache_manifest(&summary),
    )
    .map_err(|error| format!("failed to write Galaxy cache manifest: {error}"))?;
    fs::write(
        cache_root.join("index.toml"),
        render_cache_provider_index(project)?,
    )
    .map_err(|error| format!("failed to write Galaxy cache provider index: {error}"))?;

    for dependency in &project.resolved_galaxies {
        validate_cache_component("dependency name", &dependency.name)?;
        validate_cache_component("dependency version", &dependency.version)?;
        let output = cache_root.join(&dependency.name).join(&dependency.version);
        fs::create_dir_all(&output)
            .map_err(|error| format!("failed to create `{}`: {error}", output.display()))?;
        write_verified_content(
            &dependency.manifest_path,
            &dependency.manifest_content_identity,
            &output,
        )?;
        for (source, identity) in dependency
            .resolved_source_paths
            .iter()
            .zip(&dependency.source_content_identities)
            .chain(
                dependency
                    .resolved_library_paths
                    .iter()
                    .zip(&dependency.library_content_identities),
            )
        {
            write_verified_content(source, identity, &output)?;
        }
    }
    fs::write(
        cache_root.join(super::PROJECT_GALAXY_RESOLUTION_LOCK_FILE),
        lock_source,
    )
    .map_err(|error| format!("failed to write Galaxy cache lock copy: {error}"))?;
    Ok(summary)
}

pub fn verify_required_project_galaxy_resolution_cache(
    input: &Path,
) -> Result<ProjectGalaxyResolutionLockSummary, String> {
    let project_root = project_root_for_input(input)?;
    let lock_path = project_root.join(super::PROJECT_GALAXY_RESOLUTION_LOCK_FILE);
    if !lock_path.is_file() {
        return Err(format!(
            "release Galaxy lock policy requires `{}`; run `nuis galaxy lock-deps <project-dir>` and `nuis galaxy sync-deps <project-dir>`",
            lock_path.display()
        ));
    }
    let lock_source = read_lock_source(&lock_path)?;
    verify_locked_project_galaxy_cache(&project_root, &lock_source, &lock_path)
        .map(|(_, summary)| summary)
}

pub(super) fn locked_project_galaxy_cache(
    project_root: &Path,
) -> Result<Option<(PathBuf, PathBuf, String, ProjectGalaxyResolutionLockSummary)>, String> {
    let lock_path = project_root.join(super::PROJECT_GALAXY_RESOLUTION_LOCK_FILE);
    if !lock_path.exists() {
        return Ok(None);
    }
    let lock_source = read_lock_source(&lock_path)?;
    let (cache_root, summary) =
        verify_locked_project_galaxy_cache(project_root, &lock_source, &lock_path)?;
    Ok(Some((cache_root, lock_path, lock_source, summary)))
}

pub(super) fn verify_resolved_galaxy_cache_paths(
    cache_root: &Path,
    dependencies: &[crate::stdlib_registry::ResolvedGalaxyDependency],
) -> Result<(), String> {
    let canonical_root = canonical_cache_path(cache_root, "resolution cache root")?;
    for dependency in dependencies {
        let mut paths = Vec::with_capacity(
            2 + dependency.resolved_source_paths.len() + dependency.resolved_library_paths.len(),
        );
        paths.push((&dependency.module_dir, "package root"));
        paths.push((&dependency.manifest_path, "package manifest"));
        paths.extend(
            dependency
                .resolved_source_paths
                .iter()
                .map(|path| (path, "source module")),
        );
        paths.extend(
            dependency
                .resolved_library_paths
                .iter()
                .map(|path| (path, "library module")),
        );
        for (path, kind) in paths {
            let canonical = canonical_cache_path(path, kind)?;
            if !canonical.starts_with(&canonical_root) {
                return Err(format!(
                    "locked Galaxy {kind} `{}` escapes resolution cache `{}`",
                    path.display(),
                    cache_root.display()
                ));
            }
        }
    }
    Ok(())
}

fn verify_locked_project_galaxy_cache(
    project_root: &Path,
    lock_source: &str,
    lock_path: &Path,
) -> Result<(PathBuf, ProjectGalaxyResolutionLockSummary), String> {
    let summary = verify_project_galaxy_resolution_lock_source(lock_source, lock_path)?;
    let cache_root = project_galaxy_cache_root(project_root, &summary)?;
    if !cache_root.is_dir() {
        return Err(format!(
            "locked Galaxy resolution cache `{}` is missing; run `nuis galaxy sync-deps <project-dir>`",
            cache_root.display()
        ));
    }
    verify_cache_member_path(
        &cache_root,
        &cache_root.join(PROJECT_GALAXY_CACHE_MANIFEST_FILE),
        "cache manifest",
    )?;
    verify_cache_manifest(&cache_root, &summary)?;
    let cache_index_path = cache_root.join("index.toml");
    if !cache_index_path.is_file() {
        return Err(format!(
            "locked Galaxy resolution cache `{}` is missing its provider index",
            cache_root.display()
        ));
    }
    verify_cache_member_path(&cache_root, &cache_index_path, "provider index")?;
    let cache_lock_path = cache_root.join(super::PROJECT_GALAXY_RESOLUTION_LOCK_FILE);
    verify_cache_member_path(&cache_root, &cache_lock_path, "lock copy")?;
    let cache_lock_source = read_lock_source(&cache_lock_path)?;
    if cache_lock_source != lock_source {
        return Err(format!(
            "Galaxy cache lock `{}` does not match committed lock `{}`",
            cache_lock_path.display(),
            lock_path.display()
        ));
    }
    Ok((cache_root, summary))
}

fn render_cache_manifest(summary: &ProjectGalaxyResolutionLockSummary) -> String {
    format!(
        "cache_schema = \"{PROJECT_GALAXY_CACHE_SCHEMA}\"\nresolution_sha256 = \"{}\"\ndependency_count = {}\nprovider_index = \"index.toml\"\nlock_file = \"{}\"\n",
        summary.resolution_sha256,
        summary.dependencies,
        super::PROJECT_GALAXY_RESOLUTION_LOCK_FILE,
    )
}

fn verify_cache_manifest(
    cache_root: &Path,
    summary: &ProjectGalaxyResolutionLockSummary,
) -> Result<(), String> {
    let path = cache_root.join(PROJECT_GALAXY_CACHE_MANIFEST_FILE);
    let source = fs::read_to_string(&path).map_err(|error| {
        format!(
            "failed to read Galaxy cache manifest `{}`: {error}",
            path.display()
        )
    })?;
    let schema = crate::aot_toml::parse_required_toml_string(&source, "cache_schema", &path)?;
    let resolution =
        crate::aot_toml::parse_required_toml_string(&source, "resolution_sha256", &path)?;
    let dependencies =
        crate::aot_toml::parse_required_toml_usize(&source, "dependency_count", &path)?;
    if schema != PROJECT_GALAXY_CACHE_SCHEMA
        || resolution != summary.resolution_sha256
        || dependencies != summary.dependencies
        || source != render_cache_manifest(summary)
    {
        return Err(format!(
            "Galaxy cache manifest `{}` does not match locked resolution `{}`",
            path.display(),
            summary.resolution_sha256
        ));
    }
    Ok(())
}

fn render_cache_provider_index(project: &LoadedProject) -> Result<String, String> {
    let mut dependencies = project.resolved_galaxies.iter().collect::<Vec<_>>();
    dependencies.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));
    let names = dependencies
        .iter()
        .map(|dependency| dependency.name.clone())
        .collect::<Vec<_>>();
    let default_entry = names.first().map(String::as_str).unwrap_or("empty");
    let mut source = String::new();
    writeln!(source, "layout_schema = \"nuis-stdlib-layout-v1\"").unwrap();
    writeln!(source, "name = \"nuis-locked-galaxy-resolution\"").unwrap();
    writeln!(
        source,
        "default_entry = \"{}\"",
        crate::aot_toml::escape_toml_string(default_entry)
    )
    .unwrap();
    writeln!(
        source,
        "modules = {}",
        crate::aot_toml::render_string_array(&names)
    )
    .unwrap();
    for dependency in dependencies {
        validate_cache_component("dependency name", &dependency.name)?;
        validate_cache_component("dependency version", &dependency.version)?;
        let mut depends_on = dependency.depends_on.clone();
        depends_on.sort();
        depends_on.dedup();
        source.push_str("\n[[module]]\n");
        writeln!(
            source,
            "name = \"{}\"",
            crate::aot_toml::escape_toml_string(&dependency.name)
        )
        .unwrap();
        writeln!(source, "kind = \"locked\"").unwrap();
        writeln!(
            source,
            "path = \"{}/{}\"",
            dependency.name, dependency.version
        )
        .unwrap();
        writeln!(
            source,
            "package_id = \"{}\"",
            crate::aot_toml::escape_toml_string(&dependency.package_id)
        )
        .unwrap();
        writeln!(
            source,
            "depends_on = {}",
            crate::aot_toml::render_string_array(&depends_on)
        )
        .unwrap();
        writeln!(
            source,
            "summary = \"Materialized from one verified Galaxy resolution lock.\""
        )
        .unwrap();
    }
    Ok(source)
}

fn write_verified_content(
    source_path: &Path,
    identity: &crate::stdlib_registry::ResolvedGalaxyContentIdentity,
    output: &Path,
) -> Result<(), String> {
    validate_materialized_relative_path(&identity.logical_path)?;
    let source = crate::stdlib_registry::read_verified_galaxy_text(source_path, identity)?;
    let destination = output.join(&identity.logical_path);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create `{}`: {error}", parent.display()))?;
    }
    fs::write(&destination, source.as_bytes())
        .map_err(|error| format!("failed to write `{}`: {error}", destination.display()))
}

fn read_lock_source(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read Galaxy resolution lock `{}`: {error}",
            path.display()
        )
    })
}

fn canonical_cache_path(path: &Path, kind: &str) -> Result<PathBuf, String> {
    fs::canonicalize(path).map_err(|error| {
        format!(
            "failed to resolve Galaxy cache {kind} `{}`: {error}",
            path.display()
        )
    })
}

fn verify_cache_member_path(cache_root: &Path, path: &Path, kind: &str) -> Result<(), String> {
    let canonical_root = canonical_cache_path(cache_root, "resolution cache root")?;
    let canonical = canonical_cache_path(path, kind)?;
    if !canonical.starts_with(&canonical_root) {
        return Err(format!(
            "locked Galaxy {kind} `{}` escapes resolution cache `{}`",
            path.display(),
            cache_root.display()
        ));
    }
    Ok(())
}

fn project_root_for_input(input: &Path) -> Result<PathBuf, String> {
    let manifest_path = if input.is_dir() {
        input.join("nuis.toml")
    } else {
        input.to_path_buf()
    };
    manifest_path
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            format!(
                "project manifest `{}` has no parent directory",
                manifest_path.display()
            )
        })
}

fn resolution_digest_hex(summary: &ProjectGalaxyResolutionLockSummary) -> Result<&str, String> {
    let value = summary
        .resolution_sha256
        .strip_prefix("sha256:")
        .unwrap_or_default();
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!(
            "Galaxy resolution digest `{}` cannot address the project cache",
            summary.resolution_sha256
        ));
    }
    Ok(value)
}

fn validate_cache_component(label: &str, value: &str) -> Result<(), String> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {
        return Err(format!(
            "Galaxy cache {label} `{value}` is not a canonical path component"
        ));
    }
    Ok(())
}

fn validate_materialized_relative_path(path: &str) -> Result<(), String> {
    let candidate = Path::new(path);
    if path.is_empty()
        || path.contains('\\')
        || candidate.is_absolute()
        || candidate
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!(
            "Galaxy materialization path `{path}` is not a canonical portable relative path"
        ));
    }
    Ok(())
}
