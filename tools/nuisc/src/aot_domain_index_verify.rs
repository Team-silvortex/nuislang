use std::{fs, path::Path};

use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::aot_domain_profile::{
    derived_lowering_profile_for_unit, registered_feature_surfaces_for_profile,
    registered_lane_groups_for_profile,
};
use crate::aot_ffi_bridge;
use crate::aot_symbol_anchor;
use crate::aot_toml::{
    escape_toml_string, parse_required_toml_string, parse_required_toml_usize, render_string_array,
};

pub(crate) struct DomainIndexVerifyReport {
    pub bridge_registry_checked: usize,
    pub bridge_registry_entries_checked: usize,
    pub host_bridge_plan_checked: usize,
    pub host_bridge_plan_entries_checked: usize,
    pub lowering_plan_index_checked: usize,
    pub lowering_plan_entries_checked: usize,
    pub clock_protocol_checked: usize,
    pub clock_protocol_entries_checked: usize,
    pub hetero_calculate_plan_checked: usize,
    pub hetero_calculate_plan_entries_checked: usize,
}

pub(crate) struct DomainIndexArtifactRef<'a> {
    pub(crate) path: Option<&'a str>,
    pub(crate) schema: Option<&'a str>,
    pub(crate) count: usize,
    pub(crate) inline: Option<&'a str>,
}

pub(crate) struct DomainIndexVerifyInput<'a> {
    pub(crate) manifest_path: &'a Path,
    pub(crate) bridge_registry: DomainIndexArtifactRef<'a>,
    pub(crate) host_bridge_plan_index: DomainIndexArtifactRef<'a>,
    pub(crate) lowering_plan_index: DomainIndexArtifactRef<'a>,
    pub(crate) clock_protocol: DomainIndexArtifactRef<'a>,
    pub(crate) hetero_calculate_plan: DomainIndexArtifactRef<'a>,
    pub(crate) heterogeneous_domain_count: usize,
    pub(crate) domain_build_units: &'a [BuildManifestDomainBuildUnit],
}

pub(crate) fn verify_domain_index_artifacts(
    input: DomainIndexVerifyInput<'_>,
) -> Result<DomainIndexVerifyReport, String> {
    let manifest_path = input.manifest_path;
    let heterogeneous_domain_count = input.heterogeneous_domain_count;
    let domain_build_units = input.domain_build_units;
    let (bridge_registry_checked, bridge_registry_entries_checked) = verify_bridge_registry(
        manifest_path,
        input.bridge_registry.path,
        input.bridge_registry.schema,
        input.bridge_registry.count,
        input.bridge_registry.inline,
        heterogeneous_domain_count,
        domain_build_units,
    )?;
    let (host_bridge_plan_checked, host_bridge_plan_entries_checked) =
        verify_host_bridge_plan_index(
            manifest_path,
            input.host_bridge_plan_index.path,
            input.host_bridge_plan_index.schema,
            input.host_bridge_plan_index.count,
            input.host_bridge_plan_index.inline,
            heterogeneous_domain_count,
            domain_build_units,
        )?;
    let (lowering_plan_index_checked, lowering_plan_entries_checked) = verify_lowering_plan_index(
        manifest_path,
        input.lowering_plan_index.path,
        input.lowering_plan_index.schema,
        input.lowering_plan_index.count,
        input.lowering_plan_index.inline,
        heterogeneous_domain_count,
        domain_build_units,
    )?;
    let (clock_protocol_checked, clock_protocol_entries_checked) = verify_clock_protocol(
        manifest_path,
        input.clock_protocol.path,
        input.clock_protocol.schema,
        input.clock_protocol.count,
        input.clock_protocol.inline,
        domain_build_units,
    )?;
    let (hetero_calculate_plan_checked, hetero_calculate_plan_entries_checked) =
        verify_hetero_calculate_plan(
            manifest_path,
            input.hetero_calculate_plan.path,
            input.hetero_calculate_plan.schema,
            input.hetero_calculate_plan.count,
            input.hetero_calculate_plan.inline,
            heterogeneous_domain_count,
            domain_build_units,
        )?;
    Ok(DomainIndexVerifyReport {
        bridge_registry_checked,
        bridge_registry_entries_checked,
        host_bridge_plan_checked,
        host_bridge_plan_entries_checked,
        lowering_plan_index_checked,
        lowering_plan_entries_checked,
        clock_protocol_checked,
        clock_protocol_entries_checked,
        hetero_calculate_plan_checked,
        hetero_calculate_plan_entries_checked,
    })
}

fn verify_bridge_registry(
    manifest_path: &Path,
    bridge_registry_path: Option<&str>,
    bridge_registry_schema: Option<&str>,
    bridge_registry_units: usize,
    bridge_registry_inline: Option<&str>,
    heterogeneous_domain_count: usize,
    domain_build_units: &[BuildManifestDomainBuildUnit],
) -> Result<(usize, usize), String> {
    if bridge_registry_path.is_none() && bridge_registry_inline.is_none() {
        if heterogeneous_domain_count > 0 {
            return Err(format!(
                "`{}` is missing bridge registry for heterogeneous domains",
                manifest_path.display()
            ));
        }
        return Ok((0, 0));
    }
    if bridge_registry_schema != Some("nuis-bridge-registry-v1") {
        return Err(format!(
            "`{}` has unsupported bridge registry schema `{:?}`; expected `nuis-bridge-registry-v1`",
            manifest_path.display(),
            bridge_registry_schema
        ));
    }
    let (registry_source, registry_label) = if let Some(source) = bridge_registry_inline {
        (source.to_owned(), "<embedded-bridge-registry>".to_owned())
    } else {
        let bridge_registry_path = bridge_registry_path.unwrap();
        (
            fs::read_to_string(bridge_registry_path).map_err(|error| {
                format!(
                    "failed to read bridge registry `{}` referenced by `{}`: {error}",
                    bridge_registry_path,
                    manifest_path.display()
                )
            })?,
            bridge_registry_path.to_owned(),
        )
    };
    let registry_schema =
        parse_required_toml_string(&registry_source, "schema", Path::new(&registry_label))?;
    if registry_schema != "nuis-bridge-registry-v1" {
        return Err(format!(
            "bridge registry `{}` has unsupported schema `{}`",
            registry_label, registry_schema
        ));
    }
    let registry_count =
        parse_required_toml_usize(&registry_source, "bridge_count", Path::new(&registry_label))?;
    if registry_count != bridge_registry_units {
        return Err(format!(
            "bridge registry `{}` count mismatch: manifest={}, registry={}",
            registry_label, bridge_registry_units, registry_count
        ));
    }
    verify_block_count(
        &registry_source,
        "[[bridge]]",
        bridge_registry_units,
        "bridge registry",
        &registry_label,
    )?;
    if bridge_registry_units != heterogeneous_domain_count {
        return Err(format!(
            "`{}` bridge_registry_units mismatch: expected {}, found {}",
            manifest_path.display(),
            heterogeneous_domain_count,
            bridge_registry_units
        ));
    }
    let mut entries_checked = 0usize;
    for unit in domain_build_units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
    {
        verify_common_bridge_fields(&registry_source, &registry_label, "bridge registry", unit)?;
        entries_checked += 1;
    }
    Ok((1, entries_checked))
}

fn verify_host_bridge_plan_index(
    manifest_path: &Path,
    host_bridge_plan_index_path: Option<&str>,
    host_bridge_plan_index_schema: Option<&str>,
    host_bridge_plan_units: usize,
    host_bridge_plan_index_inline: Option<&str>,
    heterogeneous_domain_count: usize,
    domain_build_units: &[BuildManifestDomainBuildUnit],
) -> Result<(usize, usize), String> {
    if host_bridge_plan_index_path.is_none() && host_bridge_plan_index_inline.is_none() {
        if heterogeneous_domain_count > 0 {
            return Err(format!(
                "`{}` is missing host bridge plan index for heterogeneous domains",
                manifest_path.display()
            ));
        }
        return Ok((0, 0));
    }
    if host_bridge_plan_index_schema != Some("nuis-host-bridge-plan-index-v1") {
        return Err(format!(
            "`{}` has unsupported host bridge plan index schema `{:?}`; expected `nuis-host-bridge-plan-index-v1`",
            manifest_path.display(),
            host_bridge_plan_index_schema
        ));
    }
    let (plan_index_source, plan_index_label) = if let Some(source) = host_bridge_plan_index_inline
    {
        (
            source.to_owned(),
            "<embedded-host-bridge-plan-index>".to_owned(),
        )
    } else {
        let host_bridge_plan_index_path = host_bridge_plan_index_path.unwrap();
        (
            fs::read_to_string(host_bridge_plan_index_path).map_err(|error| {
                format!(
                    "failed to read host bridge plan index `{}` referenced by `{}`: {error}",
                    host_bridge_plan_index_path,
                    manifest_path.display()
                )
            })?,
            host_bridge_plan_index_path.to_owned(),
        )
    };
    let index_schema =
        parse_required_toml_string(&plan_index_source, "schema", Path::new(&plan_index_label))?;
    if index_schema != "nuis-host-bridge-plan-index-v1" {
        return Err(format!(
            "host bridge plan index `{}` has unsupported schema `{}`",
            plan_index_label, index_schema
        ));
    }
    verify_plan_count(
        &plan_index_source,
        host_bridge_plan_units,
        "host bridge plan index",
        &plan_index_label,
        "index",
    )?;
    verify_block_count(
        &plan_index_source,
        "[[plan]]",
        host_bridge_plan_units,
        "host bridge plan index",
        &plan_index_label,
    )?;
    if host_bridge_plan_units != heterogeneous_domain_count {
        return Err(format!(
            "`{}` host_bridge_plan_units mismatch: expected {}, found {}",
            manifest_path.display(),
            heterogeneous_domain_count,
            host_bridge_plan_units
        ));
    }
    let mut entries_checked = 0usize;
    for unit in domain_build_units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
    {
        verify_common_bridge_fields(
            &plan_index_source,
            &plan_index_label,
            "host bridge plan index",
            unit,
        )?;
        entries_checked += 1;
    }
    Ok((1, entries_checked))
}

fn verify_lowering_plan_index(
    manifest_path: &Path,
    lowering_plan_index_path: Option<&str>,
    lowering_plan_index_schema: Option<&str>,
    lowering_plan_units: usize,
    lowering_plan_index_inline: Option<&str>,
    heterogeneous_domain_count: usize,
    domain_build_units: &[BuildManifestDomainBuildUnit],
) -> Result<(usize, usize), String> {
    if lowering_plan_index_path.is_none() && lowering_plan_index_inline.is_none() {
        if heterogeneous_domain_count > 0 {
            return Err(format!(
                "`{}` is missing domain lowering plan index for heterogeneous domains",
                manifest_path.display()
            ));
        }
        return Ok((0, 0));
    }
    if lowering_plan_index_schema != Some("nuis-domain-lowering-plan-index-v1") {
        return Err(format!(
            "`{}` has unsupported lowering plan index schema `{:?}`; expected `nuis-domain-lowering-plan-index-v1`",
            manifest_path.display(),
            lowering_plan_index_schema
        ));
    }
    let (index_source, index_label) = if let Some(source) = lowering_plan_index_inline {
        (
            source.to_owned(),
            "<embedded-domain-lowering-plan-index>".to_owned(),
        )
    } else {
        let lowering_plan_index_path = lowering_plan_index_path.unwrap();
        (
            fs::read_to_string(lowering_plan_index_path).map_err(|error| {
                format!(
                    "failed to read domain lowering plan index `{}` referenced by `{}`: {error}",
                    lowering_plan_index_path,
                    manifest_path.display()
                )
            })?,
            lowering_plan_index_path.to_owned(),
        )
    };
    let index_schema =
        parse_required_toml_string(&index_source, "schema", Path::new(&index_label))?;
    if index_schema != "nuis-domain-lowering-plan-index-v1" {
        return Err(format!(
            "domain lowering plan index `{}` has unsupported schema `{}`",
            index_label, index_schema
        ));
    }
    verify_plan_count(
        &index_source,
        lowering_plan_units,
        "domain lowering plan index",
        &index_label,
        "index",
    )?;
    verify_block_count(
        &index_source,
        "[[lowering_plan]]",
        lowering_plan_units,
        "domain lowering plan index",
        &index_label,
    )?;
    if lowering_plan_units != heterogeneous_domain_count {
        return Err(format!(
            "`{}` lowering_plan_units mismatch: expected {}, found {}",
            manifest_path.display(),
            heterogeneous_domain_count,
            lowering_plan_units
        ));
    }
    let mut entries_checked = 0usize;
    for unit in domain_build_units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
    {
        verify_lowering_plan_fields(&index_source, &index_label, unit)?;
        entries_checked += 1;
    }
    Ok((1, entries_checked))
}

fn verify_hetero_calculate_plan(
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
        let hetero_calculate_plan_path = hetero_calculate_plan_path.unwrap();
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

fn verify_clock_protocol(
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
        let clock_protocol_path = clock_protocol_path.unwrap();
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
        let unit = domain_build_units
            .iter()
            .find(|unit| unit.domain_family == domain_family)
            .expect("unique domain must have source unit");
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
        entries_checked += 1;
    }
    Ok((1, entries_checked))
}

fn verify_common_bridge_fields(
    source: &str,
    label: &str,
    index_kind: &str,
    unit: &BuildManifestDomainBuildUnit,
) -> Result<(), String> {
    let expected_bridge_stub = unit
        .artifact_bridge_stub_path
        .as_deref()
        .unwrap_or("<none>");
    let expected_host_ffi_bridge = aot_ffi_bridge::bridge(unit);
    let expected_host_ffi_symbol = aot_ffi_bridge::symbol(unit);
    let expected_host_ffi_signature_hash = aot_ffi_bridge::signature_hash(unit);
    for (field, expected) in [
        ("bridge_stub_path", expected_bridge_stub),
        ("host_ffi_bridge", expected_host_ffi_bridge.as_str()),
        (
            "host_ffi_policy",
            aot_ffi_bridge::SIGNATURE_WHITELIST_POLICY,
        ),
        ("host_ffi_symbol", expected_host_ffi_symbol.as_str()),
        (
            "host_ffi_signature_hash",
            expected_host_ffi_signature_hash.as_str(),
        ),
    ] {
        if !source.contains(&format!("{field} = \"{}\"", escape_toml_string(expected))) {
            return Err(format!(
                "{index_kind} `{}` is missing {field} for `{}`",
                label, unit.domain_family
            ));
        }
    }
    Ok(())
}

fn verify_lowering_plan_fields(
    source: &str,
    label: &str,
    unit: &BuildManifestDomainBuildUnit,
) -> Result<(), String> {
    let expected_target = unit.selected_lowering_target.as_deref().unwrap_or("none");
    let expected_ir_sidecar = unit.artifact_ir_sidecar_path.as_deref().unwrap_or("<none>");
    let expected_payload_blob = unit
        .artifact_payload_blob_path
        .as_deref()
        .unwrap_or("<none>");
    let expected_bridge_stub = unit
        .artifact_bridge_stub_path
        .as_deref()
        .unwrap_or("<none>");
    let expected_symbol_namespace = aot_symbol_anchor::namespace(unit);
    let expected_debug_anchor = aot_symbol_anchor::debug_anchor(unit);
    let expected_linkage_anchor = aot_symbol_anchor::linkage_anchor(unit);
    let expected_source_map_scope = aot_symbol_anchor::source_map_scope(unit);
    let expected_host_ffi_bridge = aot_ffi_bridge::bridge(unit);
    let expected_host_ffi_symbol = aot_ffi_bridge::symbol(unit);
    let expected_host_ffi_signature_hash = aot_ffi_bridge::signature_hash(unit);
    for (field, expected) in [
        ("selected_lowering_target", expected_target),
        ("symbol_namespace", expected_symbol_namespace.as_str()),
        ("debug_anchor", expected_debug_anchor.as_str()),
        ("linkage_anchor", expected_linkage_anchor.as_str()),
        ("source_map_scope", expected_source_map_scope.as_str()),
        ("host_ffi_bridge", expected_host_ffi_bridge.as_str()),
        (
            "host_ffi_policy",
            aot_ffi_bridge::SIGNATURE_WHITELIST_POLICY,
        ),
        ("host_ffi_symbol", expected_host_ffi_symbol.as_str()),
        ("host_ffi_signature", aot_ffi_bridge::signature()),
        (
            "host_ffi_signature_hash",
            expected_host_ffi_signature_hash.as_str(),
        ),
        ("ir_sidecar_path", expected_ir_sidecar),
        ("payload_blob_path", expected_payload_blob),
        ("bridge_stub_path", expected_bridge_stub),
    ] {
        if !source.contains(&format!("{field} = \"{}\"", escape_toml_string(expected))) {
            return Err(format!(
                "domain lowering plan index `{}` is missing {field} for `{}`",
                label, unit.domain_family
            ));
        }
    }
    let profile = derived_lowering_profile_for_unit(unit);
    if let Some(feature_surfaces) = registered_feature_surfaces_for_profile(unit, &profile) {
        let expected = render_string_array(
            &feature_surfaces
                .iter()
                .map(|surface| (*surface).to_owned())
                .collect::<Vec<_>>(),
        );
        if !source.contains(&format!("registered_feature_surfaces = {expected}")) {
            return Err(format!(
                "domain lowering plan index `{}` is missing registered_feature_surfaces for `{}`",
                label, unit.domain_family
            ));
        }
    }
    if let Some(lane_groups) = registered_lane_groups_for_profile(unit, &profile) {
        let expected = render_string_array(
            &lane_groups
                .iter()
                .map(|lane| (*lane).to_owned())
                .collect::<Vec<_>>(),
        );
        if !source.contains(&format!("registered_lane_groups = {expected}")) {
            return Err(format!(
                "domain lowering plan index `{}` is missing registered_lane_groups for `{}`",
                label, unit.domain_family
            ));
        }
    }
    Ok(())
}

fn verify_plan_count(
    source: &str,
    expected_units: usize,
    index_kind: &str,
    label: &str,
    actual_label: &str,
) -> Result<(), String> {
    let plan_count = parse_required_toml_usize(source, "plan_count", Path::new(label))?;
    if plan_count != expected_units {
        return Err(format!(
            "{index_kind} `{label}` count mismatch: manifest={expected_units}, {actual_label}={plan_count}"
        ));
    }
    Ok(())
}

fn verify_block_count(
    source: &str,
    marker: &str,
    expected_units: usize,
    index_kind: &str,
    label: &str,
) -> Result<(), String> {
    let block_count = source.lines().filter(|line| line.trim() == marker).count();
    if block_count != expected_units {
        return Err(format!(
            "{index_kind} `{label}` block count mismatch: manifest={expected_units}, blocks={block_count}"
        ));
    }
    Ok(())
}
