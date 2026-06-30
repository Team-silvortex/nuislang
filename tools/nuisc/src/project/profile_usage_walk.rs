use nuis_semantics::model::{NirExpr, NirStmt};

pub(in crate::project) fn stmt_uses_expr_predicate<F>(stmt: &NirStmt, predicate: &F) -> bool
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

pub(in crate::project) fn expr_walk_any(
    expr: &NirExpr,
    predicate: &dyn Fn(&NirExpr) -> bool,
) -> bool {
    match expr {
        NirExpr::Await(inner)
        | NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
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
        | NirExpr::NetworkResult { value: inner, .. } => predicate(inner),
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
        | NirExpr::CpuThreadSpawn { args, .. }
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
