use std::path::PathBuf;

use crate::aot;
use crate::artifact_report::{verify_artifact_json, verify_build_manifest_json};
use crate::command_helpers::{load_nuis_compiled_artifact, success_logs_enabled};
use crate::project_metadata_report::resolve_build_manifest_path;

use super::command_artifact_verify_manifest::print_build_manifest_verification;

pub(crate) fn run_verify_artifact(input: PathBuf, json: bool) -> Result<(), String> {
    let artifact_input = if input.is_dir() {
        let artifact_path = input.join("nuis.compiled.artifact");
        if artifact_path.is_file() {
            artifact_path
        } else {
            let manifest_path = resolve_build_manifest_path(&input)?;
            let report = aot::verify_build_manifest(&manifest_path)?;
            PathBuf::from(report.artifact_path)
        }
    } else {
        input.clone()
    };
    let report = aot::verify_nuis_compiled_artifact(&artifact_input)?;
    if json {
        println!("{}", verify_artifact_json(&artifact_input, &report));
        return Ok(());
    }
    println!("nuis artifact verified: {}", artifact_input.display());
    println!("  schema: {}", report.schema);
    println!(
        "  artifact_container_kind: {}",
        report.artifact_container_kind
    );
    println!(
        "  artifact_container_version: {}",
        report.artifact_container_version
    );
    println!(
        "  artifact_section_count: {}",
        report.artifact_section_count
    );
    if !report.artifact_section_names.is_empty() {
        println!(
            "  artifact_section_names: {}",
            report.artifact_section_names.join(", ")
        );
    }
    println!(
        "  artifact_section_table_valid: {}",
        report.artifact_section_table_valid
    );
    println!("  lowering_unit_count: {}", report.lowering_unit_count);
    if !report.lowering_domain_families.is_empty() {
        println!(
            "  lowering_domain_families: {}",
            report.lowering_domain_families.join(", ")
        );
    }
    if !report.lowering_targets.is_empty() {
        println!("  lowering_targets: {}", report.lowering_targets.join(", "));
    }
    println!("  packaging_mode: {}", report.packaging_mode);
    println!("  binary_name: {}", report.binary_name);
    println!("  binary_bytes: {}", report.binary_bytes);
    println!("  build_manifest_bytes: {}", report.build_manifest_bytes);
    println!("  envelope_schema: {}", report.envelope_schema);
    println!(
        "  envelope_package_count: {}",
        report.envelope_package_count
    );
    println!("  lifecycle_schema: {}", report.lifecycle_schema);
    println!(
        "  lifecycle_bootstrap_entry: {}",
        report.lifecycle_bootstrap_entry
    );
    println!("  lifecycle_tick_policy: {}", report.lifecycle_tick_policy);
    println!(
        "  lifecycle_shutdown_policy: {}",
        report.lifecycle_shutdown_policy
    );
    println!("  lifecycle_yalivia_rpc: {}", report.lifecycle_yalivia_rpc);
    println!("  lifecycle_hook_count: {}", report.lifecycle_hook_count);
    println!(
        "  lifecycle_hook_surface: {}",
        report.lifecycle_hook_surface.join(", ")
    );
    println!(
        "  lifecycle_export_count: {}",
        report.lifecycle_export_count
    );
    println!(
        "  lifecycle_export_surface: {}",
        report.lifecycle_export_surface.join(", ")
    );
    println!(
        "  lifecycle_runtime_capability_flags: {}",
        report.lifecycle_runtime_capability_flags.join(", ")
    );
    println!(
        "  lifecycle_contract_consistent: {}",
        if report.lifecycle_contract_consistent {
            "true"
        } else {
            "false"
        }
    );
    println!(
        "  lifecycle_runtime_capability_flags_consistent: {}",
        if report.lifecycle_runtime_capability_flags_consistent {
            "true"
        } else {
            "false"
        }
    );
    println!(
        "  execution_contracts_checked: {}",
        report.execution_contracts_checked
    );
    println!("  cpu_target_abi: {}", report.cpu_target_abi);
    println!(
        "  cpu_target_machine: {}-{}",
        report.cpu_target_machine_arch, report.cpu_target_machine_os
    );
    println!(
        "  cpu_target_object_format: {}",
        report.cpu_target_object_format
    );
    println!(
        "  cpu_target_calling_abi: {}",
        report.cpu_target_calling_abi
    );
    println!(
        "  artifact_roundtrip_verified: {}",
        if report.artifact_roundtrip_verified {
            "true"
        } else {
            "false"
        }
    );

    Ok(())
}
pub(crate) fn run_unpack_artifact(input: PathBuf, output_dir: PathBuf) -> Result<(), String> {
    let artifact = load_nuis_compiled_artifact(&input)?;
    std::fs::create_dir_all(&output_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", output_dir.display()))?;
    let envelope_path = output_dir.join("nuis.executable.envelope.toml");
    let manifest_path = output_dir.join("nuis.build.manifest.toml");
    let artifact_path = output_dir.join("nuis.compiled.artifact");
    let binary_path = output_dir.join(&artifact.binary_name);
    aot::write_nuis_executable_envelope(&envelope_path, &artifact.envelope)?;
    std::fs::write(&binary_path, &artifact.binary_blob)
        .map_err(|error| format!("failed to write `{}`: {error}", binary_path.display()))?;
    let relocated_manifest = aot::render_relocated_unpacked_build_manifest(
        &artifact,
        &output_dir,
        &envelope_path,
        &artifact_path,
        &binary_path,
    )?;
    let mut relocated_artifact = artifact.clone();
    relocated_artifact.build_manifest_source = relocated_manifest.clone();
    relocated_artifact.build_manifest_bytes = relocated_manifest.len();
    aot::write_nuis_compiled_artifact(&artifact_path, &relocated_artifact)?;
    std::fs::write(&manifest_path, relocated_manifest)
        .map_err(|error| format!("failed to write `{}`: {error}", manifest_path.display()))?;
    println!("unpacked nuis artifact: {}", output_dir.display());
    println!("  source: {}", input.display());
    println!("  manifest: {}", manifest_path.display());
    println!("  envelope: {}", envelope_path.display());
    println!("  artifact: {}", artifact_path.display());
    println!("  binary: {}", binary_path.display());
    println!("  packaging_mode: {}", artifact.packaging_mode);

    Ok(())
}
pub(crate) fn run_verify_build_manifest(manifest: PathBuf, json: bool) -> Result<(), String> {
    let manifest = resolve_build_manifest_path(&manifest)?;
    let report = aot::verify_build_manifest(&manifest)?;
    if json {
        println!("{}", verify_build_manifest_json(&manifest, &report));
        return Ok(());
    }
    if success_logs_enabled() {
        print_build_manifest_verification(&manifest, &report);
    }

    Ok(())
}
