use super::*;

fn is_branch_local_runtime_observer(expr: &NirExpr) -> bool {
    matches!(
        expr,
        NirExpr::CpuTaskCompleted(_)
            | NirExpr::CpuTaskTimedOut(_)
            | NirExpr::CpuTaskCancelled(_)
            | NirExpr::CpuTaskFailed(_)
            | NirExpr::CpuTaskValue(_)
            | NirExpr::CpuMutexValue(_)
    )
}

fn is_branch_local_runtime_consumer(expr: &NirExpr) -> bool {
    matches!(
        expr,
        NirExpr::Await(_)
            | NirExpr::CpuSpawn { .. }
            | NirExpr::CpuThreadSpawn { .. }
            | NirExpr::CpuJoin(_)
            | NirExpr::CpuThreadJoin(_)
            | NirExpr::CpuCancel(_)
            | NirExpr::CpuJoinResult(_)
            | NirExpr::CpuThreadJoinResult(_)
            | NirExpr::CpuMutexNew(_)
            | NirExpr::CpuMutexLock(_)
            | NirExpr::CpuMutexUnlock(_)
            | NirExpr::CpuTimeout { .. }
            | NirExpr::CpuReadyAfter { .. }
    )
}

pub(super) fn expr_contains_conditional_effect_primitive(expr: &NirExpr) -> bool {
    match expr {
        NirExpr::Await(inner) => !matches!(
            inner.as_ref(),
            NirExpr::Call { .. } | NirExpr::MethodCall { .. }
        ),
        _ if is_branch_local_runtime_consumer(expr) => true,
        _ if is_branch_local_runtime_observer(expr) => false,
        NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::HostBufferHandle(inner)
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
        | NirExpr::Free(inner)
        | NirExpr::IsNull(inner)
        | NirExpr::DataReady(inner)
        | NirExpr::DataMoved(inner)
        | NirExpr::DataWindowed(inner)
        | NirExpr::DataValue(inner)
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
        | NirExpr::KernelShape(inner)
        | NirExpr::KernelRows(inner)
        | NirExpr::KernelCols(inner)
        | NirExpr::KernelRow(inner)
        | NirExpr::KernelCol(inner)
        | NirExpr::KernelRelu(inner)
        | NirExpr::KernelReduceSum(inner)
        | NirExpr::KernelReduceMax(inner)
        | NirExpr::KernelReduceMean(inner)
        | NirExpr::KernelArgmax(inner)
        | NirExpr::KernelArgmin(inner)
        | NirExpr::DataOutputPipe(inner)
        | NirExpr::DataInputPipe(inner)
        | NirExpr::CpuPresentFrame(inner)
        | NirExpr::FieldAccess { base: inner, .. }
        | NirExpr::KernelReshape { input: inner, .. }
        | NirExpr::KernelBroadcast { input: inner, .. }
        | NirExpr::KernelSort(inner)
        | NirExpr::DataResult { value: inner, .. }
        | NirExpr::ShaderResult { value: inner, .. }
        | NirExpr::KernelResult { value: inner, .. }
        | NirExpr::NetworkResult { value: inner, .. } => {
            expr_contains_conditional_effect_primitive(inner)
        }
        NirExpr::Binary { lhs, rhs, .. }
        | NirExpr::KernelMatmul { lhs, rhs }
        | NirExpr::KernelZip { lhs, rhs, .. }
        | NirExpr::KernelAddBias {
            input: lhs,
            bias: rhs,
        } => {
            expr_contains_conditional_effect_primitive(lhs)
                || expr_contains_conditional_effect_primitive(rhs)
        }
        NirExpr::AllocNode { value, next } => {
            expr_contains_conditional_effect_primitive(value)
                || expr_contains_conditional_effect_primitive(next)
        }
        NirExpr::AllocBuffer { len, fill } => {
            expr_contains_conditional_effect_primitive(len)
                || expr_contains_conditional_effect_primitive(fill)
        }
        NirExpr::StoreValue { target, value }
        | NirExpr::StoreNext {
            target,
            next: value,
        } => {
            expr_contains_conditional_effect_primitive(target)
                || expr_contains_conditional_effect_primitive(value)
        }
        NirExpr::LoadAt { buffer, index }
        | NirExpr::DataReadWindow {
            window: buffer,
            index,
        } => {
            expr_contains_conditional_effect_primitive(buffer)
                || expr_contains_conditional_effect_primitive(index)
        }
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
            expr_contains_conditional_effect_primitive(buffer)
                || expr_contains_conditional_effect_primitive(index)
                || expr_contains_conditional_effect_primitive(value)
        }
        NirExpr::Call { args, .. }
        | NirExpr::MethodCall { args, .. }
        | NirExpr::CpuExternCall { args, .. } => {
            args.iter().any(expr_contains_conditional_effect_primitive)
        }
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_contains_conditional_effect_primitive(value)),
        NirExpr::KernelMap { input, scalar, .. } | NirExpr::KernelMapAxis { input, scalar, .. } => {
            expr_contains_conditional_effect_primitive(input)
                || scalar
                    .as_ref()
                    .is_some_and(|value| expr_contains_conditional_effect_primitive(value))
        }
        NirExpr::KernelElementAt { input, row, col } => {
            expr_contains_conditional_effect_primitive(input)
                || expr_contains_conditional_effect_primitive(row)
                || expr_contains_conditional_effect_primitive(col)
        }
        NirExpr::KernelArgmaxAxis { input, .. }
        | NirExpr::KernelArgminAxis { input, .. }
        | NirExpr::KernelReduceSumAxis { input, .. }
        | NirExpr::KernelReduceMaxAxis { input, .. }
        | NirExpr::KernelReduceMeanAxis { input, .. }
        | NirExpr::KernelSortAxis { input, .. }
        | NirExpr::KernelTopkAxis { input, .. }
        | NirExpr::KernelTopk { input, .. } => expr_contains_conditional_effect_primitive(input),
        NirExpr::Null
        | NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::Var(_)
        | NirExpr::Instantiate { .. }
        | NirExpr::CpuWindow { .. }
        | NirExpr::CpuInputI64 { .. }
        | NirExpr::CpuTickI64 { .. }
        | NirExpr::DataCopyWindow { .. }
        | NirExpr::DataImmutableWindow { .. }
        | NirExpr::DataProfileSendUplink { .. }
        | NirExpr::DataProfileSendDownlink { .. }
        | NirExpr::ShaderTarget { .. }
        | NirExpr::ShaderViewport { .. }
        | NirExpr::ShaderPipeline { .. }
        | NirExpr::ShaderInlineWgsl { .. }
        | NirExpr::DataBindCore(_)
        | NirExpr::DataMarker(_)
        | NirExpr::DataHandleTable(_)
        | NirExpr::CpuBindCore(_)
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
        | NirExpr::DataProfileMarkerRef { .. }
        | NirExpr::DataProfileHandleTableRef { .. }
        | NirExpr::NetworkProfileTimeoutBudgetRef { .. }
        | NirExpr::NetworkProfileRetryBudgetRef { .. }
        | NirExpr::NetworkProfileStreamWindowRef { .. }
        | NirExpr::NetworkProfileRecvWindowRef { .. }
        | NirExpr::NetworkProfileSendWindowRef { .. }
        | NirExpr::NetworkProfileProtocolKindRef { .. }
        | NirExpr::NetworkProfileProtocolVersionRef { .. }
        | NirExpr::NetworkProfileProtocolHeaderBytesRef { .. }
        | NirExpr::KernelProfileBindCoreRef { .. }
        | NirExpr::KernelProfileQueueDepthRef { .. }
        | NirExpr::KernelProfileBatchLanesRef { .. }
        | NirExpr::KernelTensor { .. } => false,
        _ => false,
    }
}

fn stmt_contains_conditional_effect_primitive(stmt: &NirStmt) -> bool {
    match stmt {
        NirStmt::Let { value, .. }
        | NirStmt::Const { value, .. }
        | NirStmt::Print(value)
        | NirStmt::Await(value)
        | NirStmt::Expr(value)
        | NirStmt::Return(Some(value)) => expr_contains_conditional_effect_primitive(value),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_contains_conditional_effect_primitive(condition)
                || then_body
                    .iter()
                    .any(stmt_contains_conditional_effect_primitive)
                || else_body
                    .iter()
                    .any(stmt_contains_conditional_effect_primitive)
        }
        NirStmt::While { condition, body } => {
            expr_contains_conditional_effect_primitive(condition)
                || body.iter().any(stmt_contains_conditional_effect_primitive)
        }
        NirStmt::Return(None) | NirStmt::Break | NirStmt::Continue => false,
    }
}

pub(in crate::lowering) fn stmts_contain_conditional_effect_primitive(stmts: &[NirStmt]) -> bool {
    stmts.iter().any(stmt_contains_conditional_effect_primitive)
}
