use std::{collections::BTreeMap, fs, path::Path};

use crate::{
    toml::{
        parse_optional_map_string, parse_required_map_string_in_block, parse_required_toml_string,
        parse_required_toml_string_array, parse_required_toml_usize,
    },
    ArtifactError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostBridgePlanEntry {
    pub domain_family: String,
    pub package_id: String,
    pub bridge_stub_path: String,
    pub bridge_surface: String,
    pub scheduler_binding: String,
    pub phase_order: Vec<String>,
    pub plan_inline: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostBridgePlanIndex {
    pub schema: String,
    pub plan_count: usize,
    pub domains: Vec<String>,
    pub entries: Vec<HostBridgePlanEntry>,
}

impl HostBridgePlanIndex {
    pub fn find_by_domain_family(&self, domain_family: &str) -> Option<&HostBridgePlanEntry> {
        self.entries
            .iter()
            .find(|entry| entry.domain_family == domain_family)
    }
}

pub fn parse_host_bridge_plan_index(path: &Path) -> Result<HostBridgePlanIndex, ArtifactError> {
    let source = fs::read_to_string(path)
        .map_err(|error| ArtifactError::new(format!("failed to read `{}`: {error}", path.display())))?;
    parse_host_bridge_plan_index_from_source(&source, path)
}

pub fn parse_host_bridge_plan_index_from_source(
    source: &str,
    path: &Path,
) -> Result<HostBridgePlanIndex, ArtifactError> {
    let schema = parse_required_toml_string(source, "schema", path)?;
    let plan_count = parse_required_toml_usize(source, "plan_count", path)?;
    let domains = parse_required_toml_string_array(source, "domains", path)?;
    let entries = parse_plan_entries(source, path)?;
    Ok(HostBridgePlanIndex {
        schema,
        plan_count,
        domains,
        entries,
    })
}

fn parse_plan_entries(source: &str, path: &Path) -> Result<Vec<HostBridgePlanEntry>, ArtifactError> {
    let mut rows = Vec::new();
    let mut current = BTreeMap::<String, String>::new();
    let mut in_block = false;
    for raw in source.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[plan]]" {
            if in_block {
                rows.push(parse_plan_entry(&current, path)?);
                current.clear();
            }
            in_block = true;
            continue;
        }
        if line.starts_with('[') {
            if in_block {
                rows.push(parse_plan_entry(&current, path)?);
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
        rows.push(parse_plan_entry(&current, path)?);
    }
    Ok(rows)
}

fn parse_plan_entry(
    values: &BTreeMap<String, String>,
    path: &Path,
) -> Result<HostBridgePlanEntry, ArtifactError> {
    let phase_order_raw = parse_required_map_string_in_block(values, "phase_order", path, "plan")
        .or_else(|_| {
            let source = values.get("phase_order").cloned().unwrap_or_default();
            parse_inline_string_array(&source, path, "plan", "phase_order")
        })?;
    let phase_order = if phase_order_raw.contains(',') || phase_order_raw.is_empty() {
        phase_order_raw
            .split(',')
            .filter(|item| !item.trim().is_empty())
            .map(|item| item.trim().to_owned())
            .collect::<Vec<_>>()
    } else {
        vec![phase_order_raw]
    };
    Ok(HostBridgePlanEntry {
        domain_family: parse_required_map_string_in_block(values, "domain_family", path, "plan")?,
        package_id: parse_required_map_string_in_block(values, "package_id", path, "plan")?,
        bridge_stub_path: parse_required_map_string_in_block(values, "bridge_stub_path", path, "plan")?,
        bridge_surface: parse_required_map_string_in_block(values, "bridge_surface", path, "plan")?,
        scheduler_binding: parse_required_map_string_in_block(values, "scheduler_binding", path, "plan")?,
        phase_order,
        plan_inline: parse_optional_map_string(values, "plan_inline").unwrap_or_default(),
    })
}

fn parse_inline_string_array(
    value: &str,
    path: &Path,
    block_name: &str,
    key: &str,
) -> Result<String, ArtifactError> {
    let trimmed = value.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Err(ArtifactError::new(format!(
            "`{}` {block_name} key `{key}` must be a quoted string or string array",
            path.display()
        )));
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    let mut out = Vec::new();
    for part in inner.split(',') {
        let item = part.trim();
        if !item.starts_with('"') || !item.ends_with('"') || item.len() < 2 {
            return Err(ArtifactError::new(format!(
                "`{}` {block_name} key `{key}` contains a non-string array item",
                path.display()
            )));
        }
        out.push(item[1..item.len() - 1].to_owned());
    }
    Ok(out.join(","))
}
