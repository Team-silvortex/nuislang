use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::aot::{
    render_relocated_unpacked_build_manifest, verify_build_manifest, write_nuis_compiled_artifact,
    write_nuis_executable_envelope,
};
use crate::aot_artifact::{
    inspect_nuis_compiled_artifact_container, parse_nuis_compiled_artifact,
    validate_nuis_compiled_artifact_layout,
};
use crate::aot_lifecycle::build_nuis_lifecycle_contract;
use crate::aot_verify_report::NuisCompiledArtifactVerifyReport;

pub(crate) fn verify_nuis_compiled_artifact_impl(
    path: &Path,
) -> Result<NuisCompiledArtifactVerifyReport, String> {
    let container = inspect_nuis_compiled_artifact_container(path)?;
    let artifact = parse_nuis_compiled_artifact(path)?;
    validate_nuis_compiled_artifact_layout(path, &artifact)?;
    if artifact.schema != "nuis-compiled-artifact-v1" {
        return Err(format!(
            "`{}` has unsupported nuis artifact schema `{}`; expected `nuis-compiled-artifact-v1`",
            path.display(),
            artifact.schema
        ));
    }
    if artifact.binary_blob.len() != artifact.binary_bytes {
        return Err(format!(
            "`{}` binary byte length mismatch: declared={}, actual={}",
            path.display(),
            artifact.binary_bytes,
            artifact.binary_blob.len()
        ));
    }
    if artifact.build_manifest_source.len() != artifact.build_manifest_bytes {
        return Err(format!(
            "`{}` build manifest byte length mismatch: declared={}, actual={}",
            path.display(),
            artifact.build_manifest_bytes,
            artifact.build_manifest_source.len()
        ));
    }
    let expected_lifecycle =
        build_nuis_lifecycle_contract(&artifact.envelope, &artifact.packaging_mode);
    if artifact.lifecycle != expected_lifecycle {
        return Err(format!(
            "`{}` lifecycle contract mismatch: artifact lifecycle does not match the expected contract derived from envelope/package mode",
            path.display()
        ));
    }

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("failed to read current time: {error}"))?
        .as_nanos();
    let temp_root = std::env::temp_dir().join(format!("nuis_artifact_verify_{nonce}"));
    fs::create_dir_all(&temp_root)
        .map_err(|error| format!("failed to create `{}`: {error}", temp_root.display()))?;

    let manifest_path = temp_root.join("nuis.build.manifest.toml");
    let envelope_path = temp_root.join("nuis.executable.envelope.toml");
    let artifact_path = temp_root.join("nuis.compiled.artifact");
    let binary_path = temp_root.join(&artifact.binary_name);

    fs::write(&binary_path, &artifact.binary_blob)
        .map_err(|error| format!("failed to write `{}`: {error}", binary_path.display()))?;
    write_nuis_executable_envelope(&envelope_path, &artifact.envelope)?;

    let relocated_manifest = render_relocated_unpacked_build_manifest(
        &artifact,
        &temp_root,
        &envelope_path,
        &artifact_path,
        &binary_path,
    )?;
    let mut relocated_artifact = artifact.clone();
    relocated_artifact.build_manifest_source = relocated_manifest.clone();
    relocated_artifact.build_manifest_bytes = relocated_manifest.len();
    write_nuis_compiled_artifact(&artifact_path, &relocated_artifact)?;
    fs::write(&manifest_path, &relocated_manifest)
        .map_err(|error| format!("failed to write `{}`: {error}", manifest_path.display()))?;

    let manifest_report = verify_build_manifest(&manifest_path)?;
    let _ = fs::remove_dir_all(&temp_root);

    Ok(NuisCompiledArtifactVerifyReport {
        schema: artifact.schema,
        artifact_container_kind: container.container_kind,
        artifact_container_version: container.binary_version,
        artifact_section_count: container.section_count,
        artifact_section_names: container.section_names,
        artifact_section_table_valid: container.section_table_valid,
        lowering_unit_count: container.lowering_unit_count,
        lowering_domain_families: container.lowering_domain_families,
        lowering_targets: container.lowering_targets,
        lowering_units: container.lowering_units,
        packaging_mode: artifact.packaging_mode,
        binary_name: artifact.binary_name,
        binary_bytes: artifact.binary_bytes,
        build_manifest_bytes: artifact.build_manifest_bytes,
        envelope_schema: artifact.envelope.schema,
        envelope_package_count: artifact.envelope.package_count,
        lifecycle_schema: artifact.lifecycle.schema,
        lifecycle_bootstrap_entry: artifact.lifecycle.bootstrap_entry,
        lifecycle_tick_policy: artifact.lifecycle.tick_policy,
        lifecycle_shutdown_policy: artifact.lifecycle.shutdown_policy,
        lifecycle_yalivia_rpc: artifact.lifecycle.yalivia_rpc,
        lifecycle_hook_count: artifact.lifecycle.hook_surface.len(),
        lifecycle_hook_surface: artifact.lifecycle.hook_surface.clone(),
        lifecycle_export_count: artifact.lifecycle.export_surface.len(),
        lifecycle_export_surface: artifact.lifecycle.export_surface.clone(),
        lifecycle_runtime_capability_flags: artifact.lifecycle.runtime_capability_flags.clone(),
        lifecycle_contract_consistent: true,
        lifecycle_runtime_capability_flags_consistent: true,
        execution_contracts_checked: manifest_report.execution_contracts_checked,
        cpu_target_abi: artifact.cpu_target_abi,
        cpu_target_machine_arch: artifact.cpu_target_machine_arch,
        cpu_target_machine_os: artifact.cpu_target_machine_os,
        cpu_target_object_format: artifact.cpu_target_object_format,
        cpu_target_calling_abi: artifact.cpu_target_calling_abi,
        artifact_roundtrip_verified: true,
    })
}
