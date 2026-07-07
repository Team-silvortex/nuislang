use super::*;

pub(in crate::lowering) fn expr_references_names(expr: &NirExpr, names: &BTreeSet<&str>) -> bool {
    match expr {
        NirExpr::Var(name) => names.contains(name.as_str()),
        NirExpr::Await(inner)
        | NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::Move(inner)
        | NirExpr::HostBufferHandle(inner)
        | NirExpr::LoadValue(inner)
        | NirExpr::LoadNext(inner)
        | NirExpr::BufferLen(inner)
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::DataReady(inner)
        | NirExpr::DataMoved(inner)
        | NirExpr::DataWindowed(inner)
        | NirExpr::DataValue(inner)
        | NirExpr::CpuJoin(inner)
        | NirExpr::CpuCancel(inner)
        | NirExpr::CpuJoinResult(inner)
        | NirExpr::CpuTaskCompleted(inner)
        | NirExpr::CpuTaskTimedOut(inner)
        | NirExpr::CpuTaskCancelled(inner)
        | NirExpr::CpuTaskValue(inner)
        | NirExpr::CpuThreadJoin(inner)
        | NirExpr::CpuThreadJoinResult(inner)
        | NirExpr::CpuMutexNew(inner)
        | NirExpr::CpuMutexLock(inner)
        | NirExpr::CpuMutexUnlock(inner)
        | NirExpr::CpuMutexValue(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::NetworkConfigReady(inner)
        | NirExpr::NetworkSendReady(inner)
        | NirExpr::NetworkRecvReady(inner)
        | NirExpr::NetworkAcceptReady(inner)
        | NirExpr::NetworkValue(inner)
        | NirExpr::KernelConfigReady(inner)
        | NirExpr::KernelValue(inner)
        | NirExpr::KernelShape(inner)
        | NirExpr::KernelRows(inner)
        | NirExpr::KernelCols(inner)
        | NirExpr::KernelRelu(inner)
        | NirExpr::KernelReduceSum(inner)
        | NirExpr::KernelReduceMax(inner)
        | NirExpr::KernelReduceMean(inner)
        | NirExpr::KernelArgmax(inner)
        | NirExpr::KernelArgmin(inner)
        | NirExpr::KernelSort(inner)
        | NirExpr::ShaderPassReady(inner)
        | NirExpr::ShaderFrameReady(inner)
        | NirExpr::ShaderValue(inner)
        | NirExpr::CastI64ToI32(inner)
        | NirExpr::CastI32ToI64(inner)
        | NirExpr::CastI64ToBool(inner)
        | NirExpr::CastBoolToI64(inner)
        | NirExpr::CastI64ToF32(inner)
        | NirExpr::CastF32ToI64(inner)
        | NirExpr::CastI64ToF64(inner)
        | NirExpr::CastF64ToI64(inner)
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner) => expr_references_names(inner, names),
        NirExpr::Call { args, .. }
        | NirExpr::CpuSpawn { args, .. }
        | NirExpr::CpuThreadSpawn { args, .. }
        | NirExpr::CpuExternCall { args, .. }
        | NirExpr::CpuExternCallI32 { args, .. } => {
            args.iter().any(|arg| expr_references_names(arg, names))
        }
        NirExpr::MethodCall { receiver, args, .. } => {
            expr_references_names(receiver, names)
                || args.iter().any(|arg| expr_references_names(arg, names))
        }
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_references_names(value, names)),
        NirExpr::FieldAccess { base, .. }
        | NirExpr::VariantIs { base, .. }
        | NirExpr::VariantFieldAccess { base, .. } => expr_references_names(base, names),
        NirExpr::Binary { lhs, rhs, .. }
        | NirExpr::LoadAt {
            buffer: lhs,
            index: rhs,
        }
        | NirExpr::StoreValue {
            target: lhs,
            value: rhs,
        }
        | NirExpr::StoreNext {
            target: lhs,
            next: rhs,
        }
        | NirExpr::AllocNode {
            value: lhs,
            next: rhs,
        }
        | NirExpr::AllocBuffer {
            len: lhs,
            fill: rhs,
        }
        | NirExpr::DataReadWindow {
            window: lhs,
            index: rhs,
        }
        | NirExpr::CpuTimeout {
            task: lhs,
            limit: rhs,
        }
        | NirExpr::KernelElementAt {
            input: lhs,
            row: rhs,
            ..
        }
        | NirExpr::KernelZip { lhs, rhs, .. }
        | NirExpr::KernelMatmul { lhs, rhs }
        | NirExpr::KernelAddBias {
            input: lhs,
            bias: rhs,
        }
        | NirExpr::ShaderSampleUv {
            texture: lhs,
            sampler: rhs,
            ..
        }
        | NirExpr::ShaderBeginPass {
            target: lhs,
            pipeline: rhs,
            ..
        } => expr_references_names(lhs, names) || expr_references_names(rhs, names),
        _ => false,
    }
}
