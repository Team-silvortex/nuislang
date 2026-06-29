use super::*;

fn expr_references_any_name(expr: &NirExpr, names: &BTreeSet<String>) -> bool {
    match expr {
        NirExpr::Var(name) => names.contains(name),
        NirExpr::Await(inner)
        | NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::Move(inner)
        | NirExpr::CastI64ToI32(inner)
        | NirExpr::CastI32ToI64(inner)
        | NirExpr::CastI64ToBool(inner)
        | NirExpr::CastBoolToI64(inner)
        | NirExpr::CastI64ToF32(inner)
        | NirExpr::CastF32ToI64(inner)
        | NirExpr::CastI64ToF64(inner)
        | NirExpr::CastF64ToI64(inner)
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
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::DataFreezeWindow(inner)
        | NirExpr::ShaderPassReady(inner)
        | NirExpr::ShaderFrameReady(inner)
        | NirExpr::ShaderValue(inner)
        | NirExpr::NetworkConfigReady(inner)
        | NirExpr::NetworkSendReady(inner)
        | NirExpr::NetworkRecvReady(inner)
        | NirExpr::NetworkAcceptReady(inner)
        | NirExpr::NetworkValue(inner)
        | NirExpr::KernelConfigReady(inner)
        | NirExpr::KernelValue(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner)
        | NirExpr::FieldAccess { base: inner, .. } => expr_references_any_name(inner, names),
        NirExpr::Binary { lhs, rhs, .. } => {
            expr_references_any_name(lhs, names) || expr_references_any_name(rhs, names)
        }
        NirExpr::LoadAt { buffer, index }
        | NirExpr::DataReadWindow {
            window: buffer,
            index,
        } => expr_references_any_name(buffer, names) || expr_references_any_name(index, names),
        NirExpr::StoreValue { target, value }
        | NirExpr::StoreNext {
            target,
            next: value,
        } => expr_references_any_name(target, names) || expr_references_any_name(value, names),
        NirExpr::StoreAt {
            buffer,
            index,
            value,
        }
        | NirExpr::DataWriteWindow {
            window: buffer,
            index,
            value,
        } => {
            expr_references_any_name(buffer, names)
                || expr_references_any_name(index, names)
                || expr_references_any_name(value, names)
        }
        NirExpr::AllocNode { value, next } => {
            expr_references_any_name(value, names) || expr_references_any_name(next, names)
        }
        NirExpr::AllocBuffer { len, fill } => {
            expr_references_any_name(len, names) || expr_references_any_name(fill, names)
        }
        NirExpr::Call { args, .. }
        | NirExpr::CpuExternCall { args, .. }
        | NirExpr::CpuSpawn { args, .. }
        | NirExpr::CpuThreadSpawn { args, .. } => {
            args.iter().any(|arg| expr_references_any_name(arg, names))
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            expr_references_any_name(receiver, names)
                || args.iter().any(|arg| expr_references_any_name(arg, names))
        }
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_references_any_name(value, names)),
        NirExpr::DataResult { value, .. }
        | NirExpr::ShaderResult { value, .. }
        | NirExpr::NetworkResult { value, .. }
        | NirExpr::KernelResult { value, .. } => expr_references_any_name(value, names),
        _ => false,
    }
}

pub(super) fn stmt_references_any_name(stmt: &NirStmt, names: &BTreeSet<String>) -> bool {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Expr(value)
        | NirStmt::Await(value) => expr_references_any_name(value, names),
        NirStmt::Return(Some(value)) => expr_references_any_name(value, names),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_references_any_name(condition, names)
                || then_body
                    .iter()
                    .any(|stmt| stmt_references_any_name(stmt, names))
                || else_body
                    .iter()
                    .any(|stmt| stmt_references_any_name(stmt, names))
        }
        NirStmt::While { condition, body } => {
            expr_references_any_name(condition, names)
                || body
                    .iter()
                    .any(|stmt| stmt_references_any_name(stmt, names))
        }
        NirStmt::Return(None) | NirStmt::Break | NirStmt::Continue => false,
    }
}
