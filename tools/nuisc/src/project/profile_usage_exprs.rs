use nuis_semantics::model::NirExpr;

use super::profile_usage_walk::expr_walk_any;

pub(super) fn expr_uses_shader_profile_render(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::ShaderProfileRender {
            unit: shader_unit,
            packet,
        } => shader_unit == unit || expr_uses_shader_profile_render(packet, unit),
        NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::Move(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::DataReady(inner)
        | NirExpr::DataMoved(inner)
        | NirExpr::DataWindowed(inner)
        | NirExpr::DataValue(inner)
        | NirExpr::DataFreezeWindow(inner)
        | NirExpr::ShaderPassReady(inner)
        | NirExpr::ShaderFrameReady(inner)
        | NirExpr::ShaderValue(inner)
        | NirExpr::KernelConfigReady(inner)
        | NirExpr::KernelValue(inner)
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => expr_uses_shader_profile_render(inner, unit),
        NirExpr::DataResult { value: input, .. } | NirExpr::ShaderResult { value: input, .. } => {
            expr_uses_shader_profile_render(input, unit)
        }
        NirExpr::KernelResult { value: input, .. } => expr_uses_shader_profile_render(input, unit),
        NirExpr::AllocNode { value, next } => {
            expr_uses_shader_profile_render(value, unit)
                || expr_uses_shader_profile_render(next, unit)
        }
        NirExpr::AllocBuffer { len, fill } => {
            expr_uses_shader_profile_render(len, unit)
                || expr_uses_shader_profile_render(fill, unit)
        }
        NirExpr::LoadAt { buffer, index } => {
            expr_uses_shader_profile_render(buffer, unit)
                || expr_uses_shader_profile_render(index, unit)
        }
        NirExpr::DataReadWindow { window, index } => {
            expr_uses_shader_profile_render(window, unit)
                || expr_uses_shader_profile_render(index, unit)
        }
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => {
            expr_uses_shader_profile_render(window, unit)
                || expr_uses_shader_profile_render(index, unit)
                || expr_uses_shader_profile_render(value, unit)
        }
        NirExpr::StoreValue { target, value } => {
            expr_uses_shader_profile_render(target, unit)
                || expr_uses_shader_profile_render(value, unit)
        }
        NirExpr::StoreNext { target, next } => {
            expr_uses_shader_profile_render(target, unit)
                || expr_uses_shader_profile_render(next, unit)
        }
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => {
            expr_uses_shader_profile_render(buffer, unit)
                || expr_uses_shader_profile_render(index, unit)
                || expr_uses_shader_profile_render(value, unit)
        }
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            expr_uses_shader_profile_render(input, unit)
                || expr_uses_shader_profile_render(offset, unit)
                || expr_uses_shader_profile_render(len, unit)
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. }
        | NirExpr::ShaderProfileColorSeed { base: input, .. }
        | NirExpr::ShaderProfileRadiusSeed { base: input, .. } => {
            expr_uses_shader_profile_render(input, unit)
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => {
            expr_uses_shader_profile_render(delta, unit)
                || expr_uses_shader_profile_render(scale, unit)
                || expr_uses_shader_profile_render(base, unit)
        }
        NirExpr::CpuExternCall { args, .. } | NirExpr::Call { args, .. } => args
            .iter()
            .any(|arg| expr_uses_shader_profile_render(arg, unit)),
        NirExpr::MethodCall { receiver, args, .. } => {
            expr_uses_shader_profile_render(receiver, unit)
                || args
                    .iter()
                    .any(|arg| expr_uses_shader_profile_render(arg, unit))
        }
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_uses_shader_profile_render(value, unit)),
        NirExpr::FieldAccess { base, .. } => expr_uses_shader_profile_render(base, unit),
        NirExpr::Binary { lhs, rhs, .. } => {
            expr_uses_shader_profile_render(lhs, unit) || expr_uses_shader_profile_render(rhs, unit)
        }
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => {
            expr_uses_shader_profile_render(target, unit)
                || expr_uses_shader_profile_render(pipeline, unit)
                || expr_uses_shader_profile_render(viewport, unit)
        }
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => {
            expr_uses_shader_profile_render(pass, unit)
                || expr_uses_shader_profile_render(packet, unit)
                || expr_uses_shader_profile_render(vertex_count, unit)
                || expr_uses_shader_profile_render(instance_count, unit)
        }
        _ => false,
    }
}

pub(super) fn expr_uses_shader_profile_draw_instanced(expr: &NirExpr, _unit: &str) -> bool {
    fn expr_uses_draw_instanced(expr: &NirExpr) -> bool {
        match expr {
            NirExpr::ShaderDrawInstanced { .. } => true,
            _ => expr_walk_any(expr, &expr_uses_draw_instanced),
        }
    }
    expr_uses_draw_instanced(expr)
}

pub(super) fn expr_uses_shader_profile_packet(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::ShaderProfilePacket {
            unit: shader_unit,
            packet_type_name,
            ..
        } => shader_unit == unit || packet_type_name.as_deref() == Some("NovaPanelPacket"),
        NirExpr::StructLiteral { type_name, .. } => type_name == "NovaPanelPacket",
        _ => expr_walk_any(expr, &|inner| expr_uses_shader_profile_packet(inner, unit)),
    }
}

pub(super) fn expr_uses_shader_profile_color_seed(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::ShaderProfileColorSeed {
            unit: shader_unit, ..
        } => shader_unit == unit,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_shader_profile_color_seed(inner, unit)
        }),
    }
}

pub(super) fn expr_uses_shader_profile_speed_seed(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::ShaderProfileSpeedSeed {
            unit: shader_unit, ..
        } => shader_unit == unit,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_shader_profile_speed_seed(inner, unit)
        }),
    }
}

pub(super) fn expr_uses_shader_profile_radius_seed(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::ShaderProfileRadiusSeed {
            unit: shader_unit, ..
        } => shader_unit == unit,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_shader_profile_radius_seed(inner, unit)
        }),
    }
}

pub(super) fn expr_uses_shader_binding_profile_contract(
    expr: &NirExpr,
    profile_contract: &str,
) -> bool {
    match expr {
        NirExpr::ShaderBinding {
            profile_contract: Some(value_profile_contract),
            ..
        } => value_profile_contract == profile_contract,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_shader_binding_profile_contract(inner, profile_contract)
        }),
    }
}

pub(super) fn expr_uses_data_profile_handle_table(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::DataProfileHandleTableRef { unit: data_unit } => data_unit == unit,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_data_profile_handle_table(inner, unit)
        }),
    }
}

pub(super) fn expr_uses_data_profile_send_uplink(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::DataProfileSendUplink {
            unit: data_unit, ..
        } => data_unit == unit,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_data_profile_send_uplink(inner, unit)
        }),
    }
}

pub(super) fn expr_uses_data_profile_send_downlink(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::DataProfileSendDownlink {
            unit: data_unit, ..
        } => data_unit == unit,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_data_profile_send_downlink(inner, unit)
        }),
    }
}

pub(super) fn expr_uses_network_profile_bind_core(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::NetworkProfileBindCoreRef { unit: network_unit } => network_unit == unit,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_network_profile_bind_core(inner, unit)
        }),
    }
}

pub(super) fn expr_uses_network_profile_endpoint_kind(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::NetworkProfileEndpointKindRef { unit: network_unit } => network_unit == unit,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_network_profile_endpoint_kind(inner, unit)
        }),
    }
}

pub(super) fn expr_uses_network_profile_slot(expr: &NirExpr, unit: &str, slot: &str) -> bool {
    match expr {
        NirExpr::NetworkProfileBindCoreRef { unit: network_unit }
            if slot == "bind_core" && network_unit == unit =>
        {
            true
        }
        NirExpr::NetworkProfileEndpointKindRef { unit: network_unit }
            if slot == "endpoint_kind" && network_unit == unit =>
        {
            true
        }
        NirExpr::NetworkProfileTransportFamilyRef { unit: network_unit }
            if slot == "transport_family" && network_unit == unit =>
        {
            true
        }
        NirExpr::NetworkProfileLocalPortRef { unit: network_unit }
            if slot == "local_port" && network_unit == unit =>
        {
            true
        }
        NirExpr::NetworkProfileRemotePortRef { unit: network_unit }
            if slot == "remote_port" && network_unit == unit =>
        {
            true
        }
        NirExpr::NetworkProfileConnectTimeoutRef { unit: network_unit }
            if slot == "connect_timeout_ms" && network_unit == unit =>
        {
            true
        }
        NirExpr::NetworkProfileReadTimeoutRef { unit: network_unit }
            if slot == "read_timeout_ms" && network_unit == unit =>
        {
            true
        }
        NirExpr::NetworkProfileWriteTimeoutRef { unit: network_unit }
            if slot == "write_timeout_ms" && network_unit == unit =>
        {
            true
        }
        NirExpr::NetworkProfileRetryBudgetRef { unit: network_unit }
            if slot == "retry_budget" && network_unit == unit =>
        {
            true
        }
        NirExpr::NetworkProfileStreamWindowRef { unit: network_unit }
            if slot == "stream_window" && network_unit == unit =>
        {
            true
        }
        NirExpr::NetworkProfileRecvWindowRef { unit: network_unit }
            if slot == "recv_window" && network_unit == unit =>
        {
            true
        }
        NirExpr::NetworkProfileSendWindowRef { unit: network_unit }
            if slot == "send_window" && network_unit == unit =>
        {
            true
        }
        NirExpr::NetworkProfileProtocolKindRef { unit: network_unit }
            if slot == "protocol_kind" && network_unit == unit =>
        {
            true
        }
        NirExpr::NetworkProfileProtocolVersionRef { unit: network_unit }
            if slot == "protocol_version" && network_unit == unit =>
        {
            true
        }
        NirExpr::NetworkProfileProtocolHeaderBytesRef { unit: network_unit }
            if slot == "protocol_header_bytes" && network_unit == unit =>
        {
            true
        }
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_network_profile_slot(inner, unit, slot)
        }),
    }
}

pub(super) fn expr_uses_cpu_extern_call(expr: &NirExpr, callee: &str) -> bool {
    match expr {
        NirExpr::CpuExternCall {
            callee: extern_callee,
            ..
        } => extern_callee == callee,
        _ => expr_walk_any(expr, &|inner| expr_uses_cpu_extern_call(inner, callee)),
    }
}
