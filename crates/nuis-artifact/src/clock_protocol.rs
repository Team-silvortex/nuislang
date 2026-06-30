use std::{collections::BTreeMap, fs, path::Path};

use crate::{
    toml::{
        parse_optional_map_usize, parse_required_map_string_in_block, parse_required_toml_bool,
        parse_required_toml_string, parse_required_toml_usize,
    },
    ArtifactError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClockProtocol {
    pub schema: String,
    pub mode: String,
    pub source: String,
    pub default_time_mode: String,
    pub lifecycle_tick_policy: String,
    pub validation_checked: usize,
    pub validation_valid: bool,
    pub domains: Vec<ClockDomain>,
    pub edges: Vec<ClockEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClockDomain {
    pub index: usize,
    pub domain_family: String,
    pub package_id: String,
    pub clock_domain_id: String,
    pub clock_kind: String,
    pub clock_epoch_kind: String,
    pub clock_resolution: String,
    pub clock_bridge_default: String,
    pub lifecycle_hook: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClockEdge {
    pub index: usize,
    pub from: String,
    pub to: String,
    pub relation: String,
    pub source: String,
}

impl ClockProtocol {
    pub fn find_domain(&self, domain_family: &str) -> Option<&ClockDomain> {
        self.domains
            .iter()
            .find(|domain| domain.domain_family == domain_family)
    }

    pub fn happens_before_edges(&self) -> impl Iterator<Item = &ClockEdge> {
        self.edges
            .iter()
            .filter(|edge| edge.relation == "happens-before")
    }
}

pub fn parse_clock_protocol(path: &Path) -> Result<ClockProtocol, ArtifactError> {
    let source = fs::read_to_string(path).map_err(|error| {
        ArtifactError::new(format!("failed to read `{}`: {error}", path.display()))
    })?;
    parse_clock_protocol_from_source(&source, path)
}

pub fn parse_clock_protocol_from_source(
    source: &str,
    path: &Path,
) -> Result<ClockProtocol, ArtifactError> {
    let schema = parse_required_toml_string(source, "schema", path)?;
    let mode = parse_required_toml_string(source, "mode", path)?;
    let protocol_source = parse_required_toml_string(source, "source", path)?;
    let default_time_mode = parse_required_toml_string(source, "default_time_mode", path)?;
    let lifecycle_tick_policy = parse_required_toml_string(source, "lifecycle_tick_policy", path)?;
    let validation_source = section_source(source, "[validation]");
    let validation_checked = parse_required_toml_usize(&validation_source, "checked", path)?;
    let validation_valid = parse_required_toml_bool(&validation_source, "valid", path)?;
    let domains = parse_clock_domain_blocks(source, path)?;
    let edges = parse_clock_edge_blocks(source, path)?;

    Ok(ClockProtocol {
        schema,
        mode,
        source: protocol_source,
        default_time_mode,
        lifecycle_tick_policy,
        validation_checked,
        validation_valid,
        domains,
        edges,
    })
}

fn section_source(source: &str, section: &str) -> String {
    let mut out = String::new();
    let mut in_section = false;
    for raw in source.lines() {
        let line = raw.trim();
        if line == section {
            in_section = true;
            continue;
        }
        if in_section && line.starts_with('[') {
            break;
        }
        if in_section {
            out.push_str(raw);
            out.push('\n');
        }
    }
    out
}

fn parse_clock_domain_blocks(source: &str, path: &Path) -> Result<Vec<ClockDomain>, ArtifactError> {
    parse_block_rows(source, "[[clock_domain]]")
        .into_iter()
        .map(|row| {
            Ok(ClockDomain {
                index: parse_optional_map_usize(&row, "index", path, "clock_domain")?.ok_or_else(
                    || {
                        ArtifactError::new(format!(
                            "`{}` clock_domain block is missing required key `index`",
                            path.display()
                        ))
                    },
                )?,
                domain_family: parse_required_map_string_in_block(
                    &row,
                    "domain_family",
                    path,
                    "clock_domain",
                )?,
                package_id: parse_required_map_string_in_block(
                    &row,
                    "package_id",
                    path,
                    "clock_domain",
                )?,
                clock_domain_id: parse_required_map_string_in_block(
                    &row,
                    "clock_domain_id",
                    path,
                    "clock_domain",
                )?,
                clock_kind: parse_required_map_string_in_block(
                    &row,
                    "clock_kind",
                    path,
                    "clock_domain",
                )?,
                clock_epoch_kind: parse_required_map_string_in_block(
                    &row,
                    "clock_epoch_kind",
                    path,
                    "clock_domain",
                )?,
                clock_resolution: parse_required_map_string_in_block(
                    &row,
                    "clock_resolution",
                    path,
                    "clock_domain",
                )?,
                clock_bridge_default: parse_required_map_string_in_block(
                    &row,
                    "clock_bridge_default",
                    path,
                    "clock_domain",
                )?,
                lifecycle_hook: parse_required_map_string_in_block(
                    &row,
                    "lifecycle_hook",
                    path,
                    "clock_domain",
                )?,
            })
        })
        .collect()
}

fn parse_clock_edge_blocks(source: &str, path: &Path) -> Result<Vec<ClockEdge>, ArtifactError> {
    parse_block_rows(source, "[[clock_edge]]")
        .into_iter()
        .map(|row| {
            Ok(ClockEdge {
                index: parse_optional_map_usize(&row, "index", path, "clock_edge")?.ok_or_else(
                    || {
                        ArtifactError::new(format!(
                            "`{}` clock_edge block is missing required key `index`",
                            path.display()
                        ))
                    },
                )?,
                from: parse_required_map_string_in_block(&row, "from", path, "clock_edge")?,
                to: parse_required_map_string_in_block(&row, "to", path, "clock_edge")?,
                relation: parse_required_map_string_in_block(&row, "relation", path, "clock_edge")?,
                source: parse_required_map_string_in_block(&row, "source", path, "clock_edge")?,
            })
        })
        .collect()
}

fn parse_block_rows(source: &str, block_marker: &str) -> Vec<BTreeMap<String, String>> {
    let mut rows = Vec::new();
    let mut current = BTreeMap::<String, String>::new();
    let mut in_block = false;
    for raw in source.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == block_marker {
            if in_block {
                rows.push(std::mem::take(&mut current));
            }
            in_block = true;
            continue;
        }
        if line.starts_with('[') {
            if in_block {
                rows.push(std::mem::take(&mut current));
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
        rows.push(current);
    }
    rows
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::parse_clock_protocol_from_source;

    #[test]
    fn parses_clock_protocol_domains_and_edges() {
        let source = r#"schema = "nuis-clock-protocol-v1"
mode = "heterogeneous-lifecycle-clock"
source = "registry+lifecycle+hetero-linker"
default_time_mode = "logical"
lifecycle_tick_policy = "owned-pump.bootstrap-adaptive"
[validation]
checked = 22
valid = true
issues = []
[[clock_domain]]
index = 0
domain_family = "cpu"
package_id = "official.cpu"
clock_domain_id = "cpu.clock.host.v1"
clock_kind = "host-monotonic"
clock_epoch_kind = "host-epoch"
clock_resolution = "cpu.tick_i64"
clock_bridge_default = "global->monotonic:bridge"
lifecycle_hook = "on_scheduler_tick"
[[clock_edge]]
index = 0
from = "global.clock.root.v1"
to = "cpu.clock.host.v1"
relation = "global->monotonic:bridge"
source = "nustar.clock_bridge_default"
[[clock_edge]]
index = 1
from = "t0000.nuis.bootstrap.lifecycle.v1"
to = "t0001.shader"
relation = "happens-before"
source = "hetero.node.0"
"#;
        let protocol =
            parse_clock_protocol_from_source(source, Path::new("<test-clock-protocol>")).unwrap();

        assert_eq!(protocol.schema, "nuis-clock-protocol-v1");
        assert!(protocol.validation_valid);
        assert_eq!(protocol.domains.len(), 1);
        assert_eq!(protocol.edges.len(), 2);
        assert_eq!(
            protocol.find_domain("cpu").unwrap().clock_domain_id,
            "cpu.clock.host.v1"
        );
        assert_eq!(protocol.happens_before_edges().count(), 1);
    }
}
