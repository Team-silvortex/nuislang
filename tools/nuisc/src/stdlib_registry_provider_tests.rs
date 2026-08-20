use super::*;
use crate::project::ProjectGalaxyDependency;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

struct TempProvider(PathBuf);

impl TempProvider {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("nuis_galaxy_provider_{name}_{nonce}"));
        fs::create_dir_all(&root).unwrap();
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn descriptor(&self) -> GalaxyResolutionProviderDescriptor {
        GalaxyResolutionProviderDescriptor {
            provider_id: "fixture.offline".to_owned(),
            provider_kind: "offline-layout".to_owned(),
            root: self.0.clone(),
        }
    }
}

impl Drop for TempProvider {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn exact_provider_resolution_selects_independent_transitive_version() {
    let provider = TempProvider::new("exact_transitive");
    write_index(
        provider.path(),
        &[
            candidate("core", "1.0.0", "core/1.0.0", "nuis.core.v1", &[]),
            candidate("core", "2.0.0", "core/2.0.0", "nuis.core.v2", &[]),
            candidate("std", "1.5.0", "std/1.5.0", "nuis.std", &["core=2.0.0"]),
        ],
    );
    write_package(provider.path(), "core/1.0.0", "core", "nuis.core.v1", &[]);
    write_package(provider.path(), "core/2.0.0", "core", "nuis.core.v2", &[]);
    write_package(provider.path(), "std/1.5.0", "std", "nuis.std", &["core"]);

    let resolution = resolve_galaxy_dependencies_with_provider(
        &provider.descriptor(),
        &[ProjectGalaxyDependency {
            name: "std".to_owned(),
            version: "1.5.0".to_owned(),
        }],
    )
    .unwrap();

    assert_eq!(
        resolution.report.contract,
        GALAXY_RESOLUTION_PROVIDER_CONTRACT
    );
    assert_eq!(
        resolution.request.contract,
        GALAXY_RESOLUTION_PROVIDER_CONTRACT
    );
    assert_eq!(
        resolution.request.request_sha256,
        resolution.report.request_sha256
    );
    assert_eq!(resolution.report.status, "resolved-pinned-provider-closure");
    assert_eq!(resolution.report.candidate_count, 3);
    assert_eq!(resolution.report.selected_count, 2);
    assert!(resolution.report.request_sha256.starts_with("sha256:"));
    assert!(resolution.report.selection_sha256.starts_with("sha256:"));
    assert_eq!(
        resolution
            .dependencies
            .iter()
            .map(|item| (item.name.as_str(), item.version.as_str(), item.direct))
            .collect::<Vec<_>>(),
        [("core", "2.0.0", false), ("std", "1.5.0", true)]
    );
    assert_eq!(resolution.dependencies[0].requested_by, ["std"]);
}

#[test]
fn provider_resolution_is_input_order_independent() {
    let provider = TempProvider::new("order_independent");
    write_index(
        provider.path(),
        &[
            candidate("core", "1.0.0", "core", "nuis.core", &[]),
            candidate("std", "1.0.0", "std", "nuis.std", &[]),
        ],
    );
    write_package(provider.path(), "core", "core", "nuis.core", &[]);
    write_package(provider.path(), "std", "std", "nuis.std", &[]);
    let forward = [
        ProjectGalaxyDependency {
            name: "core".to_owned(),
            version: "1.0.0".to_owned(),
        },
        ProjectGalaxyDependency {
            name: "std".to_owned(),
            version: "1.0.0".to_owned(),
        },
    ];
    let reversed = [forward[1].clone(), forward[0].clone()];

    let first =
        resolve_galaxy_dependencies_with_provider(&provider.descriptor(), &forward).unwrap();
    let second =
        resolve_galaxy_dependencies_with_provider(&provider.descriptor(), &reversed).unwrap();

    assert_eq!(first, second);
}

#[test]
fn unpinned_ambiguous_transitive_candidate_fails_closed() {
    let provider = TempProvider::new("ambiguous_transitive");
    write_index(
        provider.path(),
        &[
            candidate("core", "1.0.0", "core-v1", "nuis.core.v1", &[]),
            candidate("core", "2.0.0", "core-v2", "nuis.core.v2", &[]),
            candidate("std", "1.0.0", "std", "nuis.std", &["core"]),
        ],
    );
    write_package(provider.path(), "core-v1", "core", "nuis.core.v1", &[]);
    write_package(provider.path(), "core-v2", "core", "nuis.core.v2", &[]);
    write_package(provider.path(), "std", "std", "nuis.std", &["core"]);

    let error = resolve_galaxy_dependencies_with_provider(
        &provider.descriptor(),
        &[ProjectGalaxyDependency {
            name: "std".to_owned(),
            version: "1.0.0".to_owned(),
        }],
    )
    .unwrap_err();

    assert!(error.contains("ambiguous unpinned transitive dependency `core`"));
    assert!(error.contains("candidates=[1.0.0, 2.0.0]"));
}

#[test]
fn missing_exact_version_and_ranges_fail_closed() {
    let provider = TempProvider::new("missing_exact");
    write_index(
        provider.path(),
        &[candidate("core", "1.0.0", "core", "nuis.core", &[])],
    );
    write_package(provider.path(), "core", "core", "nuis.core", &[]);

    let missing = resolve_galaxy_dependencies_with_provider(
        &provider.descriptor(),
        &[ProjectGalaxyDependency {
            name: "core".to_owned(),
            version: "2.0.0".to_owned(),
        }],
    )
    .unwrap_err();
    assert!(missing.contains("has no exact candidate `core=2.0.0`"));

    let range = resolve_galaxy_dependencies_with_provider(
        &provider.descriptor(),
        &[ProjectGalaxyDependency {
            name: "core".to_owned(),
            version: "^1.0.0".to_owned(),
        }],
    )
    .unwrap_err();
    assert!(range.contains("range solving is not registered yet"));
}

#[test]
fn unregistered_provider_and_escaping_candidate_fail_closed() {
    let provider = TempProvider::new("provider_validation");
    write_index(
        provider.path(),
        &[candidate("core", "1.0.0", "../outside", "nuis.core", &[])],
    );
    let mut descriptor = provider.descriptor();
    descriptor.provider_kind = "remote-magic".to_owned();
    let error = resolve_galaxy_dependencies_with_provider(&descriptor, &[]).unwrap_err();
    assert!(error.contains("unregistered kind `remote-magic`"));

    let error = resolve_galaxy_dependencies_with_provider(&provider.descriptor(), &[]).unwrap_err();
    assert!(error.contains("must be a normalized relative path"));
}

#[cfg(unix)]
#[test]
fn source_symlink_escape_is_rejected_before_content_read() {
    use std::os::unix::fs::symlink;

    let provider = TempProvider::new("source_symlink_escape");
    let outside = TempProvider::new("source_symlink_target");
    write_index(
        provider.path(),
        &[candidate("core", "1.0.0", "core", "nuis.core", &[])],
    );
    let package = provider.path().join("core");
    fs::create_dir_all(&package).unwrap();
    fs::write(
        package.join("module.toml"),
        "module_schema = \"nuis-stdlib-module-v1\"\nname = \"core\"\npackage_id = \"nuis.core\"\ntier = \"fixture\"\ndepends_on = []\nsummary = \"fixture\"\nsource_modules = []\nlibrary_modules = [\"lib.ns\"]\n",
    )
    .unwrap();
    symlink(outside.path(), package.join("lib.ns")).unwrap();

    let error = resolve_galaxy_dependencies_with_provider(
        &provider.descriptor(),
        &[ProjectGalaxyDependency {
            name: "core".to_owned(),
            version: "1.0.0".to_owned(),
        }],
    )
    .unwrap_err();
    assert!(error.contains("escapes provider root"), "{error}");
}

fn candidate(
    name: &str,
    version: &str,
    path: &str,
    package_id: &str,
    depends_on: &[&str],
) -> String {
    format!(
        "[[module]]\nname = \"{name}\"\nversion = \"{version}\"\nkind = \"fixture\"\npath = \"{path}\"\npackage_id = \"{package_id}\"\ndepends_on = [{}]\nsummary = \"fixture\"\n",
        depends_on
            .iter()
            .map(|item| format!("\"{item}\""))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn write_index(root: &Path, candidates: &[String]) {
    let modules = candidates
        .iter()
        .filter_map(|candidate| {
            candidate.lines().find_map(|line| {
                line.strip_prefix("name = \"")
                    .and_then(|line| line.strip_suffix('"'))
            })
        })
        .collect::<Vec<_>>();
    fs::write(
        root.join("index.toml"),
        format!(
            "layout_schema = \"nuis-stdlib-layout-v1\"\nname = \"fixture\"\ndefault_entry = \"{}\"\nmodules = [{}]\n\n{}",
            modules.first().copied().unwrap_or("empty"),
            modules
                .iter()
                .map(|item| format!("\"{item}\""))
                .collect::<Vec<_>>()
                .join(", "),
            candidates.join("\n")
        ),
    )
    .unwrap();
}

fn write_package(root: &Path, path: &str, name: &str, package_id: &str, depends_on: &[&str]) {
    let package = root.join(path);
    fs::create_dir_all(&package).unwrap();
    fs::write(
        package.join("module.toml"),
        format!(
            "module_schema = \"nuis-stdlib-module-v1\"\nname = \"{name}\"\npackage_id = \"{package_id}\"\ntier = \"fixture\"\ndepends_on = [{}]\nsummary = \"fixture\"\nsource_modules = []\nlibrary_modules = []\n",
            depends_on
                .iter()
                .map(|item| format!("\"{item}\""))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    )
    .unwrap();
}
