use std::{
    fs,
    path::{Path, PathBuf},
};

use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::aot_domain_contract::summary_for_unit as domain_build_contract_summary_for_unit;
use crate::aot_domain_profile::derived_lowering_profile_for_unit;
use crate::aot_domain_render::{
    render_domain_build_unit_bridge_plan, render_domain_build_unit_lowering_plan,
};
use crate::aot_ffi_bridge;
use crate::aot_symbol_anchor;
use crate::aot_toml::{escape_toml_string, render_string_array};

pub(crate) fn render_domain_bridge_registry(units: &[&BuildManifestDomainBuildUnit]) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-bridge-registry-v1\"\n");
    out.push_str(&format!("bridge_count = {}\n", units.len()));
    let domains = units
        .iter()
        .map(|unit| unit.domain_family.clone())
        .collect::<Vec<_>>();
    out.push_str(&format!("domains = {}\n", render_string_array(&domains)));
    for unit in units {
        out.push('\n');
        out.push_str("[[bridge]]\n");
        out.push_str(&format!(
            "domain_family = \"{}\"\n",
            escape_toml_string(&unit.domain_family)
        ));
        out.push_str(&format!(
            "package_id = \"{}\"\n",
            escape_toml_string(&unit.package_id)
        ));
        out.push_str(&format!(
            "backend_family = \"{}\"\n",
            escape_toml_string(unit.backend_family.as_deref().unwrap_or("none"))
        ));
        out.push_str(&format!(
            "vendor = \"{}\"\n",
            escape_toml_string(unit.vendor.as_deref().unwrap_or("none"))
        ));
        out.push_str(&format!(
            "device_class = \"{}\"\n",
            escape_toml_string(unit.device_class.as_deref().unwrap_or("none"))
        ));
        out.push_str(&format!(
            "selected_lowering_target = \"{}\"\n",
            escape_toml_string(unit.selected_lowering_target.as_deref().unwrap_or("none"))
        ));
        out.push_str(&format!(
            "host_ffi_bridge = \"{}\"\n",
            escape_toml_string(&aot_ffi_bridge::bridge(unit))
        ));
        out.push_str(&format!(
            "host_ffi_policy = \"{}\"\n",
            aot_ffi_bridge::SIGNATURE_WHITELIST_POLICY
        ));
        out.push_str(&format!(
            "host_ffi_symbol = \"{}\"\n",
            escape_toml_string(&aot_ffi_bridge::symbol(unit))
        ));
        out.push_str(&format!(
            "host_ffi_signature_hash = \"{}\"\n",
            escape_toml_string(&aot_ffi_bridge::signature_hash(unit))
        ));
        out.push_str(&format!(
            "bridge_stub_path = \"{}\"\n",
            escape_toml_string(
                unit.artifact_bridge_stub_path
                    .as_deref()
                    .unwrap_or("<none>")
            )
        ));
        out.push_str(&format!(
            "payload_blob_path = \"{}\"\n",
            escape_toml_string(
                unit.artifact_payload_blob_path
                    .as_deref()
                    .unwrap_or("<none>")
            )
        ));
        out.push_str(&render_domain_build_unit_bridge_plan(unit));
    }
    out
}

pub(crate) fn append_relocated_bridge_registry_manifest_section(
    out: &mut String,
    bridge_registry_path: Option<&Path>,
    units: &[BuildManifestDomainBuildUnit],
) {
    let Some(bridge_registry_path) = bridge_registry_path else {
        return;
    };
    let hetero_units = units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .collect::<Vec<_>>();
    let source = render_domain_bridge_registry(&hetero_units);
    out.push_str("[bridge_registry]\n");
    out.push_str(&format!(
        "bridge_registry_path = \"{}\"\n",
        escape_toml_string(&bridge_registry_path.display().to_string())
    ));
    out.push_str("bridge_registry_schema = \"nuis-bridge-registry-v1\"\n");
    out.push_str(&format!("bridge_registry_units = {}\n", hetero_units.len()));
    out.push_str(&format!(
        "bridge_registry_inline = \"{}\"\n",
        escape_toml_string(&source)
    ));
    out.push('\n');
}

pub(crate) fn append_relocated_host_bridge_plan_index_manifest_section(
    out: &mut String,
    host_bridge_plan_index_path: Option<&Path>,
    units: &[BuildManifestDomainBuildUnit],
) {
    let Some(host_bridge_plan_index_path) = host_bridge_plan_index_path else {
        return;
    };
    let hetero_units = units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .collect::<Vec<_>>();
    let source = render_host_bridge_plan_index(&hetero_units);
    out.push_str("[host_bridge_plan_index]\n");
    out.push_str(&format!(
        "host_bridge_plan_index_path = \"{}\"\n",
        escape_toml_string(&host_bridge_plan_index_path.display().to_string())
    ));
    out.push_str("host_bridge_plan_index_schema = \"nuis-host-bridge-plan-index-v1\"\n");
    out.push_str(&format!(
        "host_bridge_plan_units = {}\n",
        hetero_units.len()
    ));
    out.push_str(&format!(
        "host_bridge_plan_index_inline = \"{}\"\n",
        escape_toml_string(&source)
    ));
    out.push('\n');
}

pub(crate) fn append_relocated_domain_lowering_plan_index_manifest_section(
    out: &mut String,
    lowering_plan_index_path: Option<&Path>,
    units: &[BuildManifestDomainBuildUnit],
) {
    let Some(lowering_plan_index_path) = lowering_plan_index_path else {
        return;
    };
    let hetero_units = units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .collect::<Vec<_>>();
    let source = render_domain_lowering_plan_index(&hetero_units);
    out.push_str("[domain_lowering_plan_index]\n");
    out.push_str(&format!(
        "lowering_plan_index_path = \"{}\"\n",
        escape_toml_string(&lowering_plan_index_path.display().to_string())
    ));
    out.push_str("lowering_plan_index_schema = \"nuis-domain-lowering-plan-index-v1\"\n");
    out.push_str(&format!("lowering_plan_units = {}\n", hetero_units.len()));
    out.push_str(&format!(
        "lowering_plan_index_inline = \"{}\"\n",
        escape_toml_string(&source)
    ));
    out.push('\n');
}

pub(crate) fn append_build_manifest_domain_index_sections(
    out: &mut String,
    bridge_registry_path: Option<&Path>,
    bridge_registry_inline: Option<&str>,
    host_bridge_plan_index_path: Option<&Path>,
    host_bridge_plan_index_inline: Option<&str>,
    lowering_plan_index_path: Option<&Path>,
    lowering_plan_index_inline: Option<&str>,
    units: &[BuildManifestDomainBuildUnit],
) {
    let hetero_unit_count = units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .count();
    if let Some(bridge_registry_path) = bridge_registry_path {
        out.push('\n');
        out.push_str("[bridge_registry]\n");
        out.push_str(&format!(
            "bridge_registry_path = \"{}\"\n",
            escape_toml_string(&bridge_registry_path.display().to_string())
        ));
        out.push_str("bridge_registry_schema = \"nuis-bridge-registry-v1\"\n");
        out.push_str(&format!("bridge_registry_units = {hetero_unit_count}\n"));
        if let Some(source) = bridge_registry_inline {
            out.push_str(&format!(
                "bridge_registry_inline = \"{}\"\n",
                escape_toml_string(source)
            ));
        }
    }
    if let Some(host_bridge_plan_index_path) = host_bridge_plan_index_path {
        out.push('\n');
        out.push_str("[host_bridge_plan_index]\n");
        out.push_str(&format!(
            "host_bridge_plan_index_path = \"{}\"\n",
            escape_toml_string(&host_bridge_plan_index_path.display().to_string())
        ));
        out.push_str("host_bridge_plan_index_schema = \"nuis-host-bridge-plan-index-v1\"\n");
        out.push_str(&format!("host_bridge_plan_units = {hetero_unit_count}\n"));
        if let Some(source) = host_bridge_plan_index_inline {
            out.push_str(&format!(
                "host_bridge_plan_index_inline = \"{}\"\n",
                escape_toml_string(source)
            ));
        }
    }
    if let Some(lowering_plan_index_path) = lowering_plan_index_path {
        out.push('\n');
        out.push_str("[domain_lowering_plan_index]\n");
        out.push_str(&format!(
            "lowering_plan_index_path = \"{}\"\n",
            escape_toml_string(&lowering_plan_index_path.display().to_string())
        ));
        out.push_str("lowering_plan_index_schema = \"nuis-domain-lowering-plan-index-v1\"\n");
        out.push_str(&format!("lowering_plan_units = {hetero_unit_count}\n"));
        if let Some(source) = lowering_plan_index_inline {
            out.push_str(&format!(
                "lowering_plan_index_inline = \"{}\"\n",
                escape_toml_string(source)
            ));
        }
    }
}

pub(crate) fn write_domain_bridge_registry(
    output_dir: &Path,
    units: &[BuildManifestDomainBuildUnit],
) -> Result<Option<PathBuf>, String> {
    let hetero_units = units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .collect::<Vec<_>>();
    if hetero_units.is_empty() {
        return Ok(None);
    }
    let path = output_dir.join("nuis.bridge.registry.toml");
    let source = render_domain_bridge_registry(&hetero_units);
    fs::write(&path, source)
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
    Ok(Some(path))
}

pub(crate) fn write_domain_lowering_plan_index(
    output_dir: &Path,
    units: &[BuildManifestDomainBuildUnit],
) -> Result<Option<PathBuf>, String> {
    let hetero_units = units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .collect::<Vec<_>>();
    if hetero_units.is_empty() {
        return Ok(None);
    }
    let path = output_dir.join("nuis.lowering.plan-index.toml");
    let source = render_domain_lowering_plan_index(&hetero_units);
    fs::write(&path, source)
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
    Ok(Some(path))
}

pub(crate) fn write_host_bridge_plan_index(
    output_dir: &Path,
    units: &[BuildManifestDomainBuildUnit],
) -> Result<Option<PathBuf>, String> {
    let hetero_units = units
        .iter()
        .filter(|unit| unit.domain_family != "cpu")
        .collect::<Vec<_>>();
    if hetero_units.is_empty() {
        return Ok(None);
    }
    let path = output_dir.join("nuis.host-bridge.plan-index.toml");
    let source = render_host_bridge_plan_index(&hetero_units);
    fs::write(&path, source)
        .map_err(|error| format!("failed to write `{}`: {error}", path.display()))?;
    Ok(Some(path))
}

pub(crate) fn render_domain_lowering_plan_index(units: &[&BuildManifestDomainBuildUnit]) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-domain-lowering-plan-index-v1\"\n");
    out.push_str(&format!("plan_count = {}\n", units.len()));
    let domains = units
        .iter()
        .map(|unit| unit.domain_family.clone())
        .collect::<Vec<_>>();
    out.push_str(&format!("domains = {}\n", render_string_array(&domains)));
    for unit in units {
        let contract = domain_build_contract_summary_for_unit(unit);
        let profile = derived_lowering_profile_for_unit(unit);
        out.push('\n');
        out.push_str("[[lowering_plan]]\n");
        out.push_str(&format!(
            "domain_family = \"{}\"\n",
            escape_toml_string(&unit.domain_family)
        ));
        out.push_str(&format!(
            "package_id = \"{}\"\n",
            escape_toml_string(&unit.package_id)
        ));
        out.push_str(&format!(
            "contract_family = \"{}\"\n",
            escape_toml_string(&unit.contract_family)
        ));
        out.push_str(&format!(
            "backend_family = \"{}\"\n",
            escape_toml_string(unit.backend_family.as_deref().unwrap_or("none"))
        ));
        out.push_str(&format!(
            "selected_lowering_target = \"{}\"\n",
            escape_toml_string(unit.selected_lowering_target.as_deref().unwrap_or("none"))
        ));
        out.push_str(&format!(
            "lowering_profile = \"{}\"\n",
            escape_toml_string(profile.profile_key)
        ));
        out.push_str(&format!(
            "emission_kind = \"{}\"\n",
            escape_toml_string(&contract.lowering.emission_kind)
        ));
        out.push_str(&format!(
            "execution_route = \"{}\"\n",
            escape_toml_string(profile.execution_route)
        ));
        out.push_str(&format!(
            "submission_adapter = \"{}\"\n",
            escape_toml_string(profile.submission_adapter)
        ));
        out.push_str(&format!(
            "wake_adapter = \"{}\"\n",
            escape_toml_string(profile.wake_adapter)
        ));
        out.push_str(&format!(
            "symbol_namespace = \"{}\"\n",
            escape_toml_string(&aot_symbol_anchor::namespace(unit))
        ));
        out.push_str(&format!(
            "debug_anchor = \"{}\"\n",
            escape_toml_string(&aot_symbol_anchor::debug_anchor(unit))
        ));
        out.push_str(&format!(
            "linkage_anchor = \"{}\"\n",
            escape_toml_string(&aot_symbol_anchor::linkage_anchor(unit))
        ));
        out.push_str(&format!(
            "source_map_scope = \"{}\"\n",
            escape_toml_string(&aot_symbol_anchor::source_map_scope(unit))
        ));
        out.push_str(&format!(
            "host_ffi_bridge = \"{}\"\n",
            escape_toml_string(&aot_ffi_bridge::bridge(unit))
        ));
        out.push_str(&format!(
            "host_ffi_policy = \"{}\"\n",
            aot_ffi_bridge::SIGNATURE_WHITELIST_POLICY
        ));
        out.push_str(&format!(
            "host_ffi_symbol = \"{}\"\n",
            escape_toml_string(&aot_ffi_bridge::symbol(unit))
        ));
        out.push_str(&format!(
            "host_ffi_signature = \"{}\"\n",
            escape_toml_string(aot_ffi_bridge::signature())
        ));
        out.push_str(&format!(
            "host_ffi_signature_hash = \"{}\"\n",
            escape_toml_string(&aot_ffi_bridge::signature_hash(unit))
        ));
        out.push_str(&format!(
            "ir_sidecar_path = \"{}\"\n",
            escape_toml_string(unit.artifact_ir_sidecar_path.as_deref().unwrap_or("<none>"))
        ));
        out.push_str(&format!(
            "payload_blob_path = \"{}\"\n",
            escape_toml_string(
                unit.artifact_payload_blob_path
                    .as_deref()
                    .unwrap_or("<none>")
            )
        ));
        out.push_str(&format!(
            "bridge_stub_path = \"{}\"\n",
            escape_toml_string(
                unit.artifact_bridge_stub_path
                    .as_deref()
                    .unwrap_or("<none>")
            )
        ));
        out.push_str(&format!(
            "plan_inline = \"{}\"\n",
            escape_toml_string(&render_domain_build_unit_lowering_plan(unit).replace('\n', "\\n"))
        ));
    }
    out
}

pub(crate) fn render_host_bridge_plan_index(units: &[&BuildManifestDomainBuildUnit]) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-host-bridge-plan-index-v1\"\n");
    out.push_str(&format!("plan_count = {}\n", units.len()));
    let domains = units
        .iter()
        .map(|unit| unit.domain_family.clone())
        .collect::<Vec<_>>();
    out.push_str(&format!("domains = {}\n", render_string_array(&domains)));
    for unit in units {
        let contract = domain_build_contract_summary_for_unit(unit);
        out.push('\n');
        out.push_str("[[plan]]\n");
        out.push_str(&format!(
            "domain_family = \"{}\"\n",
            escape_toml_string(&unit.domain_family)
        ));
        out.push_str(&format!(
            "package_id = \"{}\"\n",
            escape_toml_string(&unit.package_id)
        ));
        out.push_str(&format!(
            "backend_family = \"{}\"\n",
            escape_toml_string(unit.backend_family.as_deref().unwrap_or("none"))
        ));
        out.push_str(&format!(
            "vendor = \"{}\"\n",
            escape_toml_string(unit.vendor.as_deref().unwrap_or("none"))
        ));
        out.push_str(&format!(
            "device_class = \"{}\"\n",
            escape_toml_string(unit.device_class.as_deref().unwrap_or("none"))
        ));
        out.push_str(&format!(
            "selected_lowering_target = \"{}\"\n",
            escape_toml_string(unit.selected_lowering_target.as_deref().unwrap_or("none"))
        ));
        out.push_str(&format!(
            "host_ffi_bridge = \"{}\"\n",
            escape_toml_string(&aot_ffi_bridge::bridge(unit))
        ));
        out.push_str(&format!(
            "host_ffi_policy = \"{}\"\n",
            aot_ffi_bridge::SIGNATURE_WHITELIST_POLICY
        ));
        out.push_str(&format!(
            "host_ffi_symbol = \"{}\"\n",
            escape_toml_string(&aot_ffi_bridge::symbol(unit))
        ));
        out.push_str(&format!(
            "host_ffi_signature_hash = \"{}\"\n",
            escape_toml_string(&aot_ffi_bridge::signature_hash(unit))
        ));
        out.push_str(&format!(
            "bridge_stub_path = \"{}\"\n",
            escape_toml_string(
                unit.artifact_bridge_stub_path
                    .as_deref()
                    .unwrap_or("<none>")
            )
        ));
        out.push_str(&format!(
            "bridge_surface = \"{}\"\n",
            escape_toml_string(&contract.bridge.bridge_surface)
        ));
        out.push_str(&format!(
            "scheduler_binding = \"{}\"\n",
            escape_toml_string(&contract.bridge.scheduler_binding)
        ));
        out.push_str(&format!(
            "phase_order = {}\n",
            render_string_array(&contract.host_bridge.phase_order)
        ));
        out.push_str(&format!(
            "plan_inline = \"{}\"\n",
            escape_toml_string(&render_domain_build_unit_bridge_plan(unit).replace('\n', "\\n"))
        ));
    }
    out
}
