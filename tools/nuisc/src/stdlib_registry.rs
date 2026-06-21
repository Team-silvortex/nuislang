use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StdlibLibraryImportPolicy {
    ProjectAuto,
    ManualOnly,
}

impl StdlibLibraryImportPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProjectAuto => "project-auto",
            Self::ManualOnly => "manual-only",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdlibLayout {
    pub name: String,
    pub default_entry: String,
    pub modules: Vec<StdlibIndexModule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdlibIndexModule {
    pub name: String,
    pub kind: String,
    pub path: String,
    pub package_id: String,
    pub depends_on: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdlibModuleManifest {
    pub name: String,
    pub package_id: String,
    pub tier: String,
    pub depends_on: Vec<String>,
    pub summary: String,
    pub surfaces: Vec<String>,
    pub source_modules: Vec<String>,
    pub library_modules: Vec<String>,
    pub library_import_policy: StdlibLibraryImportPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedGalaxyDependency {
    pub name: String,
    pub version: String,
    pub package_id: String,
    pub direct: bool,
    pub requested_by: Vec<String>,
    pub module_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub depends_on: Vec<String>,
    pub surfaces: Vec<String>,
    pub source_modules: Vec<String>,
    pub resolved_source_paths: Vec<PathBuf>,
    pub library_modules: Vec<String>,
    pub resolved_library_paths: Vec<PathBuf>,
    pub library_import_policy: StdlibLibraryImportPolicy,
    pub auto_injectable: bool,
    pub auto_inject_blockers: Vec<String>,
}

pub fn load_stdlib_layout(stdlib_root: &Path) -> Result<StdlibLayout, String> {
    let path = stdlib_root.join("index.toml");
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read stdlib layout `{}`: {error}", path.display()))?;
    let name = parse_required_string(&source, "name", &path)?;
    let default_entry = parse_required_string(&source, "default_entry", &path)?;
    let modules = parse_stdlib_index_modules(&source, &path)?;
    Ok(StdlibLayout {
        name,
        default_entry,
        modules,
    })
}

pub fn resolve_stdlib_root() -> Result<PathBuf, String> {
    let cwd_candidate = std::env::current_dir()
        .ok()
        .map(|dir| dir.join("stdlib"))
        .filter(|path| path.join("index.toml").exists());
    if let Some(path) = cwd_candidate {
        return Ok(path);
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_candidate = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .map(|path| path.join("stdlib"))
        .filter(|path| path.join("index.toml").exists());
    if let Some(path) = repo_candidate {
        return Ok(path);
    }

    Err(
        "failed to locate `stdlib/index.toml`; expected it under the current working directory or the repository root"
            .to_owned(),
    )
}

pub fn load_stdlib_module_manifest(
    stdlib_root: &Path,
    module_path: &str,
) -> Result<StdlibModuleManifest, String> {
    let path = stdlib_root.join(module_path).join("module.toml");
    let source = fs::read_to_string(&path).map_err(|error| {
        format!(
            "failed to read stdlib module manifest `{}`: {error}",
            path.display()
        )
    })?;
    Ok(StdlibModuleManifest {
        name: parse_required_string(&source, "name", &path)?,
        package_id: parse_required_string(&source, "package_id", &path)?,
        tier: parse_required_string(&source, "tier", &path)?,
        depends_on: parse_optional_string_array(&source, "depends_on").unwrap_or_default(),
        summary: parse_required_string(&source, "summary", &path)?,
        surfaces: parse_optional_string_array(&source, "surfaces").unwrap_or_default(),
        source_modules: parse_optional_string_array(&source, "source_modules").unwrap_or_default(),
        library_modules: parse_optional_string_array(&source, "library_modules").unwrap_or_default(),
        library_import_policy: parse_library_import_policy(&source, &path)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixelmagic_manifest_exposes_canonical_surface_registry_ids() {
        let stdlib_root = resolve_stdlib_root().expect("resolve stdlib root");
        let manifest =
            load_stdlib_module_manifest(&stdlib_root, "pixelmagic").expect("load pixelmagic");
        assert_eq!(manifest.name, "pixelmagic");
        assert_eq!(
            manifest.surfaces,
            vec![
                "contract.pixelmagic.image-resource-shaping.v1".to_owned(),
                "contract.pixelmagic.texture-handoff.v1".to_owned(),
                "contract.pixelmagic.shader-facing-image-prep.v1".to_owned(),
                "surface.pixelmagic.shader.contracts.v1".to_owned(),
                "surface.pixelmagic.shader.packet-bridge.v1".to_owned(),
                "surface.pixelmagic.shader.render.v1".to_owned(),
                "surface.pixelmagic.shader.texture.v1".to_owned(),
                "surface.pixelmagic.shader.pipeline.v1".to_owned(),
            ]
        );
        assert_eq!(
            manifest.library_modules,
            vec![
                "lib/image_contracts.ns".to_owned(),
                "lib/shader_contracts.ns".to_owned(),
                "lib/packet_bridge_surface.ns".to_owned(),
                "lib/render_surface.ns".to_owned(),
                "lib/texture_surface.ns".to_owned(),
                "lib/pipeline_surface.ns".to_owned(),
            ]
        );
    }

    #[test]
    fn core_manifest_exposes_canonical_surface_registry_ids() {
        let stdlib_root = resolve_stdlib_root().expect("resolve stdlib root");
        let manifest = load_stdlib_module_manifest(&stdlib_root, "core").expect("load core");
        assert_eq!(
            manifest.surfaces,
            vec![
                "contract.core.prelude.primitive-values.v1".to_owned(),
                "contract.core.prelude.ref-ownership-conventions.v1".to_owned(),
                "contract.core.prelude.basic-math.v1".to_owned(),
                "contract.core.prelude.structural-source.v1".to_owned(),
            ]
        );
    }

    #[test]
    fn std_manifest_exposes_canonical_surface_registry_ids() {
        let stdlib_root = resolve_stdlib_root().expect("resolve stdlib root");
        let manifest = load_stdlib_module_manifest(&stdlib_root, "std").expect("load std");
        assert_eq!(
            manifest.surfaces,
            vec![
                "surface.std.collections.v1".to_owned(),
                "surface.std.host-ffi-helpers.v1".to_owned(),
                "surface.std.data-plane-helpers.v1".to_owned(),
                "surface.std.project-utility.v1".to_owned(),
            ]
        );
    }

    #[test]
    fn ns_nova_manifest_exposes_canonical_surface_registry_ids() {
        let stdlib_root = resolve_stdlib_root().expect("resolve stdlib root");
        let manifest =
            load_stdlib_module_manifest(&stdlib_root, "ns-nova").expect("load ns-nova");
        assert_eq!(
            manifest.surfaces,
            vec![
                "surface.ns-nova.renderer.v1".to_owned(),
                "surface.ns-nova.scene-frame-graph.v1".to_owned(),
                "surface.ns-nova.window-input-lifecycle.v1".to_owned(),
                "surface.ns-nova.material-shader-packaging.v1".to_owned(),
                "surface.ns-nova.gpu-ui-3d-runtime.v1".to_owned(),
            ]
        );
    }
}

pub fn resolve_galaxy_dependencies(
    stdlib_root: &Path,
    requested: &[crate::project::ProjectGalaxyDependency],
) -> Result<Vec<ResolvedGalaxyDependency>, String> {
    if requested.is_empty() {
        return Ok(vec![]);
    }
    let layout = load_stdlib_layout(stdlib_root)?;
    let entries = layout
        .modules
        .iter()
        .map(|item| (item.name.clone(), item.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut resolved = BTreeMap::<String, ResolvedGalaxyDependency>::new();
    let mut stack = requested
        .iter()
        .map(|item| (item.name.clone(), item.version.clone(), true, item.name.clone()))
        .collect::<Vec<_>>();

    while let Some((name, version, direct, requested_by)) = stack.pop() {
        let entry = entries.get(&name).ok_or_else(|| {
            format!(
                "project galaxy dependency `{}` is not declared in stdlib index `{}`",
                name,
                stdlib_root.join("index.toml").display()
            )
        })?;
        let manifest = load_stdlib_module_manifest(stdlib_root, &entry.path)?;
        let module_dir = stdlib_root.join(&entry.path);
        let resolved_source_paths = manifest
            .source_modules
            .iter()
            .map(|item| module_dir.join(item))
            .collect::<Vec<_>>();
        let resolved_library_paths = manifest
            .library_modules
            .iter()
            .map(|item| module_dir.join(item))
            .collect::<Vec<_>>();
        let (auto_injectable, auto_inject_blockers) = detect_auto_injectability(
            &resolved_library_paths,
            &manifest.library_import_policy,
        )?;

        let item = resolved
            .entry(name.clone())
            .or_insert_with(|| ResolvedGalaxyDependency {
                name: name.clone(),
                version: version.clone(),
                package_id: manifest.package_id.clone(),
                direct,
                requested_by: vec![],
                module_dir: module_dir.clone(),
                manifest_path: module_dir.join("module.toml"),
                depends_on: manifest.depends_on.clone(),
                surfaces: manifest.surfaces.clone(),
                source_modules: manifest.source_modules.clone(),
                resolved_source_paths,
                library_modules: manifest.library_modules.clone(),
                resolved_library_paths,
                library_import_policy: manifest.library_import_policy.clone(),
                auto_injectable,
                auto_inject_blockers,
            });
        item.direct |= direct;
        if !item.requested_by.iter().any(|value| value == &requested_by) {
            item.requested_by.push(requested_by.clone());
            item.requested_by.sort();
        }

        for dependency in &manifest.depends_on {
            if !entries.contains_key(dependency) {
                return Err(format!(
                    "stdlib module `{}` depends on unknown stdlib module `{}`",
                    manifest.name, dependency
                ));
            }
            stack.push((dependency.clone(), version.clone(), false, name.clone()));
        }
    }

    let mut values = resolved.into_values().collect::<Vec<_>>();
    values.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));
    Ok(values)
}

fn detect_auto_injectability(
    source_paths: &[PathBuf],
    import_policy: &StdlibLibraryImportPolicy,
) -> Result<(bool, Vec<String>), String> {
    if source_paths.is_empty() {
        return Ok((
            false,
            vec!["module declares no library_modules for automatic project injection".to_owned()],
        ));
    }

    if matches!(import_policy, StdlibLibraryImportPolicy::ManualOnly) {
        return Ok((
            false,
            vec!["library import policy `manual-only` disables automatic project injection".to_owned()],
        ));
    }

    let mut seen = BTreeMap::<(String, String), usize>::new();
    let mut blockers = Vec::new();
    for path in source_paths {
        let source = fs::read_to_string(path).map_err(|error| {
            format!(
                "failed to read stdlib source module `{}`: {error}",
                path.display()
            )
        })?;
        let ast = crate::frontend::parse_nuis_ast(&source).map_err(|error| {
            format!(
                "failed to parse stdlib source module `{}` for galaxy resolution: {error}",
                path.display()
            )
        })?;
        *seen.entry((ast.domain, ast.unit)).or_insert(0) += 1;
    }
    for ((domain, unit), count) in seen {
        if count > 1 {
            blockers.push(format!(
                "duplicate module binding `mod {} {}` appears {} times across source_modules",
                domain, unit, count
            ));
        }
    }
    Ok((blockers.is_empty(), blockers))
}

fn parse_stdlib_index_modules(
    source: &str,
    path: &Path,
) -> Result<Vec<StdlibIndexModule>, String> {
    let blocks = split_table_array_blocks(source, "[[module]]");
    let mut modules = Vec::new();
    for block in blocks {
        modules.push(StdlibIndexModule {
            name: parse_required_string(&block, "name", path)?,
            kind: parse_required_string(&block, "kind", path)?,
            path: parse_required_string(&block, "path", path)?,
            package_id: parse_required_string(&block, "package_id", path)?,
            depends_on: parse_optional_string_array(&block, "depends_on").unwrap_or_default(),
            summary: parse_required_string(&block, "summary", path)?,
        });
    }
    Ok(modules)
}

fn split_table_array_blocks(source: &str, marker: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();
    let mut active = false;
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line == marker {
            if active && !current.is_empty() {
                blocks.push(current.join("\n"));
                current.clear();
            }
            active = true;
            continue;
        }
        if active {
            current.push(raw_line.to_owned());
        }
    }
    if active && !current.is_empty() {
        blocks.push(current.join("\n"));
    }
    blocks
}

fn parse_required_string(source: &str, key: &str, path: &Path) -> Result<String, String> {
    parse_optional_string(source, key).ok_or_else(|| {
        format!(
            "stdlib manifest `{}` is missing required field `{}`",
            path.display(),
            key
        )
    })
}

fn parse_library_import_policy(
    source: &str,
    path: &Path,
) -> Result<StdlibLibraryImportPolicy, String> {
    let Some(value) = parse_optional_string(source, "library_import_policy") else {
        return Ok(StdlibLibraryImportPolicy::ProjectAuto);
    };
    match value.as_str() {
        "project-auto" => Ok(StdlibLibraryImportPolicy::ProjectAuto),
        "manual-only" => Ok(StdlibLibraryImportPolicy::ManualOnly),
        other => Err(format!(
            "stdlib manifest `{}` declares unsupported library_import_policy `{other}`; expected `project-auto` or `manual-only`",
            path.display()
        )),
    }
}

fn parse_optional_string(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_quoted(rest);
        }
    }
    None
}

fn parse_optional_string_array(source: &str, key: &str) -> Option<Vec<String>> {
    let prefix = format!("{key} = ");
    let mut lines = source.lines();
    while let Some(raw_line) = lines.next() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let mut collected = rest.trim().to_owned();
            if !collected.contains(']') {
                for next_line in lines.by_ref() {
                    collected.push(' ');
                    collected.push_str(next_line.trim());
                    if next_line.contains(']') {
                        break;
                    }
                }
            }
            let body = collected.trim();
            let body = body.strip_prefix('[')?.strip_suffix(']')?;
            let mut values = Vec::new();
            for item in body.split(',') {
                let item = item.trim();
                if item.is_empty() {
                    continue;
                }
                values.push(parse_quoted(item)?);
            }
            return Some(values);
        }
    }
    None
}

fn parse_quoted(raw: &str) -> Option<String> {
    let raw = raw.trim();
    let inner = raw.strip_prefix('"')?.strip_suffix('"')?;
    Some(inner.to_owned())
}

pub fn render_resolved_galaxy_index(
    dependencies: &[ResolvedGalaxyDependency],
) -> String {
    if dependencies.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for item in dependencies {
        let requested_by = if item.requested_by.is_empty() {
            "<none>".to_owned()
        } else {
            item.requested_by.join(",")
        };
        let blockers = if item.auto_inject_blockers.is_empty() {
            "<none>".to_owned()
        } else {
            item.auto_inject_blockers.join(" | ")
        };
        out.push_str(&format!(
            "{}\tpackage={}\tdirect={}\trequested_by={}\tsource_modules={}\tauto_injectable={}\n",
            item.name,
            item.package_id,
            if item.direct { "true" } else { "false" },
            requested_by,
            item.source_modules.len(),
            if item.auto_injectable { "true" } else { "false" }
        ));
        out.push_str(&format!(
            "  library_modules={}\n",
            if item.library_modules.is_empty() {
                "<none>".to_owned()
            } else {
                item.library_modules.join(", ")
            }
        ));
        out.push_str(&format!(
            "  surfaces={}\n",
            if item.surfaces.is_empty() {
                "<none>".to_owned()
            } else {
                item.surfaces.join(", ")
            }
        ));
        out.push_str(&format!(
            "  library_import_policy={}\n",
            item.library_import_policy.as_str()
        ));
        out.push_str(&format!(
            "  manifest={}\n",
            item.manifest_path.display()
        ));
        out.push_str(&format!(
            "  depends_on={}\n",
            if item.depends_on.is_empty() {
                "<none>".to_owned()
            } else {
                item.depends_on.join(", ")
            }
        ));
        out.push_str(&format!("  blockers={}\n", blockers));
    }
    out
}
