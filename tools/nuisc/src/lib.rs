pub mod aot;
pub mod cli;
pub mod codegen_wasm;
pub mod engine;
pub mod errors;
pub mod fmt;
pub mod frontend;
pub mod lowering;
pub mod nir_verify;
pub mod nustar_binary;
pub mod pipeline;
pub mod project;
pub mod registry;
pub mod render;

use std::path::Path;

pub use cli::CommandKind;

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
        CommandKind::Registry => {
            let manifests = registry::load_all_manifests(Path::new("nustar-packages"))?;
            if manifests.is_empty() {
                let placeholder_error = errors::NuiscError {
                    message: "no nustar packages discovered",
                };
                return Err(placeholder_error.message.to_owned());
            }
            for manifest in manifests {
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
                if !manifest.support_surface.is_empty() {
                    println!("  support_surface: {}", manifest.support_surface.join(", "));
                }
                if !manifest.support_profile_slots.is_empty() {
                    println!(
                        "  support_profile_slots: {}",
                        manifest.support_profile_slots.join(", ")
                    );
                }
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
            let project = if project::is_project_input(&input) {
                Some(project::load_project(&input)?)
            } else {
                None
            };
            let artifacts = pipeline::compile_source_path(&input)?;
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
            if let Some(project) = &project {
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
            println!("  blob_bytes: {}", binary.implementation_blob.len());
        }
        CommandKind::InspectNustar { input } => {
            let binary = nustar_binary::read_from_path(&input)?;
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
            if !binary.manifest.support_surface.is_empty() {
                println!(
                    "  support_surface: {}",
                    binary.manifest.support_surface.join(", ")
                );
            }
            if !binary.manifest.support_profile_slots.is_empty() {
                println!(
                    "  support_profile_slots: {}",
                    binary.manifest.support_profile_slots.join(", ")
                );
            }
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
            println!("loader contract: {}", binary.manifest.package_id);
            println!("  loader_abi: {}", binary.manifest.loader_abi);
            println!("  loader_entry: {}", binary.manifest.loader_entry);
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
        CommandKind::VerifyBuildManifest { manifest } => {
            let report = aot::verify_build_manifest(&manifest)?;
            println!("build manifest verified: {}", manifest.display());
            println!("  schema: {}", report.schema);
            println!("  input: {}", report.input);
            println!("  output_dir: {}", report.output_dir);
            println!("  packaging_mode: {}", report.packaging_mode);
            println!("  artifacts_checked: {}", report.artifacts_checked);
        }
        CommandKind::DumpAst { input } => {
            if project::is_project_input(&input) {
                let project = project::load_project(&input)?;
                eprintln!("nuisc: {}", project::describe_project(&project));
            }
            let artifacts = pipeline::compile_source_path(&input)?;
            print!("{}", render::render_ast(&artifacts.ast));
        }
        CommandKind::DumpNir { input } => {
            if project::is_project_input(&input) {
                let project = project::load_project(&input)?;
                eprintln!("nuisc: {}", project::describe_project(&project));
            }
            let artifacts = pipeline::compile_source_path(&input)?;
            let required =
                registry::load_required_manifests(Path::new("nustar-packages"), &artifacts.yir)?;
            registry::validate_unit_binding(&required, &artifacts.ast.domain, &artifacts.ast.unit)?;
            eprintln!(
                "nuisc: lazily loaded nustar = {}",
                required
                    .iter()
                    .map(|manifest| manifest.package_id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            print!("{}", render::render_nir(&artifacts.nir));
        }
        CommandKind::DumpYir { input } => {
            if project::is_project_input(&input) {
                let project = project::load_project(&input)?;
                eprintln!("nuisc: {}", project::describe_project(&project));
            }
            let artifacts = pipeline::compile_source_path(&input)?;
            let required =
                registry::load_required_manifests(Path::new("nustar-packages"), &artifacts.yir)?;
            registry::validate_unit_binding(&required, &artifacts.ast.domain, &artifacts.ast.unit)?;
            eprintln!(
                "nuisc: lazily loaded nustar = {}",
                required
                    .iter()
                    .map(|manifest| manifest.package_id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            print!("{}", render::render_yir(&artifacts.yir));
        }
        CommandKind::Check { input } => {
            let project = if project::is_project_input(&input) {
                Some(project::load_project(&input)?)
            } else {
                None
            };
            let artifacts = pipeline::compile_source_path(&input)?;
            println!("checked nuis source: {}", input.display());
            if let Some(project) = &project {
                println!("project: {}", project::describe_project(project));
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
        CommandKind::Compile { input, output_dir } => {
            let project = if project::is_project_input(&input) {
                Some(project::load_project(&input)?)
            } else {
                None
            };
            let effective_input = if let Some(project) = &project {
                project.root.join(format!("{}.ns", project.manifest.name))
            } else {
                input.clone()
            };
            let artifacts = pipeline::compile_source_path(&input)?;
            let written = aot::write_and_link(
                &effective_input,
                &output_dir,
                &artifacts.ast,
                &artifacts.nir,
                &artifacts.yir,
                &artifacts.llvm_ir,
            )?;
            let project_metadata = if let Some(project) = &project {
                Some(project::write_project_metadata(&output_dir, project)?)
            } else {
                None
            };
            let project_abi_resolution = if let Some(project) = &project {
                Some(project::resolve_project_abi(project)?)
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
                    project: project
                        .as_ref()
                        .map(|project| aot::BuildManifestProjectInfo {
                            name: project.manifest.name.clone(),
                            abi_mode: project_abi_resolution
                                .as_ref()
                                .map(|resolution| {
                                    if resolution.explicit {
                                        "explicit".to_owned()
                                    } else {
                                        "auto-recommended".to_owned()
                                    }
                                })
                                .unwrap_or_else(|| "none".to_owned()),
                            abi_entries: project_abi_resolution
                                .as_ref()
                                .map(|resolution| {
                                    resolution
                                        .requirements
                                        .iter()
                                        .map(|item| (item.domain.clone(), item.abi.clone()))
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_default(),
                            manifest_copy_path: project_metadata
                                .as_ref()
                                .map(|item| item.manifest_copy_path.clone()),
                            modules_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.modules_index_path.clone()),
                            links_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.links_index_path.clone()),
                            host_ffi_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.host_ffi_index_path.clone()),
                            abi_index_path: project_metadata
                                .as_ref()
                                .map(|item| item.abi_index_path.clone()),
                        }),
                },
            )?;
            println!("compiled nuis source: {}", input.display());
            if let Some(project) = &project {
                println!("project: {}", project::describe_project(project));
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
            println!("ast: {}", written.ast_path);
            println!("nir: {}", written.nir_path);
            println!("yir: {}", written.yir_path);
            println!("llvm_ir: {}", written.llvm_ir_path);
            println!("packaging_mode: {}", written.packaging_mode);
            println!("binary: {}", written.binary_path);
            println!("build_manifest: {}", build_manifest);
            if let Some(metadata) = &project_metadata {
                println!("project_manifest: {}", metadata.manifest_copy_path);
                println!("project_modules: {}", metadata.modules_index_path);
                println!("project_links: {}", metadata.links_index_path);
                println!("project_host_ffi: {}", metadata.host_ffi_index_path);
                println!("project_abi: {}", metadata.abi_index_path);
            }
        }
    }

    Ok(())
}
