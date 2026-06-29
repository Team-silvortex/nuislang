use crate::registry::NustarPackageManifest;
use crate::registry_build_contract_preset::fallback_domain_build_preset;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarDomainLoweringPlanSummary {
    pub lane_policy: String,
    pub bridge_surface: String,
    pub emission_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarDomainBackendStubSummary {
    pub stub_kind: String,
    pub bridge_entry: String,
    pub submission_mode: String,
    pub wake_policy: String,
    pub scheduler_binding: String,
    pub phase_bind: Option<String>,
    pub phase_submit: Option<String>,
    pub phase_wait: Option<String>,
    pub phase_finalize: Option<String>,
    pub transport_model: Option<String>,
    pub request_shape: Option<String>,
    pub response_shape: Option<String>,
    pub dispatch_shape: Option<String>,
    pub memory_binding: Option<String>,
    pub resource_binding: Option<String>,
    pub completion_model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarDomainBridgePlanSummary {
    pub bridge_surface: String,
    pub bridge_entry: String,
    pub scheduler_binding: String,
    pub phase_bind: String,
    pub phase_submit: String,
    pub phase_wait: String,
    pub phase_finalize: String,
    pub bridge_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarHostBridgeSpecSummary {
    pub host_ffi_surface: String,
    pub handle_family: String,
    pub phase_order: Vec<String>,
    pub phase_bind_inputs: Vec<String>,
    pub phase_bind_outputs: Vec<String>,
    pub phase_submit_inputs: Vec<String>,
    pub phase_submit_outputs: Vec<String>,
    pub phase_wait_inputs: Vec<String>,
    pub phase_wait_outputs: Vec<String>,
    pub phase_finalize_inputs: Vec<String>,
    pub phase_finalize_outputs: Vec<String>,
    pub phase_bind_wake: String,
    pub phase_submit_wake: String,
    pub phase_wait_wake: String,
    pub phase_finalize_wake: String,
    pub bridge_plan_begin: bool,
    pub bridge_plan_end: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarDomainBuildContractSummary {
    pub lowering: NustarDomainLoweringPlanSummary,
    pub backend: NustarDomainBackendStubSummary,
    pub bridge: NustarDomainBridgePlanSummary,
    pub host_bridge: NustarHostBridgeSpecSummary,
}

pub fn domain_build_contract_summary(
    manifest: &NustarPackageManifest,
) -> NustarDomainBuildContractSummary {
    let fallback = domain_build_contract_summary_for_domain(&manifest.domain_family);
    let host_bridge_host_ffi_surface = manifest
        .host_bridge_host_ffi_surface
        .clone()
        .map(|values| values.join(","))
        .unwrap_or(fallback.host_bridge.host_ffi_surface);
    let host_bridge_handle_family = manifest
        .host_bridge_handle_family
        .clone()
        .map(|values| values.join(","))
        .unwrap_or(fallback.host_bridge.handle_family);
    NustarDomainBuildContractSummary {
        lowering: NustarDomainLoweringPlanSummary {
            lane_policy: manifest
                .bridge_lane_policy
                .clone()
                .unwrap_or(fallback.lowering.lane_policy),
            bridge_surface: manifest
                .bridge_surface
                .clone()
                .unwrap_or(fallback.lowering.bridge_surface),
            emission_kind: manifest
                .bridge_emission_kind
                .clone()
                .unwrap_or(fallback.lowering.emission_kind),
        },
        backend: NustarDomainBackendStubSummary {
            stub_kind: manifest
                .backend_stub_kind
                .clone()
                .unwrap_or(fallback.backend.stub_kind),
            bridge_entry: manifest
                .bridge_entry
                .clone()
                .unwrap_or(fallback.backend.bridge_entry),
            submission_mode: manifest
                .backend_submission_mode
                .clone()
                .unwrap_or(fallback.backend.submission_mode),
            wake_policy: manifest
                .backend_wake_policy
                .clone()
                .unwrap_or(fallback.backend.wake_policy),
            scheduler_binding: manifest
                .bridge_scheduler_binding
                .clone()
                .unwrap_or(fallback.backend.scheduler_binding),
            phase_bind: manifest.phase_bind.clone().or(fallback.backend.phase_bind),
            phase_submit: manifest
                .phase_submit
                .clone()
                .or(fallback.backend.phase_submit),
            phase_wait: manifest.phase_wait.clone().or(fallback.backend.phase_wait),
            phase_finalize: manifest
                .phase_finalize
                .clone()
                .or(fallback.backend.phase_finalize),
            transport_model: manifest
                .backend_transport_model
                .clone()
                .or(fallback.backend.transport_model),
            request_shape: manifest
                .backend_request_shape
                .clone()
                .or(fallback.backend.request_shape),
            response_shape: manifest
                .backend_response_shape
                .clone()
                .or(fallback.backend.response_shape),
            dispatch_shape: manifest
                .backend_dispatch_shape
                .clone()
                .or(fallback.backend.dispatch_shape),
            memory_binding: manifest
                .backend_memory_binding
                .clone()
                .or(fallback.backend.memory_binding),
            resource_binding: manifest
                .backend_resource_binding
                .clone()
                .or(fallback.backend.resource_binding),
            completion_model: manifest
                .backend_completion_model
                .clone()
                .or(fallback.backend.completion_model),
        },
        bridge: NustarDomainBridgePlanSummary {
            bridge_surface: manifest
                .bridge_surface
                .clone()
                .unwrap_or(fallback.bridge.bridge_surface),
            bridge_entry: manifest
                .bridge_entry
                .clone()
                .unwrap_or(fallback.bridge.bridge_entry),
            scheduler_binding: manifest
                .bridge_scheduler_binding
                .clone()
                .unwrap_or(fallback.bridge.scheduler_binding),
            phase_bind: manifest
                .phase_bind
                .clone()
                .unwrap_or(fallback.bridge.phase_bind),
            phase_submit: manifest
                .phase_submit
                .clone()
                .unwrap_or(fallback.bridge.phase_submit),
            phase_wait: manifest
                .phase_wait
                .clone()
                .unwrap_or(fallback.bridge.phase_wait),
            phase_finalize: manifest
                .phase_finalize
                .clone()
                .unwrap_or(fallback.bridge.phase_finalize),
            bridge_kind: manifest
                .bridge_kind
                .clone()
                .unwrap_or(fallback.bridge.bridge_kind),
        },
        host_bridge: NustarHostBridgeSpecSummary {
            host_ffi_surface: host_bridge_host_ffi_surface,
            handle_family: host_bridge_handle_family,
            phase_order: manifest
                .host_bridge_phase_order
                .clone()
                .unwrap_or(fallback.host_bridge.phase_order),
            phase_bind_inputs: manifest
                .host_bridge_phase_bind_inputs
                .clone()
                .unwrap_or(fallback.host_bridge.phase_bind_inputs),
            phase_bind_outputs: manifest
                .host_bridge_phase_bind_outputs
                .clone()
                .unwrap_or(fallback.host_bridge.phase_bind_outputs),
            phase_submit_inputs: manifest
                .host_bridge_phase_submit_inputs
                .clone()
                .unwrap_or(fallback.host_bridge.phase_submit_inputs),
            phase_submit_outputs: manifest
                .host_bridge_phase_submit_outputs
                .clone()
                .unwrap_or(fallback.host_bridge.phase_submit_outputs),
            phase_wait_inputs: manifest
                .host_bridge_phase_wait_inputs
                .clone()
                .unwrap_or(fallback.host_bridge.phase_wait_inputs),
            phase_wait_outputs: manifest
                .host_bridge_phase_wait_outputs
                .clone()
                .unwrap_or(fallback.host_bridge.phase_wait_outputs),
            phase_finalize_inputs: manifest
                .host_bridge_phase_finalize_inputs
                .clone()
                .unwrap_or(fallback.host_bridge.phase_finalize_inputs),
            phase_finalize_outputs: manifest
                .host_bridge_phase_finalize_outputs
                .clone()
                .unwrap_or(fallback.host_bridge.phase_finalize_outputs),
            phase_bind_wake: manifest
                .host_bridge_phase_bind_wake
                .clone()
                .unwrap_or(fallback.host_bridge.phase_bind_wake),
            phase_submit_wake: manifest
                .host_bridge_phase_submit_wake
                .clone()
                .unwrap_or(fallback.host_bridge.phase_submit_wake),
            phase_wait_wake: manifest
                .host_bridge_phase_wait_wake
                .clone()
                .unwrap_or(fallback.host_bridge.phase_wait_wake),
            phase_finalize_wake: manifest
                .host_bridge_phase_finalize_wake
                .clone()
                .unwrap_or(fallback.host_bridge.phase_finalize_wake),
            bridge_plan_begin: manifest
                .host_bridge_plan_begin
                .unwrap_or(fallback.host_bridge.bridge_plan_begin),
            bridge_plan_end: manifest
                .host_bridge_plan_end
                .unwrap_or(fallback.host_bridge.bridge_plan_end),
        },
    }
}

pub fn domain_build_contract_summary_for_domain(
    domain_family: &str,
) -> NustarDomainBuildContractSummary {
    fallback_domain_build_preset(domain_family)
        .unwrap_or_else(|| fallback_domain_build_preset("host").expect("host preset must exist"))
        .into_summary()
}
