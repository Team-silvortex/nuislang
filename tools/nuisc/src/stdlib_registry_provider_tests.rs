use super::*;
use crate::project::ProjectGalaxyDependency;
use crate::stdlib_registry::load_stdlib_layout;
use crate::stdlib_registry::stdlib_registry_provider_trust::{
    candidate_set_signing_payload, canonical_candidate_sha256, GALAXY_CANDIDATE_SET_CONTRACT,
    GALAXY_CANDIDATE_SET_FILE,
};
use ed25519_dalek::{Signer, SigningKey};
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
    assert_eq!(
        resolution.report.candidate_set.status,
        "unsigned-exact-only"
    );
    assert_eq!(resolution.report.candidate_set.generation, 0);
    assert_eq!(
        resolution.request.candidate_set,
        resolution.report.candidate_set
    );
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
fn missing_exact_version_and_unsigned_ranges_fail_closed() {
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
    assert!(range.contains("requires a verified `candidate-set.toml` sidecar"));
}

#[test]
fn signed_range_solver_backtracks_and_is_input_order_independent() {
    let provider = TempProvider::new("signed_backtracking");
    write_index(
        provider.path(),
        &[
            candidate("app", "1.0.0", "app-v1", "nuis.app.v1", &["core=^1.0.0"]),
            candidate("app", "1.1.0", "app-v2", "nuis.app.v2", &["core=^2.0.0"]),
            candidate("core", "1.8.0", "core-v1", "nuis.core.v1", &[]),
            candidate("core", "2.1.0", "core-v2", "nuis.core.v2", &[]),
            candidate(
                "guard",
                "1.0.0",
                "guard",
                "nuis.guard",
                &["core=>=1.0.0,<2.0.0"],
            ),
        ],
    );
    write_package(provider.path(), "app-v1", "app", "nuis.app.v1", &["core"]);
    write_package(provider.path(), "app-v2", "app", "nuis.app.v2", &["core"]);
    write_package(provider.path(), "core-v1", "core", "nuis.core.v1", &[]);
    write_package(provider.path(), "core-v2", "core", "nuis.core.v2", &[]);
    write_package(provider.path(), "guard", "guard", "nuis.guard", &["core"]);
    write_signed_candidate_set(&provider, 7, 19);

    let forward = [
        ProjectGalaxyDependency {
            name: "app".to_owned(),
            version: "^1.0.0".to_owned(),
        },
        ProjectGalaxyDependency {
            name: "guard".to_owned(),
            version: "1.0.0".to_owned(),
        },
    ];
    let reversed = [forward[1].clone(), forward[0].clone()];
    let first =
        resolve_galaxy_dependencies_with_provider(&provider.descriptor(), &forward).unwrap();
    let second =
        resolve_galaxy_dependencies_with_provider(&provider.descriptor(), &reversed).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.report.status, "resolved-signed-provider-closure");
    assert_eq!(first.report.candidate_set.generation, 7);
    assert_eq!(first.report.candidate_set.signature_count, 1);
    assert_eq!(
        first
            .dependencies
            .iter()
            .map(|item| (item.name.as_str(), item.version.as_str()))
            .collect::<Vec<_>>(),
        [("app", "1.0.0"), ("core", "1.8.0"), ("guard", "1.0.0")]
    );
    assert_eq!(
        first.request.requirements[0].version_requirement,
        ">=1.0.0,<2.0.0"
    );
}

#[test]
fn signed_candidate_set_rejects_index_and_signature_tampering() {
    let provider = TempProvider::new("signed_tamper");
    write_index(
        provider.path(),
        &[candidate("core", "1.0.0", "core", "nuis.core", &[])],
    );
    write_package(provider.path(), "core", "core", "nuis.core", &[]);
    write_signed_candidate_set(&provider, 2, 23);
    let descriptor = provider.descriptor();
    let request = [ProjectGalaxyDependency {
        name: "core".to_owned(),
        version: "^1.0.0".to_owned(),
    }];
    let index_path = provider.path().join("index.toml");
    let original_index = fs::read_to_string(&index_path).unwrap();
    fs::write(&index_path, format!("{original_index}# changed\n")).unwrap();
    let error = resolve_galaxy_dependencies_with_provider(&descriptor, &request).unwrap_err();
    assert!(error.contains("index_sha256 does not match"), "{error}");

    fs::write(&index_path, original_index).unwrap();
    let sidecar_path = provider.path().join(GALAXY_CANDIDATE_SET_FILE);
    let mut sidecar = fs::read_to_string(&sidecar_path).unwrap();
    let signature_start = sidecar.find("signature_hex = \"").unwrap() + "signature_hex = \"".len();
    let replacement = if &sidecar[signature_start..signature_start + 1] == "0" {
        "1"
    } else {
        "0"
    };
    sidecar.replace_range(signature_start..signature_start + 1, replacement);
    fs::write(&sidecar_path, sidecar).unwrap();
    let error = resolve_galaxy_dependencies_with_provider(&descriptor, &request).unwrap_err();
    assert!(
        error.contains("does not match the canonical response"),
        "{error}"
    );
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

#[cfg(unix)]
#[test]
fn provider_control_file_symlink_escapes_are_rejected() {
    use std::os::unix::fs::symlink;

    let outside = TempProvider::new("control_file_target");
    let escaped_index = TempProvider::new("escaped_index");
    fs::write(
        outside.path().join("external-index.toml"),
        "layout_schema = \"nuis-stdlib-layout-v1\"\nname = \"outside\"\ndefault_entry = \"empty\"\nmodules = []\n",
    )
    .unwrap();
    symlink(
        outside.path().join("external-index.toml"),
        escaped_index.path().join("index.toml"),
    )
    .unwrap();
    let error =
        resolve_galaxy_dependencies_with_provider(&escaped_index.descriptor(), &[]).unwrap_err();
    assert!(error.contains("escapes provider root"), "{error}");

    let escaped_sidecar = TempProvider::new("escaped_sidecar");
    write_index(
        escaped_sidecar.path(),
        &[candidate("core", "1.0.0", "core", "nuis.core", &[])],
    );
    write_package(escaped_sidecar.path(), "core", "core", "nuis.core", &[]);
    fs::write(outside.path().join("external-sidecar.toml"), "invalid\n").unwrap();
    symlink(
        outside.path().join("external-sidecar.toml"),
        escaped_sidecar.path().join(GALAXY_CANDIDATE_SET_FILE),
    )
    .unwrap();
    let error =
        resolve_galaxy_dependencies_with_provider(&escaped_sidecar.descriptor(), &[]).unwrap_err();
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

fn write_signed_candidate_set(provider: &TempProvider, generation: u64, seed: u8) {
    let descriptor = provider.descriptor();
    let index_bytes = fs::read(provider.path().join("index.toml")).unwrap();
    let layout = load_stdlib_layout(provider.path()).unwrap();
    let candidates = collect_candidates(&descriptor, layout.modules).unwrap();
    let index_sha256 = format!("sha256:{}", crate::digest_sha256::sha256_hex(&index_bytes));
    let candidate_sha256 = canonical_candidate_sha256(&candidates);
    let payload = candidate_set_signing_payload(
        &descriptor,
        generation,
        &index_sha256,
        candidates.len(),
        &candidate_sha256,
    );
    let signing_key = SigningKey::from_bytes(&[seed; 32]);
    let public_key = signing_key.verifying_key().to_bytes();
    let signer_id = format!(
        "ed25519:sha256:{}",
        crate::digest_sha256::sha256_hex(&public_key)
    );
    let signature = signing_key.sign(&payload).to_bytes();
    fs::write(
        provider.path().join(GALAXY_CANDIDATE_SET_FILE),
        format!(
            "candidate_set_contract = \"{GALAXY_CANDIDATE_SET_CONTRACT}\"\nprovider_id = \"{}\"\nprovider_kind = \"{}\"\ngeneration = {generation}\nindex_sha256 = \"{index_sha256}\"\ncandidate_count = {}\ncandidate_sha256 = \"{candidate_sha256}\"\n\n[[signature]]\nsigner_id = \"{signer_id}\"\npublic_key_hex = \"{}\"\nsignature_hex = \"{}\"\n",
            descriptor.provider_id,
            descriptor.provider_kind,
            candidates.len(),
            hex(&public_key),
            hex(&signature)
        ),
    )
    .unwrap();
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
