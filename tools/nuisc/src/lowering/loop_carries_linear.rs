use super::loop_carries_refs::{
    expr_is_loop_invariant, loop_state_ref_into_carry_source, parse_prepared_loop_state_ref_expr,
};
use super::*;

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

pub(in crate::lowering) fn parse_additive_carry_source(
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

pub(in crate::lowering) fn parse_scaled_additive_carry_source(
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

pub(in crate::lowering) fn parse_state_scaled_additive_carry_source(
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

pub(in crate::lowering) fn parse_state_plus_invariant_scaled_additive_carry_source(
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

pub(in crate::lowering) fn parse_linear_var_source(
    op: PreparedCarryLinearOp,
    rhs_name: &str,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
) -> Option<(PreparedCarryLinearOp, PreparedCarrySource)> {
    parse_loop_variant_source_name(rhs_name, binding_name, carries).map(|source| (op, source))
}
