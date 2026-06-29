use std::path::{Path, PathBuf};

use crate::command_helpers::{compile_command_input, NUSTAR_REGISTRY_ROOT};
use crate::{nustar_binary, project, registry};

pub(crate) fn run_bindings(input: PathBuf) -> Result<(), String> {
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
        Path::new(NUSTAR_REGISTRY_ROOT),
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
        if !binding.capability_tags.is_empty() {
            println!("  capability_tags: {}", binding.capability_tags.join(", "));
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
    Ok(())
}

pub(crate) fn run_pack_nustar(package_id: String, output: PathBuf) -> Result<(), String> {
    let manifest = registry::load_manifest(Path::new(NUSTAR_REGISTRY_ROOT), &package_id)?;
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
    Ok(())
}

pub(crate) fn run_inspect_nustar(input: PathBuf) -> Result<(), String> {
    let binary = nustar_binary::read_from_path(&input)?;
    let capability = registry::capability_summary(&binary.manifest);
    println!("nustar binary: {}", input.display());
    print_manifest_surface(&binary.manifest);
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
    Ok(())
}

pub(crate) fn run_loader_contract(package_id: String) -> Result<(), String> {
    let manifest = registry::load_manifest(Path::new(NUSTAR_REGISTRY_ROOT), &package_id)?;
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
    Ok(())
}

fn print_manifest_surface(manifest: &registry::NustarPackageManifest) {
    println!("  package: {}", manifest.package_id);
    println!("  domain: {}", manifest.domain_family);
    println!("  frontend: {}", manifest.frontend);
    println!("  crate: {}", manifest.entry_crate);
    println!("  ast_entry: {}", manifest.ast_entry);
    println!("  nir_entry: {}", manifest.nir_entry);
    println!("  yir_lowering_entry: {}", manifest.yir_lowering_entry);
    println!("  part_verify_entry: {}", manifest.part_verify_entry);
    println!("  loader_abi: {}", manifest.loader_abi);
    println!("  loader_entry: {}", manifest.loader_entry);
    if !manifest.abi_profiles.is_empty() {
        println!("  abi_profiles: {}", manifest.abi_profiles.join(", "));
    }
    if !manifest.abi_capabilities.is_empty() {
        println!(
            "  abi_capabilities: {}",
            manifest.abi_capabilities.join(", ")
        );
    }
    if !manifest.abi_targets.is_empty() {
        println!("  abi_targets: {}", manifest.abi_targets.join(", "));
    }
    if !manifest.host_ffi_surface.is_empty() {
        println!(
            "  host_ffi_surface: {}",
            manifest.host_ffi_surface.join(", ")
        );
        println!("  host_ffi_abis: {}", manifest.host_ffi_abis.join(", "));
        println!("  host_ffi_bridge: {}", manifest.host_ffi_bridge);
    }
}
