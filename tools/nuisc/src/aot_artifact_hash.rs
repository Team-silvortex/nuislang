use std::{collections::BTreeMap, path::Path};

use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::aot_domain_render::render_domain_build_unit_host_bridge_stub;
use crate::aot_domain_unit_render::{
    render_domain_build_unit_payload, render_domain_build_unit_stub,
};
use crate::aot_encoding::hex_decode_bytes;
use crate::aot_toml::{parse_required_map_string, parse_required_map_usize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArtifactHashRow {
    pub kind: String,
    pub path: String,
    pub bytes: usize,
    pub fnv1a64: String,
}

pub(crate) fn parse_artifact_hash_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<ArtifactHashRow>, String> {
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

pub(crate) fn artifact_hash_fallback_bytes(
    kind: &str,
    domain_build_units: &[BuildManifestDomainBuildUnit],
    bridge_registry_inline: Option<&str>,
    host_bridge_plan_index_inline: Option<&str>,
    lowering_plan_index_inline: Option<&str>,
) -> Option<Vec<u8>> {
    if kind == "domain_bridge_registry" {
        return bridge_registry_inline.map(|value| value.as_bytes().to_vec());
    }
    if kind == "host_bridge_plan_index" {
        return host_bridge_plan_index_inline.map(|value| value.as_bytes().to_vec());
    }
    if kind == "domain_lowering_plan_index" {
        return lowering_plan_index_inline.map(|value| value.as_bytes().to_vec());
    }

    let (prefix, domain_family) = [
        ("domain_stub_", "stub"),
        ("domain_payload_", "payload"),
        ("domain_payload_blob_", "payload_blob"),
        ("domain_bridge_stub_", "bridge_stub"),
    ]
    .iter()
    .find_map(|(prefix, kind_name)| kind.strip_prefix(prefix).map(|domain| (*kind_name, domain)))?;

    let unit = domain_build_units
        .iter()
        .find(|unit| unit.domain_family == domain_family)?;

    match prefix {
        "stub" => Some(render_domain_build_unit_stub(unit).into_bytes()),
        "payload" => render_domain_build_unit_payload(unit)
            .ok()
            .map(|value| value.into_bytes()),
        "payload_blob" => unit
            .artifact_payload_blob_inline
            .as_deref()
            .and_then(|value| hex_decode_bytes(value).ok()),
        "bridge_stub" => Some(render_domain_build_unit_host_bridge_stub(unit).into_bytes()),
        _ => None,
    }
}

fn parse_artifact_hash_row(
    values: &BTreeMap<String, String>,
    path: &Path,
) -> Result<ArtifactHashRow, String> {
    let kind = parse_required_map_string(values, "kind", path)?;
    let artifact_path = parse_required_map_string(values, "path", path)?;
    let bytes = parse_required_map_usize(values, "bytes", path)?;
    let fnv1a64 = parse_required_map_string(values, "fnv1a64", path)?;
    Ok(ArtifactHashRow {
        kind,
        path: artifact_path,
        bytes,
        fnv1a64,
    })
}
