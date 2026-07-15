use crate::registry::{
    NustarDomainContract, NustarDomainRegistration, NUSTAR_DOMAIN_CONTRACT_GROUP_ABI,
    NUSTAR_DOMAIN_CONTRACT_GROUP_DISPATCH_READINESS, NUSTAR_DOMAIN_CONTRACT_GROUP_EXECUTION,
    NUSTAR_DOMAIN_CONTRACT_GROUP_HOST_BRIDGE, NUSTAR_DOMAIN_CONTRACT_GROUP_LOADER,
    NUSTAR_DOMAIN_CONTRACT_GROUP_PACKAGE_IDENTITY, NUSTAR_DOMAIN_CONTRACT_GROUP_RUNTIME,
    NUSTAR_DOMAIN_CONTRACT_GROUP_SCHEDULER, NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET,
};
use crate::registry_json::{
    json_bool_field, json_field, json_object_field, json_optional_string_field,
    json_string_array_field,
};

pub fn domain_contract_object_json(contract: &NustarDomainContract) -> String {
    let package_identity_fields = vec![
        json_field("package", &contract.package_id),
        json_field("domain", &contract.domain_family),
        json_field("frontend", &contract.frontend),
    ];
    let loader_contract_fields = vec![
        json_field("loader_abi", &contract.loader_abi),
        json_field("loader_entry", &contract.loader_entry),
    ];
    let abi_contract_fields = vec![
        json_field("machine_abi_policy", &contract.machine_abi_policy),
        json_string_array_field("abi_profiles", &contract.abi_profiles),
    ];
    let host_bridge_contract_fields = vec![
        json_string_array_field("host_ffi_surface", &contract.host_ffi_surface),
        json_string_array_field("host_ffi_abis", &contract.host_ffi_abis),
        json_optional_string_field("host_ffi_bridge", contract.host_ffi_bridge.as_deref()),
    ];
    let runtime_capability_contract_fields = vec![
        json_string_array_field("support_surface", &contract.capability.support_surface),
        json_string_array_field(
            "support_profile_slots",
            &contract.capability.support_profile_slots,
        ),
        json_string_array_field("capability_tags", &contract.capability.capability_tags),
        json_string_array_field("default_lanes", &contract.capability.default_lanes),
        json_field("clock_domain_id", &contract.capability.clock.domain_id),
        json_field("clock_kind", &contract.capability.clock.kind),
        json_field("clock_epoch_kind", &contract.capability.clock.epoch_kind),
        json_field("clock_resolution", &contract.capability.clock.resolution),
        json_field(
            "clock_bridge_default",
            &contract.capability.clock.bridge_default,
        ),
    ];
    let execution_contract_fields = vec![
        json_field("skeleton_version", &contract.execution.skeleton_version),
        json_field("function_kind", &contract.execution.function_kind),
        json_field("graph_kind", &contract.execution.graph_kind),
        json_field("execution_domain", &contract.execution.execution_domain),
        json_field("default_time_mode", &contract.execution.default_time_mode),
        json_field("contract_family", &contract.execution.contract_family),
        json_string_array_field("lowering_targets", &contract.execution.lowering_targets),
    ];
    let dispatch_readiness_fields = vec![
        json_field("status", &contract.dispatch_readiness.status),
        json_string_array_field(
            "required_signals",
            &contract.dispatch_readiness.required_signals,
        ),
        json_string_array_field(
            "missing_signals",
            &contract.dispatch_readiness.missing_signals,
        ),
        json_bool_field(
            "execution_readiness_materialized",
            contract.dispatch_readiness.execution_readiness_materialized,
        ),
        json_bool_field(
            "dispatch_bridge_materialized",
            contract.dispatch_readiness.dispatch_bridge_materialized,
        ),
        json_string_array_field(
            "lifecycle_phase_order",
            &contract.dispatch_readiness.lifecycle_phase_order,
        ),
        json_field(
            "scheduler_binding",
            &contract.dispatch_readiness.scheduler_binding,
        ),
        json_field("bridge_entry", &contract.dispatch_readiness.bridge_entry),
        json_field(
            "bridge_surface",
            &contract.dispatch_readiness.bridge_surface,
        ),
        json_field(
            "backend_stub_kind",
            &contract.dispatch_readiness.backend_stub_kind,
        ),
        json_field(
            "submission_mode",
            &contract.dispatch_readiness.submission_mode,
        ),
        json_field("wake_policy", &contract.dispatch_readiness.wake_policy),
    ];
    let scheduler_contract_fields = vec![
        json_field(
            "scheduler_contract_stack",
            &contract.scheduler.contract_stack,
        ),
        json_field("scheduler_clock", &contract.scheduler.clock.brief()),
        json_field("scheduler_result_roles", &contract.scheduler.result_roles),
        json_optional_string_field(
            "scheduler_sample_navigation",
            contract.scheduler.sample_navigation.as_deref(),
        ),
        json_optional_string_field(
            "scheduler_result_samples",
            contract.scheduler.result_samples.as_deref(),
        ),
        json_optional_string_field(
            "scheduler_transport_samples",
            contract.scheduler.transport_samples.as_deref(),
        ),
        json_field("scheduler_summary_api", &contract.scheduler.summary_api),
        json_optional_string_field(
            "scheduler_summary_samples",
            contract.scheduler.summary_samples.as_deref(),
        ),
        json_field(
            "scheduler_observer_classes",
            &contract.scheduler.observer_classes,
        ),
    ];
    let std_net_extension_fields = vec![
        json_optional_string_field(
            "std_net_navigation",
            contract.std_net.sample_navigation.as_deref(),
        ),
        json_optional_string_field(
            "std_net_samples",
            contract.std_net.recipe_samples.as_deref(),
        ),
    ];
    let contract_fields = vec![
        json_field("schema", &contract.contract_schema),
        json_field("status", &contract.contract_status),
        json_bool_field("complete", contract.missing_contract_groups.is_empty()),
        json_string_array_field("groups", &contract.contract_groups),
        json_string_array_field("required_groups", &contract.required_contract_groups),
        json_string_array_field("missing_groups", &contract.missing_contract_groups),
        json_string_array_field("extensions", &contract.extension_groups),
        json_object_field(
            NUSTAR_DOMAIN_CONTRACT_GROUP_PACKAGE_IDENTITY,
            &package_identity_fields,
        ),
        json_object_field(NUSTAR_DOMAIN_CONTRACT_GROUP_LOADER, &loader_contract_fields),
        json_object_field(NUSTAR_DOMAIN_CONTRACT_GROUP_ABI, &abi_contract_fields),
        json_object_field(
            NUSTAR_DOMAIN_CONTRACT_GROUP_HOST_BRIDGE,
            &host_bridge_contract_fields,
        ),
        json_object_field(
            NUSTAR_DOMAIN_CONTRACT_GROUP_RUNTIME,
            &runtime_capability_contract_fields,
        ),
        json_object_field(
            NUSTAR_DOMAIN_CONTRACT_GROUP_EXECUTION,
            &execution_contract_fields,
        ),
        json_object_field(
            NUSTAR_DOMAIN_CONTRACT_GROUP_DISPATCH_READINESS,
            &dispatch_readiness_fields,
        ),
        json_object_field(
            NUSTAR_DOMAIN_CONTRACT_GROUP_SCHEDULER,
            &scheduler_contract_fields,
        ),
        json_object_field(
            NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET,
            &std_net_extension_fields,
        ),
    ];
    format!("{{{}}}", contract_fields.join(","))
}

pub fn domain_contract_json(contract: &NustarDomainContract) -> String {
    let fields = vec![
        json_field("package", &contract.package_id),
        json_field("domain", &contract.domain_family),
        json_field("contract_schema", &contract.contract_schema),
        json_field("contract_status", &contract.contract_status),
        json_bool_field(
            "contract_complete",
            contract.missing_contract_groups.is_empty(),
        ),
        json_string_array_field("contract_groups", &contract.contract_groups),
        json_string_array_field(
            "required_contract_groups",
            &contract.required_contract_groups,
        ),
        json_string_array_field("missing_contract_groups", &contract.missing_contract_groups),
        json_string_array_field("extension_groups", &contract.extension_groups),
        json_field("frontend", &contract.frontend),
        json_field("loader_abi", &contract.loader_abi),
        json_field("loader_entry", &contract.loader_entry),
        json_field("machine_abi_policy", &contract.machine_abi_policy),
        json_string_array_field("abi_profiles", &contract.abi_profiles),
        json_string_array_field("host_ffi_surface", &contract.host_ffi_surface),
        json_string_array_field("host_ffi_abis", &contract.host_ffi_abis),
        json_optional_string_field("host_ffi_bridge", contract.host_ffi_bridge.as_deref()),
        json_string_array_field("support_surface", &contract.capability.support_surface),
        json_string_array_field(
            "support_profile_slots",
            &contract.capability.support_profile_slots,
        ),
        json_string_array_field("capability_tags", &contract.capability.capability_tags),
        json_string_array_field("default_lanes", &contract.capability.default_lanes),
        json_field(
            "execution_skeleton_version",
            &contract.execution.skeleton_version,
        ),
        json_field("execution_function_kind", &contract.execution.function_kind),
        json_field("execution_graph_kind", &contract.execution.graph_kind),
        json_field("execution_domain", &contract.execution.execution_domain),
        json_field(
            "execution_default_time_mode",
            &contract.execution.default_time_mode,
        ),
        json_field(
            "execution_contract_family",
            &contract.execution.contract_family,
        ),
        json_string_array_field(
            "execution_lowering_targets",
            &contract.execution.lowering_targets,
        ),
        json_field(
            "dispatch_readiness_status",
            &contract.dispatch_readiness.status,
        ),
        json_bool_field(
            "dispatch_bridge_materialized",
            contract.dispatch_readiness.dispatch_bridge_materialized,
        ),
        json_bool_field(
            "execution_readiness_materialized",
            contract.dispatch_readiness.execution_readiness_materialized,
        ),
        json_string_array_field(
            "dispatch_readiness_required_signals",
            &contract.dispatch_readiness.required_signals,
        ),
        json_string_array_field(
            "dispatch_readiness_missing_signals",
            &contract.dispatch_readiness.missing_signals,
        ),
        json_string_array_field(
            "dispatch_lifecycle_phase_order",
            &contract.dispatch_readiness.lifecycle_phase_order,
        ),
        json_field(
            "dispatch_scheduler_binding",
            &contract.dispatch_readiness.scheduler_binding,
        ),
        json_field(
            "dispatch_bridge_entry",
            &contract.dispatch_readiness.bridge_entry,
        ),
        json_field(
            "dispatch_bridge_surface",
            &contract.dispatch_readiness.bridge_surface,
        ),
        json_field(
            "dispatch_backend_stub_kind",
            &contract.dispatch_readiness.backend_stub_kind,
        ),
        json_field(
            "dispatch_submission_mode",
            &contract.dispatch_readiness.submission_mode,
        ),
        json_field(
            "dispatch_wake_policy",
            &contract.dispatch_readiness.wake_policy,
        ),
        json_field(
            "scheduler_contract_stack",
            &contract.scheduler.contract_stack,
        ),
        json_field("scheduler_clock", &contract.scheduler.clock.brief()),
        json_field("scheduler_result_roles", &contract.scheduler.result_roles),
        json_optional_string_field(
            "scheduler_sample_navigation",
            contract.scheduler.sample_navigation.as_deref(),
        ),
        json_optional_string_field(
            "scheduler_result_samples",
            contract.scheduler.result_samples.as_deref(),
        ),
        json_optional_string_field(
            "scheduler_transport_samples",
            contract.scheduler.transport_samples.as_deref(),
        ),
        json_field("scheduler_summary_api", &contract.scheduler.summary_api),
        json_optional_string_field(
            "scheduler_summary_samples",
            contract.scheduler.summary_samples.as_deref(),
        ),
        json_field(
            "scheduler_observer_classes",
            &contract.scheduler.observer_classes,
        ),
        json_optional_string_field(
            "std_net_navigation",
            contract.std_net.sample_navigation.as_deref(),
        ),
        json_optional_string_field(
            "std_net_samples",
            contract.std_net.recipe_samples.as_deref(),
        ),
        format!("\"contract\":{}", domain_contract_object_json(contract)),
    ];
    format!("{{{}}}", fields.join(","))
}

pub fn domain_registration_object_json(registration: &NustarDomainRegistration) -> String {
    let registration_fields = vec![
        json_field("manifest_path", &registration.manifest_path),
        json_field("entry_crate", &registration.entry_crate),
        json_field("ast_entry", &registration.ast_entry),
        json_field("nir_entry", &registration.nir_entry),
        json_field("yir_lowering_entry", &registration.yir_lowering_entry),
        json_field("part_verify_entry", &registration.part_verify_entry),
        json_string_array_field("ast_surface", &registration.ast_surface),
        json_string_array_field("nir_surface", &registration.nir_surface),
        json_string_array_field("yir_lowering", &registration.yir_lowering),
        json_string_array_field("part_verify", &registration.part_verify),
        json_string_array_field("resource_families", &registration.resource_families),
        json_string_array_field("unit_types", &registration.unit_types),
        json_string_array_field("lowering_targets", &registration.lowering_targets),
        json_string_array_field("ops", &registration.ops),
    ];
    format!("{{{}}}", registration_fields.join(","))
}

pub fn domain_registration_json(registration: &NustarDomainRegistration) -> String {
    let mut fields = domain_contract_json(&registration.contract);
    fields.pop();
    fields.push_str(&format!(
        ",\"registration\":{}",
        domain_registration_object_json(registration)
    ));
    fields.push('}');
    fields
}
