use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[path = "stdlib_registry_parser.rs"]
mod stdlib_registry_parser;
#[path = "stdlib_registry_render.rs"]
mod stdlib_registry_render;
#[path = "stdlib_registry_types.rs"]
mod stdlib_registry_types;

use stdlib_registry_parser::{
    parse_library_import_policy, parse_optional_string_array, parse_required_string,
    parse_stdlib_index_modules,
};
pub(crate) use stdlib_registry_render::summarize_resolved_galaxy_docs;
pub use stdlib_registry_render::{render_resolved_galaxy_index, write_resolved_galaxy_index};
pub(crate) use stdlib_registry_types::ResolvedGalaxyDocSummary;
pub use stdlib_registry_types::{
    ResolvedGalaxyDependency, StdlibIndexModule, StdlibLayout, StdlibLibraryImportPolicy,
    StdlibModuleManifest,
};
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
        library_modules: parse_optional_string_array(&source, "library_modules")
            .unwrap_or_default(),
        library_import_policy: parse_library_import_policy(&source, &path)?,
    })
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
        .map(|item| {
            (
                item.name.clone(),
                item.version.clone(),
                true,
                item.name.clone(),
            )
        })
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
        let (auto_injectable, auto_inject_blockers) =
            detect_auto_injectability(&resolved_library_paths, &manifest.library_import_policy)?;

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
            vec![
                "library import policy `manual-only` disables automatic project injection"
                    .to_owned(),
            ],
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
                "contract.pixelmagic.render-plan.v1".to_owned(),
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
    fn string_array_parser_preserves_commas_inside_quoted_stdlib_values() {
        let values = parse_optional_string_array(
            r#"surfaces = ["surface.std.text,json.v1", "surface.std.io.v1"]"#,
            "surfaces",
        )
        .expect("array should parse");

        assert_eq!(
            values,
            vec![
                "surface.std.text,json.v1".to_owned(),
                "surface.std.io.v1".to_owned()
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
                "surface.std.cli-workflow-contracts.v1".to_owned(),
                "surface.std.net-session-contracts.v1".to_owned(),
                "surface.std.text-json-contracts.v1".to_owned(),
                "surface.std.time-benchmark-contracts.v1".to_owned(),
                "surface.std.hetero-test-benchmark-contracts.v1".to_owned(),
                "surface.std.cli-report-file-contracts.v1".to_owned(),
                "surface.std.language-result-hof-contracts.v1".to_owned(),
            ]
        );
        assert_eq!(
            manifest.library_modules,
            vec![
                "lib/task_contracts.ns".to_owned(),
                "lib/io_contracts.ns".to_owned(),
                "lib/fs_contracts.ns".to_owned(),
                "lib/cli_contracts.ns".to_owned(),
                "lib/net_contracts.ns".to_owned(),
                "lib/text_contracts.ns".to_owned(),
                "lib/time_contracts.ns".to_owned(),
                "lib/hetero_contracts.ns".to_owned(),
                "lib/report_contracts.ns".to_owned(),
                "lib/language_core.ns".to_owned(),
                "lib/language_ops.ns".to_owned(),
            ]
        );
    }

    #[test]
    fn ns_nova_manifest_exposes_canonical_surface_registry_ids() {
        let stdlib_root = resolve_stdlib_root().expect("resolve stdlib root");
        let manifest = load_stdlib_module_manifest(&stdlib_root, "ns-nova").expect("load ns-nova");
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
