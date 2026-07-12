use std::path::Path;

use crate::{
    parse_build_manifest_from_source,
    toml::{
        escape_toml_string, parse_optional_toml_string_array, parse_required_toml_string,
        render_string_array,
    },
    ArtifactError,
};

use super::{NuisCompiledArtifact, NuisLifecycleContract};

pub(crate) fn render_compiled_artifact_metadata(artifact: &NuisCompiledArtifact) -> String {
    format!(
        "artifact_schema = \"{}\"\npackaging_mode = \"{}\"\ncpu_target_abi = \"{}\"\ncpu_target_machine_arch = \"{}\"\ncpu_target_machine_os = \"{}\"\ncpu_target_object_format = \"{}\"\ncpu_target_calling_abi = \"{}\"\nbinary_name = \"{}\"\n",
        escape_toml_string(&artifact.schema),
        escape_toml_string(&artifact.packaging_mode),
        escape_toml_string(&artifact.cpu_target_abi),
        escape_toml_string(&artifact.cpu_target_machine_arch),
        escape_toml_string(&artifact.cpu_target_machine_os),
        escape_toml_string(&artifact.cpu_target_object_format),
        escape_toml_string(&artifact.cpu_target_calling_abi),
        escape_toml_string(&artifact.binary_name),
    )
}

pub(crate) fn render_lifecycle_contract(lifecycle: &NuisLifecycleContract) -> String {
    format!(
        "lifecycle_schema = \"{}\"\nlifecycle_bootstrap_entry = \"{}\"\nlifecycle_tick_policy = \"{}\"\nlifecycle_shutdown_policy = \"{}\"\nlifecycle_yalivia_rpc = \"{}\"\nlifecycle_hook_surface = {}\nlifecycle_export_surface = {}\nlifecycle_runtime_capability_flags = {}\n",
        escape_toml_string(&lifecycle.schema),
        escape_toml_string(&lifecycle.bootstrap_entry),
        escape_toml_string(&lifecycle.tick_policy),
        escape_toml_string(&lifecycle.shutdown_policy),
        escape_toml_string(&lifecycle.yalivia_rpc),
        render_string_array(&lifecycle.hook_surface),
        render_string_array(&lifecycle.export_surface),
        render_string_array(&lifecycle.runtime_capability_flags),
    )
}

pub(crate) fn render_lowering_index(
    artifact: &NuisCompiledArtifact,
) -> Result<String, ArtifactError> {
    let manifest = parse_build_manifest_from_source(
        &artifact.build_manifest_source,
        Path::new("<compiled-artifact-build-manifest>"),
    )?;
    let mut out = String::new();
    out.push_str("schema = \"nuis-lowering-index-v1\"\n");
    out.push_str(&format!(
        "packaging_mode = \"{}\"\n",
        escape_toml_string(&artifact.packaging_mode)
    ));
    out.push_str(&format!(
        "domain_unit_count = {}\n",
        manifest.domain_build_units.len()
    ));
    for unit in &manifest.domain_build_units {
        out.push_str("\n[[lowering_unit]]\n");
        out.push_str(&format!(
            "package_id = \"{}\"\n",
            escape_toml_string(&unit.package_id)
        ));
        out.push_str(&format!(
            "domain_family = \"{}\"\n",
            escape_toml_string(&unit.domain_family)
        ));
        if let Some(value) = &unit.backend_family {
            out.push_str(&format!(
                "backend_family = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &unit.target_device {
            out.push_str(&format!(
                "target_device = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &unit.ir_format {
            out.push_str(&format!("ir_format = \"{}\"\n", escape_toml_string(value)));
        }
        if let Some(value) = &unit.dispatch_abi {
            out.push_str(&format!(
                "dispatch_abi = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = unit.backend_priority {
            out.push_str(&format!("backend_priority = {}\n", value));
        }
        if let Some(value) = &unit.verification {
            out.push_str(&format!(
                "verification = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &unit.selected_lowering_target {
            out.push_str(&format!(
                "selected_lowering_target = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        if let Some(value) = &unit.artifact_ir_sidecar_path {
            out.push_str(&format!(
                "artifact_ir_sidecar_path = \"{}\"\n",
                escape_toml_string(value)
            ));
        }
        out.push_str(&format!(
            "contract_family = \"{}\"\n",
            escape_toml_string(&unit.contract_family)
        ));
        out.push_str(&format!(
            "packaging_role = \"{}\"\n",
            escape_toml_string(&unit.packaging_role)
        ));
    }
    Ok(out)
}

pub(crate) fn parse_lifecycle_contract(
    source: &str,
) -> Result<NuisLifecycleContract, ArtifactError> {
    let path = Path::new("<compiled-artifact-lifecycle>");
    Ok(NuisLifecycleContract {
        schema: parse_required_toml_string(source, "lifecycle_schema", path)?,
        bootstrap_entry: parse_required_toml_string(source, "lifecycle_bootstrap_entry", path)?,
        tick_policy: parse_required_toml_string(source, "lifecycle_tick_policy", path)?,
        shutdown_policy: parse_required_toml_string(source, "lifecycle_shutdown_policy", path)?,
        yalivia_rpc: parse_required_toml_string(source, "lifecycle_yalivia_rpc", path)?,
        hook_surface: parse_optional_toml_string_array(source, "lifecycle_hook_surface")
            .unwrap_or_default(),
        export_surface: parse_optional_toml_string_array(source, "lifecycle_export_surface")
            .unwrap_or_default(),
        runtime_capability_flags: parse_optional_toml_string_array(
            source,
            "lifecycle_runtime_capability_flags",
        )
        .unwrap_or_default(),
    })
}
