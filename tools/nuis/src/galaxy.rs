use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

const GALAXY_MAGIC: &[u8; 8] = b"GALAXY01";
const GALAXY_BUNDLE_VERSION: u16 = 1;

mod bundle;
mod deps;
mod local;
mod manifest;
mod package;
mod profile;
mod safety;
#[cfg(test)]
mod tests;

use deps::remove_dir_if_empty;
pub use deps::{
    doctor_project, install_project_deps, lock_project_deps, sync_project_deps, verify_project_lock,
};
use local::ensure_local_layout;
pub use local::{
    inspect_local, install_local, list_local, local_index_root, local_packages_root, local_root,
    remove_local, verify_local,
};
use manifest::{
    compare_version, escape, fnv1a64_hex, parse_local_index_entry, parse_manifest,
    parse_ns_nova_manifest, parse_optional_string, parse_optional_string_array, parse_optional_u64,
    parse_required_string, render_manifest, render_ns_nova_manifest, render_string_array,
    select_local_entry,
};
pub use package::{inspect_bundle, pack, publish_local};
use profile::default_ns_nova_manifest;
pub use profile::{inspect_ns_nova_profile, inspect_ns_nova_stdlib};
use safety::{validate_galaxy_token, validate_path_under_root, validate_relative_bundle_path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalaxyManifest {
    pub manifest_schema: String,
    pub name: String,
    pub version: String,
    pub package_kind: String,
    pub framework: Option<String>,
    pub project: String,
    pub summary: String,
    pub license: String,
    pub repository: String,
    pub authors: Vec<String>,
    pub include: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NsNovaManifest {
    pub framework_schema: String,
    pub framework: String,
    pub project: String,
    pub stdlib_schema: Option<String>,
    pub stdlib_manifest: Option<String>,
    pub stdlib_sources: Vec<String>,
    pub family_schema: Option<String>,
    pub family_layers: Vec<String>,
    pub entry_cpu_unit: Option<String>,
    pub primary_data_unit: Option<String>,
    pub primary_shader_unit: Option<String>,
    pub primary_kernel_unit: Option<String>,
    pub render_links: Vec<String>,
    pub render_schema: Option<String>,
    pub render_owner_unit: Option<String>,
    pub render_bridge_unit: Option<String>,
    pub render_surface_unit: Option<String>,
    pub selection_schema: Option<String>,
    pub selection_owner_unit: Option<String>,
    pub selection_bridge_unit: Option<String>,
    pub selection_render_unit: Option<String>,
    pub selection_controls: Vec<String>,
    pub cpu_units: Vec<String>,
    pub data_units: Vec<String>,
    pub shader_units: Vec<String>,
    pub kernel_units: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedGalaxy {
    pub root: PathBuf,
    pub manifest_path: PathBuf,
    pub manifest: GalaxyManifest,
    pub project: nuisc::project::LoadedProject,
    pub project_plan_summary: String,
    pub include_files: Vec<PathBuf>,
    pub abi_entries: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalaxyBundleEntry {
    pub path: String,
    pub bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectedGalaxyBundle {
    pub manifest: GalaxyManifest,
    pub entries: Vec<GalaxyBundleEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalGalaxyIndexEntry {
    pub name: String,
    pub version: String,
    pub package: String,
    pub project: String,
    pub abi: Vec<String>,
    pub bundle_bytes: Option<u64>,
    pub bundle_fnv1a64: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedLocalGalaxy {
    pub name: String,
    pub version: String,
    pub package: PathBuf,
    pub bundle_bytes: u64,
    pub bundle_fnv1a64: String,
    pub entries: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovedLocalGalaxy {
    pub name: String,
    pub version: String,
    pub package: PathBuf,
    pub index_entry: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledGalaxyDependency {
    pub name: String,
    pub version: String,
    pub output: PathBuf,
    pub project: PathBuf,
    pub bundle: PathBuf,
    pub bundle_bytes: u64,
    pub bundle_fnv1a64: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalaxyLockEntry {
    pub name: String,
    pub version: String,
    pub bundle: PathBuf,
    pub bundle_bytes: u64,
    pub bundle_fnv1a64: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WroteGalaxyLock {
    pub project_root: PathBuf,
    pub project_plan_summary: String,
    pub path: PathBuf,
    pub entries: Vec<GalaxyLockEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledProjectDeps {
    pub project_root: PathBuf,
    pub project_plan_summary: String,
    pub installed: Vec<InstalledGalaxyDependency>,
    pub lock: WroteGalaxyLock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedGalaxyLock {
    pub project_root: PathBuf,
    pub project_plan_summary: String,
    pub path: PathBuf,
    pub entries: Vec<GalaxyLockEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncedProjectDeps {
    pub project_root: PathBuf,
    pub project_plan_summary: String,
    pub root: PathBuf,
    pub entries: Vec<GalaxyLockEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalaxyDoctorDependency {
    pub name: String,
    pub version: String,
    pub local_available: bool,
    pub locked: bool,
    pub installed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalaxyDoctorReport {
    pub project_root: PathBuf,
    pub project_plan_summary: String,
    pub deps_root: PathBuf,
    pub local_registry_root: PathBuf,
    pub lock_path: PathBuf,
    pub lock_status: String,
    pub lock_error: Option<String>,
    pub dependencies: Vec<GalaxyDoctorDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsNovaProfileSummary {
    pub path: PathBuf,
    pub framework_schema: String,
    pub framework: String,
    pub stdlib_schema: Option<String>,
    pub stdlib_manifest: Option<String>,
    pub stdlib_sources: Vec<String>,
    pub family_schema: Option<String>,
    pub family_layers: Vec<String>,
    pub render_schema: Option<String>,
    pub render_owner_unit: Option<String>,
    pub render_bridge_unit: Option<String>,
    pub render_surface_unit: Option<String>,
    pub selection_schema: Option<String>,
    pub selection_owner_unit: Option<String>,
    pub selection_bridge_unit: Option<String>,
    pub selection_render_unit: Option<String>,
    pub selection_controls: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NsNovaStdlibSummary {
    pub path: PathBuf,
    pub source_modules: Vec<PathBuf>,
    pub missing_modules: Vec<PathBuf>,
}

pub fn init(input: &Path, framework: Option<&str>) -> Result<PathBuf, String> {
    ensure_local_layout()?;
    let project = nuisc::project::load_project(input)?;
    let manifest_path = project.root.join("galaxy.toml");
    if manifest_path.exists() {
        return Err(format!(
            "galaxy manifest already exists at `{}`",
            manifest_path.display()
        ));
    }

    let manifest = default_manifest(&project, framework)?;
    fs::write(&manifest_path, render_manifest(&manifest))
        .map_err(|error| format!("failed to write `{}`: {error}", manifest_path.display()))?;
    if framework == Some("ns-nova") {
        let nova_path = project.root.join("ns-nova.toml");
        let nova_manifest = default_ns_nova_manifest(&project);
        fs::write(&nova_path, render_ns_nova_manifest(&nova_manifest))
            .map_err(|error| format!("failed to write `{}`: {error}", nova_path.display()))?;
    }
    Ok(manifest_path)
}

pub fn check(input: &Path) -> Result<CheckedGalaxy, String> {
    ensure_local_layout()?;
    let (root, manifest_path) = resolve_galaxy_manifest(input)?;
    let source = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("failed to read `{}`: {error}", manifest_path.display()))?;
    let manifest = parse_manifest(&source, &manifest_path)?;
    let project_relative =
        validate_relative_bundle_path("project", &manifest.project, &manifest_path)?;
    let project_path = root.join(&project_relative);
    let project = nuisc::project::load_project(&project_path)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let project_plan_summary = nuisc::project::describe_project_compilation_plan(&plan);
    let abi = plan.abi_resolution.clone();

    let include_files = manifest
        .include
        .iter()
        .map(|item| {
            validate_relative_bundle_path("include", item, &manifest_path)
                .map(|relative| root.join(relative))
        })
        .collect::<Result<Vec<_>, _>>()?;
    for path in &include_files {
        if !path.exists() {
            return Err(format!(
                "galaxy manifest `{}` references missing include `{}`",
                manifest_path.display(),
                path.display()
            ));
        }
    }
    if manifest.framework.as_deref() == Some("ns-nova") {
        let profile_path = root.join("ns-nova.toml");
        let profile_source = fs::read_to_string(&profile_path)
            .map_err(|error| format!("failed to read `{}`: {error}", profile_path.display()))?;
        let profile = parse_ns_nova_manifest(&profile_source, &profile_path)?;
        if profile.project != "nuis.toml" {
            return Err(format!(
                "ns-nova profile `{}` must point at `nuis.toml`, got `{}`",
                profile_path.display(),
                profile.project
            ));
        }
        let declared = project
            .modules
            .iter()
            .map(|module| format!("{}.{}", module.ast.domain, module.ast.unit))
            .collect::<BTreeSet<_>>();
        for unit in [
            profile.entry_cpu_unit.as_deref(),
            profile.primary_data_unit.as_deref(),
            profile.primary_shader_unit.as_deref(),
            profile.primary_kernel_unit.as_deref(),
            profile.render_owner_unit.as_deref(),
            profile.render_bridge_unit.as_deref(),
            profile.render_surface_unit.as_deref(),
            profile.selection_owner_unit.as_deref(),
            profile.selection_bridge_unit.as_deref(),
            profile.selection_render_unit.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if !declared.contains(unit) {
                return Err(format!(
                    "ns-nova profile `{}` references missing project unit `{}`",
                    profile_path.display(),
                    unit
                ));
            }
        }
        if let Some(schema) = profile.family_schema.as_deref() {
            if schema != "ns-nova-family-v1" {
                return Err(format!(
                    "ns-nova profile `{}` has unsupported family_schema `{}`; expected `ns-nova-family-v1`",
                    profile_path.display(),
                    schema
                ));
            }
            if profile.family_layers.is_empty() {
                return Err(format!(
                    "ns-nova profile `{}` enables family_schema but declares no `family_layers`",
                    profile_path.display()
                ));
            }
            for layer in &profile.family_layers {
                if !matches!(layer.as_str(), "core" | "ui" | "scene") {
                    return Err(format!(
                        "ns-nova profile `{}` declares unsupported family layer `{}`",
                        profile_path.display(),
                        layer
                    ));
                }
            }
        }
        if let Some(schema) = profile.render_schema.as_deref() {
            if schema != "ns-nova-render-v1" {
                return Err(format!(
                    "ns-nova profile `{}` has unsupported render_schema `{}`; expected `ns-nova-render-v1`",
                    profile_path.display(),
                    schema
                ));
            }
            for (field, value) in [
                ("render_owner_unit", profile.render_owner_unit.as_deref()),
                ("render_bridge_unit", profile.render_bridge_unit.as_deref()),
                (
                    "render_surface_unit",
                    profile.render_surface_unit.as_deref(),
                ),
            ] {
                if value.is_none() {
                    return Err(format!(
                        "ns-nova profile `{}` enables render_schema but is missing `{field}`",
                        profile_path.display()
                    ));
                }
            }
        }
        if let Some(schema) = profile.selection_schema.as_deref() {
            if schema != "ns-nova-selection-v1" {
                return Err(format!(
                    "ns-nova profile `{}` has unsupported selection_schema `{}`; expected `ns-nova-selection-v1`",
                    profile_path.display(),
                    schema
                ));
            }
            if profile.selection_controls.is_empty() {
                return Err(format!(
                    "ns-nova profile `{}` enables selection_schema but declares no `selection_controls`",
                    profile_path.display()
                ));
            }
        }
    }

    let mut required = BTreeSet::new();
    required.insert(project.manifest_path.clone());
    for module in &project.modules {
        required.insert(module.path.clone());
    }
    for path in &required {
        if !include_files.iter().any(|item| item == path) {
            return Err(format!(
                "galaxy manifest `{}` is missing required project file `{}` in `include`",
                manifest_path.display(),
                path.display()
            ));
        }
    }

    Ok(CheckedGalaxy {
        root,
        manifest_path,
        manifest,
        project,
        project_plan_summary,
        include_files,
        abi_entries: abi
            .requirements
            .into_iter()
            .map(|item| (item.domain, item.abi))
            .collect(),
    })
}

fn default_manifest(
    project: &nuisc::project::LoadedProject,
    framework: Option<&str>,
) -> Result<GalaxyManifest, String> {
    let mut include = vec!["nuis.toml".to_owned()];
    for module in &project.modules {
        let relative = module
            .path
            .strip_prefix(&project.root)
            .unwrap_or(module.path.as_path())
            .display()
            .to_string();
        if !include.iter().any(|item| item == &relative) {
            include.push(relative);
        }
    }
    if framework == Some("ns-nova") && !include.iter().any(|item| item == "ns-nova.toml") {
        include.push("ns-nova.toml".to_owned());
    }
    include.sort();

    let framework = framework.map(|value| value.trim().to_owned());
    let package_kind = if framework.as_deref() == Some("ns-nova") {
        "nuis-framework".to_owned()
    } else {
        "nuis-project".to_owned()
    };
    let summary = if framework.as_deref() == Some("ns-nova") {
        format!(
            "Galaxy package for ns-nova framework project `{}`",
            project.manifest.name
        )
    } else {
        format!(
            "Galaxy package for nuis project `{}`",
            project.manifest.name
        )
    };

    Ok(GalaxyManifest {
        manifest_schema: "galaxy-manifest-v1".to_owned(),
        name: project.manifest.name.clone(),
        version: "0.1.0".to_owned(),
        package_kind,
        framework,
        project: "nuis.toml".to_owned(),
        summary,
        license: "UNLICENSED".to_owned(),
        repository: String::new(),
        authors: Vec::new(),
        include,
    })
}

fn resolve_galaxy_manifest(input: &Path) -> Result<(PathBuf, PathBuf), String> {
    if input.is_dir() {
        let manifest_path = input.join("galaxy.toml");
        return Ok((input.to_path_buf(), manifest_path));
    }
    if input.file_name().and_then(|item| item.to_str()) == Some("galaxy.toml") {
        let root = input.parent().ok_or_else(|| {
            format!(
                "galaxy manifest `{}` has no parent directory",
                input.display()
            )
        })?;
        return Ok((root.to_path_buf(), input.to_path_buf()));
    }
    let root = input
        .parent()
        .ok_or_else(|| format!("input `{}` has no parent directory", input.display()))?;
    Ok((root.to_path_buf(), root.join("galaxy.toml")))
}
