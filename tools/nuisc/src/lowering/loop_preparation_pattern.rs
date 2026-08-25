use super::*;

#[derive(Clone)]
enum DynamicPatternBranch {
    Matched {
        payloads: Vec<NirExpr>,
        type_args: Vec<NirTypeRef>,
    },
    Exit(NirExpr),
}

enum DynamicPatternUpdate {
    Matched {
        payloads: Vec<NirExpr>,
        type_args: Vec<NirTypeRef>,
    },
    Conditional {
        condition: NirExpr,
        then_branch: DynamicPatternBranch,
        else_branch: DynamicPatternBranch,
    },
}

struct DynamicPatternPayload {
    binding_name: String,
    field: String,
    initial: NirExpr,
}

pub(super) struct DynamicPatternPlan {
    binding_name: String,
    matched_variant: String,
    payloads: Vec<DynamicPatternPayload>,
    matched_type_args: Vec<NirTypeRef>,
    initial_condition: NirExpr,
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
    let payloads = body
        .iter()
        .enumerate()
        .map_while(|(index, stmt)| match stmt {
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
                Some((
                    index,
                    DynamicPatternPayload {
                        binding_name: name.clone(),
                        field: field.clone(),
                        initial: value.clone(),
                    },
                ))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    if payloads.is_empty() {
        return None;
    }
    let distinct_fields = payloads
        .iter()
        .map(|(_, payload)| payload.field.as_str())
        .collect::<BTreeSet<_>>();
    let distinct_bindings = payloads
        .iter()
        .map(|(_, payload)| payload.binding_name.as_str())
        .collect::<BTreeSet<_>>();
    if distinct_fields.len() != payloads.len() || distinct_bindings.len() != payloads.len() {
        return None;
    }
    let payload_indices = payloads
        .iter()
        .map(|(index, _)| *index)
        .collect::<BTreeSet<_>>();
    let payloads = payloads
        .into_iter()
        .map(|(_, payload)| payload)
        .collect::<Vec<_>>();

    let mut update = None;
    let mut transition_index = None;
    for (index, stmt) in body.iter().enumerate() {
        let candidate =
            parse_dynamic_pattern_update(stmt, binding_name, variant, &payloads, pure_helpers);
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
    let substitutions = payloads
        .iter()
        .enumerate()
        .map(|(index, payload)| {
            (
                payload.binding_name.clone(),
                NirExpr::Var(tail_recursive_prev_carry_binding(index + 1)),
            )
        })
        .collect::<Vec<_>>();
    let prepared_body = body
        .iter()
        .enumerate()
        .filter(|(index, _)| !payload_indices.contains(index) && *index != transition_index)
        .map(|(_, stmt)| substitute_stmt_bindings(stmt, &substitutions))
        .collect();

    Some((
        DynamicPatternPlan {
            binding_name: binding_name.clone(),
            matched_variant: variant.clone(),
            payloads,
            matched_type_args,
            initial_condition: gate_condition.clone(),
            update,
        },
        prepared_body,
    ))
}

fn parse_dynamic_pattern_update(
    stmt: &NirStmt,
    binding_name: &str,
    matched_variant: &str,
    payloads: &[DynamicPatternPayload],
    pure_helpers: &BTreeSet<String>,
) -> Option<DynamicPatternUpdate> {
    if let Some(DynamicPatternBranch::Matched {
        payloads,
        type_args,
    }) =
        parse_dynamic_pattern_branch(stmt, binding_name, matched_variant, payloads, pure_helpers)
    {
        return Some(DynamicPatternUpdate::Matched {
            payloads,
            type_args,
        });
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
        payloads,
        pure_helpers,
    )?;
    let else_branch = parse_dynamic_pattern_branch(
        else_stmt,
        binding_name,
        matched_variant,
        payloads,
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
    payload_slots: &[DynamicPatternPayload],
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
        if fields.len() != payload_slots.len() {
            return None;
        }
        let distinct_fields = fields
            .iter()
            .map(|(field, _)| field.as_str())
            .collect::<BTreeSet<_>>();
        if distinct_fields.len() != fields.len() {
            return None;
        }
        let payloads = payload_slots
            .iter()
            .map(|slot| {
                fields
                    .iter()
                    .find_map(|(field, payload)| (field == &slot.field).then(|| payload.clone()))
            })
            .collect::<Option<Vec<_>>>()?;
        Some(DynamicPatternBranch::Matched {
            payloads,
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
    let prepared_payloads = plan
        .payloads
        .iter()
        .enumerate()
        .map(|(index, payload)| PreparedDynamicPatternPayload {
            field: payload.field.clone(),
            carry_name: format!("__pattern_payload_{}_{index}", plan.binding_name),
            initial: payload.initial.clone(),
        })
        .collect::<Vec<_>>();
    let placeholder = || PreparedCarryUpdateKind::Linear {
        op: PreparedCarryLinearOp::Add,
        source: Box::new(PreparedCarrySource::InvariantExpr(NirExpr::Int(0))),
    };
    let mut carries = Vec::with_capacity(prepared_payloads.len() + 1);
    carries.push(PreparedCarryUpdate {
        binding_name: active_carry_name.clone(),
        kind: placeholder(),
    });
    carries.extend(prepared_payloads.iter().map(|payload| PreparedCarryUpdate {
        binding_name: payload.carry_name.clone(),
        kind: placeholder(),
    }));
    let previous_payload_substitutions = plan
        .payloads
        .iter()
        .enumerate()
        .map(|(index, payload)| {
            (
                payload.binding_name.clone(),
                NirExpr::Var(tail_recursive_prev_carry_binding(index + 1)),
            )
        })
        .collect::<Vec<_>>();
    let active_condition = PreparedLoopFlowCondition::Simple(PreparedLoopCarryCondition {
        lhs: PreparedCarryCondSource::PreviousCarry(0),
        compare: PreparedLoopCompare::Ne,
        rhs: NirExpr::Int(0),
    });

    let (condition, then_branch, else_branch, exit_value) = match plan.update {
        DynamicPatternUpdate::Matched { payloads, .. } => (
            active_condition,
            DynamicPatternBranch::Matched {
                payloads,
                type_args: plan.matched_type_args.clone(),
            },
            DynamicPatternBranch::Matched {
                payloads: plan
                    .payloads
                    .iter()
                    .map(|payload| NirExpr::Var(payload.binding_name.clone()))
                    .collect(),
                type_args: plan.matched_type_args.clone(),
            },
            None,
        ),
        DynamicPatternUpdate::Conditional {
            condition,
            then_branch,
            else_branch,
        } => {
            let rewritten = substitute_expr_bindings(&condition, &previous_payload_substitutions);
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
    carries[0].kind = PreparedCarryUpdateKind::Conditional {
        condition: condition.clone(),
        then_source: Box::new(active_source(&then_branch)),
        else_source: Box::new(active_source(&else_branch)),
    };
    for (index, prepared_payload) in prepared_payloads.iter().enumerate() {
        let payload_substitutions = plan
            .payloads
            .iter()
            .enumerate()
            .map(|(source_index, payload)| {
                let value = if source_index == index {
                    NirExpr::Var(prepared_payload.carry_name.clone())
                } else {
                    NirExpr::Var(tail_recursive_prev_carry_binding(source_index + 1))
                };
                (payload.binding_name.clone(), value)
            })
            .collect::<Vec<_>>();
        let payload_source = |branch: &DynamicPatternBranch| match branch {
            DynamicPatternBranch::Matched { payloads, .. } => prepare_payload_branch_source(
                payloads.get(index)?,
                &payload_substitutions,
                &prepared_payload.carry_name,
                loop_binding_name,
                &carries,
                inlineable_pure_helpers,
            ),
            DynamicPatternBranch::Exit(_) => Some(PreparedCarryBranchSource::keep()),
        };
        carries[index + 1].kind = PreparedCarryUpdateKind::Conditional {
            condition: condition.clone(),
            then_source: Box::new(payload_source(&then_branch)?),
            else_source: Box::new(payload_source(&else_branch)?),
        };
    }

    Some((
        PreparedDynamicPatternTransition {
            binding_name: plan.binding_name,
            matched_variant: plan.matched_variant,
            matched_type_args: plan.matched_type_args,
            active_carry_name,
            initial_condition: plan.initial_condition,
            payloads: prepared_payloads,
            exit_value,
        },
        carries,
    ))
}

fn prepare_payload_branch_source(
    payload: &NirExpr,
    payload_substitutions: &[(String, NirExpr)],
    payload_carry_name: &str,
    loop_binding_name: &str,
    carries: &[PreparedCarryUpdate],
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedCarryBranchSource> {
    let renamed = substitute_expr_bindings(payload, payload_substitutions);
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

fn substitute_expr_bindings(expr: &NirExpr, bindings: &[(String, NirExpr)]) -> NirExpr {
    bindings
        .iter()
        .fold(expr.clone(), |rewritten, (name, value)| {
            substitute_branch_binding(&rewritten, name, value)
        })
}
