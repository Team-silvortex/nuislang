use std::{collections::BTreeMap, fs, path::Path};

use crate::{
    toml::{
        parse_optional_map_string, parse_required_map_string_in_block, parse_required_toml_string,
        parse_required_toml_string_array, parse_required_toml_usize,
    },
    ArtifactError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeRegistryEntry {
    pub domain_family: String,
    pub package_id: String,
    pub backend_family: String,
    pub selected_lowering_target: String,
    pub bridge_stub_path: String,
    pub payload_blob_path: String,
    pub plan_inline: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeRegistry {
    pub schema: String,
    pub bridge_count: usize,
    pub domains: Vec<String>,
    pub entries: Vec<BridgeRegistryEntry>,
}

impl BridgeRegistry {
    pub fn find_by_domain_family(&self, domain_family: &str) -> Option<&BridgeRegistryEntry> {
        self.entries
            .iter()
            .find(|entry| entry.domain_family == domain_family)
    }
}

pub fn parse_bridge_registry(path: &Path) -> Result<BridgeRegistry, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!("failed to read `{}`: {error}", path.display()))
    })?;
    parse_bridge_registry_from_source(&source, path)
}

pub fn parse_bridge_registry_from_source(
    source: &str,
    path: &Path,
) -> Result<BridgeRegistry, ArtifactError> {
    let schema = parse_required_toml_string(source, "schema", path)?;
    let bridge_count = parse_required_toml_usize(source, "bridge_count", path)?;
    let domains = parse_required_toml_string_array(source, "domains", path)?;
    let entries = parse_bridge_entries(source, path)?;
    Ok(BridgeRegistry {
        schema,
        bridge_count,
        domains,
        entries,
    })
}

fn parse_bridge_entries(
    source: &str,
    path: &Path,
) -> Result<Vec<BridgeRegistryEntry>, ArtifactError> {
    let mut rows = Vec::new();
    let mut current = BTreeMap::<String, String>::new();
    let mut in_block = false;
    for raw in source.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[bridge]]" {
            if in_block {
                rows.push(parse_bridge_entry(&current, path)?);
                current.clear();
            }
            in_block = true;
            continue;
        }
        if line.starts_with('[') {
            if in_block {
                rows.push(parse_bridge_entry(&current, path)?);
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
        rows.push(parse_bridge_entry(&current, path)?);
    }
    Ok(rows)
}

fn parse_bridge_entry(
    values: &BTreeMap<String, String>,
    path: &Path,
) -> Result<BridgeRegistryEntry, ArtifactError> {
    Ok(BridgeRegistryEntry {
        domain_family: parse_required_map_string_in_block(values, "domain_family", path, "bridge")?,
        package_id: parse_required_map_string_in_block(values, "package_id", path, "bridge")?,
        backend_family: parse_required_map_string_in_block(
            values,
            "backend_family",
            path,
            "bridge",
        )?,
        selected_lowering_target: parse_required_map_string_in_block(
            values,
            "selected_lowering_target",
            path,
            "bridge",
        )?,
        bridge_stub_path: parse_required_map_string_in_block(
            values,
            "bridge_stub_path",
            path,
            "bridge",
        )?,
        payload_blob_path: parse_required_map_string_in_block(
            values,
            "payload_blob_path",
            path,
            "bridge",
        )?,
        plan_inline: parse_optional_map_string(values, "plan_inline").unwrap_or_default(),
    })
}
