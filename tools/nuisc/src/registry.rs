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
    pub ast_entry: String,
    pub nir_entry: String,
    pub yir_lowering_entry: String,
    pub part_verify_entry: String,
    pub ast_surface: Vec<String>,
    pub nir_surface: Vec<String>,
    pub yir_lowering: Vec<String>,
    pub part_verify: Vec<String>,
    pub binary_extension: String,
    pub package_layout: String,
    pub machine_abi_policy: String,
    pub implementation_kinds: Vec<String>,
    pub loader_entry: String,
    pub loader_abi: String,
    pub host_ffi_surface: Vec<String>,
    pub host_ffi_abis: Vec<String>,
    pub host_ffi_bridge: String,
    pub profiles: Vec<String>,
    pub resource_families: Vec<String>,
    pub unit_types: Vec<String>,
    pub lowering_targets: Vec<String>,
    pub ops: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarBinding {
    pub package_id: String,
    pub domain_family: String,
    pub ast_entry: String,
    pub nir_entry: String,
    pub yir_lowering_entry: String,
    pub part_verify_entry: String,
    pub ast_surface: Vec<String>,
    pub nir_surface: Vec<String>,
    pub yir_lowering: Vec<String>,
    pub part_verify: Vec<String>,
    pub registered_units: Vec<String>,
    pub bound_unit: Option<String>,
    pub used_units: Vec<String>,
    pub instantiated_units: Vec<String>,
    pub used_host_ffi_abis: Vec<String>,
    pub used_host_ffi_symbols: Vec<String>,
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

pub fn load_manifest_for_domain(
    root: &Path,
    domain_family: &str,
) -> Result<NustarPackageManifest, String> {
    let index = load_index(root)?;
    let entry = index
        .into_iter()
        .find(|entry| entry.domain_family == domain_family)
        .ok_or_else(|| {
            format!(
                "no nustar package is indexed for mod domain `{domain_family}` in `{}`",
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

pub fn plan_bindings(
    root: &Path,
    module: &YirModule,
    domain: &str,
    unit: &str,
    declared_used_units: &[(String, String)],
    declared_externs: &[(String, String)],
) -> Result<NustarBindingPlan, String> {
    let manifests = load_required_manifests(root, module)?;
    validate_unit_binding(&manifests, domain, unit)?;
    let mut bindings = Vec::new();

    for manifest in manifests {
        let registered_units = manifest
            .unit_types
            .iter()
            .filter(|unit| !unit.is_empty())
            .cloned()
            .collect::<Vec<_>>();
        let bound_unit = if manifest.domain_family == domain {
            Some(unit.to_owned())
        } else {
            None
        };
        let used_units = declared_used_units
            .iter()
            .filter(|(used_domain, _)| used_domain == &manifest.domain_family)
            .map(|(_, used_unit)| used_unit.clone())
            .collect::<Vec<_>>();
        let instantiated_units = module
            .nodes
            .iter()
            .filter(|node| {
                node.op.module == "cpu"
                    && node.op.instruction == "instantiate_unit"
                    && node.op.args.first().map(String::as_str) == Some(manifest.domain_family.as_str())
            })
            .filter_map(|node| node.op.args.get(1).cloned())
            .collect::<Vec<_>>();
        let used_host_ffi_abis = if manifest.domain_family == "cpu" {
            declared_externs
                .iter()
                .map(|(abi, _)| abi.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let used_host_ffi_symbols = if manifest.domain_family == "cpu" {
            declared_externs
                .iter()
                .map(|(_, symbol)| symbol.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

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

        if matched_ops.is_empty() && instantiated_units.is_empty() && used_units.is_empty() {
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
            ast_entry: manifest.ast_entry,
            nir_entry: manifest.nir_entry,
            yir_lowering_entry: manifest.yir_lowering_entry,
            part_verify_entry: manifest.part_verify_entry,
            ast_surface: manifest.ast_surface,
            nir_surface: manifest.nir_surface,
            yir_lowering: manifest.yir_lowering,
            part_verify: manifest.part_verify,
            registered_units,
            bound_unit,
            used_units,
            instantiated_units,
            used_host_ffi_abis,
            used_host_ffi_symbols,
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

pub fn validate_unit_binding(
    manifests: &[NustarPackageManifest],
    domain: &str,
    unit: &str,
) -> Result<(), String> {
    let manifest = manifests
        .iter()
        .find(|manifest| manifest.domain_family == domain)
        .ok_or_else(|| format!("no nustar manifest loaded for mod domain `{domain}`"))?;

    if manifest.unit_types.is_empty() {
        return Ok(());
    }

    if manifest.unit_types.iter().any(|candidate| candidate == unit) {
        return Ok(());
    }

    Err(format!(
        "unit `{unit}` is not registered by nustar package `{}` for mod domain `{domain}`",
        manifest.package_id
    ))
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
    let ast_entry = parse_optional_string(source, "ast_entry")
        .unwrap_or_else(|| format!("{}.ast.bootstrap.v1", domain_family));
    let nir_entry = parse_optional_string(source, "nir_entry")
        .unwrap_or_else(|| format!("{}.nir.bootstrap.v1", domain_family));
    let yir_lowering_entry = parse_optional_string(source, "yir_lowering_entry")
        .unwrap_or_else(|| format!("{}.yir.lowering.v1", domain_family));
    let part_verify_entry = parse_optional_string(source, "part_verify_entry")
        .unwrap_or_else(|| format!("{}.verify.partial.v1", domain_family));
    let ast_surface = parse_optional_string_array(source, "ast_surface")
        .unwrap_or_else(|| vec![format!("{domain_family}.mod-ast.v1")]);
    let nir_surface = parse_optional_string_array(source, "nir_surface")
        .unwrap_or_else(|| vec![format!("nir.{domain_family}.surface.v1")]);
    let yir_lowering = parse_optional_string_array(source, "yir_lowering")
        .unwrap_or_else(|| vec![format!("yir.{domain_family}.lowering.v1")]);
    let part_verify = parse_optional_string_array(source, "part_verify")
        .unwrap_or_else(|| vec![format!("verify.{domain_family}.contract.v1")]);
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
    let host_ffi_surface = parse_optional_string_array(source, "host_ffi_surface")
        .unwrap_or_default();
    let host_ffi_abis = parse_optional_string_array(source, "host_ffi_abis")
        .unwrap_or_default();
    let host_ffi_bridge = parse_optional_string(source, "host_ffi_bridge")
        .unwrap_or_else(|| "none".to_owned());
    let profiles = parse_string_array(source, "profiles", path)?;
    let resource_families = parse_string_array(source, "resource_families", path)?;
    let unit_types = parse_optional_string_array(source, "unit_types").unwrap_or_default();
    let lowering_targets = parse_string_array(source, "lowering_targets", path)?;
    let ops = parse_string_array(source, "ops", path)?;

    Ok(NustarPackageManifest {
        manifest_schema,
        package_id,
        domain_family,
        frontend,
        entry_crate,
        ast_entry,
        nir_entry,
        yir_lowering_entry,
        part_verify_entry,
        ast_surface,
        nir_surface,
        yir_lowering,
        part_verify,
        binary_extension,
        package_layout,
        machine_abi_policy,
        implementation_kinds,
        loader_entry,
        loader_abi,
        host_ffi_surface,
        host_ffi_abis,
        host_ffi_bridge,
        profiles,
        resource_families,
        unit_types,
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
