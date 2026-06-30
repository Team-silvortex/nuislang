use super::*;

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
