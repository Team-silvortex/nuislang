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

pub(super) fn render_loop_carry_kind(
    op: PreparedCarryLinearOp,
    source: PreparedCarrySource,
) -> String {
    match (op, source) {
        (PreparedCarryLinearOp::Add, PreparedCarrySource::Current) => "add_current".to_owned(),
        (PreparedCarryLinearOp::Add, PreparedCarrySource::PreviousCurrent) => {
            "add_prev_current".to_owned()
        }
        (PreparedCarryLinearOp::Add, PreparedCarrySource::PreviousCarry(index)) => {
            format!("add_prev_carry{index}")
        }
        (PreparedCarryLinearOp::Add, PreparedCarrySource::Carry(index)) => {
            format!("add_carry{index}")
        }
        (PreparedCarryLinearOp::Mul, PreparedCarrySource::Current) => "mul_current".to_owned(),
        (PreparedCarryLinearOp::Mul, PreparedCarrySource::PreviousCurrent) => {
            "mul_prev_current".to_owned()
        }
        (PreparedCarryLinearOp::Mul, PreparedCarrySource::PreviousCarry(index)) => {
            format!("mul_prev_carry{index}")
        }
        (PreparedCarryLinearOp::Mul, PreparedCarrySource::Carry(index)) => {
            format!("mul_carry{index}")
        }
    }
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
                if rhs_name == binding_name {
                    Some((PreparedCarryLinearOp::Add, PreparedCarrySource::Current))
                } else if rhs_name == TAIL_RECURSIVE_PREV_CURRENT_BINDING {
                    Some((
                        PreparedCarryLinearOp::Add,
                        PreparedCarrySource::PreviousCurrent,
                    ))
                } else if let Some(index) =
                    rhs_name.strip_prefix(TAIL_RECURSIVE_PREV_CARRY_BINDING_PREFIX)
                {
                    index
                        .parse::<usize>()
                        .ok()
                        .map(PreparedCarrySource::PreviousCarry)
                        .map(|source| (PreparedCarryLinearOp::Add, source))
                } else {
                    find_prepared_carry_index(carries, rhs_name)
                        .map(PreparedCarrySource::Carry)
                        .map(|source| (PreparedCarryLinearOp::Add, source))
                }
            }
            _ => None,
        },
        NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs,
            rhs,
        } => match (lhs.as_ref(), rhs.as_ref()) {
            (NirExpr::Var(lhs_name), NirExpr::Var(rhs_name)) if lhs_name == carry_name => {
                if rhs_name == binding_name {
                    Some((PreparedCarryLinearOp::Mul, PreparedCarrySource::Current))
                } else if rhs_name == TAIL_RECURSIVE_PREV_CURRENT_BINDING {
                    Some((
                        PreparedCarryLinearOp::Mul,
                        PreparedCarrySource::PreviousCurrent,
                    ))
                } else if let Some(index) =
                    rhs_name.strip_prefix(TAIL_RECURSIVE_PREV_CARRY_BINDING_PREFIX)
                {
                    index
                        .parse::<usize>()
                        .ok()
                        .map(PreparedCarrySource::PreviousCarry)
                        .map(|source| (PreparedCarryLinearOp::Mul, source))
                } else {
                    find_prepared_carry_index(carries, rhs_name)
                        .map(PreparedCarrySource::Carry)
                        .map(|source| (PreparedCarryLinearOp::Mul, source))
                }
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
