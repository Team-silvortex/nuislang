use std::{collections::BTreeMap, path::Path};

use crate::{
    toml::{
        parse_optional_map_string, parse_optional_map_usize, parse_required_map_string_in_block,
    },
    ArtifactError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildManifestDomainBuildUnit {
    pub package_id: String,
    pub domain_family: String,
    pub abi: Option<String>,
    pub machine_arch: Option<String>,
    pub machine_os: Option<String>,
    pub backend_family: Option<String>,
    pub selected_lowering_target: Option<String>,
    pub artifact_stub_path: Option<String>,
    pub artifact_payload_path: Option<String>,
    pub artifact_bridge_stub_path: Option<String>,
    pub artifact_payload_blob_path: Option<String>,
    pub artifact_payload_blob_bytes: Option<usize>,
    pub artifact_payload_format: Option<String>,
    pub contract_family: String,
    pub packaging_role: String,
}

impl BuildManifestDomainBuildUnit {
    pub fn is_heterogeneous(&self) -> bool {
        self.domain_family != "cpu"
    }
}

pub fn parse_domain_build_unit_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<BuildManifestDomainBuildUnit>, ArtifactError> {
    let mut rows = Vec::new();
    let mut current = BTreeMap::<String, String>::new();
    let mut in_block = false;
    for raw in source.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[domain_build_unit]]" {
            if in_block {
                rows.push(parse_domain_build_unit_row(&current, path)?);
                current.clear();
            }
            in_block = true;
            continue;
        }
        if line.starts_with('[') {
            if in_block {
                rows.push(parse_domain_build_unit_row(&current, path)?);
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
        rows.push(parse_domain_build_unit_row(&current, path)?);
    }
    Ok(rows)
}

fn parse_domain_build_unit_row(
    values: &BTreeMap<String, String>,
    path: &Path,
) -> Result<BuildManifestDomainBuildUnit, ArtifactError> {
    Ok(BuildManifestDomainBuildUnit {
        package_id: parse_required_map_string_in_block(values, "package_id", path, "domain_build_unit")?,
        domain_family: parse_required_map_string_in_block(values, "domain_family", path, "domain_build_unit")?,
        abi: parse_optional_map_string(values, "abi"),
        machine_arch: parse_optional_map_string(values, "machine_arch"),
        machine_os: parse_optional_map_string(values, "machine_os"),
        backend_family: parse_optional_map_string(values, "backend_family"),
        selected_lowering_target: parse_optional_map_string(values, "selected_lowering_target"),
        artifact_stub_path: parse_optional_map_string(values, "artifact_stub_path"),
        artifact_payload_path: parse_optional_map_string(values, "artifact_payload_path"),
        artifact_bridge_stub_path: parse_optional_map_string(values, "artifact_bridge_stub_path"),
        artifact_payload_blob_path: parse_optional_map_string(values, "artifact_payload_blob_path"),
        artifact_payload_blob_bytes: parse_optional_map_usize(values, "artifact_payload_blob_bytes", path, "domain_build_unit")?,
        artifact_payload_format: parse_optional_map_string(values, "artifact_payload_format"),
        contract_family: parse_required_map_string_in_block(values, "contract_family", path, "domain_build_unit")?,
        packaging_role: parse_required_map_string_in_block(values, "packaging_role", path, "domain_build_unit")?,
    })
}
