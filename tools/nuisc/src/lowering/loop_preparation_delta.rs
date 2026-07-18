use super::*;

pub(in crate::lowering) fn parse_loop_carry_delta_branch_source(
    op: PreparedCarryLinearOp,
    expr: &NirExpr,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedCarryBranchSource> {
    #[derive(Default)]
    struct ParsedAdditiveSource {
        terms: Vec<PreparedLoopStateRef>,
        offset: Option<NirExpr>,
    }

    fn expr_contains_loop_variant_ref(
        expr: &NirExpr,
        binding_name: &str,
        carries: &[PreparedCarryUpdate],
    ) -> bool {
        match expr {
            NirExpr::Var(name) => {
                parse_prepared_loop_state_ref_name(name, binding_name, carries).is_some()
            }
            NirExpr::Binary { lhs, rhs, .. } => {
                expr_contains_loop_variant_ref(lhs, binding_name, carries)
                    || expr_contains_loop_variant_ref(rhs, binding_name, carries)
            }
            _ => false,
        }
    }

    fn combine_invariant_terms(terms: Vec<NirExpr>) -> Option<NirExpr> {
        let mut iter = terms.into_iter();
        let first = iter.next()?;
        Some(iter.fold(first, |lhs, rhs| NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }))
    }

    fn scale_invariant_expr(expr: NirExpr, factor: i64) -> NirExpr {
        match factor {
            0 => NirExpr::Int(0),
            1 => expr,
            _ => NirExpr::Binary {
                op: NirBinaryOp::Mul,
                lhs: Box::new(expr),
                rhs: Box::new(NirExpr::Int(factor)),
            },
        }
    }

    fn scale_additive_source(
        source: PreparedCarrySource,
        factor: NirExpr,
        binding_name: &str,
        carries: &[PreparedCarryUpdate],
    ) -> Option<PreparedCarrySource> {
        let factor_state = parse_prepared_loop_state_ref_expr(&factor, binding_name, carries);
        let factor_affine = parse_additive_source_for_factor(&factor, binding_name, carries);
        let factor_scaled_affine =
            parse_scaled_additive_source_for_factor(&factor, binding_name, carries);
        let factor_group_product =
            parse_factor_group_product_for_factor(&factor, binding_name, carries);
        let factor_group_product_times_invariant =
            parse_scaled_factor_group_product_for_factor(&factor, binding_name, carries);
        match source {
            PreparedCarrySource::AddStateList { terms, offset } => {
                if let Some(factor_state) = factor_state {
                    Some(PreparedCarrySource::ScaledStateListByState {
                        terms,
                        factor: factor_state,
                        offset,
                    })
                } else if let Some((factor_terms, factor_offset)) = factor_affine {
                    match (factor_terms.as_slice(), factor_offset) {
                        ([factor_state], Some(factor_offset)) => {
                            Some(PreparedCarrySource::ScaledStateListByStatePlusInvariant {
                                terms,
                                factor: *factor_state,
                                factor_offset,
                                offset,
                            })
                        }
                        (factor_terms, factor_offset) => {
                            Some(PreparedCarrySource::ScaledStateListByFactorStateList {
                                terms,
                                factor_terms: factor_terms.to_vec(),
                                factor_offset,
                                offset,
                            })
                        }
                    }
                } else if let Some((factor_terms, factor_scale, factor_offset)) =
                    factor_scaled_affine
                {
                    Some(
                        PreparedCarrySource::ScaledStateListByFactorStateListTimesInvariant {
                            terms,
                            factor_terms,
                            factor_scale,
                            factor_offset,
                            offset,
                        },
                    )
                } else if let Some((
                    lhs_factor_terms,
                    lhs_factor_offset,
                    rhs_factor_terms,
                    rhs_factor_offset,
                )) = factor_group_product
                {
                    Some(PreparedCarrySource::ScaledStateListByFactorGroupProduct {
                        terms,
                        lhs_factor_terms,
                        lhs_factor_offset,
                        rhs_factor_terms,
                        rhs_factor_offset,
                        offset,
                    })
                } else if let Some((
                    lhs_factor_terms,
                    lhs_factor_offset,
                    rhs_factor_terms,
                    rhs_factor_offset,
                    factor_scale,
                )) = factor_group_product_times_invariant
                {
                    Some(
                        PreparedCarrySource::ScaledStateListByFactorGroupProductTimesInvariant {
                            terms,
                            lhs_factor_terms,
                            lhs_factor_offset,
                            rhs_factor_terms,
                            rhs_factor_offset,
                            factor_scale,
                            offset,
                        },
                    )
                } else {
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
            }
            PreparedCarrySource::Current
            | PreparedCarrySource::PreviousCurrent
            | PreparedCarrySource::PreviousCarry(_)
            | PreparedCarrySource::Carry(_)
            | PreparedCarrySource::ScaledStateList { .. }
            | PreparedCarrySource::ScaledStateListByState { .. }
            | PreparedCarrySource::ScaledStateListByStatePlusInvariant { .. }
            | PreparedCarrySource::ScaledStateListByFactorStateList { .. }
            | PreparedCarrySource::ScaledStateListByFactorStateListTimesInvariant { .. }
            | PreparedCarrySource::ScaledStateListByFactorGroupProduct { .. }
            | PreparedCarrySource::ScaledStateListByFactorGroupProductTimesInvariant { .. }
            | PreparedCarrySource::InvariantExpr(_)
            | PreparedCarrySource::AddInvariant { .. }
            | PreparedCarrySource::FixedRead(_)
            | PreparedCarrySource::DynamicReadAt { .. } => None,
        }
    }

    fn parse_additive_source_for_factor(
        expr: &NirExpr,
        binding_name: &str,
        carries: &[PreparedCarryUpdate],
    ) -> Option<(Vec<PreparedLoopStateRef>, Option<NirExpr>)> {
        fn parse_inner(
            expr: &NirExpr,
            binding_name: &str,
            carries: &[PreparedCarryUpdate],
            expr_contains_loop_variant_ref: &impl Fn(&NirExpr, &str, &[PreparedCarryUpdate]) -> bool,
        ) -> Option<ParsedAdditiveSource> {
            if let Some(state_ref) = parse_prepared_loop_state_ref_expr(expr, binding_name, carries)
            {
                return Some(ParsedAdditiveSource {
                    terms: vec![state_ref],
                    offset: None,
                });
            }
            if is_terminal_branch_pure_expr(expr, &BTreeSet::new())
                && !expr_contains_loop_variant_ref(expr, binding_name, carries)
            {
                return Some(ParsedAdditiveSource {
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
                    let lhs =
                        parse_inner(lhs, binding_name, carries, expr_contains_loop_variant_ref)?;
                    let rhs =
                        parse_inner(rhs, binding_name, carries, expr_contains_loop_variant_ref)?;
                    let mut terms = lhs.terms;
                    terms.extend(rhs.terms);
                    let offset = combine_invariant_terms(
                        lhs.offset.into_iter().chain(rhs.offset).collect::<Vec<_>>(),
                    );
                    Some(ParsedAdditiveSource { terms, offset })
                }
                _ => None,
            }
        }

        let parsed = parse_inner(expr, binding_name, carries, &expr_contains_loop_variant_ref)?;
        match (parsed.terms.is_empty(), parsed.offset.is_some()) {
            (false, true) | (false, false) => Some((parsed.terms, parsed.offset)),
            _ => None,
        }
    }

    fn parse_scaled_additive_source_for_factor(
        expr: &NirExpr,
        binding_name: &str,
        carries: &[PreparedCarryUpdate],
    ) -> Option<(Vec<PreparedLoopStateRef>, NirExpr, Option<NirExpr>)> {
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
                && !expr_contains_loop_variant_ref(expr, binding_name, carries)
        };
        if let Some((factor_terms, factor_offset)) =
            parse_additive_source_for_factor(lhs, binding_name, carries)
        {
            if invariant(rhs) {
                return Some((factor_terms, (**rhs).clone(), factor_offset));
            }
        }
        if let Some((factor_terms, factor_offset)) =
            parse_additive_source_for_factor(rhs, binding_name, carries)
        {
            if invariant(lhs) {
                return Some((factor_terms, (**lhs).clone(), factor_offset));
            }
        }
        None
    }

    fn parse_factor_group_product_for_factor(
        expr: &NirExpr,
        binding_name: &str,
        carries: &[PreparedCarryUpdate],
    ) -> Option<PreparedFactorGroupProduct> {
        let NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs,
            rhs,
        } = expr
        else {
            return None;
        };
        let lhs_group = parse_additive_source_for_factor(lhs, binding_name, carries)?;
        let rhs_group = parse_additive_source_for_factor(rhs, binding_name, carries)?;
        Some((lhs_group.0, lhs_group.1, rhs_group.0, rhs_group.1))
    }

    fn parse_scaled_factor_group_product_for_factor(
        expr: &NirExpr,
        binding_name: &str,
        carries: &[PreparedCarryUpdate],
    ) -> Option<PreparedScaledFactorGroupProduct> {
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
                && !expr_contains_loop_variant_ref(expr, binding_name, carries)
        };
        if let Some((lhs_terms, lhs_offset, rhs_terms, rhs_offset)) =
            parse_factor_group_product_for_factor(lhs, binding_name, carries)
        {
            if invariant(rhs) {
                return Some((
                    lhs_terms,
                    lhs_offset,
                    rhs_terms,
                    rhs_offset,
                    (**rhs).clone(),
                ));
            }
        }
        if let Some((lhs_terms, lhs_offset, rhs_terms, rhs_offset)) =
            parse_factor_group_product_for_factor(rhs, binding_name, carries)
        {
            if invariant(lhs) {
                return Some((
                    lhs_terms,
                    lhs_offset,
                    rhs_terms,
                    rhs_offset,
                    (**lhs).clone(),
                ));
            }
        }
        None
    }

    let normalized = inline_pure_helper_calls(expr, inlineable_pure_helpers);
    let parse_additive_source = |expr: &NirExpr| -> Option<PreparedCarrySource> {
        fn parse_inner(
            expr: &NirExpr,
            binding_name: &str,
            carries: &[PreparedCarryUpdate],
            expr_contains_loop_variant_ref: &impl Fn(&NirExpr, &str, &[PreparedCarryUpdate]) -> bool,
        ) -> Option<ParsedAdditiveSource> {
            if let Some(state_ref) = parse_prepared_loop_state_ref_expr(expr, binding_name, carries)
            {
                return Some(ParsedAdditiveSource {
                    terms: vec![state_ref],
                    offset: None,
                });
            }
            if is_terminal_branch_pure_expr(expr, &BTreeSet::new())
                && !expr_contains_loop_variant_ref(expr, binding_name, carries)
            {
                return Some(ParsedAdditiveSource {
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
                    let lhs =
                        parse_inner(lhs, binding_name, carries, expr_contains_loop_variant_ref)?;
                    let rhs =
                        parse_inner(rhs, binding_name, carries, expr_contains_loop_variant_ref)?;
                    let mut terms = lhs.terms;
                    terms.extend(rhs.terms);
                    let offset = combine_invariant_terms(
                        lhs.offset.into_iter().chain(rhs.offset).collect::<Vec<_>>(),
                    );
                    Some(ParsedAdditiveSource { terms, offset })
                }
                NirExpr::Binary {
                    op: NirBinaryOp::Mul,
                    lhs,
                    rhs,
                } => {
                    let (base, factor) = match (lhs.as_ref(), rhs.as_ref()) {
                        (base, NirExpr::Int(factor)) if *factor >= 0 => (base, *factor),
                        (NirExpr::Int(factor), base) if *factor >= 0 => (base, *factor),
                        _ => return None,
                    };
                    let parsed =
                        parse_inner(base, binding_name, carries, expr_contains_loop_variant_ref)?;
                    let terms = parsed
                        .terms
                        .iter()
                        .flat_map(|term| std::iter::repeat_n(*term, factor as usize))
                        .collect::<Vec<_>>();
                    let offset = parsed
                        .offset
                        .map(|offset| scale_invariant_expr(offset, factor));
                    Some(ParsedAdditiveSource { terms, offset })
                }
                _ => None,
            }
        }

        let parsed = parse_inner(expr, binding_name, carries, &expr_contains_loop_variant_ref)?;
        if parsed.terms.len() + usize::from(parsed.offset.is_some()) <= 1 {
            return None;
        }
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
    };
    if matches!(op, PreparedCarryLinearOp::Add)
        && is_terminal_branch_pure_expr(&normalized, &BTreeSet::new())
        && !expr_contains_loop_variant_ref(&normalized, binding_name, carries)
    {
        return Some(PreparedCarryBranchSource::from_linear_source(
            op,
            PreparedCarrySource::InvariantExpr(normalized),
        ));
    }
    if matches!(op, PreparedCarryLinearOp::Add) && matches!(normalized, NirExpr::Int(0)) {
        return Some(PreparedCarryBranchSource::keep());
    }
    if matches!(op, PreparedCarryLinearOp::Add) {
        if let NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs,
            rhs,
        } = &normalized
        {
            let factor_supported = |expr: &NirExpr| {
                parse_prepared_loop_state_ref_expr(expr, binding_name, carries).is_some()
                    || (is_terminal_branch_pure_expr(expr, &BTreeSet::new())
                        && !expr_contains_loop_variant_ref(expr, binding_name, carries))
                    || parse_additive_source_for_factor(expr, binding_name, carries).is_some()
                    || parse_scaled_additive_source_for_factor(expr, binding_name, carries)
                        .is_some()
                    || parse_factor_group_product_for_factor(expr, binding_name, carries).is_some()
                    || parse_scaled_factor_group_product_for_factor(expr, binding_name, carries)
                        .is_some()
            };
            let scaled = if let Some(base) = parse_additive_source(lhs) {
                if factor_supported(rhs) {
                    scale_additive_source(base, (**rhs).clone(), binding_name, carries)
                } else {
                    None
                }
            } else if let Some(base) = parse_additive_source(rhs) {
                if factor_supported(lhs) {
                    scale_additive_source(base, (**lhs).clone(), binding_name, carries)
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(source) = scaled {
                return Some(PreparedCarryBranchSource::from_linear_source(op, source));
            }
        }
        if let Some(source) = parse_additive_source(&normalized) {
            return Some(PreparedCarryBranchSource::from_linear_source(op, source));
        }
        if let NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs,
            rhs,
        } = &normalized
        {
            if let Some(base_ref) = parse_prepared_loop_state_ref_expr(lhs, binding_name, carries) {
                if is_terminal_branch_pure_expr(rhs, &BTreeSet::new()) {
                    return Some(PreparedCarryBranchSource::from_linear_source(
                        op,
                        PreparedCarrySource::AddInvariant {
                            base: Box::new(loop_state_ref_into_carry_source(base_ref)),
                            offset: (**rhs).clone(),
                        },
                    ));
                }
            }
        }
    }
    if let Some(state_ref) = parse_prepared_loop_state_ref_expr(&normalized, binding_name, carries)
    {
        return Some(PreparedCarryBranchSource::from_linear_source(
            op,
            loop_state_ref_into_carry_source(state_ref),
        ));
    }
    parse_prepared_fixed_read_carry_source(&normalized, binding_name, carries)
        .map(PreparedCarrySource::FixedRead)
        .or_else(|| parse_prepared_dynamic_read_carry_source(&normalized, binding_name, carries))
        .map(|source| PreparedCarryBranchSource::from_linear_source(op, source))
}

pub(super) fn parse_conditional_temp_driven_loop_carry_update(
    stmt: &NirStmt,
    binding_name: &str,
    carries: &[PreparedCarryUpdate],
    conditional_temps: &BTreeMap<String, PreparedConditionalTempBinding>,
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedCarryUpdate> {
    let (carry_name, carry_expr) = extract_pure_branch_binding(stmt, pure_helpers)?;
    let normalized = inline_pure_helper_calls(&carry_expr, inlineable_pure_helpers);
    let (op, rhs_name) = match &normalized {
        NirExpr::Binary {
            op: NirBinaryOp::Add,
            lhs,
            rhs,
        } => match (lhs.as_ref(), rhs.as_ref()) {
            (NirExpr::Var(lhs_name), NirExpr::Var(rhs_name)) if lhs_name == &carry_name => {
                (PreparedCarryLinearOp::Add, rhs_name)
            }
            _ => return None,
        },
        NirExpr::Binary {
            op: NirBinaryOp::Mul,
            lhs,
            rhs,
        } => match (lhs.as_ref(), rhs.as_ref()) {
            (NirExpr::Var(lhs_name), NirExpr::Var(rhs_name)) if lhs_name == &carry_name => {
                (PreparedCarryLinearOp::Mul, rhs_name)
            }
            _ => return None,
        },
        _ => return None,
    };
    let temp = conditional_temps.get(rhs_name)?;
    Some(PreparedCarryUpdate {
        binding_name: carry_name,
        kind: PreparedCarryUpdateKind::Conditional {
            condition: temp.condition.clone(),
            then_source: Box::new(parse_loop_carry_delta_branch_source(
                op,
                &temp.then_expr,
                binding_name,
                carries,
                inlineable_pure_helpers,
            )?),
            else_source: Box::new(parse_loop_carry_delta_branch_source(
                op,
                &temp.else_expr,
                binding_name,
                carries,
                inlineable_pure_helpers,
            )?),
        },
    })
}
