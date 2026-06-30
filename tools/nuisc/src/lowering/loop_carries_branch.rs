use super::loop_carries_linear::{
    parse_additive_carry_source, parse_linear_var_source, parse_scaled_additive_carry_source,
    parse_state_plus_invariant_scaled_additive_carry_source,
    parse_state_scaled_additive_carry_source,
};
use super::loop_carries_readable::{
    parse_prepared_dynamic_read_carry_source, parse_prepared_fixed_read_carry_source,
};
use super::*;

pub(in crate::lowering) fn tail_recursive_prev_carry_binding(index: usize) -> String {
    format!("{TAIL_RECURSIVE_PREV_CARRY_BINDING_PREFIX}{index}")
}

pub(in crate::lowering) fn parse_loop_carry_linear_shape<'a>(
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

pub(in crate::lowering) fn parse_loop_carry_linear(
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

pub(in crate::lowering) fn parse_loop_carry_keep_source(
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

pub(in crate::lowering) fn parse_loop_carry_branch_source(
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
