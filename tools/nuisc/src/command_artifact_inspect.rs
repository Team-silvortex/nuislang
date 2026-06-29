use std::path::PathBuf;

use crate::artifact_report::{inspect_artifact_json, reconstruct_manifest_report_from_artifact};
use crate::command_helpers::{inspect_artifact_container_for_input, load_nuis_compiled_artifact};
use crate::domain_build_report::{
    domain_build_contract_drift_checks, domain_build_unit_effective_contract_summary,
    domain_build_unit_verification_verdict, evaluate_domain_build_contract_drift,
};
use crate::execution_inspect::verdict_status;
use crate::{aot, linker};

pub(crate) fn run_inspect_artifact(input: PathBuf, json: bool) -> Result<(), String> {
    let artifact = load_nuis_compiled_artifact(&input)?;
    let is_manifest_input = input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.build.manifest.toml")
        .unwrap_or(false);
    let manifest_verify = if is_manifest_input {
        Some(aot::verify_build_manifest(&input)?)
    } else {
        Some(reconstruct_manifest_report_from_artifact(&input, &artifact)?.1)
    };
    let container = inspect_artifact_container_for_input(&input, manifest_verify.as_ref())?;
    if json {
        println!(
            "{}",
            inspect_artifact_json(
                &input,
                &artifact,
                container.as_ref(),
                manifest_verify.as_ref(),
            )
        );
        return Ok(());
    }
    println!("nuis artifact: {}", input.display());
    if let Some(container) = &container {
        println!(
            "  artifact_container: {} version {}",
            container.container_kind, container.binary_version
        );
        println!("  artifact_section_count: {}", container.section_count);
        if !container.section_names.is_empty() {
            println!(
                "  artifact_section_names: {}",
                container.section_names.join(", ")
            );
        }
        println!(
            "  artifact_section_table_valid: {}",
            container.section_table_valid
        );
        println!("  lowering_unit_count: {}", container.lowering_unit_count);
        if !container.lowering_domain_families.is_empty() {
            println!(
                "  lowering_domain_families: {}",
                container.lowering_domain_families.join(", ")
            );
        }
        if !container.lowering_targets.is_empty() {
            println!(
                "  lowering_targets: {}",
                container.lowering_targets.join(", ")
            );
        }
    }
    println!("  schema: {}", artifact.schema);
    println!("  packaging_mode: {}", artifact.packaging_mode);
    println!("  cpu_target_abi: {}", artifact.cpu_target_abi);
    println!(
        "  cpu_target_machine: {}-{}",
        artifact.cpu_target_machine_arch, artifact.cpu_target_machine_os
    );
    println!(
        "  cpu_target_object_format: {}",
        artifact.cpu_target_object_format
    );
    println!(
        "  cpu_target_calling_abi: {}",
        artifact.cpu_target_calling_abi
    );
    println!("  binary_name: {}", artifact.binary_name);
    println!("  binary_bytes: {}", artifact.binary_bytes);
    println!("  build_manifest_bytes: {}", artifact.build_manifest_bytes);
    println!("  envelope_schema: {}", artifact.envelope.schema);
    println!(
        "  envelope_contract_families: {}",
        artifact.envelope.contract_families.join(", ")
    );
    println!("  lifecycle_schema: {}", artifact.lifecycle.schema);
    println!(
        "  lifecycle_bootstrap_entry: {}",
        artifact.lifecycle.bootstrap_entry
    );
    println!(
        "  lifecycle_tick_policy: {}",
        artifact.lifecycle.tick_policy
    );
    println!(
        "  lifecycle_shutdown_policy: {}",
        artifact.lifecycle.shutdown_policy
    );
    println!(
        "  lifecycle_yalivia_rpc: {}",
        artifact.lifecycle.yalivia_rpc
    );
    println!(
        "  lifecycle_hook_count: {}",
        artifact.lifecycle.hook_surface.len()
    );
    println!(
        "  lifecycle_hook_surface: {}",
        artifact.lifecycle.hook_surface.join(", ")
    );
    println!(
        "  lifecycle_export_count: {}",
        artifact.lifecycle.export_surface.len()
    );
    println!(
        "  lifecycle_export_surface: {}",
        artifact.lifecycle.export_surface.join(", ")
    );
    println!(
        "  lifecycle_runtime_capability_flags: {}",
        artifact.lifecycle.runtime_capability_flags.join(", ")
    );
    if let Some(report) = &manifest_verify {
        let link_plan = linker::build_link_plan(report, &artifact);
        let drift_checks = domain_build_contract_drift_checks(&report.domain_build_units);
        let drift_mismatch_count = drift_checks
            .iter()
            .filter(|check| !check.consistent)
            .count();
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
        println!(
            "  bridge_registry_entries_checked: {}",
            report.bridge_registry_entries_checked
        );
        println!(
            "  host_bridge_plan_entries_checked: {}",
            report.host_bridge_plan_entries_checked
        );
        println!("  link_plan_final_stage: {}", link_plan.final_stage.kind);
        println!("  link_plan_final_driver: {}", link_plan.final_stage.driver);
        println!(
            "  link_plan_final_link_mode: {}",
            link_plan.final_stage.link_mode
        );
        println!(
            "  link_plan_final_output: {}",
            link_plan.final_stage.output_path
        );
        println!("  link_plan_domain_units: {}", link_plan.domain_units.len());
        for unit in &report.domain_build_units {
            let verdict = domain_build_unit_verification_verdict(unit, report);
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
    }

    Ok(())
}
