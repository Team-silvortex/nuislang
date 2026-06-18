pub mod aot;
pub mod cache;
pub mod cli;
pub mod codegen_wasm;
pub mod engine;
pub mod errors;
pub mod fmt;
pub mod frontend;
pub mod lowering;
pub mod nir_verify;
pub mod nustar_binary;
pub mod optimize;
pub mod pipeline;
pub mod project;
pub mod registry;
pub mod render;

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub use cli::CommandKind;

const NUSTAR_REGISTRY_ROOT: &str = "nustar-packages";

struct CompiledCommandInput {
    resolved: pipeline::ResolvedCompileInput,
    artifacts: pipeline::PipelineArtifacts,
}

fn json_escape(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}

fn json_bool_field(name: &str, value: bool) -> String {
    format!("\"{}\":{}", name, if value { "true" } else { "false" })
}

fn json_string_field(name: &str, value: &str) -> String {
    format!("\"{}\":\"{}\"", name, json_escape(value))
}

fn json_usize_field(name: &str, value: usize) -> String {
    format!("\"{}\":{}", name, value)
}

fn json_string_array_field(name: &str, values: &[String]) -> String {
    let entries = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("\"{}\":[{}]", name, entries)
}

pub fn project_compile_workflow_brief() -> &'static str {
    "health -> structure -> scheduler -> abi_lock -> check -> test -> build -> release_check"
}

pub fn project_compile_samples_brief() -> &'static str {
    "health=nuis project-doctor <project-dir>; structure=nuis project-status <project-dir>; scheduler=nuis scheduler-view <project-dir>; abi_lock=nuis project-lock-abi <project-dir>; compile=nuis check <project-dir> -> nuis test <project-dir> -> nuis build <project-dir> -> nuis release-check <project-dir> <output-dir>"
}

pub fn project_test_workflow_brief() -> &'static str {
    "list=nuis test --list <project-dir>; exact=nuis test --exact <project-dir> <test-name>; ignored=nuis test --ignored <project-dir>; include_ignored=nuis test --include-ignored <project-dir>"
}

pub fn project_galaxy_workflow_brief() -> &'static str {
    "galaxy=nuis galaxy init <project-dir> -> nuis galaxy check <project-dir> -> nuis galaxy lock-deps <project-dir> -> nuis galaxy sync-deps <project-dir> -> nuis project-doctor <project-dir>"
}

fn resolve_compile_input(input: &Path) -> Result<pipeline::ResolvedCompileInput, String> {
    pipeline::resolve_compile_input(input)
}

fn compile_command_input(input: &Path) -> Result<CompiledCommandInput, String> {
    let resolved = resolve_compile_input(input)?;
    let artifacts = resolved.compile()?;
    Ok(CompiledCommandInput {
        resolved,
        artifacts,
    })
}

fn load_nuis_executable_envelope(input: &Path) -> Result<aot::NuisExecutableEnvelope, String> {
    let bytes = std::fs::read(input)
        .map_err(|error| format!("failed to read `{}`: {error}", input.display()))?;
    if bytes.starts_with(b"NENV") {
        aot::decode_nuis_executable_envelope_binary(&bytes)
    } else if input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.build.manifest.toml")
        .unwrap_or(false)
    {
        let report = aot::verify_build_manifest(input)?;
        aot::parse_nuis_executable_envelope(Path::new(&report.envelope_path))
    } else {
        aot::parse_nuis_executable_envelope(input)
    }
}

fn load_nuis_compiled_artifact(input: &Path) -> Result<aot::NuisCompiledArtifact, String> {
    let bytes = std::fs::read(input)
        .map_err(|error| format!("failed to read `{}`: {error}", input.display()))?;
    if bytes.starts_with(b"NART") {
        aot::decode_nuis_compiled_artifact_binary(&bytes)
    } else if input
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "nuis.build.manifest.toml")
        .unwrap_or(false)
    {
        let report = aot::verify_build_manifest(input)?;
        aot::parse_nuis_compiled_artifact(Path::new(&report.artifact_path))
    } else {
        aot::parse_nuis_compiled_artifact(input)
    }
}

fn inspect_artifact_json(input: &Path, artifact: &aot::NuisCompiledArtifact) -> String {
    let fields = vec![
        json_string_field("kind", "nuis_artifact_inspect"),
        json_string_field("input", &input.display().to_string()),
        json_string_field("schema", &artifact.schema),
        json_string_field("packaging_mode", &artifact.packaging_mode),
        json_string_field("cpu_target_abi", &artifact.cpu_target_abi),
        json_string_field("cpu_target_machine_arch", &artifact.cpu_target_machine_arch),
        json_string_field("cpu_target_machine_os", &artifact.cpu_target_machine_os),
        json_string_field(
            "cpu_target_object_format",
            &artifact.cpu_target_object_format,
        ),
        json_string_field("cpu_target_calling_abi", &artifact.cpu_target_calling_abi),
        json_string_field("binary_name", &artifact.binary_name),
        json_usize_field("binary_bytes", artifact.binary_bytes),
        json_usize_field("build_manifest_bytes", artifact.build_manifest_bytes),
        json_string_field("envelope_schema", &artifact.envelope.schema),
        json_string_array_field(
            "envelope_contract_families",
            &artifact.envelope.contract_families,
        ),
        json_string_field("lifecycle_schema", &artifact.lifecycle.schema),
        json_string_field(
            "lifecycle_bootstrap_entry",
            &artifact.lifecycle.bootstrap_entry,
        ),
        json_string_field("lifecycle_tick_policy", &artifact.lifecycle.tick_policy),
        json_string_field(
            "lifecycle_shutdown_policy",
            &artifact.lifecycle.shutdown_policy,
        ),
        json_string_field("lifecycle_yalivia_rpc", &artifact.lifecycle.yalivia_rpc),
        json_usize_field(
            "lifecycle_hook_count",
            artifact.lifecycle.hook_surface.len(),
        ),
        json_string_array_field("lifecycle_hook_surface", &artifact.lifecycle.hook_surface),
        json_usize_field(
            "lifecycle_export_count",
            artifact.lifecycle.export_surface.len(),
        ),
        json_string_array_field(
            "lifecycle_export_surface",
            &artifact.lifecycle.export_surface,
        ),
        json_string_array_field(
            "lifecycle_runtime_capability_flags",
            &artifact.lifecycle.runtime_capability_flags,
        ),
    ];
    format!("{{{}}}", fields.join(","))
}

fn verify_artifact_json(input: &Path, report: &aot::NuisCompiledArtifactVerifyReport) -> String {
    let fields = vec![
        json_string_field("kind", "nuis_artifact_verify"),
        json_string_field("input", &input.display().to_string()),
        json_string_field("schema", &report.schema),
        json_string_field("packaging_mode", &report.packaging_mode),
        json_string_field("binary_name", &report.binary_name),
        json_usize_field("binary_bytes", report.binary_bytes),
        json_usize_field("build_manifest_bytes", report.build_manifest_bytes),
        json_string_field("envelope_schema", &report.envelope_schema),
        json_usize_field("envelope_package_count", report.envelope_package_count),
        json_string_field("lifecycle_schema", &report.lifecycle_schema),
        json_string_field(
            "lifecycle_bootstrap_entry",
            &report.lifecycle_bootstrap_entry,
        ),
        json_string_field("lifecycle_tick_policy", &report.lifecycle_tick_policy),
        json_string_field(
            "lifecycle_shutdown_policy",
            &report.lifecycle_shutdown_policy,
        ),
        json_string_field("lifecycle_yalivia_rpc", &report.lifecycle_yalivia_rpc),
        json_usize_field("lifecycle_hook_count", report.lifecycle_hook_count),
        json_string_array_field("lifecycle_hook_surface", &report.lifecycle_hook_surface),
        json_usize_field("lifecycle_export_count", report.lifecycle_export_count),
        json_string_array_field("lifecycle_export_surface", &report.lifecycle_export_surface),
        json_string_array_field(
            "lifecycle_runtime_capability_flags",
            &report.lifecycle_runtime_capability_flags,
        ),
        json_bool_field(
            "lifecycle_contract_consistent",
            report.lifecycle_contract_consistent,
        ),
        json_bool_field(
            "lifecycle_runtime_capability_flags_consistent",
            report.lifecycle_runtime_capability_flags_consistent,
        ),
        json_usize_field(
            "execution_contracts_checked",
            report.execution_contracts_checked,
        ),
        json_string_field("cpu_target_abi", &report.cpu_target_abi),
        json_string_field("cpu_target_machine_arch", &report.cpu_target_machine_arch),
        json_string_field("cpu_target_machine_os", &report.cpu_target_machine_os),
        json_string_field("cpu_target_object_format", &report.cpu_target_object_format),
        json_string_field("cpu_target_calling_abi", &report.cpu_target_calling_abi),
        json_bool_field(
            "artifact_roundtrip_verified",
            report.artifact_roundtrip_verified,
        ),
    ];
    format!("{{{}}}", fields.join(","))
}

fn verify_build_manifest_json(input: &Path, report: &aot::BuildManifestVerifyReport) -> String {
    let fields = vec![
        json_string_field("kind", "nuis_build_manifest_verify"),
        json_string_field("input", &input.display().to_string()),
        json_string_field("schema", &report.schema),
        json_string_field("manifest_input", &report.input),
        json_string_field("output_dir", &report.output_dir),
        json_string_field("packaging_mode", &report.packaging_mode),
        json_string_field("envelope_path", &report.envelope_path),
        json_string_field("envelope_schema", &report.envelope_schema),
        json_usize_field("envelope_package_count", report.envelope_package_count),
        json_string_field("artifact_path", &report.artifact_path),
        json_string_field("artifact_schema", &report.artifact_schema),
        json_string_field("artifact_binary_name", &report.artifact_binary_name),
        json_usize_field("artifact_binary_bytes", report.artifact_binary_bytes),
        json_string_field("lifecycle_schema", &report.lifecycle_schema),
        json_string_field(
            "lifecycle_bootstrap_entry",
            &report.lifecycle_bootstrap_entry,
        ),
        json_string_field("lifecycle_tick_policy", &report.lifecycle_tick_policy),
        json_string_field(
            "lifecycle_shutdown_policy",
            &report.lifecycle_shutdown_policy,
        ),
        json_string_field("lifecycle_yalivia_rpc", &report.lifecycle_yalivia_rpc),
        json_usize_field("lifecycle_hook_count", report.lifecycle_hook_count),
        json_string_array_field("lifecycle_hook_surface", &report.lifecycle_hook_surface),
        json_usize_field("lifecycle_export_count", report.lifecycle_export_count),
        json_string_array_field("lifecycle_export_surface", &report.lifecycle_export_surface),
        json_string_array_field(
            "lifecycle_runtime_capability_flags",
            &report.lifecycle_runtime_capability_flags,
        ),
        json_usize_field(
            "execution_contracts_checked",
            report.execution_contracts_checked,
        ),
        json_string_field("cpu_target_abi", &report.cpu_target_abi),
        json_string_field("cpu_target_machine_arch", &report.cpu_target_machine_arch),
        json_string_field("cpu_target_machine_os", &report.cpu_target_machine_os),
        json_string_field("cpu_target_object_format", &report.cpu_target_object_format),
        json_string_field("cpu_target_calling_abi", &report.cpu_target_calling_abi),
        json_string_field("cpu_target_clang", &report.cpu_target_clang),
        json_bool_field("cpu_target_cross", report.cpu_target_cross),
        json_usize_field("artifacts_checked", report.artifacts_checked),
        json_usize_field("project_metadata_checked", report.project_metadata_checked),
    ];
    format!("{{{}}}", fields.join(","))
}

fn reconstruct_manifest_report_from_artifact(
    input: &Path,
    artifact: &aot::NuisCompiledArtifact,
) -> Result<(PathBuf, aot::BuildManifestVerifyReport), String> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("failed to read current time: {error}"))?
        .as_nanos();
    let temp_root = std::env::temp_dir().join(format!("nuis_artifact_report_{nonce}"));
    std::fs::create_dir_all(&temp_root)
        .map_err(|error| format!("failed to create `{}`: {error}", temp_root.display()))?;

    let manifest_path = temp_root.join("nuis.build.manifest.toml");
    let envelope_path = temp_root.join("nuis.executable.envelope.toml");
    let artifact_path = temp_root.join("nuis.compiled.artifact");
    let binary_path = temp_root.join(&artifact.binary_name);

    let result = (|| {
        std::fs::write(&binary_path, &artifact.binary_blob)
            .map_err(|error| format!("failed to write `{}`: {error}", binary_path.display()))?;
        aot::write_nuis_executable_envelope(&envelope_path, &artifact.envelope)?;
        let relocated_manifest = aot::render_relocated_unpacked_build_manifest(
            artifact,
            &temp_root,
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
        let report = aot::verify_build_manifest(&manifest_path)?;
        Ok((manifest_path.clone(), report))
    })();

    let _ = std::fs::remove_dir_all(&temp_root);
    result.map_err(|error: String| {
        format!(
            "failed to reconstruct build manifest context for `{}`: {error}",
            input.display()
        )
    })
}

fn artifact_report_json(
    input: &Path,
    artifact: &aot::NuisCompiledArtifact,
    artifact_verify_input: &Path,
    artifact_verify: &aot::NuisCompiledArtifactVerifyReport,
    manifest_input: &Path,
    manifest_verify: &aot::BuildManifestVerifyReport,
    manifest_verify_reconstructed: bool,
) -> String {
    let fields = vec![
        json_string_field("kind", "nuis_artifact_report"),
        json_string_field("input", &input.display().to_string()),
        json_bool_field(
            "manifest_verify_reconstructed",
            manifest_verify_reconstructed,
        ),
        format!(
            "\"artifact_inspect\":{}",
            inspect_artifact_json(input, artifact)
        ),
        format!(
            "\"artifact_verify\":{}",
            verify_artifact_json(artifact_verify_input, artifact_verify)
        ),
        format!(
            "\"manifest_verify\":{}",
            verify_build_manifest_json(manifest_input, manifest_verify)
        ),
    ];
    format!("{{{}}}", fields.join(","))
}

fn print_project_context(resolved: &pipeline::ResolvedCompileInput) {
    if let Some(project) = &resolved.project {
        eprintln!("nuisc: {}", project::describe_project(project));
    }
}

fn print_required_nustar_context(artifacts: &pipeline::PipelineArtifacts) -> Result<(), String> {
    let required =
        registry::load_required_manifests(Path::new(NUSTAR_REGISTRY_ROOT), &artifacts.yir)?;
    registry::validate_unit_binding(&required, &artifacts.ast.domain, &artifacts.ast.unit)?;
    eprintln!(
        "nuisc: lazily loaded nustar = {}",
        required
            .iter()
            .map(|manifest| manifest.package_id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(())
}

pub fn run(command: CommandKind) -> Result<(), String> {
    let frontend = frontend::frontend_name();
    let backend = codegen_wasm::backend_name();
    let engine = engine::default_engine();

    match command {
        CommandKind::Status => {
            let index = registry::load_index(Path::new("nustar-packages"))?;
            println!(
                "nuisc compiler core: topology-first scheduler frontend ({frontend} -> {backend}, yir={}, profile={}, indexed_nustar={})",
                engine.version,
                engine.profile,
                index.len()
            );
            for entry in index {
                println!(
                    "  - {} [{}] -> {}",
                    entry.package_id,
                    entry.domain_family,
                    registry::manifest_path(Path::new("nustar-packages"), &entry).display()
                );
            }
        }
        CommandKind::Registry { json } => {
            let registrations = registry::load_registered_domains(Path::new("nustar-packages"))?;
            if registrations.is_empty() {
                let placeholder_error = errors::NuiscError {
                    message: "no nustar packages discovered",
                };
                return Err(placeholder_error.message.to_owned());
            }
            if json {
                let contracts = registrations
                    .iter()
                    .map(registry::domain_registration_json)
                    .collect::<Vec<_>>();
                println!(
                    "{{{},{},{}}}",
                    format!(
                        "\"contract_schema\":\"{}\"",
                        registry::NUSTAR_DOMAIN_CONTRACT_SCHEMA
                    ),
                    json_bool_field("registry_indexed", true),
                    format!("\"domains\":[{}]", contracts.join(","))
                );
                return Ok(());
            }
            for registration in registrations {
                let manifest = registry::load_manifest_for_domain(
                    Path::new("nustar-packages"),
                    &registration.domain_family,
                )?;
                let capability = registry::capability_summary(&manifest);
                let execution = registry::execution_summary(&manifest);
                let scheduler = registry::scheduler_summary(&manifest);
                println!("package: {}", manifest.package_id);
                println!("  schema: {}", manifest.manifest_schema);
                println!("  domain: {}", manifest.domain_family);
                println!("  frontend: {}", manifest.frontend);
                println!("  crate: {}", manifest.entry_crate);
                println!("  ast_entry: {}", manifest.ast_entry);
                println!("  nir_entry: {}", manifest.nir_entry);
                println!("  yir_lowering_entry: {}", manifest.yir_lowering_entry);
                println!("  part_verify_entry: {}", manifest.part_verify_entry);
                println!("  ast_surface: {}", manifest.ast_surface.join(", "));
                println!("  nir_surface: {}", manifest.nir_surface.join(", "));
                println!("  yir_lowering: {}", manifest.yir_lowering.join(", "));
                println!("  part_verify: {}", manifest.part_verify.join(", "));
                println!("  binary_extension: {}", manifest.binary_extension);
                println!("  package_layout: {}", manifest.package_layout);
                println!("  machine_abi_policy: {}", manifest.machine_abi_policy);
                if !manifest.abi_profiles.is_empty() {
                    println!("  abi_profiles: {}", manifest.abi_profiles.join(", "));
                }
                if !manifest.abi_capabilities.is_empty() {
                    println!(
                        "  abi_capabilities: {}",
                        manifest.abi_capabilities.join(", ")
                    );
                }
                println!(
                    "  implementation_kinds: {}",
                    manifest.implementation_kinds.join(", ")
                );
                println!("  loader_entry: {}", manifest.loader_entry);
                println!("  loader_abi: {}", manifest.loader_abi);
                if !manifest.host_ffi_surface.is_empty() {
                    println!(
                        "  host_ffi_surface: {}",
                        manifest.host_ffi_surface.join(", ")
                    );
                    println!("  host_ffi_abis: {}", manifest.host_ffi_abis.join(", "));
                    println!("  host_ffi_bridge: {}", manifest.host_ffi_bridge);
                }
                if !capability.support_surface.is_empty() {
                    println!(
                        "  support_surface: {}",
                        capability.support_surface.join(", ")
                    );
                }
                if !capability.support_profile_slots.is_empty() {
                    println!(
                        "  support_profile_slots: {}",
                        capability.support_profile_slots.join(", ")
                    );
                }
                if !capability.default_lanes.is_empty() {
                    println!("  default_lanes: {}", capability.default_lanes.join(", "));
                }
                println!("  clock_domain_id: {}", capability.clock.domain_id);
                println!("  clock_kind: {}", capability.clock.kind);
                println!("  clock_epoch_kind: {}", capability.clock.epoch_kind);
                println!("  clock_resolution: {}", capability.clock.resolution);
                println!(
                    "  clock_bridge_default: {}",
                    capability.clock.bridge_default
                );
                println!(
                    "  execution_skeleton_version: {}",
                    execution.skeleton_version
                );
                println!("  execution_function_kind: {}", execution.function_kind);
                println!("  execution_graph_kind: {}", execution.graph_kind);
                println!("  execution_domain: {}", execution.execution_domain);
                println!(
                    "  execution_default_time_mode: {}",
                    execution.default_time_mode
                );
                println!("  execution_contract_family: {}", execution.contract_family);
                println!("  scheduler_contract_stack: {}", scheduler.contract_stack);
                println!("  scheduler_result_roles: {}", scheduler.result_roles);
                if let Some(navigation) = scheduler.sample_navigation {
                    println!("  scheduler_sample_navigation: {}", navigation);
                }
                if let Some(samples) = scheduler.result_samples {
                    println!("  scheduler_result_samples: {}", samples);
                }
                if let Some(samples) = scheduler.transport_samples {
                    println!("  scheduler_transport_samples: {}", samples);
                }
                println!("  scheduler_summary_api: {}", scheduler.summary_api);
                if let Some(samples) = scheduler.summary_samples {
                    println!("  scheduler_summary_samples: {}", samples);
                }
                println!(
                    "  scheduler_observer_classes: {}",
                    scheduler.observer_classes
                );
                println!("  profiles: {}", manifest.profiles.join(", "));
                println!(
                    "  resource_families: {}",
                    manifest.resource_families.join(", ")
                );
                println!(
                    "  unit_types: {}",
                    if manifest.unit_types.is_empty() {
                        "<any>".to_owned()
                    } else {
                        manifest.unit_types.join(", ")
                    }
                );
                println!(
                    "  lowering_targets: {}",
                    manifest.lowering_targets.join(", ")
                );
                println!("  ops: {}", manifest.ops.join(", "));
            }
        }
        CommandKind::Fmt { input } => {
            let report = fmt::format_input(&input)?;
            println!("formatted nuis input: {}", input.display());
            println!("  total_files: {}", report.total_files);
            println!("  changed_files: {}", report.changed_files.len());
            for file in report.changed_files {
                println!("  - {}", file);
            }
        }
        CommandKind::Bindings { input } => {
            let compiled = compile_command_input(&input)?;
            let artifacts = &compiled.artifacts;
            let declared_used_units = artifacts
                .ast
                .uses
                .iter()
                .map(|item| (item.domain.clone(), item.unit.clone()))
                .collect::<Vec<_>>();
            let declared_externs = artifacts
                .ast
                .externs
                .iter()
                .map(|item| (item.abi.clone(), item.name.clone()))
                .chain(
                    artifacts
                        .ast
                        .extern_interfaces
                        .iter()
                        .flat_map(|interface| {
                            interface.methods.iter().map(move |method| {
                                (
                                    method.abi.clone(),
                                    format!("{}__{}", interface.name, method.name),
                                )
                            })
                        }),
                )
                .collect::<Vec<_>>();
            let plan = registry::plan_bindings(
                Path::new("nustar-packages"),
                &artifacts.nir,
                &artifacts.yir,
                &artifacts.ast.domain,
                &artifacts.ast.unit,
                &declared_used_units,
                &declared_externs,
            )?;
            println!("binding plan for: {}", input.display());
            if let Some(project) = &compiled.resolved.project {
                println!("project: {}", project::describe_project(project));
            }
            for binding in plan.bindings {
                println!("package: {}", binding.package_id);
                println!("  domain: {}", binding.domain_family);
                println!("  frontend: {}", binding.frontend);
                println!("  crate: {}", binding.entry_crate);
                println!("  ast_entry: {}", binding.ast_entry);
                println!("  nir_entry: {}", binding.nir_entry);
                println!("  yir_lowering_entry: {}", binding.yir_lowering_entry);
                println!("  part_verify_entry: {}", binding.part_verify_entry);
                println!("  machine_abi_policy: {}", binding.machine_abi_policy);
                if !binding.abi_profiles.is_empty() {
                    println!("  abi_profiles: {}", binding.abi_profiles.join(", "));
                }
                if !binding.abi_capabilities.is_empty() {
                    println!(
                        "  abi_capabilities: {}",
                        binding.abi_capabilities.join(", ")
                    );
                }
                println!("  ast_surface: {}", binding.ast_surface.join(", "));
                println!("  nir_surface: {}", binding.nir_surface.join(", "));
                println!("  yir_lowering: {}", binding.yir_lowering.join(", "));
                println!("  part_verify: {}", binding.part_verify.join(", "));
                if !binding.support_surface.is_empty() {
                    println!("  support_surface: {}", binding.support_surface.join(", "));
                }
                if !binding.support_profile_slots.is_empty() {
                    println!(
                        "  support_profile_slots: {}",
                        binding.support_profile_slots.join(", ")
                    );
                }
                if !binding.default_lanes.is_empty() {
                    println!("  default_lanes: {}", binding.default_lanes.join(", "));
                }
                println!(
                    "  execution_skeleton_version: {}",
                    binding.execution.skeleton_version
                );
                println!(
                    "  execution_function_kind: {}",
                    binding.execution.function_kind
                );
                println!("  execution_graph_kind: {}", binding.execution.graph_kind);
                println!("  execution_domain: {}", binding.execution.execution_domain);
                println!(
                    "  execution_default_time_mode: {}",
                    binding.execution.default_time_mode
                );
                println!(
                    "  execution_contract_family: {}",
                    binding.execution.contract_family
                );
                if !binding.execution.lowering_targets.is_empty() {
                    println!(
                        "  execution_lowering_targets: {}",
                        binding.execution.lowering_targets.join(", ")
                    );
                }
                if !binding.matched_support_surface.is_empty() {
                    println!(
                        "  matched_support_surface: {}",
                        binding.matched_support_surface.join(", ")
                    );
                }
                if !binding.matched_support_profile_slots.is_empty() {
                    println!(
                        "  matched_support_profile_slots: {}",
                        binding.matched_support_profile_slots.join(", ")
                    );
                }
                if !binding.covered_support_profile_slots.is_empty() {
                    println!(
                        "  covered_support_profile_slots: {}",
                        binding.covered_support_profile_slots.join(", ")
                    );
                }
                if !binding.uncovered_support_profile_slots.is_empty() {
                    println!(
                        "  uncovered_support_profile_slots: {}",
                        binding.uncovered_support_profile_slots.join(", ")
                    );
                }
                println!(
                    "  registered_units: {}",
                    if binding.registered_units.is_empty() {
                        "<registry-only>".to_owned()
                    } else {
                        binding.registered_units.join(", ")
                    }
                );
                if let Some(bound_unit) = &binding.bound_unit {
                    println!("  bound_unit: {}", bound_unit);
                }
                if !binding.used_units.is_empty() {
                    println!("  used_units: {}", binding.used_units.join(", "));
                }
                if !binding.instantiated_units.is_empty() {
                    println!(
                        "  instantiated_units: {}",
                        binding.instantiated_units.join(", ")
                    );
                }
                if !binding.used_host_ffi_abis.is_empty() {
                    println!(
                        "  used_host_ffi_abis: {}",
                        binding.used_host_ffi_abis.join(", ")
                    );
                }
                if !binding.used_host_ffi_symbols.is_empty() {
                    println!(
                        "  used_host_ffi_symbols: {}",
                        binding.used_host_ffi_symbols.join(", ")
                    );
                }
                println!(
                    "  matched_resources: {}",
                    if binding.matched_resources.is_empty() {
                        "<none>".to_owned()
                    } else {
                        binding.matched_resources.join(", ")
                    }
                );
                println!(
                    "  matched_ops: {}",
                    if binding.matched_ops.is_empty() {
                        "<none>".to_owned()
                    } else {
                        binding.matched_ops.join(", ")
                    }
                );
                if !binding.undeclared_ops.is_empty() {
                    println!("  undeclared_ops: {}", binding.undeclared_ops.join(", "));
                }
            }
        }
        CommandKind::PackNustar { package_id, output } => {
            let manifest = registry::load_manifest(Path::new("nustar-packages"), &package_id)?;
            nustar_binary::validate_manifest_for_packaging(&manifest)?;
            let blob = format!(
                "nustar_impl_stub\npackage={}\nfrontend={}\nentry_crate={}\n",
                manifest.package_id, manifest.frontend, manifest.entry_crate
            )
            .into_bytes();
            let binary = nustar_binary::default_binary(manifest, blob);
            nustar_binary::write_to_path(&output, &binary)?;
            println!("packed nustar binary: {}", output.display());
            println!("  package: {}", binary.manifest.package_id);
            println!("  extension: .nustar");
            println!("  format_version: {}", binary.format_version);
            println!("  abi: {}", binary.abi_tag);
            println!("  machine_arch: {}", binary.machine_arch);
            println!("  machine_os: {}", binary.machine_os);
            println!("  object_format: {}", binary.object_format);
            println!("  calling_abi: {}", binary.calling_abi);
            println!("  format: {}", binary.implementation_format);
            println!("  checksum: {}", binary.implementation_checksum);
            println!(
                "  abi_profiles: {}",
                binary.manifest.abi_profiles.join(", ")
            );
            println!(
                "  abi_capabilities: {}",
                binary.manifest.abi_capabilities.join(", ")
            );
            if !binary.manifest.abi_targets.is_empty() {
                println!("  abi_targets: {}", binary.manifest.abi_targets.join(", "));
            }
            println!("  blob_bytes: {}", binary.implementation_blob.len());
        }
        CommandKind::InspectNustar { input } => {
            let binary = nustar_binary::read_from_path(&input)?;
            let capability = registry::capability_summary(&binary.manifest);
            println!("nustar binary: {}", input.display());
            println!("  package: {}", binary.manifest.package_id);
            println!("  domain: {}", binary.manifest.domain_family);
            println!("  frontend: {}", binary.manifest.frontend);
            println!("  crate: {}", binary.manifest.entry_crate);
            println!("  ast_entry: {}", binary.manifest.ast_entry);
            println!("  nir_entry: {}", binary.manifest.nir_entry);
            println!(
                "  yir_lowering_entry: {}",
                binary.manifest.yir_lowering_entry
            );
            println!("  part_verify_entry: {}", binary.manifest.part_verify_entry);
            println!("  loader_abi: {}", binary.manifest.loader_abi);
            println!("  loader_entry: {}", binary.manifest.loader_entry);
            if !binary.manifest.abi_profiles.is_empty() {
                println!(
                    "  abi_profiles: {}",
                    binary.manifest.abi_profiles.join(", ")
                );
            }
            if !binary.manifest.abi_capabilities.is_empty() {
                println!(
                    "  abi_capabilities: {}",
                    binary.manifest.abi_capabilities.join(", ")
                );
            }
            if !binary.manifest.abi_targets.is_empty() {
                println!("  abi_targets: {}", binary.manifest.abi_targets.join(", "));
            }
            if !binary.manifest.host_ffi_surface.is_empty() {
                println!(
                    "  host_ffi_surface: {}",
                    binary.manifest.host_ffi_surface.join(", ")
                );
                println!(
                    "  host_ffi_abis: {}",
                    binary.manifest.host_ffi_abis.join(", ")
                );
                println!("  host_ffi_bridge: {}", binary.manifest.host_ffi_bridge);
            }
            if !capability.support_surface.is_empty() {
                println!(
                    "  support_surface: {}",
                    capability.support_surface.join(", ")
                );
            }
            if !capability.support_profile_slots.is_empty() {
                println!(
                    "  support_profile_slots: {}",
                    capability.support_profile_slots.join(", ")
                );
            }
            if !capability.default_lanes.is_empty() {
                println!("  default_lanes: {}", capability.default_lanes.join(", "));
            }
            println!("  clock_domain_id: {}", capability.clock.domain_id);
            println!("  clock_kind: {}", capability.clock.kind);
            println!("  clock_epoch_kind: {}", capability.clock.epoch_kind);
            println!("  clock_resolution: {}", capability.clock.resolution);
            println!(
                "  clock_bridge_default: {}",
                capability.clock.bridge_default
            );
            println!("  format_version: {}", binary.format_version);
            println!("  abi: {}", binary.abi_tag);
            println!("  machine_arch: {}", binary.machine_arch);
            println!("  machine_os: {}", binary.machine_os);
            println!("  object_format: {}", binary.object_format);
            println!("  calling_abi: {}", binary.calling_abi);
            println!(
                "  machine_abi_compatible_with_host: {}",
                nustar_binary::machine_abi_matches_host(&binary)
            );
            println!("  format: {}", binary.implementation_format);
            println!("  checksum: {}", binary.implementation_checksum);
            println!("  profiles: {}", binary.manifest.profiles.join(", "));
            println!(
                "  resource_families: {}",
                binary.manifest.resource_families.join(", ")
            );
            println!(
                "  unit_types: {}",
                if binary.manifest.unit_types.is_empty() {
                    "<any>".to_owned()
                } else {
                    binary.manifest.unit_types.join(", ")
                }
            );
            println!(
                "  lowering_targets: {}",
                binary.manifest.lowering_targets.join(", ")
            );
            println!("  ops: {}", binary.manifest.ops.join(", "));
            println!("  blob_bytes: {}", binary.implementation_blob.len());
        }
        CommandKind::LoaderContract { package_id } => {
            let manifest = registry::load_manifest(Path::new("nustar-packages"), &package_id)?;
            let binary = nustar_binary::default_binary(manifest, Vec::new());
            let capability = registry::capability_summary(&binary.manifest);
            println!("loader contract: {}", binary.manifest.package_id);
            println!("  loader_abi: {}", binary.manifest.loader_abi);
            println!("  loader_entry: {}", binary.manifest.loader_entry);
            if !capability.support_surface.is_empty() {
                println!(
                    "  support_surface: {}",
                    capability.support_surface.join(", ")
                );
            }
            if !capability.support_profile_slots.is_empty() {
                println!(
                    "  support_profile_slots: {}",
                    capability.support_profile_slots.join(", ")
                );
            }
            if !capability.default_lanes.is_empty() {
                println!("  default_lanes: {}", capability.default_lanes.join(", "));
            }
            println!("  clock_domain_id: {}", capability.clock.domain_id);
            println!("  clock_kind: {}", capability.clock.kind);
            println!("  clock_epoch_kind: {}", capability.clock.epoch_kind);
            println!("  clock_resolution: {}", capability.clock.resolution);
            println!(
                "  clock_bridge_default: {}",
                capability.clock.bridge_default
            );
            println!(
                "  canonical_entry_signature: {}",
                nustar_binary::CANONICAL_ENTRY_SIGNATURE
            );
            println!(
                "  canonical_host_abi_struct: {}",
                nustar_binary::CANONICAL_HOST_ABI_STRUCT
            );
            println!(
                "  canonical_result_struct: {}",
                nustar_binary::CANONICAL_RESULT_STRUCT
            );
            println!(
                "  loader_status_convention: {}",
                nustar_binary::CANONICAL_LOADER_STATUS_CONVENTION
            );
            println!(
                "  machine_abi_policy: {}",
                binary.manifest.machine_abi_policy
            );
            println!("  host_machine_arch: {}", binary.machine_arch);
            println!("  host_machine_os: {}", binary.machine_os);
            println!("  host_object_format: {}", binary.object_format);
            println!("  host_calling_abi: {}", binary.calling_abi);
            for contract in nustar_binary::implementation_contracts(&binary) {
                println!("  kind: {}", contract.kind);
                println!("    loader_abi: {}", contract.loader_abi);
                println!("    entry_symbol: {}", contract.entry_symbol);
                println!("    entry_signature: {}", contract.entry_signature);
                println!("    host_abi_struct: {}", contract.host_abi_struct);
                println!("    result_struct: {}", contract.result_struct);
                println!("    status_convention: {}", contract.status_convention);
                println!("    artifact_container: {}", contract.artifact_container);
                println!(
                    "    implementation_section: {}",
                    contract.implementation_section
                );
                println!(
                    "    required_exports: {}",
                    contract.required_exports.join(", ")
                );
                println!(
                    "    required_metadata: {}",
                    contract.required_metadata.join(", ")
                );
                println!("    link_mode: {}", contract.link_mode);
                println!("    machine_abi_policy: {}", contract.machine_abi_policy);
                println!("    notes: {}", contract.notes);
            }
        }
        CommandKind::PackEnvelope { input, output } => {
            let envelope = load_nuis_executable_envelope(&input)?;
            let encoded = aot::encode_nuis_executable_envelope_binary(&envelope)?;
            std::fs::write(&output, encoded)
                .map_err(|error| format!("failed to write `{}`: {error}", output.display()))?;
            println!("packed nuis envelope: {}", output.display());
            println!("  source: {}", input.display());
            println!("  schema: {}", envelope.schema);
            println!("  executable_kind: {}", envelope.executable_kind);
            println!("  package_count: {}", envelope.package_count);
        }
        CommandKind::UnpackEnvelope { input, output } => {
            let envelope = load_nuis_executable_envelope(&input)?;
            aot::write_nuis_executable_envelope(&output, &envelope)?;
            println!("unpacked nuis envelope: {}", output.display());
            println!("  source: {}", input.display());
            println!("  schema: {}", envelope.schema);
            println!("  executable_kind: {}", envelope.executable_kind);
            println!("  package_count: {}", envelope.package_count);
        }
        CommandKind::InspectEnvelope { input } => {
            let envelope = load_nuis_executable_envelope(&input)?;
            println!("nuis envelope: {}", input.display());
            println!("  schema: {}", envelope.schema);
            println!("  executable_kind: {}", envelope.executable_kind);
            println!("  package_count: {}", envelope.package_count);
            println!("  domain_families: {}", envelope.domain_families.join(", "));
            println!(
                "  contract_families: {}",
                envelope.contract_families.join(", ")
            );
            println!("  function_kind: {}", envelope.function_kind);
            println!("  graph_kind: {}", envelope.graph_kind);
            println!("  default_time_mode: {}", envelope.default_time_mode);
        }
        CommandKind::InspectArtifact { input, json } => {
            let artifact = load_nuis_compiled_artifact(&input)?;
            if json {
                println!("{}", inspect_artifact_json(&input, &artifact));
                return Ok(());
            }
            println!("nuis artifact: {}", input.display());
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
        }
        CommandKind::ArtifactReport { input, json } => {
            let is_manifest_input = input
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name == "nuis.build.manifest.toml")
                .unwrap_or(false);
            let artifact = load_nuis_compiled_artifact(&input)?;
            let artifact_verify_input = if is_manifest_input {
                let manifest_report = aot::verify_build_manifest(&input)?;
                PathBuf::from(manifest_report.artifact_path)
            } else {
                input.clone()
            };
            let artifact_verify = aot::verify_nuis_compiled_artifact(&artifact_verify_input)?;
            let (manifest_input, manifest_verify, manifest_verify_reconstructed) =
                if is_manifest_input {
                    let report = aot::verify_build_manifest(&input)?;
                    (input.clone(), report, false)
                } else {
                    let (manifest_input, manifest_verify) =
                        reconstruct_manifest_report_from_artifact(&input, &artifact)?;
                    (manifest_input, manifest_verify, true)
                };
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
            println!(
                "  execution_contracts_checked: {}",
                manifest_verify.execution_contracts_checked
            );
            println!(
                "  lifecycle_runtime_capability_flags: {}",
                manifest_verify
                    .lifecycle_runtime_capability_flags
                    .join(", ")
            );
        }
        CommandKind::VerifyArtifact { input, json } => {
            let report = aot::verify_nuis_compiled_artifact(&input)?;
            if json {
                println!("{}", verify_artifact_json(&input, &report));
                return Ok(());
            }
            println!("nuis artifact verified: {}", input.display());
            println!("  schema: {}", report.schema);
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
            let report = aot::verify_build_manifest(&manifest)?;
            if json {
                println!("{}", verify_build_manifest_json(&manifest, &report));
                return Ok(());
            }
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
            println!("yir_nodes: {}", artifacts.yir.nodes.len());
            println!("yir_edges: {}", artifacts.yir.edges.len());
            println!("llvm_ir_bytes: {}", artifacts.llvm_ir.len());
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
            let artifacts = resolved.compile_with_options(&pipeline::PipelineCompileOptions {
                lowering_target: Some(lowering::LoweringTargetConfig::from_cpu_build_target(
                    &cpu_target,
                )),
            })?;
            let written = if let Some(entry) = &cache_hit {
                cache::restore_compile_cache(entry, &output_dir)?;
                aot::compile_artifacts_for_output_dir(
                    &resolved.effective_input_path,
                    &output_dir,
                    &artifacts.yir,
                )?
            } else {
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
                written
            };
            let project_metadata =
                if let (Some(project), Some(plan)) = (&resolved.project, &resolved.project_plan) {
                    Some(project::write_project_metadata(&output_dir, project, plan)?)
                } else {
                    None
                };
            let build_manifest = aot::write_build_manifest(
                &output_dir,
                &written,
                &aot::BuildManifestContext {
                    input_path: input.display().to_string(),
                    output_dir: output_dir.display().to_string(),
                    loaded_nustar: artifacts.loaded_nustar.clone(),
                    compile_cache: Some(aot::BuildManifestCacheInfo {
                        status: if cache_hit.is_some() {
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
                    cpu_target: cpu_target.clone(),
                },
            )?;
            println!("compiled nuis source: {}", input.display());
            println!(
                "compile_cache: {} ({})",
                if cache_hit.is_some() { "hit" } else { "miss" },
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
                artifacts
                    .loaded_nustar
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
                        if let Ok(target) = registry::registered_abi_target(&manifest, &item.abi) {
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
            println!("build_manifest: {}", build_manifest);
            if let Some(metadata) = &project_metadata {
                println!("project_manifest: {}", metadata.manifest_copy_path);
                println!("project_plan_index: {}", metadata.plan_index_path);
                println!("project_organization: {}", metadata.organization_index_path);
                println!("project_exchange: {}", metadata.exchange_index_path);
                println!("project_modules: {}", metadata.modules_index_path);
                println!("project_links: {}", metadata.links_index_path);
                println!("project_packet: {}", metadata.packet_index_path);
                println!("project_host_ffi: {}", metadata.host_ffi_index_path);
                println!("project_abi: {}", metadata.abi_index_path);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_contract_json_exposes_grouped_contract_sections() {
        let contract =
            registry::load_domain_contract_for_domain(Path::new(NUSTAR_REGISTRY_ROOT), "network")
                .expect("expected network domain contract");
        let json = registry::domain_contract_json(&contract);

        assert!(json.contains("\"contract_schema\":\"nustar-domain-contract-v1\""));
        assert!(json.contains("\"contract\":{"));
        assert!(json.contains("\"schema\":\"nustar-domain-contract-v1\""));
        assert!(json.contains("\"groups\":[\"package_identity\""));
        assert!(json.contains("\"package_identity\":{"));
        assert!(json.contains("\"loader_contract\":{"));
        assert!(json.contains("\"abi_contract\":{"));
        assert!(json.contains("\"host_bridge_contract\":{"));
        assert!(json.contains("\"runtime_capability_contract\":{"));
        assert!(json.contains("\"scheduler_contract\":{"));
        assert!(json.contains("\"std_net_extension\":{"));
        assert!(json.contains("\"domain\":\"network\""));
    }

    #[test]
    fn domain_registration_json_exposes_registration_section() {
        let registration = registry::load_registered_domains(Path::new(NUSTAR_REGISTRY_ROOT))
            .expect("expected registered domains")
            .into_iter()
            .find(|item| item.domain_family == "network")
            .expect("expected network registration");
        let json = registry::domain_registration_json(&registration);

        assert!(json.contains("\"registration\":{"));
        assert!(json.contains("\"manifest_path\":"));
        assert!(json.contains("\"entry_crate\":"));
        assert!(json.contains("\"ast_entry\":"));
        assert!(json.contains("\"nir_entry\":"));
        assert!(json.contains("\"yir_lowering_entry\":"));
        assert!(json.contains("\"part_verify_entry\":"));
        assert!(json.contains("\"ast_surface\":["));
        assert!(json.contains("\"nir_surface\":["));
        assert!(json.contains("\"ops\":["));
    }
}
