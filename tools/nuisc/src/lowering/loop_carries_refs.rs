use super::*;

pub(in crate::lowering) fn loop_compare_from_binary_op(
    op: NirBinaryOp,
) -> Option<PreparedLoopCompare> {
    match op {
        NirBinaryOp::Eq => Some(PreparedLoopCompare::Eq),
        NirBinaryOp::Ne => Some(PreparedLoopCompare::Ne),
        NirBinaryOp::Lt => Some(PreparedLoopCompare::Lt),
        NirBinaryOp::Le => Some(PreparedLoopCompare::Le),
        NirBinaryOp::Gt => Some(PreparedLoopCompare::Gt),
        NirBinaryOp::Ge => Some(PreparedLoopCompare::Ge),
        _ => None,
    }
}

pub(in crate::lowering) fn render_loop_compare(compare: PreparedLoopCompare) -> &'static str {
    match compare {
        PreparedLoopCompare::Eq => "eq",
        PreparedLoopCompare::Ne => "ne",
        PreparedLoopCompare::Lt => "lt",
        PreparedLoopCompare::Le => "le",
        PreparedLoopCompare::Gt => "gt",
        PreparedLoopCompare::Ge => "ge",
    }
}

pub(in crate::lowering) fn render_loop_cond_kind(
    lhs: &PreparedCarryCondSource,
    compare: PreparedLoopCompare,
) -> String {
    match (lhs, compare) {
        (PreparedCarryCondSource::Current, PreparedLoopCompare::Eq) => "current_eq".to_owned(),
        (PreparedCarryCondSource::Current, PreparedLoopCompare::Ne) => "current_ne".to_owned(),
        (PreparedCarryCondSource::Current, PreparedLoopCompare::Lt) => "current_lt".to_owned(),
        (PreparedCarryCondSource::Current, PreparedLoopCompare::Le) => "current_le".to_owned(),
        (PreparedCarryCondSource::Current, PreparedLoopCompare::Gt) => "current_gt".to_owned(),
        (PreparedCarryCondSource::Current, PreparedLoopCompare::Ge) => "current_ge".to_owned(),
        (PreparedCarryCondSource::PreviousCurrent, PreparedLoopCompare::Eq) => {
            "prev_current_eq".to_owned()
        }
        (PreparedCarryCondSource::PreviousCurrent, PreparedLoopCompare::Ne) => {
            "prev_current_ne".to_owned()
        }
        (PreparedCarryCondSource::PreviousCurrent, PreparedLoopCompare::Lt) => {
            "prev_current_lt".to_owned()
        }
        (PreparedCarryCondSource::PreviousCurrent, PreparedLoopCompare::Le) => {
            "prev_current_le".to_owned()
        }
        (PreparedCarryCondSource::PreviousCurrent, PreparedLoopCompare::Gt) => {
            "prev_current_gt".to_owned()
        }
        (PreparedCarryCondSource::PreviousCurrent, PreparedLoopCompare::Ge) => {
            "prev_current_ge".to_owned()
        }
        (PreparedCarryCondSource::PreviousCarry(index), PreparedLoopCompare::Eq) => {
            format!("prev_carry{index}_eq")
        }
        (PreparedCarryCondSource::PreviousCarry(index), PreparedLoopCompare::Ne) => {
            format!("prev_carry{index}_ne")
        }
        (PreparedCarryCondSource::PreviousCarry(index), PreparedLoopCompare::Lt) => {
            format!("prev_carry{index}_lt")
        }
        (PreparedCarryCondSource::PreviousCarry(index), PreparedLoopCompare::Le) => {
            format!("prev_carry{index}_le")
        }
        (PreparedCarryCondSource::PreviousCarry(index), PreparedLoopCompare::Gt) => {
            format!("prev_carry{index}_gt")
        }
        (PreparedCarryCondSource::PreviousCarry(index), PreparedLoopCompare::Ge) => {
            format!("prev_carry{index}_ge")
        }
        (PreparedCarryCondSource::Carry(index), PreparedLoopCompare::Eq) => {
            format!("carry{index}_eq")
        }
        (PreparedCarryCondSource::Carry(index), PreparedLoopCompare::Ne) => {
            format!("carry{index}_ne")
        }
        (PreparedCarryCondSource::Carry(index), PreparedLoopCompare::Lt) => {
            format!("carry{index}_lt")
        }
        (PreparedCarryCondSource::Carry(index), PreparedLoopCompare::Le) => {
            format!("carry{index}_le")
        }
        (PreparedCarryCondSource::Carry(index), PreparedLoopCompare::Gt) => {
            format!("carry{index}_gt")
        }
        (PreparedCarryCondSource::Carry(index), PreparedLoopCompare::Ge) => {
            format!("carry{index}_ge")
        }
    }
}

pub(in crate::lowering) fn render_loop_logic_op(op: PreparedLoopLogicOp) -> &'static str {
    match op {
        PreparedLoopLogicOp::And => "and",
        PreparedLoopLogicOp::Or => "or",
    }
}

pub(in crate::lowering) fn parse_prepared_loop_state_ref_name_from_carry_names(
    name: &str,
    binding_name: &str,
    carry_binding_names: &[String],
) -> Option<PreparedLoopStateRef> {
    if name == binding_name {
        Some(PreparedLoopStateRef::Current)
    } else if name == TAIL_RECURSIVE_PREV_CURRENT_BINDING {
        Some(PreparedLoopStateRef::PreviousCurrent)
    } else if let Some(index) = name.strip_prefix(TAIL_RECURSIVE_PREV_CARRY_BINDING_PREFIX) {
        index
            .parse::<usize>()
            .ok()
            .map(PreparedLoopStateRef::PreviousCarry)
    } else {
        carry_binding_names
            .iter()
            .position(|carry_name| carry_name == name)
            .map(PreparedLoopStateRef::Carry)
    }
}

pub(in crate::lowering) fn parse_prepared_loop_state_ref_name(
    name: &str,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
) -> Option<PreparedLoopStateRef> {
    let carry_binding_names = carries
        .iter()
        .map(|carry| carry.binding_name.clone())
        .collect::<Vec<_>>();
    parse_prepared_loop_state_ref_name_from_carry_names(name, binding_name, &carry_binding_names)
}

pub(in crate::lowering) fn parse_prepared_loop_state_ref_expr(
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
) -> Option<PreparedLoopStateRef> {
    let NirExpr::Var(name) = expr else {
        return None;
    };
    parse_prepared_loop_state_ref_name(name, binding_name, carries)
}

pub(in crate::lowering) fn loop_state_ref_into_carry_source(
    state_ref: PreparedLoopStateRef,
) -> PreparedCarrySource {
    match state_ref {
        PreparedLoopStateRef::Current => PreparedCarrySource::Current,
        PreparedLoopStateRef::PreviousCurrent => PreparedCarrySource::PreviousCurrent,
        PreparedLoopStateRef::PreviousCarry(index) => PreparedCarrySource::PreviousCarry(index),
        PreparedLoopStateRef::Carry(index) => PreparedCarrySource::Carry(index),
    }
}

pub(in crate::lowering) fn loop_state_ref_into_cond_source(
    state_ref: PreparedLoopStateRef,
) -> PreparedCarryCondSource {
    match state_ref {
        PreparedLoopStateRef::Current => PreparedCarryCondSource::Current,
        PreparedLoopStateRef::PreviousCurrent => PreparedCarryCondSource::PreviousCurrent,
        PreparedLoopStateRef::PreviousCarry(index) => PreparedCarryCondSource::PreviousCarry(index),
        PreparedLoopStateRef::Carry(index) => PreparedCarryCondSource::Carry(index),
    }
}

pub(in crate::lowering) fn expr_contains_loop_variant_name(
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
) -> bool {
    match expr {
        NirExpr::Var(name) => {
            parse_prepared_loop_state_ref_name(name, binding_name, carries).is_some()
        }
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
        | NirExpr::FieldAccess { base: inner, .. } => {
            expr_contains_loop_variant_name(inner, binding_name, carries)
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            expr_contains_loop_variant_name(lhs, binding_name, carries)
                || expr_contains_loop_variant_name(rhs, binding_name, carries)
        }
        NirExpr::LoadAt { buffer, index }
        | NirExpr::DataReadWindow {
            window: buffer,
            index,
        } => {
            expr_contains_loop_variant_name(buffer, binding_name, carries)
                || expr_contains_loop_variant_name(index, binding_name, carries)
        }
        NirExpr::StoreValue { target, value }
        | NirExpr::StoreNext {
            target,
            next: value,
        } => {
            expr_contains_loop_variant_name(target, binding_name, carries)
                || expr_contains_loop_variant_name(value, binding_name, carries)
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
            expr_contains_loop_variant_name(buffer, binding_name, carries)
                || expr_contains_loop_variant_name(index, binding_name, carries)
                || expr_contains_loop_variant_name(value, binding_name, carries)
        }
        NirExpr::AllocNode { value, next } => {
            expr_contains_loop_variant_name(value, binding_name, carries)
                || expr_contains_loop_variant_name(next, binding_name, carries)
        }
        NirExpr::AllocBuffer { len, fill } => {
            expr_contains_loop_variant_name(len, binding_name, carries)
                || expr_contains_loop_variant_name(fill, binding_name, carries)
        }
        NirExpr::Call { args, .. }
        | NirExpr::CpuExternCall { args, .. }
        | NirExpr::CpuSpawn { args, .. }
        | NirExpr::CpuThreadSpawn { args, .. } => args
            .iter()
            .any(|arg| expr_contains_loop_variant_name(arg, binding_name, carries)),
        NirExpr::MethodCall { receiver, args, .. } => {
            expr_contains_loop_variant_name(receiver, binding_name, carries)
                || args
                    .iter()
                    .any(|arg| expr_contains_loop_variant_name(arg, binding_name, carries))
        }
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_contains_loop_variant_name(value, binding_name, carries)),
        NirExpr::DataResult { value, .. }
        | NirExpr::ShaderResult { value, .. }
        | NirExpr::NetworkResult { value, .. }
        | NirExpr::KernelResult { value, .. } => {
            expr_contains_loop_variant_name(value, binding_name, carries)
        }
        _ => false,
    }
}

pub(in crate::lowering) fn expr_is_loop_invariant(
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
) -> bool {
    !expr_contains_loop_variant_name(expr, binding_name, carries)
}
