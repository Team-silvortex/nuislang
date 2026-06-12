use std::collections::BTreeMap;

use nuis_semantics::model::{
    NirAddressClass, NirBinaryOp, NirDataFlowState, NirExpr, NirKernelFlowState,
    NirNetworkFlowState, NirResultFamily, NirResultStage, NirShaderFlowState, NirStructDef,
    NirTypeRef, NirWindowMode,
};

use super::super::render_type_name;
use super::builtin_fields::builtin_struct_field_type;
use crate::frontend::FunctionSignature;

pub(crate) fn infer_result_stage(expr: &NirExpr) -> Option<NirResultStage> {
    match expr {
        NirExpr::DataBindCore(_)
        | NirExpr::DataMarker(_)
        | NirExpr::DataHandleTable(_)
        | NirExpr::DataInputPipe(_) => Some(NirDataFlowState::Ready.into()),
        NirExpr::DataOutputPipe(_) => Some(NirDataFlowState::Moved.into()),
        NirExpr::DataCopyWindow { .. }
        | NirExpr::DataWriteWindow { .. }
        | NirExpr::DataFreezeWindow(_)
        | NirExpr::DataImmutableWindow { .. }
        | NirExpr::DataProfileSendUplink { .. }
        | NirExpr::DataProfileSendDownlink { .. } => Some(NirDataFlowState::Windowed.into()),
        NirExpr::ShaderBeginPass { .. } => Some(NirShaderFlowState::PassReady.into()),
        NirExpr::ShaderDrawInstanced { .. } | NirExpr::ShaderProfileRender { .. } => {
            Some(NirShaderFlowState::FrameReady.into())
        }
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
        | NirExpr::NetworkProfileProtocolHeaderBytesRef { .. } => {
            Some(NirNetworkFlowState::ConfigReady.into())
        }
        NirExpr::CpuExternCall { callee, .. }
            if callee == "host_network_open_tcp_stream"
                || callee == "host_network_open_udp_datagram"
                || callee == "host_network_open_tcp_listener"
                || callee == "host_network_bind_udp_datagram" =>
        {
            Some(NirNetworkFlowState::ConfigReady.into())
        }
        NirExpr::CpuExternCall { callee, .. } if callee == "host_network_send_probe" => {
            Some(NirNetworkFlowState::SendReady.into())
        }
        NirExpr::CpuExternCall { callee, .. } if callee == "host_network_send_owned" => {
            Some(NirNetworkFlowState::SendReady.into())
        }
        NirExpr::CpuExternCall { callee, .. } if callee == "host_network_accept_probe" => {
            Some(NirNetworkFlowState::AcceptReady.into())
        }
        NirExpr::CpuExternCall { callee, .. } if callee == "host_network_accept_owned" => {
            Some(NirNetworkFlowState::AcceptReady.into())
        }
        NirExpr::CpuExternCall { callee, .. } if callee == "host_network_recv_probe" => {
            Some(NirNetworkFlowState::RecvReady.into())
        }
        NirExpr::CpuExternCall { callee, .. } if callee == "host_network_recv_owned" => {
            Some(NirNetworkFlowState::RecvReady.into())
        }
        NirExpr::CpuExternCall { callee, .. }
            if callee == "host_network_recv_http_status_owned" =>
        {
            Some(NirNetworkFlowState::RecvReady.into())
        }
        NirExpr::CpuExternCall { callee, .. } if callee == "host_network_close" => {
            Some(NirNetworkFlowState::Closed.into())
        }
        NirExpr::KernelProfileBindCoreRef { .. }
        | NirExpr::KernelProfileQueueDepthRef { .. }
        | NirExpr::KernelProfileBatchLanesRef { .. }
        | NirExpr::KernelReduceSum(_)
        | NirExpr::KernelReduceMax(_)
        | NirExpr::KernelReduceMean(_)
        | NirExpr::KernelArgmax(_)
        | NirExpr::KernelArgmin(_) => Some(NirKernelFlowState::ConfigReady.into()),
        _ => None,
    }
}

pub(crate) fn ensure_result_like(
    name: &str,
    expr: &NirExpr,
    family: NirResultFamily,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<(), String> {
    match infer_nir_expr_type(expr, bindings, signatures, struct_table) {
        Some(ty) if ty.result_family() == Some(family) => Ok(()),
        Some(ty) => Err(format!(
            "{name}(...) expects `{}<...>`, found `{}`",
            family.type_name(),
            render_type_name(&ty)
        )),
        None => Err(format!(
            "{name}(...) requires a typed {} in the current frontend",
            family.type_name().to_ascii_lowercase()
        )),
    }
}

pub(crate) fn make_result_type(family: NirResultFamily, payload: NirTypeRef) -> NirTypeRef {
    generic_named_type(family.type_name(), vec![payload])
}

pub(crate) fn expr_type(
    expr: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Option<NirTypeRef> {
    infer_nir_expr_type(expr, bindings, signatures, struct_table)
}

pub(crate) fn result_payload_type(
    expr: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Option<NirTypeRef> {
    expr_type(expr, bindings, signatures, struct_table).and_then(|ty| {
        ty.result_payload()
            .cloned()
            .or_else(|| ty.container_payload().cloned())
    })
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn infer_nir_expr_address_class(
    expr: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    address_classes: &BTreeMap<String, NirAddressClass>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Option<NirAddressClass> {
    let ty = infer_nir_expr_type(expr, bindings, signatures, struct_table)?;
    if !ty.is_address_type() {
        return None;
    }

    match expr {
        NirExpr::Var(name) => address_classes.get(name).copied(),
        NirExpr::Borrow(_) => Some(NirAddressClass::Borrowed),
        NirExpr::Move(inner) => {
            infer_nir_expr_address_class(inner, bindings, address_classes, signatures, struct_table)
        }
        NirExpr::AllocNode { .. } | NirExpr::AllocBuffer { .. } => Some(NirAddressClass::Owned),
        NirExpr::LoadNext(inner) => {
            infer_nir_expr_address_class(inner, bindings, address_classes, signatures, struct_table)
        }
        _ => None,
    }
}

pub(crate) fn infer_nir_expr_type(
    expr: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Option<NirTypeRef> {
    match expr {
        NirExpr::Bool(_) | NirExpr::IsNull(_) => Some(bool_type()),
        NirExpr::Text(_) => Some(string_type()),
        NirExpr::Int(_) => Some(i64_type()),
        NirExpr::F32(_) => Some(f32_type()),
        NirExpr::F64(_) => Some(f64_type()),
        NirExpr::Var(name) => bindings.get(name).cloned(),
        NirExpr::Await(value) => infer_nir_expr_type(value, bindings, signatures, struct_table),
        NirExpr::Instantiate { unit, .. } => {
            Some(generic_named_type("Instance", vec![named_type(unit)]))
        }
        NirExpr::Null => None,
        NirExpr::Borrow(value) | NirExpr::Move(value) => {
            infer_nir_expr_type(value, bindings, signatures, struct_table)
        }
        NirExpr::BorrowEnd(_) => Some(unit_type()),
        NirExpr::HostBufferHandle(_) => Some(i64_type()),
        NirExpr::AllocNode { .. } => Some(ref_type("Node")),
        NirExpr::AllocBuffer { .. } => Some(ref_type("Buffer")),
        NirExpr::DataBindCore(_) | NirExpr::CpuBindCore(_) => Some(unit_type()),
        NirExpr::CpuWindow { .. } => Some(named_type("Window")),
        NirExpr::CpuInputI64 { .. } | NirExpr::CpuTickI64 { .. } => Some(i64_type()),
        NirExpr::CpuSpawn { callee, .. } => signatures
            .get(callee)
            .and_then(|sig| sig.return_type.clone())
            .map(|ty| generic_named_type("Task", vec![ty])),
        NirExpr::CpuJoin(task) => result_payload_type(task, bindings, signatures, struct_table),
        NirExpr::CpuCancel(task) => infer_nir_expr_type(task, bindings, signatures, struct_table),
        NirExpr::CpuJoinResult(task) => {
            result_payload_type(task, bindings, signatures, struct_table)
                .map(|ty| make_result_type(NirResultFamily::Task, ty))
        }
        NirExpr::CpuTaskCompleted(_)
        | NirExpr::CpuTaskTimedOut(_)
        | NirExpr::CpuTaskCancelled(_) => Some(bool_type()),
        NirExpr::CpuTaskValue(result) => {
            result_payload_type(result, bindings, signatures, struct_table)
        }
        NirExpr::CpuTimeout { task, .. } => {
            infer_nir_expr_type(task, bindings, signatures, struct_table)
        }
        NirExpr::CpuPresentFrame(_) => Some(unit_type()),
        NirExpr::ShaderProfileTargetRef { .. } => Some(named_type("Target")),
        NirExpr::ShaderProfileViewportRef { .. } => Some(named_type("Viewport")),
        NirExpr::ShaderProfilePipelineRef { .. } => Some(named_type("Pipeline")),
        NirExpr::ShaderProfileVertexCountRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfileInstanceCountRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePacketColorSlotRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePacketSpeedSlotRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePacketRadiusSlotRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePacketTagRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfileMaterialModeRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePassKindRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePacketFieldCountRef { .. } => Some(i64_type()),
        NirExpr::ShaderProfileColorSeed { .. } => Some(i64_type()),
        NirExpr::ShaderProfileSpeedSeed { .. } => Some(i64_type()),
        NirExpr::ShaderProfileRadiusSeed { .. } => Some(i64_type()),
        NirExpr::ShaderProfilePacket {
            unit,
            packet_type_name,
            ..
        } => {
            let packet_name = packet_type_name
                .clone()
                .unwrap_or_else(|| format!("{unit}Packet"));
            Some(named_type(&packet_name))
        }
        NirExpr::DataProfileBindCoreRef { .. } => Some(named_type("Unit")),
        NirExpr::DataProfileWindowOffsetRef { .. } => Some(i64_type()),
        NirExpr::DataProfileUplinkLenRef { .. } => Some(i64_type()),
        NirExpr::DataProfileDownlinkLenRef { .. } => Some(i64_type()),
        NirExpr::DataProfileHandleTableRef { .. } => Some(named_type("HandleTable")),
        NirExpr::DataProfileMarkerRef { .. } => Some(named_type("Marker")),
        NirExpr::NetworkProfileBindCoreRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileEndpointKindRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileTransportFamilyRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileLocalPortRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileRemotePortRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileConnectTimeoutRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileReadTimeoutRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileWriteTimeoutRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileTimeoutBudgetRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileRetryBudgetRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileStreamWindowRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileRecvWindowRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileSendWindowRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileProtocolKindRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileProtocolVersionRef { .. } => Some(i64_type()),
        NirExpr::NetworkProfileProtocolHeaderBytesRef { .. } => Some(i64_type()),
        NirExpr::NetworkResult { value, .. } => {
            expr_type(value, bindings, signatures, struct_table)
                .map(|inner| make_result_type(NirResultFamily::Network, inner))
        }
        NirExpr::NetworkConfigReady(_)
        | NirExpr::NetworkSendReady(_)
        | NirExpr::NetworkRecvReady(_)
        | NirExpr::NetworkAcceptReady(_) => Some(bool_type()),
        NirExpr::NetworkValue(result) => {
            result_payload_type(result, bindings, signatures, struct_table)
        }
        NirExpr::KernelProfileBindCoreRef { .. } => Some(i64_type()),
        NirExpr::KernelProfileQueueDepthRef { .. } => Some(i64_type()),
        NirExpr::KernelProfileBatchLanesRef { .. } => Some(i64_type()),
        NirExpr::KernelResult { value, .. } => expr_type(value, bindings, signatures, struct_table)
            .map(|inner| make_result_type(NirResultFamily::Kernel, inner)),
        NirExpr::KernelConfigReady(_) => Some(bool_type()),
        NirExpr::KernelValue(result) => {
            result_payload_type(result, bindings, signatures, struct_table)
        }
        NirExpr::KernelShape(_) => Some(named_type("TensorShape")),
        NirExpr::KernelRows(_) | NirExpr::KernelCols(_) | NirExpr::KernelElementAt { .. } => {
            Some(i64_type())
        }
        NirExpr::KernelTensor { .. }
        | NirExpr::KernelRow(_)
        | NirExpr::KernelCol(_)
        | NirExpr::KernelReshape { .. }
        | NirExpr::KernelBroadcast { .. }
        | NirExpr::KernelReduceSumAxis { .. }
        | NirExpr::KernelReduceMaxAxis { .. }
        | NirExpr::KernelReduceMeanAxis { .. }
        | NirExpr::KernelArgmaxAxis { .. }
        | NirExpr::KernelArgminAxis { .. }
        | NirExpr::KernelSort(_)
        | NirExpr::KernelSortAxis { .. }
        | NirExpr::KernelTopk { .. }
        | NirExpr::KernelTopkAxis { .. }
        | NirExpr::KernelMap { .. }
        | NirExpr::KernelMapAxis { .. }
        | NirExpr::KernelZip { .. }
        | NirExpr::KernelMatmul { .. }
        | NirExpr::KernelAddBias { .. }
        | NirExpr::KernelRelu(_) => Some(named_type("Tensor")),
        NirExpr::KernelReduceSum(_)
        | NirExpr::KernelReduceMax(_)
        | NirExpr::KernelReduceMean(_)
        | NirExpr::KernelArgmax(_)
        | NirExpr::KernelArgmin(_) => Some(i64_type()),
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. } => {
            let window_inner = infer_nir_expr_type(input, bindings, signatures, struct_table)?;
            Some(generic_named_type("Window", vec![window_inner]))
        }
        NirExpr::DataResult { value, .. } => expr_type(value, bindings, signatures, struct_table)
            .map(|inner| make_result_type(NirResultFamily::Data, inner)),
        NirExpr::DataReady(_) | NirExpr::DataMoved(_) | NirExpr::DataWindowed(_) => {
            Some(bool_type())
        }
        NirExpr::DataValue(result) => {
            result_payload_type(result, bindings, signatures, struct_table)
        }
        NirExpr::DataFreezeWindow(input) => {
            let inner = infer_nir_expr_type(input, bindings, signatures, struct_table)?;
            let payload = match inner.window_mode() {
                Some(NirWindowMode::Mutable | NirWindowMode::Immutable) => {
                    inner.container_payload()?.clone()
                }
                None => return None,
            };
            Some(generic_named_type("Window", vec![payload]))
        }
        NirExpr::CastI64ToI32(inner) => {
            let inner_ty = infer_nir_expr_type(inner, bindings, signatures, struct_table)?;
            if inner_ty == i64_type() {
                Some(i32_type())
            } else {
                None
            }
        }
        NirExpr::CpuExternCall { callee, .. } => signatures
            .get(callee)
            .and_then(|sig| sig.return_type.clone()),
        NirExpr::DataMarker(_) => Some(named_type("Marker")),
        NirExpr::DataHandleTable(_) => Some(named_type("HandleTable")),
        NirExpr::ShaderTarget { .. } => Some(named_type("Target")),
        NirExpr::ShaderViewport { .. } => Some(named_type("Viewport")),
        NirExpr::ShaderPipeline { .. } => Some(named_type("Pipeline")),
        NirExpr::ShaderInlineWgsl { .. } => Some(named_type("ShaderModule")),
        NirExpr::ShaderResult { value, .. } => expr_type(value, bindings, signatures, struct_table)
            .map(|inner| make_result_type(NirResultFamily::Shader, inner)),
        NirExpr::ShaderPassReady(_) | NirExpr::ShaderFrameReady(_) => Some(bool_type()),
        NirExpr::ShaderValue(result) => {
            result_payload_type(result, bindings, signatures, struct_table)
        }
        NirExpr::ShaderBeginPass { .. } => Some(named_type("Pass")),
        NirExpr::ShaderDrawInstanced { .. } => Some(named_type("Frame")),
        NirExpr::ShaderProfileRender { .. } => Some(named_type("Frame")),
        NirExpr::DataOutputPipe(value) => {
            let inner = infer_nir_expr_type(value, bindings, signatures, struct_table)?;
            Some(generic_named_type("Pipe", vec![inner]))
        }
        NirExpr::DataCopyWindow { input, .. } => infer_data_window_type(
            input,
            bindings,
            signatures,
            struct_table,
            NirWindowMode::Mutable,
        ),
        NirExpr::DataReadWindow { window, .. } => {
            let window_ty = infer_nir_expr_type(window, bindings, signatures, struct_table)?;
            window_ty.container_payload().cloned()
        }
        NirExpr::DataWriteWindow { window, value, .. } => {
            let window_ty = infer_nir_expr_type(window, bindings, signatures, struct_table)?;
            if window_ty.window_mode() != Some(NirWindowMode::Mutable) {
                return None;
            }
            let payload = window_ty.container_payload()?.clone();
            let value_ty = infer_nir_expr_type(value, bindings, signatures, struct_table)?;
            if compatible_types(&payload, &value_ty) {
                Some(window_ty)
            } else {
                None
            }
        }
        NirExpr::DataImmutableWindow { input, .. } => infer_data_window_type(
            input,
            bindings,
            signatures,
            struct_table,
            NirWindowMode::Immutable,
        ),
        NirExpr::DataInputPipe(value) => {
            let pipe_ty = infer_nir_expr_type(value, bindings, signatures, struct_table)?;
            pipe_ty.generic_args.first().cloned()
        }
        NirExpr::LoadValue(_) | NirExpr::LoadAt { .. } | NirExpr::BufferLen(_) => Some(i64_type()),
        NirExpr::LoadNext(_) => Some(ref_type("Node")),
        NirExpr::StoreValue { .. }
        | NirExpr::StoreNext { .. }
        | NirExpr::StoreAt { .. }
        | NirExpr::Free(_) => Some(unit_type()),
        NirExpr::Call { callee, .. } => signatures
            .get(callee)
            .and_then(|sig| sig.return_type.clone()),
        NirExpr::MethodCall { .. } => None,
        NirExpr::StructLiteral {
            type_name,
            type_args,
            ..
        } => Some(generic_named_type(type_name, type_args.clone())),
        NirExpr::FieldAccess { base, field } => {
            let base_ty = infer_nir_expr_type(base, bindings, signatures, struct_table)?;
            struct_field_type(&base_ty, field, struct_table)
        }
        NirExpr::Binary { op, lhs, rhs } => {
            let lhs_ty = infer_nir_expr_type(lhs, bindings, signatures, struct_table)?;
            let rhs_ty = infer_nir_expr_type(rhs, bindings, signatures, struct_table)?;
            match op {
                NirBinaryOp::And | NirBinaryOp::Or => {
                    if compatible_types(&lhs_ty, &rhs_ty)
                        && lhs_ty.is_bool_scalar()
                        && rhs_ty.is_bool_scalar()
                    {
                        Some(bool_type())
                    } else {
                        None
                    }
                }
                NirBinaryOp::Add | NirBinaryOp::Sub | NirBinaryOp::Mul | NirBinaryOp::Div => {
                    if compatible_types(&lhs_ty, &rhs_ty) && lhs_ty.is_numeric_scalar() {
                        Some(lhs_ty)
                    } else {
                        None
                    }
                }
                NirBinaryOp::Eq | NirBinaryOp::Ne => {
                    if compatible_types(&lhs_ty, &rhs_ty)
                        && ((lhs_ty.is_integer_scalar() && rhs_ty.is_integer_scalar())
                            || (lhs_ty.is_bool_scalar() && rhs_ty.is_bool_scalar()))
                    {
                        Some(bool_type())
                    } else {
                        None
                    }
                }
                NirBinaryOp::Lt | NirBinaryOp::Le | NirBinaryOp::Gt | NirBinaryOp::Ge => {
                    if compatible_types(&lhs_ty, &rhs_ty)
                        && lhs_ty.is_integer_scalar()
                        && rhs_ty.is_integer_scalar()
                    {
                        Some(bool_type())
                    } else {
                        None
                    }
                }
            }
        }
    }
}

pub(crate) fn infer_data_window_type(
    input: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    mode: NirWindowMode,
) -> Option<NirTypeRef> {
    let inner = infer_nir_expr_type(input, bindings, signatures, struct_table)?;
    let payload = if inner.is_ref && inner.name == "Buffer" {
        i64_type()
    } else {
        inner
    };
    Some(match mode {
        NirWindowMode::Mutable => generic_named_type("WindowMut", vec![payload]),
        NirWindowMode::Immutable => generic_named_type("Window", vec![payload]),
    })
}

pub(crate) fn resolve_declared_or_inferred(
    name: &str,
    declared: Option<NirTypeRef>,
    inferred: Option<NirTypeRef>,
) -> Result<NirTypeRef, String> {
    match (declared, inferred) {
        (Some(declared), Some(inferred)) => {
            if compatible_types(&declared, &inferred) {
                Ok(declared)
            } else {
                Err(format!(
                    "binding `{name}` expected type `{}`, found `{}`",
                    render_type_name(&declared),
                    render_type_name(&inferred)
                ))
            }
        }
        (Some(declared), None) => Ok(declared),
        (None, Some(inferred)) => Ok(inferred),
        (None, None) => Err(format!(
            "binding `{name}` requires an explicit type annotation in the current minimal frontend"
        )),
    }
}

pub(crate) fn compatible_types(expected: &NirTypeRef, actual: &NirTypeRef) -> bool {
    if expected.window_mode() == Some(NirWindowMode::Immutable)
        && actual.window_mode() == Some(NirWindowMode::Mutable)
        && expected.is_optional == actual.is_optional
        && expected.is_ref == actual.is_ref
        && expected.generic_args.len() == actual.generic_args.len()
    {
        return expected
            .generic_args
            .iter()
            .zip(&actual.generic_args)
            .all(|(lhs, rhs)| compatible_types(lhs, rhs));
    }
    if expected.name == actual.name
        && !expected.is_ref
        && !actual.is_ref
        && !expected.is_optional
        && !actual.is_optional
        && matches!(expected.name.as_str(), "Marker" | "HandleTable")
    {
        return expected.generic_args.is_empty()
            || actual.generic_args.is_empty()
            || (expected.generic_args.len() == actual.generic_args.len()
                && expected
                    .generic_args
                    .iter()
                    .zip(&actual.generic_args)
                    .all(|(lhs, rhs)| compatible_types(lhs, rhs)));
    }
    if expected.name != actual.name
        || expected.is_ref != actual.is_ref
        || expected.is_optional != actual.is_optional
        || expected.generic_args.len() != actual.generic_args.len()
    {
        return expected.is_ref && actual.is_ref && expected.generic_args.is_empty();
    }
    expected
        .generic_args
        .iter()
        .zip(&actual.generic_args)
        .all(|(lhs, rhs)| compatible_types(lhs, rhs))
}

pub(crate) fn named_type(name: &str) -> NirTypeRef {
    NirTypeRef {
        name: name.to_owned(),
        generic_args: Vec::new(),
        is_optional: false,
        is_ref: false,
    }
}

pub(crate) fn generic_named_type(name: &str, generic_args: Vec<NirTypeRef>) -> NirTypeRef {
    NirTypeRef {
        name: name.to_owned(),
        generic_args,
        is_optional: false,
        is_ref: false,
    }
}

pub(crate) fn ref_type(name: &str) -> NirTypeRef {
    NirTypeRef {
        name: name.to_owned(),
        generic_args: Vec::new(),
        is_optional: false,
        is_ref: true,
    }
}

pub(crate) fn i64_type() -> NirTypeRef {
    named_type("i64")
}
pub(crate) fn i32_type() -> NirTypeRef {
    named_type("i32")
}
pub(crate) fn f32_type() -> NirTypeRef {
    named_type("f32")
}
pub(crate) fn f64_type() -> NirTypeRef {
    named_type("f64")
}
pub(crate) fn bool_type() -> NirTypeRef {
    named_type("bool")
}
pub(crate) fn string_type() -> NirTypeRef {
    named_type("String")
}
pub(crate) fn unit_type() -> NirTypeRef {
    named_type("Unit")
}

pub(crate) fn struct_field_type(
    base_ty: &NirTypeRef,
    field: &str,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Option<NirTypeRef> {
    if let Some(builtin) = builtin_struct_field_type(&base_ty.name, field) {
        return Some(builtin);
    }
    let definition = struct_table.get(&base_ty.name)?;
    Some(instantiate_struct_field_type(
        base_ty,
        definition,
        &definition.field(field)?.ty,
    ))
}

pub(crate) fn instantiate_struct_field_type(
    base_ty: &NirTypeRef,
    definition: &NirStructDef,
    field_ty: &NirTypeRef,
) -> NirTypeRef {
    super::struct_generics::instantiate_struct_field_type(base_ty, definition, field_ty)
}
