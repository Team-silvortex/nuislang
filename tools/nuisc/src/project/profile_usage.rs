use nuis_semantics::model::{NirExpr, NirModule, NirStmt};

pub(crate) fn nir_uses_shader_profile_render(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_shader_profile_render(stmt, unit))
    })
}

pub(crate) fn nir_uses_shader_profile_draw_instanced(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_shader_profile_draw_instanced(stmt, unit))
    })
}

pub(crate) fn nir_uses_shader_profile_packet(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_shader_profile_packet(stmt, unit))
    })
}

pub(crate) fn nir_uses_shader_profile_color_seed(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_shader_profile_color_seed(stmt, unit))
    })
}

pub(crate) fn nir_uses_shader_profile_speed_seed(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_shader_profile_speed_seed(stmt, unit))
    })
}

pub(crate) fn nir_uses_shader_profile_radius_seed(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_shader_profile_radius_seed(stmt, unit))
    })
}

pub(crate) fn nir_uses_data_profile_handle_table(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_data_profile_handle_table(stmt, unit))
    })
}

pub(crate) fn nir_uses_data_profile_send_uplink(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_data_profile_send_uplink(stmt, unit))
    })
}

pub(crate) fn nir_uses_data_profile_send_downlink(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_data_profile_send_downlink(stmt, unit))
    })
}

pub(crate) fn nir_uses_network_profile_bind_core(module: &NirModule, unit: &str) -> bool {
    module.functions.iter().any(|function| {
        function
            .body
            .iter()
            .any(|stmt| stmt_uses_network_profile_bind_core(stmt, unit))
    })
}

fn stmt_uses_shader_profile_render(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| expr_uses_shader_profile_render(value, unit))
}

fn stmt_uses_shader_profile_draw_instanced(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_shader_profile_draw_instanced(value, unit)
    })
}

fn stmt_uses_shader_profile_packet(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| expr_uses_shader_profile_packet(value, unit))
}

fn stmt_uses_shader_profile_color_seed(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_shader_profile_color_seed(value, unit)
    })
}

fn stmt_uses_shader_profile_speed_seed(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_shader_profile_speed_seed(value, unit)
    })
}

fn stmt_uses_shader_profile_radius_seed(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_shader_profile_radius_seed(value, unit)
    })
}

fn stmt_uses_data_profile_handle_table(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_data_profile_handle_table(value, unit)
    })
}

fn stmt_uses_data_profile_send_uplink(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_data_profile_send_uplink(value, unit)
    })
}

fn stmt_uses_data_profile_send_downlink(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_data_profile_send_downlink(value, unit)
    })
}

fn stmt_uses_network_profile_bind_core(stmt: &NirStmt, unit: &str) -> bool {
    stmt_uses_expr_predicate(stmt, &|value| {
        expr_uses_network_profile_bind_core(value, unit)
    })
}

pub(super) fn stmt_uses_expr_predicate<F>(stmt: &NirStmt, predicate: &F) -> bool
where
    F: Fn(&NirExpr) -> bool,
{
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Await(value)
        | NirStmt::Expr(value) => predicate(value),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            predicate(condition)
                || then_body
                    .iter()
                    .any(|stmt| stmt_uses_expr_predicate(stmt, predicate))
                || else_body
                    .iter()
                    .any(|stmt| stmt_uses_expr_predicate(stmt, predicate))
        }
        NirStmt::While { condition, body } => {
            predicate(condition)
                || body
                    .iter()
                    .any(|stmt| stmt_uses_expr_predicate(stmt, predicate))
        }
        NirStmt::Break | NirStmt::Continue => false,
        NirStmt::Return(value) => value.as_ref().is_some_and(predicate),
    }
}

fn expr_uses_shader_profile_render(expr: &NirExpr, unit: &str) -> bool {
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

fn expr_uses_shader_profile_draw_instanced(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::ShaderDrawInstanced { .. } => true,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_shader_profile_draw_instanced(inner, unit)
        }),
    }
}

fn expr_uses_shader_profile_packet(expr: &NirExpr, unit: &str) -> bool {
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

fn expr_uses_shader_profile_color_seed(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::ShaderProfileColorSeed {
            unit: shader_unit, ..
        } => shader_unit == unit,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_shader_profile_color_seed(inner, unit)
        }),
    }
}

fn expr_uses_shader_profile_speed_seed(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::ShaderProfileSpeedSeed {
            unit: shader_unit, ..
        } => shader_unit == unit,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_shader_profile_speed_seed(inner, unit)
        }),
    }
}

fn expr_uses_shader_profile_radius_seed(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::ShaderProfileRadiusSeed {
            unit: shader_unit, ..
        } => shader_unit == unit,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_shader_profile_radius_seed(inner, unit)
        }),
    }
}

fn expr_uses_data_profile_handle_table(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::DataProfileHandleTableRef { unit: data_unit } => data_unit == unit,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_data_profile_handle_table(inner, unit)
        }),
    }
}

fn expr_uses_data_profile_send_uplink(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::DataProfileSendUplink {
            unit: data_unit, ..
        } => data_unit == unit,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_data_profile_send_uplink(inner, unit)
        }),
    }
}

fn expr_uses_data_profile_send_downlink(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::DataProfileSendDownlink {
            unit: data_unit, ..
        } => data_unit == unit,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_data_profile_send_downlink(inner, unit)
        }),
    }
}

fn expr_uses_network_profile_bind_core(expr: &NirExpr, unit: &str) -> bool {
    match expr {
        NirExpr::NetworkProfileBindCoreRef { unit: network_unit } => network_unit == unit,
        _ => expr_walk_any(expr, &|inner| {
            expr_uses_network_profile_bind_core(inner, unit)
        }),
    }
}

pub(super) fn expr_walk_any(expr: &NirExpr, predicate: &dyn Fn(&NirExpr) -> bool) -> bool {
    match expr {
        NirExpr::Await(inner)
        | NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::Move(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::CpuJoin(inner)
        | NirExpr::CpuCancel(inner)
        | NirExpr::CpuJoinResult(inner)
        | NirExpr::CpuTaskCompleted(inner)
        | NirExpr::CpuTaskTimedOut(inner)
        | NirExpr::CpuTaskCancelled(inner)
        | NirExpr::CpuTaskValue(inner)
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
        | NirExpr::IsNull(inner)
        | NirExpr::FieldAccess { base: inner, .. } => predicate(inner),
        NirExpr::DataResult { value: inner, .. }
        | NirExpr::ShaderResult { value: inner, .. }
        | NirExpr::NetworkResult { value: inner, .. } => {
            predicate(inner)
        }
        NirExpr::KernelResult { value: inner, .. } => predicate(inner),
        NirExpr::AllocNode { value, next } => predicate(value) || predicate(next),
        NirExpr::AllocBuffer { len, fill } => predicate(len) || predicate(fill),
        NirExpr::LoadAt { buffer, index } => predicate(buffer) || predicate(index),
        NirExpr::DataReadWindow { window, index } => predicate(window) || predicate(index),
        NirExpr::DataWriteWindow {
            window,
            index,
            value,
        } => predicate(window) || predicate(index) || predicate(value),
        NirExpr::StoreValue { target, value } => predicate(target) || predicate(value),
        NirExpr::StoreNext { target, next } => predicate(target) || predicate(next),
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        } => predicate(buffer) || predicate(index) || predicate(value),
        NirExpr::DataCopyWindow { input, offset, len }
        | NirExpr::DataImmutableWindow { input, offset, len } => {
            predicate(input) || predicate(offset) || predicate(len)
        }
        NirExpr::DataProfileSendUplink { input, .. }
        | NirExpr::DataProfileSendDownlink { input, .. } => predicate(input),
        NirExpr::ShaderProfileColorSeed { base, delta, .. }
        | NirExpr::ShaderProfileRadiusSeed { base, delta, .. } => {
            predicate(base) || predicate(delta)
        }
        NirExpr::ShaderProfileSpeedSeed {
            delta, scale, base, ..
        } => predicate(delta) || predicate(scale) || predicate(base),
        NirExpr::ShaderProfilePacket {
            color,
            speed,
            radius,
            ..
        } => predicate(color) || predicate(speed) || predicate(radius),
        NirExpr::CpuSpawn { args, .. }
        | NirExpr::CpuExternCall { args, .. }
        | NirExpr::Call { args, .. } => args.iter().any(predicate),
        NirExpr::CpuTimeout { task, limit } => predicate(task) || predicate(limit),
        NirExpr::MethodCall { receiver, args, .. } => {
            predicate(receiver) || args.iter().any(predicate)
        }
        NirExpr::StructLiteral { fields, .. } => fields.iter().any(|(_, value)| predicate(value)),
        NirExpr::Binary { lhs, rhs, .. } => predicate(lhs) || predicate(rhs),
        NirExpr::ShaderBeginPass {
            target,
            pipeline,
            viewport,
        } => predicate(target) || predicate(pipeline) || predicate(viewport),
        NirExpr::ShaderDrawInstanced {
            pass,
            packet,
            vertex_count,
            instance_count,
        } => {
            predicate(pass)
                || predicate(packet)
                || predicate(vertex_count)
                || predicate(instance_count)
        }
        NirExpr::ShaderProfileRender { packet, .. } => predicate(packet),
        _ => false,
    }
}
