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
            modules: modules.iter().map(|(path, _)| (*path).to_owned()).collect(),
            tests: vec![],
            links: vec![],
            abi_requirements,
            galaxy_dependencies: vec![],
        },
        entry_path: PathBuf::from("main.ns"),
        entry_source: entry_source.to_owned(),
        modules: modules
            .into_iter()
            .map(|(path, source)| ProjectModule {
                path: PathBuf::from(path),
                ast: crate::frontend::parse_nuis_ast(source).unwrap(),
            })
            .collect(),
    }
}
