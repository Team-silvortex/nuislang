use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use nuis_semantics::model::{
    nir_expr_effect_class, NirBinaryOp, NirExpr, NirExprEffectClass, NirFunction, NirKernelMapOp,
    NirModule, NirStmt, NirStructDef,
};
use yir_core::{
    Edge, EdgeKind, Node, Operation, Resource, ResourceKind, SemanticOp, TaskLifecycleState,
    YirModule, YirResultRole, YirResultState,
};

use crate::registry::NustarPackageManifest;

#[path = "lowering/body_lowering.rs"]
mod body_lowering;
#[path = "lowering/bootstrap.rs"]
mod bootstrap;
#[path = "lowering/bridge_helpers.rs"]
mod bridge_helpers;
#[path = "lowering/call_exprs.rs"]
mod call_exprs;
#[path = "lowering/core_exprs.rs"]
mod core_exprs;
#[path = "lowering/cpu_exprs.rs"]
mod cpu_exprs;
#[path = "lowering/data_cpu_exprs.rs"]
mod data_cpu_exprs;
#[path = "lowering/data_profile_refs.rs"]
mod data_profile_refs;
#[path = "lowering/direct_calls.rs"]
mod direct_calls;
#[path = "lowering/edge_helpers.rs"]
mod edge_helpers;
#[path = "lowering/guard_ops.rs"]
mod guard_ops;
#[path = "lowering/if_lowering.rs"]
mod if_lowering;
#[path = "lowering/kernel_exprs.rs"]
mod kernel_exprs;
#[path = "lowering/loop_carries.rs"]
mod loop_carries;
#[path = "lowering/loop_execution.rs"]
mod loop_execution;
#[path = "lowering/loop_flow_nodes.rs"]
mod loop_flow_nodes;
#[path = "lowering/loop_nodes.rs"]
mod loop_nodes;
#[path = "lowering/loop_preparation.rs"]
mod loop_preparation;
#[path = "lowering/loop_purity.rs"]
mod loop_purity;
#[path = "lowering/loop_types.rs"]
mod loop_types;
#[path = "lowering/network_exprs.rs"]
mod network_exprs;
#[path = "lowering/owned_loop_lowering.rs"]
mod owned_loop_lowering;
#[path = "lowering/owned_struct_layout.rs"]
mod owned_struct_layout;
#[path = "lowering/result_nodes.rs"]
mod result_nodes;
#[path = "lowering/scheduler_contracts.rs"]
mod scheduler_contracts;
#[path = "lowering/scoped_loop_lowering.rs"]
mod scoped_loop_lowering;
#[path = "lowering/shader_exprs.rs"]
mod shader_exprs;
#[path = "lowering/shader_packets.rs"]
mod shader_packets;
#[path = "lowering/state.rs"]
mod state;
#[path = "lowering/tail_recursion.rs"]
mod tail_recursion;

use body_lowering::{
    lower_async_call_boundary, lower_call_expr, lower_function_body, lower_unary_cpu_expr,
};
use bootstrap::dispatch_nustar_lowering;
#[cfg(test)]
use bootstrap::lower_nir_to_yir_builtin_cpu;
#[cfg(test)]
use bootstrap::lower_nir_to_yir_builtin_cpu_with_target;
use bridge_helpers::{lower_data_profile_send, lower_project_profile_ref};
use call_exprs::lower_call_family_expr;
use core_exprs::lower_core_expr;
use cpu_exprs::lower_cpu_expr;
use data_cpu_exprs::lower_data_cpu_expr;
use data_profile_refs::lower_data_profile_ref_expr;
use direct_calls::{lower_direct_call_helper_function, push_direct_call_node};
use edge_helpers::{
    ensure_fabric_resource, ensure_kernel_resource, ensure_network_resource,
    ensure_shader_resource, push_dep_edges, push_lifetime_edge, push_xfer_edge,
};
use guard_ops::{
    lower_branch_drop_owned_bytes_return, lower_branch_host_call_return, lower_branch_print_return,
    lower_guard_drop_owned_bytes_return, lower_guard_host_call_return, lower_guard_print,
    lower_guard_print_return, lower_guard_return, lower_select, PreparedHostCallReturnSpec,
};
use if_lowering::lower_if_pair;
use kernel_exprs::lower_kernel_expr;
use loop_carries::{
    encode_loop_carry_branch_source_args, encode_loop_carry_source_args, render_loop_compare,
    render_loop_cond_kind, render_loop_logic_op,
};
use loop_execution::{lower_prepared_loop_body, prepare_guarded_loop_body};
use loop_flow_nodes::{
    encode_loop_flow_control_args, lower_async_flow_while, lower_async_post_flow_while,
    lower_flow_while, lower_post_flow_while,
};
use loop_nodes::{lower_async_chained_while, lower_chained_while, lower_counted_while};
use loop_preparation::{
    diagnose_unsupported_prepared_while_carry, prepare_async_chained_while,
    prepare_async_flow_while, prepare_async_post_flow_while, prepare_chained_while,
    prepare_counted_while, prepare_flow_while, prepare_post_flow_while,
};
use loop_purity::{
    collect_inlineable_pure_helper_exprs, collect_pure_helper_blocks,
    collect_pure_helper_functions, extract_pure_branch_binding, inline_pure_helper_calls,
    is_terminal_branch_pure_expr, prepare_terminal_branch, substitute_prepared_loop_body,
};
use loop_types::*;
use network_exprs::lower_network_expr;
use owned_struct_layout::{function_owned_struct_layout, module_owned_struct_layout};
use result_nodes::{
    lower_result_observe_node, lower_result_unary_value_effect, lower_task_result_entry_node,
    lower_task_result_observer_node, push_await_node,
};
pub(crate) use scheduler_contracts::{
    assign_default_lanes, materialize_doc_contract_nodes,
    materialize_registered_scheduler_contract_nodes,
};
use shader_exprs::lower_shader_expr;
use shader_packets::lower_shader_packet_expr;
use state::{next_name, LoweringState, ResultLoweringDomain};
use tail_recursion::rewrite_self_tail_recursive_functions;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringTargetConfig {
    pub abi: String,
    pub machine_arch: String,
    pub machine_os: String,
    pub object_format: String,
    pub calling_abi: String,
    pub clang_target: String,
    pub isa_family: String,
    pub isa_features: Vec<String>,
}

impl LoweringTargetConfig {
    pub fn from_cpu_build_target(target: &crate::aot::CpuBuildTarget) -> Self {
        Self {
            abi: target.abi.clone(),
            machine_arch: target.machine_arch.clone(),
            machine_os: target.machine_os.clone(),
            object_format: target.object_format.clone(),
            calling_abi: target.calling_abi.clone(),
            clang_target: target.clang_target.clone(),
            isa_family: target.isa_family.clone(),
            isa_features: target.isa_features.clone(),
        }
    }

    pub fn cpu_vector_bits(&self) -> i64 {
        match self.machine_arch.as_str() {
            "arm64" | "aarch64" | "x86_64" => 128,
            _ => 0,
        }
    }

    pub fn supports_host_ffi_abi(&self, abi: &str) -> bool {
        match abi {
            "c" | "libc" => true,
            "nurs" => self.abi.contains(".nurs."),
            _ => false,
        }
    }
}

pub fn lower_nir_to_yir(
    module: &NirModule,
    nustar_manifest: &NustarPackageManifest,
    target_config: Option<&LoweringTargetConfig>,
) -> Result<YirModule, String> {
    dispatch_nustar_lowering(module, nustar_manifest, target_config)
}

fn lower_expr(
    expr: &NirExpr,
    state: &mut LoweringState<'_>,
    bindings: &BTreeMap<String, String>,
) -> Result<String, String> {
    if let Some(lowered) = lower_data_cpu_expr(expr, state, bindings) {
        return lowered;
    }
    if let Some(lowered) = lower_network_expr(expr, state, bindings) {
        return lowered;
    }
    if let Some(lowered) = lower_kernel_expr(expr, state, bindings) {
        return lowered;
    }
    if let Some(lowered) = lower_shader_expr(expr, state, bindings) {
        return lowered;
    }
    if let Some(lowered) = lower_shader_packet_expr(expr, state, bindings) {
        return lowered;
    }
    if let Some(lowered) = lower_call_family_expr(expr, state, bindings) {
        return lowered;
    }
    if let Some(lowered) = lower_data_profile_ref_expr(expr, state) {
        return lowered;
    }
    if let Some(lowered) = lower_core_expr(expr, state, bindings) {
        return lowered;
    }

    match expr {
        NirExpr::DataProfileSendUplink { .. }
        | NirExpr::DataProfileSendDownlink { .. }
        | NirExpr::DataBindCore(_)
        | NirExpr::DataMarker(_)
        | NirExpr::DataOutputPipe(_)
        | NirExpr::DataInputPipe(_)
        | NirExpr::DataResult { .. }
        | NirExpr::DataReady(_)
        | NirExpr::DataMoved(_)
        | NirExpr::DataWindowed(_)
        | NirExpr::DataValue(_)
        | NirExpr::DataCopyWindow { .. }
        | NirExpr::DataReadWindow { .. }
        | NirExpr::DataWriteWindow { .. }
        | NirExpr::DataFreezeWindow(_)
        | NirExpr::DataImmutableWindow { .. }
        | NirExpr::DataHandleTable(_) => lower_data_cpu_expr(expr, state, bindings)
            .expect("data expr family must be handled by lower_data_cpu_expr"),
        NirExpr::Instantiate { .. }
        | NirExpr::CpuBindCore(_)
        | NirExpr::CpuWindow { .. }
        | NirExpr::CpuInputI64 { .. }
        | NirExpr::CpuTickI64 { .. }
        | NirExpr::CpuSpawn { .. }
        | NirExpr::CpuThreadSpawn { .. }
        | NirExpr::CpuJoin(_)
        | NirExpr::CpuThreadJoin(_)
        | NirExpr::CpuCancel(_)
        | NirExpr::CpuJoinResult(_)
        | NirExpr::CpuThreadJoinResult(_)
        | NirExpr::CpuTaskCompleted(_)
        | NirExpr::CpuTaskTimedOut(_)
        | NirExpr::CpuTaskCancelled(_)
        | NirExpr::CpuTaskFailed(_)
        | NirExpr::CpuTaskValue(_)
        | NirExpr::CpuMutexNew(_)
        | NirExpr::CpuMutexLock(_)
        | NirExpr::CpuMutexUnlock(_)
        | NirExpr::CpuMutexValue(_)
        | NirExpr::CpuTimeout { .. }
        | NirExpr::CpuReadyAfter { .. }
        | NirExpr::CpuPresentFrame(_)
        | NirExpr::CpuExternCall { .. }
        | NirExpr::CpuExternCallI32 { .. } => lower_cpu_expr(expr, state, bindings)
            .expect("cpu expr family must be handled by lower_cpu_expr"),
        NirExpr::NetworkProfileBindCoreRef { .. }
        | NirExpr::NetworkProfileEndpointKindRef { .. }
        | NirExpr::NetworkProfileTransportFamilyRef { .. }
        | NirExpr::NetworkProfileLocalPortRef { .. }
        | NirExpr::NetworkProfileRemotePortRef { .. }
        | NirExpr::NetworkProfileConnectTimeoutRef { .. }
        | NirExpr::NetworkProfileReadTimeoutRef { .. }
        | NirExpr::NetworkProfileWriteTimeoutRef { .. }
        | NirExpr::NetworkProfileTimeoutBudgetRef { .. }
        | NirExpr::NetworkProfileRetryBudgetRef { .. }
        | NirExpr::NetworkProfileStreamWindowRef { .. }
        | NirExpr::NetworkProfileRecvWindowRef { .. }
        | NirExpr::NetworkProfileSendWindowRef { .. }
        | NirExpr::NetworkProfileProtocolKindRef { .. }
        | NirExpr::NetworkProfileProtocolVersionRef { .. }
        | NirExpr::NetworkProfileProtocolHeaderBytesRef { .. }
        | NirExpr::NetworkResult { .. }
        | NirExpr::NetworkConfigReady(_)
        | NirExpr::NetworkSendReady(_)
        | NirExpr::NetworkRecvReady(_)
        | NirExpr::NetworkAcceptReady(_)
        | NirExpr::NetworkValue(_) => lower_network_expr(expr, state, bindings)
            .expect("network expr family must be handled by lower_network_expr"),
        NirExpr::KernelProfileBindCoreRef { .. }
        | NirExpr::KernelProfileQueueDepthRef { .. }
        | NirExpr::KernelProfileBatchLanesRef { .. }
        | NirExpr::KernelResult { .. }
        | NirExpr::KernelConfigReady(_)
        | NirExpr::KernelValue(_)
        | NirExpr::KernelTensor { .. }
        | NirExpr::KernelShape(_)
        | NirExpr::KernelRows(_)
        | NirExpr::KernelCols(_)
        | NirExpr::KernelRow(_)
        | NirExpr::KernelCol(_)
        | NirExpr::KernelElementAt { .. }
        | NirExpr::KernelReshape { .. }
        | NirExpr::KernelBroadcast { .. }
        | NirExpr::KernelMap { .. }
        | NirExpr::KernelMapAxis { .. }
        | NirExpr::KernelZip { .. }
        | NirExpr::KernelMatmul { .. }
        | NirExpr::KernelAddBias { .. }
        | NirExpr::KernelRelu(_)
        | NirExpr::KernelReduceSum(_)
        | NirExpr::KernelReduceSumAxis { .. }
        | NirExpr::KernelReduceMax(_)
        | NirExpr::KernelReduceMaxAxis { .. }
        | NirExpr::KernelReduceMean(_)
        | NirExpr::KernelReduceMeanAxis { .. }
        | NirExpr::KernelArgmax(_)
        | NirExpr::KernelArgmaxAxis { .. }
        | NirExpr::KernelArgmin(_)
        | NirExpr::KernelArgminAxis { .. }
        | NirExpr::KernelSort(_)
        | NirExpr::KernelSortAxis { .. }
        | NirExpr::KernelTopk { .. }
        | NirExpr::KernelTopkAxis { .. } => lower_kernel_expr(expr, state, bindings)
            .expect("kernel expr family must be handled by lower_kernel_expr"),
        NirExpr::ShaderProfileColorSeed { .. }
        | NirExpr::ShaderProfileSpeedSeed { .. }
        | NirExpr::ShaderProfileRadiusSeed { .. }
        | NirExpr::ShaderProfileRender { .. }
        | NirExpr::ShaderProfileTargetRef { .. }
        | NirExpr::ShaderProfileViewportRef { .. }
        | NirExpr::ShaderProfilePipelineRef { .. }
        | NirExpr::ShaderProfileVertexCountRef { .. }
        | NirExpr::ShaderProfileInstanceCountRef { .. }
        | NirExpr::ShaderProfilePacketColorSlotRef { .. }
        | NirExpr::ShaderProfilePacketSpeedSlotRef { .. }
        | NirExpr::ShaderProfilePacketRadiusSlotRef { .. }
        | NirExpr::ShaderProfileSliderColorSlotRef { .. }
        | NirExpr::ShaderProfileSliderSpeedSlotRef { .. }
        | NirExpr::ShaderProfileSliderRadiusSlotRef { .. }
        | NirExpr::ShaderProfileHeaderAccentSlotRef { .. }
        | NirExpr::ShaderProfileToggleLiveSlotRef { .. }
        | NirExpr::ShaderProfileFocusSlotRef { .. }
        | NirExpr::ShaderProfilePacketTagRef { .. }
        | NirExpr::ShaderProfileMaterialModeRef { .. }
        | NirExpr::ShaderProfilePassKindRef { .. }
        | NirExpr::ShaderProfilePacketFieldCountRef { .. }
        | NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. }
        | NirExpr::ShaderTexture2d { .. }
        | NirExpr::ShaderSampler { .. }
        | NirExpr::ShaderUv { .. }
        | NirExpr::ShaderSample { .. }
        | NirExpr::ShaderSampleUv { .. }
        | NirExpr::ShaderBinding { .. }
        | NirExpr::ShaderBindSet { .. }
        | NirExpr::ShaderInlineWgsl { .. }
        | NirExpr::ShaderResult { .. }
        | NirExpr::ShaderPassReady(_)
        | NirExpr::ShaderFrameReady(_)
        | NirExpr::ShaderValue(_)
        | NirExpr::ShaderBeginPass { .. }
        | NirExpr::ShaderDrawInstanced { .. } => lower_shader_expr(expr, state, bindings)
            .expect("shader expr family must be handled by lower_shader_expr"),
        NirExpr::Await(_) => lower_call_family_expr(expr, state, bindings)
            .expect("call expr family must be handled by lower_call_family_expr"),
        NirExpr::ShaderProfilePacket { .. } => lower_shader_packet_expr(expr, state, bindings)
            .expect("shader packet family must be handled by lower_shader_packet_expr"),
        NirExpr::DataProfileBindCoreRef { .. }
        | NirExpr::DataProfileWindowOffsetRef { .. }
        | NirExpr::DataProfileUplinkLenRef { .. }
        | NirExpr::DataProfileDownlinkLenRef { .. }
        | NirExpr::DataProfileHandleTableRef { .. }
        | NirExpr::DataProfileMarkerRef { .. } => lower_data_profile_ref_expr(expr, state)
            .expect("data profile ref family must be handled by lower_data_profile_ref_expr"),
        NirExpr::Call { .. } | NirExpr::MethodCall { .. } => {
            lower_call_family_expr(expr, state, bindings)
                .expect("call expr family must be handled by lower_call_family_expr")
        }
        NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::CastI64ToI32(_)
        | NirExpr::CastI32ToI64(_)
        | NirExpr::CastI64ToBool(_)
        | NirExpr::CastBoolToI64(_)
        | NirExpr::CastI64ToF32(_)
        | NirExpr::CastF32ToI64(_)
        | NirExpr::CastI64ToF64(_)
        | NirExpr::CastF64ToI64(_)
        | NirExpr::HostBufferHandle(_)
        | NirExpr::Var(_)
        | NirExpr::Null
        | NirExpr::Borrow(_)
        | NirExpr::BorrowEnd(_)
        | NirExpr::Move(_)
        | NirExpr::AllocNode { .. }
        | NirExpr::AllocBuffer { .. }
        | NirExpr::LoadValue(_)
        | NirExpr::LoadNext(_)
        | NirExpr::BufferLen(_)
        | NirExpr::CopyBufferOwned(_)
        | NirExpr::BytesLen(_)
        | NirExpr::DropBytes(_)
        | NirExpr::IsNull(_)
        | NirExpr::LoadAt { .. }
        | NirExpr::StoreValue { .. }
        | NirExpr::StoreNext { .. }
        | NirExpr::StoreAt { .. }
        | NirExpr::Free(_)
        | NirExpr::Binary { .. }
        | NirExpr::StructLiteral { .. }
        | NirExpr::FieldAccess { .. }
        | NirExpr::VariantIs { .. }
        | NirExpr::VariantFieldAccess { .. } => lower_core_expr(expr, state, bindings)
            .expect("core expr family must be handled by lower_core_expr"),
    }
}

#[cfg(test)]
#[path = "lowering/tests_async_memory.rs"]
mod tests_async_memory;
#[cfg(test)]
#[path = "lowering/tests_async_network_runtime.rs"]
mod tests_async_network_runtime;
#[cfg(test)]
#[path = "lowering/tests_async_runtime.rs"]
mod tests_async_runtime;
#[cfg(test)]
#[path = "lowering/tests_branch_helpers.rs"]
mod tests_branch_helpers;
#[cfg(test)]
#[path = "lowering/tests_branch_host_calls.rs"]
mod tests_branch_host_calls;
#[cfg(test)]
#[path = "lowering/tests_branch_summaries.rs"]
mod tests_branch_summaries;
#[cfg(test)]
#[path = "lowering/tests_direct_calls.rs"]
mod tests_direct_calls;
#[cfg(test)]
#[path = "lowering/tests_doc_contracts.rs"]
mod tests_doc_contracts;
#[cfg(test)]
#[path = "lowering/tests_guard_return_survivor.rs"]
mod tests_guard_return_survivor;
#[cfg(test)]
#[path = "lowering/tests_higher_order_direct_calls.rs"]
mod tests_higher_order_direct_calls;
#[cfg(test)]
#[path = "lowering/tests_higher_order_direct_calls_fn23.rs"]
mod tests_higher_order_direct_calls_fn23;
#[cfg(test)]
#[path = "lowering/tests_kernel_tensor.rs"]
mod tests_kernel_tensor;
#[cfg(test)]
#[path = "lowering/tests_lane_policy.rs"]
mod tests_lane_policy;
#[cfg(test)]
#[path = "lowering/tests_loop_flow.rs"]
mod tests_loop_flow;
#[cfg(test)]
#[path = "lowering/tests_loop_post_flow.rs"]
mod tests_loop_post_flow;
#[cfg(test)]
#[path = "lowering/tests_loops_basic.rs"]
mod tests_loops_basic;
#[cfg(test)]
#[path = "lowering/tests_loops_owned.rs"]
mod tests_loops_owned;
#[cfg(test)]
#[path = "lowering/tests_recursion.rs"]
mod tests_recursion;
#[cfg(test)]
#[path = "lowering/tests_recursive_composed_calls.rs"]
mod tests_recursive_composed_calls;
#[cfg(test)]
#[path = "lowering/tests_target_config.rs"]
mod tests_target_config;
