use super::loop_carries_branch::{parse_loop_carry_keep_source, parse_loop_carry_linear_shape};
use super::loop_carries_refs::{
    expr_is_loop_invariant, loop_state_ref_into_carry_source, parse_prepared_loop_state_ref_expr,
};
use super::*;

pub(in crate::lowering) fn parse_prepared_readable_carry_source_candidate(
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

pub(in crate::lowering) fn parse_prepared_fixed_read_carry_source(
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

pub(in crate::lowering) fn parse_prepared_dynamic_read_carry_source(
    rhs: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
) -> Option<PreparedCarrySource> {
    fn direct_dynamic_index_driver(
        expr: &NirExpr,
        binding_name: &str,
        carries: &[PreparedCarryUpdate],
    ) -> Option<PreparedCarrySource> {
        if let Some(state_ref) = parse_prepared_loop_state_ref_expr(expr, binding_name, carries) {
            return Some(loop_state_ref_into_carry_source(state_ref));
        }

        let PreparedCarrySource::AddInvariant { base, offset } =
            parse_additive_carry_source(expr, binding_name, carries)?
        else {
            return None;
        };
        if offset != NirExpr::Int(0) {
            return None;
        }
        match *base {
            PreparedCarrySource::Current
            | PreparedCarrySource::PreviousCurrent
            | PreparedCarrySource::PreviousCarry(_)
            | PreparedCarrySource::Carry(_) => Some(*base),
            _ => None,
        }
    }

    let candidate = parse_prepared_readable_carry_source_candidate(rhs, binding_name, carries)?;
    match candidate {
        PreparedReadableCarrySourceCandidate::DynamicIndexAt { buffer, index } => {
            let index_source = direct_dynamic_index_driver(&index, binding_name, carries)?;
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

pub(in crate::lowering) fn diagnose_unsupported_loop_carry_expr(
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

pub(in crate::lowering) fn render_loop_carry_kind(
    op: PreparedCarryLinearOp,
    source: &PreparedCarrySource,
) -> String {
    source.contract_kind(op)
}

pub(in crate::lowering) fn lower_prepared_fixed_read_carry_source_args(
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

pub(in crate::lowering) fn encode_loop_carry_source_args(
    op: PreparedCarryLinearOp,
    source: &PreparedCarrySource,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<EncodedLoopArgs, String> {
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

pub(in crate::lowering) fn encode_loop_carry_branch_source_args(
    source: &PreparedCarryBranchSource,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<EncodedLoopArgs, String> {
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

pub(in crate::lowering) fn unsupported_loop_carry_branch_source_message(
    source: &PreparedCarryBranchSource,
) -> Option<String> {
    match source.view() {
        PreparedCarryBranchView::KeepCurrentValue
        | PreparedCarryBranchView::KeepPreviousValue
        | PreparedCarryBranchView::Source { .. } => None,
    }
}
