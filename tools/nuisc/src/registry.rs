use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use yir_core::YirModule;

const INDEX_FILE: &str = "index.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarPackageIndexEntry {
    pub package_id: String,
    pub manifest: String,
    pub domain_family: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarPackageManifest {
    pub manifest_schema: String,
    pub package_id: String,
    pub domain_family: String,
    pub frontend: String,
    pub entry_crate: String,
    pub binary_extension: String,
    pub package_layout: String,
    pub machine_abi_policy: String,
    pub implementation_kinds: Vec<String>,
    pub loader_entry: String,
    pub loader_abi: String,
    pub profiles: Vec<String>,
    pub resource_families: Vec<String>,
    pub lowering_targets: Vec<String>,
    pub ops: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarBinding {
    pub package_id: String,
    pub domain_family: String,
    pub matched_resources: Vec<String>,
    pub matched_ops: Vec<String>,
    pub undeclared_ops: Vec<String>,
    pub frontend: String,
    pub entry_crate: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarBindingPlan {
    pub bindings: Vec<NustarBinding>,
}

pub fn load_index(root: &Path) -> Result<Vec<NustarPackageIndexEntry>, String> {
    let path = root.join(INDEX_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    parse_index(&source, &path)
}

pub fn load_manifest(root: &Path, package_id: &str) -> Result<NustarPackageManifest, String> {
    let index = load_index(root)?;
    let entry = index
        .into_iter()
        .find(|entry| entry.package_id == package_id)
        .ok_or_else(|| {
            format!(
                "nustar package `{package_id}` is not present in `{}`",
                root.join(INDEX_FILE).display()
            )
        })?;
    let path = manifest_path(root, &entry);
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

pub fn plan_bindings(root: &Path, module: &YirModule) -> Result<NustarBindingPlan, String> {
    let manifests = load_required_manifests(root, module)?;
    let mut bindings = Vec::new();

    for manifest in manifests {
        let matched_resources = module
            .resources
            .iter()
            .filter(|resource| {
                manifest
                    .resource_families
                    .iter()
                    .any(|family| family == resource.kind.family())
            })
            .map(|resource| resource.name.clone())
            .collect::<Vec<_>>();

        let matched_ops = module
            .nodes
            .iter()
            .filter(|node| node.op.module == manifest.domain_family)
            .map(|node| node.op.full_name())
            .collect::<Vec<_>>();

        if matched_ops.is_empty() {
            return Err(format!(
                "nustar package `{}` was selected but no matching ops were bound",
                manifest.package_id
            ));
        }

        let undeclared_ops = matched_ops
            .iter()
            .filter(|op| !manifest.ops.iter().any(|candidate| candidate == *op))
            .cloned()
            .collect::<Vec<_>>();

        bindings.push(NustarBinding {
            package_id: manifest.package_id,
            domain_family: manifest.domain_family,
            matched_resources,
            matched_ops,
            undeclared_ops,
            frontend: manifest.frontend,
            entry_crate: manifest.entry_crate,
        });
    }

    bindings.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(NustarBindingPlan { bindings })
}

pub fn manifest_path(root: &Path, entry: &NustarPackageIndexEntry) -> PathBuf {
    root.join(&entry.manifest)
}

fn parse_index(source: &str, path: &Path) -> Result<Vec<NustarPackageIndexEntry>, String> {
    let mut entries = Vec::new();
    let mut current = Vec::<String>::new();

    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line == "[[package]]" {
            if !current.is_empty() {
                entries.push(parse_index_entry(&current.join("\n"), path)?);
                current.clear();
            }
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        current.push(line.to_owned());
    }

    if !current.is_empty() {
        entries.push(parse_index_entry(&current.join("\n"), path)?);
    }

    entries.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(entries)
}

fn parse_index_entry(source: &str, path: &Path) -> Result<NustarPackageIndexEntry, String> {
    Ok(NustarPackageIndexEntry {
        package_id: parse_required_string(source, "package_id", path)?,
        manifest: parse_required_string(source, "manifest", path)?,
        domain_family: parse_required_string(source, "domain_family", path)?,
    })
}

fn parse_manifest(source: &str, path: &Path) -> Result<NustarPackageManifest, String> {
    let manifest_schema = parse_optional_string(source, "manifest_schema")
        .unwrap_or_else(|| "nustar-manifest-v1".to_owned());
    let package_id = parse_required_string(source, "package_id", path)?;
    let domain_family = parse_required_string(source, "domain_family", path)?;
    let frontend = parse_required_string(source, "frontend", path)?;
    let entry_crate = parse_required_string(source, "entry_crate", path)?;
    let binary_extension =
        parse_optional_string(source, "binary_extension").unwrap_or_else(|| "nustar".to_owned());
    let package_layout = parse_optional_string(source, "package_layout")
        .unwrap_or_else(|| "single-envelope".to_owned());
    let machine_abi_policy = parse_optional_string(source, "machine_abi_policy")
        .unwrap_or_else(|| "exact-match".to_owned());
    let implementation_kinds = parse_optional_string_array(source, "implementation_kinds")
        .unwrap_or_else(|| vec!["native-stub".to_owned()]);
    let loader_entry = parse_optional_string(source, "loader_entry")
        .unwrap_or_else(|| "nustar.bootstrap.v1".to_owned());
    let loader_abi = parse_optional_string(source, "loader_abi")
        .unwrap_or_else(|| "nustar-loader-v1".to_owned());
    let profiles = parse_string_array(source, "profiles", path)?;
    let resource_families = parse_string_array(source, "resource_families", path)?;
    let lowering_targets = parse_string_array(source, "lowering_targets", path)?;
    let ops = parse_string_array(source, "ops", path)?;

    Ok(NustarPackageManifest {
        manifest_schema,
        package_id,
        domain_family,
        frontend,
        entry_crate,
        binary_extension,
        package_layout,
        machine_abi_policy,
        implementation_kinds,
        loader_entry,
        loader_abi,
        profiles,
        resource_families,
        lowering_targets,
        ops,
    })
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

fn parse_required_string(source: &str, key: &str, path: &Path) -> Result<String, String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_quoted(rest).ok_or_else(|| {
                format!(
                    "manifest `{}` has invalid string value for `{key}`",
                    path.display()
                )
            });
        }
    }

    Err(format!(
        "manifest `{}` is missing required key `{key}`",
        path.display()
    ))
}

fn parse_string_array(source: &str, key: &str, path: &Path) -> Result<Vec<String>, String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_array(rest).ok_or_else(|| {
                format!(
                    "manifest `{}` has invalid array value for `{key}`",
                    path.display()
                )
            });
        }
    }

    Err(format!(
        "manifest `{}` is missing required key `{key}`",
        path.display()
    ))
}

fn parse_optional_string_array(source: &str, key: &str) -> Option<Vec<String>> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_array(rest);
        }
    }
    None
}

fn parse_quoted(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        Some(trimmed[1..trimmed.len() - 1].to_owned())
    } else {
        None
    }
}

fn parse_array(raw: &str) -> Option<Vec<String>> {
    let trimmed = raw.trim();
    if !(trimmed.starts_with('[') && trimmed.ends_with(']')) {
        return None;
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    if inner.trim().is_empty() {
        return Some(Vec::new());
    }

    let mut items = Vec::new();
    for part in inner.split(',') {
        items.push(parse_quoted(part.trim())?);
    }
    Some(items)
}
