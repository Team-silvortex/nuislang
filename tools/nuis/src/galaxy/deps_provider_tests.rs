use super::resolve_project_deps_with_provider;
use ed25519_dalek::{Signer, SigningKey};
use nuisc::stdlib_registry::{
    GalaxyResolutionProviderDescriptor, GalaxyResolutionProviderTrustPolicy,
    GALAXY_CANDIDATE_SET_CONTRACT, GALAXY_PROVIDER_TRUST_REGISTRY_CONTRACT,
    GALAXY_RESOLUTION_PROVIDER_CONTRACT,
};
use sha2::{Digest, Sha256};
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

#[test]
fn persistent_provider_trust_does_not_enter_lock_or_addressed_cache() {
    let unsigned_provider = TempRoot::new("unsigned_provider");
    let trusted_provider = TempRoot::new("trusted_provider");
    write_offline_provider(unsigned_provider.path());
    write_offline_provider(trusted_provider.path());
    let unsigned_project = TempRoot::new("unsigned_project");
    let trusted_project = TempRoot::new("trusted_project");
    write_project(unsigned_project.path());
    write_project(trusted_project.path());

    let unsigned = resolve_project_deps_with_provider(
        unsigned_project.path(),
        &descriptor(unsigned_provider.path()),
    )
    .unwrap();
    let trusted_descriptor = write_trusted_candidate_set(
        trusted_provider.path(),
        trusted_project.path().join("provider-trust"),
    );
    let trusted =
        resolve_project_deps_with_provider(trusted_project.path(), &trusted_descriptor).unwrap();

    assert_eq!(
        trusted.provider.candidate_set.status,
        "verified-trusted-candidate-set"
    );
    let trust = trusted.provider.candidate_set.trust.as_ref().unwrap();
    assert_eq!(trust.status, "verified-persistent-trust");
    assert_eq!(trust.highest_candidate_generation, 7);
    assert_eq!(unsigned.lock.summary, trusted.lock.summary);
    assert_eq!(
        fs::read(&unsigned.lock.path).unwrap(),
        fs::read(&trusted.lock.path).unwrap()
    );
    assert_eq!(
        snapshot_tree(&unsigned.synced.root),
        snapshot_tree(&trusted.synced.root)
    );
    let state_path = &trusted_descriptor.trust_policy.as_ref().unwrap().state_path;
    let state_source = fs::read_to_string(state_path).unwrap();
    assert!(state_source.contains("nuis-galaxy-provider-trust-state-v1"));
    assert!(!fs::read_to_string(&trusted.lock.path)
        .unwrap()
        .contains("provider-trust-state"));

    let state_before = fs::read(state_path).unwrap();
    let repeated =
        resolve_project_deps_with_provider(trusted_project.path(), &trusted_descriptor).unwrap();
    assert_eq!(repeated.provider, trusted.provider);
    assert_eq!(fs::read(state_path).unwrap(), state_before);
}

fn descriptor(root: &Path) -> GalaxyResolutionProviderDescriptor {
    GalaxyResolutionProviderDescriptor {
        provider_id: "fixture.offline-mirror".to_owned(),
        provider_kind: "offline-layout".to_owned(),
        root: root.to_path_buf(),
        trust_policy: None,
    }
}

fn write_trusted_candidate_set(
    root: &Path,
    trust_root: PathBuf,
) -> GalaxyResolutionProviderDescriptor {
    fs::create_dir_all(&trust_root).unwrap();
    let mut descriptor = descriptor(root);
    let index_bytes = fs::read(root.join("index.toml")).unwrap();
    let index_sha256 = sha256(&index_bytes);
    let mut candidates = String::new();
    append_text(&mut candidates, GALAXY_CANDIDATE_SET_CONTRACT);
    for fields in [
        [
            "core",
            "1.0.0",
            "foundation",
            "core/1.0.0",
            "nuis.core",
            "",
            "fixture core",
        ],
        [
            "std",
            "1.5.0",
            "systems",
            "std/1.5.0",
            "nuis.std",
            "core=1.0.0",
            "fixture std",
        ],
    ] {
        for field in &fields[..5] {
            append_text(&mut candidates, field);
        }
        if !fields[5].is_empty() {
            append_text(&mut candidates, fields[5]);
        }
        append_text(&mut candidates, fields[6]);
    }
    let candidate_sha256 = sha256(candidates.as_bytes());
    let mut payload = String::new();
    append_text(&mut payload, GALAXY_CANDIDATE_SET_CONTRACT);
    append_text(&mut payload, &descriptor.provider_id);
    append_text(&mut payload, &descriptor.provider_kind);
    payload.push_str("generation=7\n");
    append_text(&mut payload, &index_sha256);
    payload.push_str("candidate_count=2\n");
    append_text(&mut payload, &candidate_sha256);
    let signing_key = SigningKey::from_bytes(&[37; 32]);
    let public_key = signing_key.verifying_key().to_bytes();
    let signer_id = format!("ed25519:{}", sha256(&public_key));
    let signature = signing_key.sign(payload.as_bytes()).to_bytes();
    fs::write(
        root.join("candidate-set.toml"),
        format!(
            "candidate_set_contract = \"{GALAXY_CANDIDATE_SET_CONTRACT}\"\nprovider_id = \"{}\"\nprovider_kind = \"{}\"\ngeneration = 7\nindex_sha256 = \"{index_sha256}\"\ncandidate_count = 2\ncandidate_sha256 = \"{candidate_sha256}\"\n\n[[signature]]\nsigner_id = \"{signer_id}\"\npublic_key_hex = \"{}\"\nsignature_hex = \"{}\"\n",
            descriptor.provider_id,
            descriptor.provider_kind,
            hex(&public_key),
            hex(&signature)
        ),
    )
    .unwrap();
    let registry_path = trust_root.join("registry.toml");
    fs::write(
        &registry_path,
        format!(
            "trust_registry_contract = \"{GALAXY_PROVIDER_TRUST_REGISTRY_CONTRACT}\"\nprovider_id = \"{}\"\nprovider_kind = \"{}\"\ngeneration = 1\n\n[[signer]]\nsigner_id = \"{signer_id}\"\nstatus = \"active\"\n",
            descriptor.provider_id, descriptor.provider_kind
        ),
    )
    .unwrap();
    descriptor.trust_policy = Some(GalaxyResolutionProviderTrustPolicy {
        registry_path,
        state_path: trust_root.join("state.toml"),
    });
    descriptor
}

fn append_text(out: &mut String, value: &str) {
    use std::fmt::Write as _;
    writeln!(out, "text:{}:{value}", value.len()).unwrap();
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
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
