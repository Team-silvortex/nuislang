use std::{
    collections::BTreeSet,
    fmt, fs,
    path::{Path, PathBuf},
};

use crate::data_markers::{
    all_downlink_directional_marker_slots, all_sync_marker_slots,
    all_uplink_directional_marker_slots, data_common_marker_slots, data_marker_surface,
};
use nuis_semantics::model::{NirExpr, NirModule, NirStmt};
use yir_core::YirModule;

const INDEX_FILE: &str = "index.toml";
pub const NUSTAR_DOMAIN_CONTRACT_SCHEMA: &str = "nustar-domain-contract-v1";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_PACKAGE_IDENTITY: &str = "package_identity";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_LOADER: &str = "loader_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_ABI: &str = "abi_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_HOST_BRIDGE: &str = "host_bridge_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_RUNTIME: &str = "runtime_capability_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_SCHEDULER: &str = "scheduler_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_EXECUTION: &str = "execution_skeleton_contract";
pub const NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET: &str = "std_net_extension";

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

fn json_field(name: &str, value: &str) -> String {
    format!("\"{}\":\"{}\"", name, json_escape(value))
}

fn json_optional_string_field(name: &str, value: Option<&str>) -> String {
    match value {
        Some(value) => format!("\"{}\":\"{}\"", name, json_escape(value)),
        None => format!("\"{}\":null", name),
    }
}

fn json_bool_field(name: &str, value: bool) -> String {
    format!("\"{}\":{}", name, if value { "true" } else { "false" })
}

fn json_string_array_field(name: &str, values: &[String]) -> String {
    let entries = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("\"{}\":[{}]", name, entries)
}

fn json_object_field(name: &str, fields: &[String]) -> String {
    format!("\"{}\":{{{}}}", name, fields.join(","))
}

fn resolve_registry_root(root: &Path) -> PathBuf {
    if root.exists() {
        return root.to_path_buf();
    }
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let candidate = workspace_root.join(root);
    if candidate.exists() {
        candidate
    } else {
        root.to_path_buf()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarPackageIndexEntry {
    pub package_id: String,
    pub manifest: String,
    pub domain_family: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarPackageManifest {
    pub manifest_schema: String,
    pub package_id: String,
    pub domain_family: String,
    pub frontend: String,
    pub entry_crate: String,
    pub ast_entry: String,
    pub nir_entry: String,
    pub yir_lowering_entry: String,
    pub part_verify_entry: String,
    pub ast_surface: Vec<String>,
    pub nir_surface: Vec<String>,
    pub yir_lowering: Vec<String>,
    pub part_verify: Vec<String>,
    pub binary_extension: String,
    pub package_layout: String,
    pub machine_abi_policy: String,
    pub abi_profiles: Vec<String>,
    pub abi_capabilities: Vec<String>,
    pub abi_targets: Vec<String>,
    pub implementation_kinds: Vec<String>,
    pub loader_entry: String,
    pub loader_abi: String,
    pub host_ffi_surface: Vec<String>,
    pub host_ffi_abis: Vec<String>,
    pub host_ffi_bridge: String,
    pub bridge_lane_policy: Option<String>,
    pub bridge_surface: Option<String>,
    pub bridge_emission_kind: Option<String>,
    pub bridge_entry: Option<String>,
    pub bridge_kind: Option<String>,
    pub bridge_scheduler_binding: Option<String>,
    pub backend_stub_kind: Option<String>,
    pub backend_submission_mode: Option<String>,
    pub backend_wake_policy: Option<String>,
    pub backend_transport_model: Option<String>,
    pub backend_request_shape: Option<String>,
    pub backend_response_shape: Option<String>,
    pub backend_dispatch_shape: Option<String>,
    pub backend_memory_binding: Option<String>,
    pub backend_resource_binding: Option<String>,
    pub backend_completion_model: Option<String>,
    pub phase_bind: Option<String>,
    pub phase_submit: Option<String>,
    pub phase_wait: Option<String>,
    pub phase_finalize: Option<String>,
    pub host_bridge_host_ffi_surface: Option<Vec<String>>,
    pub host_bridge_handle_family: Option<Vec<String>>,
    pub host_bridge_phase_order: Option<Vec<String>>,
    pub host_bridge_phase_bind_inputs: Option<Vec<String>>,
    pub host_bridge_phase_bind_outputs: Option<Vec<String>>,
    pub host_bridge_phase_submit_inputs: Option<Vec<String>>,
    pub host_bridge_phase_submit_outputs: Option<Vec<String>>,
    pub host_bridge_phase_wait_inputs: Option<Vec<String>>,
    pub host_bridge_phase_wait_outputs: Option<Vec<String>>,
    pub host_bridge_phase_finalize_inputs: Option<Vec<String>>,
    pub host_bridge_phase_finalize_outputs: Option<Vec<String>>,
    pub host_bridge_phase_bind_wake: Option<String>,
    pub host_bridge_phase_submit_wake: Option<String>,
    pub host_bridge_phase_wait_wake: Option<String>,
    pub host_bridge_phase_finalize_wake: Option<String>,
    pub host_bridge_plan_begin: Option<bool>,
    pub host_bridge_plan_end: Option<bool>,
    pub support_surface: Vec<String>,
    pub support_profile_slots: Vec<String>,
    pub capability_tags: Vec<String>,
    pub default_lanes: Vec<String>,
    pub clock_domain_id: String,
    pub clock_kind: String,
    pub clock_epoch_kind: String,
    pub clock_resolution: String,
    pub clock_bridge_default: String,
    pub profiles: Vec<String>,
    pub resource_families: Vec<String>,
    pub unit_types: Vec<String>,
    pub lowering_targets: Vec<String>,
    pub ops: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredAbiTarget {
    pub abi: String,
    pub machine_arch: String,
    pub machine_os: String,
    pub object_format: String,
    pub calling_abi: String,
    pub clang_target: String,
    pub backend_family: Option<String>,
    pub vendor: Option<String>,
    pub device_class: Option<String>,
    pub host_adaptive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarBinding {
    pub package_id: String,
    pub domain_family: String,
    pub ast_entry: String,
    pub nir_entry: String,
    pub yir_lowering_entry: String,
    pub part_verify_entry: String,
    pub machine_abi_policy: String,
    pub abi_profiles: Vec<String>,
    pub abi_capabilities: Vec<String>,
    pub ast_surface: Vec<String>,
    pub nir_surface: Vec<String>,
    pub yir_lowering: Vec<String>,
    pub part_verify: Vec<String>,
    pub support_surface: Vec<String>,
    pub support_profile_slots: Vec<String>,
    pub capability_tags: Vec<String>,
    pub default_lanes: Vec<String>,
    pub execution: NustarExecutionSummary,
    pub matched_support_surface: Vec<String>,
    pub matched_support_profile_slots: Vec<String>,
    pub covered_support_profile_slots: Vec<String>,
    pub uncovered_support_profile_slots: Vec<String>,
    pub registered_units: Vec<String>,
    pub bound_unit: Option<String>,
    pub used_units: Vec<String>,
    pub instantiated_units: Vec<String>,
    pub used_host_ffi_abis: Vec<String>,
    pub used_host_ffi_symbols: Vec<String>,
    pub matched_resources: Vec<String>,
    pub matched_ops: Vec<String>,
    pub undeclared_ops: Vec<String>,
    pub frontend: String,
    pub entry_crate: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarBindingPlan {
    pub bindings: Vec<NustarBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarClockSummary {
    pub domain_id: String,
    pub kind: String,
    pub epoch_kind: String,
    pub resolution: String,
    pub bridge_default: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarCapabilitySummary {
    pub support_surface: Vec<String>,
    pub support_profile_slots: Vec<String>,
    pub capability_tags: Vec<String>,
    pub default_lanes: Vec<String>,
    pub clock: NustarClockSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarExecutionSummary {
    pub skeleton_version: String,
    pub function_kind: String,
    pub graph_kind: String,
    pub execution_domain: String,
    pub default_time_mode: String,
    pub contract_family: String,
    pub lowering_targets: Vec<String>,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct FallbackDomainBuildPreset {
    lane_policy: &'static str,
    bridge_surface: &'static str,
    emission_kind: &'static str,
    bridge_entry: &'static str,
    bridge_kind: &'static str,
    stub_kind: &'static str,
    submission_mode: &'static str,
    wake_policy: &'static str,
    scheduler_binding: &'static str,
    phase_bind: &'static str,
    phase_submit: &'static str,
    phase_wait: &'static str,
    phase_finalize: &'static str,
    transport_model: Option<&'static str>,
    request_shape: Option<&'static str>,
    response_shape: Option<&'static str>,
    dispatch_shape: Option<&'static str>,
    memory_binding: Option<&'static str>,
    resource_binding: Option<&'static str>,
    completion_model: Option<&'static str>,
    host_ffi_surface: &'static [&'static str],
    handle_family: &'static [&'static str],
    phase_bind_inputs: &'static [&'static str],
    phase_bind_outputs: &'static [&'static str],
    phase_submit_inputs: &'static [&'static str],
    phase_submit_outputs: &'static [&'static str],
    phase_wait_inputs: &'static [&'static str],
    phase_wait_outputs: &'static [&'static str],
    phase_finalize_inputs: &'static [&'static str],
    phase_finalize_outputs: &'static [&'static str],
    phase_bind_wake: &'static str,
    phase_submit_wake: &'static str,
    phase_wait_wake: &'static str,
    phase_finalize_wake: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FallbackBackendFacet {
    stub_kind: &'static str,
    submission_mode: &'static str,
    wake_policy: &'static str,
    transport_model: Option<&'static str>,
    request_shape: Option<&'static str>,
    response_shape: Option<&'static str>,
    dispatch_shape: Option<&'static str>,
    memory_binding: Option<&'static str>,
    resource_binding: Option<&'static str>,
    completion_model: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FallbackBridgeFlowFacet {
    lane_policy: &'static str,
    bridge_surface: &'static str,
    bridge_entry: &'static str,
    scheduler_binding: &'static str,
    phase_bind: &'static str,
    phase_submit: &'static str,
    phase_wait: &'static str,
    phase_finalize: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FallbackHostBridgeFacet {
    host_ffi_surface: &'static [&'static str],
    handle_family: &'static [&'static str],
    phase_bind_inputs: &'static [&'static str],
    phase_bind_outputs: &'static [&'static str],
    phase_submit_inputs: &'static [&'static str],
    phase_submit_outputs: &'static [&'static str],
    phase_wait_inputs: &'static [&'static str],
    phase_wait_outputs: &'static [&'static str],
    phase_finalize_inputs: &'static [&'static str],
    phase_finalize_outputs: &'static [&'static str],
    phase_bind_wake: &'static str,
    phase_submit_wake: &'static str,
    phase_wait_wake: &'static str,
    phase_finalize_wake: &'static str,
}

const DEFAULT_BRIDGE_PHASE_ORDER: [&str; 4] = ["bind", "submit", "wait", "finalize"];

fn preset_vec(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| (*v).to_owned()).collect()
}

fn preset_csv(values: &[&str]) -> String {
    values.join(",")
}

fn fallback_preset(
    flow: FallbackBridgeFlowFacet,
    backend: FallbackBackendFacet,
    host_bridge: FallbackHostBridgeFacet,
) -> FallbackDomainBuildPreset {
    FallbackDomainBuildPreset {
        lane_policy: flow.lane_policy,
        bridge_surface: flow.bridge_surface,
        emission_kind: "sidecar-plan",
        bridge_entry: flow.bridge_entry,
        bridge_kind: "managed-lifecycle-bridge",
        stub_kind: backend.stub_kind,
        submission_mode: backend.submission_mode,
        wake_policy: backend.wake_policy,
        scheduler_binding: flow.scheduler_binding,
        phase_bind: flow.phase_bind,
        phase_submit: flow.phase_submit,
        phase_wait: flow.phase_wait,
        phase_finalize: flow.phase_finalize,
        transport_model: backend.transport_model,
        request_shape: backend.request_shape,
        response_shape: backend.response_shape,
        dispatch_shape: backend.dispatch_shape,
        memory_binding: backend.memory_binding,
        resource_binding: backend.resource_binding,
        completion_model: backend.completion_model,
        host_ffi_surface: host_bridge.host_ffi_surface,
        handle_family: host_bridge.handle_family,
        phase_bind_inputs: host_bridge.phase_bind_inputs,
        phase_bind_outputs: host_bridge.phase_bind_outputs,
        phase_submit_inputs: host_bridge.phase_submit_inputs,
        phase_submit_outputs: host_bridge.phase_submit_outputs,
        phase_wait_inputs: host_bridge.phase_wait_inputs,
        phase_wait_outputs: host_bridge.phase_wait_outputs,
        phase_finalize_inputs: host_bridge.phase_finalize_inputs,
        phase_finalize_outputs: host_bridge.phase_finalize_outputs,
        phase_bind_wake: host_bridge.phase_bind_wake,
        phase_submit_wake: host_bridge.phase_submit_wake,
        phase_wait_wake: host_bridge.phase_wait_wake,
        phase_finalize_wake: host_bridge.phase_finalize_wake,
    }
}

impl NustarClockSummary {
    pub fn brief(&self) -> String {
        format!(
            "{} [{}] bridge={}",
            self.domain_id, self.kind, self.bridge_default
        )
    }
}

pub fn capability_summary(manifest: &NustarPackageManifest) -> NustarCapabilitySummary {
    NustarCapabilitySummary {
        support_surface: manifest.support_surface.clone(),
        support_profile_slots: manifest.support_profile_slots.clone(),
        capability_tags: manifest.capability_tags.clone(),
        default_lanes: manifest.default_lanes.clone(),
        clock: NustarClockSummary {
            domain_id: manifest.clock_domain_id.clone(),
            kind: manifest.clock_kind.clone(),
            epoch_kind: manifest.clock_epoch_kind.clone(),
            resolution: manifest.clock_resolution.clone(),
            bridge_default: manifest.clock_bridge_default.clone(),
        },
    }
}

pub fn execution_summary(manifest: &NustarPackageManifest) -> NustarExecutionSummary {
    NustarExecutionSummary {
        skeleton_version: "nustar-execution-skeleton-v1".to_owned(),
        function_kind: "function-node".to_owned(),
        graph_kind: "function-graph".to_owned(),
        execution_domain: manifest.domain_family.clone(),
        default_time_mode: "logical".to_owned(),
        contract_family: format!("nustar.{}", manifest.domain_family),
        lowering_targets: manifest.lowering_targets.clone(),
    }
}

fn infer_default_capability_tags(domain_family: &str) -> Vec<String> {
    match domain_family {
        "cpu" => vec![
            "host-execution",
            "native-llvm",
            "memory-runtime",
            "syscall-surface",
        ],
        "data" => vec![
            "fabric-plane",
            "packet-layout",
            "cross-domain-marker",
            "window-transfer",
        ],
        "shader" => vec!["gpu-render", "frame-graph", "shader-ir", "resource-binding"],
        "kernel" => vec![
            "accelerator-compute",
            "tensor-kernel",
            "device-dispatch",
            "buffer-table",
        ],
        "network" => vec![
            "io-reactor",
            "socket-transport",
            "packet-exchange",
            "async-bridge",
        ],
        _ => vec!["custom-domain"],
    }
    .into_iter()
    .map(str::to_owned)
    .collect()
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

impl FallbackDomainBuildPreset {
    fn into_summary(self) -> NustarDomainBuildContractSummary {
        NustarDomainBuildContractSummary {
            lowering: NustarDomainLoweringPlanSummary {
                lane_policy: self.lane_policy.to_owned(),
                bridge_surface: self.bridge_surface.to_owned(),
                emission_kind: self.emission_kind.to_owned(),
            },
            backend: NustarDomainBackendStubSummary {
                stub_kind: self.stub_kind.to_owned(),
                bridge_entry: self.bridge_entry.to_owned(),
                submission_mode: self.submission_mode.to_owned(),
                wake_policy: self.wake_policy.to_owned(),
                scheduler_binding: self.scheduler_binding.to_owned(),
                phase_bind: Some(self.phase_bind.to_owned()),
                phase_submit: Some(self.phase_submit.to_owned()),
                phase_wait: Some(self.phase_wait.to_owned()),
                phase_finalize: Some(self.phase_finalize.to_owned()),
                transport_model: self.transport_model.map(str::to_owned),
                request_shape: self.request_shape.map(str::to_owned),
                response_shape: self.response_shape.map(str::to_owned),
                dispatch_shape: self.dispatch_shape.map(str::to_owned),
                memory_binding: self.memory_binding.map(str::to_owned),
                resource_binding: self.resource_binding.map(str::to_owned),
                completion_model: self.completion_model.map(str::to_owned),
            },
            bridge: NustarDomainBridgePlanSummary {
                bridge_surface: self.bridge_surface.to_owned(),
                bridge_entry: self.bridge_entry.to_owned(),
                scheduler_binding: self.scheduler_binding.to_owned(),
                phase_bind: self.phase_bind.to_owned(),
                phase_submit: self.phase_submit.to_owned(),
                phase_wait: self.phase_wait.to_owned(),
                phase_finalize: self.phase_finalize.to_owned(),
                bridge_kind: self.bridge_kind.to_owned(),
            },
            host_bridge: NustarHostBridgeSpecSummary {
                host_ffi_surface: preset_csv(self.host_ffi_surface),
                handle_family: preset_csv(self.handle_family),
                phase_order: preset_vec(&DEFAULT_BRIDGE_PHASE_ORDER),
                phase_bind_inputs: preset_vec(self.phase_bind_inputs),
                phase_bind_outputs: preset_vec(self.phase_bind_outputs),
                phase_submit_inputs: preset_vec(self.phase_submit_inputs),
                phase_submit_outputs: preset_vec(self.phase_submit_outputs),
                phase_wait_inputs: preset_vec(self.phase_wait_inputs),
                phase_wait_outputs: preset_vec(self.phase_wait_outputs),
                phase_finalize_inputs: preset_vec(self.phase_finalize_inputs),
                phase_finalize_outputs: preset_vec(self.phase_finalize_outputs),
                phase_bind_wake: self.phase_bind_wake.to_owned(),
                phase_submit_wake: self.phase_submit_wake.to_owned(),
                phase_wait_wake: self.phase_wait_wake.to_owned(),
                phase_finalize_wake: self.phase_finalize_wake.to_owned(),
                bridge_plan_begin: true,
                bridge_plan_end: true,
            },
        }
    }
}

fn fallback_domain_build_preset(domain_family: &str) -> Option<FallbackDomainBuildPreset> {
    match domain_family {
        "network" => Some(fallback_preset(
            FallbackBridgeFlowFacet {
                lane_policy: "dispatch-lanes.io-bound",
                bridge_surface: "host-ffi.bridge.network",
                bridge_entry: "nuis.network.bridge.dispatch.v1",
                scheduler_binding: "network-poll-bridge",
                phase_bind: "socket-bind-or-session-open",
                phase_submit: "packet-write-dispatch",
                phase_wait: "callback-or-read-ready",
                phase_finalize: "response-commit-and-wake",
            },
            FallbackBackendFacet {
                stub_kind: "network-host-bridge",
                submission_mode: "request-response",
                wake_policy: "io-ready",
                transport_model: Some("client-session"),
                request_shape: Some("packetized-exchange"),
                response_shape: Some("completion-callback"),
                dispatch_shape: None,
                memory_binding: None,
                resource_binding: None,
                completion_model: None,
            },
            FallbackHostBridgeFacet {
                host_ffi_surface: &["socket", "urlsession"],
                handle_family: &["network.request", "network.response"],
                phase_bind_inputs: &["request.packet", "bridge.config", "host.session"],
                phase_bind_outputs: &["session.handle", "request.handle"],
                phase_submit_inputs: &["session.handle", "request.handle", "request.packet"],
                phase_submit_outputs: &["inflight.request", "poll.token"],
                phase_wait_inputs: &["poll.token", "callback.slot"],
                phase_wait_outputs: &["response.packet", "completion.signal"],
                phase_finalize_inputs: &["response.packet", "completion.signal"],
                phase_finalize_outputs: &["result.value", "scheduler.wake"],
                phase_bind_wake: "bind-ready",
                phase_submit_wake: "submit-ready",
                phase_wait_wake: "io-ready",
                phase_finalize_wake: "result-commit",
            },
        )),
        "kernel" => Some(fallback_preset(
            FallbackBridgeFlowFacet {
                lane_policy: "dispatch-lanes.accelerator-bound",
                bridge_surface: "host-ffi.bridge.hetero",
                bridge_entry: "nuis.kernel.bridge.dispatch.v1",
                scheduler_binding: "hetero-submit-bridge",
                phase_bind: "buffer-and-argument-bind",
                phase_submit: "queue-dispatch-submit",
                phase_wait: "fence-await-or-poll",
                phase_finalize: "result-commit-and-release",
            },
            FallbackBackendFacet {
                stub_kind: "kernel-dispatch",
                submission_mode: "accelerator-dispatch",
                wake_policy: "completion-fence",
                transport_model: None,
                request_shape: None,
                response_shape: None,
                dispatch_shape: Some("grid-launch"),
                memory_binding: Some("buffer-table"),
                resource_binding: None,
                completion_model: Some("device-fence"),
            },
            FallbackHostBridgeFacet {
                host_ffi_surface: &["buffer", "queue", "fence"],
                handle_family: &["kernel.buffer", "kernel.dispatch"],
                phase_bind_inputs: &["kernel.args", "buffer.table", "dispatch.config"],
                phase_bind_outputs: &["bound.buffer.table", "dispatch.handle"],
                phase_submit_inputs: &["dispatch.handle", "bound.buffer.table", "queue.slot"],
                phase_submit_outputs: &["inflight.dispatch", "fence.handle"],
                phase_wait_inputs: &["fence.handle", "poll.token"],
                phase_wait_outputs: &["completion.state", "result.buffer"],
                phase_finalize_inputs: &["completion.state", "result.buffer"],
                phase_finalize_outputs: &["result.value", "resource.release"],
                phase_bind_wake: "bind-ready",
                phase_submit_wake: "submit-ready",
                phase_wait_wake: "completion-fence",
                phase_finalize_wake: "result-commit",
            },
        )),
        "shader" => Some(fallback_preset(
            FallbackBridgeFlowFacet {
                lane_policy: "dispatch-lanes.render-bound",
                bridge_surface: "host-ffi.bridge.hetero",
                bridge_entry: "nuis.shader.bridge.dispatch.v1",
                scheduler_binding: "render-submit-bridge",
                phase_bind: "pipeline-and-resource-bind",
                phase_submit: "draw-or-dispatch-encode",
                phase_wait: "frame-fence-await",
                phase_finalize: "present-or-signal",
            },
            FallbackBackendFacet {
                stub_kind: "shader-dispatch",
                submission_mode: "frame-graph-dispatch",
                wake_policy: "frame-present",
                transport_model: None,
                request_shape: None,
                response_shape: None,
                dispatch_shape: Some("render-pass"),
                memory_binding: None,
                resource_binding: Some("pipeline-layout"),
                completion_model: Some("frame-fence"),
            },
            FallbackHostBridgeFacet {
                host_ffi_surface: &["pipeline", "resource", "fence"],
                handle_family: &["shader.pipeline", "shader.dispatch"],
                phase_bind_inputs: &["pipeline.layout", "resource.table", "frame.config"],
                phase_bind_outputs: &["bound.pipeline", "dispatch.handle"],
                phase_submit_inputs: &["dispatch.handle", "bound.pipeline", "encoder.slot"],
                phase_submit_outputs: &["inflight.frame", "frame.fence"],
                phase_wait_inputs: &["frame.fence", "present.slot"],
                phase_wait_outputs: &["frame.state", "present.signal"],
                phase_finalize_inputs: &["frame.state", "present.signal"],
                phase_finalize_outputs: &["present.result", "scheduler.wake"],
                phase_bind_wake: "bind-ready",
                phase_submit_wake: "submit-ready",
                phase_wait_wake: "frame-present",
                phase_finalize_wake: "present-commit",
            },
        )),
        "host" | "cpu" => Some(fallback_preset(
            FallbackBridgeFlowFacet {
                lane_policy: "dispatch-lanes.host-bound",
                bridge_surface: "host-ffi.bridge.none",
                bridge_entry: "nuis.host.bridge.dispatch.v1",
                scheduler_binding: "host-inline",
                phase_bind: "direct-bind",
                phase_submit: "direct-call",
                phase_wait: "immediate",
                phase_finalize: "return-and-finish",
            },
            FallbackBackendFacet {
                stub_kind: "host-fallback",
                submission_mode: "direct-call",
                wake_policy: "immediate",
                transport_model: None,
                request_shape: None,
                response_shape: None,
                dispatch_shape: None,
                memory_binding: None,
                resource_binding: None,
                completion_model: None,
            },
            FallbackHostBridgeFacet {
                host_ffi_surface: &["host-inline"],
                handle_family: &["host.inline"],
                phase_bind_inputs: &["call.args"],
                phase_bind_outputs: &["bound.call"],
                phase_submit_inputs: &["bound.call"],
                phase_submit_outputs: &["inflight.call"],
                phase_wait_inputs: &["inflight.call"],
                phase_wait_outputs: &["call.result"],
                phase_finalize_inputs: &["call.result"],
                phase_finalize_outputs: &["result.value"],
                phase_bind_wake: "bind-ready",
                phase_submit_wake: "submit-ready",
                phase_wait_wake: "immediate",
                phase_finalize_wake: "return",
            },
        )),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarSchedulerSummary {
    pub contract_stack: String,
    pub clock: NustarClockSummary,
    pub result_roles: String,
    pub sample_navigation: Option<String>,
    pub result_samples: Option<String>,
    pub transport_samples: Option<String>,
    pub summary_api: String,
    pub summary_samples: Option<String>,
    pub observer_classes: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarStdNetSummary {
    pub sample_navigation: Option<String>,
    pub recipe_samples: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarDomainContract {
    pub contract_schema: String,
    pub contract_groups: Vec<String>,
    pub extension_groups: Vec<String>,
    pub package_id: String,
    pub domain_family: String,
    pub frontend: String,
    pub loader_abi: String,
    pub loader_entry: String,
    pub machine_abi_policy: String,
    pub abi_profiles: Vec<String>,
    pub host_ffi_surface: Vec<String>,
    pub host_ffi_abis: Vec<String>,
    pub host_ffi_bridge: Option<String>,
    pub capability: NustarCapabilitySummary,
    pub execution: NustarExecutionSummary,
    pub scheduler: NustarSchedulerSummary,
    pub std_net: NustarStdNetSummary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarDomainRegistration {
    pub manifest_path: String,
    pub package_id: String,
    pub domain_family: String,
    pub frontend: String,
    pub entry_crate: String,
    pub ast_entry: String,
    pub nir_entry: String,
    pub yir_lowering_entry: String,
    pub part_verify_entry: String,
    pub ast_surface: Vec<String>,
    pub nir_surface: Vec<String>,
    pub yir_lowering: Vec<String>,
    pub part_verify: Vec<String>,
    pub resource_families: Vec<String>,
    pub unit_types: Vec<String>,
    pub lowering_targets: Vec<String>,
    pub ops: Vec<String>,
    pub contract: NustarDomainContract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NustarRegistryIssueKind {
    IndexEmpty,
    DuplicatePackageId,
    DuplicateDomainFamily,
    PackageIdentityMismatch,
    DomainFamilyMismatch,
    ManifestSchemaMismatch,
    LoaderContractMismatch,
    ResourceFamilyContractMismatch,
    OpContractMismatch,
    LaneContractMismatch,
    PackagingContractMismatch,
    DomainContractMismatch,
}

impl NustarRegistryIssueKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IndexEmpty => "index_empty",
            Self::DuplicatePackageId => "duplicate_package_id",
            Self::DuplicateDomainFamily => "duplicate_domain_family",
            Self::PackageIdentityMismatch => "package_identity_mismatch",
            Self::DomainFamilyMismatch => "domain_family_mismatch",
            Self::ManifestSchemaMismatch => "manifest_schema_mismatch",
            Self::LoaderContractMismatch => "loader_contract_mismatch",
            Self::ResourceFamilyContractMismatch => "resource_family_contract_mismatch",
            Self::OpContractMismatch => "op_contract_mismatch",
            Self::LaneContractMismatch => "lane_contract_mismatch",
            Self::PackagingContractMismatch => "packaging_contract_mismatch",
            Self::DomainContractMismatch => "domain_contract_mismatch",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::IndexEmpty => "NRV001",
            Self::DuplicatePackageId => "NRV002",
            Self::DuplicateDomainFamily => "NRV003",
            Self::PackageIdentityMismatch => "NRV004",
            Self::DomainFamilyMismatch => "NRV005",
            Self::ManifestSchemaMismatch => "NRV006",
            Self::LoaderContractMismatch => "NRV007",
            Self::ResourceFamilyContractMismatch => "NRV008",
            Self::OpContractMismatch => "NRV009",
            Self::LaneContractMismatch => "NRV010",
            Self::PackagingContractMismatch => "NRV011",
            Self::DomainContractMismatch => "NRV012",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NustarRegistryIssue {
    pub kind: NustarRegistryIssueKind,
    pub package: Option<String>,
    pub domain: Option<String>,
    pub manifest_path: Option<String>,
    pub message: String,
}

impl NustarRegistryIssue {
    pub fn summary(&self) -> String {
        let package = self.package.as_deref().unwrap_or("<none>");
        let domain = self.domain.as_deref().unwrap_or("<none>");
        format!(
            "{} {} package={} domain={}: {}",
            self.kind.code(),
            self.kind.as_str(),
            package,
            domain,
            self.message
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectDomainRegistryIssueKind {
    DomainNotRegistered,
    ContractSchemaMismatch,
    AbiNotRegistered,
    ExecutionContractMismatch,
}

impl ProjectDomainRegistryIssueKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DomainNotRegistered => "domain_not_registered",
            Self::ContractSchemaMismatch => "contract_schema_mismatch",
            Self::AbiNotRegistered => "abi_not_registered",
            Self::ExecutionContractMismatch => "execution_contract_mismatch",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::DomainNotRegistered => "NRG001",
            Self::ContractSchemaMismatch => "NRG002",
            Self::AbiNotRegistered => "NRG003",
            Self::ExecutionContractMismatch => "NRG004",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDomainRegistryIssue {
    pub kind: ProjectDomainRegistryIssueKind,
    pub message: String,
}

impl ProjectDomainRegistryIssue {
    pub fn summary(&self) -> String {
        format!(
            "{} {}: {}",
            self.kind.code(),
            self.kind.as_str(),
            self.message
        )
    }
}

pub fn project_domain_registry_issue_json(issue: &ProjectDomainRegistryIssue) -> String {
    format!(
        "{{{},{},{}}}",
        json_field("code", issue.kind.code()),
        json_field("kind", issue.kind.as_str()),
        json_field("message", &issue.message)
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDomainRegistryCheck {
    pub domain: String,
    pub package: Option<String>,
    pub contract_schema: Option<String>,
    pub abi: Option<String>,
    pub abi_registered: bool,
    pub ok: bool,
    pub issues: Vec<ProjectDomainRegistryIssue>,
}

impl ProjectDomainRegistryCheck {
    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }

    pub fn summary_line(&self) -> String {
        format!(
            "{} (package={}, abi={}): {}",
            self.domain,
            self.package.as_deref().unwrap_or("<missing>"),
            self.abi.as_deref().unwrap_or("<none>"),
            if self.issues.is_empty() {
                "ok".to_owned()
            } else {
                self.issues
                    .iter()
                    .map(ProjectDomainRegistryIssue::summary)
                    .collect::<Vec<_>>()
                    .join("; ")
            }
        )
    }
}

pub fn project_domain_registry_check_json(check: &ProjectDomainRegistryCheck) -> String {
    let issue_json = check
        .issues
        .iter()
        .map(project_domain_registry_issue_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{{},{},{},{},{},{},{}}}",
        json_field("domain", &check.domain),
        json_optional_string_field("package", check.package.as_deref()),
        json_optional_string_field("contract_schema", check.contract_schema.as_deref()),
        json_optional_string_field("abi", check.abi.as_deref()),
        json_bool_field("abi_registered", check.abi_registered),
        json_bool_field("ok", check.ok),
        format!("\"issues\":[{}]", issue_json)
    )
}

pub fn render_project_domain_registry_check_lines(
    check: &ProjectDomainRegistryCheck,
) -> Vec<String> {
    let mut out = String::new();
    write_project_domain_registry_check_lines(&mut out, check)
        .expect("writing project domain registry check lines to String should not fail");
    out.lines().map(str::to_owned).collect()
}

pub fn write_project_domain_registry_check_lines<W: fmt::Write>(
    out: &mut W,
    check: &ProjectDomainRegistryCheck,
) -> fmt::Result {
    writeln!(
        out,
        "registry: {} package={} schema={} abi={} ok={} abi_registered={} issues={}",
        check.domain,
        check.package.as_deref().unwrap_or("<missing>"),
        check.contract_schema.as_deref().unwrap_or("<missing>"),
        check.abi.as_deref().unwrap_or("<none>"),
        if check.ok { "yes" } else { "no" },
        if check.abi_registered { "yes" } else { "no" },
        check.issue_count()
    )?;
    for issue in &check.issues {
        writeln!(
            out,
            "registry_issue: {} {} {}",
            issue.kind.code(),
            issue.kind.as_str(),
            issue.message
        )?;
    }
    Ok(())
}

pub fn domain_contract(manifest: &NustarPackageManifest) -> NustarDomainContract {
    let mut contract_groups = vec![
        NUSTAR_DOMAIN_CONTRACT_GROUP_PACKAGE_IDENTITY.to_owned(),
        NUSTAR_DOMAIN_CONTRACT_GROUP_LOADER.to_owned(),
        NUSTAR_DOMAIN_CONTRACT_GROUP_ABI.to_owned(),
        NUSTAR_DOMAIN_CONTRACT_GROUP_RUNTIME.to_owned(),
        NUSTAR_DOMAIN_CONTRACT_GROUP_EXECUTION.to_owned(),
        NUSTAR_DOMAIN_CONTRACT_GROUP_SCHEDULER.to_owned(),
    ];
    if !manifest.host_ffi_surface.is_empty() {
        contract_groups.push(NUSTAR_DOMAIN_CONTRACT_GROUP_HOST_BRIDGE.to_owned());
    }
    let mut extension_groups = Vec::new();
    if manifest.domain_family == "network" {
        extension_groups.push(NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET.to_owned());
    }
    NustarDomainContract {
        contract_schema: NUSTAR_DOMAIN_CONTRACT_SCHEMA.to_owned(),
        contract_groups,
        extension_groups,
        package_id: manifest.package_id.clone(),
        domain_family: manifest.domain_family.clone(),
        frontend: manifest.frontend.clone(),
        loader_abi: manifest.loader_abi.clone(),
        loader_entry: manifest.loader_entry.clone(),
        machine_abi_policy: manifest.machine_abi_policy.clone(),
        abi_profiles: manifest.abi_profiles.clone(),
        host_ffi_surface: manifest.host_ffi_surface.clone(),
        host_ffi_abis: manifest.host_ffi_abis.clone(),
        host_ffi_bridge: if manifest.host_ffi_surface.is_empty() {
            None
        } else {
            Some(manifest.host_ffi_bridge.clone())
        },
        capability: capability_summary(manifest),
        execution: execution_summary(manifest),
        scheduler: scheduler_summary(manifest),
        std_net: std_net_summary(&manifest.domain_family),
    }
}

pub fn load_domain_contract_for_domain(
    root: &Path,
    domain_family: &str,
) -> Result<NustarDomainContract, String> {
    let manifest = load_manifest_for_domain(root, domain_family)?;
    Ok(domain_contract(&manifest))
}

pub fn domain_registration(
    root: &Path,
    entry: &NustarPackageIndexEntry,
) -> Result<NustarDomainRegistration, String> {
    let root = resolve_registry_root(root);
    let path = manifest_path(&root, entry);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    let manifest = parse_manifest(&source, &path)?;
    Ok(NustarDomainRegistration {
        manifest_path: path.display().to_string(),
        package_id: manifest.package_id.clone(),
        domain_family: manifest.domain_family.clone(),
        frontend: manifest.frontend.clone(),
        entry_crate: manifest.entry_crate.clone(),
        ast_entry: manifest.ast_entry.clone(),
        nir_entry: manifest.nir_entry.clone(),
        yir_lowering_entry: manifest.yir_lowering_entry.clone(),
        part_verify_entry: manifest.part_verify_entry.clone(),
        ast_surface: manifest.ast_surface.clone(),
        nir_surface: manifest.nir_surface.clone(),
        yir_lowering: manifest.yir_lowering.clone(),
        part_verify: manifest.part_verify.clone(),
        resource_families: manifest.resource_families.clone(),
        unit_types: manifest.unit_types.clone(),
        lowering_targets: manifest.lowering_targets.clone(),
        ops: manifest.ops.clone(),
        contract: domain_contract(&manifest),
    })
}

fn load_registered_domains_unvalidated(
    root: &Path,
) -> Result<Vec<NustarDomainRegistration>, String> {
    let root = resolve_registry_root(root);
    let mut registrations = load_index(&root)?
        .into_iter()
        .map(|entry| domain_registration(&root, &entry))
        .collect::<Result<Vec<_>, _>>()?;
    registrations.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(registrations)
}

fn lane_target_from_entry(entry: &str) -> Option<&str> {
    let (target, _) = entry.split_once('=')?;
    let target = target.trim();
    if target.is_empty() {
        None
    } else {
        Some(target)
    }
}

fn lane_target_is_declared(manifest: &NustarPackageManifest, target: &str) -> bool {
    if manifest.ops.iter().any(|op| op == target) {
        return true;
    }
    let prefix = format!("{}.", manifest.domain_family);
    let Some(slot) = target.strip_prefix(&prefix) else {
        return false;
    };
    manifest
        .support_profile_slots
        .iter()
        .any(|candidate| candidate == slot)
}

fn parse_backend_family_from_abi_target(raw: &str) -> Option<String> {
    let (_, fields) = raw.split_once(':')?;
    for field in fields
        .split('|')
        .map(str::trim)
        .filter(|field| !field.is_empty())
    {
        let (key, value) = field.split_once('=')?;
        if key.trim() == "backend" {
            return Some(value.trim().to_owned());
        }
    }
    None
}

fn validate_shader_domain_contract(
    manifest: &NustarPackageManifest,
    manifest_path: &Path,
) -> Vec<NustarRegistryIssue> {
    let mut issues = Vec::new();
    let lowering = manifest
        .lowering_targets
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let backend_families = manifest
        .abi_targets
        .iter()
        .filter_map(|raw| parse_backend_family_from_abi_target(raw))
        .collect::<BTreeSet<_>>();
    let missing_lowering = backend_families
        .iter()
        .filter(|backend| !lowering.contains(*backend))
        .cloned()
        .collect::<Vec<_>>();
    if !missing_lowering.is_empty() {
        issues.push(NustarRegistryIssue {
            kind: NustarRegistryIssueKind::DomainContractMismatch,
            package: Some(manifest.package_id.clone()),
            domain: Some(manifest.domain_family.clone()),
            manifest_path: Some(manifest_path.display().to_string()),
            message: format!(
                "shader ABI backends must be represented in lowering_targets; missing: {}",
                missing_lowering.join(", ")
            ),
        });
    }
    if lowering.contains("metal")
        && !manifest
            .support_surface
            .iter()
            .any(|surface| surface == "shader.inline.wgsl.v1")
    {
        issues.push(NustarRegistryIssue {
            kind: NustarRegistryIssueKind::DomainContractMismatch,
            package: Some(manifest.package_id.clone()),
            domain: Some(manifest.domain_family.clone()),
            manifest_path: Some(manifest_path.display().to_string()),
            message:
                "shader lowering_targets containing `metal` must expose `shader.inline.wgsl.v1`"
                    .to_owned(),
        });
    }
    if !manifest
        .support_profile_slots
        .iter()
        .any(|slot| slot == "target")
        || !manifest
            .support_profile_slots
            .iter()
            .any(|slot| slot == "viewport")
        || !manifest
            .support_profile_slots
            .iter()
            .any(|slot| slot == "pipeline")
    {
        issues.push(NustarRegistryIssue {
            kind: NustarRegistryIssueKind::DomainContractMismatch,
            package: Some(manifest.package_id.clone()),
            domain: Some(manifest.domain_family.clone()),
            manifest_path: Some(manifest_path.display().to_string()),
            message: "shader domain must expose target/viewport/pipeline support_profile_slots"
                .to_owned(),
        });
    }
    issues
}

fn validate_kernel_domain_contract(
    manifest: &NustarPackageManifest,
    manifest_path: &Path,
) -> Vec<NustarRegistryIssue> {
    let mut issues = Vec::new();
    let lowering = manifest
        .lowering_targets
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let backend_families = manifest
        .abi_targets
        .iter()
        .filter_map(|raw| parse_backend_family_from_abi_target(raw))
        .collect::<BTreeSet<_>>();
    let missing_lowering = backend_families
        .iter()
        .filter(|backend| !lowering.contains(*backend))
        .cloned()
        .collect::<Vec<_>>();
    if !missing_lowering.is_empty() {
        issues.push(NustarRegistryIssue {
            kind: NustarRegistryIssueKind::DomainContractMismatch,
            package: Some(manifest.package_id.clone()),
            domain: Some(manifest.domain_family.clone()),
            manifest_path: Some(manifest_path.display().to_string()),
            message: format!(
                "kernel ABI backends must be represented in lowering_targets; missing: {}",
                missing_lowering.join(", ")
            ),
        });
    }
    for required_slot in ["bind_core", "queue_depth", "batch_lanes", "entry"] {
        if !manifest
            .support_profile_slots
            .iter()
            .any(|slot| slot == required_slot)
        {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DomainContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "kernel domain must expose `{}` in support_profile_slots",
                    required_slot
                ),
            });
        }
    }
    if lowering.contains("coreml") || lowering.contains("ane") {
        let has_apple_resource = manifest
            .resource_families
            .iter()
            .any(|family| family.starts_with("kernel.apple") || family.starts_with("npu.apple"));
        if !has_apple_resource {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DomainContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message:
                    "kernel lowering_targets containing `coreml` or `ane` must expose apple/npu resource families"
                        .to_owned(),
            });
        }
    }
    issues
}

fn validate_network_domain_contract(
    manifest: &NustarPackageManifest,
    manifest_path: &Path,
) -> Vec<NustarRegistryIssue> {
    let mut issues = Vec::new();
    let lowering = manifest
        .lowering_targets
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for required_target in ["socket-abi", "urlsession", "winsock"] {
        if !lowering.contains(required_target) {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DomainContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "network domain must keep `{}` in lowering_targets",
                    required_target
                ),
            });
        }
    }
    for required_surface in [
        "network.profile.connect.v1",
        "network.profile.accept.v1",
        "network.profile.send.v1",
        "network.profile.recv.v1",
        "network.profile.close.v1",
        "network.profile.protocol.v1",
    ] {
        if !manifest
            .support_surface
            .iter()
            .any(|surface| surface == required_surface)
        {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DomainContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "network domain must expose `{}` in support_surface",
                    required_surface
                ),
            });
        }
    }
    for required_slot in [
        "endpoint_kind",
        "transport_family",
        "protocol_kind",
        "protocol_version",
        "protocol_header_bytes",
    ] {
        if !manifest
            .support_profile_slots
            .iter()
            .any(|slot| slot == required_slot)
        {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DomainContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "network domain must expose `{}` in support_profile_slots",
                    required_slot
                ),
            });
        }
    }
    for required_op in [
        "network.connect",
        "network.accept",
        "network.send",
        "network.recv",
        "network.close",
        "network.poll",
    ] {
        if !manifest.ops.iter().any(|op| op == required_op) {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DomainContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!("network domain must expose `{}` in ops", required_op),
            });
        }
    }
    issues
}

fn validate_build_contract_fields(
    manifest: &NustarPackageManifest,
    manifest_path: &Path,
) -> Vec<NustarRegistryIssue> {
    let mut issues = Vec::new();
    let has_bridge_contract = manifest.bridge_lane_policy.is_some()
        || manifest.bridge_surface.is_some()
        || manifest.bridge_emission_kind.is_some()
        || manifest.bridge_entry.is_some()
        || manifest.bridge_kind.is_some()
        || manifest.bridge_scheduler_binding.is_some()
        || manifest.backend_stub_kind.is_some()
        || manifest.backend_submission_mode.is_some()
        || manifest.backend_wake_policy.is_some()
        || manifest.backend_transport_model.is_some()
        || manifest.backend_request_shape.is_some()
        || manifest.backend_response_shape.is_some()
        || manifest.backend_dispatch_shape.is_some()
        || manifest.backend_memory_binding.is_some()
        || manifest.backend_resource_binding.is_some()
        || manifest.backend_completion_model.is_some()
        || manifest.phase_bind.is_some()
        || manifest.phase_submit.is_some()
        || manifest.phase_wait.is_some()
        || manifest.phase_finalize.is_some();
    if has_bridge_contract {
        let mut missing = Vec::new();
        for (name, value) in [
            ("bridge_lane_policy", manifest.bridge_lane_policy.as_deref()),
            ("bridge_surface", manifest.bridge_surface.as_deref()),
            (
                "bridge_emission_kind",
                manifest.bridge_emission_kind.as_deref(),
            ),
            ("bridge_entry", manifest.bridge_entry.as_deref()),
            ("bridge_kind", manifest.bridge_kind.as_deref()),
            (
                "bridge_scheduler_binding",
                manifest.bridge_scheduler_binding.as_deref(),
            ),
            ("backend_stub_kind", manifest.backend_stub_kind.as_deref()),
            (
                "backend_submission_mode",
                manifest.backend_submission_mode.as_deref(),
            ),
            (
                "backend_wake_policy",
                manifest.backend_wake_policy.as_deref(),
            ),
            ("phase_bind", manifest.phase_bind.as_deref()),
            ("phase_submit", manifest.phase_submit.as_deref()),
            ("phase_wait", manifest.phase_wait.as_deref()),
            ("phase_finalize", manifest.phase_finalize.as_deref()),
        ] {
            if value.is_none() {
                missing.push(name.to_owned());
            }
        }
        if !missing.is_empty() {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DomainContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "bridge/build contract must declare a complete minimum skeleton; missing: {}",
                    missing.join(", ")
                ),
            });
        }
    }

    let has_host_bridge_contract = manifest.host_bridge_host_ffi_surface.is_some()
        || manifest.host_bridge_handle_family.is_some()
        || manifest.host_bridge_phase_order.is_some()
        || manifest.host_bridge_phase_bind_inputs.is_some()
        || manifest.host_bridge_phase_bind_outputs.is_some()
        || manifest.host_bridge_phase_submit_inputs.is_some()
        || manifest.host_bridge_phase_submit_outputs.is_some()
        || manifest.host_bridge_phase_wait_inputs.is_some()
        || manifest.host_bridge_phase_wait_outputs.is_some()
        || manifest.host_bridge_phase_finalize_inputs.is_some()
        || manifest.host_bridge_phase_finalize_outputs.is_some()
        || manifest.host_bridge_phase_bind_wake.is_some()
        || manifest.host_bridge_phase_submit_wake.is_some()
        || manifest.host_bridge_phase_wait_wake.is_some()
        || manifest.host_bridge_phase_finalize_wake.is_some()
        || manifest.host_bridge_plan_begin.is_some()
        || manifest.host_bridge_plan_end.is_some();
    if has_host_bridge_contract {
        let mut missing = Vec::new();
        for (name, present) in [
            (
                "host_bridge_host_ffi_surface",
                manifest.host_bridge_host_ffi_surface.is_some(),
            ),
            (
                "host_bridge_handle_family",
                manifest.host_bridge_handle_family.is_some(),
            ),
            (
                "host_bridge_phase_order",
                manifest.host_bridge_phase_order.is_some(),
            ),
            (
                "host_bridge_phase_bind_inputs",
                manifest.host_bridge_phase_bind_inputs.is_some(),
            ),
            (
                "host_bridge_phase_bind_outputs",
                manifest.host_bridge_phase_bind_outputs.is_some(),
            ),
            (
                "host_bridge_phase_submit_inputs",
                manifest.host_bridge_phase_submit_inputs.is_some(),
            ),
            (
                "host_bridge_phase_submit_outputs",
                manifest.host_bridge_phase_submit_outputs.is_some(),
            ),
            (
                "host_bridge_phase_wait_inputs",
                manifest.host_bridge_phase_wait_inputs.is_some(),
            ),
            (
                "host_bridge_phase_wait_outputs",
                manifest.host_bridge_phase_wait_outputs.is_some(),
            ),
            (
                "host_bridge_phase_finalize_inputs",
                manifest.host_bridge_phase_finalize_inputs.is_some(),
            ),
            (
                "host_bridge_phase_finalize_outputs",
                manifest.host_bridge_phase_finalize_outputs.is_some(),
            ),
            (
                "host_bridge_phase_bind_wake",
                manifest.host_bridge_phase_bind_wake.is_some(),
            ),
            (
                "host_bridge_phase_submit_wake",
                manifest.host_bridge_phase_submit_wake.is_some(),
            ),
            (
                "host_bridge_phase_wait_wake",
                manifest.host_bridge_phase_wait_wake.is_some(),
            ),
            (
                "host_bridge_phase_finalize_wake",
                manifest.host_bridge_phase_finalize_wake.is_some(),
            ),
            (
                "host_bridge_plan_begin",
                manifest.host_bridge_plan_begin.is_some(),
            ),
            (
                "host_bridge_plan_end",
                manifest.host_bridge_plan_end.is_some(),
            ),
        ] {
            if !present {
                missing.push(name.to_owned());
            }
        }
        if !missing.is_empty() {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DomainContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "host bridge contract must declare every phase field; missing: {}",
                    missing.join(", ")
                ),
            });
        } else {
            if manifest
                .host_bridge_host_ffi_surface
                .as_ref()
                .is_some_and(Vec::is_empty)
            {
                issues.push(NustarRegistryIssue {
                    kind: NustarRegistryIssueKind::DomainContractMismatch,
                    package: Some(manifest.package_id.clone()),
                    domain: Some(manifest.domain_family.clone()),
                    manifest_path: Some(manifest_path.display().to_string()),
                    message: "host bridge contract must expose at least one host_ffi surface"
                        .to_owned(),
                });
            }
            if manifest
                .host_bridge_handle_family
                .as_ref()
                .is_some_and(Vec::is_empty)
            {
                issues.push(NustarRegistryIssue {
                    kind: NustarRegistryIssueKind::DomainContractMismatch,
                    package: Some(manifest.package_id.clone()),
                    domain: Some(manifest.domain_family.clone()),
                    manifest_path: Some(manifest_path.display().to_string()),
                    message: "host bridge contract must expose at least one handle family"
                        .to_owned(),
                });
            }
            if manifest.host_bridge_phase_order.as_deref()
                != Some(
                    &[
                        "bind".to_owned(),
                        "submit".to_owned(),
                        "wait".to_owned(),
                        "finalize".to_owned(),
                    ][..],
                )
            {
                issues.push(NustarRegistryIssue {
                    kind: NustarRegistryIssueKind::DomainContractMismatch,
                    package: Some(manifest.package_id.clone()),
                    domain: Some(manifest.domain_family.clone()),
                    manifest_path: Some(manifest_path.display().to_string()),
                    message:
                        "host bridge phase_order must be [\"bind\", \"submit\", \"wait\", \"finalize\"]"
                            .to_owned(),
                });
            }
        }
    }

    issues
}

fn validate_domain_specific_contracts(
    manifest: &NustarPackageManifest,
    manifest_path: &Path,
) -> Vec<NustarRegistryIssue> {
    match manifest.domain_family.as_str() {
        "shader" => validate_shader_domain_contract(manifest, manifest_path),
        "kernel" => validate_kernel_domain_contract(manifest, manifest_path),
        "network" => validate_network_domain_contract(manifest, manifest_path),
        _ => Vec::new(),
    }
}

pub fn validate_registered_domains(root: &Path) -> Result<Vec<NustarRegistryIssue>, String> {
    let root = resolve_registry_root(root);
    let index = load_index(&root)?;
    if index.is_empty() {
        return Ok(vec![NustarRegistryIssue {
            kind: NustarRegistryIssueKind::IndexEmpty,
            package: None,
            domain: None,
            manifest_path: Some(root.join(INDEX_FILE).display().to_string()),
            message: format!(
                "no nustar packages are indexed in `{}`",
                root.join(INDEX_FILE).display()
            ),
        }]);
    }

    let mut issues = Vec::new();
    let mut seen_packages = BTreeSet::new();
    for entry in &index {
        let manifest_path = manifest_path(&root, entry);
        if !seen_packages.insert(entry.package_id.clone()) {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DuplicatePackageId,
                package: Some(entry.package_id.clone()),
                domain: Some(entry.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "package `{}` appears more than once in `{}`",
                    entry.package_id,
                    root.join(INDEX_FILE).display()
                ),
            });
        }

        let source = fs::read_to_string(&manifest_path)
            .map_err(|error| format!("failed to read `{}`: {error}", manifest_path.display()))?;
        let manifest = parse_manifest(&source, &manifest_path)?;

        if manifest.package_id != entry.package_id {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::PackageIdentityMismatch,
                package: Some(entry.package_id.clone()),
                domain: Some(entry.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "index package `{}` does not match manifest package `{}`",
                    entry.package_id, manifest.package_id
                ),
            });
        }
        if manifest.domain_family != entry.domain_family {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::DomainFamilyMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(entry.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "index domain `{}` does not match manifest domain `{}`",
                    entry.domain_family, manifest.domain_family
                ),
            });
        }
        if manifest.manifest_schema != "nustar-manifest-v1" {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::ManifestSchemaMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "manifest schema `{}` is not supported; expected `nustar-manifest-v1`",
                    manifest.manifest_schema
                ),
            });
        }
        if manifest.loader_abi != "nustar-loader-v1"
            || manifest.loader_entry != "nustar.bootstrap.v1"
        {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::LoaderContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "loader contract must be `nustar-loader-v1` + `nustar.bootstrap.v1`, got abi=`{}` entry=`{}`",
                    manifest.loader_abi, manifest.loader_entry
                ),
            });
        }
        if !manifest
            .resource_families
            .iter()
            .any(|family| family == &manifest.domain_family)
        {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::ResourceFamilyContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "resource_families must include the owning domain `{}`",
                    manifest.domain_family
                ),
            });
        }
        let op_prefix = format!("{}.", manifest.domain_family);
        let invalid_ops = manifest
            .ops
            .iter()
            .filter(|op| !op.starts_with(&op_prefix))
            .cloned()
            .collect::<Vec<_>>();
        if !invalid_ops.is_empty() {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::OpContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "ops must stay inside domain prefix `{}`; invalid ops: {}",
                    op_prefix,
                    invalid_ops.join(", ")
                ),
            });
        }
        let invalid_lane_targets = manifest
            .default_lanes
            .iter()
            .filter_map(|entry| {
                let target = lane_target_from_entry(entry)?;
                if lane_target_is_declared(&manifest, target) {
                    None
                } else {
                    Some(target.to_owned())
                }
            })
            .collect::<Vec<_>>();
        if !invalid_lane_targets.is_empty() {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::LaneContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: format!(
                    "default_lanes reference undeclared ops: {}",
                    invalid_lane_targets.join(", ")
                ),
            });
        }
        if let Err(error) = crate::nustar_binary::validate_manifest_for_packaging(&manifest) {
            issues.push(NustarRegistryIssue {
                kind: NustarRegistryIssueKind::PackagingContractMismatch,
                package: Some(manifest.package_id.clone()),
                domain: Some(manifest.domain_family.clone()),
                manifest_path: Some(manifest_path.display().to_string()),
                message: error,
            });
        }
        issues.extend(validate_build_contract_fields(&manifest, &manifest_path));
        issues.extend(validate_domain_specific_contracts(
            &manifest,
            &manifest_path,
        ));
    }

    Ok(issues)
}

pub fn ensure_registered_domains_valid(root: &Path) -> Result<(), String> {
    let issues = validate_registered_domains(root)?;
    if issues.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "nustar registry validation failed:\n{}",
            issues
                .iter()
                .map(NustarRegistryIssue::summary)
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }
}

pub fn load_registered_domains(root: &Path) -> Result<Vec<NustarDomainRegistration>, String> {
    ensure_registered_domains_valid(root)?;
    load_registered_domains_unvalidated(root)
}

pub fn load_domain_registration_for_domain(
    root: &Path,
    domain_family: &str,
) -> Result<NustarDomainRegistration, String> {
    let root = resolve_registry_root(root);
    let entry = load_index(&root)?
        .into_iter()
        .find(|entry| entry.domain_family == domain_family)
        .ok_or_else(|| {
            format!(
                "no nustar package is indexed for mod domain `{domain_family}` in `{}`",
                root.join(INDEX_FILE).display()
            )
        })?;
    domain_registration(&root, &entry)
}

pub fn validate_project_domain_registry(
    plan: &crate::project::ProjectCompilationPlan,
) -> Vec<ProjectDomainRegistryCheck> {
    plan.abi_resolution
        .requirements
        .iter()
        .map(|item| {
            let mut issues = Vec::new();
            let mut package = None;
            let mut contract_schema = None;
            let mut abi_registered = false;
            match load_domain_registration_for_domain(Path::new("nustar-packages"), &item.domain) {
                Ok(registration) => {
                    package = Some(registration.package_id.clone());
                    contract_schema = Some(registration.contract.contract_schema.clone());
                    if registration.contract.contract_schema != NUSTAR_DOMAIN_CONTRACT_SCHEMA {
                        issues.push(ProjectDomainRegistryIssue {
                            kind: ProjectDomainRegistryIssueKind::ContractSchemaMismatch,
                            message: format!(
                                "unexpected contract schema `{}`",
                                registration.contract.contract_schema
                            ),
                        });
                    }
                    abi_registered = registration
                        .contract
                        .abi_profiles
                        .iter()
                        .any(|candidate| candidate == &item.abi);
                    if !abi_registered {
                        issues.push(ProjectDomainRegistryIssue {
                            kind: ProjectDomainRegistryIssueKind::AbiNotRegistered,
                            message: format!(
                                "abi `{}` is not declared by registered profiles",
                                item.abi
                            ),
                        });
                    }
                    if registration.contract.execution.execution_domain != item.domain {
                        issues.push(ProjectDomainRegistryIssue {
                            kind: ProjectDomainRegistryIssueKind::ExecutionContractMismatch,
                            message: format!(
                                "execution domain `{}` does not match project domain `{}`",
                                registration.contract.execution.execution_domain, item.domain
                            ),
                        });
                    }
                    if registration.contract.execution.contract_family
                        != format!("nustar.{}", item.domain)
                    {
                        issues.push(ProjectDomainRegistryIssue {
                            kind: ProjectDomainRegistryIssueKind::ExecutionContractMismatch,
                            message: format!(
                                "execution contract family `{}` does not match expected `nustar.{}`",
                                registration.contract.execution.contract_family, item.domain
                            ),
                        });
                    }
                    if registration.contract.execution.lowering_targets.is_empty() {
                        issues.push(ProjectDomainRegistryIssue {
                            kind: ProjectDomainRegistryIssueKind::ExecutionContractMismatch,
                            message: "execution skeleton declares no lowering targets".to_owned(),
                        });
                    }
                }
                Err(error) => issues.push(ProjectDomainRegistryIssue {
                    kind: ProjectDomainRegistryIssueKind::DomainNotRegistered,
                    message: error,
                }),
            }
            ProjectDomainRegistryCheck {
                domain: item.domain.clone(),
                package,
                contract_schema,
                abi: Some(item.abi.clone()),
                abi_registered,
                ok: issues.is_empty(),
                issues,
            }
        })
        .collect()
}

pub fn ensure_project_domain_registry_valid(
    plan: &crate::project::ProjectCompilationPlan,
) -> Result<(), String> {
    let checks = validate_project_domain_registry(plan);
    let failures = checks
        .iter()
        .filter(|check| !check.ok)
        .map(ProjectDomainRegistryCheck::summary_line)
        .collect::<Vec<_>>();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "project domain registry validation failed:\n{}",
            failures.join("\n")
        ))
    }
}

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
        json_string_array_field("groups", &contract.contract_groups),
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
        json_string_array_field("contract_groups", &contract.contract_groups),
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

pub fn scheduler_summary(manifest: &NustarPackageManifest) -> NustarSchedulerSummary {
    let domain = manifest.domain_family.as_str();
    NustarSchedulerSummary {
        contract_stack:
            "placement -> timing -> result observation -> async summary observation -> observer classification"
                .to_owned(),
        clock: capability_summary(manifest).clock,
        result_roles: "entry=result-entry, probe=result-ready-probe, value=result-payload-value; variants=config_ready|send_ready|recv_ready|connect_ready|accept_ready|closed".to_owned(),
        sample_navigation: scheduler_sample_navigation(domain).map(str::to_owned),
        result_samples: scheduler_result_samples(domain).map(str::to_owned),
        transport_samples: scheduler_transport_samples(domain).map(str::to_owned),
        summary_api: "policy=async-policy-summary, batch=async-batch-summary, windowed=async-windowed-summary; classes=transport_split|transport_windowed_split|transport_session_bridge_split|control_split|control_windowed|control_session_bridge".to_owned(),
        summary_samples: scheduler_summary_samples(domain).map(str::to_owned),
        observer_classes: "source=profile-backed|result-backed|summary-backed; stage=entry|ready|payload|policy|batch|windowed; scope=local|cross-lane|cross-domain|bridge-visible; branch=primary|secondary|fallback|send|recv".to_owned(),
    }
}

pub fn std_net_summary(domain: &str) -> NustarStdNetSummary {
    NustarStdNetSummary {
        sample_navigation: std_net_sample_navigation(domain).map(str::to_owned),
        recipe_samples: std_net_recipe_samples(domain).map(str::to_owned),
    }
}

fn scheduler_sample_navigation(domain: &str) -> Option<&'static str> {
    match domain {
        "shader" => Some("policy -> windowed"),
        "kernel" => Some("policy -> windowed"),
        "network" => Some(
            "result_ladder -> transport_split_ladder -> transport_summary_ladder -> summary_classes",
        ),
        _ => None,
    }
}

fn scheduler_result_samples(domain: &str) -> Option<&'static str> {
    match domain {
        "network" => Some(
            "result_ladder=network_result_profile_demo -> network_connect_result_demo -> network_accept_result_demo -> network_result_task_policy_demo -> network_result_task_batch_demo -> network_result_task_windowed_batch_demo -> network_result_session_bridge_demo; control_ladder=network_connect_result_demo -> network_accept_result_demo -> network_connect_accept_task_policy_demo -> network_connect_accept_task_batch_demo -> network_connect_accept_task_windowed_batch_demo",
        ),
        _ => None,
    }
}

fn scheduler_transport_samples(domain: &str) -> Option<&'static str> {
    match domain {
        "network" => Some(
            "transport_runtime=network_host_handle_runtime_demo -> network_host_handle_transport_runtime_demo -> network_owned_transport_result_demo -> network_host_transport_runtime_demo -> network_transport_result_demo; transport_split_ladder=network_transport_result_policy_split_demo -> network_transport_result_batch_split_demo -> network_transport_result_windowed_split_demo -> network_transport_result_session_bridge_split_demo; transport_summary_ladder=network_owned_transport_result_task_policy_demo -> network_owned_transport_result_task_batch_demo -> network_owned_transport_result_task_windowed_batch_demo -> network_owned_transport_result_session_bridge_demo -> network_transport_result_task_policy_demo -> network_transport_result_task_batch_demo -> network_transport_result_task_windowed_batch_demo -> network_transport_result_session_bridge_demo",
        ),
        _ => None,
    }
}

fn scheduler_summary_samples(domain: &str) -> Option<&'static str> {
    match domain {
        "shader" => Some(
            "policy=shader_async_policy_profile_demo -> shader_async_fallback_profile_demo; windowed=shader_async_batch_profile_demo -> shader_async_windowed_batch_profile_demo",
        ),
        "kernel" => Some(
            "policy=kernel_async_tensor_policy_profile_demo -> kernel_async_tensor_fallback_profile_demo; windowed=kernel_async_tensor_batch_profile_demo -> kernel_async_tensor_windowed_batch_profile_demo",
        ),
        "network" => Some(
            "transport_split=network_transport_result_policy_split_demo -> network_transport_result_batch_split_demo -> network_transport_result_windowed_split_demo -> network_transport_result_session_bridge_split_demo; control_split=network_connect_accept_task_policy_demo -> network_connect_accept_task_batch_demo -> network_connect_accept_task_windowed_batch_demo",
        ),
        _ => None,
    }
}

fn std_net_sample_navigation(domain: &str) -> Option<&'static str> {
    match domain {
        "network" => {
            Some("profile_core -> transport_edge -> syscall_edge -> socket_edge -> control_edge -> protocol_edge -> http_edge -> result_spine -> task_spine -> session")
        }
        _ => None,
    }
}

fn std_net_recipe_samples(domain: &str) -> Option<&'static str> {
    match domain {
        "network" => Some(
            "profile_core=net_endpoint_recipe -> net_endpoint_recipe_demo; transport_edge=net_ip_packet_recipe -> net_tcp_stream_recipe -> net_udp_datagram_recipe -> net_ip_packet_recipe_demo -> net_tcp_stream_recipe_demo -> net_udp_datagram_recipe_demo; syscall_edge=net_tcp_open_recipe -> net_udp_open_recipe -> net_udp_bind_recipe -> net_udp_bound_socket_recipe -> net_udp_datagram_flow_recipe -> net_tcp_listener_recipe -> net_tcp_client_flow_recipe -> net_tcp_server_flow_recipe -> net_tcp_accepted_socket_recipe -> net_owned_send_recipe -> net_owned_recv_recipe -> net_owned_accept_recipe -> net_owned_close_recipe -> net_tcp_open_recipe_demo -> net_udp_open_recipe_demo -> net_udp_bind_recipe_demo -> net_udp_bound_socket_recipe_demo -> net_udp_datagram_flow_recipe_demo -> net_tcp_listener_recipe_demo -> net_tcp_client_flow_recipe_demo -> net_tcp_server_flow_recipe_demo -> net_tcp_accepted_socket_recipe_demo -> net_owned_send_recipe_demo -> net_owned_recv_recipe_demo -> net_owned_accept_recipe_demo -> net_owned_close_recipe_demo; flow_group=tcp client flow -> tcp server flow -> udp datagram flow; flow=net_tcp_client_flow_recipe -> net_tcp_server_flow_recipe -> net_udp_datagram_flow_recipe -> net_tcp_client_flow_recipe_demo -> net_tcp_server_flow_recipe_demo -> net_udp_datagram_flow_recipe_demo; socket_edge=net_tcp_connect_socket_recipe -> net_tcp_client_flow_recipe -> net_tcp_socket_recipe -> net_tcp_server_socket_recipe -> net_tcp_server_flow_recipe -> net_tcp_accepted_socket_recipe -> net_udp_bound_socket_recipe -> net_udp_datagram_flow_recipe -> net_udp_socket_recipe -> net_ip_socket_recipe -> net_tcp_connect_socket_recipe_demo -> net_tcp_client_flow_recipe_demo -> net_tcp_socket_recipe_demo -> net_tcp_server_socket_recipe_demo -> net_tcp_server_flow_recipe_demo -> net_tcp_accepted_socket_recipe_demo -> net_udp_bound_socket_recipe_demo -> net_udp_datagram_flow_recipe_demo -> net_udp_socket_recipe_demo -> net_ip_socket_recipe_demo; control_edge=net_connect_recipe -> net_listen_recipe -> net_close_recipe -> net_connect_recipe_demo -> net_listen_recipe_demo -> net_close_recipe_demo; protocol_edge=net_protocol_experiment_recipe -> net_line_protocol_recipe -> net_datagram_protocol_recipe -> net_dnsish_protocol_recipe -> net_dnsish_query_recipe -> net_httpish_protocol_recipe -> net_httpish_request_recipe -> net_httpish_response_recipe -> net_httpish_roundtrip_recipe -> net_protocol_experiment_recipe_demo -> net_line_protocol_recipe_demo -> net_datagram_protocol_recipe_demo -> net_dnsish_protocol_recipe_demo -> net_dnsish_query_recipe_demo -> net_httpish_protocol_recipe_demo -> net_httpish_request_recipe_demo -> net_httpish_response_recipe_demo -> net_httpish_roundtrip_recipe_demo; http_edge=net_http_client_recipe -> net_http_request_builder_recipe -> net_http_client_headers_recipe -> net_http_client_url_recipe -> net_http_client_body_recipe -> net_http_client_status_recipe -> net_http_request_recipe -> net_http_response_recipe -> net_http_client_exchange_recipe -> net_http_client_session_recipe -> net_http_client_get_recipe -> net_http_client_post_recipe -> net_http_client_recipe_demo -> net_http_request_builder_recipe_demo -> net_http_client_headers_recipe_demo -> net_http_client_url_recipe_demo -> net_http_client_body_recipe_demo -> net_http_client_status_recipe_demo -> net_http_request_recipe_demo -> net_http_response_recipe_demo -> net_http_client_exchange_recipe_demo -> net_http_client_session_recipe_demo -> net_http_client_get_recipe_demo -> net_http_client_post_recipe_demo; result_spine=net_result_recipe -> net_result_bridge_recipe -> net_result_recipe_demo -> net_result_bridge_recipe_demo; task_spine=net_task_policy_recipe -> net_task_batch_recipe -> net_task_windowed_recipe -> net_task_windowed_bridge_recipe -> net_task_policy_recipe_demo -> net_task_batch_recipe_demo -> net_task_windowed_recipe_demo -> net_task_windowed_bridge_recipe_demo; compare_group=transport compare -> dnsish compare -> httpish compare; compare=net_transport_path_compare_recipe -> net_dnsish_path_compare_recipe -> net_httpish_path_compare_recipe -> net_transport_path_compare_recipe_demo -> net_dnsish_path_compare_recipe_demo -> net_httpish_path_compare_recipe_demo; owned_session=net_owned_transport_session_recipe -> net_owned_datagram_session_recipe -> net_owned_dnsish_exchange_session_recipe -> net_owned_dnsish_pipeline_recipe -> net_owned_transport_session_recipe_demo -> net_owned_datagram_session_recipe_demo -> net_owned_dnsish_exchange_session_recipe_demo -> net_owned_dnsish_pipeline_recipe_demo; session=net_control_session_recipe -> net_transport_session_recipe -> net_owned_transport_session_recipe -> net_tcp_listener_session_recipe -> net_transport_path_compare_recipe -> net_protocol_session_recipe -> net_datagram_session_recipe -> net_owned_datagram_session_recipe -> net_udp_bound_session_recipe -> net_datagram_exchange_session_recipe -> net_datagram_pipeline_recipe -> net_dnsish_exchange_session_recipe -> net_owned_dnsish_exchange_session_recipe -> net_dnsish_path_compare_recipe -> net_dnsish_pipeline_recipe -> net_owned_dnsish_pipeline_recipe -> net_http_client_session_recipe -> net_httpish_session_recipe -> net_httpish_exchange_session_recipe -> net_httpish_path_compare_recipe -> net_session_recipe -> net_control_session_recipe_demo -> net_transport_session_recipe_demo -> net_owned_transport_session_recipe_demo -> net_tcp_listener_session_recipe_demo -> net_transport_path_compare_recipe_demo -> net_protocol_session_recipe_demo -> net_datagram_session_recipe_demo -> net_owned_datagram_session_recipe_demo -> net_udp_bound_session_recipe_demo -> net_datagram_exchange_session_recipe_demo -> net_datagram_pipeline_recipe_demo -> net_dnsish_exchange_session_recipe_demo -> net_owned_dnsish_exchange_session_recipe_demo -> net_dnsish_path_compare_recipe_demo -> net_dnsish_pipeline_recipe_demo -> net_owned_dnsish_pipeline_recipe_demo -> net_http_client_session_recipe_demo -> net_httpish_session_recipe_demo -> net_httpish_exchange_session_recipe_demo -> net_httpish_path_compare_recipe_demo -> net_session_recipe_demo",
        ),
        _ => None,
    }
}

pub fn load_index(root: &Path) -> Result<Vec<NustarPackageIndexEntry>, String> {
    let path = root.join(INDEX_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    parse_index(&source, &path)
}

pub fn load_manifest(root: &Path, package_id: &str) -> Result<NustarPackageManifest, String> {
    let root = resolve_registry_root(root);
    let index = load_index(&root)?;
    let entry = index
        .into_iter()
        .find(|entry| entry.package_id == package_id)
        .ok_or_else(|| {
            format!(
                "nustar package `{package_id}` is not present in `{}`",
                root.join(INDEX_FILE).display()
            )
        })?;
    let path = manifest_path(&root, &entry);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    parse_manifest(&source, &path)
}

pub fn load_manifest_for_domain(
    root: &Path,
    domain_family: &str,
) -> Result<NustarPackageManifest, String> {
    let root = resolve_registry_root(root);
    let path = match load_index(&root) {
        Ok(index) => {
            match index
                .into_iter()
                .find(|entry| entry.domain_family == domain_family)
            {
                Some(entry) => manifest_path(&root, &entry),
                None => {
                    let direct = root.join(format!("{domain_family}.toml"));
                    if direct.exists() {
                        direct
                    } else {
                        return Err(format!(
                            "no nustar package is indexed for mod domain `{domain_family}` in `{}`",
                            root.join(INDEX_FILE).display()
                        ));
                    }
                }
            }
        }
        Err(index_error) => {
            let direct = root.join(format!("{domain_family}.toml"));
            if direct.exists() {
                direct
            } else {
                return Err(index_error);
            }
        }
    };
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
    parse_manifest(&source, &path)
}

pub fn load_all_manifests(root: &Path) -> Result<Vec<NustarPackageManifest>, String> {
    let mut manifests = Vec::new();
    for entry in load_index(root)? {
        manifests.push(load_manifest(root, &entry.package_id)?);
    }
    manifests.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(manifests)
}

pub fn required_package_ids(module: &YirModule) -> Vec<String> {
    let mut package_ids = BTreeSet::new();
    for node in &module.nodes {
        package_ids.insert(format!("official.{}", node.op.module));
        if node.op.module == "cpu" && node.op.instruction == "instantiate_unit" {
            if let Some(domain) = node.op.args.first() {
                package_ids.insert(format!("official.{domain}"));
            }
        }
    }
    package_ids.into_iter().collect()
}

pub fn load_required_manifests(
    root: &Path,
    module: &YirModule,
) -> Result<Vec<NustarPackageManifest>, String> {
    let mut manifests = Vec::new();
    for package_id in required_package_ids(module) {
        manifests.push(load_manifest(root, &package_id)?);
    }
    manifests.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(manifests)
}

pub fn plan_bindings(
    root: &Path,
    nir: &NirModule,
    module: &YirModule,
    domain: &str,
    unit: &str,
    declared_used_units: &[(String, String)],
    declared_externs: &[(String, String)],
) -> Result<NustarBindingPlan, String> {
    let mut manifests = load_required_manifests(root, module)?;
    let mut loaded_domains = manifests
        .iter()
        .map(|manifest| manifest.domain_family.clone())
        .collect::<BTreeSet<_>>();
    if loaded_domains.insert(domain.to_owned()) {
        manifests.push(load_manifest_for_domain(root, domain)?);
    }
    for (used_domain, _) in declared_used_units {
        if loaded_domains.insert(used_domain.clone()) {
            manifests.push(load_manifest_for_domain(root, used_domain)?);
        }
    }
    manifests.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    validate_unit_binding(&manifests, domain, unit)?;
    let mut bindings = Vec::new();

    for manifest in manifests {
        let execution = execution_summary(&manifest);
        let registered_units = manifest
            .unit_types
            .iter()
            .filter(|unit| !unit.is_empty())
            .cloned()
            .collect::<Vec<_>>();
        let bound_unit = if manifest.domain_family == domain {
            Some(unit.to_owned())
        } else {
            None
        };
        let used_units = declared_used_units
            .iter()
            .filter(|(used_domain, _)| used_domain == &manifest.domain_family)
            .map(|(_, used_unit)| used_unit.clone())
            .collect::<Vec<_>>();
        let instantiated_units = module
            .nodes
            .iter()
            .filter(|node| {
                node.op.module == "cpu"
                    && node.op.instruction == "instantiate_unit"
                    && node.op.args.first().map(String::as_str)
                        == Some(manifest.domain_family.as_str())
            })
            .filter_map(|node| node.op.args.get(1).cloned())
            .collect::<Vec<_>>();
        let used_host_ffi_abis = if manifest.domain_family == "cpu" {
            declared_externs
                .iter()
                .map(|(abi, _)| abi.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let used_host_ffi_symbols = if manifest.domain_family == "cpu" {
            declared_externs
                .iter()
                .map(|(_, symbol)| symbol.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let (matched_support_surface, matched_support_profile_slots) =
            detect_matched_support_usage(nir, &manifest.domain_family);
        let covered_support_profile_slots = covered_profile_slots(
            &manifest.domain_family,
            &matched_support_surface,
            &matched_support_profile_slots,
        );
        let uncovered_support_profile_slots = manifest
            .support_profile_slots
            .iter()
            .filter(|slot| {
                !covered_support_profile_slots
                    .iter()
                    .any(|covered| covered == *slot)
            })
            .cloned()
            .collect::<Vec<_>>();

        let mut matched_resources = module
            .resources
            .iter()
            .filter(|resource| {
                manifest
                    .resource_families
                    .iter()
                    .any(|family| family == resource.kind.family())
            })
            .map(|resource| resource.name.clone())
            .collect::<BTreeSet<_>>();
        collect_resource_usage_hints(&nir, &manifest.domain_family, &mut matched_resources);
        let matched_resources = matched_resources.into_iter().collect::<Vec<_>>();

        let matched_ops = module
            .nodes
            .iter()
            .filter(|node| node.op.module == manifest.domain_family)
            .map(|node| node.op.full_name())
            .collect::<Vec<_>>();

        if matched_ops.is_empty() && instantiated_units.is_empty() && used_units.is_empty() {
            return Err(format!(
                "nustar package `{}` was selected but no matching ops were bound",
                manifest.package_id
            ));
        }

        let undeclared_ops = matched_ops
            .iter()
            .filter(|op| !manifest.ops.iter().any(|candidate| candidate == *op))
            .cloned()
            .collect::<Vec<_>>();

        bindings.push(NustarBinding {
            package_id: manifest.package_id,
            domain_family: manifest.domain_family,
            ast_entry: manifest.ast_entry,
            nir_entry: manifest.nir_entry,
            yir_lowering_entry: manifest.yir_lowering_entry,
            part_verify_entry: manifest.part_verify_entry,
            machine_abi_policy: manifest.machine_abi_policy,
            abi_profiles: manifest.abi_profiles,
            abi_capabilities: manifest.abi_capabilities,
            ast_surface: manifest.ast_surface,
            nir_surface: manifest.nir_surface,
            yir_lowering: manifest.yir_lowering,
            part_verify: manifest.part_verify,
            support_surface: manifest.support_surface,
            support_profile_slots: manifest.support_profile_slots,
            capability_tags: manifest.capability_tags,
            default_lanes: manifest.default_lanes,
            execution,
            matched_support_surface,
            matched_support_profile_slots,
            covered_support_profile_slots,
            uncovered_support_profile_slots,
            registered_units,
            bound_unit,
            used_units,
            instantiated_units,
            used_host_ffi_abis,
            used_host_ffi_symbols,
            matched_resources,
            matched_ops,
            undeclared_ops,
            frontend: manifest.frontend,
            entry_crate: manifest.entry_crate,
        });
    }

    bindings.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(NustarBindingPlan { bindings })
}

fn covered_profile_slots(
    domain_family: &str,
    matched_support_surface: &[String],
    matched_support_profile_slots: &[String],
) -> Vec<String> {
    let mut covered = matched_support_profile_slots
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for surface in matched_support_surface {
        for slot in implied_slots_for_surface(domain_family, surface) {
            covered.insert(slot.to_string());
        }
    }
    covered.into_iter().collect::<Vec<_>>()
}

fn collect_resource_usage_hints(
    module: &NirModule,
    domain_family: &str,
    resources: &mut BTreeSet<String>,
) {
    for function in &module.functions {
        for stmt in &function.body {
            collect_resource_usage_hints_stmt(stmt, domain_family, resources);
        }
    }
}

fn collect_resource_usage_hints_stmt(
    stmt: &NirStmt,
    domain_family: &str,
    resources: &mut BTreeSet<String>,
) {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Await(value)
        | NirStmt::Expr(value) => {
            collect_resource_usage_hints_expr(value, domain_family, resources)
        }
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            collect_resource_usage_hints_expr(condition, domain_family, resources);
            for stmt in then_body {
                collect_resource_usage_hints_stmt(stmt, domain_family, resources);
            }
            for stmt in else_body {
                collect_resource_usage_hints_stmt(stmt, domain_family, resources);
            }
        }
        NirStmt::While { condition, body } => {
            collect_resource_usage_hints_expr(condition, domain_family, resources);
            for stmt in body {
                collect_resource_usage_hints_stmt(stmt, domain_family, resources);
            }
        }
        NirStmt::Return(Some(value)) => {
            collect_resource_usage_hints_expr(value, domain_family, resources);
        }
        NirStmt::Return(None) | NirStmt::Break | NirStmt::Continue => {}
    }
}

fn collect_resource_usage_hints_expr(
    expr: &NirExpr,
    domain_family: &str,
    resources: &mut BTreeSet<String>,
) {
    if domain_family == "shader" {
        match expr {
            NirExpr::ShaderBinding {
                kind,
                layout,
                profile_contract,
                ..
            } => {
                resources.insert(format!("shader.binding.{kind}"));
                if let Some(layout) = layout {
                    resources.insert(format!("shader.binding.layout.{layout}"));
                }
                if let Some(profile_contract) = profile_contract {
                    resources.insert(format!("shader.binding.contract.{profile_contract}"));
                }
            }
            NirExpr::ShaderBindSet { .. } => {
                resources.insert("shader.binding.set".to_owned());
            }
            NirExpr::ShaderTexture2d { .. } => {
                resources.insert("shader.resource.texture2d".to_owned());
            }
            NirExpr::ShaderSampler { .. } => {
                resources.insert("shader.resource.sampler".to_owned());
            }
            _ => {}
        }
    }

    walk_child_exprs(expr, &mut |child| {
        collect_resource_usage_hints_expr(child, domain_family, resources);
    });
}

fn implied_slots_for_surface(domain_family: &str, surface: &str) -> Vec<String> {
    let slots: &[&str] = match (domain_family, surface) {
        ("shader", "shader.profile.render.v1") => &[
            "target",
            "viewport",
            "pipeline",
            "vertex_count",
            "instance_count",
            "pass_kind",
            "packet_field_count",
        ],
        ("shader", "shader.profile.draw.v1") => &[
            "target",
            "viewport",
            "pipeline",
            "vertex_count",
            "instance_count",
            "pass_kind",
            "packet_field_count",
        ],
        ("shader", "shader.profile.seed.color.v1") => {
            &["packet_color_slot", "slider_color_slot", "material_mode"]
        }
        ("shader", "shader.profile.seed.speed.v1") => {
            &["packet_speed_slot", "slider_speed_slot", "packet_tag"]
        }
        ("shader", "shader.profile.seed.radius.v1") => &[
            "packet_radius_slot",
            "slider_radius_slot",
            "packet_field_count",
        ],
        ("shader", "shader.profile.packet.v1") => &[
            "packet_color_slot",
            "packet_speed_slot",
            "packet_radius_slot",
            "slider_color_slot",
            "slider_speed_slot",
            "slider_radius_slot",
            "header_accent_slot",
            "toggle_live_slot",
            "focus_slot",
        ],
        ("shader", "shader.profile.packet.nova.v1") => &[
            "slider_color_slot",
            "slider_speed_slot",
            "slider_radius_slot",
            "header_accent_slot",
            "toggle_live_slot",
            "focus_slot",
        ],
        ("shader", "shader.profile.target.v1") => &["target"],
        ("shader", "shader.profile.viewport.v1") => &["viewport"],
        ("shader", "shader.profile.pipeline.v1") => &["pipeline"],
        ("shader", "shader.profile.draw-budget.v1") => &["vertex_count", "instance_count"],
        ("shader", "shader.profile.packet-slots.v1") => &[
            "packet_color_slot",
            "packet_speed_slot",
            "packet_radius_slot",
            "slider_color_slot",
            "slider_speed_slot",
            "slider_radius_slot",
            "header_accent_slot",
            "toggle_live_slot",
            "focus_slot",
        ],
        ("shader", "shader.profile.packet-tag.v1") => &["packet_tag"],
        ("shader", "shader.profile.material-mode.v1") => &["material_mode"],
        ("shader", "shader.profile.pass-kind.v1") => &["pass_kind"],
        ("shader", "shader.profile.packet-field-count.v1") => &["packet_field_count"],
        ("data", "data.profile.bind-core.v1") => &["bind_core"],
        ("data", "data.profile.send.uplink.v1") => {
            let mut slots = vec!["window_offset".to_owned(), "uplink_len".to_owned()];
            slots.extend(all_uplink_directional_marker_slots());
            slots.extend(data_common_marker_slots().iter().filter_map(|slot| {
                slot.starts_with("marker:uplink_")
                    .then_some((*slot).to_owned())
            }));
            return slots;
        }
        ("data", "data.profile.send.downlink.v1") => {
            let mut slots = vec!["window_offset".to_owned(), "downlink_len".to_owned()];
            slots.extend(all_downlink_directional_marker_slots());
            slots.extend(data_common_marker_slots().iter().filter_map(|slot| {
                slot.starts_with("marker:downlink_")
                    .then_some((*slot).to_owned())
            }));
            return slots;
        }
        ("data", "data.profile.handle-table.v1") => &["handle_table"],
        ("data", "data.profile.window-layout.v1") => {
            &["window_offset", "uplink_len", "downlink_len"]
        }
        ("data", "data.profile.sync-markers.v1") => return all_sync_marker_slots(),
        ("data", "data.profile.pipe-markers.v1") => &["marker:uplink_pipe", "marker:downlink_pipe"],
        ("data", "data.profile.pipe-class.v1") => {
            &["marker:uplink_pipe_class", "marker:downlink_pipe_class"]
        }
        ("data", "data.profile.payload-class.v1") => &[
            "marker:uplink_payload_class",
            "marker:downlink_payload_class",
        ],
        ("data", "data.profile.payload-shape.v1") => &[
            "marker:uplink_payload_shape",
            "marker:downlink_payload_shape",
        ],
        ("data", "data.profile.window-policy.v1") => &[
            "marker:uplink_window_policy",
            "marker:downlink_window_policy",
        ],
        ("network", "network.profile.bind-core.v1") => &["bind_core"],
        ("network", "network.profile.connect.v1") => {
            &["remote_port", "connect_timeout_ms", "endpoint_kind"]
        }
        ("network", "network.profile.accept.v1") => &[
            "local_port",
            "read_timeout_ms",
            "write_timeout_ms",
            "endpoint_kind",
        ],
        ("network", "network.profile.send.v1") => &["send_window", "stream_window"],
        ("network", "network.profile.recv.v1") => &["recv_window", "stream_window"],
        ("network", "network.profile.close.v1") => &[],
        ("network", "network.profile.timeout.v1") => &[
            "connect_timeout_ms",
            "read_timeout_ms",
            "write_timeout_ms",
            "timeout_budget",
        ],
        ("network", "network.profile.retry.v1") => &["retry_budget"],
        ("network", "network.profile.endpoint-kind.v1") => &["endpoint_kind"],
        ("network", "network.profile.stream-window.v1") => {
            &["stream_window", "recv_window", "send_window"]
        }
        ("network", "network.profile.transport.v1") => &["transport_family"],
        ("network", "network.profile.protocol.v1") => {
            &["protocol_kind", "protocol_version", "protocol_header_bytes"]
        }
        _ => &[],
    };
    slots.iter().map(|slot| (*slot).to_owned()).collect()
}

fn detect_matched_support_usage(
    module: &NirModule,
    domain_family: &str,
) -> (Vec<String>, Vec<String>) {
    let mut surfaces = BTreeSet::new();
    let mut slots = BTreeSet::new();
    for function in &module.functions {
        for stmt in &function.body {
            collect_support_usage_stmt(stmt, domain_family, &mut surfaces, &mut slots);
        }
    }
    (
        surfaces.into_iter().collect::<Vec<_>>(),
        slots.into_iter().collect::<Vec<_>>(),
    )
}

fn collect_support_usage_stmt(
    stmt: &NirStmt,
    domain_family: &str,
    surfaces: &mut BTreeSet<String>,
    slots: &mut BTreeSet<String>,
) {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Await(value)
        | NirStmt::Expr(value) => collect_support_usage_expr(value, domain_family, surfaces, slots),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            collect_support_usage_expr(condition, domain_family, surfaces, slots);
            for stmt in then_body {
                collect_support_usage_stmt(stmt, domain_family, surfaces, slots);
            }
            for stmt in else_body {
                collect_support_usage_stmt(stmt, domain_family, surfaces, slots);
            }
        }
        NirStmt::While { condition, body } => {
            collect_support_usage_expr(condition, domain_family, surfaces, slots);
            for stmt in body {
                collect_support_usage_stmt(stmt, domain_family, surfaces, slots);
            }
        }
        NirStmt::Break | NirStmt::Continue => {}
        NirStmt::Return(value) => {
            if let Some(value) = value {
                collect_support_usage_expr(value, domain_family, surfaces, slots);
            }
        }
    }
}

fn collect_support_usage_expr(
    expr: &NirExpr,
    domain_family: &str,
    surfaces: &mut BTreeSet<String>,
    slots: &mut BTreeSet<String>,
) {
    match expr {
        NirExpr::ShaderProfileTargetRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.target.v1".to_owned());
            slots.insert("target".to_owned());
        }
        NirExpr::ShaderProfileViewportRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.viewport.v1".to_owned());
            slots.insert("viewport".to_owned());
        }
        NirExpr::ShaderProfilePipelineRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.pipeline.v1".to_owned());
            slots.insert("pipeline".to_owned());
        }
        NirExpr::ShaderProfileVertexCountRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.draw-budget.v1".to_owned());
            slots.insert("vertex_count".to_owned());
        }
        NirExpr::ShaderProfileInstanceCountRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.draw-budget.v1".to_owned());
            slots.insert("instance_count".to_owned());
        }
        NirExpr::ShaderProfilePacketColorSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            slots.insert("packet_color_slot".to_owned());
        }
        NirExpr::ShaderProfilePacketSpeedSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            slots.insert("packet_speed_slot".to_owned());
        }
        NirExpr::ShaderProfilePacketRadiusSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            slots.insert("packet_radius_slot".to_owned());
        }
        NirExpr::ShaderProfileSliderColorSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            surfaces.insert("shader.profile.packet.nova.v1".to_owned());
            slots.insert("slider_color_slot".to_owned());
        }
        NirExpr::ShaderProfileSliderSpeedSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            surfaces.insert("shader.profile.packet.nova.v1".to_owned());
            slots.insert("slider_speed_slot".to_owned());
        }
        NirExpr::ShaderProfileSliderRadiusSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            surfaces.insert("shader.profile.packet.nova.v1".to_owned());
            slots.insert("slider_radius_slot".to_owned());
        }
        NirExpr::ShaderProfileHeaderAccentSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            surfaces.insert("shader.profile.packet.nova.v1".to_owned());
            slots.insert("header_accent_slot".to_owned());
        }
        NirExpr::ShaderProfileToggleLiveSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            surfaces.insert("shader.profile.packet.nova.v1".to_owned());
            slots.insert("toggle_live_slot".to_owned());
        }
        NirExpr::ShaderProfileFocusSlotRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-slots.v1".to_owned());
            surfaces.insert("shader.profile.packet.nova.v1".to_owned());
            slots.insert("focus_slot".to_owned());
        }
        NirExpr::ShaderProfilePacketTagRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-tag.v1".to_owned());
            slots.insert("packet_tag".to_owned());
        }
        NirExpr::ShaderProfileMaterialModeRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.material-mode.v1".to_owned());
            slots.insert("material_mode".to_owned());
        }
        NirExpr::ShaderProfilePassKindRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.pass-kind.v1".to_owned());
            slots.insert("pass_kind".to_owned());
        }
        NirExpr::ShaderProfilePacketFieldCountRef { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet-field-count.v1".to_owned());
            slots.insert("packet_field_count".to_owned());
        }
        NirExpr::ShaderProfileColorSeed { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.seed.color.v1".to_owned());
        }
        NirExpr::ShaderProfileSpeedSeed { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.seed.speed.v1".to_owned());
        }
        NirExpr::ShaderProfileRadiusSeed { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.seed.radius.v1".to_owned());
        }
        NirExpr::ShaderProfilePacket {
            packet_type_name,
            accent,
            toggle_state,
            focus_index,
            ..
        } if domain_family == "shader" => {
            surfaces.insert("shader.profile.packet.v1".to_owned());
            let is_nova_panel = packet_type_name.as_deref() == Some("NovaPanelPacket")
                || accent.is_some()
                || toggle_state.is_some()
                || focus_index.is_some();
            if is_nova_panel {
                surfaces.insert("shader.profile.packet.nova.v1".to_owned());
            }
        }
        NirExpr::ShaderBinding {
            profile_contract: Some(profile_contract),
            ..
        } if domain_family == "shader" => {
            surfaces.insert(profile_contract.clone());
        }
        NirExpr::ShaderProfileRender { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.render.v1".to_owned());
        }
        NirExpr::ShaderDrawInstanced { .. } if domain_family == "shader" => {
            surfaces.insert("shader.profile.draw.v1".to_owned());
        }
        NirExpr::ShaderInlineWgsl { .. } if domain_family == "shader" => {
            surfaces.insert("shader.inline.wgsl.v1".to_owned());
        }
        NirExpr::DataProfileBindCoreRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.bind-core.v1".to_owned());
            slots.insert("bind_core".to_owned());
        }
        NirExpr::DataProfileWindowOffsetRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.window-layout.v1".to_owned());
            slots.insert("window_offset".to_owned());
        }
        NirExpr::DataProfileUplinkLenRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.window-layout.v1".to_owned());
            slots.insert("uplink_len".to_owned());
        }
        NirExpr::DataProfileDownlinkLenRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.window-layout.v1".to_owned());
            slots.insert("downlink_len".to_owned());
        }
        NirExpr::DataProfileHandleTableRef { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.handle-table.v1".to_owned());
            slots.insert("handle_table".to_owned());
        }
        NirExpr::DataProfileMarkerRef { tag, .. } if domain_family == "data" => {
            surfaces.insert(data_marker_surface(tag).to_owned());
            slots.insert(format!("marker:{tag}"));
        }
        NirExpr::NetworkProfileBindCoreRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.bind-core.v1".to_owned());
            slots.insert("bind_core".to_owned());
        }
        NirExpr::NetworkProfileEndpointKindRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.endpoint-kind.v1".to_owned());
            slots.insert("endpoint_kind".to_owned());
        }
        NirExpr::NetworkProfileTransportFamilyRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.transport.v1".to_owned());
            slots.insert("transport_family".to_owned());
        }
        NirExpr::NetworkProfileLocalPortRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.accept.v1".to_owned());
            slots.insert("local_port".to_owned());
        }
        NirExpr::NetworkProfileRemotePortRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.connect.v1".to_owned());
            slots.insert("remote_port".to_owned());
        }
        NirExpr::NetworkProfileConnectTimeoutRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.timeout.v1".to_owned());
            slots.insert("connect_timeout_ms".to_owned());
        }
        NirExpr::NetworkProfileReadTimeoutRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.timeout.v1".to_owned());
            slots.insert("read_timeout_ms".to_owned());
        }
        NirExpr::NetworkProfileWriteTimeoutRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.timeout.v1".to_owned());
            slots.insert("write_timeout_ms".to_owned());
        }
        NirExpr::NetworkProfileTimeoutBudgetRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.timeout.v1".to_owned());
            slots.insert("timeout_budget".to_owned());
        }
        NirExpr::NetworkProfileRetryBudgetRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.retry.v1".to_owned());
            slots.insert("retry_budget".to_owned());
        }
        NirExpr::NetworkProfileStreamWindowRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.stream-window.v1".to_owned());
            slots.insert("stream_window".to_owned());
        }
        NirExpr::NetworkProfileRecvWindowRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.recv.v1".to_owned());
            slots.insert("recv_window".to_owned());
        }
        NirExpr::NetworkProfileSendWindowRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.send.v1".to_owned());
            slots.insert("send_window".to_owned());
        }
        NirExpr::NetworkProfileProtocolKindRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.protocol.v1".to_owned());
            slots.insert("protocol_kind".to_owned());
        }
        NirExpr::NetworkProfileProtocolVersionRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.protocol.v1".to_owned());
            slots.insert("protocol_version".to_owned());
        }
        NirExpr::NetworkProfileProtocolHeaderBytesRef { .. } if domain_family == "network" => {
            surfaces.insert("network.profile.protocol.v1".to_owned());
            slots.insert("protocol_header_bytes".to_owned());
        }
        NirExpr::KernelProfileBindCoreRef { .. } if domain_family == "kernel" => {
            surfaces.insert("kernel.profile.bind-core.v1".to_owned());
            slots.insert("bind_core".to_owned());
        }
        NirExpr::KernelProfileQueueDepthRef { .. } if domain_family == "kernel" => {
            surfaces.insert("kernel.profile.queue-depth.v1".to_owned());
            slots.insert("queue_depth".to_owned());
        }
        NirExpr::KernelProfileBatchLanesRef { .. } if domain_family == "kernel" => {
            surfaces.insert("kernel.profile.batch-lanes.v1".to_owned());
            slots.insert("batch_lanes".to_owned());
        }
        NirExpr::DataProfileSendUplink { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.send.uplink.v1".to_owned());
        }
        NirExpr::DataProfileSendDownlink { .. } if domain_family == "data" => {
            surfaces.insert("data.profile.send.downlink.v1".to_owned());
        }
        _ => {}
    }

    walk_child_exprs(expr, &mut |child| {
        collect_support_usage_expr(child, domain_family, surfaces, slots);
    });
}

fn walk_child_exprs(expr: &NirExpr, f: &mut dyn FnMut(&NirExpr)) {
    match expr {
        NirExpr::Await(inner)
        | NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::HostBufferHandle(inner)
        | NirExpr::Move(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::CpuJoin(inner)
        | NirExpr::CpuThreadJoin(inner)
        | NirExpr::CpuCancel(inner)
        | NirExpr::CpuJoinResult(inner)
        | NirExpr::CpuThreadJoinResult(inner)
        | NirExpr::CpuTaskCompleted(inner)
        | NirExpr::CpuTaskTimedOut(inner)
        | NirExpr::CpuTaskCancelled(inner)
        | NirExpr::CpuTaskValue(inner)
        | NirExpr::CpuMutexNew(inner)
        | NirExpr::CpuMutexLock(inner)
        | NirExpr::CpuMutexUnlock(inner)
        | NirExpr::CpuMutexValue(inner)
        | NirExpr::DataReady(inner)
        | NirExpr::DataMoved(inner)
        | NirExpr::DataWindowed(inner)
        | NirExpr::DataValue(inner)
        | NirExpr::DataFreezeWindow(inner)
        | NirExpr::NetworkConfigReady(inner)
        | NirExpr::NetworkSendReady(inner)
        | NirExpr::NetworkRecvReady(inner)
        | NirExpr::NetworkAcceptReady(inner)
        | NirExpr::NetworkValue(inner)
        | NirExpr::ShaderPassReady(inner)
        | NirExpr::ShaderFrameReady(inner)
        | NirExpr::ShaderValue(inner)
        | NirExpr::KernelConfigReady(inner)
        | NirExpr::KernelValue(inner)
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => f(inner),
        NirExpr::AllocNode { value, next } => {
            f(value);
            f(next);
        }
        NirExpr::AllocBuffer { len, fill } => {
            f(len);
            f(fill);
        }
        NirExpr::LoadAt { buffer, index } => {
            f(buffer);
            f(index);
        }
        NirExpr::StoreValue { target, value } => {
            f(target);
            f(value);
        }
        NirExpr::StoreNext { target, next } => {
            f(target);
            f(next);
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            f(buffer);
            f(index);
            f(value);
        }
        NirExpr::DataResult { value: input, .. }
        | NirExpr::NetworkResult { value: input, .. }
        | NirExpr::ShaderResult { value: input, .. }
        | NirExpr::KernelResult { value: input, .. } => f(input),
        NirExpr::DataReadWindow { window, index } => {
            f(window);
            f(index);
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => {
            f(window);
            f(index);
            f(value);
        }
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            f(input);
            f(offset);
            f(len);
        }
        NirExpr::ShaderProfileColorSeed { base, delta, .. }
        | NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            f(base);
            f(delta);
        }
        NirExpr::ShaderProfilePacket {
            color,
            speed,
            radius,
            accent,
            toggle_state,
            focus_index,
            ..
        } => {
            f(color);
            f(speed);
            f(radius);
            if let Some(accent) = accent {
                f(accent);
            }
            if let Some(toggle_state) = toggle_state {
                f(toggle_state);
            }
            if let Some(focus_index) = focus_index {
                f(focus_index);
            }
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            f(delta);
            f(scale);
            f(base);
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. }
        | NirExpr::ShaderProfileRender { packet: input, .. }
        | NirExpr::FieldAccess { base: input, .. } => f(input),
        NirExpr::CpuSpawn { args, .. }
        | NirExpr::CpuThreadSpawn { args, .. }
        | NirExpr::CpuExternCall { args, .. }
        | NirExpr::Call { args, .. } => {
            for arg in args {
                f(arg);
            }
        }
        NirExpr::CpuTimeout { task, limit } => {
            f(task);
            f(limit);
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            f(receiver);
            for arg in args {
                f(arg);
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                f(value);
            }
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            f(lhs);
            f(rhs);
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            f(target);
            f(pipeline);
            f(viewport);
        }
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => {
            f(pass);
            f(packet);
            f(vertex_count);
            f(instance_count);
        }
        _ => {}
    }
}

pub fn validate_unit_binding(
    manifests: &[NustarPackageManifest],
    domain: &str,
    unit: &str,
) -> Result<(), String> {
    let manifest = manifests
        .iter()
        .find(|manifest| manifest.domain_family == domain)
        .ok_or_else(|| format!("no nustar manifest loaded for mod domain `{domain}`"))?;

    if manifest.unit_types.is_empty() {
        return Ok(());
    }

    if manifest
        .unit_types
        .iter()
        .any(|candidate| candidate == unit)
    {
        return Ok(());
    }

    Err(format!(
        "unit `{unit}` is not registered by nustar package `{}` for mod domain `{domain}`",
        manifest.package_id
    ))
}

pub fn validate_manifest_abi(
    manifest: &NustarPackageManifest,
    required_abi: &str,
) -> Result<(), String> {
    if manifest
        .abi_profiles
        .iter()
        .any(|profile| profile == required_abi)
    {
        return Ok(());
    }
    Err(format!(
        "nustar package `{}` for domain `{}` does not declare required ABI `{}`; declared ABI profiles: {}",
        manifest.package_id,
        manifest.domain_family,
        required_abi,
        if manifest.abi_profiles.is_empty() {
            "<none>".to_owned()
        } else {
            manifest.abi_profiles.join(", ")
        }
    ))
}

pub fn registered_abi_target(
    manifest: &NustarPackageManifest,
    required_abi: &str,
) -> Result<RegisteredAbiTarget, String> {
    if manifest.abi_targets.is_empty() {
        return Err(format!(
            "nustar package `{}` for domain `{}` does not declare any `abi_targets`",
            manifest.package_id, manifest.domain_family
        ));
    }
    for raw in &manifest.abi_targets {
        let Some((abi, fields)) = raw.split_once(':') else {
            return Err(format!(
                "nustar package `{}` has invalid abi_targets entry `{}`; expected `abi:arch=...|os=...|object=...|calling=...|clang=...`",
                manifest.package_id, raw
            ));
        };
        if abi.trim() != required_abi {
            continue;
        }
        return parse_registered_abi_target(required_abi, fields, manifest, raw);
    }
    Err(format!(
        "nustar package `{}` for domain `{}` does not declare abi target metadata for `{}`",
        manifest.package_id, manifest.domain_family, required_abi
    ))
}

pub fn registered_abi_target_for_clang(
    manifest: &NustarPackageManifest,
    clang_target: &str,
) -> Result<RegisteredAbiTarget, String> {
    if manifest.abi_targets.is_empty() {
        return Err(format!(
            "nustar package `{}` for domain `{}` does not declare any `abi_targets`",
            manifest.package_id, manifest.domain_family
        ));
    }
    let mut matches = Vec::new();
    for raw in &manifest.abi_targets {
        let Some((abi, fields)) = raw.split_once(':') else {
            return Err(format!(
                "nustar package `{}` has invalid abi_targets entry `{}`; expected `abi:arch=...|os=...|object=...|calling=...|clang=...`",
                manifest.package_id, raw
            ));
        };
        let target = parse_registered_abi_target(abi.trim(), fields, manifest, raw)?;
        if target.clang_target == clang_target {
            matches.push(target);
        }
    }
    matches.into_iter().next().ok_or_else(|| {
        format!(
            "nustar package `{}` for domain `{}` does not register clang target `{}` in `abi_targets`",
            manifest.package_id, manifest.domain_family, clang_target
        )
    })
}

pub fn used_ops_for_domain(module: &YirModule, domain_family: &str) -> Vec<String> {
    let mut ops = module
        .nodes
        .iter()
        .filter(|node| node.op.module == domain_family)
        .map(|node| node.op.full_name())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    ops.sort();
    ops
}

pub fn validate_abi_capabilities(
    manifest: &NustarPackageManifest,
    required_abi: &str,
    used_surfaces: &[String],
    used_ops: &[String],
) -> Result<(), String> {
    if manifest.abi_capabilities.is_empty() {
        return Ok(());
    }

    let mut surface_allowed = BTreeSet::new();
    let mut op_allowed = BTreeSet::new();
    let mut saw_required_abi = false;
    for raw in &manifest.abi_capabilities {
        let Some((abi, caps)) = raw.split_once(':') else {
            return Err(format!(
                "nustar package `{}` has invalid abi_capabilities entry `{}`; expected `abi:kind:value[|kind:value...]`",
                manifest.package_id, raw
            ));
        };
        if abi.trim().is_empty() {
            return Err(format!(
                "nustar package `{}` has invalid abi_capabilities entry `{}`; ABI id must not be empty",
                manifest.package_id, raw
            ));
        }
        let abi_matches = abi.trim() == required_abi;
        if !abi_matches {
            continue;
        }
        saw_required_abi = true;
        for cap in caps.split('|').map(str::trim).filter(|cap| !cap.is_empty()) {
            if let Some(value) = cap.strip_prefix("surface:") {
                if value.trim().is_empty() {
                    return Err(format!(
                        "nustar package `{}` has invalid abi_capabilities entry `{}`; `surface:` capability must include a pattern",
                        manifest.package_id, raw
                    ));
                }
                surface_allowed.insert(value.to_owned());
            } else if let Some(value) = cap.strip_prefix("op:") {
                if value.trim().is_empty() {
                    return Err(format!(
                        "nustar package `{}` has invalid abi_capabilities entry `{}`; `op:` capability must include a pattern",
                        manifest.package_id, raw
                    ));
                }
                op_allowed.insert(value.to_owned());
            } else {
                return Err(format!(
                    "nustar package `{}` has invalid abi_capabilities capability `{}` in `{}`; expected `surface:<pattern>` or `op:<pattern>`",
                    manifest.package_id, cap, raw
                ));
            }
        }
    }

    if !saw_required_abi {
        return Err(format!(
            "ABI `{}` of nustar package `{}` has no abi_capabilities mapping; add `{}:...` in manifest",
            required_abi, manifest.package_id, required_abi
        ));
    }

    if !surface_allowed.is_empty() && !surface_allowed.contains("*") {
        for surface in used_surfaces {
            if !surface_allowed
                .iter()
                .any(|allowed| capability_matches(allowed, surface))
            {
                return Err(format!(
                    "ABI `{}` of nustar package `{}` does not allow support surface `{}` (allowed: {})",
                    required_abi,
                    manifest.package_id,
                    surface,
                    surface_allowed
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
    }

    if !op_allowed.is_empty() && !op_allowed.contains("*") {
        for op in used_ops {
            if !op_allowed
                .iter()
                .any(|allowed| capability_matches(allowed, op))
            {
                return Err(format!(
                    "ABI `{}` of nustar package `{}` does not allow op `{}` (allowed: {})",
                    required_abi,
                    manifest.package_id,
                    op,
                    op_allowed.iter().cloned().collect::<Vec<_>>().join(", ")
                ));
            }
        }
    }

    Ok(())
}

fn capability_matches(pattern: &str, actual: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return actual.starts_with(prefix);
    }
    pattern == actual
}

pub fn manifest_path(root: &Path, entry: &NustarPackageIndexEntry) -> PathBuf {
    root.join(&entry.manifest)
}

fn parse_index(source: &str, path: &Path) -> Result<Vec<NustarPackageIndexEntry>, String> {
    let mut entries = Vec::new();
    let mut current = Vec::<String>::new();

    for raw_line in source.lines() {
        let line = raw_line.trim();
        if line == "[[package]]" {
            if !current.is_empty() {
                entries.push(parse_index_entry(&current.join("\n"), path)?);
                current.clear();
            }
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        current.push(line.to_owned());
    }

    if !current.is_empty() {
        entries.push(parse_index_entry(&current.join("\n"), path)?);
    }

    entries.sort_by(|lhs, rhs| lhs.package_id.cmp(&rhs.package_id));
    Ok(entries)
}

fn parse_index_entry(source: &str, path: &Path) -> Result<NustarPackageIndexEntry, String> {
    Ok(NustarPackageIndexEntry {
        package_id: parse_required_string(source, "package_id", path)?,
        manifest: parse_required_string(source, "manifest", path)?,
        domain_family: parse_required_string(source, "domain_family", path)?,
    })
}

fn parse_manifest(source: &str, path: &Path) -> Result<NustarPackageManifest, String> {
    let manifest_schema = parse_optional_string(source, "manifest_schema")
        .unwrap_or_else(|| "nustar-manifest-v1".to_owned());
    let package_id = parse_required_string(source, "package_id", path)?;
    let domain_family = parse_required_string(source, "domain_family", path)?;
    let frontend = parse_required_string(source, "frontend", path)?;
    let entry_crate = parse_required_string(source, "entry_crate", path)?;
    let ast_entry = parse_optional_string(source, "ast_entry")
        .unwrap_or_else(|| format!("{}.ast.bootstrap.v1", domain_family));
    let nir_entry = parse_optional_string(source, "nir_entry")
        .unwrap_or_else(|| format!("{}.nir.bootstrap.v1", domain_family));
    let yir_lowering_entry = parse_optional_string(source, "yir_lowering_entry")
        .unwrap_or_else(|| format!("{}.yir.lowering.v1", domain_family));
    let part_verify_entry = parse_optional_string(source, "part_verify_entry")
        .unwrap_or_else(|| format!("{}.verify.partial.v1", domain_family));
    let ast_surface = parse_optional_string_array(source, "ast_surface")
        .unwrap_or_else(|| vec![format!("{domain_family}.mod-ast.v1")]);
    let nir_surface = parse_optional_string_array(source, "nir_surface")
        .unwrap_or_else(|| vec![format!("nir.{domain_family}.surface.v1")]);
    let yir_lowering = parse_optional_string_array(source, "yir_lowering")
        .unwrap_or_else(|| vec![format!("yir.{domain_family}.lowering.v1")]);
    let part_verify = parse_optional_string_array(source, "part_verify")
        .unwrap_or_else(|| vec![format!("verify.{domain_family}.contract.v1")]);
    let binary_extension =
        parse_optional_string(source, "binary_extension").unwrap_or_else(|| "nustar".to_owned());
    let package_layout = parse_optional_string(source, "package_layout")
        .unwrap_or_else(|| "single-envelope".to_owned());
    let machine_abi_policy = parse_optional_string(source, "machine_abi_policy")
        .unwrap_or_else(|| "exact-match".to_owned());
    let abi_profiles = parse_optional_string_array(source, "abi_profiles").unwrap_or_default();
    let abi_capabilities =
        parse_optional_string_array(source, "abi_capabilities").unwrap_or_default();
    let abi_targets = parse_optional_string_array(source, "abi_targets").unwrap_or_default();
    let implementation_kinds = parse_optional_string_array(source, "implementation_kinds")
        .unwrap_or_else(|| vec!["native-stub".to_owned()]);
    let loader_entry = parse_optional_string(source, "loader_entry")
        .unwrap_or_else(|| "nustar.bootstrap.v1".to_owned());
    let loader_abi = parse_optional_string(source, "loader_abi")
        .unwrap_or_else(|| "nustar-loader-v1".to_owned());
    let host_ffi_surface =
        parse_optional_string_array(source, "host_ffi_surface").unwrap_or_default();
    let host_ffi_abis = parse_optional_string_array(source, "host_ffi_abis").unwrap_or_default();
    let host_ffi_bridge =
        parse_optional_string(source, "host_ffi_bridge").unwrap_or_else(|| "none".to_owned());
    let bridge_lane_policy = parse_optional_string(source, "bridge_lane_policy");
    let bridge_surface = parse_optional_string(source, "bridge_surface");
    let bridge_emission_kind = parse_optional_string(source, "bridge_emission_kind");
    let bridge_entry = parse_optional_string(source, "bridge_entry");
    let bridge_kind = parse_optional_string(source, "bridge_kind");
    let bridge_scheduler_binding = parse_optional_string(source, "bridge_scheduler_binding");
    let backend_stub_kind = parse_optional_string(source, "backend_stub_kind");
    let backend_submission_mode = parse_optional_string(source, "backend_submission_mode");
    let backend_wake_policy = parse_optional_string(source, "backend_wake_policy");
    let backend_transport_model = parse_optional_string(source, "backend_transport_model");
    let backend_request_shape = parse_optional_string(source, "backend_request_shape");
    let backend_response_shape = parse_optional_string(source, "backend_response_shape");
    let backend_dispatch_shape = parse_optional_string(source, "backend_dispatch_shape");
    let backend_memory_binding = parse_optional_string(source, "backend_memory_binding");
    let backend_resource_binding = parse_optional_string(source, "backend_resource_binding");
    let backend_completion_model = parse_optional_string(source, "backend_completion_model");
    let phase_bind = parse_optional_string(source, "phase_bind");
    let phase_submit = parse_optional_string(source, "phase_submit");
    let phase_wait = parse_optional_string(source, "phase_wait");
    let phase_finalize = parse_optional_string(source, "phase_finalize");
    let host_bridge_host_ffi_surface =
        parse_optional_string_array(source, "host_bridge_host_ffi_surface");
    let host_bridge_handle_family =
        parse_optional_string_array(source, "host_bridge_handle_family");
    let host_bridge_phase_order = parse_optional_string_array(source, "host_bridge_phase_order");
    let host_bridge_phase_bind_inputs =
        parse_optional_string_array(source, "host_bridge_phase_bind_inputs");
    let host_bridge_phase_bind_outputs =
        parse_optional_string_array(source, "host_bridge_phase_bind_outputs");
    let host_bridge_phase_submit_inputs =
        parse_optional_string_array(source, "host_bridge_phase_submit_inputs");
    let host_bridge_phase_submit_outputs =
        parse_optional_string_array(source, "host_bridge_phase_submit_outputs");
    let host_bridge_phase_wait_inputs =
        parse_optional_string_array(source, "host_bridge_phase_wait_inputs");
    let host_bridge_phase_wait_outputs =
        parse_optional_string_array(source, "host_bridge_phase_wait_outputs");
    let host_bridge_phase_finalize_inputs =
        parse_optional_string_array(source, "host_bridge_phase_finalize_inputs");
    let host_bridge_phase_finalize_outputs =
        parse_optional_string_array(source, "host_bridge_phase_finalize_outputs");
    let host_bridge_phase_bind_wake = parse_optional_string(source, "host_bridge_phase_bind_wake");
    let host_bridge_phase_submit_wake =
        parse_optional_string(source, "host_bridge_phase_submit_wake");
    let host_bridge_phase_wait_wake = parse_optional_string(source, "host_bridge_phase_wait_wake");
    let host_bridge_phase_finalize_wake =
        parse_optional_string(source, "host_bridge_phase_finalize_wake");
    let host_bridge_plan_begin = parse_optional_bool(source, "host_bridge_plan_begin");
    let host_bridge_plan_end = parse_optional_bool(source, "host_bridge_plan_end");
    let support_surface =
        parse_optional_string_array(source, "support_surface").unwrap_or_default();
    let support_profile_slots =
        parse_optional_string_array(source, "support_profile_slots").unwrap_or_default();
    let capability_tags = parse_optional_string_array(source, "capability_tags")
        .unwrap_or_else(|| infer_default_capability_tags(&domain_family));
    let default_lanes = parse_optional_string_array(source, "default_lanes").unwrap_or_default();
    let clock_domain_id = parse_optional_string(source, "clock_domain_id")
        .unwrap_or_else(|| format!("{domain_family}.clock.local.v1"));
    let clock_kind =
        parse_optional_string(source, "clock_kind").unwrap_or_else(|| "local-monotonic".to_owned());
    let clock_epoch_kind = parse_optional_string(source, "clock_epoch_kind")
        .unwrap_or_else(|| "domain-epoch".to_owned());
    let clock_resolution =
        parse_optional_string(source, "clock_resolution").unwrap_or_else(|| "tick:1ns".to_owned());
    let clock_bridge_default =
        parse_optional_string(source, "clock_bridge_default").unwrap_or_else(|| "self".to_owned());
    let profiles = parse_string_array(source, "profiles", path)?;
    let resource_families = parse_string_array(source, "resource_families", path)?;
    let unit_types = parse_optional_string_array(source, "unit_types").unwrap_or_default();
    let lowering_targets = parse_string_array(source, "lowering_targets", path)?;
    let ops = parse_string_array(source, "ops", path)?;

    Ok(NustarPackageManifest {
        manifest_schema,
        package_id,
        domain_family,
        frontend,
        entry_crate,
        ast_entry,
        nir_entry,
        yir_lowering_entry,
        part_verify_entry,
        ast_surface,
        nir_surface,
        yir_lowering,
        part_verify,
        binary_extension,
        package_layout,
        machine_abi_policy,
        abi_profiles,
        abi_capabilities,
        abi_targets,
        implementation_kinds,
        loader_entry,
        loader_abi,
        host_ffi_surface,
        host_ffi_abis,
        host_ffi_bridge,
        bridge_lane_policy,
        bridge_surface,
        bridge_emission_kind,
        bridge_entry,
        bridge_kind,
        bridge_scheduler_binding,
        backend_stub_kind,
        backend_submission_mode,
        backend_wake_policy,
        backend_transport_model,
        backend_request_shape,
        backend_response_shape,
        backend_dispatch_shape,
        backend_memory_binding,
        backend_resource_binding,
        backend_completion_model,
        phase_bind,
        phase_submit,
        phase_wait,
        phase_finalize,
        host_bridge_host_ffi_surface,
        host_bridge_handle_family,
        host_bridge_phase_order,
        host_bridge_phase_bind_inputs,
        host_bridge_phase_bind_outputs,
        host_bridge_phase_submit_inputs,
        host_bridge_phase_submit_outputs,
        host_bridge_phase_wait_inputs,
        host_bridge_phase_wait_outputs,
        host_bridge_phase_finalize_inputs,
        host_bridge_phase_finalize_outputs,
        host_bridge_phase_bind_wake,
        host_bridge_phase_submit_wake,
        host_bridge_phase_wait_wake,
        host_bridge_phase_finalize_wake,
        host_bridge_plan_begin,
        host_bridge_plan_end,
        support_surface,
        support_profile_slots,
        capability_tags,
        default_lanes,
        clock_domain_id,
        clock_kind,
        clock_epoch_kind,
        clock_resolution,
        clock_bridge_default,
        profiles,
        resource_families,
        unit_types,
        lowering_targets,
        ops,
    })
}

fn parse_registered_abi_target(
    abi: &str,
    fields: &str,
    manifest: &NustarPackageManifest,
    raw: &str,
) -> Result<RegisteredAbiTarget, String> {
    let mut host_adaptive = false;
    let mut machine_arch = None::<String>;
    let mut machine_os = None::<String>;
    let mut object_format = None::<String>;
    let mut calling_abi = None::<String>;
    let mut clang_target = None::<String>;
    let mut backend_family = None::<String>;
    let mut vendor = None::<String>;
    let mut device_class = None::<String>;
    for field in fields
        .split('|')
        .map(str::trim)
        .filter(|field| !field.is_empty())
    {
        let Some((key, value)) = field.split_once('=') else {
            return Err(format!(
                "nustar package `{}` has invalid abi_targets field `{}` in `{}`; expected `key=value`",
                manifest.package_id, field, raw
            ));
        };
        let value = value.trim();
        if value == "host" {
            host_adaptive = true;
        }
        match key.trim() {
            "arch" => machine_arch = Some(resolve_host_adaptive_arch(value).to_owned()),
            "os" => machine_os = Some(resolve_host_adaptive_os(value).to_owned()),
            "object" => object_format = Some(resolve_host_adaptive_object(value).to_owned()),
            "calling" => calling_abi = Some(resolve_host_adaptive_calling(value).to_owned()),
            "clang" => clang_target = Some(resolve_host_adaptive_clang(value).to_owned()),
            "backend" => backend_family = Some(value.to_owned()),
            "vendor" => vendor = Some(value.to_owned()),
            "device" => device_class = Some(value.to_owned()),
            other => {
                return Err(format!(
                    "nustar package `{}` has invalid abi_targets key `{}` in `{}`; expected `arch`, `os`, `object`, `calling`, `clang`, `backend`, `vendor`, or `device`",
                    manifest.package_id, other, raw
                ));
            }
        }
    }
    Ok(RegisteredAbiTarget {
        abi: abi.to_owned(),
        machine_arch: machine_arch.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `arch=`",
                manifest.package_id, raw
            )
        })?,
        machine_os: machine_os.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `os=`",
                manifest.package_id, raw
            )
        })?,
        object_format: object_format.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `object=`",
                manifest.package_id, raw
            )
        })?,
        calling_abi: calling_abi.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `calling=`",
                manifest.package_id, raw
            )
        })?,
        clang_target: clang_target.ok_or_else(|| {
            format!(
                "nustar package `{}` abi_targets entry `{}` is missing `clang=`",
                manifest.package_id, raw
            )
        })?,
        backend_family,
        vendor,
        device_class,
        host_adaptive,
    })
}

fn resolve_host_adaptive_arch(value: &str) -> &'static str {
    if value == "host" {
        host_arch()
    } else {
        match value {
            "arm64" => "arm64",
            "amd64" => "x86_64",
            "x86_64" => "x86_64",
            other => Box::leak(other.to_owned().into_boxed_str()),
        }
    }
}

fn resolve_host_adaptive_os(value: &str) -> &'static str {
    if value == "host" {
        host_os()
    } else {
        match value {
            "darwin" => "darwin",
            "linux" => "linux",
            "windows" => "windows",
            other => Box::leak(other.to_owned().into_boxed_str()),
        }
    }
}

fn resolve_host_adaptive_object(value: &str) -> &'static str {
    if value == "host" {
        host_object_format()
    } else {
        match value {
            "mach-o" => "mach-o",
            "elf" => "elf",
            "coff" => "coff",
            other => Box::leak(other.to_owned().into_boxed_str()),
        }
    }
}

fn resolve_host_adaptive_calling(value: &str) -> &'static str {
    if value == "host" {
        host_calling_abi()
    } else {
        match value {
            "aapcs64-darwin" => "aapcs64-darwin",
            "aapcs64" => "aapcs64",
            "sysv64" => "sysv64",
            "win64" => "win64",
            other => Box::leak(other.to_owned().into_boxed_str()),
        }
    }
}

fn resolve_host_adaptive_clang(value: &str) -> String {
    if value == "host" {
        host_clang_target()
    } else {
        value.to_owned()
    }
}

fn host_arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        "amd64" => "x86_64",
        other => Box::leak(other.to_owned().into_boxed_str()),
    }
}

fn host_os() -> &'static str {
    match std::env::consts::OS {
        "macos" => "darwin",
        other => Box::leak(other.to_owned().into_boxed_str()),
    }
}

fn host_object_format() -> &'static str {
    match std::env::consts::OS {
        "macos" => "mach-o",
        "linux" => "elf",
        "windows" => "coff",
        other => Box::leak(other.to_owned().into_boxed_str()),
    }
}

fn host_calling_abi() -> &'static str {
    match (host_arch(), host_os()) {
        ("arm64", "darwin") => "aapcs64-darwin",
        ("arm64", _) => "aapcs64",
        ("x86_64", "windows") => "win64",
        ("x86_64", _) => "sysv64",
        _ => "unknown",
    }
}

fn host_clang_target() -> String {
    match (host_arch(), host_os()) {
        ("arm64", "darwin") => "aarch64-apple-darwin".to_owned(),
        ("arm64", "linux") => "aarch64-unknown-linux-gnu".to_owned(),
        ("x86_64", "darwin") => "x86_64-apple-darwin".to_owned(),
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu".to_owned(),
        ("x86_64", "windows") => "x86_64-pc-windows-msvc".to_owned(),
        (arch, os) => format!("{arch}-unknown-{os}"),
    }
}

fn parse_optional_string(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_quoted(rest);
        }
    }
    None
}

fn parse_optional_bool(source: &str, key: &str) -> Option<bool> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return match rest.trim() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            };
        }
    }
    None
}

fn parse_required_string(source: &str, key: &str, path: &Path) -> Result<String, String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_quoted(rest).ok_or_else(|| {
                format!(
                    "manifest `{}` has invalid string value for `{key}`",
                    path.display()
                )
            });
        }
    }

    Err(format!(
        "manifest `{}` is missing required key `{key}`",
        path.display()
    ))
}

fn parse_string_array(source: &str, key: &str, path: &Path) -> Result<Vec<String>, String> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_array(rest).ok_or_else(|| {
                format!(
                    "manifest `{}` has invalid array value for `{key}`",
                    path.display()
                )
            });
        }
    }

    Err(format!(
        "manifest `{}` is missing required key `{key}`",
        path.display()
    ))
}

fn parse_optional_string_array(source: &str, key: &str) -> Option<Vec<String>> {
    let prefix = format!("{key} = ");
    for raw_line in source.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            return parse_array(rest);
        }
    }
    None
}

fn parse_quoted(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        Some(trimmed[1..trimmed.len() - 1].to_owned())
    } else {
        None
    }
}

fn parse_array(raw: &str) -> Option<Vec<String>> {
    let trimmed = raw.trim();
    if !(trimmed.starts_with('[') && trimmed.ends_with(']')) {
        return None;
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    if inner.trim().is_empty() {
        return Some(Vec::new());
    }

    let mut items = Vec::new();
    for part in inner.split(',') {
        items.push(parse_quoted(part.trim())?);
    }
    Some(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{
        ProjectAbiRequirement, ProjectAbiResolution, ProjectCompilationPlan,
        ProjectExchangeOrganization, ProjectOrganization, ProjectOutputIntent,
        ProjectSyntheticInput,
    };
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_project_plan(domain: &str, abi: &str) -> ProjectCompilationPlan {
        ProjectCompilationPlan {
            project_name: "registry-check-demo".to_owned(),
            entry: "main.ns".to_owned(),
            organization: ProjectOrganization {
                entry: "main.ns".to_owned(),
                domains: vec![domain.to_owned()],
                modules: Vec::new(),
                links: Vec::new(),
            },
            exchanges: ProjectExchangeOrganization { routes: Vec::new() },
            abi_resolution: ProjectAbiResolution {
                requirements: vec![ProjectAbiRequirement {
                    domain: domain.to_owned(),
                    abi: abi.to_owned(),
                }],
                explicit: true,
            },
            dependencies: Vec::new(),
            synthetic_input: ProjectSyntheticInput {
                kind: "test".to_owned(),
                path: PathBuf::from("main.ns"),
            },
            output_intents: Vec::<ProjectOutputIntent>::new(),
            effective_input_path: PathBuf::from("main.ns"),
        }
    }
    use crate::pipeline;

    const DATA_BINDING_SOURCE: &str = r#"
use data FabricPlane;

mod cpu Main {
  fn capture_data_profile_summary() -> i64 {
    let bind_core: Unit = data_profile_bind_core("FabricPlane");
    let window_offset: i64 = data_profile_window_offset("FabricPlane");
    let uplink_len: i64 = data_profile_uplink_len("FabricPlane");
    let downlink_len: i64 = data_profile_downlink_len("FabricPlane");
    let _ = bind_core;
    return window_offset + uplink_len + downlink_len;
  }

  fn main() {
    print(capture_data_profile_summary());
  }
}
"#;

    fn binding_plan_from_source(source: &str) -> NustarBindingPlan {
        let artifacts = pipeline::compile_source(source).expect("source should compile");
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

        let registry_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("nustar-packages");
        plan_bindings(
            &registry_root,
            &artifacts.nir,
            &artifacts.yir,
            &artifacts.ast.domain,
            &artifacts.ast.unit,
            &declared_used_units,
            &declared_externs,
        )
        .expect("binding plan should resolve")
    }

    fn cpu_manifest_with_host_target() -> NustarPackageManifest {
        NustarPackageManifest {
            manifest_schema: "nustar-manifest-v1".to_owned(),
            package_id: "official.cpu".to_owned(),
            domain_family: "cpu".to_owned(),
            frontend: "nustar-cpu".to_owned(),
            entry_crate: "crates/yir-domain-cpu".to_owned(),
            ast_entry: "cpu.ast.bootstrap.v1".to_owned(),
            nir_entry: "cpu.nir.bootstrap.v1".to_owned(),
            yir_lowering_entry: "cpu.yir.lowering.v1".to_owned(),
            part_verify_entry: "cpu.verify.partial.v1".to_owned(),
            ast_surface: vec!["cpu.mod-ast.v1".to_owned()],
            nir_surface: vec!["nir.cpu.surface.v1".to_owned()],
            yir_lowering: vec!["yir.cpu.lowering.v1".to_owned()],
            part_verify: vec!["verify.cpu.contract.v1".to_owned()],
            binary_extension: "nustar".to_owned(),
            package_layout: "single-envelope".to_owned(),
            machine_abi_policy: "exact-match".to_owned(),
            abi_profiles: vec!["cpu.host.v1".to_owned()],
            abi_capabilities: vec!["cpu.host.v1:op:cpu.*".to_owned()],
            abi_targets: vec![
                "cpu.host.v1:arch=host|os=host|object=host|calling=host|clang=host".to_owned(),
            ],
            implementation_kinds: vec!["native-stub".to_owned()],
            loader_entry: "nustar.bootstrap.v1".to_owned(),
            loader_abi: "nustar-loader-v1".to_owned(),
            host_ffi_surface: Vec::new(),
            host_ffi_abis: Vec::new(),
            host_ffi_bridge: "none".to_owned(),
            bridge_lane_policy: None,
            bridge_surface: None,
            bridge_emission_kind: None,
            bridge_entry: None,
            bridge_kind: None,
            bridge_scheduler_binding: None,
            backend_stub_kind: None,
            backend_submission_mode: None,
            backend_wake_policy: None,
            backend_transport_model: None,
            backend_request_shape: None,
            backend_response_shape: None,
            backend_dispatch_shape: None,
            backend_memory_binding: None,
            backend_resource_binding: None,
            backend_completion_model: None,
            phase_bind: None,
            phase_submit: None,
            phase_wait: None,
            phase_finalize: None,
            host_bridge_host_ffi_surface: None,
            host_bridge_handle_family: None,
            host_bridge_phase_order: None,
            host_bridge_phase_bind_inputs: None,
            host_bridge_phase_bind_outputs: None,
            host_bridge_phase_submit_inputs: None,
            host_bridge_phase_submit_outputs: None,
            host_bridge_phase_wait_inputs: None,
            host_bridge_phase_wait_outputs: None,
            host_bridge_phase_finalize_inputs: None,
            host_bridge_phase_finalize_outputs: None,
            host_bridge_phase_bind_wake: None,
            host_bridge_phase_submit_wake: None,
            host_bridge_phase_wait_wake: None,
            host_bridge_phase_finalize_wake: None,
            host_bridge_plan_begin: None,
            host_bridge_plan_end: None,
            support_surface: Vec::new(),
            support_profile_slots: Vec::new(),
            capability_tags: Vec::new(),
            default_lanes: Vec::new(),
            clock_domain_id: "cpu.clock.host.v1".to_owned(),
            clock_kind: "host-monotonic".to_owned(),
            clock_epoch_kind: "host-epoch".to_owned(),
            clock_resolution: "cpu.tick_i64".to_owned(),
            clock_bridge_default: "global->monotonic:bridge".to_owned(),
            profiles: vec!["aot".to_owned()],
            resource_families: vec!["cpu".to_owned()],
            unit_types: vec!["Main".to_owned()],
            lowering_targets: vec!["llvm".to_owned()],
            ops: vec!["cpu.const".to_owned()],
        }
    }

    fn render_manifest_text(manifest: &NustarPackageManifest) -> String {
        fn render_array(values: &[String]) -> String {
            format!(
                "[{}]",
                values
                    .iter()
                    .map(|value| format!("\"{value}\""))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }

        fn render_optional_string(value: Option<&str>) -> String {
            match value {
                Some(value) => format!("\"{value}\""),
                None => "null".to_owned(),
            }
        }

        fn render_optional_array(value: Option<&[String]>) -> String {
            match value {
                Some(values) => render_array(values),
                None => "null".to_owned(),
            }
        }

        fn render_optional_bool(value: Option<bool>) -> String {
            match value {
                Some(true) => "true".to_owned(),
                Some(false) => "false".to_owned(),
                None => "null".to_owned(),
            }
        }

        format!(
            concat!(
                "manifest_schema = \"{}\"\n",
                "package_id = \"{}\"\n",
                "domain_family = \"{}\"\n",
                "frontend = \"{}\"\n",
                "entry_crate = \"{}\"\n",
                "ast_entry = \"{}\"\n",
                "nir_entry = \"{}\"\n",
                "yir_lowering_entry = \"{}\"\n",
                "part_verify_entry = \"{}\"\n",
                "ast_surface = {}\n",
                "nir_surface = {}\n",
                "yir_lowering = {}\n",
                "part_verify = {}\n",
                "binary_extension = \"{}\"\n",
                "package_layout = \"{}\"\n",
                "machine_abi_policy = \"{}\"\n",
                "abi_profiles = {}\n",
                "abi_capabilities = {}\n",
                "abi_targets = {}\n",
                "implementation_kinds = {}\n",
                "loader_entry = \"{}\"\n",
                "loader_abi = \"{}\"\n",
                "host_ffi_surface = {}\n",
                "host_ffi_abis = {}\n",
                "host_ffi_bridge = \"{}\"\n",
                "bridge_lane_policy = {}\n",
                "bridge_surface = {}\n",
                "bridge_emission_kind = {}\n",
                "bridge_entry = {}\n",
                "bridge_kind = {}\n",
                "bridge_scheduler_binding = {}\n",
                "backend_stub_kind = {}\n",
                "backend_submission_mode = {}\n",
                "backend_wake_policy = {}\n",
                "backend_transport_model = {}\n",
                "backend_request_shape = {}\n",
                "backend_response_shape = {}\n",
                "backend_dispatch_shape = {}\n",
                "backend_memory_binding = {}\n",
                "backend_resource_binding = {}\n",
                "backend_completion_model = {}\n",
                "phase_bind = {}\n",
                "phase_submit = {}\n",
                "phase_wait = {}\n",
                "phase_finalize = {}\n",
                "host_bridge_host_ffi_surface = {}\n",
                "host_bridge_handle_family = {}\n",
                "host_bridge_phase_order = {}\n",
                "host_bridge_phase_bind_inputs = {}\n",
                "host_bridge_phase_bind_outputs = {}\n",
                "host_bridge_phase_submit_inputs = {}\n",
                "host_bridge_phase_submit_outputs = {}\n",
                "host_bridge_phase_wait_inputs = {}\n",
                "host_bridge_phase_wait_outputs = {}\n",
                "host_bridge_phase_finalize_inputs = {}\n",
                "host_bridge_phase_finalize_outputs = {}\n",
                "host_bridge_phase_bind_wake = {}\n",
                "host_bridge_phase_submit_wake = {}\n",
                "host_bridge_phase_wait_wake = {}\n",
                "host_bridge_phase_finalize_wake = {}\n",
                "host_bridge_plan_begin = {}\n",
                "host_bridge_plan_end = {}\n",
                "support_surface = {}\n",
                "support_profile_slots = {}\n",
                "capability_tags = {}\n",
                "default_lanes = {}\n",
                "clock_domain_id = \"{}\"\n",
                "clock_kind = \"{}\"\n",
                "clock_epoch_kind = \"{}\"\n",
                "clock_resolution = \"{}\"\n",
                "clock_bridge_default = \"{}\"\n",
                "profiles = {}\n",
                "resource_families = {}\n",
                "unit_types = {}\n",
                "lowering_targets = {}\n",
                "ops = {}\n"
            ),
            manifest.manifest_schema,
            manifest.package_id,
            manifest.domain_family,
            manifest.frontend,
            manifest.entry_crate,
            manifest.ast_entry,
            manifest.nir_entry,
            manifest.yir_lowering_entry,
            manifest.part_verify_entry,
            render_array(&manifest.ast_surface),
            render_array(&manifest.nir_surface),
            render_array(&manifest.yir_lowering),
            render_array(&manifest.part_verify),
            manifest.binary_extension,
            manifest.package_layout,
            manifest.machine_abi_policy,
            render_array(&manifest.abi_profiles),
            render_array(&manifest.abi_capabilities),
            render_array(&manifest.abi_targets),
            render_array(&manifest.implementation_kinds),
            manifest.loader_entry,
            manifest.loader_abi,
            render_array(&manifest.host_ffi_surface),
            render_array(&manifest.host_ffi_abis),
            manifest.host_ffi_bridge,
            render_optional_string(manifest.bridge_lane_policy.as_deref()),
            render_optional_string(manifest.bridge_surface.as_deref()),
            render_optional_string(manifest.bridge_emission_kind.as_deref()),
            render_optional_string(manifest.bridge_entry.as_deref()),
            render_optional_string(manifest.bridge_kind.as_deref()),
            render_optional_string(manifest.bridge_scheduler_binding.as_deref()),
            render_optional_string(manifest.backend_stub_kind.as_deref()),
            render_optional_string(manifest.backend_submission_mode.as_deref()),
            render_optional_string(manifest.backend_wake_policy.as_deref()),
            render_optional_string(manifest.backend_transport_model.as_deref()),
            render_optional_string(manifest.backend_request_shape.as_deref()),
            render_optional_string(manifest.backend_response_shape.as_deref()),
            render_optional_string(manifest.backend_dispatch_shape.as_deref()),
            render_optional_string(manifest.backend_memory_binding.as_deref()),
            render_optional_string(manifest.backend_resource_binding.as_deref()),
            render_optional_string(manifest.backend_completion_model.as_deref()),
            render_optional_string(manifest.phase_bind.as_deref()),
            render_optional_string(manifest.phase_submit.as_deref()),
            render_optional_string(manifest.phase_wait.as_deref()),
            render_optional_string(manifest.phase_finalize.as_deref()),
            render_optional_array(manifest.host_bridge_host_ffi_surface.as_deref()),
            render_optional_array(manifest.host_bridge_handle_family.as_deref()),
            render_optional_array(manifest.host_bridge_phase_order.as_deref()),
            render_optional_array(manifest.host_bridge_phase_bind_inputs.as_deref()),
            render_optional_array(manifest.host_bridge_phase_bind_outputs.as_deref()),
            render_optional_array(manifest.host_bridge_phase_submit_inputs.as_deref()),
            render_optional_array(manifest.host_bridge_phase_submit_outputs.as_deref()),
            render_optional_array(manifest.host_bridge_phase_wait_inputs.as_deref()),
            render_optional_array(manifest.host_bridge_phase_wait_outputs.as_deref()),
            render_optional_array(manifest.host_bridge_phase_finalize_inputs.as_deref()),
            render_optional_array(manifest.host_bridge_phase_finalize_outputs.as_deref()),
            render_optional_string(manifest.host_bridge_phase_bind_wake.as_deref()),
            render_optional_string(manifest.host_bridge_phase_submit_wake.as_deref()),
            render_optional_string(manifest.host_bridge_phase_wait_wake.as_deref()),
            render_optional_string(manifest.host_bridge_phase_finalize_wake.as_deref()),
            render_optional_bool(manifest.host_bridge_plan_begin),
            render_optional_bool(manifest.host_bridge_plan_end),
            render_array(&manifest.support_surface),
            render_array(&manifest.support_profile_slots),
            render_array(&manifest.capability_tags),
            render_array(&manifest.default_lanes),
            manifest.clock_domain_id,
            manifest.clock_kind,
            manifest.clock_epoch_kind,
            manifest.clock_resolution,
            manifest.clock_bridge_default,
            render_array(&manifest.profiles),
            render_array(&manifest.resource_families),
            render_array(&manifest.unit_types),
            render_array(&manifest.lowering_targets),
            render_array(&manifest.ops),
        )
    }

    fn temp_registry_root(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("nuisc-{label}-{nanos}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_registry_fixture(
        root: &Path,
        entries: &[NustarPackageIndexEntry],
        manifests: &[NustarPackageManifest],
    ) {
        let mut index_text = String::new();
        for entry in entries {
            index_text.push_str("[[package]]\n");
            index_text.push_str(&format!("package_id = \"{}\"\n", entry.package_id));
            index_text.push_str(&format!("manifest = \"{}\"\n", entry.manifest));
            index_text.push_str(&format!("domain_family = \"{}\"\n\n", entry.domain_family));
        }
        fs::write(root.join(INDEX_FILE), index_text).unwrap();
        for (entry, manifest) in entries.iter().zip(manifests.iter()) {
            fs::write(root.join(&entry.manifest), render_manifest_text(manifest)).unwrap();
        }
    }

    #[test]
    fn registered_abi_target_expands_host_adaptive_contract() {
        let manifest = cpu_manifest_with_host_target();
        let target = registered_abi_target(&manifest, "cpu.host.v1").unwrap();
        assert_eq!(target.machine_arch, host_arch());
        assert_eq!(target.machine_os, host_os());
        assert_eq!(target.object_format, host_object_format());
        assert_eq!(target.calling_abi, host_calling_abi());
        assert_eq!(target.clang_target, host_clang_target());
        assert!(target.host_adaptive);
    }

    #[test]
    fn registered_abi_target_preserves_backend_family() {
        let mut manifest = cpu_manifest_with_host_target();
        manifest.abi_profiles = vec!["cpu.backend.v1".to_owned()];
        manifest.abi_capabilities = vec!["cpu.backend.v1:op:cpu.*".to_owned()];
        manifest.abi_targets = vec![
            "cpu.backend.v1:arch=arm64|os=darwin|object=mach-o|calling=aapcs64-darwin|clang=aarch64-apple-darwin|backend=metal|vendor=apple|device=apple-silicon-gpu".to_owned(),
        ];
        let target = registered_abi_target(&manifest, "cpu.backend.v1").unwrap();
        assert_eq!(target.backend_family.as_deref(), Some("metal"));
        assert_eq!(target.vendor.as_deref(), Some("apple"));
        assert_eq!(target.device_class.as_deref(), Some("apple-silicon-gpu"));
        assert!(!target.host_adaptive);
    }

    #[test]
    fn network_manifest_skeleton_is_registered() {
        let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
        assert_eq!(manifest.package_id, "official.network");
        assert_eq!(manifest.clock_domain_id, "network.clock.io.v1");
        assert_eq!(manifest.clock_kind, "io-monotonic");
        assert!(manifest
            .support_surface
            .contains(&"network.profile.bind-core.v1".to_owned()));
        assert!(manifest
            .support_surface
            .contains(&"network.profile.connect.v1".to_owned()));
        assert!(manifest
            .support_surface
            .contains(&"network.profile.stream-window.v1".to_owned()));
        assert!(manifest
            .support_surface
            .contains(&"network.profile.transport.v1".to_owned()));
        assert!(manifest
            .support_profile_slots
            .contains(&"bind_core".to_owned()));
        assert!(manifest
            .support_profile_slots
            .contains(&"endpoint_kind".to_owned()));
        assert!(manifest
            .support_profile_slots
            .contains(&"transport_family".to_owned()));
        assert!(manifest
            .support_profile_slots
            .contains(&"retry_budget".to_owned()));
        assert!(manifest
            .support_profile_slots
            .contains(&"stream_window".to_owned()));
        assert!(manifest
            .support_profile_slots
            .contains(&"protocol_kind".to_owned()));
        assert!(manifest
            .default_lanes
            .contains(&"network.send=tx".to_owned()));
        assert!(manifest
            .default_lanes
            .contains(&"network.recv=rx".to_owned()));
    }

    #[test]
    fn cpu_manifest_contract_is_registered() {
        let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "cpu").unwrap();
        assert_eq!(manifest.package_id, "official.cpu");
        assert_eq!(manifest.loader_abi, "nustar-loader-v1");
        assert_eq!(manifest.loader_entry, "nustar.bootstrap.v1");
        assert_eq!(manifest.machine_abi_policy, "exact-match");
        assert_eq!(manifest.clock_domain_id, "cpu.clock.host.v1");
        assert_eq!(manifest.clock_kind, "host-monotonic");
        assert_eq!(manifest.clock_bridge_default, "global->monotonic:bridge");
        assert!(manifest
            .host_ffi_surface
            .contains(&"cpu.host-ffi.nurs.v1".to_owned()));
        assert!(manifest
            .host_ffi_surface
            .contains(&"cpu.host-ffi.c-bridge.v1".to_owned()));
        assert!(manifest.host_ffi_abis.contains(&"nurs".to_owned()));
        assert!(manifest.host_ffi_abis.contains(&"c".to_owned()));
        assert!(manifest
            .default_lanes
            .contains(&"cpu.window=main".to_owned()));
        assert!(manifest
            .default_lanes
            .contains(&"cpu.alloc_node=mem".to_owned()));
        assert!(manifest
            .default_lanes
            .contains(&"cpu.instantiate_unit=main".to_owned()));
        assert!(manifest
            .abi_profiles
            .contains(&"cpu.arm64.apple_aapcs64".to_owned()));
    }

    #[test]
    fn scheduler_summary_uses_manifest_clock_and_domain_samples() {
        let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
        let summary = scheduler_summary(&manifest);
        assert_eq!(summary.clock.domain_id, "network.clock.io.v1");
        assert_eq!(summary.clock.kind, "io-monotonic");
        assert_eq!(
            summary.sample_navigation.as_deref(),
            Some(
                "result_ladder -> transport_split_ladder -> transport_summary_ladder -> summary_classes"
            )
        );
        assert!(summary
            .result_samples
            .as_deref()
            .unwrap_or_default()
            .contains("network_result_profile_demo"));
        assert!(summary
            .transport_samples
            .as_deref()
            .unwrap_or_default()
            .contains("network_transport_result_policy_split_demo"));
    }

    #[test]
    fn capability_summary_tracks_support_and_clock_contract() {
        let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
        let summary = capability_summary(&manifest);
        assert!(summary
            .support_surface
            .contains(&"network.profile.transport.v1".to_owned()));
        assert!(summary
            .support_profile_slots
            .contains(&"protocol_kind".to_owned()));
        assert!(summary.capability_tags.contains(&"io-reactor".to_owned()));
        assert!(summary
            .capability_tags
            .contains(&"protocol-framing".to_owned()));
        assert!(summary
            .default_lanes
            .contains(&"network.send=tx".to_owned()));
        assert_eq!(summary.clock.domain_id, "network.clock.io.v1");
        assert_eq!(summary.clock.bridge_default, "global->io:bridge");
    }

    #[test]
    fn execution_summary_derives_minimum_execution_skeleton() {
        let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "kernel").unwrap();
        let summary = execution_summary(&manifest);
        assert_eq!(summary.skeleton_version, "nustar-execution-skeleton-v1");
        assert_eq!(summary.function_kind, "function-node");
        assert_eq!(summary.graph_kind, "function-graph");
        assert_eq!(summary.execution_domain, "kernel");
        assert_eq!(summary.default_time_mode, "logical");
        assert_eq!(summary.contract_family, "nustar.kernel");
        assert!(summary.lowering_targets.contains(&"coreml".to_owned()));
    }

    #[test]
    fn domain_build_contract_summary_prefers_manifest_registered_bridge_fields() {
        let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
        assert_eq!(
            manifest.bridge_lane_policy.as_deref(),
            Some("dispatch-lanes.io-bound")
        );
        assert_eq!(
            manifest.bridge_surface.as_deref(),
            Some("host-ffi.bridge.network")
        );
        assert_eq!(
            manifest.bridge_entry.as_deref(),
            Some("nuis.network.bridge.dispatch.v1")
        );
        assert_eq!(
            manifest.bridge_scheduler_binding.as_deref(),
            Some("network-poll-bridge")
        );
        assert_eq!(
            manifest.bridge_emission_kind.as_deref(),
            Some("sidecar-plan")
        );
        assert_eq!(
            manifest.bridge_kind.as_deref(),
            Some("managed-lifecycle-bridge")
        );
        let summary = domain_build_contract_summary(&manifest);
        assert_eq!(summary.lowering.lane_policy, "dispatch-lanes.io-bound");
        assert_eq!(summary.lowering.bridge_surface, "host-ffi.bridge.network");
        assert_eq!(summary.lowering.emission_kind, "sidecar-plan");
        assert_eq!(summary.backend.stub_kind, "network-host-bridge");
        assert_eq!(summary.backend.submission_mode, "request-response");
        assert_eq!(summary.backend.wake_policy, "io-ready");
        assert_eq!(
            summary.backend.transport_model.as_deref(),
            Some("client-session")
        );
        assert_eq!(summary.bridge.scheduler_binding, "network-poll-bridge");
        assert_eq!(summary.bridge.phase_submit, "packet-write-dispatch");
        assert_eq!(summary.bridge.phase_wait, "callback-or-read-ready");
        assert_eq!(summary.bridge.bridge_kind, "managed-lifecycle-bridge");
        assert_eq!(summary.host_bridge.host_ffi_surface, "socket,urlsession");
        assert_eq!(
            summary.host_bridge.handle_family,
            "network.request,network.response"
        );
        assert_eq!(
            summary.host_bridge.phase_bind_inputs,
            vec![
                "request.packet".to_owned(),
                "bridge.config".to_owned(),
                "host.session".to_owned()
            ]
        );
        assert_eq!(
            summary.host_bridge.phase_submit_outputs,
            vec!["inflight.request".to_owned(), "poll.token".to_owned()]
        );
        assert_eq!(summary.host_bridge.phase_wait_wake, "io-ready");
        assert!(summary.host_bridge.bridge_plan_begin);
        assert!(summary.host_bridge.bridge_plan_end);
    }

    #[test]
    fn domain_contract_collects_registered_runtime_and_loader_facts() {
        let manifest = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
        let contract = domain_contract(&manifest);
        assert_eq!(contract.contract_schema, NUSTAR_DOMAIN_CONTRACT_SCHEMA);
        assert!(contract
            .contract_groups
            .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_PACKAGE_IDENTITY.to_owned()));
        assert!(contract
            .contract_groups
            .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_LOADER.to_owned()));
        assert!(contract
            .contract_groups
            .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_ABI.to_owned()));
        assert!(contract
            .contract_groups
            .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_RUNTIME.to_owned()));
        assert!(contract
            .contract_groups
            .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_EXECUTION.to_owned()));
        assert!(contract
            .contract_groups
            .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_SCHEDULER.to_owned()));
        assert!(contract
            .extension_groups
            .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET.to_owned()));
        assert_eq!(contract.package_id, "official.network");
        assert_eq!(contract.domain_family, "network");
        assert_eq!(contract.frontend, "nustar-network");
        assert_eq!(contract.loader_abi, "nustar-loader-v1");
        assert_eq!(contract.loader_entry, "nustar.bootstrap.v1");
        assert_eq!(contract.machine_abi_policy, "exact-match");
        assert!(contract
            .abi_profiles
            .contains(&"network.socket.v1".to_owned()));
        assert!(contract
            .capability
            .support_surface
            .contains(&"network.profile.transport.v1".to_owned()));
        assert!(contract
            .capability
            .capability_tags
            .contains(&"socket-transport".to_owned()));
        assert_eq!(contract.execution.execution_domain, "network");
        assert_eq!(contract.execution.contract_family, "nustar.network");
        assert_eq!(contract.scheduler.clock.domain_id, "network.clock.io.v1");
        assert!(contract
            .std_net
            .recipe_samples
            .as_deref()
            .unwrap_or_default()
            .contains("net_http_client_recipe"));
        let json = domain_contract_json(&contract);
        assert!(json.contains("\"execution_skeleton_version\":\"nustar-execution-skeleton-v1\""));
        assert!(json.contains("\"execution_contract_family\":\"nustar.network\""));
        assert!(json.contains("\"capability_tags\":[\"io-reactor\""));
    }

    #[test]
    fn std_net_summary_is_owned_by_registry() {
        let summary = std_net_summary("network");
        assert_eq!(
            summary.sample_navigation.as_deref(),
            Some(
                "profile_core -> transport_edge -> syscall_edge -> socket_edge -> control_edge -> protocol_edge -> http_edge -> result_spine -> task_spine -> session"
            )
        );
        assert!(summary
            .recipe_samples
            .as_deref()
            .unwrap_or_default()
            .contains("net_http_client_recipe"));
    }

    #[test]
    fn load_registered_domains_covers_all_indexed_nustar_modules() {
        let registrations = load_registered_domains(Path::new("nustar-packages")).unwrap();
        let domains = registrations
            .iter()
            .map(|item| item.domain_family.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            domains,
            vec!["cpu", "cpu", "data", "kernel", "network", "shader"]
        );
        let cpu_packages = registrations
            .iter()
            .filter(|item| item.domain_family == "cpu")
            .map(|item| item.package_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(cpu_packages, vec!["official.cpu", "official.cpu.aarch64"]);
        let network = registrations
            .iter()
            .find(|item| item.domain_family == "network")
            .unwrap();
        assert!(network
            .manifest_path
            .ends_with("nustar-packages/network.toml"));
        assert_eq!(
            network.contract.contract_schema,
            NUSTAR_DOMAIN_CONTRACT_SCHEMA
        );
        assert!(network
            .contract
            .extension_groups
            .contains(&NUSTAR_DOMAIN_CONTRACT_GROUP_STD_NET.to_owned()));
        assert!(!network.ops.is_empty());
    }

    #[test]
    fn aarch64_cpu_nustar_is_independent_package_for_cpu_domain() {
        let generic_cpu = load_manifest_for_domain(Path::new("nustar-packages"), "cpu").unwrap();
        assert_eq!(generic_cpu.package_id, "official.cpu");
        assert!(generic_cpu
            .abi_profiles
            .contains(&"cpu.x86_64.sysv64".to_owned()));

        let aarch64_cpu =
            load_manifest(Path::new("nustar-packages"), "official.cpu.aarch64").unwrap();
        assert_eq!(aarch64_cpu.domain_family, "cpu");
        assert_eq!(aarch64_cpu.package_id, "official.cpu.aarch64");
        assert!(aarch64_cpu
            .capability_tags
            .contains(&"formal-verification-ready".to_owned()));
        assert!(aarch64_cpu
            .capability_tags
            .contains(&"aarch64-only".to_owned()));
        assert!(aarch64_cpu
            .part_verify
            .contains(&"verify.cpu.aarch64.call-frame.v1".to_owned()));
        assert!(aarch64_cpu
            .abi_profiles
            .iter()
            .all(|abi| abi.starts_with("cpu.arm64.")));
        assert!(aarch64_cpu
            .lowering_targets
            .contains(&"aarch64-proof-skeleton".to_owned()));
    }

    #[test]
    fn validate_registered_domains_accepts_current_mainline_registry() {
        let issues = validate_registered_domains(Path::new("nustar-packages")).unwrap();
        assert!(issues.is_empty(), "unexpected registry issues: {issues:?}");
        ensure_registered_domains_valid(Path::new("nustar-packages")).unwrap();
    }

    #[test]
    fn validate_registered_domains_allows_duplicate_domain_but_rejects_bad_lane_target() {
        let root = temp_registry_root("registry-duplicate-domain");
        let cpu = cpu_manifest_with_host_target();
        let mut network =
            load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
        network.default_lanes.push("network.ghost=rx".to_owned());
        let entries = vec![
            NustarPackageIndexEntry {
                package_id: cpu.package_id.clone(),
                manifest: "cpu.toml".to_owned(),
                domain_family: cpu.domain_family.clone(),
            },
            NustarPackageIndexEntry {
                package_id: network.package_id.clone(),
                manifest: "network.toml".to_owned(),
                domain_family: cpu.domain_family.clone(),
            },
        ];
        write_registry_fixture(&root, &entries, &[cpu, network]);

        let issues = validate_registered_domains(&root).unwrap();
        assert!(issues
            .iter()
            .any(|issue| issue.kind == NustarRegistryIssueKind::DomainFamilyMismatch));
        assert!(issues
            .iter()
            .any(|issue| issue.kind == NustarRegistryIssueKind::LaneContractMismatch));
        let error = ensure_registered_domains_valid(&root).unwrap_err();
        assert!(error.contains("NRV005"));
        assert!(error.contains("NRV010"));
    }

    #[test]
    fn validate_registered_domains_rejects_loader_and_op_contract_mismatch() {
        let root = temp_registry_root("registry-loader-op");
        let mut cpu = cpu_manifest_with_host_target();
        cpu.loader_abi = "wrong-loader".to_owned();
        cpu.ops.push("shader.draw".to_owned());
        let entries = vec![NustarPackageIndexEntry {
            package_id: cpu.package_id.clone(),
            manifest: "cpu.toml".to_owned(),
            domain_family: cpu.domain_family.clone(),
        }];
        write_registry_fixture(&root, &entries, &[cpu]);

        let issues = validate_registered_domains(&root).unwrap();
        assert!(issues
            .iter()
            .any(|issue| issue.kind == NustarRegistryIssueKind::LoaderContractMismatch));
        assert!(issues
            .iter()
            .any(|issue| issue.kind == NustarRegistryIssueKind::OpContractMismatch));
    }

    #[test]
    fn validate_registered_domains_rejects_shader_backend_without_lowering_target() {
        let root = temp_registry_root("registry-shader-backend");
        let mut shader = load_manifest_for_domain(Path::new("nustar-packages"), "shader").unwrap();
        shader
            .lowering_targets
            .retain(|target| target != "cpu-fallback");
        let entries = vec![NustarPackageIndexEntry {
            package_id: shader.package_id.clone(),
            manifest: "shader.toml".to_owned(),
            domain_family: shader.domain_family.clone(),
        }];
        write_registry_fixture(&root, &entries, &[shader]);

        let issues = validate_registered_domains(&root).unwrap();
        assert!(issues.iter().any(|issue| {
            issue.kind == NustarRegistryIssueKind::DomainContractMismatch
                && issue.message.contains("cpu-fallback")
        }));
    }

    #[test]
    fn validate_registered_domains_rejects_kernel_missing_profile_slot() {
        let root = temp_registry_root("registry-kernel-slot");
        let mut kernel = load_manifest_for_domain(Path::new("nustar-packages"), "kernel").unwrap();
        kernel
            .support_profile_slots
            .retain(|slot| slot != "batch_lanes");
        let entries = vec![NustarPackageIndexEntry {
            package_id: kernel.package_id.clone(),
            manifest: "kernel.toml".to_owned(),
            domain_family: kernel.domain_family.clone(),
        }];
        write_registry_fixture(&root, &entries, &[kernel]);

        let issues = validate_registered_domains(&root).unwrap();
        assert!(issues.iter().any(|issue| {
            issue.kind == NustarRegistryIssueKind::DomainContractMismatch
                && issue.message.contains("batch_lanes")
        }));
    }

    #[test]
    fn validate_registered_domains_rejects_network_missing_socket_lowering_target() {
        let root = temp_registry_root("registry-network-lowering");
        let mut network =
            load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
        network
            .lowering_targets
            .retain(|target| target != "socket-abi");
        let entries = vec![NustarPackageIndexEntry {
            package_id: network.package_id.clone(),
            manifest: "network.toml".to_owned(),
            domain_family: network.domain_family.clone(),
        }];
        write_registry_fixture(&root, &entries, &[network]);

        let issues = validate_registered_domains(&root).unwrap();
        assert!(issues.iter().any(|issue| {
            issue.kind == NustarRegistryIssueKind::DomainContractMismatch
                && issue.message.contains("socket-abi")
        }));
    }

    #[test]
    fn validate_registered_domains_rejects_incomplete_host_bridge_contract() {
        let root = temp_registry_root("registry-host-bridge-missing");
        let mut network =
            load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
        network.host_bridge_phase_wait_wake = None;
        let entries = vec![NustarPackageIndexEntry {
            package_id: network.package_id.clone(),
            manifest: "network.toml".to_owned(),
            domain_family: network.domain_family.clone(),
        }];
        write_registry_fixture(&root, &entries, &[network]);

        let issues = validate_registered_domains(&root).unwrap();
        assert!(issues.iter().any(|issue| {
            issue.kind == NustarRegistryIssueKind::DomainContractMismatch
                && issue.message.contains("host_bridge_phase_wait_wake")
        }));
    }

    #[test]
    fn validate_registered_domains_rejects_invalid_host_bridge_phase_order() {
        let root = temp_registry_root("registry-host-bridge-order");
        let mut kernel = load_manifest_for_domain(Path::new("nustar-packages"), "kernel").unwrap();
        kernel.host_bridge_phase_order = Some(vec![
            "bind".to_owned(),
            "wait".to_owned(),
            "submit".to_owned(),
            "finalize".to_owned(),
        ]);
        let entries = vec![NustarPackageIndexEntry {
            package_id: kernel.package_id.clone(),
            manifest: "kernel.toml".to_owned(),
            domain_family: kernel.domain_family.clone(),
        }];
        write_registry_fixture(&root, &entries, &[kernel]);

        let issues = validate_registered_domains(&root).unwrap();
        assert!(issues.iter().any(|issue| {
            issue.kind == NustarRegistryIssueKind::DomainContractMismatch
                && issue.message.contains("phase_order")
        }));
    }

    #[test]
    fn ensure_project_domain_registry_valid_accepts_registered_abi() {
        let plan = test_project_plan("network", "network.socket.macos.arm64.v1");
        let checks = validate_project_domain_registry(&plan);
        assert!(checks.iter().all(|check| check.issues.is_empty()));
        let network = checks
            .iter()
            .find(|check| check.domain == "network")
            .unwrap();
        assert_eq!(network.issue_count(), 0);
        assert!(network.summary_line().contains(": ok"));
        assert_eq!(network.abi_registered, true);
        ensure_project_domain_registry_valid(&plan).unwrap();
    }

    #[test]
    fn ensure_project_domain_registry_valid_rejects_unknown_abi() {
        let plan = test_project_plan("network", "network.socket.unknown.v1");
        let checks = validate_project_domain_registry(&plan);
        let network = checks
            .iter()
            .find(|check| check.domain == "network")
            .unwrap();
        assert!(network
            .issues
            .iter()
            .any(|issue| issue.kind == ProjectDomainRegistryIssueKind::AbiNotRegistered));
        assert!(network
            .issues
            .iter()
            .any(|issue| issue.kind.code() == "NRG003"));
        assert!(network.summary_line().contains("NRG003 abi_not_registered"));
        let error = ensure_project_domain_registry_valid(&plan).unwrap_err();
        assert!(error.contains("project domain registry validation failed"));
        assert!(error.contains("network"));
        assert!(error.contains("network.socket.unknown.v1"));
        assert!(error.contains("NRG003"));
        assert!(error.contains("abi_not_registered"));
    }

    #[test]
    fn binding_plan_carries_execution_skeleton_summary() {
        let plan = binding_plan_from_source(
            r#"
use shader SurfaceShader;

mod cpu Main {
  fn main() {
    print(0);
  }
}
"#,
        );
        let shader = plan
            .bindings
            .iter()
            .find(|binding| binding.domain_family == "shader")
            .expect("shader binding should exist");
        assert_eq!(
            shader.execution.skeleton_version,
            "nustar-execution-skeleton-v1"
        );
        assert_eq!(shader.execution.function_kind, "function-node");
        assert_eq!(shader.execution.graph_kind, "function-graph");
        assert_eq!(shader.execution.execution_domain, "shader");
        assert_eq!(shader.execution.contract_family, "nustar.shader");
        assert!(shader
            .execution
            .lowering_targets
            .contains(&"metal".to_owned()));
    }

    #[test]
    fn project_domain_registry_check_renderers_expose_codes_and_issue_counts() {
        let plan = test_project_plan("network", "network.socket.unknown.v1");
        let check = validate_project_domain_registry(&plan)
            .into_iter()
            .find(|check| check.domain == "network")
            .expect("network check");
        let json = project_domain_registry_check_json(&check);
        assert!(json.contains("\"domain\":\"network\""));
        assert!(json.contains("\"code\":\"NRG003\""));
        assert!(json.contains("\"kind\":\"abi_not_registered\""));
        let lines = render_project_domain_registry_check_lines(&check);
        assert!(!lines.is_empty());
        assert!(lines[0].contains("issues=1"));
        assert!(lines
            .iter()
            .any(|line| line.contains("NRG003 abi_not_registered")));
        let mut written = String::new();
        write_project_domain_registry_check_lines(&mut written, &check).unwrap();
        assert_eq!(written.lines().collect::<Vec<_>>(), lines);
    }

    #[test]
    fn registered_abi_target_accepts_darwin_x86_64_domain_profiles() {
        let network = load_manifest_for_domain(Path::new("nustar-packages"), "network").unwrap();
        let data = load_manifest_for_domain(Path::new("nustar-packages"), "data").unwrap();
        let shader = load_manifest_for_domain(Path::new("nustar-packages"), "shader").unwrap();

        let network_target =
            registered_abi_target(&network, "network.socket.macos.x86_64.v1").unwrap();
        assert_eq!(network_target.machine_arch, "x86_64");
        assert_eq!(network_target.machine_os, "darwin");
        assert_eq!(network_target.clang_target, "x86_64-apple-darwin");

        let data_target = registered_abi_target(&data, "data.fabric.macos.x86_64.v1").unwrap();
        assert_eq!(data_target.machine_arch, "x86_64");
        assert_eq!(data_target.machine_os, "darwin");
        assert_eq!(data_target.clang_target, "x86_64-apple-darwin");

        let shader_target = registered_abi_target(&shader, "shader.metal.x86_64.msl2_4").unwrap();
        assert_eq!(shader_target.machine_arch, "x86_64");
        assert_eq!(shader_target.machine_os, "darwin");
        assert_eq!(shader_target.clang_target, "x86_64-apple-darwin");
        assert_eq!(shader_target.backend_family.as_deref(), Some("metal"));
        assert_eq!(shader_target.vendor.as_deref(), Some("apple"));
        assert_eq!(
            shader_target.device_class.as_deref(),
            Some("mac-discrete-or-integrated-gpu")
        );
    }

    #[test]
    fn network_binding_plan_detects_profile_surfaces_and_slots() {
        let source = r#"
use network NetworkUnit;

mod cpu Main {
  fn capture_network_profile_summary() -> i64 {
    let bind_core: i64 = network_profile_bind_core("NetworkUnit");
    let endpoint_kind: i64 = network_profile_endpoint_kind("NetworkUnit");
    let timeout_budget: i64 = network_profile_timeout_budget("NetworkUnit");
    let retry_budget: i64 = network_profile_retry_budget("NetworkUnit");
    let stream_window: i64 = network_profile_stream_window("NetworkUnit");
    let recv_window: i64 = network_profile_recv_window("NetworkUnit");
    let send_window: i64 = network_profile_send_window("NetworkUnit");
    return bind_core + endpoint_kind + timeout_budget + retry_budget + stream_window + recv_window + send_window;
  }

  fn main() {
    print(capture_network_profile_summary());
  }
}
"#;
        let plan = binding_plan_from_source(source);

        let binding = plan
            .bindings
            .iter()
            .find(|binding| binding.domain_family == "network")
            .expect("network binding should be present");

        for surface in [
            "network.profile.bind-core.v1",
            "network.profile.endpoint-kind.v1",
            "network.profile.timeout.v1",
            "network.profile.retry.v1",
            "network.profile.stream-window.v1",
            "network.profile.recv.v1",
            "network.profile.send.v1",
        ] {
            assert!(
                binding
                    .matched_support_surface
                    .iter()
                    .any(|candidate| candidate == surface),
                "expected matched network surface `{surface}`"
            );
        }

        for slot in [
            "bind_core",
            "endpoint_kind",
            "timeout_budget",
            "retry_budget",
            "stream_window",
            "recv_window",
            "send_window",
        ] {
            assert!(
                binding
                    .matched_support_profile_slots
                    .iter()
                    .any(|candidate| candidate == slot),
                "expected matched network slot `{slot}`"
            );
            assert!(
                binding
                    .covered_support_profile_slots
                    .iter()
                    .any(|candidate| candidate == slot),
                "expected covered network slot `{slot}`"
            );
        }
        assert!(binding.capability_tags.contains(&"async-bridge".to_owned()));
    }

    #[test]
    fn data_binding_plan_detects_profile_surfaces_and_slots() {
        let plan = binding_plan_from_source(DATA_BINDING_SOURCE);
        let binding = plan
            .bindings
            .iter()
            .find(|binding| binding.domain_family == "data")
            .expect("data binding should be present");
        for surface in ["data.profile.bind-core.v1", "data.profile.window-layout.v1"] {
            assert!(
                binding
                    .matched_support_surface
                    .iter()
                    .any(|candidate| candidate == surface),
                "expected matched data surface `{surface}`"
            );
        }
        for slot in ["bind_core", "window_offset", "uplink_len", "downlink_len"] {
            assert!(
                binding
                    .matched_support_profile_slots
                    .iter()
                    .any(|candidate| candidate == slot),
                "expected matched data slot `{slot}`"
            );
        }
    }

    #[test]
    fn kernel_binding_plan_detects_profile_surfaces_and_slots() {
        let source = r#"
use kernel KernelUnit;

mod cpu Main {
  fn capture_kernel_profile_summary() -> i64 {
    let bind_core: i64 = kernel_profile_bind_core("KernelUnit");
    let queue_depth: i64 = kernel_profile_queue_depth("KernelUnit");
    let batch_lanes: i64 = kernel_profile_batch_lanes("KernelUnit");
    return bind_core + queue_depth + batch_lanes;
  }

  fn main() {
    print(capture_kernel_profile_summary());
  }
}
"#;
        let plan = binding_plan_from_source(source);
        let binding = plan
            .bindings
            .iter()
            .find(|binding| binding.domain_family == "kernel")
            .expect("kernel binding should be present");
        for surface in [
            "kernel.profile.bind-core.v1",
            "kernel.profile.queue-depth.v1",
            "kernel.profile.batch-lanes.v1",
        ] {
            assert!(
                binding
                    .matched_support_surface
                    .iter()
                    .any(|candidate| candidate == surface),
                "expected matched kernel surface `{surface}`"
            );
        }
        for slot in ["bind_core", "queue_depth", "batch_lanes"] {
            assert!(
                binding
                    .matched_support_profile_slots
                    .iter()
                    .any(|candidate| candidate == slot),
                "expected matched kernel slot `{slot}`"
            );
        }
    }

    #[test]
    fn shader_binding_plan_detects_profile_surfaces_and_slots() {
        let source = r#"
use shader SurfaceShader;

mod cpu Main {
  fn capture_shader_profile_summary() -> i64 {
    let target: Target = shader_profile_target("SurfaceShader");
    let viewport: Viewport = shader_profile_viewport("SurfaceShader");
    let pipeline: Pipeline = shader_profile_pipeline("SurfaceShader");
    let vertex_count: i64 = shader_profile_vertex_count("SurfaceShader");
    let instance_count: i64 = shader_profile_instance_count("SurfaceShader");
    let _ = target;
    let _ = viewport;
    let _ = pipeline;
    return vertex_count + instance_count;
  }

  fn main() {
    print(capture_shader_profile_summary());
  }
}
"#;
        let plan = binding_plan_from_source(source);
        let binding = plan
            .bindings
            .iter()
            .find(|binding| binding.domain_family == "shader")
            .expect("shader binding should be present");
        for surface in [
            "shader.profile.target.v1",
            "shader.profile.viewport.v1",
            "shader.profile.pipeline.v1",
            "shader.profile.draw-budget.v1",
        ] {
            assert!(
                binding
                    .matched_support_surface
                    .iter()
                    .any(|candidate| candidate == surface),
                "expected matched shader surface `{surface}`"
            );
        }
        for slot in [
            "target",
            "viewport",
            "pipeline",
            "vertex_count",
            "instance_count",
        ] {
            assert!(
                binding
                    .matched_support_profile_slots
                    .iter()
                    .any(|candidate| candidate == slot),
                "expected matched shader slot `{slot}`"
            );
        }
    }

    #[test]
    fn shader_binding_plan_detects_nova_packet_surface_and_covered_slots() {
        let source = r#"
use shader SurfaceShader;

mod cpu Main {
  fn main() {
    let packet: NovaPanelPacket =
      shader_profile_panel_packet("SurfaceShader", 1, 2, 3, 4, 5, 6);
    let _ = packet;
    print(0);
  }
}
"#;
        let plan = binding_plan_from_source(source);
        let binding = plan
            .bindings
            .iter()
            .find(|binding| binding.domain_family == "shader")
            .expect("shader binding should be present");
        assert!(binding
            .matched_support_surface
            .iter()
            .any(|surface| surface == "shader.profile.packet.nova.v1"));
        for slot in [
            "slider_color_slot",
            "slider_speed_slot",
            "slider_radius_slot",
            "header_accent_slot",
            "toggle_live_slot",
            "focus_slot",
        ] {
            assert!(
                binding
                    .covered_support_profile_slots
                    .iter()
                    .any(|candidate| candidate == slot),
                "expected covered shader slot `{slot}`"
            );
        }
    }

    #[test]
    fn shader_binding_plan_detects_nova_profile_slot_accessors() {
        let source = r#"
use shader SurfaceShader;

mod cpu Main {
  fn capture_shader_nova_profile_summary() -> i64 {
    let slider_color: i64 = shader_profile_slider_color_slot("SurfaceShader");
    let slider_speed: i64 = shader_profile_slider_speed_slot("SurfaceShader");
    let slider_radius: i64 = shader_profile_slider_radius_slot("SurfaceShader");
    let header_accent: i64 = shader_profile_header_accent_slot("SurfaceShader");
    let toggle_live: i64 = shader_profile_toggle_live_slot("SurfaceShader");
    let focus: i64 = shader_profile_focus_slot("SurfaceShader");
    return slider_color + slider_speed + slider_radius + header_accent + toggle_live + focus;
  }

  fn main() {
    print(capture_shader_nova_profile_summary());
  }
}
"#;
        let plan = binding_plan_from_source(source);
        let binding = plan
            .bindings
            .iter()
            .find(|binding| binding.domain_family == "shader")
            .expect("shader binding should be present");
        for surface in [
            "shader.profile.packet-slots.v1",
            "shader.profile.packet.nova.v1",
        ] {
            assert!(
                binding
                    .matched_support_surface
                    .iter()
                    .any(|candidate| candidate == surface),
                "expected matched shader surface `{surface}`"
            );
        }
        for slot in [
            "slider_color_slot",
            "slider_speed_slot",
            "slider_radius_slot",
            "header_accent_slot",
            "toggle_live_slot",
            "focus_slot",
        ] {
            assert!(
                binding
                    .matched_support_profile_slots
                    .iter()
                    .any(|candidate| candidate == slot),
                "expected matched shader slot `{slot}`"
            );
        }
    }

    #[test]
    fn shader_binding_plan_detects_packet_binding_profile_contract_surface() {
        let source = r#"
use shader SurfaceShader;

mod cpu Main {
  fn main() {
    let packet: NovaPanelPacket =
      shader_profile_panel_packet("SurfaceShader", 1, 2, 3, 4, 5, 6);
    let binding: Binding = shader_packet_uniform_binding(4, packet);
    print(binding);
  }
}
"#;
        let nir = crate::frontend::parse_nuis_module(source).expect("source should lower to nir");
        let (matched_support_surface, matched_support_profile_slots) =
            detect_matched_support_usage(&nir, "shader");
        let covered_support_profile_slots = covered_profile_slots(
            "shader",
            &matched_support_surface,
            &matched_support_profile_slots,
        );
        assert!(matched_support_surface
            .iter()
            .any(|surface| surface == "shader.profile.packet.nova.v1"));
        for slot in [
            "slider_color_slot",
            "slider_speed_slot",
            "slider_radius_slot",
            "header_accent_slot",
            "toggle_live_slot",
            "focus_slot",
        ] {
            assert!(
                covered_support_profile_slots
                    .iter()
                    .any(|candidate| candidate == slot),
                "expected covered shader slot `{slot}`"
            );
        }
    }

    #[test]
    fn shader_binding_plan_collects_packet_binding_resource_hints() {
        let source = r#"
use shader SurfaceShader;

mod cpu Main {
  fn main() {
    let packet: NovaPanelPacket =
      shader_profile_panel_packet("SurfaceShader", 1, 2, 3, 4, 5, 6);
    let binding: Binding = shader_packet_uniform_binding(4, packet);
    let pipeline: Pipeline = shader_profile_pipeline("SurfaceShader");
    let bindings: BindingSet = shader_bind_set(pipeline, binding);
    print(bindings);
  }
}
"#;
        let nir = crate::frontend::parse_nuis_module(source).expect("source should lower to nir");
        let mut resources = BTreeSet::new();
        collect_resource_usage_hints(&nir, "shader", &mut resources);
        for resource in [
            "shader.binding.uniform_binding",
            "shader.binding.layout.std140",
            "shader.binding.contract.shader.profile.packet.nova.v1",
            "shader.binding.set",
        ] {
            assert!(
                resources.iter().any(|candidate| candidate == resource),
                "expected matched shader resource hint `{resource}`"
            );
        }
    }
}
