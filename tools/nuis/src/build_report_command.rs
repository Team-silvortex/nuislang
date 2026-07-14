use crate::{
    artifact_doctor::{collect_artifact_output_diagnostics, probe_artifact_doctor},
    build_report_nsld_status::print_nsld_artifact_chain_status,
    build_report_render::{render_build_report_json, runtime_execution_summary},
    runtime_host_yir,
};
use std::path::PathBuf;

pub(crate) fn handle_build_report(input: PathBuf, json: bool) -> Result<(), String> {
    if json {
        println!("{}", render_build_report_json(&input));
        return Ok(());
    }
    let doctor = probe_artifact_doctor(&input);
    let diagnostics = collect_artifact_output_diagnostics(&input, &doctor);
    let manifest_verify = doctor
        .manifest_path
        .as_ref()
        .filter(|_| doctor.manifest_verified)
        .and_then(|path| nuisc::aot::verify_build_manifest(path).ok());
    let artifact_verify = doctor
        .artifact_path
        .as_ref()
        .filter(|_| doctor.artifact_verified)
        .and_then(|path| nuisc::aot::verify_nuis_compiled_artifact(path).ok());
    println!("build report: {}", doctor.input.display());
    println!("  source_kind: {}", doctor.source_kind);
    println!("  ready_to_run: {}", doctor.ready_to_run);
    println!(
        "  artifact_diagnostic_code: {}",
        diagnostics.artifact_diagnostic_code
    );
    println!("  self_check_ready: {}", diagnostics.self_check.ready);
    println!("  self_check_code: {}", diagnostics.self_check.code);
    println!("  recommended_next_step: {}", doctor.recommended_next_step);
    println!("  recommended_command: {}", doctor.recommended_command);
    if let Some(error) = diagnostics.self_check.error.as_deref() {
        println!("  self_check_error: {}", error);
    }
    println!(
        "  project_checks_available: {}",
        diagnostics.project_checks.available()
    );
    println!("  project_checks_code: {}", diagnostics.project_checks.code);
    if let Some(snapshot) = diagnostics.project_checks.snapshot.as_ref() {
        println!("  project_checks_root: {}", snapshot.project_root.display());
        println!(
            "  abi_checks_ok: {} ({})",
            snapshot.abi_checks.iter().all(|check| check.ok),
            snapshot.abi_checks.len()
        );
        println!(
            "  registry_checks_ok: {} ({})",
            snapshot.registry_checks.iter().all(|check| check.ok),
            snapshot.registry_checks.len()
        );
        println!(
            "  lowering_checks_ok: {} ({})",
            snapshot.lowering_checks.iter().all(|check| check.ok),
            snapshot.lowering_checks.len()
        );
    }
    if let Some(report) = manifest_verify.as_ref() {
        println!(
            "  text_handle_rewrite_helper_hits: {}",
            report.project_text_handle_rewrite_helper_hits
        );
        println!(
            "  text_handle_rewrite_local_hits: {}",
            report.project_text_handle_rewrite_local_hits
        );
        println!(
            "  text_handle_rewrite_total_hits: {}",
            report.project_text_handle_rewrite_helper_hits
                + report.project_text_handle_rewrite_local_hits
        );
        println!("  packaging_mode: {}", report.packaging_mode);
        println!("  binary_name: {}", report.artifact_binary_name);
        println!("  binary_bytes: {}", report.artifact_binary_bytes);
        println!("  cpu_target_abi: {}", report.cpu_target_abi);
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
        println!(
            "  lifecycle_runtime_capability_flags: {}",
            if report.lifecycle_runtime_capability_flags.is_empty() {
                "<none>".to_owned()
            } else {
                report.lifecycle_runtime_capability_flags.join(", ")
            }
        );
        println!(
            "  heterogeneous_domain_count: {}",
            report.heterogeneous_domain_count
        );
        println!(
            "  bridge_registry_path: {}",
            report.bridge_registry_path.as_deref().unwrap_or("<none>")
        );
        println!("  bridge_registry_units: {}", report.bridge_registry_units);
        println!(
            "  bridge_registry_checked: {}",
            report.bridge_registry_checked
        );
        println!(
            "  bridge_registry_entries_checked: {}",
            report.bridge_registry_entries_checked
        );
        println!(
            "  host_bridge_plan_index_path: {}",
            report
                .host_bridge_plan_index_path
                .as_deref()
                .unwrap_or("<none>")
        );
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
        println!(
            "  lowering_plan_index_path: {}",
            report
                .lowering_plan_index_path
                .as_deref()
                .unwrap_or("<none>")
        );
        println!("  lowering_plan_units: {}", report.lowering_plan_units);
        println!(
            "  lowering_plan_index_checked: {}",
            report.lowering_plan_index_checked
        );
        println!(
            "  lowering_plan_entries_checked: {}",
            report.lowering_plan_entries_checked
        );
        println!("  domain_units: {}", report.domain_build_units.len());
        for unit in &report.domain_build_units {
            let abi = unit.abi.as_deref().unwrap_or("<none>");
            let lowering = unit.selected_lowering_target.as_deref().unwrap_or("<none>");
            let backend = unit.backend_family.as_deref().unwrap_or("<none>");
            println!(
                "  domain_unit: {} package={} role={} abi={} lowering={} backend={}",
                unit.domain_family, unit.package_id, unit.packaging_role, abi, lowering, backend
            );
        }
    } else {
        println!("  packaging_mode: <unavailable>");
        println!("  domain_units: 0");
    }
    if let Some(report) = artifact_verify.as_ref() {
        println!(
            "  artifact_roundtrip_verified: {}",
            report.artifact_roundtrip_verified
        );
        println!(
            "  lifecycle_contract_consistent: {}",
            report.lifecycle_contract_consistent
        );
        println!(
            "  lifecycle_runtime_capability_flags_consistent: {}",
            report.lifecycle_runtime_capability_flags_consistent
        );
    }
    if doctor.artifact_verified {
        if let Some(path) = doctor.artifact_path.as_ref() {
            match nuis_runtime::RuntimeLoader.load_from_artifact_path(path) {
                Ok(loaded) => {
                    let host_consumable = loaded.host_consumable_summary();
                    println!("  runtime_load_attempted: true");
                    println!("  runtime_load_ok: true");
                    println!(
                        "  runtime_loaded_lifecycle_entry: {}",
                        loaded.artifact.lifecycle.bootstrap_entry
                    );
                    println!(
                        "  runtime_loaded_domain_units: {}",
                        loaded.domain_units.len()
                    );
                    println!(
                        "  runtime_loaded_heterogeneous_units: {}",
                        loaded.heterogeneous_units().count()
                    );
                    println!(
                        "  runtime_loaded_payload_blobs: {}",
                        loaded.domain_payload_blobs.len()
                    );
                    println!(
                        "  runtime_payload_backed_heterogeneous_units: {}",
                        host_consumable.payload_backed_units
                    );
                    println!(
                        "  runtime_cpu_fallback_units: {}",
                        host_consumable.cpu_fallback_units
                    );
                    println!(
                        "  runtime_host_consumable_units: {}",
                        host_consumable.host_consumable_units
                    );
                    println!(
                        "  runtime_loaded_bridge_registry: {}",
                        loaded.bridge_registry.is_some()
                    );
                    println!(
                        "  runtime_loaded_host_bridge_plan_index: {}",
                        loaded.host_bridge_plan_index.is_some()
                    );
                }
                Err(error) => {
                    println!("  runtime_load_attempted: true");
                    println!("  runtime_load_ok: false");
                    println!("  runtime_load_error: {}", error);
                }
            }
            match runtime_execution_summary(path) {
                Ok((
                    domains,
                    plan_phases,
                    trace_events,
                    host_fallback_events,
                    kernel_host_reference_events,
                )) => {
                    println!("  runtime_execution_attempted: true");
                    println!("  runtime_execution_ok: true");
                    println!("  runtime_execution_domains: {}", domains);
                    println!("  runtime_execution_plan_phases: {}", plan_phases);
                    println!("  runtime_execution_trace_events: {}", trace_events);
                    println!(
                        "  runtime_execution_host_fallback_events: {}",
                        host_fallback_events
                    );
                    println!(
                        "  runtime_execution_kernel_host_reference_events: {}",
                        kernel_host_reference_events
                    );
                }
                Err(error) => {
                    println!("  runtime_execution_attempted: true");
                    println!("  runtime_execution_ok: false");
                    println!("  runtime_execution_error: {}", error);
                }
            }
            match runtime_host_yir::summary(path) {
                Ok(Some((yir_path, summary))) => {
                    println!("  runtime_host_yir_attempted: true");
                    println!("  runtime_host_yir_ok: true");
                    println!("  runtime_host_yir_path: {}", yir_path);
                    println!("  runtime_host_yir_nodes: {}", summary.nodes_executed);
                    println!(
                        "  runtime_host_yir_kernel_nodes: {}",
                        summary.kernel_nodes_executed
                    );
                    println!(
                        "  runtime_host_yir_tensor_values: {}",
                        summary.tensor_values
                    );
                    println!(
                        "  runtime_host_yir_scalar_values: {}",
                        summary.scalar_values
                    );
                    println!("  runtime_host_yir_frame_values: {}", summary.frame_values);
                    println!(
                        "  runtime_host_yir_integer_checksum: {}",
                        summary.integer_checksum
                    );
                    println!(
                        "  runtime_host_yir_kernel_integer_checksum: {}",
                        summary.kernel_integer_checksum
                    );
                }
                Ok(None) => {
                    println!("  runtime_host_yir_attempted: false");
                    println!("  runtime_host_yir_ok: false");
                    println!("  runtime_host_yir_skip_reason: host_ffi_externs_present_or_no_yir");
                }
                Err(error) => {
                    println!("  runtime_host_yir_attempted: true");
                    println!("  runtime_host_yir_ok: false");
                    println!("  runtime_host_yir_error: {}", error);
                }
            }
        }
    } else {
        println!("  runtime_load_attempted: false");
        println!("  runtime_load_ok: false");
        println!("  runtime_execution_attempted: false");
        println!("  runtime_execution_ok: false");
        println!("  runtime_execution_host_fallback_events: 0");
        println!("  runtime_execution_kernel_host_reference_events: 0");
        println!("  runtime_host_yir_attempted: false");
        println!("  runtime_host_yir_ok: false");
    }
    println!(
        "  link_plan_available: {}",
        diagnostics.link_plan.plan.is_some()
    );
    if let Some(plan) = diagnostics.link_plan.plan.as_ref() {
        println!("  link_plan_final_stage: {}", plan.final_stage.kind);
        println!("  link_plan_final_driver: {}", plan.final_stage.driver);
        println!(
            "  link_plan_final_link_mode: {}",
            plan.final_stage.link_mode
        );
        println!("  link_plan_final_output: {}", plan.final_stage.output_path);
        println!(
            "  link_plan_lowering_plan_index_path: {}",
            plan.lowering_plan_index_path.as_deref().unwrap_or("<none>")
        );
        println!(
            "  link_plan_lowering_plan_index_source: {}",
            plan.lowering_plan_index_source
        );
        print_nsld_artifact_chain_status(plan);
    }
    Ok(())
}
