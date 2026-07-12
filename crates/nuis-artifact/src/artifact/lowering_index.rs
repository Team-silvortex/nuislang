use std::{collections::BTreeMap, path::Path};

use crate::{
    parse_build_manifest_from_source,
    toml::{
        parse_optional_map_string, parse_optional_map_usize, parse_required_map_string_in_block,
        parse_required_toml_string, parse_required_toml_usize,
    },
    ArtifactError, BuildManifestDomainBuildUnit,
};

use super::{NuisLoweringIndex, NuisLoweringIndexUnit};

pub fn parse_nuis_lowering_index_from_source(
    source: &str,
    path: &Path,
) -> Result<NuisLoweringIndex, ArtifactError> {
    let schema = parse_required_toml_string(source, "schema", path)?;
    let packaging_mode = parse_required_toml_string(source, "packaging_mode", path)?;
    let domain_unit_count = parse_required_toml_usize(source, "domain_unit_count", path)?;
    let units = parse_lowering_unit_blocks(source, path)?;
    if units.len() != domain_unit_count {
        return Err(ArtifactError::new(format!(
            "`{}` lowering index domain_unit_count mismatch: declared={}, actual={}",
            path.display(),
            domain_unit_count,
            units.len()
        )));
    }
    Ok(NuisLoweringIndex {
        schema,
        packaging_mode,
        domain_unit_count,
        units,
    })
}

fn parse_lowering_unit_blocks(
    source: &str,
    path: &Path,
) -> Result<Vec<NuisLoweringIndexUnit>, ArtifactError> {
    let mut rows = Vec::new();
    let mut current = BTreeMap::<String, String>::new();
    let mut in_block = false;
    for raw in source.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[lowering_unit]]" {
            if in_block {
                rows.push(parse_lowering_unit_row(&current, path)?);
                current.clear();
            }
            in_block = true;
            continue;
        }
        if line.starts_with('[') {
            if in_block {
                rows.push(parse_lowering_unit_row(&current, path)?);
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
        rows.push(parse_lowering_unit_row(&current, path)?);
    }
    Ok(rows)
}

fn parse_lowering_unit_row(
    values: &BTreeMap<String, String>,
    path: &Path,
) -> Result<NuisLoweringIndexUnit, ArtifactError> {
    Ok(NuisLoweringIndexUnit {
        package_id: parse_required_map_string_in_block(
            values,
            "package_id",
            path,
            "lowering_unit",
        )?,
        domain_family: parse_required_map_string_in_block(
            values,
            "domain_family",
            path,
            "lowering_unit",
        )?,
        backend_family: parse_optional_map_string(values, "backend_family"),
        target_device: parse_optional_map_string(values, "target_device"),
        ir_format: parse_optional_map_string(values, "ir_format"),
        dispatch_abi: parse_optional_map_string(values, "dispatch_abi"),
        backend_priority: parse_optional_map_usize(
            values,
            "backend_priority",
            path,
            "lowering_unit",
        )?,
        verification: parse_optional_map_string(values, "verification"),
        selected_lowering_target: parse_optional_map_string(values, "selected_lowering_target"),
        artifact_ir_sidecar_path: parse_optional_map_string(values, "artifact_ir_sidecar_path"),
        contract_family: parse_required_map_string_in_block(
            values,
            "contract_family",
            path,
            "lowering_unit",
        )?,
        packaging_role: parse_required_map_string_in_block(
            values,
            "packaging_role",
            path,
            "lowering_unit",
        )?,
    })
}

pub(super) fn validate_lowering_index_against_build_manifest(
    lowering_index_source: &str,
    build_manifest_source: &str,
) -> Result<(), ArtifactError> {
    let lowering_index = parse_nuis_lowering_index_from_source(
        lowering_index_source,
        Path::new("<compiled-artifact-lowering-index>"),
    )?;
    let build_manifest = parse_build_manifest_from_source(
        build_manifest_source,
        Path::new("<compiled-artifact-build-manifest>"),
    )?;
    if lowering_index.packaging_mode != build_manifest.packaging_mode {
        return Err(ArtifactError::new(format!(
            "compiled artifact lowering index packaging_mode `{}` does not match build manifest packaging_mode `{}`",
            lowering_index.packaging_mode, build_manifest.packaging_mode
        )));
    }
    if lowering_index.units.len() != build_manifest.domain_build_units.len() {
        return Err(ArtifactError::new(format!(
            "compiled artifact lowering index unit count `{}` does not match build manifest domain build unit count `{}`",
            lowering_index.units.len(),
            build_manifest.domain_build_units.len()
        )));
    }
    for (index, (lowering_unit, manifest_unit)) in lowering_index
        .units
        .iter()
        .zip(build_manifest.domain_build_units.iter())
        .enumerate()
    {
        validate_lowering_unit_matches_manifest_unit(index, lowering_unit, manifest_unit)?;
    }
    Ok(())
}

fn validate_lowering_unit_matches_manifest_unit(
    index: usize,
    lowering_unit: &NuisLoweringIndexUnit,
    manifest_unit: &BuildManifestDomainBuildUnit,
) -> Result<(), ArtifactError> {
    let unit_label = format!(
        "lowering unit #{index} ({}/{})",
        lowering_unit.package_id, lowering_unit.domain_family
    );
    validate_lowering_unit_string(
        &unit_label,
        "package_id",
        &lowering_unit.package_id,
        &manifest_unit.package_id,
    )?;
    validate_lowering_unit_string(
        &unit_label,
        "domain_family",
        &lowering_unit.domain_family,
        &manifest_unit.domain_family,
    )?;
    validate_lowering_unit_option(
        &unit_label,
        "backend_family",
        lowering_unit.backend_family.as_deref(),
        manifest_unit.backend_family.as_deref(),
    )?;
    validate_lowering_unit_option(
        &unit_label,
        "target_device",
        lowering_unit.target_device.as_deref(),
        manifest_unit.target_device.as_deref(),
    )?;
    validate_lowering_unit_option(
        &unit_label,
        "ir_format",
        lowering_unit.ir_format.as_deref(),
        manifest_unit.ir_format.as_deref(),
    )?;
    validate_lowering_unit_option(
        &unit_label,
        "dispatch_abi",
        lowering_unit.dispatch_abi.as_deref(),
        manifest_unit.dispatch_abi.as_deref(),
    )?;
    validate_lowering_unit_option(
        &unit_label,
        "backend_priority",
        lowering_unit
            .backend_priority
            .map(|value| value.to_string())
            .as_deref(),
        manifest_unit
            .backend_priority
            .map(|value| value.to_string())
            .as_deref(),
    )?;
    validate_lowering_unit_option(
        &unit_label,
        "verification",
        lowering_unit.verification.as_deref(),
        manifest_unit.verification.as_deref(),
    )?;
    validate_lowering_unit_option(
        &unit_label,
        "selected_lowering_target",
        lowering_unit.selected_lowering_target.as_deref(),
        manifest_unit.selected_lowering_target.as_deref(),
    )?;
    validate_lowering_unit_option(
        &unit_label,
        "artifact_ir_sidecar_path",
        lowering_unit.artifact_ir_sidecar_path.as_deref(),
        manifest_unit.artifact_ir_sidecar_path.as_deref(),
    )?;
    validate_lowering_unit_string(
        &unit_label,
        "contract_family",
        &lowering_unit.contract_family,
        &manifest_unit.contract_family,
    )?;
    validate_lowering_unit_string(
        &unit_label,
        "packaging_role",
        &lowering_unit.packaging_role,
        &manifest_unit.packaging_role,
    )
}

fn validate_lowering_unit_string(
    unit_label: &str,
    field: &str,
    lowering_value: &str,
    manifest_value: &str,
) -> Result<(), ArtifactError> {
    if lowering_value == manifest_value {
        return Ok(());
    }
    Err(ArtifactError::new(format!(
        "compiled artifact lowering index {unit_label} field `{field}` value `{lowering_value}` does not match build manifest value `{manifest_value}`"
    )))
}

fn validate_lowering_unit_option(
    unit_label: &str,
    field: &str,
    lowering_value: Option<&str>,
    manifest_value: Option<&str>,
) -> Result<(), ArtifactError> {
    if lowering_value == manifest_value {
        return Ok(());
    }
    Err(ArtifactError::new(format!(
        "compiled artifact lowering index {unit_label} field `{field}` value `{}` does not match build manifest value `{}`",
        lowering_value.unwrap_or("<none>"),
        manifest_value.unwrap_or("<none>")
    )))
}
