use std::{collections::BTreeMap, fs, path::Path};

use crate::{
    parse_domain_build_unit_blocks,
    toml::{
        parse_optional_map_usize, parse_optional_toml_string, parse_optional_toml_usize,
        parse_required_map_string_in_block, parse_required_toml_bool, parse_required_toml_string,
        parse_required_toml_string_array, parse_required_toml_usize,
    },
    ArtifactError, BuildManifestDomainBuildUnit,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactHashEntry {
    pub kind: String,
    pub path: String,
    pub bytes: usize,
    pub fnv1a64: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildManifest {
    pub schema: String,
    pub input: String,
    pub output_dir: String,
    pub packaging_mode: String,
    pub envelope_path: String,
    pub envelope_schema: String,
    pub envelope_package_count: usize,
    pub artifact_path: String,
    pub artifact_schema: String,
    pub artifact_binary_name: String,
    pub artifact_binary_bytes: usize,
    pub lifecycle_schema: String,
    pub lifecycle_bootstrap_entry: String,
    pub lifecycle_tick_policy: String,
    pub lifecycle_shutdown_policy: String,
    pub lifecycle_yalivia_rpc: String,
    pub lifecycle_hook_surface: Vec<String>,
    pub lifecycle_export_surface: Vec<String>,
    pub lifecycle_runtime_capability_flags: Vec<String>,
    pub envelope_function_kind: String,
    pub envelope_graph_kind: String,
    pub envelope_default_time_mode: String,
    pub cpu_target_abi: String,
    pub cpu_target_machine_arch: String,
    pub cpu_target_machine_os: String,
    pub cpu_target_object_format: String,
    pub cpu_target_calling_abi: String,
    pub cpu_target_clang: String,
    pub cpu_target_cross: bool,
    pub compile_cache_status: Option<String>,
    pub compile_cache_key: Option<String>,
    pub compile_cache_root: Option<String>,
    pub project_plan_index: Option<String>,
    pub project_packet_index: Option<String>,
    pub project_plan_summary: Option<String>,
    pub bridge_registry_path: Option<String>,
    pub bridge_registry_schema: Option<String>,
    pub bridge_registry_units: usize,
    pub bridge_registry_inline: Option<String>,
    pub host_bridge_plan_index_path: Option<String>,
    pub host_bridge_plan_index_schema: Option<String>,
    pub host_bridge_plan_units: usize,
    pub host_bridge_plan_index_inline: Option<String>,
    pub clock_protocol_path: Option<String>,
    pub clock_protocol_schema: Option<String>,
    pub clock_protocol_domains: usize,
    pub clock_protocol_inline: Option<String>,
    pub artifact_hashes: Vec<ArtifactHashEntry>,
    pub execution_contract_count: usize,
    pub domain_build_units: Vec<BuildManifestDomainBuildUnit>,
}

impl BuildManifest {
    pub fn heterogeneous_domain_count(&self) -> usize {
        self.domain_build_units
            .iter()
            .filter(|unit| unit.domain_family != "cpu")
            .count()
    }
}

pub fn parse_build_manifest(path: &Path) -> Result<BuildManifest, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!("failed to read `{}`: {error}", path.display()))
    })?;
    parse_build_manifest_from_source(&source, path)
}

pub fn parse_build_manifest_from_source(
    source: &str,
    path: &Path,
) -> Result<BuildManifest, ArtifactError> {
    let schema = parse_required_toml_string(source, "manifest_schema", path)?;
    let input = parse_required_toml_string(source, "input", path)?;
    let output_dir = parse_required_toml_string(source, "output_dir", path)?;
    let packaging_mode = parse_required_toml_string(source, "packaging_mode", path)?;
    let envelope_path = parse_required_toml_string(source, "path", path)?;
    let envelope_schema = parse_required_toml_string(source, "schema", path)?;
    let envelope_package_count = parse_required_toml_usize(source, "package_count", path)?;
    let artifact_path = parse_required_toml_string(source, "artifact_path", path)?;
    let artifact_schema = parse_required_toml_string(source, "artifact_schema", path)?;
    let artifact_binary_name = parse_required_toml_string(source, "artifact_binary_name", path)?;
    let artifact_binary_bytes = parse_required_toml_usize(source, "artifact_binary_bytes", path)?;
    let lifecycle_schema = parse_required_toml_string(source, "lifecycle_schema", path)?;
    let lifecycle_bootstrap_entry =
        parse_required_toml_string(source, "lifecycle_bootstrap_entry", path)?;
    let lifecycle_tick_policy = parse_required_toml_string(source, "lifecycle_tick_policy", path)?;
    let lifecycle_shutdown_policy =
        parse_required_toml_string(source, "lifecycle_shutdown_policy", path)?;
    let lifecycle_yalivia_rpc = parse_required_toml_string(source, "lifecycle_yalivia_rpc", path)?;
    let lifecycle_hook_surface =
        parse_required_toml_string_array(source, "lifecycle_hook_surface", path)?;
    let lifecycle_export_surface =
        parse_required_toml_string_array(source, "lifecycle_export_surface", path)?;
    let lifecycle_runtime_capability_flags =
        parse_required_toml_string_array(source, "lifecycle_runtime_capability_flags", path)?;
    let envelope_function_kind = parse_required_toml_string(source, "function_kind", path)?;
    let envelope_graph_kind = parse_required_toml_string(source, "graph_kind", path)?;
    let envelope_default_time_mode = parse_required_toml_string(source, "default_time_mode", path)?;
    let cpu_target_abi = parse_required_toml_string(source, "cpu_target_abi", path)?;
    let cpu_target_machine_arch =
        parse_required_toml_string(source, "cpu_target_machine_arch", path)?;
    let cpu_target_machine_os = parse_required_toml_string(source, "cpu_target_machine_os", path)?;
    let cpu_target_object_format =
        parse_required_toml_string(source, "cpu_target_object_format", path)?;
    let cpu_target_calling_abi =
        parse_required_toml_string(source, "cpu_target_calling_abi", path)?;
    let cpu_target_clang = parse_required_toml_string(source, "cpu_target_clang", path)?;
    let cpu_target_cross = parse_required_toml_bool(source, "cpu_target_cross", path)?;
    let compile_cache_status = parse_optional_toml_string(source, "compile_cache_status");
    let compile_cache_key = parse_optional_toml_string(source, "compile_cache_key");
    let compile_cache_root = parse_optional_toml_string(source, "compile_cache_root");
    let project_plan_index = parse_optional_toml_string(source, "plan_index");
    let project_packet_index = parse_optional_toml_string(source, "packet_index");
    let project_plan_summary = parse_optional_toml_string(source, "plan_summary");
    let bridge_registry_path = parse_optional_toml_string(source, "bridge_registry_path");
    let bridge_registry_schema = parse_optional_toml_string(source, "bridge_registry_schema");
    let bridge_registry_units =
        parse_optional_toml_usize(source, "bridge_registry_units").unwrap_or(0);
    let bridge_registry_inline = parse_optional_toml_string(source, "bridge_registry_inline");
    let host_bridge_plan_index_path =
        parse_optional_toml_string(source, "host_bridge_plan_index_path");
    let host_bridge_plan_index_schema =
        parse_optional_toml_string(source, "host_bridge_plan_index_schema");
    let host_bridge_plan_units =
        parse_optional_toml_usize(source, "host_bridge_plan_units").unwrap_or(0);
    let host_bridge_plan_index_inline =
        parse_optional_toml_string(source, "host_bridge_plan_index_inline");
    let clock_protocol_path = parse_optional_toml_string(source, "clock_protocol_path");
    let clock_protocol_schema = parse_optional_toml_string(source, "clock_protocol_schema");
    let clock_protocol_domains =
        parse_optional_toml_usize(source, "clock_protocol_domains").unwrap_or(0);
    let clock_protocol_inline = parse_optional_toml_string(source, "clock_protocol_inline");
    let artifact_hashes = parse_artifact_hash_blocks(source, path)?;
    let execution_contract_count = source
        .lines()
        .filter(|line| line.trim() == "[[execution_contract]]")
        .count();
    let domain_build_units = parse_domain_build_unit_blocks(source, path)?;

    Ok(BuildManifest {
        schema,
        input,
        output_dir,
        packaging_mode,
        envelope_path,
        envelope_schema,
        envelope_package_count,
        artifact_path,
        artifact_schema,
        artifact_binary_name,
        artifact_binary_bytes,
        lifecycle_schema,
        lifecycle_bootstrap_entry,
        lifecycle_tick_policy,
        lifecycle_shutdown_policy,
        lifecycle_yalivia_rpc,
        lifecycle_hook_surface,
        lifecycle_export_surface,
        lifecycle_runtime_capability_flags,
        envelope_function_kind,
        envelope_graph_kind,
        envelope_default_time_mode,
        cpu_target_abi,
        cpu_target_machine_arch,
        cpu_target_machine_os,
        cpu_target_object_format,
        cpu_target_calling_abi,
        cpu_target_clang,
        cpu_target_cross,
        compile_cache_status,
        compile_cache_key,
        compile_cache_root,
        project_plan_index,
        project_packet_index,
        project_plan_summary,
        bridge_registry_path,
        bridge_registry_schema,
        bridge_registry_units,
        bridge_registry_inline,
        host_bridge_plan_index_path,
        host_bridge_plan_index_schema,
        host_bridge_plan_units,
        host_bridge_plan_index_inline,
        clock_protocol_path,
        clock_protocol_schema,
        clock_protocol_domains,
        clock_protocol_inline,
        artifact_hashes,
        execution_contract_count,
        domain_build_units,
    })
}

fn parse_artifact_hash_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<ArtifactHashEntry>, ArtifactError> {
    let mut rows = Vec::new();
    let mut current = BTreeMap::<String, String>::new();
    let mut in_block = false;
    for raw in source.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[artifact_hash]]" {
            if in_block {
                rows.push(parse_artifact_hash_row(&current, path)?);
                current.clear();
            }
            in_block = true;
            continue;
        }
        if line.starts_with('[') {
            if in_block {
                rows.push(parse_artifact_hash_row(&current, path)?);
                current.clear();
                in_block = false;
            }
            continue;
        }
        if in_block {
            if let Some((key, value)) = line.split_once('=') {
                current.insert(key.trim().to_owned(), value.trim().to_owned());
            }
        }
    }
    if in_block {
        rows.push(parse_artifact_hash_row(&current, path)?);
    }
    Ok(rows)
}

fn parse_artifact_hash_row(
    values: &BTreeMap<String, String>,
    path: &Path,
) -> Result<ArtifactHashEntry, ArtifactError> {
    Ok(ArtifactHashEntry {
        kind: parse_required_map_string_in_block(values, "kind", path, "artifact_hash")?,
        path: parse_required_map_string_in_block(values, "path", path, "artifact_hash")?,
        bytes: parse_optional_map_usize(values, "bytes", path, "artifact_hash")?.ok_or_else(
            || {
                ArtifactError::new(format!(
                    "`{}` artifact_hash block is missing required key `bytes`",
                    path.display()
                ))
            },
        )?,
        fnv1a64: parse_required_map_string_in_block(values, "fnv1a64", path, "artifact_hash")?,
    })
}
