use super::*;

#[derive(Clone)]
enum DynamicPatternBranch {
    Matched {
        payload: NirExpr,
        type_args: Vec<NirTypeRef>,
    },
    Exit(NirExpr),
}

enum DynamicPatternUpdate {
    Matched {
        payload: NirExpr,
        type_args: Vec<NirTypeRef>,
    },
    Conditional {
        condition: NirExpr,
        then_branch: DynamicPatternBranch,
        else_branch: DynamicPatternBranch,
    },
}

pub(super) struct DynamicPatternPlan {
    binding_name: String,
    matched_variant: String,
    payload_binding_name: String,
    payload_field: String,
    matched_type_args: Vec<NirTypeRef>,
    initial_condition: NirExpr,
    initial_payload: NirExpr,
    update: DynamicPatternUpdate,
}

pub(super) fn prepare_terminal_pattern_transition(
    gate_condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
) -> Option<(PreparedTerminalPatternTransition, Vec<NirStmt>)> {
    let NirExpr::VariantIs { base, variant } = gate_condition else {
        return None;
    };
    let NirExpr::Var(binding_name) = base.as_ref() else {
        return None;
    };

    let mut transition = None;
    for (index, stmt) in body.iter().enumerate() {
        let (name, value) = match stmt {
            NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => (name, value),
            _ => continue,
        };
        if name != binding_name {
            continue;
        }
        if transition.is_some() {
            return None;
        }
        let NirExpr::StructLiteral { type_name, .. } = value else {
            return None;
        };
        if type_name == variant || variant_parent(type_name) != variant_parent(variant) {
            return None;
        }
        if !is_terminal_branch_pure_expr(value, pure_helpers) {
            return None;
        }
        transition = Some((index, value.clone()));
    }

    let (transition_index, value) = transition?;
    let rebound_names = body
        .iter()
        .filter_map(|stmt| match stmt {
            NirStmt::Let { name, .. } | NirStmt::Const { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    if expr_references_names(&value, &rebound_names) {
        return None;
    }

    let prepared_body = body
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != transition_index)
        .map(|(_, stmt)| stmt.clone())
        .collect();
    Some((
        PreparedTerminalPatternTransition {
            binding_name: binding_name.clone(),
            value,
        },
        prepared_body,
    ))
}

fn variant_parent(name: &str) -> Option<&str> {
    name.rsplit_once('.').map(|(parent, _)| parent)
}

pub(super) fn prepare_dynamic_pattern_plan(
    gate_condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
) -> Option<(DynamicPatternPlan, Vec<NirStmt>)> {
    let NirExpr::VariantIs { base, variant } = gate_condition else {
        return None;
    };
    let NirExpr::Var(binding_name) = base.as_ref() else {
        return None;
    };
    let (payload_index, payload_binding_name, payload_field, initial_payload) = body
        .iter()
        .enumerate()
        .find_map(|(index, stmt)| match stmt {
            NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => {
                let NirExpr::VariantFieldAccess {
                    base,
                    variant: field_variant,
                    field,
                } = value
                else {
                    return None;
                };
                if field_variant != variant
                    || !matches!(base.as_ref(), NirExpr::Var(name) if name == binding_name)
                {
                    return None;
                }
                Some((index, name.clone(), field.clone(), value.clone()))
            }
            _ => None,
        })?;

    let mut update = None;
    let mut transition_index = None;
    for (index, stmt) in body.iter().enumerate() {
        let candidate =
            parse_dynamic_pattern_update(stmt, binding_name, variant, &payload_field, pure_helpers);
        if candidate.is_some() {
            if update.is_some() {
                return None;
            }
            update = candidate;
            transition_index = Some(index);
        }
    }
    let update = update?;
    let matched_type_args = dynamic_pattern_update_type_args(&update)?.to_vec();
    let transition_index = transition_index?;
    let previous_payload = NirExpr::Var(tail_recursive_prev_carry_binding(1));
    let substitutions = vec![(payload_binding_name.clone(), previous_payload)];
    let prepared_body = body
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != payload_index && *index != transition_index)
        .map(|(_, stmt)| substitute_stmt_bindings(stmt, &substitutions))
        .collect();

    Some((
        DynamicPatternPlan {
            binding_name: binding_name.clone(),
            matched_variant: variant.clone(),
            payload_binding_name,
            payload_field,
            matched_type_args,
            initial_condition: gate_condition.clone(),
            initial_payload,
            update,
        },
        prepared_body,
    ))
}

fn parse_dynamic_pattern_update(
    stmt: &NirStmt,
    binding_name: &str,
    matched_variant: &str,
    payload_field: &str,
    pure_helpers: &BTreeSet<String>,
) -> Option<DynamicPatternUpdate> {
    if let Some(DynamicPatternBranch::Matched { payload, type_args }) = parse_dynamic_pattern_branch(
        stmt,
        binding_name,
        matched_variant,
        payload_field,
        pure_helpers,
    ) {
        return Some(DynamicPatternUpdate::Matched { payload, type_args });
    }
    let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = stmt
    else {
        return None;
    };
    let [then_stmt] = then_body.as_slice() else {
        return None;
    };
    let [else_stmt] = else_body.as_slice() else {
        return None;
    };
    if !is_terminal_branch_pure_expr(condition, pure_helpers) {
        return None;
    }
    let then_branch = parse_dynamic_pattern_branch(
        then_stmt,
        binding_name,
        matched_variant,
        payload_field,
        pure_helpers,
    )?;
    let else_branch = parse_dynamic_pattern_branch(
        else_stmt,
        binding_name,
        matched_variant,
        payload_field,
        pure_helpers,
    )?;
    if matches!(
        (&then_branch, &else_branch),
        (DynamicPatternBranch::Exit(_), DynamicPatternBranch::Exit(_))
    ) {
        return None;
    }
    Some(DynamicPatternUpdate::Conditional {
        condition: condition.clone(),
        then_branch,
        else_branch,
    })
}

fn dynamic_pattern_update_type_args(update: &DynamicPatternUpdate) -> Option<&[NirTypeRef]> {
    match update {
        DynamicPatternUpdate::Matched { type_args, .. } => Some(type_args),
        DynamicPatternUpdate::Conditional {
            then_branch,
            else_branch,
            ..
        } => [then_branch, else_branch]
            .into_iter()
            .find_map(|branch| match branch {
                DynamicPatternBranch::Matched { type_args, .. } => Some(type_args.as_slice()),
                DynamicPatternBranch::Exit(_) => None,
            }),
    }
}

fn parse_dynamic_pattern_branch(
    stmt: &NirStmt,
    binding_name: &str,
    matched_variant: &str,
    payload_field: &str,
    pure_helpers: &BTreeSet<String>,
) -> Option<DynamicPatternBranch> {
    let (name, value) = match stmt {
        NirStmt::Let { name, value, .. } | NirStmt::Const { name, value, .. } => (name, value),
        _ => return None,
    };
    if name != binding_name || !is_terminal_branch_pure_expr(value, pure_helpers) {
        return None;
    }
    let NirExpr::StructLiteral {
        type_name,
        type_args,
        fields,
    } = value
    else {
        return None;
    };
    if variant_parent(type_name) != variant_parent(matched_variant) {
        return None;
    }
    if type_name == matched_variant {
        let [(field, payload)] = fields.as_slice() else {
            return None;
        };
        if field != payload_field {
            return None;
        }
        Some(DynamicPatternBranch::Matched {
            payload: payload.clone(),
            type_args: type_args.clone(),
        })
    } else if fields.is_empty() {
        Some(DynamicPatternBranch::Exit(value.clone()))
    } else {
        None
    }
}

pub(super) fn prepare_dynamic_pattern_carries(
    plan: DynamicPatternPlan,
    loop_binding_name: &str,
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<(PreparedDynamicPatternTransition, Vec<PreparedCarryUpdate>)> {
    let active_carry_name = format!("__pattern_active_{}", plan.binding_name);
    let payload_carry_name = format!("__pattern_payload_{}", plan.binding_name);
    let placeholder = || PreparedCarryUpdateKind::Linear {
        op: PreparedCarryLinearOp::Add,
        source: Box::new(PreparedCarrySource::InvariantExpr(NirExpr::Int(0))),
    };
    let mut carries = vec![
        PreparedCarryUpdate {
            binding_name: active_carry_name.clone(),
            kind: placeholder(),
        },
        PreparedCarryUpdate {
            binding_name: payload_carry_name.clone(),
            kind: placeholder(),
        },
    ];
    let active_condition = PreparedLoopFlowCondition::Simple(PreparedLoopCarryCondition {
        lhs: PreparedCarryCondSource::PreviousCarry(0),
        compare: PreparedLoopCompare::Ne,
        rhs: NirExpr::Int(0),
    });

    let (condition, then_branch, else_branch, exit_value) = match plan.update {
        DynamicPatternUpdate::Matched { payload, .. } => (
            active_condition,
            DynamicPatternBranch::Matched {
                payload,
                type_args: plan.matched_type_args.clone(),
            },
            DynamicPatternBranch::Matched {
                payload: NirExpr::Var(plan.payload_binding_name.clone()),
                type_args: plan.matched_type_args.clone(),
            },
            None,
        ),
        DynamicPatternUpdate::Conditional {
            condition,
            then_branch,
            else_branch,
        } => {
            let rewritten = substitute_branch_binding(
                &condition,
                &plan.payload_binding_name,
                &NirExpr::Var(tail_recursive_prev_carry_binding(1)),
            );
            let condition = parse_loop_flow_condition(
                &rewritten,
                loop_binding_name,
                &carries,
                pure_helpers,
                inlineable_pure_helpers,
            )?;
            let exit_value =
                [&then_branch, &else_branch]
                    .into_iter()
                    .find_map(|branch| match branch {
                        DynamicPatternBranch::Exit(value) => Some(value.clone()),
                        DynamicPatternBranch::Matched { .. } => None,
                    });
            (condition, then_branch, else_branch, exit_value)
        }
    };

    let active_source = |branch: &DynamicPatternBranch| match branch {
        DynamicPatternBranch::Matched { .. } => PreparedCarryBranchSource::keep(),
        DynamicPatternBranch::Exit(_) => PreparedCarryBranchSource::from_linear_source(
            PreparedCarryLinearOp::Add,
            PreparedCarrySource::InvariantExpr(NirExpr::Int(-1)),
        ),
    };
    let payload_source = |branch: &DynamicPatternBranch| match branch {
        DynamicPatternBranch::Matched { payload, .. } => prepare_payload_branch_source(
            payload,
            &plan.payload_binding_name,
            &payload_carry_name,
            loop_binding_name,
            &carries,
            inlineable_pure_helpers,
        ),
        DynamicPatternBranch::Exit(_) => Some(PreparedCarryBranchSource::keep()),
    };
    let then_payload = payload_source(&then_branch)?;
    let else_payload = payload_source(&else_branch)?;
    carries[0].kind = PreparedCarryUpdateKind::Conditional {
        condition: condition.clone(),
        then_source: Box::new(active_source(&then_branch)),
        else_source: Box::new(active_source(&else_branch)),
    };
    carries[1].kind = PreparedCarryUpdateKind::Conditional {
        condition,
        then_source: Box::new(then_payload),
        else_source: Box::new(else_payload),
    };

    Some((
        PreparedDynamicPatternTransition {
            binding_name: plan.binding_name,
            matched_variant: plan.matched_variant,
            matched_type_args: plan.matched_type_args,
            payload_field: plan.payload_field,
            active_carry_name,
            payload_carry_name,
            initial_condition: plan.initial_condition,
            initial_payload: plan.initial_payload,
            exit_value,
        },
        carries,
    ))
}

fn prepare_payload_branch_source(
    payload: &NirExpr,
    payload_binding_name: &str,
    payload_carry_name: &str,
    loop_binding_name: &str,
    carries: &[PreparedCarryUpdate],
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedCarryBranchSource> {
    let renamed = substitute_branch_binding(
        payload,
        payload_binding_name,
        &NirExpr::Var(payload_carry_name.to_owned()),
    );
    let normalized = match renamed {
        NirExpr::Binary {
            op: NirBinaryOp::Sub,
            lhs,
            rhs,
        } => {
            let NirExpr::Int(value) = *rhs else {
                return None;
            };
            NirExpr::Binary {
                op: NirBinaryOp::Add,
                lhs,
                rhs: Box::new(NirExpr::Int(value.checked_neg()?)),
            }
        }
        other => other,
    };
    parse_loop_carry_branch_source(
        payload_carry_name,
        &normalized,
        loop_binding_name,
        carries,
        inlineable_pure_helpers,
    )
}
