use super::*;
use crate::lowering::loop_purity::normalize_pure_bool_test_expr;

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

fn find_prepared_carry_index(carries: &[PreparedCarryUpdate], name: &str) -> Option<usize> {
    carries.iter().position(|carry| carry.binding_name == name)
}

fn expr_contains_loop_variant_name(
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
) -> bool {
    match expr {
        NirExpr::Var(name) => {
            name == binding_name
                || name == TAIL_RECURSIVE_PREV_CURRENT_BINDING
                || name.starts_with(TAIL_RECURSIVE_PREV_CARRY_BINDING_PREFIX)
                || carries.iter().any(|carry| carry.binding_name == *name)
        }
        NirExpr::Await(inner)
        | NirExpr::Borrow(inner)
        | NirExpr::BorrowEnd(inner)
        | NirExpr::Move(inner)
        | NirExpr::CastI64ToI32(inner)
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
        | NirExpr::CpuSpawn { args, .. } => args
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

fn parse_linear_var_source(
    op: PreparedCarryLinearOp,
    rhs_name: &str,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
) -> Option<(PreparedCarryLinearOp, PreparedCarrySource)> {
    if rhs_name == binding_name {
        Some((op, PreparedCarrySource::Current))
    } else if rhs_name == TAIL_RECURSIVE_PREV_CURRENT_BINDING {
        Some((op, PreparedCarrySource::PreviousCurrent))
    } else if let Some(index) = rhs_name.strip_prefix(TAIL_RECURSIVE_PREV_CARRY_BINDING_PREFIX) {
        index
            .parse::<usize>()
            .ok()
            .map(PreparedCarrySource::PreviousCarry)
            .map(|source| (op, source))
    } else {
        find_prepared_carry_index(carries, rhs_name)
            .map(PreparedCarrySource::Carry)
            .map(|source| (op, source))
    }
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
    if !source.is_fixed_read() {
        return Ok((vec![kind], vec![], vec![]));
    }
    match source.fixed_read() {
        Some(fixed_read) => {
            let (dep_inputs, effect_inputs) =
                lower_prepared_fixed_read_carry_source_args(fixed_read, state, bindings)?;
            let mut args = vec![kind];
            args.extend(dep_inputs.iter().cloned());
            Ok((args, dep_inputs, effect_inputs))
        }
        None => unreachable!("fixed-read carries must expose a fixed-read payload"),
    }
}

pub(super) fn encode_loop_carry_branch_source_args(
    source: &PreparedCarryBranchSource,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<(Vec<String>, Vec<String>, Vec<String>), String> {
    if source.is_keep() {
        return Ok((vec!["keep".to_owned()], vec![], vec![]));
    }
    let (op, carry_source) = source
        .source_parts()
        .expect("non-keep branch carry sources must expose source parts");
    encode_loop_carry_source_args(op, carry_source, state, bindings)
}

pub(super) fn tail_recursive_prev_carry_binding(index: usize) -> String {
    format!("{TAIL_RECURSIVE_PREV_CARRY_BINDING_PREFIX}{index}")
}

pub(super) fn parse_loop_carry_linear(
    carry_name: &str,
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<(PreparedCarryLinearOp, PreparedCarrySource)> {
    let normalized = inline_pure_helper_calls(expr, inlineable_pure_helpers);
    match &normalized {
        NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs,
            rhs,
        } => match (lhs.as_ref(), rhs.as_ref()) {
            (NirExpr::Var(lhs_name), NirExpr::Var(rhs_name)) if lhs_name == carry_name => {
                parse_linear_var_source(PreparedCarryLinearOp::Add, rhs_name, binding_name, carries)
            }
            (NirExpr::Var(lhs_name), rhs) if lhs_name == carry_name => {
                parse_prepared_fixed_read_carry_source(rhs, binding_name, carries)
                    .map(PreparedCarrySource::FixedRead)
                    .map(|source| (PreparedCarryLinearOp::Add, source))
            }
            _ => None,
        },
        NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs,
            rhs,
        } => match (lhs.as_ref(), rhs.as_ref()) {
            (NirExpr::Var(lhs_name), NirExpr::Var(rhs_name)) if lhs_name == carry_name => {
                parse_linear_var_source(PreparedCarryLinearOp::Mul, rhs_name, binding_name, carries)
            }
            (NirExpr::Var(lhs_name), rhs) if lhs_name == carry_name => {
                parse_prepared_fixed_read_carry_source(rhs, binding_name, carries)
                    .map(PreparedCarrySource::FixedRead)
                    .map(|source| (PreparedCarryLinearOp::Mul, source))
            }
            _ => None,
        },
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
    match &normalized {
        NirExpr::Var(name) if name == carry_name => Some(PreparedCarryBranchSource::Keep),
        NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs,
            rhs,
        } => match (lhs.as_ref(), rhs.as_ref()) {
            (NirExpr::Var(lhs_name), NirExpr::Int(0)) if lhs_name == carry_name => {
                Some(PreparedCarryBranchSource::Keep)
            }
            _ => parse_loop_carry_linear(
                carry_name,
                &normalized,
                binding_name,
                carries,
                inlineable_pure_helpers,
            )
            .map(|(op, source)| PreparedCarryBranchSource::Source { op, source }),
        },
        _ => parse_loop_carry_linear(
            carry_name,
            &normalized,
            binding_name,
            carries,
            inlineable_pure_helpers,
        )
        .map(|(op, source)| PreparedCarryBranchSource::Source { op, source }),
    }
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
}

pub(super) fn parse_loop_carry_condition(
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedLoopCarryCondition> {
    let normalized =
        normalize_pure_bool_test_expr(inline_pure_helper_calls(expr, inlineable_pure_helpers));
    let (lhs, compare, rhs) = match &normalized {
        NirExpr::Binary {
            op: NirBinaryOp::Eq,
            lhs,
            rhs,
        } if is_terminal_branch_pure_expr(rhs, pure_helpers) => {
            (lhs.as_ref(), PreparedLoopCompare::Eq, rhs.as_ref().clone())
        }
        NirExpr::Binary {
            op: NirBinaryOp::Ne,
            lhs,
            rhs,
        } if is_terminal_branch_pure_expr(rhs, pure_helpers) => {
            (lhs.as_ref(), PreparedLoopCompare::Ne, rhs.as_ref().clone())
        }
        NirExpr::Binary {
            op: NirBinaryOp::Lt,
            lhs,
            rhs,
        } if is_terminal_branch_pure_expr(rhs, pure_helpers) => {
            (lhs.as_ref(), PreparedLoopCompare::Lt, rhs.as_ref().clone())
        }
        NirExpr::Binary {
            op: NirBinaryOp::Le,
            lhs,
            rhs,
        } if is_terminal_branch_pure_expr(rhs, pure_helpers) => {
            (lhs.as_ref(), PreparedLoopCompare::Le, rhs.as_ref().clone())
        }
        NirExpr::Binary {
            op: NirBinaryOp::Gt,
            lhs,
            rhs,
        } if is_terminal_branch_pure_expr(rhs, pure_helpers) => {
            (lhs.as_ref(), PreparedLoopCompare::Gt, rhs.as_ref().clone())
        }
        NirExpr::Binary {
            op: NirBinaryOp::Ge,
            lhs,
            rhs,
        } if is_terminal_branch_pure_expr(rhs, pure_helpers) => {
            (lhs.as_ref(), PreparedLoopCompare::Ge, rhs.as_ref().clone())
        }
        _ => return None,
    };
    let lhs = match lhs {
        NirExpr::Var(name) if name == binding_name => PreparedCarryCondSource::Current,
        NirExpr::Var(name) if name == TAIL_RECURSIVE_PREV_CURRENT_BINDING => {
            PreparedCarryCondSource::PreviousCurrent
        }
        NirExpr::Var(name) if name.starts_with(TAIL_RECURSIVE_PREV_CARRY_BINDING_PREFIX) => {
            PreparedCarryCondSource::PreviousCarry(
                name[TAIL_RECURSIVE_PREV_CARRY_BINDING_PREFIX.len()..]
                    .parse::<usize>()
                    .ok()?,
            )
        }
        NirExpr::Var(name) => {
            PreparedCarryCondSource::Carry(find_prepared_carry_index(carries, name)?)
        }
        _ => return None,
    };
    Some(PreparedLoopCarryCondition { lhs, compare, rhs })
}
