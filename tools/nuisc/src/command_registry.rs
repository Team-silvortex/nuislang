use std::path::Path;

use crate::artifact_report::domain_registry_json;
use crate::json_report::json_bool_field;
use crate::{codegen_wasm, engine, errors, frontend, registry};

pub(crate) fn run_status() -> Result<(), String> {
    let frontend = frontend::frontend_name();
    let backend = codegen_wasm::backend_name();
    let engine = engine::default_engine();
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
    Ok(())
}

pub(crate) fn run_registry(json: bool) -> Result<(), String> {
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
            .map(|registration| {
                let manifest = registry::load_manifest_for_domain(
                    Path::new("nustar-packages"),
                    &registration.domain_family,
                )?;
                Ok(domain_registry_json(registration, &manifest))
            })
            .collect::<Result<Vec<_>, String>>()?;
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
        let build_contract = registry::domain_build_contract_summary(&manifest);
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
        print_build_contract(&build_contract);
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
    Ok(())
}

fn print_build_contract(build_contract: &registry::NustarDomainBuildContractSummary) {
    println!(
        "  build_lowering_lane_policy: {}",
        build_contract.lowering.lane_policy
    );
    println!(
        "  build_lowering_bridge_surface: {}",
        build_contract.lowering.bridge_surface
    );
    println!(
        "  build_lowering_emission_kind: {}",
        build_contract.lowering.emission_kind
    );
    println!(
        "  build_backend_stub_kind: {}",
        build_contract.backend.stub_kind
    );
    println!(
        "  build_backend_bridge_entry: {}",
        build_contract.backend.bridge_entry
    );
    println!(
        "  build_backend_submission_mode: {}",
        build_contract.backend.submission_mode
    );
    println!(
        "  build_backend_wake_policy: {}",
        build_contract.backend.wake_policy
    );
    println!(
        "  build_backend_scheduler_binding: {}",
        build_contract.backend.scheduler_binding
    );
    if let Some(phase_bind) = build_contract.backend.phase_bind.as_deref() {
        println!("  build_backend_phase_bind: {}", phase_bind);
    }
    if let Some(phase_submit) = build_contract.backend.phase_submit.as_deref() {
        println!("  build_backend_phase_submit: {}", phase_submit);
    }
    if let Some(phase_wait) = build_contract.backend.phase_wait.as_deref() {
        println!("  build_backend_phase_wait: {}", phase_wait);
    }
    if let Some(phase_finalize) = build_contract.backend.phase_finalize.as_deref() {
        println!("  build_backend_phase_finalize: {}", phase_finalize);
    }
    if let Some(transport_model) = build_contract.backend.transport_model.as_deref() {
        println!("  build_backend_transport_model: {}", transport_model);
    }
    if let Some(request_shape) = build_contract.backend.request_shape.as_deref() {
        println!("  build_backend_request_shape: {}", request_shape);
    }
    if let Some(response_shape) = build_contract.backend.response_shape.as_deref() {
        println!("  build_backend_response_shape: {}", response_shape);
    }
    if let Some(dispatch_shape) = build_contract.backend.dispatch_shape.as_deref() {
        println!("  build_backend_dispatch_shape: {}", dispatch_shape);
    }
    if let Some(memory_binding) = build_contract.backend.memory_binding.as_deref() {
        println!("  build_backend_memory_binding: {}", memory_binding);
    }
    if let Some(resource_binding) = build_contract.backend.resource_binding.as_deref() {
        println!("  build_backend_resource_binding: {}", resource_binding);
    }
    if let Some(completion_model) = build_contract.backend.completion_model.as_deref() {
        println!("  build_backend_completion_model: {}", completion_model);
    }
    println!(
        "  build_bridge_surface: {}",
        build_contract.bridge.bridge_surface
    );
    println!(
        "  build_bridge_entry: {}",
        build_contract.bridge.bridge_entry
    );
    println!(
        "  build_bridge_scheduler_binding: {}",
        build_contract.bridge.scheduler_binding
    );
    println!(
        "  build_bridge_phase_bind: {}",
        build_contract.bridge.phase_bind
    );
    println!(
        "  build_bridge_phase_submit: {}",
        build_contract.bridge.phase_submit
    );
    println!(
        "  build_bridge_phase_wait: {}",
        build_contract.bridge.phase_wait
    );
    println!(
        "  build_bridge_phase_finalize: {}",
        build_contract.bridge.phase_finalize
    );
    println!("  build_bridge_kind: {}", build_contract.bridge.bridge_kind);
    println!(
        "  host_bridge_host_ffi_surface: {}",
        build_contract.host_bridge.host_ffi_surface
    );
    println!(
        "  host_bridge_handle_family: {}",
        build_contract.host_bridge.handle_family
    );
    println!(
        "  host_bridge_phase_order: {}",
        build_contract.host_bridge.phase_order.join(", ")
    );
    println!(
        "  host_bridge_phase_bind_inputs: {}",
        build_contract.host_bridge.phase_bind_inputs.join(", ")
    );
    println!(
        "  host_bridge_phase_bind_outputs: {}",
        build_contract.host_bridge.phase_bind_outputs.join(", ")
    );
    println!(
        "  host_bridge_phase_submit_inputs: {}",
        build_contract.host_bridge.phase_submit_inputs.join(", ")
    );
    println!(
        "  host_bridge_phase_submit_outputs: {}",
        build_contract.host_bridge.phase_submit_outputs.join(", ")
    );
    println!(
        "  host_bridge_phase_wait_inputs: {}",
        build_contract.host_bridge.phase_wait_inputs.join(", ")
    );
    println!(
        "  host_bridge_phase_wait_outputs: {}",
        build_contract.host_bridge.phase_wait_outputs.join(", ")
    );
    println!(
        "  host_bridge_phase_finalize_inputs: {}",
        build_contract.host_bridge.phase_finalize_inputs.join(", ")
    );
    println!(
        "  host_bridge_phase_finalize_outputs: {}",
        build_contract.host_bridge.phase_finalize_outputs.join(", ")
    );
    println!(
        "  host_bridge_phase_bind_wake: {}",
        build_contract.host_bridge.phase_bind_wake
    );
    println!(
        "  host_bridge_phase_submit_wake: {}",
        build_contract.host_bridge.phase_submit_wake
    );
    println!(
        "  host_bridge_phase_wait_wake: {}",
        build_contract.host_bridge.phase_wait_wake
    );
    println!(
        "  host_bridge_phase_finalize_wake: {}",
        build_contract.host_bridge.phase_finalize_wake
    );
    println!(
        "  host_bridge_plan_begin: {}",
        build_contract.host_bridge.bridge_plan_begin
    );
    println!(
        "  host_bridge_plan_end: {}",
        build_contract.host_bridge.bridge_plan_end
    );
}
