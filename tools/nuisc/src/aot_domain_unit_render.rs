use std::path::Path;

use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::aot_toml::{escape_toml_string, render_string_array};

pub(crate) fn render_domain_build_unit_manifest_block(
    unit: &BuildManifestDomainBuildUnit,
) -> String {
    let mut out = String::new();
    out.push_str("[[domain_build_unit]]\n");
    append_common_unit_fields(&mut out, unit);
    if let Some(value) = &unit.artifact_stub_path {
        out.push_str(&format!(
            "artifact_stub_path = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    append_inline_artifact_fields(&mut out, unit);
    out.push_str(&format!(
        "contract_family = \"{}\"\n",
        escape_toml_string(&unit.contract_family)
    ));
    out.push_str(&format!(
        "packaging_role = \"{}\"\n",
        escape_toml_string(&unit.packaging_role)
    ));
    out.push('\n');
    out
}

pub(crate) fn append_domain_build_unit_manifest_sections(
    out: &mut String,
    units: &[BuildManifestDomainBuildUnit],
) {
    for unit in units {
        out.push('\n');
        out.push_str(render_domain_build_unit_manifest_block(unit).trim_end());
        out.push('\n');
    }
}

pub(crate) fn render_domain_build_unit_stub(unit: &BuildManifestDomainBuildUnit) -> String {
    let mut out = String::new();
    out.push_str("schema = \"nuis-domain-build-unit-v1\"\n");
    append_common_unit_fields(&mut out, unit);
    append_inline_artifact_fields(&mut out, unit);
    out.push_str(&format!(
        "contract_family = \"{}\"\n",
        escape_toml_string(&unit.contract_family)
    ));
    out.push_str(&format!(
        "packaging_role = \"{}\"\n",
        escape_toml_string(&unit.packaging_role)
    ));
    out
}

pub(crate) fn render_domain_build_unit_payload(
    unit: &BuildManifestDomainBuildUnit,
) -> Result<String, String> {
    let manifest = crate::registry::load_manifest_for_domain(
        Path::new("nustar-packages"),
        &unit.domain_family,
    )?;
    let capability = crate::registry::capability_summary(&manifest);
    let execution = crate::registry::execution_summary(&manifest);
    let mut out = String::new();
    out.push_str("schema = \"nuis-domain-build-payload-v1\"\n");
    out.push_str(&format!(
        "package_id = \"{}\"\n",
        escape_toml_string(&unit.package_id)
    ));
    out.push_str(&format!(
        "domain_family = \"{}\"\n",
        escape_toml_string(&unit.domain_family)
    ));
    if let Some(value) = &unit.abi {
        out.push_str(&format!("abi = \"{}\"\n", escape_toml_string(value)));
    }
    if let Some(value) = &unit.backend_family {
        out.push_str(&format!(
            "backend_family = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = &unit.vendor {
        out.push_str(&format!("vendor = \"{}\"\n", escape_toml_string(value)));
    }
    if let Some(value) = &unit.device_class {
        out.push_str(&format!(
            "device_class = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = &unit.selected_lowering_target {
        out.push_str(&format!(
            "selected_lowering_target = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    out.push_str(&format!(
        "contract_family = \"{}\"\n",
        escape_toml_string(&unit.contract_family)
    ));
    out.push_str("payload_kind = \"contract-sidecar\"\n");
    out.push_str("payload_format = \"toml\"\n");
    out.push_str(&format!(
        "frontend = \"{}\"\n",
        escape_toml_string(&manifest.frontend)
    ));
    out.push_str(&format!(
        "entry_crate = \"{}\"\n",
        escape_toml_string(&manifest.entry_crate)
    ));
    out.push_str(&format!(
        "loader_abi = \"{}\"\n",
        escape_toml_string(&manifest.loader_abi)
    ));
    out.push_str(&format!(
        "loader_entry = \"{}\"\n",
        escape_toml_string(&manifest.loader_entry)
    ));
    out.push_str(&format!(
        "clock_domain_id = \"{}\"\n",
        escape_toml_string(&capability.clock.domain_id)
    ));
    out.push_str(&format!(
        "clock_kind = \"{}\"\n",
        escape_toml_string(&capability.clock.kind)
    ));
    out.push_str(&format!(
        "clock_epoch_kind = \"{}\"\n",
        escape_toml_string(&capability.clock.epoch_kind)
    ));
    out.push_str(&format!(
        "clock_resolution = \"{}\"\n",
        escape_toml_string(&capability.clock.resolution)
    ));
    out.push_str(&format!(
        "clock_bridge_default = \"{}\"\n",
        escape_toml_string(&capability.clock.bridge_default)
    ));
    out.push_str(&format!(
        "execution_skeleton_version = \"{}\"\n",
        escape_toml_string(&execution.skeleton_version)
    ));
    out.push_str(&format!(
        "execution_function_kind = \"{}\"\n",
        escape_toml_string(&execution.function_kind)
    ));
    out.push_str(&format!(
        "execution_graph_kind = \"{}\"\n",
        escape_toml_string(&execution.graph_kind)
    ));
    out.push_str(&format!(
        "execution_default_time_mode = \"{}\"\n",
        escape_toml_string(&execution.default_time_mode)
    ));
    out.push_str(&format!(
        "packaging_role = \"{}\"\n",
        escape_toml_string(&unit.packaging_role)
    ));
    out.push_str(&format!(
        "support_surface = {}\n",
        render_string_array(&capability.support_surface)
    ));
    out.push_str(&format!(
        "support_profile_slots = {}\n",
        render_string_array(&capability.support_profile_slots)
    ));
    out.push_str(&format!(
        "default_lanes = {}\n",
        render_string_array(&capability.default_lanes)
    ));
    out.push_str(&format!(
        "resource_families = {}\n",
        render_string_array(&manifest.resource_families)
    ));
    out.push_str(&format!(
        "unit_types = {}\n",
        render_string_array(&manifest.unit_types)
    ));
    out.push_str(&format!(
        "lowering_targets = {}\n",
        render_string_array(&execution.lowering_targets)
    ));
    out.push_str(&format!("ops = {}\n", render_string_array(&manifest.ops)));
    out.push_str(&format!(
        "host_ffi_surface = {}\n",
        render_string_array(&manifest.host_ffi_surface)
    ));
    out.push_str(&format!(
        "host_ffi_abis = {}\n",
        render_string_array(&manifest.host_ffi_abis)
    ));
    if !manifest.host_ffi_bridge.is_empty() {
        out.push_str(&format!(
            "host_ffi_bridge = \"{}\"\n",
            escape_toml_string(&manifest.host_ffi_bridge)
        ));
    }
    Ok(out)
}

fn append_common_unit_fields(out: &mut String, unit: &BuildManifestDomainBuildUnit) {
    out.push_str(&format!(
        "package_id = \"{}\"\n",
        escape_toml_string(&unit.package_id)
    ));
    out.push_str(&format!(
        "domain_family = \"{}\"\n",
        escape_toml_string(&unit.domain_family)
    ));
    if let Some(value) = &unit.abi {
        out.push_str(&format!("abi = \"{}\"\n", escape_toml_string(value)));
    }
    if let Some(value) = &unit.machine_arch {
        out.push_str(&format!(
            "machine_arch = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = &unit.machine_os {
        out.push_str(&format!("machine_os = \"{}\"\n", escape_toml_string(value)));
    }
    if let Some(value) = &unit.backend_family {
        out.push_str(&format!(
            "backend_family = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = &unit.vendor {
        out.push_str(&format!("vendor = \"{}\"\n", escape_toml_string(value)));
    }
    if let Some(value) = &unit.device_class {
        out.push_str(&format!(
            "device_class = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = &unit.selected_lowering_target {
        out.push_str(&format!(
            "selected_lowering_target = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
}

fn append_inline_artifact_fields(out: &mut String, unit: &BuildManifestDomainBuildUnit) {
    if let Some(value) = &unit.artifact_stub_inline {
        out.push_str(&format!(
            "artifact_stub_inline = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = &unit.artifact_payload_path {
        out.push_str(&format!(
            "artifact_payload_path = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = &unit.artifact_bridge_stub_path {
        out.push_str(&format!(
            "artifact_bridge_stub_path = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = &unit.artifact_ir_sidecar_path {
        out.push_str(&format!(
            "artifact_ir_sidecar_path = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = &unit.artifact_bridge_stub_inline {
        out.push_str(&format!(
            "artifact_bridge_stub_inline = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = &unit.artifact_payload_blob_path {
        out.push_str(&format!(
            "artifact_payload_blob_path = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = unit.artifact_payload_blob_bytes {
        out.push_str(&format!("artifact_payload_blob_bytes = {}\n", value));
    }
    if let Some(value) = &unit.artifact_payload_format {
        out.push_str(&format!(
            "artifact_payload_format = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
    if let Some(value) = &unit.artifact_payload_blob_inline {
        out.push_str(&format!(
            "artifact_payload_blob_inline = \"{}\"\n",
            escape_toml_string(value)
        ));
    }
}
