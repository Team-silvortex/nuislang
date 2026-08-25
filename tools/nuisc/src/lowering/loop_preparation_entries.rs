use super::*;

pub(in crate::lowering) fn prepare_counted_while(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    _pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedCountedWhile> {
    let (binding_name, limit, compare) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;
    let (temp_bindings, step_binding, rest) = split_temp_prefixed_loop_step_bindings(
        body,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    if !rest.is_empty() {
        return None;
    }
    let substituted_step = substitute_stmt_bindings(step_binding, &temp_bindings);
    let (step, step_kind) = parse_prepared_loop_step(
        &substituted_step,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    Some(PreparedCountedWhile {
        binding_name,
        limit,
        step,
        compare,
        step_kind,
    })
}

pub(in crate::lowering) fn prepare_chained_while(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedChainedWhile> {
    let (binding_name, limit, compare) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;
    let (temp_bindings, step_binding, carry_bindings) = split_temp_prefixed_loop_step_bindings(
        body,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    if carry_bindings.is_empty() {
        return None;
    }
    let substituted_step = substitute_stmt_bindings(step_binding, &temp_bindings);
    let (step, step_kind) = parse_prepared_loop_step(
        &substituted_step,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    let substituted_carry_bindings = carry_bindings
        .iter()
        .map(|stmt| substitute_stmt_bindings(stmt, &temp_bindings))
        .collect::<Vec<_>>();

    let carries = prepare_loop_carry_sequence(
        &substituted_carry_bindings,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
        pure_helper_blocks,
    )?;
    if carries.is_empty() {
        return None;
    }

    Some(PreparedChainedWhile {
        binding_name,
        limit,
        step,
        compare,
        step_kind,
        carries,
    })
}

pub(in crate::lowering) fn prepare_async_chained_while(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedAsyncChainedWhile> {
    let (binding_name, limit, compare) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;
    let (temp_bindings, step_binding, carry_bindings) = split_temp_prefixed_loop_step_bindings(
        body,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    if carry_bindings.is_empty() {
        return None;
    }
    let substituted_step = substitute_stmt_bindings(step_binding, &temp_bindings);
    let step_callee = parse_prepared_async_loop_step(&substituted_step, &binding_name)?;
    let substituted_carry_bindings = carry_bindings
        .iter()
        .map(|stmt| substitute_stmt_bindings(stmt, &temp_bindings))
        .collect::<Vec<_>>();

    let carries = prepare_loop_carry_sequence(
        &substituted_carry_bindings,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
        pure_helper_blocks,
    )?;
    if carries.is_empty() {
        return None;
    }

    Some(PreparedAsyncChainedWhile {
        binding_name,
        limit,
        compare,
        step_callee,
        carries,
    })
}

pub(in crate::lowering) fn prepare_async_flow_while(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedAsyncFlowWhile> {
    let (binding_name, limit, compare) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;
    let (temp_bindings, step_binding, rest) = split_temp_prefixed_loop_step_bindings(
        body,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    if rest.is_empty() {
        return None;
    }
    let substituted_step = substitute_stmt_bindings(step_binding, &temp_bindings);
    let step_callee = parse_prepared_async_loop_step(&substituted_step, &binding_name)?;
    let substituted_rest = rest
        .iter()
        .map(|stmt| substitute_stmt_bindings(stmt, &temp_bindings))
        .collect::<Vec<_>>();
    let (control_temp_bindings, raw_control_stmt, carry_bindings) =
        split_temp_prefixed_loop_flow_control(&substituted_rest, pure_helpers)?;
    let substituted_control_stmt =
        substitute_stmt_bindings(raw_control_stmt, &control_temp_bindings);
    let (control, prepared_carries) = if let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = &substituted_control_stmt
    {
        if carry_bindings.is_empty() && !else_body.is_empty() {
            let [action_stmt] = then_body.as_slice() else {
                return None;
            };
            if let Some(action) = match action_stmt {
                NirStmt::Break => Some(PreparedLoopFlowAction::Break),
                NirStmt::Continue => Some(PreparedLoopFlowAction::Continue),
                _ => None,
            } {
                if let Some(prepared_carries) = prepare_loop_carry_sequence(
                    else_body,
                    &binding_name,
                    pure_helpers,
                    inlineable_pure_helpers,
                    pure_helper_blocks,
                ) {
                    let control_condition = parse_loop_flow_condition(
                        condition,
                        &binding_name,
                        &prepared_carries,
                        pure_helpers,
                        inlineable_pure_helpers,
                    )?;
                    (
                        PreparedLoopFlowControl::Terminal {
                            condition: control_condition,
                            action,
                        },
                        prepared_carries,
                    )
                } else {
                    let prepared_carries = prepare_loop_carry_sequence(
                        carry_bindings,
                        &binding_name,
                        pure_helpers,
                        inlineable_pure_helpers,
                        pure_helper_blocks,
                    )?;
                    let control = parse_loop_flow_control(
                        &substituted_control_stmt,
                        &binding_name,
                        &prepared_carries,
                        pure_helpers,
                        inlineable_pure_helpers,
                    )?;
                    (control, prepared_carries)
                }
            } else {
                let prepared_carries = prepare_loop_carry_sequence(
                    carry_bindings,
                    &binding_name,
                    pure_helpers,
                    inlineable_pure_helpers,
                    pure_helper_blocks,
                )?;
                let control = parse_loop_flow_control(
                    &substituted_control_stmt,
                    &binding_name,
                    &prepared_carries,
                    pure_helpers,
                    inlineable_pure_helpers,
                )?;
                (control, prepared_carries)
            }
        } else {
            let prepared_carries = prepare_loop_carry_sequence(
                carry_bindings,
                &binding_name,
                pure_helpers,
                inlineable_pure_helpers,
                pure_helper_blocks,
            )?;
            let control = parse_loop_flow_control(
                &substituted_control_stmt,
                &binding_name,
                &prepared_carries,
                pure_helpers,
                inlineable_pure_helpers,
            )?;
            (control, prepared_carries)
        }
    } else {
        let prepared_carries = prepare_loop_carry_sequence(
            carry_bindings,
            &binding_name,
            pure_helpers,
            inlineable_pure_helpers,
            pure_helper_blocks,
        )?;
        let control = parse_loop_flow_control(
            &substituted_control_stmt,
            &binding_name,
            &prepared_carries,
            pure_helpers,
            inlineable_pure_helpers,
        )?;
        (control, prepared_carries)
    };

    Some(PreparedAsyncFlowWhile {
        binding_name,
        limit,
        compare,
        step_callee,
        control,
        carries: prepared_carries,
    })
}

pub(in crate::lowering) fn prepare_flow_while(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedFlowWhile> {
    let (binding_name, limit, compare) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;
    let (temp_bindings, step_binding, rest) = split_temp_prefixed_loop_step_bindings(
        body,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    if rest.is_empty() {
        return None;
    }
    let substituted_step = substitute_stmt_bindings(step_binding, &temp_bindings);
    let (step, step_kind) = parse_prepared_loop_step(
        &substituted_step,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    let substituted_rest = rest
        .iter()
        .map(|stmt| substitute_stmt_bindings(stmt, &temp_bindings))
        .collect::<Vec<_>>();
    let (control_temp_bindings, raw_control_stmt, carry_bindings) =
        split_temp_prefixed_loop_flow_control(&substituted_rest, pure_helpers)?;
    let substituted_control_stmt =
        substitute_stmt_bindings(raw_control_stmt, &control_temp_bindings);
    let (control, prepared_carries) = if let NirStmt::If {
        condition,
        then_body,
        else_body,
    } = &substituted_control_stmt
    {
        if carry_bindings.is_empty() && !else_body.is_empty() {
            let [action_stmt] = then_body.as_slice() else {
                return None;
            };
            if let Some(action) = match action_stmt {
                NirStmt::Break => Some(PreparedLoopFlowAction::Break),
                NirStmt::Continue => Some(PreparedLoopFlowAction::Continue),
                _ => None,
            } {
                if let Some(prepared_carries) = prepare_loop_carry_sequence(
                    else_body,
                    &binding_name,
                    pure_helpers,
                    inlineable_pure_helpers,
                    pure_helper_blocks,
                ) {
                    let control_condition = parse_loop_flow_condition(
                        condition,
                        &binding_name,
                        &prepared_carries,
                        pure_helpers,
                        inlineable_pure_helpers,
                    )?;
                    (
                        PreparedLoopFlowControl::Terminal {
                            condition: control_condition,
                            action,
                        },
                        prepared_carries,
                    )
                } else {
                    let prepared_carries = prepare_loop_carry_sequence(
                        carry_bindings,
                        &binding_name,
                        pure_helpers,
                        inlineable_pure_helpers,
                        pure_helper_blocks,
                    )?;
                    let control = parse_loop_flow_control(
                        &substituted_control_stmt,
                        &binding_name,
                        &prepared_carries,
                        pure_helpers,
                        inlineable_pure_helpers,
                    )?;
                    (control, prepared_carries)
                }
            } else {
                let prepared_carries = prepare_loop_carry_sequence(
                    carry_bindings,
                    &binding_name,
                    pure_helpers,
                    inlineable_pure_helpers,
                    pure_helper_blocks,
                )?;
                let control = parse_loop_flow_control(
                    &substituted_control_stmt,
                    &binding_name,
                    &prepared_carries,
                    pure_helpers,
                    inlineable_pure_helpers,
                )?;
                (control, prepared_carries)
            }
        } else {
            let prepared_carries = prepare_loop_carry_sequence(
                carry_bindings,
                &binding_name,
                pure_helpers,
                inlineable_pure_helpers,
                pure_helper_blocks,
            )?;
            let control = parse_loop_flow_control(
                &substituted_control_stmt,
                &binding_name,
                &prepared_carries,
                pure_helpers,
                inlineable_pure_helpers,
            )?;
            (control, prepared_carries)
        }
    } else {
        let prepared_carries = prepare_loop_carry_sequence(
            carry_bindings,
            &binding_name,
            pure_helpers,
            inlineable_pure_helpers,
            pure_helper_blocks,
        )?;
        let control = parse_loop_flow_control(
            &substituted_control_stmt,
            &binding_name,
            &prepared_carries,
            pure_helpers,
            inlineable_pure_helpers,
        )?;
        (control, prepared_carries)
    };
    Some(PreparedFlowWhile {
        binding_name,
        limit,
        step,
        compare,
        step_kind,
        control,
        carries: prepared_carries,
    })
}

pub(in crate::lowering) fn prepare_post_flow_while(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedPostFlowWhile> {
    let pattern_gate = match (condition, body) {
        (
            NirExpr::Bool(true),
            [NirStmt::If {
                condition: gate_condition @ NirExpr::VariantIs { .. },
                then_body,
                else_body,
            }],
        ) if matches!(else_body.as_slice(), [NirStmt::Break])
            && is_terminal_branch_pure_expr(gate_condition, pure_helpers) =>
        {
            Some((gate_condition.clone(), then_body.as_slice()))
        }
        _ => None,
    };
    let mut terminal_pattern_transition = None;
    let mut transitioned_pattern_body = None;
    if let Some((gate_condition, then_body)) = &pattern_gate {
        if let Some((transition, prepared_body)) =
            prepare_terminal_pattern_transition(gate_condition, then_body, pure_helpers)
        {
            terminal_pattern_transition = Some(transition);
            transitioned_pattern_body = Some(prepared_body);
        }
    }
    let mut dynamic_pattern_plan = None;
    let mut dynamic_pattern_body = None;
    if terminal_pattern_transition.is_none() {
        if let Some((gate_condition, then_body)) = &pattern_gate {
            if let Some((plan, prepared_body)) =
                prepare_dynamic_pattern_plan(gate_condition, then_body, pure_helpers)
            {
                dynamic_pattern_plan = Some(plan);
                dynamic_pattern_body = Some(prepared_body);
            }
        }
    }
    let prepared_body = dynamic_pattern_body
        .as_deref()
        .or(transitioned_pattern_body.as_deref())
        .unwrap_or_else(|| {
            pattern_gate
                .as_ref()
                .map_or(body, |(_, then_body)| *then_body)
        });
    let (binding_name, entry_condition) = if let Some((condition, _)) = &pattern_gate {
        (
            parse_unbounded_loop_step_binding(
                &NirExpr::Bool(true),
                prepared_body,
                pure_helpers,
                inlineable_pure_helpers,
            )?,
            if dynamic_pattern_plan.is_some() {
                PreparedLoopEntryCondition::DynamicPattern {
                    active_carry_index: 0,
                }
            } else {
                PreparedLoopEntryCondition::InvariantPattern {
                    condition: condition.clone(),
                }
            },
        )
    } else if let Some((binding_name, limit, compare)) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)
    {
        (
            binding_name,
            PreparedLoopEntryCondition::Bounded { limit, compare },
        )
    } else {
        (
            parse_unbounded_loop_step_binding(
                condition,
                body,
                pure_helpers,
                inlineable_pure_helpers,
            )?,
            PreparedLoopEntryCondition::Unbounded,
        )
    };
    let (temp_bindings, step_binding, rest) = split_temp_prefixed_loop_step_bindings(
        prepared_body,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    let [middle @ .., control_stmt] = rest else {
        return None;
    };
    if middle.is_empty() {
        return None;
    }
    let substituted_step = substitute_stmt_bindings(step_binding, &temp_bindings);
    let (step, step_kind) = parse_prepared_loop_step(
        &substituted_step,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    let substituted_middle = middle
        .iter()
        .map(|stmt| substitute_stmt_bindings(stmt, &temp_bindings))
        .collect::<Vec<_>>();
    let substituted_control_stmt = substitute_stmt_bindings(control_stmt, &temp_bindings);

    let (carry_bindings, control_temp_bindings) = split_trailing_loop_control_temp_bindings(
        &substituted_middle,
        &substituted_control_stmt,
        pure_helpers,
    )?;
    let (dynamic_pattern_transition, carry_prefix) = if let Some(plan) = dynamic_pattern_plan {
        let (transition, carries) = prepare_dynamic_pattern_carries(
            plan,
            &binding_name,
            pure_helpers,
            inlineable_pure_helpers,
        )?;
        (Some(transition), carries)
    } else {
        (None, Vec::new())
    };
    let prepared_carries = prepare_loop_carry_sequence_with_prefix(
        carry_bindings,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
        pure_helper_blocks,
        carry_prefix,
    )?;
    let final_control_stmt =
        substitute_stmt_bindings(&substituted_control_stmt, &control_temp_bindings);
    let control = parse_loop_flow_control(
        &final_control_stmt,
        &binding_name,
        &prepared_carries,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    if let PreparedLoopEntryCondition::InvariantPattern { condition } = &entry_condition {
        let mut updated_names = BTreeSet::from([binding_name.as_str()]);
        updated_names.extend(
            prepared_carries
                .iter()
                .map(|carry| carry.binding_name.as_str()),
        );
        if terminal_pattern_transition.is_none() && expr_references_names(condition, &updated_names)
        {
            return None;
        }
    }
    Some(PreparedPostFlowWhile {
        binding_name,
        entry_condition,
        terminal_pattern_transition,
        dynamic_pattern_transition,
        step,
        step_kind,
        carries: prepared_carries,
        control,
    })
}

pub(in crate::lowering) fn prepare_async_post_flow_while(
    condition: &NirExpr,
    body: &[NirStmt],
    pure_helpers: &BTreeSet<String>,
    inlineable_pure_helpers: &BTreeMap<String, InlineablePureHelper>,
    pure_helper_blocks: &BTreeMap<String, PureHelperBlock>,
) -> Option<PreparedAsyncPostFlowWhile> {
    let (binding_name, limit, compare) =
        parse_prepared_loop_header(condition, pure_helpers, inlineable_pure_helpers)?;
    let (temp_bindings, step_binding, rest) = split_temp_prefixed_loop_step_bindings(
        body,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    let [middle @ .., control_stmt] = rest else {
        return None;
    };
    if middle.is_empty() {
        return None;
    }
    let substituted_step = substitute_stmt_bindings(step_binding, &temp_bindings);
    let step_callee = parse_prepared_async_loop_step(&substituted_step, &binding_name)?;
    let substituted_middle = middle
        .iter()
        .map(|stmt| substitute_stmt_bindings(stmt, &temp_bindings))
        .collect::<Vec<_>>();
    let substituted_control_stmt = substitute_stmt_bindings(control_stmt, &temp_bindings);

    let (carry_bindings, control_temp_bindings) = split_trailing_loop_control_temp_bindings(
        &substituted_middle,
        &substituted_control_stmt,
        pure_helpers,
    )?;
    let prepared_carries = prepare_loop_carry_sequence(
        carry_bindings,
        &binding_name,
        pure_helpers,
        inlineable_pure_helpers,
        pure_helper_blocks,
    )?;
    let final_control_stmt =
        substitute_stmt_bindings(&substituted_control_stmt, &control_temp_bindings);
    let control = parse_loop_flow_control(
        &final_control_stmt,
        &binding_name,
        &prepared_carries,
        pure_helpers,
        inlineable_pure_helpers,
    )?;
    Some(PreparedAsyncPostFlowWhile {
        binding_name,
        limit,
        compare,
        step_callee,
        carries: prepared_carries,
        control,
    })
}
