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
mod command_cache;
mod command_compile;
mod command_helpers;
mod command_inspect;
mod command_nustar;
mod command_project_metadata;
mod command_registry;
pub mod data_markers;
mod domain_build_report;
pub mod engine;
pub mod errors;
mod execution_inspect;
mod execution_inspect_report;
pub mod fmt;
pub mod frontend;
mod host_ffi_index;
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

pub use cli::CommandKind;

#[allow(unused_imports)]
pub(crate) use crate::command_helpers::{
    inspect_artifact_container_for_input, load_nuis_compiled_artifact, NUSTAR_REGISTRY_ROOT,
};

pub use crate::command_helpers::{
    nuisc_compile_pipeline_brief, project_compile_samples_brief, project_compile_workflow_brief,
    project_galaxy_workflow_brief, project_test_workflow_brief,
};

#[allow(unused_imports)]
use crate::artifact_report::{
    artifact_report_json, artifact_report_summary_lines, domain_build_contract_summary_json,
    domain_registry_json, inspect_artifact_json, reconstruct_manifest_report_from_artifact,
    verify_artifact_json, verify_build_manifest_json,
};

#[allow(unused_imports)]
use crate::execution_inspect::{execution_inspect_issues, ExecutionInspectOverview};
#[cfg(test)]
use crate::execution_inspect::{ExecutionInspectDomainOverview, ExecutionInspectIssue};
#[allow(unused_imports)]
use crate::inspect_report::{
    collect_benchmark_inventory, collect_doc_indexes, inspect_benchmarks_json, inspect_docs_json,
    inspect_galaxy_doc_summary, inspect_galaxy_docs_json, inspect_stdlib_doc_summary,
    inspect_stdlib_docs_json, write_json_output,
};

pub(crate) use crate::json_report::{
    artifact_lowering_units_json, json_bool_field, json_escape, json_optional_i64_field,
    json_optional_string_field, json_string_array_field, json_string_field, json_usize_field,
};

#[allow(unused_imports)]
use crate::link_report::link_plan_json;

#[allow(unused_imports)]
use crate::project_metadata_report::{
    inspect_project_metadata, inspect_project_metadata_json,
    render_project_metadata_compact_summary, render_project_metadata_paths,
    repair_project_metadata_target, resolve_build_manifest_path, ProjectMetadataSummary,
};

#[cfg(test)]
use crate::domain_build_report::domain_build_unit_contract_json;
#[allow(unused_imports)]
use crate::domain_build_report::{
    domain_build_unit_verification_verdict, evaluate_domain_build_contract_drift,
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
            command_artifact::run_inspect_artifact(input, json)?
        }
        CommandKind::InspectExecution { input, json } => {
            command_inspect::run_inspect_execution(input, json)?
        }
        CommandKind::ArtifactReport {
            input,
            json,
            summary,
        } => command_artifact::run_artifact_report(input, json, summary)?,
        CommandKind::VerifyArtifact { input, json } => {
            command_artifact::run_verify_artifact(input, json)?
        }
        CommandKind::UnpackArtifact { input, output_dir } => {
            command_artifact::run_unpack_artifact(input, output_dir)?
        }
        CommandKind::VerifyBuildManifest { manifest, json } => {
            command_artifact::run_verify_build_manifest(manifest, json)?
        }
        CommandKind::InspectBenchmarks { input, json } => {
            command_inspect::run_inspect_benchmarks(input, json)?
        }
        CommandKind::InspectDocs {
            input,
            json,
            output,
        } => command_inspect::run_inspect_docs(input, json, output)?,
        CommandKind::InspectGalaxyDocs { galaxy, json } => {
            command_inspect::run_inspect_galaxy_docs(galaxy, json)?
        }
        CommandKind::InspectStdlibDocs { json } => command_inspect::run_inspect_stdlib_docs(json)?,
        CommandKind::InspectProjectMetadata {
            input,
            json,
            summary,
            paths_only,
        } => command_project_metadata::run_inspect_project_metadata(
            input, json, summary, paths_only,
        )?,
        CommandKind::RepairProjectMetadata { input, dry_run } => {
            command_project_metadata::run_repair_project_metadata(input, dry_run)?
        }
        CommandKind::CacheStatus {
            input,
            all,
            verbose_cache,
            json,
        } => command_cache::run_cache_status(input, all, verbose_cache, json)?,
        CommandKind::CleanCache { input, all, json } => {
            command_cache::run_clean_cache(input, all, json)?
        }
        CommandKind::PruneCache {
            input,
            all,
            keep,
            json,
        } => command_cache::run_prune_cache(input, all, keep, json)?,
        CommandKind::DumpAst { input } => command_compile::run_dump_ast(input)?,
        CommandKind::DumpNir { input } => command_compile::run_dump_nir(input)?,
        CommandKind::DumpYir { input } => command_compile::run_dump_yir(input)?,
        CommandKind::Check { input } => command_compile::run_check(input)?,
        CommandKind::Compile {
            input,
            output_dir,
            verbose_cache,
            cpu_abi,
            target,
            packaging_mode,
        } => command_compile::run_compile(
            input,
            output_dir,
            verbose_cache,
            cpu_abi,
            target,
            packaging_mode,
        )?,
    }

    Ok(())
}
