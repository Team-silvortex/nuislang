use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[path = "stdlib_registry_parser.rs"]
mod stdlib_registry_parser;
#[path = "stdlib_registry_provider.rs"]
mod stdlib_registry_provider;
#[path = "stdlib_registry_provider_semver.rs"]
mod stdlib_registry_provider_semver;
#[path = "stdlib_registry_provider_solver.rs"]
mod stdlib_registry_provider_solver;
#[path = "stdlib_registry_provider_trust.rs"]
mod stdlib_registry_provider_trust;
#[path = "stdlib_registry_render.rs"]
mod stdlib_registry_render;
#[path = "stdlib_registry_types.rs"]
mod stdlib_registry_types;

use stdlib_registry_parser::{
    parse_library_import_policy, parse_optional_string_array, parse_required_string,
    parse_stdlib_index_modules,
};
pub use stdlib_registry_provider::{
    resolve_galaxy_dependencies_with_provider, GALAXY_RESOLUTION_PROVIDER_CONTRACT,
    GALAXY_RESOLUTION_PROVIDER_KINDS,
};
pub use stdlib_registry_provider_trust::{
    GALAXY_CANDIDATE_SET_CONTRACT, GALAXY_CANDIDATE_SET_FILE,
};
pub(crate) use stdlib_registry_render::summarize_resolved_galaxy_docs;
pub use stdlib_registry_render::{render_resolved_galaxy_index, write_resolved_galaxy_index};
pub(crate) use stdlib_registry_types::ResolvedGalaxyDocSummary;
pub use stdlib_registry_types::{
    GalaxyResolutionCandidateSetReport, GalaxyResolutionProviderDescriptor,
    GalaxyResolutionProviderReport, GalaxyResolutionProviderRequest,
    GalaxyResolutionProviderRequirement, GalaxyResolutionProviderResolution,
    GalaxyResolutionProviderSelection, ResolvedGalaxyContentIdentity, ResolvedGalaxyDependency,
    StdlibIndexModule, StdlibLayout, StdlibLibraryImportPolicy, StdlibModuleManifest,
};
pub fn load_stdlib_layout(stdlib_root: &Path) -> Result<StdlibLayout, String> {
    let path = stdlib_root.join("index.toml");
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read stdlib layout `{}`: {error}", path.display()))?;
    parse_stdlib_layout_source(&source, &path)
}

pub(in crate::stdlib_registry) fn parse_stdlib_layout_source(
    source: &str,
    path: &Path,
) -> Result<StdlibLayout, String> {
    let schema = parse_required_string(source, "layout_schema", path)?;
    let name = parse_required_string(source, "name", path)?;
    let default_entry = parse_required_string(source, "default_entry", path)?;
    let modules = parse_stdlib_index_modules(source, path)?;
    Ok(StdlibLayout {
        schema,
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
    load_stdlib_module_manifest_with_identity(stdlib_root, module_path)
        .map(|(manifest, _)| manifest)
}

pub(in crate::stdlib_registry) fn load_stdlib_module_manifest_with_identity(
    stdlib_root: &Path,
    module_path: &str,
) -> Result<(StdlibModuleManifest, ResolvedGalaxyContentIdentity), String> {
    let path = stdlib_root.join(module_path).join("module.toml");
    let source = fs::read_to_string(&path).map_err(|error| {
        format!(
            "failed to read stdlib module manifest `{}`: {error}",
            path.display()
        )
    })?;
    let manifest = StdlibModuleManifest {
        name: parse_required_string(&source, "name", &path)?,
        package_id: parse_required_string(&source, "package_id", &path)?,
        tier: parse_required_string(&source, "tier", &path)?,
        depends_on: parse_optional_string_array(&source, "depends_on").unwrap_or_default(),
        summary: parse_required_string(&source, "summary", &path)?,
        surfaces: parse_optional_string_array(&source, "surfaces").unwrap_or_default(),
        code_assets: parse_optional_string_array(&source, "code_assets").unwrap_or_default(),
        source_modules: parse_optional_string_array(&source, "source_modules").unwrap_or_default(),
        library_modules: parse_optional_string_array(&source, "library_modules")
            .unwrap_or_default(),
        library_import_policy: parse_library_import_policy(&source, &path)?,
    };
    Ok((manifest, content_identity("module.toml", source.as_bytes())))
}

pub fn resolve_galaxy_dependencies(
    stdlib_root: &Path,
    requested: &[crate::project::ProjectGalaxyDependency],
) -> Result<Vec<ResolvedGalaxyDependency>, String> {
    let provider = GalaxyResolutionProviderDescriptor {
        provider_id: "official.workspace".to_owned(),
        provider_kind: "workspace-layout".to_owned(),
        root: stdlib_root.to_path_buf(),
    };
    resolve_galaxy_dependencies_with_provider(&provider, requested)
        .map(|resolution| resolution.dependencies)
}

pub(in crate::stdlib_registry) fn detect_auto_injectability(
    source_paths: &[PathBuf],
    content_identities: &[ResolvedGalaxyContentIdentity],
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
    if source_paths.len() != content_identities.len() {
        return Err("Galaxy resolution produced mismatched library content identities".to_owned());
    }
    for (path, identity) in source_paths.iter().zip(content_identities) {
        let source = read_verified_galaxy_text(path, identity)?;
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

pub fn read_verified_galaxy_text(
    path: &Path,
    identity: &ResolvedGalaxyContentIdentity,
) -> Result<String, String> {
    let source = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read Galaxy source `{}` for `{}`: {error}",
            path.display(),
            identity.logical_path
        )
    })?;
    let actual = content_identity(&identity.logical_path, source.as_bytes());
    if actual != *identity {
        return Err(format!(
            "Galaxy source `{}` drifted after resolution: expected bytes={} sha256={}, actual bytes={} sha256={}",
            identity.logical_path,
            identity.bytes,
            identity.sha256,
            actual.bytes,
            actual.sha256
        ));
    }
    Ok(source)
}

pub(in crate::stdlib_registry) fn read_content_identities(
    logical_paths: &[String],
    physical_paths: &[PathBuf],
) -> Result<Vec<ResolvedGalaxyContentIdentity>, String> {
    if logical_paths.len() != physical_paths.len() {
        return Err(
            "Galaxy resolution produced mismatched logical and physical source tables".to_owned(),
        );
    }
    logical_paths
        .iter()
        .zip(physical_paths)
        .map(|(logical_path, physical_path)| {
            let source = fs::read_to_string(physical_path).map_err(|error| {
                format!(
                    "failed to read Galaxy source `{}`: {error}",
                    physical_path.display()
                )
            })?;
            Ok(content_identity(logical_path, source.as_bytes()))
        })
        .collect()
}

fn content_identity(logical_path: &str, bytes: &[u8]) -> ResolvedGalaxyContentIdentity {
    ResolvedGalaxyContentIdentity {
        logical_path: logical_path.to_owned(),
        bytes: bytes.len(),
        sha256: format!("sha256:{}", crate::digest_sha256::sha256_hex(bytes)),
    }
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
                "contract.pixelmagic.provider-sample-input-registration.v1".to_owned(),
                "contract.pixelmagic.filter-plan.v1".to_owned(),
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
    fn witsage_manifest_exposes_cross_domain_code_asset_requirements() {
        let stdlib_root = resolve_stdlib_root().expect("resolve stdlib root");
        let manifest = load_stdlib_module_manifest(&stdlib_root, "witsage").expect("load witsage");

        assert_eq!(
            manifest.code_assets,
            vec![
                "shader.witsage.vector-bias.metal".to_owned(),
                "shader.witsage.argmax.metal".to_owned(),
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
                "surface.std.provider-worker-lifecycle.v1".to_owned(),
                "surface.std.provider-worker-dispatch.v1".to_owned(),
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
                "lib/provider_worker_contracts.ns".to_owned(),
                "lib/provider_worker_dispatch_contracts.ns".to_owned(),
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
