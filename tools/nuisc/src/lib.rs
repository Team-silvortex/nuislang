pub mod aot;
mod aot_artifact;
mod aot_artifact_hash;
mod aot_c_shim_buffer_runtime;
mod aot_c_shim_env_io_runtime;
mod aot_c_shim_file_runtime;
mod aot_c_shim_fs_runtime;
mod aot_c_shim_header_runtime;
mod aot_c_shim_helpers;
mod aot_c_shim_http_runtime;
mod aot_c_shim_network_owned_runtime;
mod aot_c_shim_network_probe_runtime;
mod aot_c_shim_network_runtime;
mod aot_c_shim_path_runtime;
mod aot_c_shim_process_runtime;
mod aot_c_shim_runtime;
mod aot_c_shim_serialization_runtime;
mod aot_c_shim_source;
mod aot_c_shim_text_runtime;
mod aot_c_shim_time_debug_runtime;
mod aot_compile_driver;
mod aot_compiled_artifact_builder;
mod aot_compiled_artifact_verify;
mod aot_cpu_target;
mod aot_domain_artifact_writer;
mod aot_domain_contract;
mod aot_domain_index_render;
mod aot_domain_index_verify;
mod aot_domain_payload_blob;
mod aot_domain_payload_verify;
mod aot_domain_profile;
mod aot_domain_render;
mod aot_domain_unit_render;
mod aot_domain_unit_verify;
mod aot_encoding;
mod aot_ffi_bridge;
mod aot_kernel_sidecar;
mod aot_lifecycle;
mod aot_manifest_artifacts;
mod aot_manifest_core_verify;
mod aot_manifest_domain_model;
mod aot_manifest_execution_render;
mod aot_manifest_fields;
mod aot_manifest_header_render;
mod aot_manifest_path;
mod aot_manifest_project_render;
mod aot_manifest_relocate;
mod aot_manifest_render;
mod aot_manifest_report;
mod aot_manifest_types;
mod aot_manifest_verify;
mod aot_manifest_writer;
mod aot_native_runner;
mod aot_network_sidecar;
mod aot_output_layout;
mod aot_project_metadata_verify;
mod aot_shader_sidecar;
mod aot_symbol_anchor;
mod aot_toml;
mod aot_vcs_info;
mod aot_verify_report;
mod artifact_report;
pub mod cache;
pub mod cli;
pub mod codegen_wasm;
mod command_artifact;
mod command_helpers;
mod command_nustar;
mod command_registry;
pub mod data_markers;
mod domain_build_report;
pub mod engine;
pub mod errors;
mod execution_inspect;
mod execution_inspect_report;
pub mod fmt;
pub mod frontend;
mod inspect_report;
mod json_report;
#[cfg(test)]
mod lib_tests;
mod link_report;
pub mod linker;
pub mod lowering;
pub mod nir_verify;
mod nir_walk;
pub mod nustar_binary;
pub mod optimize;
pub mod pipeline;
pub mod project;
mod project_metadata_report;
pub mod registry;
mod registry_abi_helpers;
mod registry_abi_target;
mod registry_binding_plan;
mod registry_build_contract_preset;
mod registry_build_contract_summary;
mod registry_contract;
mod registry_domain_contract_validate;
mod registry_domain_json;
mod registry_host_ffi;
mod registry_json;
mod registry_load;
mod registry_manifest_parse;
mod registry_project_check_render;
mod registry_scheduler_summary;
mod registry_support_usage;
mod registry_types;
mod registry_validation;
pub mod render;
pub mod shader_source;
pub mod stdlib_registry;

use std::path::{Path, PathBuf};

pub use cli::CommandKind;

pub(crate) use crate::command_helpers::{
    compile_command_input, inspect_artifact_container_for_input, load_nuis_compiled_artifact,
    print_project_context, print_required_nustar_context, resolve_compile_input,
    success_logs_enabled, NUSTAR_REGISTRY_ROOT,
};

pub use crate::command_helpers::{
    nuisc_compile_pipeline_brief, project_compile_samples_brief, project_compile_workflow_brief,
    project_galaxy_workflow_brief, project_test_workflow_brief,
};

use crate::artifact_report::{
    artifact_report_json, artifact_report_summary_lines, domain_build_contract_summary_json,
    domain_registry_json, inspect_artifact_json, reconstruct_manifest_report_from_artifact,
    verify_artifact_json, verify_build_manifest_json,
};

use crate::execution_inspect::{
    execution_inspect_issues, verdict_status, ExecutionInspectOverview,
};
#[cfg(test)]
use crate::execution_inspect::{ExecutionInspectDomainOverview, ExecutionInspectIssue};
use crate::execution_inspect_report::{
    inspect_execution_json, inspect_execution_overview, render_execution_report,
};

use crate::inspect_report::{
    collect_benchmark_inventory, collect_doc_indexes, collect_doc_indexes_from_manifest_input,
    inspect_benchmarks_json, inspect_docs_json, inspect_galaxy_doc_summary,
    inspect_galaxy_docs_json, inspect_stdlib_doc_summary, inspect_stdlib_docs_json,
    summarize_doc_indexes, write_compile_doc_index, write_json_output,
};

pub(crate) use crate::json_report::{
    artifact_lowering_units_json, json_bool_field, json_escape, json_optional_i64_field,
    json_optional_string_field, json_string_array_field, json_string_field, json_usize_field,
};

use crate::link_report::link_plan_json;

use crate::project_metadata_report::{
    inspect_project_metadata, inspect_project_metadata_json,
    project_metadata_summary_from_manifest_report, render_project_metadata_compact_summary,
    render_project_metadata_paths, render_project_metadata_summary, repair_project_metadata_target,
    resolve_artifact_report_inputs, resolve_build_manifest_path, ProjectMetadataSummary,
};

#[cfg(test)]
use crate::domain_build_report::domain_build_unit_contract_json;
use crate::domain_build_report::{
    collect_domain_build_unit_verdicts, domain_build_contract_drift_checks,
    domain_build_unit_effective_contract_summary, domain_build_unit_verification_verdict,
    evaluate_domain_build_contract_drift, summarize_domain_build_verification,
    DomainBuildVerificationSummary,
};

pub fn run(command: CommandKind) -> Result<(), String> {
    match command {
        CommandKind::Status => command_registry::run_status()?,
        CommandKind::Registry { json } => command_registry::run_registry(json)?,
        CommandKind::Fmt { input } => {
            let report = fmt::format_input(&input)?;
            println!("formatted nuis input: {}", input.display());
            println!("  total_files: {}", report.total_files);
            println!("  changed_files: {}", report.changed_files.len());
            for file in report.changed_files {
                println!("  - {}", file);
            }
        }
        CommandKind::Bindings { input } => command_nustar::run_bindings(input)?,
        CommandKind::PackNustar { package_id, output } => {
            command_nustar::run_pack_nustar(package_id, output)?
        }
        CommandKind::InspectNustar { input } => command_nustar::run_inspect_nustar(input)?,
        CommandKind::LoaderContract { package_id } => {
            command_nustar::run_loader_contract(package_id)?
        }
        CommandKind::PackEnvelope { input, output } => {
            command_artifact::run_pack_envelope(input, output)?
        }
        CommandKind::UnpackEnvelope { input, output } => {
            command_artifact::run_unpack_envelope(input, output)?
        }
        CommandKind::InspectEnvelope { input } => command_artifact::run_inspect_envelope(input)?,
        CommandKind::InspectArtifact { input, json } => {
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
        }
        CommandKind::InspectExecution { input, json } => {
            if json {
                println!("{}", inspect_execution_json(&input)?);
            } else {
                println!("{}", render_execution_report(&input)?);
            }
        }
        CommandKind::ArtifactReport {
            input,
            json,
            summary,
        } => {
            let (
                manifest_input,
                artifact,
                artifact_verify_input,
                manifest_verify,
                manifest_verify_reconstructed,
            ) = resolve_artifact_report_inputs(&input)?;
            let artifact_verify = aot::verify_nuis_compiled_artifact(&artifact_verify_input)?;
            if json {
                println!(
                    "{}",
                    artifact_report_json(
                        &input,
                        &artifact,
                        &artifact_verify_input,
                        &artifact_verify,
                        &manifest_input,
                        &manifest_verify,
                        manifest_verify_reconstructed,
                    )
                );
                return Ok(());
            }
            let verdicts = collect_domain_build_unit_verdicts(&manifest_verify);
            let summary_view = summarize_domain_build_verification(&verdicts);
            let execution_overview = inspect_execution_overview(&manifest_input).ok();
            let doc_indexes = collect_doc_indexes_from_manifest_input(&manifest_verify).ok();
            let project_metadata = project_metadata_summary_from_manifest_report(
                "build-manifest",
                Some(&manifest_input),
                Some(&artifact_verify_input),
                &manifest_verify,
            );
            if summary {
                println!("nuis artifact report summary: {}", input.display());
                for line in artifact_report_summary_lines(
                    &artifact_verify,
                    &summary_view,
                    Some(&linker::build_link_plan(&manifest_verify, &artifact)),
                    manifest_verify_reconstructed,
                    execution_overview.as_ref(),
                    doc_indexes.as_deref(),
                    Some(&project_metadata),
                ) {
                    println!("  {}", line);
                }
                return Ok(());
            }
            println!("nuis artifact report: {}", input.display());
            println!("  artifact_schema: {}", artifact.schema);
            println!("  packaging_mode: {}", artifact.packaging_mode);
            println!("  binary_name: {}", artifact.binary_name);
            println!(
                "  artifact_roundtrip_verified: {}",
                if artifact_verify.artifact_roundtrip_verified {
                    "true"
                } else {
                    "false"
                }
            );
            println!(
                "  lifecycle_contract_consistent: {}",
                if artifact_verify.lifecycle_contract_consistent {
                    "true"
                } else {
                    "false"
                }
            );
            println!(
                "  lifecycle_runtime_capability_flags_consistent: {}",
                if artifact_verify.lifecycle_runtime_capability_flags_consistent {
                    "true"
                } else {
                    "false"
                }
            );
            println!("  manifest_schema: {}", manifest_verify.schema);
            println!("  manifest_input: {}", manifest_input.display());
            println!(
                "  manifest_verify_reconstructed: {}",
                if manifest_verify_reconstructed {
                    "true"
                } else {
                    "false"
                }
            );
            println!(
                "  manifest_artifact_path: {}",
                manifest_verify.artifact_path
            );
            if let Some(indexes) = &doc_indexes {
                println!(
                    "  documented_modules: {}",
                    indexes
                        .iter()
                        .filter(|index| !index.items.is_empty())
                        .count()
                );
                println!(
                    "  documented_items: {}",
                    indexes.iter().map(|index| index.items.len()).sum::<usize>()
                );
            }
            println!(
                "  execution_contracts_checked: {}",
                manifest_verify.execution_contracts_checked
            );
            let summary = summary_view;
            for line in artifact_report_summary_lines(
                &artifact_verify,
                &summary,
                Some(&linker::build_link_plan(&manifest_verify, &artifact)),
                manifest_verify_reconstructed,
                execution_overview.as_ref(),
                doc_indexes.as_deref(),
                Some(&project_metadata),
            ) {
                println!("  {}", line);
            }
            println!(
                "  all_units_consistent: {}",
                if summary.all_units_consistent {
                    "true"
                } else {
                    "false"
                }
            );
            println!("  total_units: {}", summary.total_units);
            println!("  host_units_checked: {}", summary.host_units_checked);
            println!("  hetero_units_checked: {}", summary.hetero_units_checked);
            println!("  registry_drift_units: {}", summary.registry_drift_units);
            println!(
                "  failing_units: {}",
                if summary.failing_units.is_empty() {
                    "<none>".to_owned()
                } else {
                    summary.failing_units.join(", ")
                }
            );
            println!(
                "  domain_payload_blobs_checked: {}",
                manifest_verify.domain_payload_blobs_checked
            );
            println!(
                "  domain_payload_blob_sections_checked: {}",
                manifest_verify.domain_payload_blob_sections_checked
            );
            println!(
                "  domain_payload_lowering_plans_checked: {}",
                manifest_verify.domain_payload_lowering_plans_checked
            );
            println!(
                "  domain_payload_backend_stubs_checked: {}",
                manifest_verify.domain_payload_backend_stubs_checked
            );
            println!(
                "  domain_payload_bridge_plans_checked: {}",
                manifest_verify.domain_payload_bridge_plans_checked
            );
            println!(
                "  domain_bridge_stubs_checked: {}",
                manifest_verify.domain_bridge_stubs_checked
            );
            println!(
                "  bridge_registry_entries_checked: {}",
                manifest_verify.bridge_registry_entries_checked
            );
            println!(
                "  host_bridge_plan_entries_checked: {}",
                manifest_verify.host_bridge_plan_entries_checked
            );
            println!(
                "  lifecycle_runtime_capability_flags: {}",
                manifest_verify
                    .lifecycle_runtime_capability_flags
                    .join(", ")
            );
        }
        CommandKind::VerifyArtifact { input, json } => {
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
        }
        CommandKind::UnpackArtifact { input, output_dir } => {
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
            std::fs::write(&manifest_path, relocated_manifest).map_err(|error| {
                format!("failed to write `{}`: {error}", manifest_path.display())
            })?;
            println!("unpacked nuis artifact: {}", output_dir.display());
            println!("  source: {}", input.display());
            println!("  manifest: {}", manifest_path.display());
            println!("  envelope: {}", envelope_path.display());
            println!("  artifact: {}", artifact_path.display());
            println!("  binary: {}", binary_path.display());
            println!("  packaging_mode: {}", artifact.packaging_mode);
        }
        CommandKind::VerifyBuildManifest { manifest, json } => {
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
        }
        CommandKind::InspectBenchmarks { input, json } => {
            let compiled = compile_command_input(&input)?;
            let benchmarks = collect_benchmark_inventory(&compiled.artifacts);
            if json {
                println!("{}", inspect_benchmarks_json(&input, &compiled.artifacts));
                return Ok(());
            }
            print_project_context(&compiled.resolved);
            println!("benchmark inventory: {}", input.display());
            println!(
                "  domain_unit: {}::{}",
                compiled.artifacts.nir.domain, compiled.artifacts.nir.unit
            );
            println!("  benchmark_count: {}", benchmarks.len());
            for entry in benchmarks {
                println!("  benchmark: {}", entry.symbol);
                println!("    label: {}", entry.label);
                println!(
                    "    async: {}",
                    if entry.is_async { "true" } else { "false" }
                );
                println!("    return_type: {}", entry.return_type);
                println!(
                    "    warmup_iters: {}",
                    entry
                        .warmup_iters
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "-".to_owned())
                );
                println!(
                    "    measure_iters: {}",
                    entry
                        .measure_iters
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "-".to_owned())
                );
                println!(
                    "    timeout_ms: {}",
                    entry
                        .timeout_ms
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "-".to_owned())
                );
                println!(
                    "    clock_domain: {}",
                    entry.clock_domain.as_deref().unwrap_or("-")
                );
                println!(
                    "    clock_policy: {}",
                    entry.clock_policy.as_deref().unwrap_or("-")
                );
            }
        }
        CommandKind::InspectDocs {
            input,
            json,
            output,
        } => {
            let indexes = collect_doc_indexes(&input)?;
            if json {
                let payload = inspect_docs_json(&input, &indexes);
                if let Some(path) = output {
                    write_json_output(&path, &payload)?;
                    println!("wrote doc index: {}", path.display());
                    println!("  source: {}", input.display());
                    println!("  bytes: {}", payload.len());
                } else {
                    println!("{payload}");
                }
                return Ok(());
            }
            if project::is_project_input(&input) {
                let resolved = resolve_compile_input(&input)?;
                print_project_context(&resolved);
            }
            let summaries = summarize_doc_indexes(&indexes);
            let total_items = summaries
                .iter()
                .map(|summary| summary.item_count)
                .sum::<usize>();
            println!("doc index: {}", input.display());
            println!("  module_count: {}", summaries.len());
            println!("  documented_item_count: {}", total_items);
            for (index, summary) in indexes.iter().zip(summaries.iter()) {
                println!("  module: {}", summary.module_path);
                println!("    documented_items: {}", summary.item_count);
                for item in &index.items {
                    println!("    item: {} {}", item.kind, item.path);
                    if let Some(signature) = &item.signature {
                        println!("      signature: {}", signature);
                    }
                    for line in &item.docs {
                        println!("      doc: {}", line);
                    }
                }
            }
        }
        CommandKind::InspectGalaxyDocs { galaxy, json } => {
            let summary = inspect_galaxy_doc_summary(&galaxy)?;
            if json {
                println!("{}", inspect_galaxy_docs_json(&summary));
                return Ok(());
            }
            println!("galaxy doc index: {}", summary.galaxy);
            println!("  package_id: {}", summary.package_id);
            println!("  library_module_count: {}", summary.library_module_count);
            println!(
                "  documented_library_module_count: {}",
                summary.documented_library_module_count
            );
            println!("  documented_item_count: {}", summary.documented_item_count);
            for module in summary.modules {
                println!("  library_module: {}", module.library_module);
                println!("    module_path: {}", module.module_path);
                println!("    documented_items: {}", module.documented_item_count);
                for item in module.doc_index.items {
                    println!("    item: {} {}", item.kind, item.path);
                    if let Some(signature) = item.signature {
                        println!("      signature: {}", signature);
                    }
                    for line in item.docs {
                        println!("      doc: {}", line);
                    }
                }
            }
        }
        CommandKind::InspectStdlibDocs { json } => {
            let summary = inspect_stdlib_doc_summary()?;
            if json {
                println!("{}", inspect_stdlib_docs_json(&summary));
                return Ok(());
            }
            println!("stdlib doc index");
            println!("  galaxy_count: {}", summary.galaxy_count);
            println!(
                "  documented_galaxy_count: {}",
                summary.documented_galaxy_count
            );
            println!("  documented_item_count: {}", summary.documented_item_count);
            for galaxy in summary.galaxies {
                println!("  galaxy: {}", galaxy.galaxy);
                println!("    package_id: {}", galaxy.package_id);
                println!("    library_module_count: {}", galaxy.library_module_count);
                println!(
                    "    documented_library_module_count: {}",
                    galaxy.documented_library_module_count
                );
                println!(
                    "    documented_item_count: {}",
                    galaxy.documented_item_count
                );
            }
        }
        CommandKind::InspectProjectMetadata {
            input,
            json,
            summary,
            paths_only,
        } => {
            let metadata = inspect_project_metadata(&input)?;
            if json {
                println!("{}", inspect_project_metadata_json(&metadata));
                return Ok(());
            }
            if summary {
                println!("{}", render_project_metadata_compact_summary(&metadata));
                return Ok(());
            }
            if paths_only {
                println!("{}", render_project_metadata_paths(&metadata));
                return Ok(());
            }
            println!("{}", render_project_metadata_summary(&metadata));
        }
        CommandKind::RepairProjectMetadata { input, dry_run } => {
            let (project_input, output_dir) = repair_project_metadata_target(&input)?;
            if dry_run {
                println!("project metadata repair plan");
                println!("  source: {}", input.display());
                println!("  input: {}", project_input.display());
                println!("  output_dir: {}", output_dir.display());
                println!(
                    "  command: nuisc compile \"{}\" \"{}\"",
                    project_input.display(),
                    output_dir.display()
                );
                return Ok(());
            }
            run(CommandKind::Compile {
                input: project_input.clone(),
                output_dir: output_dir.clone(),
                verbose_cache: false,
                cpu_abi: None,
                target: None,
            })?;
            let repaired_manifest = output_dir.join("nuis.build.manifest.toml");
            let repaired_summary = inspect_project_metadata(&repaired_manifest)?;
            println!(
                "project metadata repaired: input={} output_dir={}",
                project_input.display(),
                output_dir.display()
            );
            println!(
                "{}",
                render_project_metadata_compact_summary(&repaired_summary)
            );
        }
        CommandKind::CacheStatus {
            input,
            all,
            verbose_cache,
            json,
        } => {
            if all {
                let workspace_root = std::env::current_dir()
                    .map_err(|error| format!("failed to resolve current directory: {error}"))?;
                let summary = cache::compile_cache_inventory_summary(&workspace_root)?;
                if json {
                    print!(
                        "{{\"kind\":\"compile_cache_inventory\",\"workspace_root\":\"{}\",\"roots_count\":{},\"entries\":{},\"files\":{},\"bytes\":{},\"roots\":[",
                        json_escape(&summary.workspace_root.display().to_string()),
                        summary.roots.len(),
                        summary.total_entries,
                        summary.total_files,
                        summary.total_bytes
                    );
                    for (root_index, inventory) in summary.roots.iter().enumerate() {
                        if root_index > 0 {
                            print!(",");
                        }
                        print!(
                            "{{\"root\":\"{}\",\"entries\":{},\"files\":{},\"bytes\":{}",
                            json_escape(&inventory.root.display().to_string()),
                            inventory.entry_count,
                            inventory.total_files,
                            inventory.total_bytes
                        );
                        if verbose_cache {
                            print!(",\"items\":[");
                            for (entry_index, entry) in inventory.entries.iter().enumerate() {
                                if entry_index > 0 {
                                    print!(",");
                                }
                                print!(
                                    "{{\"key\":\"{}\",\"files\":{},\"bytes\":{},\"dir\":\"{}\"}}",
                                    json_escape(&entry.key),
                                    entry.file_count,
                                    entry.total_bytes,
                                    json_escape(&entry.entry_dir.display().to_string())
                                );
                            }
                            print!("]");
                        }
                        print!("}}");
                    }
                    println!("]}}");
                } else {
                    println!("compile cache inventory");
                    println!("  workspace_root: {}", summary.workspace_root.display());
                    println!("  roots: {}", summary.roots.len());
                    println!("  entries: {}", summary.total_entries);
                    println!("  files: {}", summary.total_files);
                    println!("  bytes: {}", summary.total_bytes);
                    for inventory in summary.roots {
                        println!("  root: {}", inventory.root.display());
                        println!("    entries: {}", inventory.entry_count);
                        println!("    files: {}", inventory.total_files);
                        println!("    bytes: {}", inventory.total_bytes);
                        if verbose_cache {
                            for entry in inventory.entries {
                                println!(
                                    "    entry: {} files={} bytes={} dir={}",
                                    entry.key,
                                    entry.file_count,
                                    entry.total_bytes,
                                    entry.entry_dir.display()
                                );
                            }
                        }
                    }
                }
            } else {
                let input = input.expect("cache-status input must exist when --all is not set");
                let resolved = resolve_compile_input(&input)?;
                let status = cache::compile_cache_status_with_plan(
                    &input,
                    resolved.project.as_ref(),
                    resolved.project_plan.as_ref(),
                )?;
                if json {
                    print!(
                        "{{\"kind\":\"compile_cache_status\",\"input\":\"{}\",\"root\":\"{}\",\"key\":\"{}\",\"state\":\"{}\",\"entry_dir\":\"{}\",\"files\":{},\"bytes\":{},\"fingerprint_inputs\":{}",
                        json_escape(&input.display().to_string()),
                        json_escape(&status.root.display().to_string()),
                        json_escape(&status.key),
                        if status.entry_exists { "present" } else { "missing" },
                        json_escape(&status.entry_dir.display().to_string()),
                        status.file_count,
                        status.total_bytes,
                        status.input_labels.len()
                    );
                    if verbose_cache {
                        print!(",\"inputs\":[");
                        for (index, label) in status.input_labels.iter().enumerate() {
                            if index > 0 {
                                print!(",");
                            }
                            print!("\"{}\"", json_escape(label));
                        }
                        print!("]");
                    }
                    println!("}}");
                } else {
                    println!("compile cache status: {}", input.display());
                    println!("  root: {}", status.root.display());
                    println!("  key: {}", status.key);
                    println!(
                        "  state: {}",
                        if status.entry_exists {
                            "present"
                        } else {
                            "missing"
                        }
                    );
                    println!("  entry_dir: {}", status.entry_dir.display());
                    println!("  files: {}", status.file_count);
                    println!("  bytes: {}", status.total_bytes);
                    println!("  fingerprint_inputs: {}", status.input_labels.len());
                    if verbose_cache {
                        for label in status.input_labels {
                            println!("  input: {}", label);
                        }
                    }
                }
            }
        }
        CommandKind::CleanCache { input, all, json } => {
            if all {
                let workspace_root = std::env::current_dir()
                    .map_err(|error| format!("failed to resolve current directory: {error}"))?;
                let cleaned = cache::clean_compile_cache_summary(&workspace_root)?;
                if json {
                    print!(
                        "{{\"kind\":\"compile_cache_cleaned\",\"workspace_root\":\"{}\",\"cleaned_roots\":{},\"removed_entries\":{},\"removed_bytes\":{},\"roots\":[",
                        json_escape(&cleaned.workspace_root.display().to_string()),
                        cleaned.cleaned_roots.len(),
                        cleaned.removed_entries,
                        cleaned.removed_bytes
                    );
                    for (index, root) in cleaned.cleaned_roots.iter().enumerate() {
                        if index > 0 {
                            print!(",");
                        }
                        print!(
                            "{{\"root\":\"{}\",\"removed_entries\":{},\"removed_bytes\":{}}}",
                            json_escape(&root.root.display().to_string()),
                            root.removed_entries,
                            root.removed_bytes
                        );
                    }
                    println!("]}}");
                } else {
                    println!("compile cache cleaned");
                    println!("  workspace_root: {}", cleaned.workspace_root.display());
                    println!("  cleaned_roots: {}", cleaned.cleaned_roots.len());
                    println!("  removed_entries: {}", cleaned.removed_entries);
                    println!("  removed_bytes: {}", cleaned.removed_bytes);
                    for root in cleaned.cleaned_roots {
                        println!("  root: {}", root.root.display());
                        println!("    removed_entries: {}", root.removed_entries);
                        println!("    removed_bytes: {}", root.removed_bytes);
                    }
                }
            } else {
                let input = input.expect("clean-cache input must exist when --all is not set");
                let resolved = resolve_compile_input(&input)?;
                let cleaned = cache::clean_compile_cache_with_plan(
                    &input,
                    resolved.project.as_ref(),
                    resolved.project_plan.as_ref(),
                )?;
                if json {
                    println!(
                        "{{\"kind\":\"compile_cache_cleaned\",\"input\":\"{}\",\"root\":\"{}\",\"removed_entries\":{},\"removed_bytes\":{}}}",
                        json_escape(&input.display().to_string()),
                        json_escape(&cleaned.root.display().to_string()),
                        cleaned.removed_entries,
                        cleaned.removed_bytes
                    );
                } else {
                    println!("compile cache cleaned: {}", input.display());
                    println!("  root: {}", cleaned.root.display());
                    println!("  removed_entries: {}", cleaned.removed_entries);
                    println!("  removed_bytes: {}", cleaned.removed_bytes);
                }
            }
        }
        CommandKind::PruneCache {
            input,
            all,
            keep,
            json,
        } => {
            if all {
                let workspace_root = std::env::current_dir()
                    .map_err(|error| format!("failed to resolve current directory: {error}"))?;
                let pruned = cache::prune_compile_cache_summary(&workspace_root, keep)?;
                if json {
                    print!(
                        "{{\"kind\":\"compile_cache_pruned\",\"workspace_root\":\"{}\",\"keep\":{},\"pruned_roots\":{},\"kept_entries\":{},\"removed_entries\":{},\"removed_bytes\":{},\"roots\":[",
                        json_escape(&pruned.workspace_root.display().to_string()),
                        keep,
                        pruned.pruned_roots.len(),
                        pruned.kept_entries,
                        pruned.removed_entries,
                        pruned.removed_bytes
                    );
                    for (index, root) in pruned.pruned_roots.iter().enumerate() {
                        if index > 0 {
                            print!(",");
                        }
                        print!(
                            "{{\"root\":\"{}\",\"kept_entries\":{},\"removed_entries\":{},\"removed_bytes\":{}}}",
                            json_escape(&root.root.display().to_string()),
                            root.kept_entries,
                            root.removed_entries,
                            root.removed_bytes
                        );
                    }
                    println!("]}}");
                } else {
                    println!("compile cache pruned");
                    println!("  workspace_root: {}", pruned.workspace_root.display());
                    println!("  keep: {}", keep);
                    println!("  pruned_roots: {}", pruned.pruned_roots.len());
                    println!("  kept_entries: {}", pruned.kept_entries);
                    println!("  removed_entries: {}", pruned.removed_entries);
                    println!("  removed_bytes: {}", pruned.removed_bytes);
                    for root in pruned.pruned_roots {
                        println!("  root: {}", root.root.display());
                        println!("    kept_entries: {}", root.kept_entries);
                        println!("    removed_entries: {}", root.removed_entries);
                        println!("    removed_bytes: {}", root.removed_bytes);
                    }
                }
            } else {
                let input = input.expect("cache-prune input must exist when --all is not set");
                let resolved = resolve_compile_input(&input)?;
                let pruned = cache::prune_compile_cache_with_plan(
                    &input,
                    resolved.project.as_ref(),
                    resolved.project_plan.as_ref(),
                    keep,
                )?;
                if json {
                    println!(
                        "{{\"kind\":\"compile_cache_pruned\",\"input\":\"{}\",\"root\":\"{}\",\"keep\":{},\"kept_entries\":{},\"removed_entries\":{},\"removed_bytes\":{}}}",
                        json_escape(&input.display().to_string()),
                        json_escape(&pruned.root.display().to_string()),
                        keep,
                        pruned.kept_entries,
                        pruned.removed_entries,
                        pruned.removed_bytes
                    );
                } else {
                    println!("compile cache pruned: {}", input.display());
                    println!("  root: {}", pruned.root.display());
                    println!("  keep: {}", keep);
                    println!("  kept_entries: {}", pruned.kept_entries);
                    println!("  removed_entries: {}", pruned.removed_entries);
                    println!("  removed_bytes: {}", pruned.removed_bytes);
                }
            }
        }
        CommandKind::DumpAst { input } => {
            let compiled = compile_command_input(&input)?;
            print_project_context(&compiled.resolved);
            print!("{}", render::render_ast(&compiled.artifacts.ast));
        }
        CommandKind::DumpNir { input } => {
            let compiled = compile_command_input(&input)?;
            print_project_context(&compiled.resolved);
            print_required_nustar_context(&compiled.artifacts)?;
            print!("{}", render::render_nir(&compiled.artifacts.nir));
        }
        CommandKind::DumpYir { input } => {
            let compiled = compile_command_input(&input)?;
            print_project_context(&compiled.resolved);
            print_required_nustar_context(&compiled.artifacts)?;
            print!("{}", render::render_yir(&compiled.artifacts.yir));
        }
        CommandKind::Check { input } => {
            let resolved = resolve_compile_input(&input)?;
            let artifacts = resolved.compile()?;
            let benchmarks = collect_benchmark_inventory(&artifacts);
            if success_logs_enabled() {
                println!("checked nuis source: {}", input.display());
                if let Some(project) = &resolved.project {
                    println!("project: {}", project::describe_project(project));
                }
                if let Some(plan) = &resolved.project_plan {
                    println!(
                        "project_plan: {}",
                        project::describe_project_compilation_plan(plan)
                    );
                    println!(
                        "project_abi_graph: {}",
                        project::render_project_abi_graph_line(&plan.abi_resolution)
                    );
                }
                println!(
                    "loaded_nustar: {}",
                    artifacts
                        .loaded_nustar
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                println!("nir_functions: {}", artifacts.nir.functions.len());
                println!("nir_benchmarks: {}", benchmarks.len());
                if !benchmarks.is_empty() {
                    println!(
                        "benchmark_symbols: {}",
                        benchmarks
                            .iter()
                            .map(|entry| entry.symbol.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
                println!("yir_nodes: {}", artifacts.yir.nodes.len());
                println!("yir_edges: {}", artifacts.yir.edges.len());
                println!("llvm_ir_bytes: {}", artifacts.llvm_ir.len());
            }
        }
        CommandKind::Compile {
            input,
            output_dir,
            verbose_cache,
            cpu_abi,
            target,
        } => {
            let resolved = resolve_compile_input(&input)?;
            let cpu_target = aot::resolve_cpu_build_target(
                Path::new("nustar-packages"),
                resolved
                    .project_plan
                    .as_ref()
                    .map(|plan| &plan.abi_resolution),
                cpu_abi.as_deref(),
                target.as_deref(),
            )?;
            let cache_key = cache::compute_compile_cache_key_with_plan(
                &input,
                resolved.project.as_ref(),
                resolved.project_plan.as_ref(),
            )?;
            let cache_hit = cache::lookup_compile_cache(&cache_key)?;
            let compile_fresh = || -> Result<(aot::CompileArtifacts, Vec<String>), String> {
                let artifacts =
                    resolved.compile_with_options(&pipeline::PipelineCompileOptions {
                        lowering_target: Some(
                            lowering::LoweringTargetConfig::from_cpu_build_target(&cpu_target),
                        ),
                    })?;
                let written = aot::write_and_link(
                    &resolved.effective_input_path,
                    &output_dir,
                    &artifacts.ast,
                    &artifacts.nir,
                    &artifacts.yir,
                    &artifacts.llvm_ir,
                    &cpu_target,
                )?;
                let _ = cache::store_compile_cache(&cache_key, &output_dir)?;
                Ok((written, artifacts.loaded_nustar))
            };
            let (written, loaded_nustar, used_cache_restore) = if let Some(entry) = &cache_hit {
                match cache::restore_compile_cache(entry, &output_dir).and_then(|_| {
                    aot::verify_build_manifest(&output_dir.join("nuis.build.manifest.toml"))
                }) {
                    Ok(restored_manifest) => {
                        let written = aot::compile_artifacts_for_output_dir_with_packaging_mode(
                            &resolved.effective_input_path,
                            &output_dir,
                            &restored_manifest.packaging_mode,
                        )?;
                        (written, restored_manifest.loaded_nustar, true)
                    }
                    Err(_) => {
                        let (written, loaded_nustar) = compile_fresh()?;
                        (written, loaded_nustar, false)
                    }
                }
            } else {
                let (written, loaded_nustar) = compile_fresh()?;
                (written, loaded_nustar, false)
            };
            let project_metadata =
                if let (Some(project), Some(plan)) = (&resolved.project, &resolved.project_plan) {
                    Some(project::write_project_metadata(&output_dir, project, plan)?)
                } else {
                    None
                };
            let project_text_handle_rewrite = resolved
                .project
                .as_ref()
                .map(project::summarize_project_text_handle_rewrites)
                .transpose()?;
            let doc_index = write_compile_doc_index(&input, &output_dir)?;
            let build_manifest = aot::write_build_manifest(
                &output_dir,
                &written,
                &aot::BuildManifestContext {
                    input_path: input.display().to_string(),
                    output_dir: output_dir.display().to_string(),
                    loaded_nustar: loaded_nustar.clone(),
                    compile_cache: Some(aot::BuildManifestCacheInfo {
                        status: if used_cache_restore {
                            "hit".to_owned()
                        } else {
                            "miss".to_owned()
                        },
                        key: cache_key.key.clone(),
                        root: cache_key.root.display().to_string(),
                    }),
                    project: resolved
                        .project
                        .as_ref()
                        .zip(resolved.project_plan.as_ref())
                        .map(|(project, plan)| aot::BuildManifestProjectInfo {
                            name: project.manifest.name.clone(),
                            abi_mode: if plan.abi_resolution.explicit {
                                "explicit".to_owned()
                            } else {
                                "auto-recommended".to_owned()
                            },
                            abi_graph_summary: Some(project::render_project_abi_graph_line(
                                &plan.abi_resolution,
                            )),
                            abi_entries: plan
                                .abi_resolution
                                .requirements
                                .iter()
                                .map(|item| (item.domain.clone(), item.abi.clone()))
                                .collect::<Vec<_>>(),
                            plan_summary: Some(project::describe_project_compilation_plan(plan)),
                            effective_input: Some(plan.effective_input_path.display().to_string()),
                            text_handle_rewrite_helper_hits: project_text_handle_rewrite
                                .map(|summary| summary.helper_hits)
                                .unwrap_or(0),
                            text_handle_rewrite_local_hits: project_text_handle_rewrite
                                .map(|summary| summary.local_hits)
                                .unwrap_or(0),
                            manifest_copy_path: project_metadata
                                .as_ref()
                                .map(|item| item.manifest_copy_path.clone()),
                            plan_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.plan_index_path.clone()),
                            organization_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.organization_index_path.clone()),
                            exchange_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.exchange_index_path.clone()),
                            modules_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.modules_index_path.clone()),
                            docs_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.docs_index_path.clone()),
                            docs_module_count: project_metadata
                                .as_ref()
                                .map(|item| item.docs_summary.modules)
                                .unwrap_or(0),
                            docs_documented_module_count: project_metadata
                                .as_ref()
                                .map(|item| item.docs_summary.documented_modules)
                                .unwrap_or(0),
                            docs_documented_item_count: project_metadata
                                .as_ref()
                                .map(|item| item.docs_summary.documented_items)
                                .unwrap_or(0),
                            imports_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.imports_index_path.clone()),
                            imports_library_count: project_metadata
                                .as_ref()
                                .map(|item| item.imports_summary.libraries)
                                .unwrap_or(0),
                            imports_visible_library_count: project_metadata
                                .as_ref()
                                .map(|item| item.imports_summary.visible_libraries)
                                .unwrap_or(0),
                            imports_visible_module_count: project_metadata
                                .as_ref()
                                .map(|item| item.imports_summary.visible_modules)
                                .unwrap_or(0),
                            imports_documented_visible_module_count: project_metadata
                                .as_ref()
                                .map(|item| item.imports_summary.documented_visible_modules)
                                .unwrap_or(0),
                            imports_documented_visible_item_count: project_metadata
                                .as_ref()
                                .map(|item| item.imports_summary.documented_visible_items)
                                .unwrap_or(0),
                            galaxy_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.galaxy_index_path.clone()),
                            galaxy_count: project_metadata
                                .as_ref()
                                .map(|item| item.galaxy_summary.galaxies)
                                .unwrap_or(0),
                            galaxy_documented_count: project_metadata
                                .as_ref()
                                .map(|item| item.galaxy_summary.documented_galaxies)
                                .unwrap_or(0),
                            galaxy_documented_library_module_count: project_metadata
                                .as_ref()
                                .map(|item| item.galaxy_summary.documented_library_modules)
                                .unwrap_or(0),
                            galaxy_documented_item_count: project_metadata
                                .as_ref()
                                .map(|item| item.galaxy_summary.documented_items)
                                .unwrap_or(0),
                            links_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.links_index_path.clone()),
                            packet_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.packet_index_path.clone()),
                            host_ffi_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.host_ffi_index_path.clone()),
                            abi_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.abi_index_path.clone()),
                        }),
                    doc_index: Some(doc_index.clone()),
                    cpu_target: cpu_target.clone(),
                },
            )?;
            if success_logs_enabled() {
                println!("compiled nuis source: {}", input.display());
                println!(
                    "compile_cache: {} ({})",
                    if used_cache_restore { "hit" } else { "miss" },
                    cache_key.key
                );
                println!("compile_cache_inputs: {}", cache_key.input_labels.len());
                if verbose_cache {
                    for label in &cache_key.input_labels {
                        println!("  compile_cache_input: {}", label);
                    }
                }
                if let Some(project) = &resolved.project {
                    println!("project: {}", project::describe_project(project));
                    if let Ok(graph) = project::describe_project_abi_graph(project) {
                        println!("project_abi_graph: {}", graph);
                    }
                }
                if let Some(plan) = &resolved.project_plan {
                    println!(
                        "project_plan: {}",
                        project::describe_project_compilation_plan(plan)
                    );
                    println!(
                        "project_abi_graph: {}",
                        project::render_project_abi_graph_line(&plan.abi_resolution)
                    );
                }
                println!(
                    "loaded_nustar: {}",
                    loaded_nustar
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                println!("cpu_target_abi: {}", cpu_target.abi);
                println!(
                    "cpu_target_machine: {}-{}",
                    cpu_target.machine_arch, cpu_target.machine_os
                );
                println!("cpu_target_clang: {}", cpu_target.clang_target);
                println!(
                    "cpu_target_cross: {}",
                    if cpu_target.cross_compile {
                        "true"
                    } else {
                        "false"
                    }
                );
                if let Some(plan) = &resolved.project_plan {
                    for item in &plan.abi_resolution.requirements {
                        println!("abi: {}={}", item.domain, item.abi);
                        if let Ok(manifest) = registry::load_manifest_for_domain(
                            Path::new("nustar-packages"),
                            &item.domain,
                        ) {
                            if let Ok(target) =
                                registry::registered_abi_target(&manifest, &item.abi)
                            {
                                println!(
                                    "  abi_target_machine: {}-{}",
                                    target.machine_arch, target.machine_os
                                );
                                println!("  abi_target_object: {}", target.object_format);
                                println!("  abi_target_calling: {}", target.calling_abi);
                                println!("  abi_target_clang: {}", target.clang_target);
                                if let Some(backend) = target.backend_family {
                                    println!("  abi_target_backend: {}", backend);
                                }
                                if let Some(vendor) = target.vendor {
                                    println!("  abi_target_vendor: {}", vendor);
                                }
                                if let Some(device_class) = target.device_class {
                                    println!("  abi_target_device: {}", device_class);
                                }
                                println!(
                                    "  abi_target_host_adaptive: {}",
                                    if target.host_adaptive {
                                        "true"
                                    } else {
                                        "false"
                                    }
                                );
                            }
                        }
                    }
                }
                println!("ast: {}", written.ast_path);
                println!("nir: {}", written.nir_path);
                println!("yir: {}", written.yir_path);
                println!("llvm_ir: {}", written.llvm_ir_path);
                println!("packaging_mode: {}", written.packaging_mode);
                println!("binary: {}", written.binary_path);
                println!(
                    "compiled_artifact: {}",
                    output_dir.join("nuis.compiled.artifact").display()
                );
                println!("doc_index: {}", doc_index.path);
                println!("doc_index_modules: {}", doc_index.module_count);
                println!(
                    "doc_index_documented_items: {}",
                    doc_index.documented_item_count
                );
                println!("build_manifest: {}", build_manifest);
                if let Some(metadata) = &project_metadata {
                    println!("project_manifest: {}", metadata.manifest_copy_path);
                    println!("project_plan_index: {}", metadata.plan_index_path);
                    println!("project_organization: {}", metadata.organization_index_path);
                    println!("project_exchange: {}", metadata.exchange_index_path);
                    println!("project_modules: {}", metadata.modules_index_path);
                    println!("project_docs: {}", metadata.docs_index_path);
                    println!("project_imports: {}", metadata.imports_index_path);
                    println!("project_galaxy: {}", metadata.galaxy_index_path);
                    println!("project_links: {}", metadata.links_index_path);
                    println!("project_packet: {}", metadata.packet_index_path);
                    println!("project_host_ffi: {}", metadata.host_ffi_index_path);
                    println!("project_abi: {}", metadata.abi_index_path);
                }
            }
        }
    }

    Ok(())
}
