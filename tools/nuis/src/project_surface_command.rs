use crate::{
    galaxy, hidden_manual_only_library_modules_for_project, print_workflow_frontdoor_surface,
    project_frontdoor_surface, single_source_frontdoor_surface, surface_render,
};
use std::path::{Path, PathBuf};

pub(crate) fn handle_scheduler_view(input: PathBuf, json: bool) -> Result<(), String> {
    if json {
        return handle_scheduler_view_json(input);
    }
    println!("scheduler view: {}", input.display());
    if nuisc::project::is_project_input(&input) {
        let project = nuisc::project::load_project(&input)?;
        let plan = nuisc::project::build_project_compilation_plan(&project)?;
        let declared_tests = project
            .manifest
            .tests
            .iter()
            .map(|relative| project.root.join(relative))
            .collect::<Vec<_>>();
        let missing_tests = declared_tests
            .iter()
            .filter(|path| !path.exists())
            .cloned()
            .collect::<Vec<_>>();
        let galaxy_manifest_path = project.root.join("galaxy.toml");
        let galaxy_check = if galaxy_manifest_path.exists() {
            Some(galaxy::check(&project.root))
        } else {
            None
        };
        let galaxy_check_invalid = matches!(galaxy_check.as_ref(), Some(Err(_)));
        let galaxy_doctor = galaxy::doctor_project(&project.root)?;
        let hidden_manual_only_library_modules =
            hidden_manual_only_library_modules_for_project(&project);
        let frontdoor = project_frontdoor_surface(
            &plan,
            &declared_tests,
            &missing_tests,
            &galaxy_doctor,
            galaxy_check_invalid,
            !hidden_manual_only_library_modules.is_empty(),
        );
        println!("  source_kind: project");
        println!("  project: {}", project.manifest.name);
        print_workflow_frontdoor_surface(&frontdoor);
        println!(
            "  recommended_next_step: {}",
            frontdoor.recommended_next_step
        );
        println!("  recommended_command: {}", frontdoor.recommended_command);
        println!("  recommended_reason: {}", frontdoor.recommended_reason);
        println!(
            "  project_plan: {}",
            nuisc::project::describe_project_compilation_plan(&plan)
        );
        println!(
            "  synthetic_input: {} ({})",
            plan.synthetic_input.path.display(),
            plan.synthetic_input.kind
        );
        println!("  output_intents: {}", plan.output_intents.len());
        println!(
            "  output_intent_categories: {}",
            nuisc::project::describe_project_output_intent_categories(&plan)
        );
        println!(
            "  abi_mode: {}",
            if plan.abi_resolution.explicit {
                "explicit"
            } else {
                "auto-recommended"
            }
        );
        println!(
            "  resolved_domains: {}",
            plan.abi_resolution.requirements.len()
        );
        for item in nuisc::project::project_abi_selection_views(&plan.abi_resolution) {
            let domain = item.domain.clone();
            println!("  domain: {}", item.domain);
            for line in nuisc::project::render_project_abi_selection_view_lines(&item) {
                if let Some(detail) = line.strip_prefix("abi: ") {
                    println!(
                        "    abi: {}",
                        detail.split_once('=').map(|(_, abi)| abi).unwrap_or(detail)
                    );
                } else {
                    println!("    {}", line.trim_start());
                }
            }
            print_project_scheduler_contract_view(&domain)?;
        }
        return Ok(());
    }

    let artifacts = nuisc::pipeline::compile_source_path(&input)?;
    let manifests = nuisc::registry::load_required_manifests(
        std::path::Path::new("nustar-packages"),
        &artifacts.yir,
    )?;
    let frontdoor = single_source_frontdoor_surface();
    println!("  source_kind: single-file");
    println!("  ast_domain: {}", artifacts.ast.domain);
    println!("  ast_unit: {}", artifacts.ast.unit);
    print_workflow_frontdoor_surface(&frontdoor);
    println!(
        "  recommended_next_step: {}",
        frontdoor.recommended_next_step
    );
    println!("  recommended_command: {}", frontdoor.recommended_command);
    println!("  recommended_reason: {}", frontdoor.recommended_reason);
    println!("  resolved_domains: {}", manifests.len());
    for manifest in manifests {
        println!("  domain: {}", manifest.domain_family);
        println!("    package: {}", manifest.package_id);
        print_project_scheduler_contract_view(&manifest.domain_family)?;
    }
    Ok(())
}

fn handle_scheduler_view_json(input: PathBuf) -> Result<(), String> {
    println!("{}", render_scheduler_view_json(&input)?);
    Ok(())
}

pub(crate) fn render_scheduler_view_json(input: &Path) -> Result<String, String> {
    surface_render::render_scheduler_view_json(input)
}

pub(crate) fn handle_project_status(input: PathBuf, json: bool) -> Result<(), String> {
    if json {
        return handle_project_status_json(input);
    }
    let mut rendered = String::new();
    surface_render::write_project_status_text_summary(&mut rendered, &input)?;
    print!("{rendered}");
    let project = nuisc::project::load_project(&input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    for item in nuisc::project::project_abi_selection_views(&plan.abi_resolution) {
        let domain = item.domain.clone();
        for line in nuisc::project::render_project_abi_selection_view_lines(&item) {
            if let Some(detail) = line.strip_prefix("abi: ") {
                println!("  abi: {}", detail);
            } else {
                println!("    {}", line.trim_start());
            }
        }
        print_project_scheduler_contract_view(&domain)?;
    }
    Ok(())
}

fn handle_project_status_json(input: PathBuf) -> Result<(), String> {
    println!("{}", render_project_status_json(&input)?);
    Ok(())
}

pub(crate) fn render_project_status_json(input: &Path) -> Result<String, String> {
    surface_render::render_project_status_json(input)
}

pub(crate) fn handle_project_doctor(input: PathBuf, json: bool) -> Result<(), String> {
    if json {
        return handle_project_doctor_json(input);
    }
    let mut rendered = String::new();
    surface_render::write_project_doctor_text_summary(&mut rendered, &input)?;
    print!("{rendered}");
    let project = nuisc::project::load_project(&input)?;
    let plan = nuisc::project::build_project_compilation_plan(&project)?;
    let nova_profile = galaxy::inspect_ns_nova_profile(&project.root)?;
    let abi_checks =
        nuisc::project::validate_project_abi_selections(&project, &plan.abi_resolution)?;
    let registry_checks = nuisc::registry::validate_project_domain_registry(&plan);
    let lowering_checks =
        nuisc::project::validate_project_lowering_selections(&plan.abi_resolution);
    for check in &abi_checks {
        let mut rendered = String::new();
        nuisc::project::write_project_abi_selection_check_lines(&mut rendered, check)
            .expect("writing project abi selection check lines should not fail");
        for line in rendered.lines() {
            println!("  {}", line);
        }
    }
    for check in &registry_checks {
        let mut rendered = String::new();
        nuisc::registry::write_project_domain_registry_check_lines(&mut rendered, check)
            .expect("writing project domain registry check lines should not fail");
        for line in rendered.lines() {
            println!("  {}", line);
        }
    }
    for check in &lowering_checks {
        for line in nuisc::project::render_project_lowering_selection_lines(check) {
            println!("  {}", line);
        }
    }
    for item in &plan.abi_resolution.requirements {
        println!("  abi: {}={}", item.domain, item.abi);
        print_project_scheduler_contract_view(&item.domain)?;
    }
    if let Some(profile) = nova_profile.as_ref() {
        println!(
            "  ns_nova_stdlib_schema: {}",
            profile.stdlib_schema.as_deref().unwrap_or("<none>")
        );
        println!(
            "  ns_nova_stdlib_manifest_ref: {}",
            profile.stdlib_manifest.as_deref().unwrap_or("<none>")
        );
        println!(
            "  ns_nova_stdlib_declared_sources: {}",
            profile.stdlib_sources.len()
        );
        println!(
            "  ns_nova_family_schema: {}",
            profile.family_schema.as_deref().unwrap_or("<none>")
        );
        println!(
            "  ns_nova_family_layers: {}",
            if profile.family_layers.is_empty() {
                "<none>".to_owned()
            } else {
                profile.family_layers.join(", ")
            }
        );
        println!(
            "  ns_nova_render_schema: {}",
            profile.render_schema.as_deref().unwrap_or("<none>")
        );
        println!(
            "  ns_nova_render_units: owner={} bridge={} surface={}",
            profile.render_owner_unit.as_deref().unwrap_or("<none>"),
            profile.render_bridge_unit.as_deref().unwrap_or("<none>"),
            profile.render_surface_unit.as_deref().unwrap_or("<none>")
        );
        println!(
            "  ns_nova_selection_schema: {}",
            profile.selection_schema.as_deref().unwrap_or("<none>")
        );
        println!(
            "  ns_nova_selection_units: owner={} bridge={} render={}",
            profile.selection_owner_unit.as_deref().unwrap_or("<none>"),
            profile.selection_bridge_unit.as_deref().unwrap_or("<none>"),
            profile.selection_render_unit.as_deref().unwrap_or("<none>")
        );
        println!(
            "  ns_nova_selection_controls: {}",
            if profile.selection_controls.is_empty() {
                "<none>".to_owned()
            } else {
                profile.selection_controls.join(", ")
            }
        );
    }

    Ok(())
}

fn handle_project_doctor_json(input: PathBuf) -> Result<(), String> {
    println!("{}", render_project_doctor_json(&input)?);
    Ok(())
}

pub(crate) fn render_project_doctor_json(input: &Path) -> Result<String, String> {
    surface_render::render_project_doctor_json(input)
}

fn print_domain_contract_completeness(contract: &nuisc::registry::NustarDomainContract) {
    println!("contract_status: {}", contract.contract_status);
    print_scheduler_sample_field(
        "required_contract_groups",
        &contract.required_contract_groups.join("; "),
    );
    if contract.missing_contract_groups.is_empty() {
        println!("missing_contract_groups: <none>");
    } else {
        print_scheduler_sample_field(
            "missing_contract_groups",
            &contract.missing_contract_groups.join("; "),
        );
    }
}

fn print_domain_contract_group(contract: &nuisc::registry::NustarDomainContract, group: &str) {
    println!("    {}:", group);
    match group {
        nuisc::registry::NUSTAR_DOMAIN_CONTRACT_GROUP_PACKAGE_IDENTITY => {
            println!("      package: {}", contract.package_id);
            println!("      contract_schema: {}", contract.contract_schema);
            println!("      frontend: {}", contract.frontend);
        }
        nuisc::registry::NUSTAR_DOMAIN_CONTRACT_GROUP_LOADER => {
            println!("      loader_abi: {}", contract.loader_abi);
            println!("      loader_entry: {}", contract.loader_entry);
        }
        nuisc::registry::NUSTAR_DOMAIN_CONTRACT_GROUP_ABI => {
            println!("      machine_abi_policy: {}", contract.machine_abi_policy);
            if !contract.abi_profiles.is_empty() {
                print_scheduler_sample_field(
                    "      abi_profiles",
                    &contract.abi_profiles.join("; "),
                );
            }
        }
        nuisc::registry::NUSTAR_DOMAIN_CONTRACT_GROUP_HOST_BRIDGE => {
            if !contract.host_ffi_surface.is_empty() {
                print_scheduler_sample_field(
                    "      host_ffi_surface",
                    &contract.host_ffi_surface.join("; "),
                );
                print_scheduler_sample_field(
                    "      host_ffi_abis",
                    &contract.host_ffi_abis.join("; "),
                );
            }
            if let Some(host_ffi_bridge) = contract.host_ffi_bridge.as_deref() {
                println!("      host_ffi_bridge: {}", host_ffi_bridge);
            }
        }
        nuisc::registry::NUSTAR_DOMAIN_CONTRACT_GROUP_RUNTIME => {
            if !contract.capability.support_surface.is_empty() {
                print_scheduler_sample_field(
                    "      support_surface",
                    &contract.capability.support_surface.join("; "),
                );
            }
            if !contract.capability.support_profile_slots.is_empty() {
                print_scheduler_sample_field(
                    "      support_profile_slots",
                    &contract.capability.support_profile_slots.join("; "),
                );
            }
            if !contract.capability.capability_tags.is_empty() {
                print_scheduler_sample_field(
                    "      capability_tags",
                    &contract.capability.capability_tags.join("; "),
                );
            }
            if !contract.capability.default_lanes.is_empty() {
                print_scheduler_sample_field(
                    "      default_lanes",
                    &contract.capability.default_lanes.join("; "),
                );
            }
            println!(
                "      scheduler_clock: {}",
                contract.scheduler.clock.brief()
            );
        }
        nuisc::registry::NUSTAR_DOMAIN_CONTRACT_GROUP_SCHEDULER => {
            println!(
                "      scheduler_contract_stack: {}",
                contract.scheduler.contract_stack
            );
            println!(
                "      scheduler_result_roles: {}",
                contract.scheduler.result_roles
            );
            println!(
                "      scheduler_summary_api: {}",
                contract.scheduler.summary_api
            );
            println!(
                "      scheduler_observer_classes: {}",
                contract.scheduler.observer_classes
            );
            if let Some(navigation) = contract.scheduler.sample_navigation.as_deref() {
                println!("      scheduler_sample_navigation: {}", navigation);
            }
            if let Some(samples) = contract.scheduler.result_samples.as_deref() {
                print_scheduler_sample_field("      scheduler_result_samples", samples);
            }
            if let Some(samples) = contract.scheduler.transport_samples.as_deref() {
                print_scheduler_sample_field("      scheduler_transport_samples", samples);
            }
            if let Some(samples) = contract.scheduler.summary_samples.as_deref() {
                print_scheduler_sample_field("      scheduler_summary_samples", samples);
            }
        }
        nuisc::registry::NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET => {
            if let Some(navigation) = contract.std_net.sample_navigation.as_deref() {
                println!("      std_net_navigation: {}", navigation);
            }
            if let Some(samples) = contract.std_net.recipe_samples.as_deref() {
                print_scheduler_sample_field("      std_net_samples", samples);
            }
        }
        _ => {
            println!("      <unrecognized contract group>");
        }
    }
}

fn print_project_scheduler_contract_view(domain: &str) -> Result<(), String> {
    let registration = nuisc::registry::load_domain_registration_for_domain(
        std::path::Path::new("nustar-packages"),
        domain,
    )?;
    let contract = registration.contract;
    println!("    registration:");
    println!("      manifest_path: {}", registration.manifest_path);
    println!("      entry_crate: {}", registration.entry_crate);
    println!("      ast_entry: {}", registration.ast_entry);
    println!("      nir_entry: {}", registration.nir_entry);
    println!(
        "      yir_lowering_entry: {}",
        registration.yir_lowering_entry
    );
    println!(
        "      part_verify_entry: {}",
        registration.part_verify_entry
    );
    if !registration.ast_surface.is_empty() {
        print_scheduler_sample_field("      ast_surface", &registration.ast_surface.join("; "));
    }
    if !registration.nir_surface.is_empty() {
        print_scheduler_sample_field("      nir_surface", &registration.nir_surface.join("; "));
    }
    if !registration.yir_lowering.is_empty() {
        print_scheduler_sample_field("      yir_lowering", &registration.yir_lowering.join("; "));
    }
    if !registration.part_verify.is_empty() {
        print_scheduler_sample_field("      part_verify", &registration.part_verify.join("; "));
    }
    if !registration.resource_families.is_empty() {
        print_scheduler_sample_field(
            "      resource_families",
            &registration.resource_families.join("; "),
        );
    }
    if !registration.unit_types.is_empty() {
        print_scheduler_sample_field("      unit_types", &registration.unit_types.join("; "));
    }
    if !registration.lowering_targets.is_empty() {
        print_scheduler_sample_field(
            "      lowering_targets",
            &registration.lowering_targets.join("; "),
        );
    }
    if !registration.ops.is_empty() {
        print_scheduler_sample_field("      ops", &registration.ops.join("; "));
    }
    print_domain_contract_completeness(&contract);
    print_scheduler_sample_field("contract_groups", &contract.contract_groups.join("; "));
    if !contract.extension_groups.is_empty() {
        print_scheduler_sample_field("extension_groups", &contract.extension_groups.join("; "));
    }
    for group in &contract.contract_groups {
        print_domain_contract_group(&contract, group);
    }
    for group in &contract.extension_groups {
        print_domain_contract_group(&contract, group);
    }
    Ok(())
}

pub(crate) fn print_scheduler_sample_field(label: &str, value: &str) {
    if value.contains("; ") {
        println!("    {}:", label);
        for segment in value.split("; ") {
            println!("      - {}", segment);
        }
    } else {
        println!("    {}: {}", label, value);
    }
}

pub(crate) fn print_project_management_hints(include_galaxy_flow: bool) {
    println!(
        "  project_compile_workflow: {}",
        nuisc::project_compile_workflow_brief()
    );
    print_scheduler_sample_field(
        "project_compile_samples",
        nuisc::project_compile_samples_brief(),
    );
    print_scheduler_sample_field(
        "project_test_workflow",
        nuisc::project_test_workflow_brief(),
    );
    if include_galaxy_flow {
        print_scheduler_sample_field(
            "project_galaxy_workflow",
            nuisc::project_galaxy_workflow_brief(),
        );
    }
}
