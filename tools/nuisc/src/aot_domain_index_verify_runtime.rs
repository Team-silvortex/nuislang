use std::{fs, path::Path};

use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::aot_toml::{escape_toml_string, parse_required_toml_string};

use super::verify_block_count;

pub(super) fn verify_hetero_calculate_plan(
    manifest_path: &Path,
    hetero_calculate_plan_path: Option<&str>,
    hetero_calculate_plan_schema: Option<&str>,
    hetero_calculate_plan_units: usize,
    hetero_calculate_plan_inline: Option<&str>,
    heterogeneous_domain_count: usize,
    domain_build_units: &[BuildManifestDomainBuildUnit],
) -> Result<(usize, usize), String> {
    if hetero_calculate_plan_path.is_none() && hetero_calculate_plan_inline.is_none() {
        if heterogeneous_domain_count > 0 {
            return Err(format!(
                "`{}` is missing hetero calculate plan for heterogeneous domains",
                manifest_path.display()
            ));
        }
        return Ok((0, 0));
    }
    if hetero_calculate_plan_schema != Some("nuis-hetero-calculate-link-plan-v1") {
        return Err(format!(
            "`{}` has unsupported hetero calculate plan schema `{:?}`; expected `nuis-hetero-calculate-link-plan-v1`",
            manifest_path.display(),
            hetero_calculate_plan_schema
        ));
    }
    let (plan_source, plan_label) = if let Some(source) = hetero_calculate_plan_inline {
        (
            source.to_owned(),
            "<embedded-hetero-calculate-plan>".to_owned(),
        )
    } else {
        let Some(hetero_calculate_plan_path) = hetero_calculate_plan_path else {
            return Err(format!(
                "`{}` is missing hetero calculate plan path",
                manifest_path.display()
            ));
        };
        (
            fs::read_to_string(hetero_calculate_plan_path).map_err(|error| {
                format!(
                    "failed to read hetero calculate plan `{}` referenced by `{}`: {error}",
                    hetero_calculate_plan_path,
                    manifest_path.display()
                )
            })?,
            hetero_calculate_plan_path.to_owned(),
        )
    };
    let schema = parse_required_toml_string(&plan_source, "schema", Path::new(&plan_label))?;
    if schema != "nuis-hetero-calculate-link-plan-v1" {
        return Err(format!(
            "hetero calculate plan `{}` has unsupported schema `{}`",
            plan_label, schema
        ));
    }
    if !plan_source.contains("valid = true") {
        return Err(format!(
            "hetero calculate plan `{}` validation is not valid",
            plan_label
        ));
    }
    verify_block_count(
        &plan_source,
        "[[node]]",
        hetero_calculate_plan_units,
        "hetero calculate plan",
        &plan_label,
    )?;
    verify_block_count(
        &plan_source,
        "[[data_segment]]",
        hetero_calculate_plan_units,
        "hetero calculate plan",
        &plan_label,
    )?;
    if hetero_calculate_plan_units != heterogeneous_domain_count {
        return Err(format!(
            "`{}` hetero_calculate_plan_units mismatch: expected {}, found {}",
            manifest_path.display(),
            heterogeneous_domain_count,
            hetero_calculate_plan_units
        ));
    }
    let mut entries_checked = 0usize;
    for unit in domain_build_units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
    {
        if !plan_source.contains(&format!(
            "domain_family = \"{}\"",
            escape_toml_string(&unit.domain_family)
        )) {
            return Err(format!(
                "hetero calculate plan `{}` is missing domain `{}`",
                plan_label, unit.domain_family
            ));
        }
        if !plan_source.contains(&format!(
            "package_id = \"{}\"",
            escape_toml_string(&unit.package_id)
        )) && !plan_source.contains(&format!(
            "owner_package = \"{}\"",
            escape_toml_string(&unit.package_id)
        )) {
            return Err(format!(
                "hetero calculate plan `{}` is missing package `{}`",
                plan_label, unit.package_id
            ));
        }
        entries_checked += 1;
    }
    Ok((1, entries_checked))
}

pub(super) fn verify_clock_protocol(
    manifest_path: &Path,
    clock_protocol_path: Option<&str>,
    clock_protocol_schema: Option<&str>,
    clock_protocol_domains: usize,
    clock_protocol_inline: Option<&str>,
    domain_build_units: &[BuildManifestDomainBuildUnit],
) -> Result<(usize, usize), String> {
    if clock_protocol_path.is_none() && clock_protocol_inline.is_none() {
        return Err(format!(
            "`{}` is missing clock protocol artifact",
            manifest_path.display()
        ));
    }
    if clock_protocol_schema != Some("nuis-clock-protocol-v1") {
        return Err(format!(
            "`{}` has unsupported clock protocol schema `{:?}`; expected `nuis-clock-protocol-v1`",
            manifest_path.display(),
            clock_protocol_schema
        ));
    }
    let (protocol_source, protocol_label) = if let Some(source) = clock_protocol_inline {
        (source.to_owned(), "<embedded-clock-protocol>".to_owned())
    } else {
        let Some(clock_protocol_path) = clock_protocol_path else {
            return Err(format!(
                "`{}` is missing clock protocol path",
                manifest_path.display()
            ));
        };
        (
            fs::read_to_string(clock_protocol_path).map_err(|error| {
                format!(
                    "failed to read clock protocol `{}` referenced by `{}`: {error}",
                    clock_protocol_path,
                    manifest_path.display()
                )
            })?,
            clock_protocol_path.to_owned(),
        )
    };
    let schema =
        parse_required_toml_string(&protocol_source, "schema", Path::new(&protocol_label))?;
    if schema != "nuis-clock-protocol-v1" {
        return Err(format!(
            "clock protocol `{}` has unsupported schema `{}`",
            protocol_label, schema
        ));
    }
    if !protocol_source.contains("valid = true") {
        return Err(format!(
            "clock protocol `{}` validation is not valid",
            protocol_label
        ));
    }
    verify_block_count(
        &protocol_source,
        "[[clock_domain]]",
        clock_protocol_domains,
        "clock protocol",
        &protocol_label,
    )?;
    let heterogeneous_domain_count = domain_build_units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .count();
    let expected_clock_edge_count = clock_protocol_domains + heterogeneous_domain_count * 2;
    verify_block_count(
        &protocol_source,
        "[[clock_edge]]",
        expected_clock_edge_count,
        "clock protocol",
        &protocol_label,
    )?;
    let mut unique_domains = domain_build_units
        .iter()
        .map(|unit| unit.domain_family.as_str())
        .collect::<Vec<_>>();
    unique_domains.sort_unstable();
    unique_domains.dedup();
    if clock_protocol_domains != unique_domains.len() {
        return Err(format!(
            "`{}` clock_protocol_domains mismatch: expected {}, found {}",
            manifest_path.display(),
            unique_domains.len(),
            clock_protocol_domains
        ));
    }
    let mut entries_checked = 0usize;
    for domain_family in unique_domains {
        let Some(unit) = domain_build_units
            .iter()
            .find(|unit| unit.domain_family == domain_family)
        else {
            return Err(format!(
                "`{}` clock protocol domain `{domain_family}` has no source build unit",
                manifest_path.display()
            ));
        };
        if !protocol_source.contains(&format!(
            "domain_family = \"{}\"",
            escape_toml_string(&unit.domain_family)
        )) {
            return Err(format!(
                "clock protocol `{}` is missing domain `{}`",
                protocol_label, unit.domain_family
            ));
        }
        if !protocol_source.contains(&format!(
            "package_id = \"{}\"",
            escape_toml_string(&unit.package_id)
        )) {
            return Err(format!(
                "clock protocol `{}` is missing package `{}`",
                protocol_label, unit.package_id
            ));
        }
        if unit.domain_family != "cpu" {
            if !protocol_source.contains(&format!(".{}.data_commit", unit.domain_family))
                || !protocol_source.contains("relation = \"data-segment-commit\"")
            {
                return Err(format!(
                    "clock protocol `{}` is missing data segment commit edge for `{}`",
                    protocol_label, unit.domain_family
                ));
            }
            if !protocol_source.contains(&format!("source = \"hetero.data_segment.")) {
                return Err(format!(
                    "clock protocol `{}` is missing hetero data segment edge source",
                    protocol_label
                ));
            }
        }
        entries_checked += 1;
    }
    Ok((1, entries_checked + expected_clock_edge_count))
}
