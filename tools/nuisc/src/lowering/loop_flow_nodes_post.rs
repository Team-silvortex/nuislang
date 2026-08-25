use super::*;

pub(in crate::lowering) fn lower_post_flow_while(
    prepared: PreparedPostFlowWhile,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let Some(initial_name) = bindings.get(&prepared.binding_name).cloned() else {
        return Err(format!(
            "post-flow `while` expected an existing binding for `{}` before the loop",
            prepared.binding_name
        ));
    };
    let dynamic_pattern_initials = if let Some(transition) = &prepared.dynamic_pattern_transition {
        if transition.protocol != DYNAMIC_PATTERN_PAYLOAD_CARRY_PROTOCOL_V2 {
            return Err(format!(
                "dynamic pattern transition requires `{DYNAMIC_PATTERN_PAYLOAD_CARRY_PROTOCOL_V2}`, found `{}`",
                transition.protocol
            ));
        }
        let initial_variant = bindings
            .get(&transition.binding_name)
            .cloned()
            .ok_or_else(|| {
                format!(
                    "dynamic pattern transition expected an existing binding for `{}` before the loop",
                    transition.binding_name
                )
        })?;
        let active_initial = lower_expr(&transition.initial_condition, state, bindings)?;
        bindings.insert(transition.active_carry_name.clone(), active_initial.clone());
        for payload in &transition.payloads {
            let projected_payload = lower_expr(&payload.initial, state, bindings)?;
            let neutral_payload = lower_expr(&payload.transport.neutral_expr(), state, bindings)?;
            let selected_payload = next_name(state, "pattern_payload_initial");
            state.yir.nodes.push(Node {
                name: selected_payload.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "select".to_owned(),
                    args: vec![
                        active_initial.clone(),
                        projected_payload.clone(),
                        neutral_payload.clone(),
                    ],
                },
            });
            for dep in [&active_initial, &projected_payload, &neutral_payload] {
                push_dep_edges(state, dep, &selected_payload);
            }
            let payload_initial =
                encode_dynamic_pattern_payload(payload.transport, selected_payload, state);
            bindings.insert(payload.carry_name.clone(), payload_initial);
        }
        Some((initial_variant, active_initial))
    } else {
        None
    };
    let mut carry_initial_names = Vec::with_capacity(prepared.carries.len());
    for carry in &prepared.carries {
        let Some(carry_initial_name) = bindings.get(&carry.binding_name).cloned() else {
            return Err(format!(
                "post-flow `while` expected an existing binding for `{}` before the loop",
                carry.binding_name
            ));
        };
        carry_initial_names.push(carry_initial_name);
    }
    let terminal_pattern_transition = if let Some(transition) =
        &prepared.terminal_pattern_transition
    {
        let initial_variant = bindings
            .get(&transition.binding_name)
            .cloned()
            .ok_or_else(|| {
                format!(
                "terminal pattern transition expected an existing binding for `{}` before the loop",
                transition.binding_name
            )
            })?;
        let transitioned_variant = lower_expr(&transition.value, state, bindings)?;
        Some((
            transition.binding_name.clone(),
            initial_variant,
            transitioned_variant,
        ))
    } else {
        None
    };
    let (limit_name, compare) = match &prepared.entry_condition {
        PreparedLoopEntryCondition::Bounded { limit, compare } => (
            lower_expr(limit, state, bindings)?,
            render_loop_compare(*compare).to_owned(),
        ),
        PreparedLoopEntryCondition::InvariantPattern { condition } => (
            lower_expr(condition, state, bindings)?,
            if terminal_pattern_transition.is_some() {
                "pattern_exit".to_owned()
            } else {
                "invariant_true".to_owned()
            },
        ),
        PreparedLoopEntryCondition::DynamicPattern { active_carry_index } => (
            lower_expr(&NirExpr::Int(0), state, bindings)?,
            format!("pattern_carry{active_carry_index}"),
        ),
        PreparedLoopEntryCondition::Unbounded => (
            lower_expr(&NirExpr::Int(0), state, bindings)?,
            "always".to_owned(),
        ),
    };
    let step_name = lower_expr(&prepared.step, state, bindings)?;
    let (control_args, control_dep_inputs, control_effect_inputs, control_uses_cond_chain) =
        encode_loop_flow_control_args(&prepared.control, state, bindings)?;
    let has_conditional = prepared
        .carries
        .iter()
        .any(|carry| matches!(carry.kind, PreparedCarryUpdateKind::Conditional { .. }));
    let uses_cond_chain = has_conditional || control_uses_cond_chain;
    let step_kind = match prepared.step_kind {
        PreparedLoopStepKind::Add => "add",
        PreparedLoopStepKind::Sub => "sub",
    };
    let mut args = vec![
        initial_name.clone(),
        limit_name.clone(),
        step_name.clone(),
        compare,
        step_kind.to_owned(),
    ];
    args.extend(control_args);
    let mut extra_dep_inputs: Vec<String> = Vec::new();
    let mut extra_effect_inputs: Vec<String> = Vec::new();
    for (index, carry_initial_name) in carry_initial_names.iter().enumerate() {
        args.push(carry_initial_name.clone());
        match &prepared.carries[index].kind {
            PreparedCarryUpdateKind::Linear { op, source } => {
                if has_conditional {
                    args.push("always".to_owned());
                    args.push(initial_name.clone());
                    let (carry_args, carry_dep_inputs, carry_effect_inputs) =
                        encode_loop_carry_source_args(*op, source, state, bindings)?;
                    args.extend(carry_args.clone());
                    args.extend(carry_args);
                    extra_dep_inputs.push(initial_name.clone());
                    extra_effect_inputs.push(initial_name.clone());
                    extra_dep_inputs.extend(carry_dep_inputs);
                    extra_effect_inputs.extend(carry_effect_inputs);
                } else {
                    let (carry_args, carry_dep_inputs, carry_effect_inputs) =
                        encode_loop_carry_source_args(*op, source, state, bindings)?;
                    args.extend(carry_args);
                    extra_dep_inputs.extend(carry_dep_inputs);
                    extra_effect_inputs.extend(carry_effect_inputs);
                }
            }
            PreparedCarryUpdateKind::Conditional {
                condition,
                then_source,
                else_source,
            } => {
                let (condition_args, cond_dep_inputs, cond_effect_inputs) =
                    encode_carry_condition_args(condition, state, bindings)?;
                args.extend(condition_args);
                let (then_args, then_dep_inputs, then_effect_inputs) =
                    encode_loop_carry_branch_source_args(then_source, state, bindings)?;
                let (else_args, else_dep_inputs, else_effect_inputs) =
                    encode_loop_carry_branch_source_args(else_source, state, bindings)?;
                args.extend(then_args);
                args.extend(else_args);
                extra_dep_inputs.extend(cond_dep_inputs);
                extra_effect_inputs.extend(cond_effect_inputs);
                extra_dep_inputs.extend(then_dep_inputs);
                extra_dep_inputs.extend(else_dep_inputs);
                extra_effect_inputs.extend(then_effect_inputs);
                extra_effect_inputs.extend(else_effect_inputs);
            }
        }
    }
    if let Some(transition) = &prepared.dynamic_pattern_transition {
        let contract = DynamicPatternPayloadCarryContract {
            slots: transition
                .payloads
                .iter()
                .enumerate()
                .map(|(index, payload)| DynamicPatternPayloadCarrySlot {
                    carry_index: index + 1,
                    codec: payload.transport.yir_codec(),
                })
                .collect(),
        };
        args.extend(encode_dynamic_pattern_payload_carry_trailer(&contract)?);
    }
    let name = next_name(
        state,
        if uses_cond_chain {
            "loop_while_scalar_post_flow_cond_chain"
        } else {
            "loop_while_scalar_post_flow_chain"
        },
    );
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: if uses_cond_chain {
                "loop_while_scalar_post_flow_cond_chain".to_owned()
            } else {
                "loop_while_scalar_post_flow_chain".to_owned()
            },
            args,
        },
    });
    for dep in [&initial_name, &limit_name, &step_name] {
        push_dep_edges(state, dep, &name);
    }
    for control_dep_input in &control_dep_inputs {
        push_dep_edges(state, control_dep_input, &name);
    }
    for carry_initial_name in &carry_initial_names {
        push_dep_edges(state, carry_initial_name, &name);
    }
    for extra_dep_input in &extra_dep_inputs {
        push_dep_edges(state, extra_dep_input, &name);
    }
    push_effect_edge(state, &initial_name, &name);
    push_effect_edge(state, &limit_name, &name);
    push_effect_edge(state, &step_name, &name);
    for control_effect_input in &control_effect_inputs {
        push_effect_edge(state, control_effect_input, &name);
    }
    for carry_initial_name in &carry_initial_names {
        push_effect_edge(state, carry_initial_name, &name);
    }
    for extra_effect_input in &extra_effect_inputs {
        push_effect_edge(state, extra_effect_input, &name);
    }
    super::body_lowering::chain_statement_effect(state, &name);

    let current_name = next_name(state, "loop_current");
    state.yir.nodes.push(Node {
        name: current_name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "field".to_owned(),
            args: vec![name.clone(), "current".to_owned()],
        },
    });
    push_dep_edges(state, &name, &current_name);
    bindings.insert(prepared.binding_name, current_name);
    for (index, carry) in prepared.carries.iter().enumerate() {
        let carry_name = next_name(state, "loop_carry");
        state.yir.nodes.push(Node {
            name: carry_name.clone(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "field".to_owned(),
                args: vec![name.clone(), format!("carry{index}")],
            },
        });
        push_dep_edges(state, &name, &carry_name);
        bindings.insert(carry.binding_name.clone(), carry_name);
    }
    if let Some((binding_name, initial_variant, transitioned_variant)) = terminal_pattern_transition
    {
        let selected_variant = next_name(state, "loop_variant_state");
        state.yir.nodes.push(Node {
            name: selected_variant.clone(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "select".to_owned(),
                args: vec![
                    limit_name.clone(),
                    transitioned_variant.clone(),
                    initial_variant.clone(),
                ],
            },
        });
        for dep in [&limit_name, &transitioned_variant, &initial_variant, &name] {
            push_dep_edges(state, dep, &selected_variant);
        }
        bindings.insert(binding_name, selected_variant);
    }
    if let (Some(transition), Some((initial_variant, initial_condition))) = (
        &prepared.dynamic_pattern_transition,
        dynamic_pattern_initials,
    ) {
        let active = bindings
            .get(&transition.active_carry_name)
            .cloned()
            .ok_or_else(|| "dynamic pattern loop lost its active carry result".to_owned())?;
        let matched_variant = lower_expr(
            &NirExpr::StructLiteral {
                type_name: transition.matched_variant.clone(),
                type_args: transition.matched_type_args.clone(),
                fields: transition
                    .payloads
                    .iter()
                    .map(|payload| {
                        (
                            payload.field.clone(),
                            payload
                                .transport
                                .decode_expr(NirExpr::Var(payload.carry_name.clone())),
                        )
                    })
                    .collect(),
            },
            state,
            bindings,
        )?;
        let inactive_variant = if let Some(exit_value) = &transition.exit_value {
            let exited_variant = lower_expr(exit_value, state, bindings)?;
            let selected_exit = next_name(state, "loop_variant_exit_state");
            state.yir.nodes.push(Node {
                name: selected_exit.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "select".to_owned(),
                    args: vec![
                        initial_condition.clone(),
                        exited_variant.clone(),
                        initial_variant.clone(),
                    ],
                },
            });
            for dep in [&initial_condition, &exited_variant, &initial_variant] {
                push_dep_edges(state, dep, &selected_exit);
            }
            selected_exit
        } else {
            initial_variant
        };
        let selected_variant = next_name(state, "loop_variant_state");
        state.yir.nodes.push(Node {
            name: selected_variant.clone(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "select".to_owned(),
                args: vec![
                    active.clone(),
                    matched_variant.clone(),
                    inactive_variant.clone(),
                ],
            },
        });
        for dep in [&active, &matched_variant, &inactive_variant, &name] {
            push_dep_edges(state, dep, &selected_variant);
        }
        bindings.insert(transition.binding_name.clone(), selected_variant);
    }
    Ok(())
}

fn encode_dynamic_pattern_payload(
    transport: PreparedDynamicPatternPayloadTransport,
    source: String,
    state: &mut LoweringState<'_>,
) -> String {
    match transport {
        PreparedDynamicPatternPayloadTransport::I64Identity => source,
        PreparedDynamicPatternPayloadTransport::BoolAsI64 => {
            let encoded = next_name(state, "pattern_payload_bool_i64");
            state.yir.nodes.push(Node {
                name: encoded.clone(),
                resource: "cpu0".to_owned(),
                op: Operation {
                    module: "cpu".to_owned(),
                    instruction: "cast_bool_to_i64".to_owned(),
                    args: vec![source.clone()],
                },
            });
            push_dep_edges(state, &source, &encoded);
            encoded
        }
    }
}
