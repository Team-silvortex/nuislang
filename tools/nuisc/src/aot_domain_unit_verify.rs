use std::path::Path;

use nuis_artifact::{
    parse_domain_build_unit_blocks as shared_parse_domain_build_unit_blocks,
    BuildManifestDomainBuildUnit,
};

use crate::aot_manifest_core_verify::ManifestCoreVerification;
use crate::aot_manifest_path::validate_manifest_path_in_output_dir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DomainBuildUnitVerification {
    pub execution_contracts_checked: usize,
    pub heterogeneous_domain_count: usize,
    pub domain_build_units: Vec<BuildManifestDomainBuildUnit>,
}

fn parse_domain_build_unit_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<BuildManifestDomainBuildUnit>, String> {
    shared_parse_domain_build_unit_blocks(source, path).map_err(|error| error.to_string())
}

pub(crate) fn verify_domain_build_units(
    source: &str,
    path: &Path,
    core: &ManifestCoreVerification,
) -> Result<DomainBuildUnitVerification, String> {
    let execution_contracts_checked = source
        .lines()
        .filter(|line| line.trim() == "[[execution_contract]]")
        .count();
    if execution_contracts_checked != core.envelope_package_count {
        return Err(format!(
            "`{}` execution_contract block count mismatch: envelope package_count={}, blocks={}",
            path.display(),
            core.envelope_package_count,
            execution_contracts_checked
        ));
    }

    let domain_build_units = parse_domain_build_unit_blocks(source, path)?;
    if domain_build_units.len() != core.envelope_package_count {
        return Err(format!(
            "`{}` domain_build_unit block count mismatch: envelope package_count={}, blocks={}",
            path.display(),
            core.envelope_package_count,
            domain_build_units.len()
        ));
    }

    let heterogeneous_domain_count = domain_build_units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .count();
    for unit in &domain_build_units {
        for (field, value) in [
            (
                "domain_build_unit.artifact_payload_blob_path",
                unit.artifact_payload_blob_path.as_deref(),
            ),
            (
                "domain_build_unit.artifact_stub_path",
                unit.artifact_stub_path.as_deref(),
            ),
            (
                "domain_build_unit.artifact_payload_path",
                unit.artifact_payload_path.as_deref(),
            ),
            (
                "domain_build_unit.artifact_bridge_stub_path",
                unit.artifact_bridge_stub_path.as_deref(),
            ),
            (
                "domain_build_unit.artifact_ir_sidecar_path",
                unit.artifact_ir_sidecar_path.as_deref(),
            ),
        ] {
            if let Some(value) = value {
                validate_manifest_path_in_output_dir(field, value, &core.output_dir, path)?;
            }
        }
    }

    Ok(DomainBuildUnitVerification {
        execution_contracts_checked,
        heterogeneous_domain_count,
        domain_build_units,
    })
}
