use std::path::PathBuf;

use crate::aot;
use crate::artifact_report::{verify_artifact_json, verify_build_manifest_json};
use crate::command_helpers::{load_nuis_compiled_artifact, success_logs_enabled};
use crate::domain_build_report::{
    domain_build_contract_drift_checks, domain_build_unit_effective_contract_summary,
    domain_build_unit_verification_verdict, evaluate_domain_build_contract_drift,
};
use crate::execution_inspect::verdict_status;
use crate::project_metadata_report::resolve_build_manifest_path;

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
        println!("build manifest verified: {}", manifest.display());
        println!("  schema: {}", report.schema);
        println!("  input: {}", report.input);
        println!("  output_dir: {}", report.output_dir);
        println!("  packaging_mode: {}", report.packaging_mode);
        println!("  envelope_path: {}", report.envelope_path);
        println!("  envelope_schema: {}", report.envelope_schema);
        println!(
            "  envelope_package_count: {}",
            report.envelope_package_count
        );
        println!("  artifact_path: {}", report.artifact_path);
        println!("  artifact_schema: {}", report.artifact_schema);
        println!("  artifact_binary_name: {}", report.artifact_binary_name);
        println!("  artifact_binary_bytes: {}", report.artifact_binary_bytes);
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
            "  execution_contracts_checked: {}",
            report.execution_contracts_checked
        );
        println!(
            "  domain_build_unit_count: {}",
            report.domain_build_unit_count
        );
        println!(
            "  heterogeneous_domain_count: {}",
            report.heterogeneous_domain_count
        );
        println!(
            "  domain_payload_blobs_checked: {}",
            report.domain_payload_blobs_checked
        );
        println!(
            "  domain_payload_blob_sections_checked: {}",
            report.domain_payload_blob_sections_checked
        );
        println!(
            "  domain_payload_contract_sections_checked: {}",
            report.domain_payload_contract_sections_checked
        );
        println!(
            "  domain_payload_lowering_plans_checked: {}",
            report.domain_payload_lowering_plans_checked
        );
        println!(
            "  domain_payload_backend_stubs_checked: {}",
            report.domain_payload_backend_stubs_checked
        );
        println!(
            "  domain_payload_bridge_plans_checked: {}",
            report.domain_payload_bridge_plans_checked
        );
        println!(
            "  domain_bridge_stubs_checked: {}",
            report.domain_bridge_stubs_checked
        );
        let drift_checks = domain_build_contract_drift_checks(&report.domain_build_units);
        let drift_mismatch_count = drift_checks
            .iter()
            .filter(|check| !check.consistent)
            .count();
        println!(
            "  domain_build_contract_drift_checked: {}",
            drift_checks.len()
        );
        println!(
            "  domain_build_contract_drift_mismatches: {}",
            drift_mismatch_count
        );
        println!(
            "  domain_build_contracts_consistent: {}",
            if drift_mismatch_count == 0 {
                "true"
            } else {
                "false"
            }
        );
        for unit in &report.domain_build_units {
            let verdict = domain_build_unit_verification_verdict(unit, &report);
            let build_contract = domain_build_unit_effective_contract_summary(unit);
            println!(
                "  domain_build_contract: {} [{}]",
                unit.package_id, unit.domain_family
            );
            if let Some(abi) = unit.abi.as_deref() {
                println!("    abi: {}", abi);
            }
            if let Some(target) = unit.selected_lowering_target.as_deref() {
                println!("    selected_lowering_target: {}", target);
            }
            println!(
                "    lowering: lane_policy={}, bridge_surface={}, emission_kind={}",
                build_contract.lowering.lane_policy,
                build_contract.lowering.bridge_surface,
                build_contract.lowering.emission_kind
            );
            println!(
                "    backend: stub_kind={}, bridge_entry={}, submission_mode={}, wake_policy={}, scheduler_binding={}",
                build_contract.backend.stub_kind,
                build_contract.backend.bridge_entry,
                build_contract.backend.submission_mode,
                build_contract.backend.wake_policy,
                build_contract.backend.scheduler_binding
            );
            println!(
                "    bridge: bridge_surface={}, bridge_entry={}, scheduler_binding={}, phase_bind={}, phase_submit={}, phase_wait={}, phase_finalize={}, bridge_kind={}",
                build_contract.bridge.bridge_surface,
                build_contract.bridge.bridge_entry,
                build_contract.bridge.scheduler_binding,
                build_contract.bridge.phase_bind,
                build_contract.bridge.phase_submit,
                build_contract.bridge.phase_wait,
                build_contract.bridge.phase_finalize,
                build_contract.bridge.bridge_kind
            );
            println!(
                "    host_bridge: host_ffi_surface={}, handle_family={}, phase_order={}, phase_bind_wake={}, phase_submit_wake={}, phase_wait_wake={}, phase_finalize_wake={}, bridge_plan_begin={}, bridge_plan_end={}",
                build_contract.host_bridge.host_ffi_surface,
                build_contract.host_bridge.handle_family,
                build_contract.host_bridge.phase_order.join(", "),
                build_contract.host_bridge.phase_bind_wake,
                build_contract.host_bridge.phase_submit_wake,
                build_contract.host_bridge.phase_wait_wake,
                build_contract.host_bridge.phase_finalize_wake,
                build_contract.host_bridge.bridge_plan_begin,
                build_contract.host_bridge.bridge_plan_end
            );
            let drift = evaluate_domain_build_contract_drift(unit);
            println!(
                "    registry_alignment: {}",
                if drift.consistent { "ok" } else { "drift" }
            );
            println!(
                "    verification_verdict: kind={} payload_blob={} lowering_plan={} backend_stub={} bridge_plan={} bridge_stub={} bridge_registry={} host_bridge_plan={} registry_alignment={} consistent={}",
                verdict.kind,
                verdict_status(verdict.payload_blob_ok, verdict.kind == "hetero"),
                verdict_status(verdict.lowering_plan_ok, verdict.kind == "hetero"),
                verdict_status(verdict.backend_stub_ok, verdict.kind == "hetero"),
                verdict_status(verdict.bridge_plan_ok, verdict.kind == "hetero"),
                verdict_status(verdict.bridge_stub_ok, verdict.kind == "hetero"),
                verdict_status(verdict.bridge_registry_ok, verdict.kind == "hetero"),
                verdict_status(verdict.host_bridge_plan_ok, verdict.kind == "hetero"),
                if verdict.registry_alignment_ok { "ok" } else { "drift" },
                if verdict.consistent { "true" } else { "false" }
            );
            if !verdict.failure_reasons.is_empty() {
                println!(
                    "      failure_reasons: {}",
                    verdict.failure_reasons.join(", ")
                );
            }
            for issue in drift.issues {
                println!("      issue: {}", issue);
            }
        }
        if let Some(path) = &report.bridge_registry_path {
            println!("  bridge_registry_path: {}", path);
        }
        println!("  bridge_registry_units: {}", report.bridge_registry_units);
        println!(
            "  bridge_registry_checked: {}",
            report.bridge_registry_checked
        );
        println!(
            "  bridge_registry_entries_checked: {}",
            report.bridge_registry_entries_checked
        );
        if let Some(path) = &report.host_bridge_plan_index_path {
            println!("  host_bridge_plan_index_path: {}", path);
        }
        println!(
            "  host_bridge_plan_units: {}",
            report.host_bridge_plan_units
        );
        println!(
            "  host_bridge_plan_checked: {}",
            report.host_bridge_plan_checked
        );
        println!(
            "  host_bridge_plan_entries_checked: {}",
            report.host_bridge_plan_entries_checked
        );
        if let Some(path) = &report.lowering_plan_index_path {
            println!("  lowering_plan_index_path: {}", path);
        }
        println!("  lowering_plan_units: {}", report.lowering_plan_units);
        println!(
            "  lowering_plan_index_checked: {}",
            report.lowering_plan_index_checked
        );
        println!(
            "  lowering_plan_entries_checked: {}",
            report.lowering_plan_entries_checked
        );
        if let Some(path) = &report.doc_index_path {
            println!("  doc_index_path: {}", path);
        }
        println!(
            "  doc_index_module_count: {}",
            report.doc_index_module_count
        );
        println!(
            "  doc_index_documented_item_count: {}",
            report.doc_index_documented_item_count
        );
        println!("  doc_index_checked: {}", report.doc_index_checked);
        if let Some(path) = &report.project_docs_index {
            println!("  project_docs_index: {}", path);
        }
        println!(
            "  project_docs_module_count: {}",
            report.project_docs_module_count
        );
        println!(
            "  project_docs_documented_module_count: {}",
            report.project_docs_documented_module_count
        );
        println!(
            "  project_docs_documented_item_count: {}",
            report.project_docs_documented_item_count
        );
        if let Some(path) = &report.project_imports_index {
            println!("  project_imports_index: {}", path);
        }
        println!(
            "  project_imports_library_count: {}",
            report.project_imports_library_count
        );
        println!(
            "  project_imports_visible_library_count: {}",
            report.project_imports_visible_library_count
        );
        println!(
            "  project_imports_visible_module_count: {}",
            report.project_imports_visible_module_count
        );
        println!(
            "  project_imports_documented_visible_module_count: {}",
            report.project_imports_documented_visible_module_count
        );
        println!(
            "  project_imports_documented_visible_item_count: {}",
            report.project_imports_documented_visible_item_count
        );
        if let Some(path) = &report.project_galaxy_index {
            println!("  project_galaxy_index: {}", path);
        }
        println!("  project_galaxy_count: {}", report.project_galaxy_count);
        println!(
            "  project_documented_galaxy_count: {}",
            report.project_documented_galaxy_count
        );
        println!(
            "  project_documented_galaxy_library_module_count: {}",
            report.project_documented_galaxy_library_module_count
        );
        println!(
            "  project_documented_galaxy_item_count: {}",
            report.project_documented_galaxy_item_count
        );
        for unit in &report.domain_build_units {
            let payload_blob_bytes = unit
                .artifact_payload_blob_bytes
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<none>".to_owned());
            println!(
                "  domain_build_unit: {} package={} abi={} lowering={} backend={} role={} stub={} payload={} bridge_stub={} payload_blob={} payload_blob_bytes={} payload_format={}",
                unit.domain_family,
                unit.package_id,
                unit.abi.as_deref().unwrap_or("<none>"),
                unit.selected_lowering_target.as_deref().unwrap_or("<none>"),
                unit.backend_family.as_deref().unwrap_or("<none>"),
                unit.packaging_role,
                unit.artifact_stub_path.as_deref().unwrap_or("<none>"),
                unit.artifact_payload_path.as_deref().unwrap_or("<none>"),
                unit.artifact_bridge_stub_path.as_deref().unwrap_or("<none>"),
                unit.artifact_payload_blob_path.as_deref().unwrap_or("<none>"),
                payload_blob_bytes,
                unit.artifact_payload_format.as_deref().unwrap_or("<none>")
            );
        }
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
        println!("  cpu_target_clang: {}", report.cpu_target_clang);
        println!(
            "  cpu_target_cross: {}",
            if report.cpu_target_cross {
                "true"
            } else {
                "false"
            }
        );
        if let Some(status) = report.compile_cache_status {
            println!("  compile_cache_status: {}", status);
        }
        if let Some(key) = report.compile_cache_key {
            println!("  compile_cache_key: {}", key);
        }
        if let Some(root) = report.compile_cache_root {
            println!("  compile_cache_root: {}", root);
        }
        if let Some(plan_index) = report.project_plan_index {
            println!("  project_plan_index: {}", plan_index);
        }
        if let Some(packet_index) = report.project_packet_index {
            println!("  project_packet_index: {}", packet_index);
        }
        println!("  artifacts_checked: {}", report.artifacts_checked);
        println!(
            "  project_metadata_checked: {}",
            report.project_metadata_checked
        );
    }

    Ok(())
}
