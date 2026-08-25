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
        then_branch: Box<DynamicPatternBranch>,
        else_branch: Box<DynamicPatternBranch>,
    },
}

struct DynamicPatternPayload {
    binding_name: String,
    field: String,
    ty: NirTypeRef,
    transport: PreparedDynamicPatternPayloadTransport,
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
    let payloads = collect_dynamic_pattern_payloads(gate_condition, body)?.ok()?;
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

pub(super) fn diagnose_dynamic_pattern_payload_admission(
    gate_condition: &NirExpr,
    body: &[NirStmt],
) -> Option<String> {
    collect_dynamic_pattern_payloads(gate_condition, body)?.err()
}

fn collect_dynamic_pattern_payloads(
    gate_condition: &NirExpr,
    body: &[NirStmt],
) -> Option<Result<Vec<(usize, DynamicPatternPayload)>, String>> {
    let NirExpr::VariantIs { base, variant } = gate_condition else {
        return None;
    };
    let NirExpr::Var(binding_name) = base.as_ref() else {
        return None;
    };
    let mut payloads = Vec::new();
    for (index, stmt) in body.iter().enumerate() {
        let (name, ty, value) = match stmt {
            NirStmt::Let { name, ty, value } => (name, ty.as_ref(), value),
            NirStmt::Const { name, ty, value } => (name, Some(ty), value),
            _ => break,
        };
        let NirExpr::VariantFieldAccess {
            base,
            variant: field_variant,
            field,
        } = value
        else {
            break;
        };
        if field_variant != variant
            || !matches!(base.as_ref(), NirExpr::Var(name) if name == binding_name)
        {
            break;
        }
        let Some(ty) = ty else {
            return Some(Err(format!(
                "pattern-controlled `while let` cannot lower payload field `{field}` bound as `{name}` because its NIR type is unresolved; dynamic backedges require an explicit payload transport type"
            )));
        };
        let Some(transport) = PreparedDynamicPatternPayloadTransport::for_type(ty) else {
            let required_contract = match ty.scalar_kind() {
                Some(
                    NirScalarKind::I32
                    | NirScalarKind::F32
                    | NirScalarKind::F64
                    | NirScalarKind::Unit,
                ) => "typed-scalar payload carry contract",
                Some(NirScalarKind::Text) | None => "GLM-owned payload carry contract",
                Some(NirScalarKind::Bool | NirScalarKind::I64) => {
                    unreachable!("admitted scalar payloads have a transport")
                }
            };
            return Some(Err(format!(
                "pattern-controlled `while let` dynamic backedges currently support `i64` and `bool` payload carries through `{DYNAMIC_PATTERN_PAYLOAD_CARRY_PROTOCOL_V2}`; field `{field}` bound as `{name}` has type `{}` and requires a {required_contract}",
                ty.render()
            )));
        };
        payloads.push((
            index,
            DynamicPatternPayload {
                binding_name: name.clone(),
                field: field.clone(),
                ty: ty.clone(),
                transport,
                initial: value.clone(),
            },
        ));
    }
    (!payloads.is_empty()).then_some(Ok(payloads))
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
        then_branch: Box::new(then_branch),
        else_branch: Box::new(else_branch),
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
            .find_map(|branch| match branch.as_ref() {
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
    if plan.payloads.iter().any(|payload| {
        PreparedDynamicPatternPayloadTransport::for_type(&payload.ty) != Some(payload.transport)
    }) {
        return None;
    }
    let active_carry_name = format!("__pattern_active_{}", plan.binding_name);
    let prepared_payloads = plan
        .payloads
        .iter()
        .enumerate()
        .map(|(index, payload)| PreparedDynamicPatternPayload {
            field: payload.field.clone(),
            carry_name: format!("__pattern_payload_{}_{index}", plan.binding_name),
            transport: payload.transport,
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
                payload
                    .transport
                    .decode_expr(NirExpr::Var(tail_recursive_prev_carry_binding(index + 1))),
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
            let exit_value = [then_branch.as_ref(), else_branch.as_ref()]
                .into_iter()
                .find_map(|branch| match branch {
                    DynamicPatternBranch::Exit(value) => Some(value.clone()),
                    DynamicPatternBranch::Matched { .. } => None,
                });
            (condition, *then_branch, *else_branch, exit_value)
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
                (
                    payload.binding_name.clone(),
                    payload.transport.decode_expr(value),
                )
            })
            .collect::<Vec<_>>();
        let payload_source = |branch: &DynamicPatternBranch| match branch {
            DynamicPatternBranch::Matched { payloads, .. } => prepare_payload_branch_source(
                payloads.get(index)?,
                &payload_substitutions,
                &prepared_payload.carry_name,
                prepared_payload.transport,
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
            protocol: DYNAMIC_PATTERN_PAYLOAD_CARRY_PROTOCOL_V2,
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
    transport: PreparedDynamicPatternPayloadTransport,
    loop_binding_name: &str,
    carries: &[PreparedCarryUpdate],
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
) -> Option<PreparedCarryBranchSource> {
    let renamed = substitute_expr_bindings(payload, payload_substitutions);
    let encoded = transport.encode_expr(renamed);
    let normalized = match encoded {
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
