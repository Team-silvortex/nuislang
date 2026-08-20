use super::resolve_project_deps_with_provider;
use nuisc::stdlib_registry::{
    GalaxyResolutionProviderDescriptor, GALAXY_RESOLUTION_PROVIDER_CONTRACT,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

struct TempRoot(PathBuf);

impl TempRoot {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("nuis_galaxy_offline_{name}_{nonce}"));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn two_offline_providers_produce_identical_lock_and_addressed_cache() {
    let provider_a = TempRoot::new("provider_a");
    let provider_b = TempRoot::new("provider_b");
    write_offline_provider(provider_a.path());
    write_offline_provider(provider_b.path());
    let project_a = TempRoot::new("project_a");
    let project_b = TempRoot::new("project_b");
    write_project(project_a.path());
    write_project(project_b.path());
    let descriptor_a = descriptor(provider_a.path());
    let descriptor_b = descriptor(provider_b.path());

    let resolved_a = resolve_project_deps_with_provider(project_a.path(), &descriptor_a).unwrap();
    let resolved_b = resolve_project_deps_with_provider(project_b.path(), &descriptor_b).unwrap();

    assert_eq!(resolved_a.provider, resolved_b.provider);
    assert_eq!(resolved_a.request, resolved_b.request);
    assert_eq!(
        resolved_a.request.contract,
        GALAXY_RESOLUTION_PROVIDER_CONTRACT
    );
    assert_eq!(
        resolved_a.provider.contract,
        GALAXY_RESOLUTION_PROVIDER_CONTRACT
    );
    assert_eq!(resolved_a.provider.provider_kind, "offline-layout");
    assert_eq!(resolved_a.provider.candidate_count, 2);
    assert_eq!(resolved_a.provider.selected_count, 2);
    assert_eq!(resolved_a.lock.summary, resolved_b.lock.summary);
    assert_eq!(resolved_a.synced.summary, resolved_b.synced.summary);
    assert_eq!(
        fs::read(&resolved_a.lock.path).unwrap(),
        fs::read(&resolved_b.lock.path).unwrap()
    );
    assert_eq!(
        snapshot_tree(&resolved_a.synced.root),
        snapshot_tree(&resolved_b.synced.root)
    );
    let lock_source = fs::read_to_string(&resolved_a.lock.path).unwrap();
    assert!(lock_source.contains("name = \"core\"\nversion = \"1.0.0\""));
    assert!(lock_source.contains("name = \"std\"\nversion = \"1.5.0\""));
    assert!(!lock_source.contains(&provider_a.path().display().to_string()));
    assert!(!lock_source.contains(&provider_b.path().display().to_string()));
    let cache_index = fs::read_to_string(resolved_a.synced.root.join("index.toml")).unwrap();
    assert!(cache_index.contains("version = \"1.0.0\""));
    assert!(cache_index.contains("version = \"1.5.0\""));

    fs::remove_dir_all(provider_a.path()).unwrap();
    fs::remove_dir_all(provider_b.path()).unwrap();
    let compiled_a = nuisc::project::load_project_for_compile(project_a.path()).unwrap();
    let compiled_b = nuisc::project::load_project_for_compile(project_b.path()).unwrap();
    assert_eq!(
        compiled_a
            .resolved_galaxies
            .iter()
            .map(|item| (item.name.as_str(), item.version.as_str()))
            .collect::<Vec<_>>(),
        [("core", "1.0.0"), ("std", "1.5.0")]
    );
    assert!(compiled_a
        .resolved_galaxies
        .iter()
        .all(|item| item.module_dir.starts_with(&resolved_a.synced.root)));
    assert!(compiled_b
        .resolved_galaxies
        .iter()
        .all(|item| item.module_dir.starts_with(&resolved_b.synced.root)));
}

fn descriptor(root: &Path) -> GalaxyResolutionProviderDescriptor {
    GalaxyResolutionProviderDescriptor {
        provider_id: "fixture.offline-mirror".to_owned(),
        provider_kind: "offline-layout".to_owned(),
        root: root.to_path_buf(),
    }
}

fn write_project(root: &Path) {
    fs::write(
        root.join("nuis.toml"),
        "name = \"offline-provider-project\"\nentry = \"main.ns\"\ngalaxy = [\"std=1.5.0\"]\n",
    )
    .unwrap();
    fs::write(
        root.join("main.ns"),
        "mod cpu Main {\n  fn main() -> i64 {\n    return 0;\n  }\n}\n",
    )
    .unwrap();
}

fn write_offline_provider(root: &Path) {
    fs::write(
        root.join("index.toml"),
        r#"layout_schema = "nuis-stdlib-layout-v1"
name = "offline-fixture"
default_entry = "core"
modules = ["core", "std"]

[[module]]
name = "core"
version = "1.0.0"
kind = "foundation"
path = "core/1.0.0"
package_id = "nuis.core"
depends_on = []
summary = "fixture core"

[[module]]
name = "std"
version = "1.5.0"
kind = "systems"
path = "std/1.5.0"
package_id = "nuis.std"
depends_on = ["core=1.0.0"]
summary = "fixture std"
"#,
    )
    .unwrap();
    write_package(
        root,
        "core/1.0.0",
        "core",
        "nuis.core",
        &[],
        "lib/core_contracts.ns",
        "CoreContracts",
    );
    write_package(
        root,
        "std/1.5.0",
        "std",
        "nuis.std",
        &["core"],
        "lib/std_contracts.ns",
        "StdContracts",
    );
}

fn write_package(
    root: &Path,
    path: &str,
    name: &str,
    package_id: &str,
    depends_on: &[&str],
    library_module: &str,
    unit: &str,
) {
    let package = root.join(path);
    let library = package.join(library_module);
    fs::create_dir_all(library.parent().unwrap()).unwrap();
    fs::write(
        package.join("module.toml"),
        format!(
            "module_schema = \"nuis-stdlib-module-v1\"\nname = \"{name}\"\npackage_id = \"{package_id}\"\ntier = \"fixture\"\ndepends_on = [{}]\nsummary = \"fixture\"\nlibrary_modules = [\"{library_module}\"]\nlibrary_import_policy = \"project-auto\"\nsource_modules = []\n",
            depends_on
                .iter()
                .map(|item| format!("\"{item}\""))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    )
    .unwrap();
    fs::write(
        library,
        format!("mod cpu {unit} {{\n  pub fn value() -> i64 {{\n    return 1;\n  }}\n}}\n"),
    )
    .unwrap();
}

fn snapshot_tree(root: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(root: &Path, current: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        let mut entries = fs::read_dir(current)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                out.push((
                    path.strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                    fs::read(path).unwrap(),
                ));
            }
        }
    }

    let mut snapshot = Vec::new();
    walk(root, root, &mut snapshot);
    snapshot
}
