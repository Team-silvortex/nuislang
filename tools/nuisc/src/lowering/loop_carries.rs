use super::*;

pub(super) fn loop_compare_from_binary_op(op: NirBinaryOp) -> Option<PreparedLoopCompare> {
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

pub(super) fn render_loop_compare(compare: PreparedLoopCompare) -> &'static str {
    match compare {
        PreparedLoopCompare::Eq => "eq",
        PreparedLoopCompare::Ne => "ne",
        PreparedLoopCompare::Lt => "lt",
        PreparedLoopCompare::Le => "le",
        PreparedLoopCompare::Gt => "gt",
        PreparedLoopCompare::Ge => "ge",
    }
}

pub(super) fn render_loop_cond_kind(
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

pub(super) fn render_loop_logic_op(op: PreparedLoopLogicOp) -> &'static str {
    match op {
        PreparedLoopLogicOp::And => "and",
        PreparedLoopLogicOp::Or => "or",
    }
}

pub(super) fn parse_prepared_loop_state_ref_name_from_carry_names(
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

pub(super) fn parse_prepared_loop_state_ref_name(
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

pub(super) fn parse_prepared_loop_state_ref_expr(
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
) -> Option<PreparedLoopStateRef> {
    let NirExpr::Var(name) = expr else {
        return None;
    };
    parse_prepared_loop_state_ref_name(name, binding_name, carries)
}

pub(super) fn loop_state_ref_into_carry_source(
    state_ref: PreparedLoopStateRef,
) -> PreparedCarrySource {
    match state_ref {
        PreparedLoopStateRef::Current => PreparedCarrySource::Current,
        PreparedLoopStateRef::PreviousCurrent => PreparedCarrySource::PreviousCurrent,
        PreparedLoopStateRef::PreviousCarry(index) => PreparedCarrySource::PreviousCarry(index),
        PreparedLoopStateRef::Carry(index) => PreparedCarrySource::Carry(index),
    }
}

pub(super) fn loop_state_ref_into_cond_source(
    state_ref: PreparedLoopStateRef,
) -> PreparedCarryCondSource {
    match state_ref {
        PreparedLoopStateRef::Current => PreparedCarryCondSource::Current,
        PreparedLoopStateRef::PreviousCurrent => PreparedCarryCondSource::PreviousCurrent,
        PreparedLoopStateRef::PreviousCarry(index) => PreparedCarryCondSource::PreviousCarry(index),
        PreparedLoopStateRef::Carry(index) => PreparedCarryCondSource::Carry(index),
    }
}

fn expr_contains_loop_variant_name(
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

fn expr_is_loop_invariant(
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
) -> bool {
    !expr_contains_loop_variant_name(expr, binding_name, carries)
}

#[derive(Clone)]
struct ParsedAdditiveCarrySource {
    terms: Vec<PreparedLoopStateRef>,
    offset: Option<NirExpr>,
}

fn combine_invariant_additive_terms(terms: Vec<NirExpr>) -> Option<NirExpr> {
    let mut iter = terms.into_iter();
    let first = iter.next()?;
    Some(iter.fold(first, |lhs, rhs| NirExpr::Binary {
        op: NirBinaryOp::Add,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }))
}

fn parse_additive_carry_source(
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
) -> Option<PreparedCarrySource> {
    fn parse_inner(
        expr: &NirExpr,
        binding_name: &str,
        carries: &[PreparedCarryUpdate],
    ) -> Option<ParsedAdditiveCarrySource> {
        if let Some(state_ref) = parse_prepared_loop_state_ref_expr(expr, binding_name, carries) {
            return Some(ParsedAdditiveCarrySource {
                terms: vec![state_ref],
                offset: None,
            });
        }
        if is_terminal_branch_pure_expr(expr, &BTreeSet::new())
            && expr_is_loop_invariant(expr, binding_name, carries)
        {
            return Some(ParsedAdditiveCarrySource {
                terms: Vec::new(),
                offset: Some(expr.clone()),
            });
        }
        match expr {
            NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs,
                rhs,
            } => {
                let lhs = parse_inner(lhs, binding_name, carries)?;
                let rhs = parse_inner(rhs, binding_name, carries)?;
                let mut terms = lhs.terms;
                terms.extend(rhs.terms);
                let offset = combine_invariant_additive_terms(
                    lhs.offset.into_iter().chain(rhs.offset).collect::<Vec<_>>(),
                );
                Some(ParsedAdditiveCarrySource { terms, offset })
            }
            _ => None,
        }
    }

    let parsed = parse_inner(expr, binding_name, carries)?;
    match (parsed.terms.len(), parsed.offset) {
        (0, Some(invariant)) => Some(PreparedCarrySource::InvariantExpr(invariant)),
        (1, Some(invariant)) => Some(PreparedCarrySource::AddInvariant {
            base: Box::new(loop_state_ref_into_carry_source(parsed.terms[0])),
            offset: invariant,
        }),
        (count, offset) if count >= 2 => Some(PreparedCarrySource::AddStateList {
            terms: parsed.terms,
            offset,
        }),
        _ => None,
    }
}

fn parse_scaled_additive_carry_source(
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
) -> Option<PreparedCarrySource> {
    let NirExpr::Binary {
        op: NirBinaryOp::Mul,
        lhs,
        rhs,
    } = expr
    else {
        return None;
    };
    let invariant = |expr: &NirExpr| {
        is_terminal_branch_pure_expr(expr, &BTreeSet::new())
            && expr_is_loop_invariant(expr, binding_name, carries)
    };
    let into_scaled_state_list = |source: PreparedCarrySource,
                                  factor: NirExpr|
     -> Option<PreparedCarrySource> {
        match source {
            PreparedCarrySource::AddStateList { terms, offset } => {
                let scaled_offset = offset.map(|offset| NirExpr::Binary {
                    op: NirBinaryOp::Mul,
                    lhs: Box::new(offset),
                    rhs: Box::new(factor.clone()),
                });
                Some(PreparedCarrySource::ScaledStateList {
                    terms,
                    factor,
                    offset: scaled_offset,
                })
            }
            PreparedCarrySource::AddInvariant { base, offset } => {
                let state_ref = match *base {
                    PreparedCarrySource::Current => PreparedLoopStateRef::Current,
                    PreparedCarrySource::PreviousCurrent => PreparedLoopStateRef::PreviousCurrent,
                    PreparedCarrySource::PreviousCarry(index) => {
                        PreparedLoopStateRef::PreviousCarry(index)
                    }
                    PreparedCarrySource::Carry(index) => PreparedLoopStateRef::Carry(index),
                    _ => return None,
                };
                let scaled_offset = NirExpr::Binary {
                    op: NirBinaryOp::Mul,
                    lhs: Box::new(offset),
                    rhs: Box::new(factor.clone()),
                };
                Some(PreparedCarrySource::ScaledStateList {
                    terms: vec![state_ref],
                    factor,
                    offset: Some(scaled_offset),
                })
            }
            PreparedCarrySource::Current => Some(PreparedCarrySource::ScaledStateList {
                terms: vec![PreparedLoopStateRef::Current],
                factor,
                offset: None,
            }),
            PreparedCarrySource::PreviousCurrent => Some(PreparedCarrySource::ScaledStateList {
                terms: vec![PreparedLoopStateRef::PreviousCurrent],
                factor,
                offset: None,
            }),
            PreparedCarrySource::PreviousCarry(index) => {
                Some(PreparedCarrySource::ScaledStateList {
                    terms: vec![PreparedLoopStateRef::PreviousCarry(index)],
                    factor,
                    offset: None,
                })
            }
            PreparedCarrySource::Carry(index) => Some(PreparedCarrySource::ScaledStateList {
                terms: vec![PreparedLoopStateRef::Carry(index)],
                factor,
                offset: None,
            }),
            _ => None,
        }
    };
    if let Some(source) = parse_additive_carry_source(lhs, binding_name, carries) {
        if invariant(rhs) {
            return into_scaled_state_list(source, (**rhs).clone());
        }
    }
    if let Some(source) = parse_additive_carry_source(rhs, binding_name, carries) {
        if invariant(lhs) {
            return into_scaled_state_list(source, (**lhs).clone());
        }
    }
    None
}

fn parse_state_scaled_additive_carry_source(
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
) -> Option<PreparedCarrySource> {
    let NirExpr::Binary {
        op: NirBinaryOp::Mul,
        lhs,
        rhs,
    } = expr
    else {
        return None;
    };
    let into_scaled_by_state = |source: PreparedCarrySource,
                                factor: PreparedLoopStateRef|
     -> Option<PreparedCarrySource> {
        match source {
            PreparedCarrySource::AddStateList { terms, offset } => {
                Some(PreparedCarrySource::ScaledStateListByState {
                    terms,
                    factor,
                    offset,
                })
            }
            PreparedCarrySource::AddInvariant { base, offset } => {
                let state_ref = match *base {
                    PreparedCarrySource::Current => PreparedLoopStateRef::Current,
                    PreparedCarrySource::PreviousCurrent => PreparedLoopStateRef::PreviousCurrent,
                    PreparedCarrySource::PreviousCarry(index) => {
                        PreparedLoopStateRef::PreviousCarry(index)
                    }
                    PreparedCarrySource::Carry(index) => PreparedLoopStateRef::Carry(index),
                    _ => return None,
                };
                Some(PreparedCarrySource::ScaledStateListByState {
                    terms: vec![state_ref],
                    factor,
                    offset: Some(offset),
                })
            }
            PreparedCarrySource::Current => Some(PreparedCarrySource::ScaledStateListByState {
                terms: vec![PreparedLoopStateRef::Current],
                factor,
                offset: None,
            }),
            PreparedCarrySource::PreviousCurrent => {
                Some(PreparedCarrySource::ScaledStateListByState {
                    terms: vec![PreparedLoopStateRef::PreviousCurrent],
                    factor,
                    offset: None,
                })
            }
            PreparedCarrySource::PreviousCarry(index) => {
                Some(PreparedCarrySource::ScaledStateListByState {
                    terms: vec![PreparedLoopStateRef::PreviousCarry(index)],
                    factor,
                    offset: None,
                })
            }
            PreparedCarrySource::Carry(index) => {
                Some(PreparedCarrySource::ScaledStateListByState {
                    terms: vec![PreparedLoopStateRef::Carry(index)],
                    factor,
                    offset: None,
                })
            }
            _ => None,
        }
    };
    if let Some(factor) = parse_prepared_loop_state_ref_expr(rhs, binding_name, carries) {
        if let Some(source) = parse_additive_carry_source(lhs, binding_name, carries) {
            return into_scaled_by_state(source, factor);
        }
    }
    if let Some(factor) = parse_prepared_loop_state_ref_expr(lhs, binding_name, carries) {
        if let Some(source) = parse_additive_carry_source(rhs, binding_name, carries) {
            return into_scaled_by_state(source, factor);
        }
    }
    None
}

fn parse_state_plus_invariant_scaled_additive_carry_source(
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
) -> Option<PreparedCarrySource> {
    let NirExpr::Binary {
        op: NirBinaryOp::Mul,
        lhs,
        rhs,
    } = expr
    else {
        return None;
    };
    let parse_factor = |expr: &NirExpr| -> Option<(PreparedLoopStateRef, NirExpr)> {
        let source = parse_additive_carry_source(expr, binding_name, carries)?;
        match source {
            PreparedCarrySource::AddInvariant { base, offset } => {
                let factor = match *base {
                    PreparedCarrySource::Current => PreparedLoopStateRef::Current,
                    PreparedCarrySource::PreviousCurrent => PreparedLoopStateRef::PreviousCurrent,
                    PreparedCarrySource::PreviousCarry(index) => {
                        PreparedLoopStateRef::PreviousCarry(index)
                    }
                    PreparedCarrySource::Carry(index) => PreparedLoopStateRef::Carry(index),
                    _ => return None,
                };
                Some((factor, offset))
            }
            _ => None,
        }
    };
    let into_scaled_by_state_plus_invariant = |source: PreparedCarrySource,
                                               factor: PreparedLoopStateRef,
                                               factor_offset: NirExpr|
     -> Option<PreparedCarrySource> {
        match source {
            PreparedCarrySource::AddStateList { terms, offset } => {
                Some(PreparedCarrySource::ScaledStateListByStatePlusInvariant {
                    terms,
                    factor,
                    factor_offset,
                    offset,
                })
            }
            PreparedCarrySource::AddInvariant { base, offset } => {
                let state_ref = match *base {
                    PreparedCarrySource::Current => PreparedLoopStateRef::Current,
                    PreparedCarrySource::PreviousCurrent => PreparedLoopStateRef::PreviousCurrent,
                    PreparedCarrySource::PreviousCarry(index) => {
                        PreparedLoopStateRef::PreviousCarry(index)
                    }
                    PreparedCarrySource::Carry(index) => PreparedLoopStateRef::Carry(index),
                    _ => return None,
                };
                Some(PreparedCarrySource::ScaledStateListByStatePlusInvariant {
                    terms: vec![state_ref],
                    factor,
                    factor_offset,
                    offset: Some(offset),
                })
            }
            PreparedCarrySource::Current => {
                Some(PreparedCarrySource::ScaledStateListByStatePlusInvariant {
                    terms: vec![PreparedLoopStateRef::Current],
                    factor,
                    factor_offset,
                    offset: None,
                })
            }
            PreparedCarrySource::PreviousCurrent => {
                Some(PreparedCarrySource::ScaledStateListByStatePlusInvariant {
                    terms: vec![PreparedLoopStateRef::PreviousCurrent],
                    factor,
                    factor_offset,
                    offset: None,
                })
            }
            PreparedCarrySource::PreviousCarry(index) => {
                Some(PreparedCarrySource::ScaledStateListByStatePlusInvariant {
                    terms: vec![PreparedLoopStateRef::PreviousCarry(index)],
                    factor,
                    factor_offset,
                    offset: None,
                })
            }
            PreparedCarrySource::Carry(index) => {
                Some(PreparedCarrySource::ScaledStateListByStatePlusInvariant {
                    terms: vec![PreparedLoopStateRef::Carry(index)],
                    factor,
                    factor_offset,
                    offset: None,
                })
            }
            _ => None,
        }
    };
    if let Some((factor, factor_offset)) = parse_factor(rhs) {
        if let Some(source) = parse_additive_carry_source(lhs, binding_name, carries) {
            return into_scaled_by_state_plus_invariant(source, factor, factor_offset);
        }
    }
    if let Some((factor, factor_offset)) = parse_factor(lhs) {
        if let Some(source) = parse_additive_carry_source(rhs, binding_name, carries) {
            return into_scaled_by_state_plus_invariant(source, factor, factor_offset);
        }
    }
    None
}

fn parse_loop_variant_source_name(
    rhs_name: &str,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
) -> Option<PreparedCarrySource> {
    parse_prepared_loop_state_ref_name(rhs_name, binding_name, carries)
        .map(loop_state_ref_into_carry_source)
}

fn parse_linear_var_source(
    op: PreparedCarryLinearOp,
    rhs_name: &str,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
) -> Option<(PreparedCarryLinearOp, PreparedCarrySource)> {
    parse_loop_variant_source_name(rhs_name, binding_name, carries).map(|source| (op, source))
}

pub(super) fn parse_prepared_readable_carry_source_candidate(
    rhs: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
) -> Option<PreparedReadableCarrySourceCandidate> {
    match rhs {
        NirExpr::LoadValue(inner) if expr_is_loop_invariant(inner, binding_name, carries) => {
            Some(PreparedReadableCarrySourceCandidate::Fixed(
                PreparedFixedReadCarrySource::Value((**inner).clone()),
            ))
        }
        NirExpr::LoadAt { buffer, index }
            if expr_is_loop_invariant(buffer, binding_name, carries)
                && expr_is_loop_invariant(index, binding_name, carries) =>
        {
            Some(PreparedReadableCarrySourceCandidate::Fixed(
                PreparedFixedReadCarrySource::At {
                    buffer: (**buffer).clone(),
                    index: (**index).clone(),
                },
            ))
        }
        NirExpr::LoadAt { buffer, index }
            if expr_is_loop_invariant(buffer, binding_name, carries)
                && !expr_is_loop_invariant(index, binding_name, carries) =>
        {
            Some(PreparedReadableCarrySourceCandidate::DynamicIndexAt {
                buffer: (**buffer).clone(),
                index: (**index).clone(),
            })
        }
        NirExpr::LoadNext(inner) => Some(PreparedReadableCarrySourceCandidate::TraversalNext {
            base: (**inner).clone(),
        }),
        _ => None,
    }
}

pub(super) fn parse_prepared_fixed_read_carry_source(
    rhs: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
) -> Option<PreparedFixedReadCarrySource> {
    let candidate = parse_prepared_readable_carry_source_candidate(rhs, binding_name, carries)?;
    match candidate.family() {
        PreparedReadableCarrySourceFamily::Fixed => candidate.fixed_read().cloned(),
        PreparedReadableCarrySourceFamily::DynamicIndexAt
        | PreparedReadableCarrySourceFamily::TraversalNext => None,
    }
}

pub(super) fn parse_prepared_dynamic_read_carry_source(
    rhs: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
) -> Option<PreparedCarrySource> {
    let candidate = parse_prepared_readable_carry_source_candidate(rhs, binding_name, carries)?;
    match candidate {
        PreparedReadableCarrySourceCandidate::DynamicIndexAt { buffer, index } => {
            let index_source = loop_state_ref_into_carry_source(
                parse_prepared_loop_state_ref_expr(&index, binding_name, carries)?,
            );
            Some(PreparedCarrySource::DynamicReadAt {
                buffer,
                index_source: Box::new(index_source),
            })
        }
        PreparedReadableCarrySourceCandidate::Fixed(_)
        | PreparedReadableCarrySourceCandidate::TraversalNext { .. } => None,
    }
}

fn unsupported_readable_carry_source_message(
    candidate: &PreparedReadableCarrySourceCandidate,
) -> Option<String> {
    match candidate.family() {
        PreparedReadableCarrySourceFamily::Fixed => None,
        PreparedReadableCarrySourceFamily::DynamicIndexAt => Some(
            "loop carry updates using dynamic `load_at(buffer, index)` reads currently support only direct loop-state index drivers like `current`, `prev_current`, `prev_carryN`, or earlier `carryN`; more general dynamic index expressions are not supported yet in lowering"
                .to_owned(),
        ),
        PreparedReadableCarrySourceFamily::TraversalNext => Some(
            "loop carry updates using `load_next(...)` traversal reads are recognized but not supported yet in lowering; only loop-invariant fixed carry reads are currently supported"
                .to_owned(),
        ),
    }
}

pub(super) fn diagnose_unsupported_loop_carry_expr(
    carry_name: &str,
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<String> {
    let normalized = inline_pure_helper_calls(expr, inlineable_pure_helpers);
    if parse_loop_carry_keep_source(carry_name, &normalized, carries).is_some() {
        return None;
    }
    let (_, rhs) = parse_loop_carry_linear_shape(carry_name, &normalized)?;
    let candidate = parse_prepared_readable_carry_source_candidate(rhs, binding_name, carries)?;
    unsupported_readable_carry_source_message(&candidate)
}

pub(super) fn render_loop_carry_kind(
    op: PreparedCarryLinearOp,
    source: &PreparedCarrySource,
) -> String {
    source.contract_kind(op)
}

pub(super) fn lower_prepared_fixed_read_carry_source_args(
    source: &PreparedFixedReadCarrySource,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<(Vec<String>, Vec<String>), String> {
    match source {
        PreparedFixedReadCarrySource::Value(expr) => {
            let ptr_name = lower_expr(expr, state, bindings)?;
            Ok((vec![ptr_name.clone()], vec![ptr_name]))
        }
        PreparedFixedReadCarrySource::At { buffer, index } => {
            let buffer_name = lower_expr(buffer, state, bindings)?;
            let index_name = lower_expr(index, state, bindings)?;
            Ok((
                vec![buffer_name.clone(), index_name.clone()],
                vec![buffer_name, index_name],
            ))
        }
    }
}

pub(super) fn encode_loop_carry_source_args(
    op: PreparedCarryLinearOp,
    source: &PreparedCarrySource,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<(Vec<String>, Vec<String>, Vec<String>), String> {
    let kind = render_loop_carry_kind(op, source);
    if let Some(expr) = source.invariant_expr() {
        let expr_name = lower_expr(expr, state, bindings)?;
        return Ok((
            vec![kind, expr_name.clone()],
            vec![expr_name.clone()],
            vec![expr_name],
        ));
    }
    if let Some((_, offset)) = source.add_state_list() {
        if let Some(offset) = offset {
            let offset_name = lower_expr(offset, state, bindings)?;
            return Ok((
                vec![kind, offset_name.clone()],
                vec![offset_name.clone()],
                vec![offset_name],
            ));
        }
        return Ok((vec![kind], vec![], vec![]));
    }
    if let Some((_, factor, offset)) = source.scaled_state_list() {
        let factor_name = lower_expr(factor, state, bindings)?;
        let mut args = vec![kind, factor_name.clone()];
        let mut dep_inputs = vec![factor_name.clone()];
        let mut effect_inputs = vec![factor_name];
        if let Some(offset) = offset {
            let offset_name = lower_expr(offset, state, bindings)?;
            args.push(offset_name.clone());
            dep_inputs.push(offset_name.clone());
            effect_inputs.push(offset_name);
        }
        return Ok((args, dep_inputs, effect_inputs));
    }
    if let Some((_, _, offset)) = source.scaled_state_list_by_state() {
        if let Some(offset) = offset {
            let offset_name = lower_expr(offset, state, bindings)?;
            return Ok((
                vec![kind, offset_name.clone()],
                vec![offset_name.clone()],
                vec![offset_name],
            ));
        }
        return Ok((vec![kind], vec![], vec![]));
    }
    if let Some((_, _, factor_offset, offset)) = source.scaled_state_list_by_state_plus_invariant()
    {
        let factor_offset_name = lower_expr(factor_offset, state, bindings)?;
        let mut args = vec![kind, factor_offset_name.clone()];
        let mut dep_inputs = vec![factor_offset_name.clone()];
        let mut effect_inputs = vec![factor_offset_name];
        if let Some(offset) = offset {
            let offset_name = lower_expr(offset, state, bindings)?;
            args.push(offset_name.clone());
            dep_inputs.push(offset_name.clone());
            effect_inputs.push(offset_name);
        }
        return Ok((args, dep_inputs, effect_inputs));
    }
    if let Some((_, _, factor_offset, offset)) = source.scaled_state_list_by_factor_state_list() {
        let mut args = vec![kind];
        let mut dep_inputs = Vec::new();
        let mut effect_inputs = Vec::new();
        if let Some(factor_offset) = factor_offset {
            let factor_offset_name = lower_expr(factor_offset, state, bindings)?;
            args.push(factor_offset_name.clone());
            dep_inputs.push(factor_offset_name.clone());
            effect_inputs.push(factor_offset_name);
        }
        if let Some(offset) = offset {
            let offset_name = lower_expr(offset, state, bindings)?;
            args.push(offset_name.clone());
            dep_inputs.push(offset_name.clone());
            effect_inputs.push(offset_name);
        }
        return Ok((args, dep_inputs, effect_inputs));
    }
    if let Some((_, _, factor_scale, factor_offset, offset)) =
        source.scaled_state_list_by_factor_state_list_times_invariant()
    {
        let factor_scale_name = lower_expr(factor_scale, state, bindings)?;
        let mut args = vec![kind, factor_scale_name.clone()];
        let mut dep_inputs = vec![factor_scale_name.clone()];
        let mut effect_inputs = vec![factor_scale_name];
        if let Some(factor_offset) = factor_offset {
            let factor_offset_name = lower_expr(factor_offset, state, bindings)?;
            args.push(factor_offset_name.clone());
            dep_inputs.push(factor_offset_name.clone());
            effect_inputs.push(factor_offset_name);
        }
        if let Some(offset) = offset {
            let offset_name = lower_expr(offset, state, bindings)?;
            args.push(offset_name.clone());
            dep_inputs.push(offset_name.clone());
            effect_inputs.push(offset_name);
        }
        return Ok((args, dep_inputs, effect_inputs));
    }
    if let Some((_, _, lhs_factor_offset, _, rhs_factor_offset, offset)) =
        source.scaled_state_list_by_factor_group_product()
    {
        let mut args = vec![kind];
        let mut dep_inputs = Vec::new();
        let mut effect_inputs = Vec::new();
        if let Some(lhs_factor_offset) = lhs_factor_offset {
            let lhs_factor_offset_name = lower_expr(lhs_factor_offset, state, bindings)?;
            args.push(lhs_factor_offset_name.clone());
            dep_inputs.push(lhs_factor_offset_name.clone());
            effect_inputs.push(lhs_factor_offset_name);
        }
        if let Some(rhs_factor_offset) = rhs_factor_offset {
            let rhs_factor_offset_name = lower_expr(rhs_factor_offset, state, bindings)?;
            args.push(rhs_factor_offset_name.clone());
            dep_inputs.push(rhs_factor_offset_name.clone());
            effect_inputs.push(rhs_factor_offset_name);
        }
        if let Some(offset) = offset {
            let offset_name = lower_expr(offset, state, bindings)?;
            args.push(offset_name.clone());
            dep_inputs.push(offset_name.clone());
            effect_inputs.push(offset_name);
        }
        return Ok((args, dep_inputs, effect_inputs));
    }
    if let Some((_, _, lhs_factor_offset, _, rhs_factor_offset, factor_scale, offset)) =
        source.scaled_state_list_by_factor_group_product_times_invariant()
    {
        let factor_scale_name = lower_expr(factor_scale, state, bindings)?;
        let mut args = vec![kind, factor_scale_name.clone()];
        let mut dep_inputs = vec![factor_scale_name.clone()];
        let mut effect_inputs = vec![factor_scale_name];
        if let Some(lhs_factor_offset) = lhs_factor_offset {
            let lhs_factor_offset_name = lower_expr(lhs_factor_offset, state, bindings)?;
            args.push(lhs_factor_offset_name.clone());
            dep_inputs.push(lhs_factor_offset_name.clone());
            effect_inputs.push(lhs_factor_offset_name);
        }
        if let Some(rhs_factor_offset) = rhs_factor_offset {
            let rhs_factor_offset_name = lower_expr(rhs_factor_offset, state, bindings)?;
            args.push(rhs_factor_offset_name.clone());
            dep_inputs.push(rhs_factor_offset_name.clone());
            effect_inputs.push(rhs_factor_offset_name);
        }
        if let Some(offset) = offset {
            let offset_name = lower_expr(offset, state, bindings)?;
            args.push(offset_name.clone());
            dep_inputs.push(offset_name.clone());
            effect_inputs.push(offset_name);
        }
        return Ok((args, dep_inputs, effect_inputs));
    }
    if let Some((base, offset)) = source.add_invariant() {
        let offset_name = lower_expr(offset, state, bindings)?;
        let mut args = vec![kind];
        let mut dep_inputs = vec![offset_name.clone()];
        let mut effect_inputs = vec![offset_name];
        match base {
            PreparedCarrySource::Current
            | PreparedCarrySource::PreviousCurrent
            | PreparedCarrySource::PreviousCarry(_)
            | PreparedCarrySource::Carry(_) => {}
            PreparedCarrySource::FixedRead(fixed_read) => {
                let (base_dep_inputs, base_effect_inputs) =
                    lower_prepared_fixed_read_carry_source_args(fixed_read, state, bindings)?;
                args.extend(base_dep_inputs.iter().cloned());
                dep_inputs.extend(base_dep_inputs);
                effect_inputs.extend(base_effect_inputs);
            }
            PreparedCarrySource::DynamicReadAt { buffer, .. } => {
                let buffer_name = lower_expr(buffer, state, bindings)?;
                args.push(buffer_name.clone());
                dep_inputs.push(buffer_name.clone());
                effect_inputs.push(buffer_name);
            }
            PreparedCarrySource::InvariantExpr(_)
            | PreparedCarrySource::AddInvariant { .. }
            | PreparedCarrySource::AddStateList { .. }
            | PreparedCarrySource::ScaledStateList { .. }
            | PreparedCarrySource::ScaledStateListByState { .. }
            | PreparedCarrySource::ScaledStateListByStatePlusInvariant { .. }
            | PreparedCarrySource::ScaledStateListByFactorStateList { .. }
            | PreparedCarrySource::ScaledStateListByFactorStateListTimesInvariant { .. }
            | PreparedCarrySource::ScaledStateListByFactorGroupProduct { .. }
            | PreparedCarrySource::ScaledStateListByFactorGroupProductTimesInvariant { .. } => {
                unreachable!("nested invariant affine carry sources are not supported")
            }
        }
        args.extend(dep_inputs.iter().take(1).cloned());
        return Ok((args, dep_inputs, effect_inputs));
    }
    if !source.is_fixed_read() && !source.is_dynamic_read_at() {
        return Ok((vec![kind], vec![], vec![]));
    }
    if let Some(fixed_read) = source.fixed_read() {
        let (dep_inputs, effect_inputs) =
            lower_prepared_fixed_read_carry_source_args(fixed_read, state, bindings)?;
        let mut args = vec![kind];
        args.extend(dep_inputs.iter().cloned());
        return Ok((args, dep_inputs, effect_inputs));
    }
    if let Some((buffer, _)) = source.dynamic_read_at() {
        let buffer_name = lower_expr(buffer, state, bindings)?;
        return Ok((
            vec![kind, buffer_name.clone()],
            vec![buffer_name.clone()],
            vec![buffer_name],
        ));
    }
    unreachable!("carry source payload classification must be exhaustive")
}

pub(super) fn encode_loop_carry_branch_source_args(
    source: &PreparedCarryBranchSource,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<(Vec<String>, Vec<String>, Vec<String>), String> {
    match source.view() {
        PreparedCarryBranchView::KeepCurrentValue => Ok((vec!["keep".to_owned()], vec![], vec![])),
        PreparedCarryBranchView::KeepPreviousValue => {
            Ok((vec!["keep_prev_carry".to_owned()], vec![], vec![]))
        }
        PreparedCarryBranchView::Source { op, source } => {
            encode_loop_carry_source_args(op, source, state, bindings)
        }
    }
}

pub(super) fn unsupported_loop_carry_branch_source_message(
    source: &PreparedCarryBranchSource,
) -> Option<String> {
    match source.view() {
        PreparedCarryBranchView::KeepCurrentValue
        | PreparedCarryBranchView::KeepPreviousValue
        | PreparedCarryBranchView::Source { .. } => None,
    }
}

pub(super) fn tail_recursive_prev_carry_binding(index: usize) -> String {
    format!("{TAIL_RECURSIVE_PREV_CARRY_BINDING_PREFIX}{index}")
}

fn parse_loop_carry_linear_shape<'a>(
    carry_name: &str,
    expr: &'a NirExpr,
) -> Option<(PreparedCarryLinearOp, &'a NirExpr)> {
    match expr {
        NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs,
            rhs,
        } => match lhs.as_ref() {
            NirExpr::Var(lhs_name) if lhs_name == carry_name => {
                Some((PreparedCarryLinearOp::Add, rhs.as_ref()))
            }
            _ => None,
        },
        NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs,
            rhs,
        } => match lhs.as_ref() {
            NirExpr::Var(lhs_name) if lhs_name == carry_name => {
                Some((PreparedCarryLinearOp::Mul, rhs.as_ref()))
            }
            _ => None,
        },
        _ => None,
    }
}

pub(super) fn parse_loop_carry_linear(
    carry_name: &str,
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<(PreparedCarryLinearOp, PreparedCarrySource)> {
    let normalized = inline_pure_helper_calls(expr, inlineable_pure_helpers);
    let (op, rhs) = parse_loop_carry_linear_shape(carry_name, &normalized)?;
    match rhs {
        NirExpr::Var(rhs_name) => parse_linear_var_source(op, rhs_name, binding_name, carries),
        _ => match op {
            PreparedCarryLinearOp::Mul => {
                parse_state_plus_invariant_scaled_additive_carry_source(rhs, binding_name, carries)
                    .or_else(|| {
                        parse_state_scaled_additive_carry_source(rhs, binding_name, carries)
                    })
                    .or_else(|| parse_scaled_additive_carry_source(rhs, binding_name, carries))
                    .or_else(|| parse_additive_carry_source(rhs, binding_name, carries))
                    .or_else(|| {
                        parse_prepared_fixed_read_carry_source(rhs, binding_name, carries)
                            .map(PreparedCarrySource::FixedRead)
                    })
                    .or_else(|| {
                        parse_prepared_dynamic_read_carry_source(rhs, binding_name, carries)
                    })
                    .map(|source| (op, source))
            }
            PreparedCarryLinearOp::Add => {
                parse_prepared_fixed_read_carry_source(rhs, binding_name, carries)
                    .map(PreparedCarrySource::FixedRead)
                    .or_else(|| {
                        parse_prepared_dynamic_read_carry_source(rhs, binding_name, carries)
                    })
                    .map(|source| (op, source))
            }
        },
    }
}

fn parse_loop_carry_keep_source(
    carry_name: &str,
    expr: &NirExpr,
    carries: &[PreparedCarryUpdate],
) -> Option<PreparedCarryBranchSource> {
    match expr {
        NirExpr::Var(name) if name == carry_name => Some(PreparedCarryBranchSource::keep()),
        NirExpr::Var(name) if *name == tail_recursive_prev_carry_binding(carries.len()) => {
            Some(PreparedCarryBranchSource::keep_previous_value())
        }
        _ if matches!(
            parse_loop_carry_linear_shape(carry_name, expr),
            Some((PreparedCarryLinearOp::Add, NirExpr::Int(0)))
        ) =>
        {
            Some(PreparedCarryBranchSource::keep())
        }
        _ => None,
    }
}

pub(super) fn parse_loop_carry_branch_source(
    carry_name: &str,
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedCarryBranchSource> {
    let normalized = inline_pure_helper_calls(expr, inlineable_pure_helpers);
    parse_loop_carry_keep_source(carry_name, &normalized, carries).or_else(|| {
        parse_loop_carry_linear(
            carry_name,
            &normalized,
            binding_name,
            carries,
            inlineable_pure_helpers,
        )
        .map(|(op, source)| PreparedCarryBranchSource::from_linear_source(op, source))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readable_carry_candidate_recognizes_fixed_load_value() {
        let expr = NirExpr::LoadValue(Box::new(NirExpr::Var("head".to_owned())));
        let candidate = parse_prepared_readable_carry_source_candidate(&expr, "current", &[])
            .expect("expected readable carry candidate");
        assert_eq!(candidate.family_name(), "fixed_read");
        assert!(matches!(
            candidate.fixed_read(),
            Some(PreparedFixedReadCarrySource::Value(_))
        ));
    }

    #[test]
    fn readable_carry_candidate_recognizes_fixed_load_at() {
        let expr = NirExpr::LoadAt {
            buffer: Box::new(NirExpr::Var("buffer".to_owned())),
            index: Box::new(NirExpr::Int(0)),
        };
        let candidate = parse_prepared_readable_carry_source_candidate(&expr, "current", &[])
            .expect("expected readable carry candidate");
        assert_eq!(candidate.family_name(), "fixed_read");
        assert!(matches!(
            candidate.fixed_read(),
            Some(PreparedFixedReadCarrySource::At { .. })
        ));
    }

    #[test]
    fn readable_carry_candidate_recognizes_dynamic_index_load_at_separately() {
        let expr = NirExpr::LoadAt {
            buffer: Box::new(NirExpr::Var("buffer".to_owned())),
            index: Box::new(NirExpr::Var("current".to_owned())),
        };
        let candidate = parse_prepared_readable_carry_source_candidate(&expr, "current", &[])
            .expect("expected readable carry candidate");
        assert_eq!(candidate.family_name(), "dynamic_index_at");
        assert!(candidate.fixed_read().is_none());
    }

    #[test]
    fn readable_carry_candidate_recognizes_load_next_traversal_separately() {
        let expr = NirExpr::LoadNext(Box::new(NirExpr::Var("head_ref".to_owned())));
        let candidate = parse_prepared_readable_carry_source_candidate(&expr, "current", &[])
            .expect("expected readable carry candidate");
        assert_eq!(candidate.family_name(), "traversal_next");
        assert!(candidate.fixed_read().is_none());
    }

    #[test]
    fn parse_prepared_dynamic_read_carry_source_accepts_current_index_driver() {
        let expr = NirExpr::LoadAt {
            buffer: Box::new(NirExpr::Var("buffer".to_owned())),
            index: Box::new(NirExpr::Var("current".to_owned())),
        };
        let source = parse_prepared_dynamic_read_carry_source(&expr, "current", &[])
            .expect("expected dynamic read carry source");
        assert_eq!(
            source.contract_kind(PreparedCarryLinearOp::Add),
            "add_read_at_dynamic_current"
        );
    }

    #[test]
    fn parse_prepared_dynamic_read_carry_source_accepts_prior_carry_index_driver() {
        let expr = NirExpr::LoadAt {
            buffer: Box::new(NirExpr::Var("buffer".to_owned())),
            index: Box::new(NirExpr::Var("slot".to_owned())),
        };
        let carries = vec![PreparedCarryUpdate {
            binding_name: "slot".to_owned(),
            kind: PreparedCarryUpdateKind::Linear {
                op: PreparedCarryLinearOp::Add,
                source: PreparedCarrySource::Current,
            },
        }];
        let source = parse_prepared_dynamic_read_carry_source(&expr, "current", &carries)
            .expect("expected dynamic read carry source");
        assert_eq!(
            source.contract_kind(PreparedCarryLinearOp::Add),
            "add_read_at_dynamic_carry0"
        );
    }

    #[test]
    fn parse_prepared_dynamic_read_carry_source_rejects_non_direct_index_expr() {
        let expr = NirExpr::LoadAt {
            buffer: Box::new(NirExpr::Var("buffer".to_owned())),
            index: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Var("current".to_owned())),
                rhs: Box::new(NirExpr::Int(1)),
            }),
        };
        assert!(parse_prepared_dynamic_read_carry_source(&expr, "current", &[]).is_none());
    }

    #[test]
    fn dynamic_read_contract_kind_supports_prev_current_and_prev_carry_index_drivers() {
        let prev_current = PreparedCarrySource::DynamicReadAt {
            buffer: NirExpr::Var("buffer".to_owned()),
            index_source: Box::new(PreparedCarrySource::PreviousCurrent),
        };
        let prev_carry = PreparedCarrySource::DynamicReadAt {
            buffer: NirExpr::Var("buffer".to_owned()),
            index_source: Box::new(PreparedCarrySource::PreviousCarry(0)),
        };
        assert_eq!(
            prev_current.contract_kind(PreparedCarryLinearOp::Add),
            "add_read_at_dynamic_prev_current"
        );
        assert_eq!(
            prev_carry.contract_kind(PreparedCarryLinearOp::Add),
            "add_read_at_dynamic_prev_carry0"
        );
    }

    #[test]
    fn parse_loop_carry_linear_accepts_multiplicative_state_plus_invariant_source() {
        let expr = NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs: Box::new(NirExpr::Var("acc".to_owned())),
            rhs: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Var("current".to_owned())),
                rhs: Box::new(NirExpr::Int(1)),
            }),
        };
        let (op, source) = parse_loop_carry_linear("acc", &expr, "current", &[], &BTreeMap::new())
            .expect("expected multiplicative additive carry source");
        assert!(matches!(op, PreparedCarryLinearOp::Mul));
        assert_eq!(
            source.contract_kind(PreparedCarryLinearOp::Mul),
            "mul_current_plus_invariant"
        );
    }

    #[test]
    fn parse_loop_carry_linear_accepts_multiplicative_multi_state_additive_source() {
        let carries = vec![PreparedCarryUpdate {
            binding_name: "slot".to_owned(),
            kind: PreparedCarryUpdateKind::Linear {
                op: PreparedCarryLinearOp::Add,
                source: PreparedCarrySource::Current,
            },
        }];
        let expr = NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs: Box::new(NirExpr::Var("acc".to_owned())),
            rhs: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Var("current".to_owned())),
                rhs: Box::new(NirExpr::Var("slot".to_owned())),
            }),
        };
        let (op, source) =
            parse_loop_carry_linear("acc", &expr, "current", &carries, &BTreeMap::new())
                .expect("expected multiplicative additive carry source");
        assert!(matches!(op, PreparedCarryLinearOp::Mul));
        assert_eq!(
            source.contract_kind(PreparedCarryLinearOp::Mul),
            "mul_current_plus_carry0"
        );
    }

    #[test]
    fn parse_loop_carry_linear_accepts_scaled_multiplicative_additive_source() {
        let expr = NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs: Box::new(NirExpr::Var("acc".to_owned())),
            rhs: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Mul,
                lhs: Box::new(NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Var("current".to_owned())),
                    rhs: Box::new(NirExpr::Int(1)),
                }),
                rhs: Box::new(NirExpr::Int(2)),
            }),
        };
        let (op, source) = parse_loop_carry_linear("acc", &expr, "current", &[], &BTreeMap::new())
            .expect("expected scaled multiplicative additive carry source");
        assert!(matches!(op, PreparedCarryLinearOp::Mul));
        assert_eq!(
            source.contract_kind(PreparedCarryLinearOp::Mul),
            "mul_scaled_current_plus_invariant"
        );
    }

    #[test]
    fn parse_loop_carry_linear_accepts_state_scaled_multiplicative_additive_source() {
        let carries = vec![PreparedCarryUpdate {
            binding_name: "sum".to_owned(),
            kind: PreparedCarryUpdateKind::Linear {
                op: PreparedCarryLinearOp::Add,
                source: PreparedCarrySource::Current,
            },
        }];
        let expr = NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs: Box::new(NirExpr::Var("acc".to_owned())),
            rhs: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Mul,
                lhs: Box::new(NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Var("current".to_owned())),
                    rhs: Box::new(NirExpr::Var("sum".to_owned())),
                }),
                rhs: Box::new(NirExpr::Var("current".to_owned())),
            }),
        };
        let (op, source) =
            parse_loop_carry_linear("acc", &expr, "current", &carries, &BTreeMap::new())
                .expect("expected state-scaled multiplicative additive carry source");
        assert!(matches!(op, PreparedCarryLinearOp::Mul));
        assert_eq!(
            source.contract_kind(PreparedCarryLinearOp::Mul),
            "mul_scaled_by_current_current_plus_carry0"
        );
    }

    #[test]
    fn parse_loop_carry_linear_accepts_state_plus_invariant_scaled_multiplicative_additive_source()
    {
        let carries = vec![PreparedCarryUpdate {
            binding_name: "sum".to_owned(),
            kind: PreparedCarryUpdateKind::Linear {
                op: PreparedCarryLinearOp::Add,
                source: PreparedCarrySource::Current,
            },
        }];
        let expr = NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs: Box::new(NirExpr::Var("acc".to_owned())),
            rhs: Box::new(NirExpr::Binary {
                op: NirBinaryOp::Mul,
                lhs: Box::new(NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Var("current".to_owned())),
                    rhs: Box::new(NirExpr::Var("sum".to_owned())),
                }),
                rhs: Box::new(NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Var("current".to_owned())),
                    rhs: Box::new(NirExpr::Int(1)),
                }),
            }),
        };
        let (op, source) =
            parse_loop_carry_linear("acc", &expr, "current", &carries, &BTreeMap::new()).expect(
                "expected state-plus-invariant scaled multiplicative additive carry source",
            );
        assert!(matches!(op, PreparedCarryLinearOp::Mul));
        assert_eq!(
            source.contract_kind(PreparedCarryLinearOp::Mul),
            "mul_scaled_by_current_plus_factor_invariant_current_plus_carry0"
        );
    }

    #[test]
    fn parses_loop_state_refs_for_current_prev_and_carry_slots() {
        let carries = vec![PreparedCarryUpdate {
            binding_name: "slot".to_owned(),
            kind: PreparedCarryUpdateKind::Linear {
                op: PreparedCarryLinearOp::Add,
                source: PreparedCarrySource::Current,
            },
        }];
        assert_eq!(
            parse_prepared_loop_state_ref_name("current", "current", &carries),
            Some(PreparedLoopStateRef::Current)
        );
        assert_eq!(
            parse_prepared_loop_state_ref_name("__tailrec_prev_current", "current", &carries),
            Some(PreparedLoopStateRef::PreviousCurrent)
        );
        assert_eq!(
            parse_prepared_loop_state_ref_name("__tailrec_prev_carry_0", "current", &carries),
            Some(PreparedLoopStateRef::PreviousCarry(0))
        );
        assert_eq!(
            parse_prepared_loop_state_ref_name("slot", "current", &carries),
            Some(PreparedLoopStateRef::Carry(0))
        );
    }

    #[test]
    fn parses_loop_state_refs_directly_from_carry_binding_names() {
        let carry_binding_names = vec!["slot".to_owned(), "acc".to_owned()];
        assert_eq!(
            parse_prepared_loop_state_ref_name_from_carry_names(
                "current",
                "current",
                &carry_binding_names,
            ),
            Some(PreparedLoopStateRef::Current)
        );
        assert_eq!(
            parse_prepared_loop_state_ref_name_from_carry_names(
                "slot",
                "current",
                &carry_binding_names,
            ),
            Some(PreparedLoopStateRef::Carry(0))
        );
        assert_eq!(
            parse_prepared_loop_state_ref_name_from_carry_names(
                "acc",
                "current",
                &carry_binding_names,
            ),
            Some(PreparedLoopStateRef::Carry(1))
        );
    }

    #[test]
    fn parses_loop_state_refs_directly_from_var_exprs() {
        let carries = vec![PreparedCarryUpdate {
            binding_name: "slot".to_owned(),
            kind: PreparedCarryUpdateKind::Linear {
                op: PreparedCarryLinearOp::Add,
                source: PreparedCarrySource::Current,
            },
        }];
        assert_eq!(
            parse_prepared_loop_state_ref_expr(
                &NirExpr::Var("__tailrec_prev_current".to_owned()),
                "current",
                &carries,
            ),
            Some(PreparedLoopStateRef::PreviousCurrent)
        );
        assert_eq!(
            parse_prepared_loop_state_ref_expr(
                &NirExpr::Var("slot".to_owned()),
                "current",
                &carries,
            ),
            Some(PreparedLoopStateRef::Carry(0))
        );
        assert_eq!(
            parse_prepared_loop_state_ref_expr(&NirExpr::Int(1), "current", &carries),
            None
        );
    }

    #[test]
    fn parses_loop_carry_keep_source_for_identity_and_add_zero_forms() {
        assert!(matches!(
            parse_loop_carry_keep_source("acc", &NirExpr::Var("acc".to_owned()), &[]),
            Some(source) if matches!(source.view(), PreparedCarryBranchView::KeepCurrentValue)
        ));
        assert!(matches!(
            parse_loop_carry_keep_source(
                "acc",
                &NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Var("acc".to_owned())),
                    rhs: Box::new(NirExpr::Int(0)),
                }
            , &[]),
            Some(source) if matches!(source.view(), PreparedCarryBranchView::KeepCurrentValue)
        ));
        assert!(parse_loop_carry_keep_source(
            "acc",
            &NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Var("acc".to_owned())),
                rhs: Box::new(NirExpr::Int(1)),
            },
            &[],
        )
        .is_none());
    }

    #[test]
    fn parses_loop_carry_keep_source_for_explicit_previous_value_placeholder() {
        let carries = vec![PreparedCarryUpdate {
            binding_name: "acc".to_owned(),
            kind: PreparedCarryUpdateKind::Linear {
                op: PreparedCarryLinearOp::Add,
                source: PreparedCarrySource::Current,
            },
        }];
        let previous_name = tail_recursive_prev_carry_binding(carries.len());
        assert!(matches!(
            parse_loop_carry_keep_source("slot", &NirExpr::Var(previous_name), &carries),
            Some(source) if matches!(source.view(), PreparedCarryBranchView::KeepPreviousValue)
        ));
    }

    #[test]
    fn prepared_carry_branch_source_helpers_round_trip_keep_and_source_variants() {
        let keep = PreparedCarryBranchSource::keep();
        assert!(matches!(
            keep.view(),
            PreparedCarryBranchView::KeepCurrentValue
        ));
        assert_eq!(encode_branch_view_name(keep.view()), "keep_current_value");
        assert!(matches!(
            keep.value_kind(),
            PreparedCarryBranchValueKind::KeepCurrentValue
        ));

        let keep_previous = PreparedCarryBranchSource::keep_previous_value();
        assert!(matches!(
            keep_previous.view(),
            PreparedCarryBranchView::KeepPreviousValue
        ));
        assert_eq!(
            encode_branch_view_name(keep_previous.view()),
            "keep_previous_value"
        );
        assert!(matches!(
            keep_previous.value_kind(),
            PreparedCarryBranchValueKind::KeepPreviousValue
        ));

        let source = PreparedCarryBranchSource::from_linear_source(
            PreparedCarryLinearOp::Add,
            PreparedCarrySource::Current,
        );
        assert!(matches!(
            source.value_kind(),
            PreparedCarryBranchValueKind::LinearSource {
                op: PreparedCarryLinearOp::Add,
                source: PreparedCarrySource::Current
            }
        ));
        assert!(matches!(
            source.view(),
            PreparedCarryBranchView::Source {
                op: PreparedCarryLinearOp::Add,
                source: PreparedCarrySource::Current
            }
        ));
    }

    #[test]
    fn previous_value_branch_view_is_constructible_and_distinct_from_current_keep() {
        let source = PreparedCarryBranchSource::keep_previous_value();
        assert!(matches!(
            source.value_kind(),
            PreparedCarryBranchValueKind::KeepPreviousValue
        ));
        assert!(matches!(
            source.view(),
            PreparedCarryBranchView::KeepPreviousValue
        ));
        assert!(
            parse_loop_carry_keep_source("acc", &NirExpr::Var("acc".to_owned()), &[]).is_some_and(
                |parsed| matches!(
                    parsed.value_kind(),
                    PreparedCarryBranchValueKind::KeepCurrentValue
                )
            )
        );
    }

    fn encode_branch_view_name(view: PreparedCarryBranchView<'_>) -> &'static str {
        match view {
            PreparedCarryBranchView::KeepCurrentValue => "keep_current_value",
            PreparedCarryBranchView::KeepPreviousValue => "keep_previous_value",
            PreparedCarryBranchView::Source { .. } => "source",
        }
    }

    #[test]
    fn keep_like_loop_carry_exprs_do_not_report_unsupported_diagnostics() {
        assert!(diagnose_unsupported_loop_carry_expr(
            "acc",
            &NirExpr::Var("acc".to_owned()),
            "current",
            &[],
            &BTreeMap::new(),
        )
        .is_none());
        assert!(diagnose_unsupported_loop_carry_expr(
            "acc",
            &NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs: Box::new(NirExpr::Var("acc".to_owned())),
                rhs: Box::new(NirExpr::Int(0)),
            },
            "current",
            &[],
            &BTreeMap::new(),
        )
        .is_none());
    }

    #[test]
    fn diagnose_unsupported_loop_carry_expr_reports_non_direct_dynamic_index_reads() {
        let expr = NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Var("acc".to_owned())),
            rhs: Box::new(NirExpr::LoadAt {
                buffer: Box::new(NirExpr::Var("buffer".to_owned())),
                index: Box::new(NirExpr::Binary {
                    op: NirBinaryOp::Add,
                    lhs: Box::new(NirExpr::Var("current".to_owned())),
                    rhs: Box::new(NirExpr::Int(1)),
                }),
            }),
        };
        let diagnostic =
            diagnose_unsupported_loop_carry_expr("acc", &expr, "current", &[], &BTreeMap::new())
                .expect("expected unsupported carry diagnostic");
        assert!(diagnostic.contains("dynamic `load_at(buffer, index)` reads currently support only direct loop-state index drivers"));
    }

    #[test]
    fn diagnose_unsupported_loop_carry_expr_reports_traversal_next_reads() {
        let expr = NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(NirExpr::Var("acc".to_owned())),
            rhs: Box::new(NirExpr::LoadNext(Box::new(NirExpr::Var(
                "head_ref".to_owned(),
            )))),
        };
        let diagnostic =
            diagnose_unsupported_loop_carry_expr("acc", &expr, "current", &[], &BTreeMap::new())
                .expect("expected unsupported carry diagnostic");
        assert!(diagnostic.contains("`load_next(...)` traversal reads"));
    }
}
