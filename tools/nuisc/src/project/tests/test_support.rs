use super::*;
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

// Test-builder guidance:
// - Use `project_with_modules(...)` for small AST/contract unit tests that only need parsed modules.
// - Use `loaded_project_fixture(...)` when a test needs a full in-memory `LoadedProject` with ABI/link metadata.
// - Use `write_temp_project_fixture(...)` when a test must go through filesystem-backed compile/pipeline entrypoints.

pub(super) fn write_temp_project_fixture(
    name: &str,
    manifest: &str,
    entry_source: &str,
    extra_modules: Vec<(&str, &str)>,
) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("nuisc_{name}_{nonce}"));
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("nuis.toml"), manifest).unwrap();
    fs::write(root.join("main.ns"), entry_source).unwrap();
    for (path, source) in extra_modules {
        fs::write(root.join(path), source).unwrap();
    }
    root
}

pub(super) fn dummy_galaxy_content_identity(
    logical_path: &str,
) -> crate::stdlib_registry::ResolvedGalaxyContentIdentity {
    crate::stdlib_registry::ResolvedGalaxyContentIdentity {
        logical_path: logical_path.to_owned(),
        bytes: 0,
        sha256: "sha256:0000000000000000000000000000000000000000000000000000000000000000"
            .to_owned(),
    }
}

pub(super) fn resolved_galaxy_dependency_fixture(
    name: &str,
    version: &str,
    package_id: &str,
    surface: &str,
    library_module: &str,
    import_policy: crate::stdlib_registry::StdlibLibraryImportPolicy,
) -> crate::stdlib_registry::ResolvedGalaxyDependency {
    let module_dir = PathBuf::from(format!("stdlib/{name}"));
    let auto_injectable = matches!(
        import_policy,
        crate::stdlib_registry::StdlibLibraryImportPolicy::ProjectAuto
    );
    crate::stdlib_registry::ResolvedGalaxyDependency {
        name: name.to_owned(),
        version: version.to_owned(),
        package_id: package_id.to_owned(),
        direct: true,
        requested_by: vec![name.to_owned()],
        manifest_path: module_dir.join("module.toml"),
        manifest_content_identity: dummy_galaxy_content_identity("module.toml"),
        module_dir: module_dir.clone(),
        depends_on: vec![],
        surfaces: vec![surface.to_owned()],
        code_assets: vec![],
        source_modules: vec![],
        resolved_source_paths: vec![],
        source_content_identities: vec![],
        library_modules: vec![library_module.to_owned()],
        resolved_library_paths: vec![module_dir.join(library_module)],
        library_content_identities: vec![dummy_galaxy_content_identity(library_module)],
        library_import_policy: import_policy,
        auto_injectable,
        auto_inject_blockers: if auto_injectable {
            vec![]
        } else {
            vec![
                "library import policy `manual-only` disables automatic project injection"
                    .to_owned(),
            ]
        },
    }
}

pub(super) fn append_manifest_links(manifest: &mut String, links: &[&str]) {
    if links.is_empty() {
        return;
    }
    manifest.push_str("links = [\n");
    for link in links {
        manifest.push_str("  \"");
        manifest.push_str(link);
        manifest.push_str("\",\n");
    }
    manifest.push_str("]\n");
}

pub(super) fn loaded_project_fixture(
    name: &str,
    abi_requirements: Vec<ProjectAbiRequirement>,
    entry_source: &str,
    extra_modules: Vec<(&str, &str)>,
) -> LoadedProject {
    let mut modules = vec![("main.ns", entry_source)];
    modules.extend(extra_modules);

    LoadedProject {
        root: PathBuf::from("."),
        manifest_path: PathBuf::from("nuis.toml"),
        manifest: NuisProjectManifest {
            name: name.to_owned(),
            entry: "main.ns".to_owned(),
            packaging_mode: None,
            artifact_provider_metadata: vec![],
            code_assets: vec![],
            modules: modules.iter().map(|(path, _)| (*path).to_owned()).collect(),
            tests: vec![],
            links: vec![],
            abi_requirements,
            galaxy_dependencies: vec![],
            galaxy_imports: vec![],
        },
        entry_path: PathBuf::from("main.ns"),
        entry_source: entry_source.to_owned(),
        modules: modules
            .into_iter()
            .map(|(path, source)| ProjectModule {
                path: PathBuf::from(path),
                ast: crate::frontend::parse_nuis_ast(source).unwrap(),
                origin: ProjectModuleOrigin::LocalProject {
                    manifest_spec: path.to_owned(),
                },
            })
            .collect(),
        resolved_galaxies: vec![],
    }
}
