use nuis_artifact::BuildManifestDomainBuildUnit;

use crate::aot_domain_contract::summary_for_unit as domain_build_contract_summary_for_unit;
use crate::aot_domain_profile::{
    derived_lowering_profile_for_unit, kernel_registered_feature_surfaces_for_profile,
    kernel_registered_lane_groups_for_profile, kernel_supported_dispatch_kinds_for_profile,
    render_target_specific_backend_fields, render_target_specific_lowering_fields,
    shader_registered_feature_surfaces_for_profile, shader_registered_lane_groups_for_profile,
    shader_supported_stages_for_profile,
};
use crate::aot_toml::{escape_toml_string, render_string_array};

pub(crate) fn render_domain_build_unit_lowering_plan(
    unit: &BuildManifestDomainBuildUnit,
) -> String {
    let contract = domain_build_contract_summary_for_unit(unit);
    let profile = derived_lowering_profile_for_unit(unit);
    let mut out = String::new();
    out.push_str("schema = \"nuis-domain-lowering-plan-v1\"\n");
    out.push_str(&format!(
        "domain_family = \"{}\"\n",
        escape_toml_string(&unit.domain_family)
    ));
    out.push_str(&format!(
        "package_id = \"{}\"\n",
        escape_toml_string(&unit.package_id)
    ));
    out.push_str(&format!(
        "contract_family = \"{}\"\n",
        escape_toml_string(&unit.contract_family)
    ));
    out.push_str(&format!(
        "packaging_role = \"{}\"\n",
        escape_toml_string(&unit.packaging_role)
    ));
    out.push_str(&format!(
        "backend_family = \"{}\"\n",
        escape_toml_string(unit.backend_family.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "vendor = \"{}\"\n",
        escape_toml_string(unit.vendor.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "device_class = \"{}\"\n",
        escape_toml_string(unit.device_class.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "selected_lowering_target = \"{}\"\n",
        escape_toml_string(unit.selected_lowering_target.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "lowering_profile = \"{}\"\n",
        escape_toml_string(profile.profile_key)
    ));
    out.push_str(&format!(
        "machine_arch = \"{}\"\n",
        escape_toml_string(unit.machine_arch.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "machine_os = \"{}\"\n",
        escape_toml_string(unit.machine_os.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "lane_policy = \"{}\"\n",
        escape_toml_string(&contract.lowering.lane_policy)
    ));
    out.push_str(&format!(
        "bridge_surface = \"{}\"\n",
        escape_toml_string(&contract.lowering.bridge_surface)
    ));
    out.push_str(&format!(
        "emission_kind = \"{}\"\n",
        escape_toml_string(&contract.lowering.emission_kind)
    ));
    out.push_str(&format!(
        "execution_route = \"{}\"\n",
        escape_toml_string(profile.execution_route)
    ));
    out.push_str(&format!(
        "submission_adapter = \"{}\"\n",
        escape_toml_string(profile.submission_adapter)
    ));
    out.push_str(&format!(
        "wake_adapter = \"{}\"\n",
        escape_toml_string(profile.wake_adapter)
    ));
    if let Some(dispatch_kinds) = kernel_supported_dispatch_kinds_for_profile(unit, &profile) {
        out.push_str(&format!(
            "supported_dispatch_kinds = {}\n",
            render_string_array(
                &dispatch_kinds
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect::<Vec<_>>()
            )
        ));
    }
    if let Some(feature_surfaces) = kernel_registered_feature_surfaces_for_profile(unit, &profile) {
        out.push_str(&format!(
            "registered_feature_surfaces = {}\n",
            render_string_array(
                &feature_surfaces
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect::<Vec<_>>()
            )
        ));
    }
    if let Some(lane_groups) = kernel_registered_lane_groups_for_profile(unit, &profile) {
        out.push_str(&format!(
            "registered_lane_groups = {}\n",
            render_string_array(
                &lane_groups
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect::<Vec<_>>()
            )
        ));
    }
    if let Some(stages) = shader_supported_stages_for_profile(unit, &profile) {
        out.push_str(&format!(
            "supported_stages = {}\n",
            render_string_array(&stages.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>())
        ));
    }
    if let Some(feature_surfaces) = shader_registered_feature_surfaces_for_profile(unit, &profile) {
        out.push_str(&format!(
            "registered_feature_surfaces = {}\n",
            render_string_array(
                &feature_surfaces
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect::<Vec<_>>()
            )
        ));
    }
    if let Some(lane_groups) = shader_registered_lane_groups_for_profile(unit, &profile) {
        out.push_str(&format!(
            "registered_lane_groups = {}\n",
            render_string_array(
                &lane_groups
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect::<Vec<_>>()
            )
        ));
    }
    out.push_str(&render_target_specific_lowering_fields(unit, &profile));
    out
}

pub(crate) fn render_domain_build_unit_backend_stub(unit: &BuildManifestDomainBuildUnit) -> String {
    let contract = domain_build_contract_summary_for_unit(unit);
    let backend = contract.backend;
    let profile = derived_lowering_profile_for_unit(unit);
    let mut out = String::new();
    out.push_str("schema = \"nuis-domain-backend-stub-v1\"\n");
    out.push_str(&format!(
        "domain_family = \"{}\"\n",
        escape_toml_string(&unit.domain_family)
    ));
    out.push_str(&format!(
        "package_id = \"{}\"\n",
        escape_toml_string(&unit.package_id)
    ));
    out.push_str(&format!(
        "backend_family = \"{}\"\n",
        escape_toml_string(unit.backend_family.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "vendor = \"{}\"\n",
        escape_toml_string(unit.vendor.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "device_class = \"{}\"\n",
        escape_toml_string(unit.device_class.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "selected_lowering_target = \"{}\"\n",
        escape_toml_string(unit.selected_lowering_target.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "backend_profile = \"{}\"\n",
        escape_toml_string(profile.profile_key)
    ));
    out.push_str(&format!(
        "contract_family = \"{}\"\n",
        escape_toml_string(&unit.contract_family)
    ));
    out.push_str(&format!(
        "packaging_role = \"{}\"\n",
        escape_toml_string(&unit.packaging_role)
    ));
    out.push_str(&format!(
        "machine_arch = \"{}\"\n",
        escape_toml_string(unit.machine_arch.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "machine_os = \"{}\"\n",
        escape_toml_string(unit.machine_os.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "stub_kind = \"{}\"\n",
        escape_toml_string(&backend.stub_kind)
    ));
    out.push_str(&format!(
        "bridge_entry = \"{}\"\n",
        escape_toml_string(&backend.bridge_entry)
    ));
    out.push_str(&format!(
        "submission_mode = \"{}\"\n",
        escape_toml_string(&backend.submission_mode)
    ));
    out.push_str(&format!(
        "wake_policy = \"{}\"\n",
        escape_toml_string(&backend.wake_policy)
    ));
    if let Some(value) = backend.transport_model {
        out.push_str(&format!(
            "transport_model = \"{}\"\n",
            escape_toml_string(&value)
        ));
    }
    if let Some(value) = backend.request_shape {
        out.push_str(&format!(
            "request_shape = \"{}\"\n",
            escape_toml_string(&value)
        ));
    }
    if let Some(value) = backend.response_shape {
        out.push_str(&format!(
            "response_shape = \"{}\"\n",
            escape_toml_string(&value)
        ));
    }
    if let Some(value) = backend.dispatch_shape {
        out.push_str(&format!(
            "dispatch_shape = \"{}\"\n",
            escape_toml_string(&value)
        ));
    }
    if let Some(value) = backend.memory_binding {
        out.push_str(&format!(
            "memory_binding = \"{}\"\n",
            escape_toml_string(&value)
        ));
    }
    if let Some(value) = backend.resource_binding {
        out.push_str(&format!(
            "resource_binding = \"{}\"\n",
            escape_toml_string(&value)
        ));
    }
    if let Some(value) = backend.completion_model {
        out.push_str(&format!(
            "completion_model = \"{}\"\n",
            escape_toml_string(&value)
        ));
    }
    out.push_str(&format!(
        "scheduler_binding = \"{}\"\n",
        escape_toml_string(&backend.scheduler_binding)
    ));
    out.push_str(&format!(
        "execution_route = \"{}\"\n",
        escape_toml_string(profile.execution_route)
    ));
    out.push_str(&format!(
        "submission_adapter = \"{}\"\n",
        escape_toml_string(profile.submission_adapter)
    ));
    out.push_str(&format!(
        "wake_adapter = \"{}\"\n",
        escape_toml_string(profile.wake_adapter)
    ));
    if let Some(dispatch_kinds) = kernel_supported_dispatch_kinds_for_profile(unit, &profile) {
        out.push_str(&format!(
            "supported_dispatch_kinds = {}\n",
            render_string_array(
                &dispatch_kinds
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect::<Vec<_>>()
            )
        ));
    }
    out.push_str(&render_target_specific_backend_fields(unit, &profile));
    if let Some(value) = backend.phase_bind {
        let key = if unit.domain_family == "network" {
            "connect_phase"
        } else {
            "bind_phase"
        };
        out.push_str(&format!("{key} = \"{}\"\n", escape_toml_string(&value)));
    }
    if let Some(value) = backend.phase_submit {
        let key = if unit.domain_family == "network" {
            "send_phase"
        } else {
            "launch_phase"
        };
        out.push_str(&format!("{key} = \"{}\"\n", escape_toml_string(&value)));
    }
    if let Some(value) = backend.phase_wait {
        let key = if unit.domain_family == "network" {
            "recv_phase"
        } else {
            "wait_phase"
        };
        out.push_str(&format!("{key} = \"{}\"\n", escape_toml_string(&value)));
    }
    if let Some(value) = backend.phase_finalize {
        out.push_str(&format!(
            "finalize_phase = \"{}\"\n",
            escape_toml_string(&value)
        ));
    }
    out
}

pub(crate) fn render_domain_build_unit_bridge_plan(unit: &BuildManifestDomainBuildUnit) -> String {
    let contract = domain_build_contract_summary_for_unit(unit);
    let bridge = contract.bridge;
    let mut out = String::new();
    out.push_str("schema = \"nuis-domain-bridge-plan-v1\"\n");
    out.push_str(&format!(
        "domain_family = \"{}\"\n",
        escape_toml_string(&unit.domain_family)
    ));
    out.push_str(&format!(
        "package_id = \"{}\"\n",
        escape_toml_string(&unit.package_id)
    ));
    out.push_str(&format!(
        "bridge_surface = \"{}\"\n",
        escape_toml_string(&bridge.bridge_surface)
    ));
    out.push_str(&format!(
        "bridge_entry = \"{}\"\n",
        escape_toml_string(&bridge.bridge_entry)
    ));
    out.push_str(&format!(
        "scheduler_binding = \"{}\"\n",
        escape_toml_string(&bridge.scheduler_binding)
    ));
    out.push_str(&format!(
        "phase_bind = \"{}\"\n",
        escape_toml_string(&bridge.phase_bind)
    ));
    out.push_str(&format!(
        "phase_submit = \"{}\"\n",
        escape_toml_string(&bridge.phase_submit)
    ));
    out.push_str(&format!(
        "phase_wait = \"{}\"\n",
        escape_toml_string(&bridge.phase_wait)
    ));
    out.push_str(&format!(
        "phase_finalize = \"{}\"\n",
        escape_toml_string(&bridge.phase_finalize)
    ));
    out.push_str(&format!(
        "bridge_kind = \"{}\"\n",
        escape_toml_string(&bridge.bridge_kind)
    ));
    out
}

pub(crate) fn render_domain_build_unit_host_bridge_stub(
    unit: &BuildManifestDomainBuildUnit,
) -> String {
    let contract = domain_build_contract_summary_for_unit(unit);
    let bridge = &contract.bridge;
    let host_bridge = &contract.host_bridge;
    let profile = derived_lowering_profile_for_unit(unit);
    let bridge_plan = render_domain_build_unit_bridge_plan(unit);
    let mut out = String::new();
    out.push_str("schema = \"nuis-host-bridge-spec-v1\"\n");
    out.push_str(&format!(
        "domain_family = \"{}\"\n",
        escape_toml_string(&unit.domain_family)
    ));
    out.push_str(&format!(
        "package_id = \"{}\"\n",
        escape_toml_string(&unit.package_id)
    ));
    out.push_str(&format!(
        "backend_family = \"{}\"\n",
        escape_toml_string(unit.backend_family.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "vendor = \"{}\"\n",
        escape_toml_string(unit.vendor.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "device_class = \"{}\"\n",
        escape_toml_string(unit.device_class.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "selected_lowering_target = \"{}\"\n",
        escape_toml_string(unit.selected_lowering_target.as_deref().unwrap_or("none"))
    ));
    out.push_str(&format!(
        "bridge_profile = \"{}\"\n",
        escape_toml_string(profile.profile_key)
    ));
    out.push_str(&format!(
        "bridge_surface = \"{}\"\n",
        escape_toml_string(&bridge.bridge_surface)
    ));
    out.push_str(&format!(
        "bridge_entry = \"{}\"\n",
        escape_toml_string(&bridge.bridge_entry)
    ));
    out.push_str(&format!(
        "scheduler_binding = \"{}\"\n",
        escape_toml_string(&bridge.scheduler_binding)
    ));
    out.push_str(&format!(
        "execution_route = \"{}\"\n",
        escape_toml_string(profile.execution_route)
    ));
    out.push_str(&format!(
        "submission_adapter = \"{}\"\n",
        escape_toml_string(profile.submission_adapter)
    ));
    out.push_str(&format!(
        "wake_adapter = \"{}\"\n",
        escape_toml_string(profile.wake_adapter)
    ));
    out.push_str(&format!(
        "host_ffi_surface = \"{}\"\n",
        escape_toml_string(&host_bridge.host_ffi_surface)
    ));
    out.push_str(&format!(
        "handle_family = \"{}\"\n",
        escape_toml_string(&host_bridge.handle_family)
    ));
    out.push_str(&format!(
        "phase_order = {}\n",
        render_string_array(&host_bridge.phase_order)
    ));
    out.push_str(&format!(
        "phase_bind_inputs = {}\n",
        render_string_array(&host_bridge.phase_bind_inputs)
    ));
    out.push_str(&format!(
        "phase_bind_outputs = {}\n",
        render_string_array(&host_bridge.phase_bind_outputs)
    ));
    out.push_str(&format!(
        "phase_submit_inputs = {}\n",
        render_string_array(&host_bridge.phase_submit_inputs)
    ));
    out.push_str(&format!(
        "phase_submit_outputs = {}\n",
        render_string_array(&host_bridge.phase_submit_outputs)
    ));
    out.push_str(&format!(
        "phase_wait_inputs = {}\n",
        render_string_array(&host_bridge.phase_wait_inputs)
    ));
    out.push_str(&format!(
        "phase_wait_outputs = {}\n",
        render_string_array(&host_bridge.phase_wait_outputs)
    ));
    out.push_str(&format!(
        "phase_finalize_inputs = {}\n",
        render_string_array(&host_bridge.phase_finalize_inputs)
    ));
    out.push_str(&format!(
        "phase_finalize_outputs = {}\n",
        render_string_array(&host_bridge.phase_finalize_outputs)
    ));
    out.push_str(&format!(
        "phase_bind_wake = \"{}\"\n",
        escape_toml_string(&host_bridge.phase_bind_wake)
    ));
    out.push_str(&format!(
        "phase_submit_wake = \"{}\"\n",
        escape_toml_string(&host_bridge.phase_submit_wake)
    ));
    out.push_str(&format!(
        "phase_wait_wake = \"{}\"\n",
        escape_toml_string(&host_bridge.phase_wait_wake)
    ));
    out.push_str(&format!(
        "phase_finalize_wake = \"{}\"\n",
        escape_toml_string(&host_bridge.phase_finalize_wake)
    ));
    out.push_str(&format!(
        "bridge_plan_begin = {}\n",
        if host_bridge.bridge_plan_begin {
            "true"
        } else {
            "false"
        }
    ));
    out.push_str(&bridge_plan);
    if !bridge_plan.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&format!(
        "bridge_plan_end = {}\n",
        if host_bridge.bridge_plan_end {
            "true"
        } else {
            "false"
        }
    ));
    out
}
