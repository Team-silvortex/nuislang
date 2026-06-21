use std::{collections::BTreeSet, path::Path};

use crate::{galaxy, PublicSurfaceModuleRecord, WorkflowFrontdoorSurface};

pub(crate) fn workflow_contract_json_fields(
    frontdoor: &WorkflowFrontdoorSurface,
    include_project_compile: bool,
    include_project_test: bool,
    include_project_galaxy: bool,
    include_debug: bool,
) -> Vec<String> {
    let mut fields = vec![
        crate::json_object_field(
            "frontdoor",
            &crate::workflow_frontdoor_json_fields(frontdoor),
        ),
        crate::json_field("workflow_kind", frontdoor.workflow_kind),
        crate::json_field("workflow_brief", frontdoor.workflow_brief),
        crate::json_field("workflow_samples", frontdoor.workflow_samples),
        crate::json_field("recommended_next_step", frontdoor.recommended_next_step),
        crate::json_field("recommended_command", frontdoor.recommended_command),
        crate::json_field("recommended_reason", frontdoor.recommended_reason),
    ];
    if include_project_compile {
        fields.push(crate::json_field(
            "project_compile_workflow",
            frontdoor.workflow_brief,
        ));
        fields.push(crate::json_field(
            "project_compile_samples",
            frontdoor.workflow_samples,
        ));
    }
    if include_project_test {
        fields.push(crate::json_field(
            "project_test_workflow",
            nuisc::project_test_workflow_brief(),
        ));
    }
    if include_project_galaxy {
        fields.push(crate::json_field(
            "project_galaxy_workflow",
            nuisc::project_galaxy_workflow_brief(),
        ));
    }
    if include_debug {
        fields.push(crate::json_field(
            "debug_workflow",
            crate::debug_workflow_brief(),
        ));
        fields.push(crate::json_field(
            "debug_samples",
            crate::debug_workflow_samples_brief(),
        ));
    }
    fields
}

pub(crate) fn public_surface_summary_json_fields(
    records: &[PublicSurfaceModuleRecord],
) -> Vec<String> {
    let public_extern_count = records
        .iter()
        .map(|record| record.externs.len())
        .sum::<usize>();
    let public_extern_interface_count = records
        .iter()
        .map(|record| record.extern_interfaces.len())
        .sum::<usize>();
    let public_const_count = records
        .iter()
        .map(|record| record.consts.len())
        .sum::<usize>();
    let public_function_count = records
        .iter()
        .map(|record| record.functions.len())
        .sum::<usize>();
    let public_type_alias_count = records
        .iter()
        .map(|record| record.type_aliases.len())
        .sum::<usize>();
    let public_struct_count = records
        .iter()
        .map(|record| record.structs.len())
        .sum::<usize>();
    let public_trait_count = records
        .iter()
        .map(|record| record.traits.len())
        .sum::<usize>();
    vec![
        crate::json_usize_field("public_surface_modules", records.len()),
        crate::json_usize_field("public_externs", public_extern_count),
        crate::json_usize_field("public_extern_interfaces", public_extern_interface_count),
        crate::json_usize_field("public_consts", public_const_count),
        crate::json_usize_field("public_type_aliases", public_type_alias_count),
        crate::json_usize_field("public_functions", public_function_count),
        crate::json_usize_field("public_structs", public_struct_count),
        crate::json_usize_field("public_traits", public_trait_count),
    ]
}

pub(crate) fn project_plan_json_fields(
    plan: &nuisc::project::ProjectCompilationPlan,
) -> Vec<String> {
    vec![
        crate::json_field(
            "project_plan",
            &nuisc::project::describe_project_compilation_plan(plan),
        ),
        crate::json_field(
            "project_plan_dependency_categories",
            &nuisc::project::describe_project_dependency_categories(plan),
        ),
        crate::json_usize_field("project_plan_dependency_count", plan.dependencies.len()),
        crate::json_field(
            "project_plan_synthetic_input_kind",
            &plan.synthetic_input.kind,
        ),
        crate::json_field(
            "project_plan_synthetic_input",
            &plan.synthetic_input.path.display().to_string(),
        ),
        crate::json_field(
            "project_plan_output_categories",
            &nuisc::project::describe_project_output_intent_categories(plan),
        ),
        crate::json_usize_field("project_plan_output_count", plan.output_intents.len()),
        crate::json_field("project_organization_entry", &plan.organization.entry),
        crate::json_field("project_domains", &plan.organization.domains.join(", ")),
        crate::json_field(
            "project_exchange_route_classes",
            &nuisc::project::describe_project_exchange_route_classes(plan),
        ),
        crate::json_usize_field("project_exchange_route_count", plan.exchanges.routes.len()),
    ]
}

pub(crate) fn project_check_summary_json_fields(
    abi_checks: &[nuisc::project::ProjectAbiSelectionCheck],
    registry_checks: &[nuisc::registry::ProjectDomainRegistryCheck],
    lowering_checks: &[nuisc::project::ProjectLoweringSelectionView],
) -> Vec<String> {
    vec![
        crate::json_usize_field("abi_checks_count", abi_checks.len()),
        crate::json_bool_field("abi_checks_ok", abi_checks.iter().all(|check| check.ok)),
        crate::json_usize_field("registry_checks_count", registry_checks.len()),
        crate::json_bool_field(
            "registry_checks_ok",
            registry_checks.iter().all(|check| check.ok),
        ),
        crate::json_usize_field("lowering_checks_count", lowering_checks.len()),
        crate::json_bool_field(
            "lowering_checks_ok",
            lowering_checks.iter().all(|check| check.ok),
        ),
    ]
}

pub(crate) fn galaxy_lock_json_fields(
    status: Result<galaxy::VerifiedGalaxyLock, String>,
    lock_path: &Path,
    declared_dependencies: &[String],
) -> Vec<String> {
    match status {
        Ok(lock) => {
            let locked = lock
                .entries
                .iter()
                .map(|item| format!("{}={}", item.name, item.version))
                .collect::<BTreeSet<_>>();
            let declared = declared_dependencies
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            vec![
                crate::json_field("galaxy_lock_status", "ok"),
                crate::json_field("galaxy_lock_path", &lock.path.display().to_string()),
                crate::json_usize_field("galaxy_lock_dependencies", lock.entries.len()),
                crate::json_bool_field("galaxy_lock_matches_manifest", declared == locked),
                crate::json_string_array_field(
                    "galaxy_lock_entries",
                    &lock
                        .entries
                        .iter()
                        .map(|item| {
                            format!("{}={} {}", item.name, item.version, item.bundle_fnv1a64)
                        })
                        .collect::<Vec<_>>(),
                ),
            ]
        }
        Err(error) if lock_path.exists() => vec![
            crate::json_field("galaxy_lock_status", "invalid"),
            crate::json_field("galaxy_lock_path", &lock_path.display().to_string()),
            crate::json_field("galaxy_lock_error", &error),
        ],
        Err(_) => vec![
            crate::json_field("galaxy_lock_status", "missing"),
            crate::json_field("galaxy_lock_path", &lock_path.display().to_string()),
        ],
    }
}
