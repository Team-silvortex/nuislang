use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use crate::registry::{NustarPackageIndexEntry, NustarPackageManifest};
use crate::registry_manifest_parse::{manifest_path, parse_index, parse_manifest};
use yir_core::YirModule;

pub(crate) const INDEX_FILE: &str = "index.toml";

pub(crate) fn resolve_registry_root(root: &Path) -> PathBuf {
    if root.is_absolute() {
        return root.to_path_buf();
    }
    let workspace_candidate = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(root);
    if workspace_candidate.exists() {
        return workspace_candidate;
    }
    if root.exists() {
        return root.to_path_buf();
    }
    root.to_path_buf()
}

pub fn load_index(root: &Path) -> Result<Vec<NustarPackageIndexEntry>, String> {
    let root = resolve_registry_root(root);
    let path = root.join(INDEX_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    parse_index(&source, &path)
}

pub fn load_manifest(root: &Path, package_id: &str) -> Result<NustarPackageManifest, String> {
    let root = resolve_registry_root(root);
    let index = load_index(&root)?;
    let entry = index
        .into_iter()
        .find(|entry| entry.package_id == package_id)
        .ok_or_else(|| {
            format!(
                "nustar package `{package_id}` is not present in `{}`",
                root.join(INDEX_FILE).display()
            )
        })?;
    let path = manifest_path(&root, &entry);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    parse_manifest(&source, &path)
}

pub fn load_manifest_for_domain(
    root: &Path,
    domain_family: &str,
) -> Result<NustarPackageManifest, String> {
    let root = resolve_registry_root(root);
    let path = match load_index(&root) {
        Ok(index) => {
            match index
                .into_iter()
                .find(|entry| entry.domain_family == domain_family)
            {
                Some(entry) => manifest_path(&root, &entry),
                None => {
                    let direct = root.join(format!("{domain_family}.toml"));
                    if direct.exists() {
                        direct
                    } else {
                        return Err(format!(
                            "no nustar package is indexed for mod domain `{domain_family}` in `{}`",
                            root.join(INDEX_FILE).display()
                        ));
                    }
                }
            }
        }
        Err(index_error) => {
            let direct = root.join(format!("{domain_family}.toml"));
            if direct.exists() {
                direct
            } else {
                return Err(index_error);
            }
        }
    };
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    parse_manifest(&source, &path)
}

pub fn load_all_manifests(root: &Path) -> Result<Vec<NustarPackageManifest>, String> {
    let mut manifests = Vec::new();
    for entry in load_index(root)? {
        manifests.push(load_manifest(root, &entry.package_id)?);
    }
    manifests.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(manifests)
}

pub fn required_package_ids(module: &YirModule) -> Vec<String> {
    let mut package_ids = BTreeSet::new();
    for node in &module.nodes {
        package_ids.insert(format!("official.{}", node.op.module));
        if node.op.module == "cpu" && node.op.instruction == "instantiate_unit" {
            if let Some(domain) = node.op.args.first() {
                package_ids.insert(format!("official.{domain}"));
            }
        }
    }
    package_ids.into_iter().collect()
}

pub fn load_required_manifests(
    root: &Path,
    module: &YirModule,
) -> Result<Vec<NustarPackageManifest>, String> {
    let mut manifests = Vec::new();
    for package_id in required_package_ids(module) {
        manifests.push(load_manifest(root, &package_id)?);
    }
    manifests.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(manifests)
}
