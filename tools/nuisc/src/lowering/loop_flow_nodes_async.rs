use super::*;

pub(in crate::lowering) fn lower_async_flow_while(
    prepared: PreparedAsyncFlowWhile,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let Some(function) = state
        .function_map
        .get(prepared.step_callee.as_str())
        .copied()
    else {
        return Err(format!(
            "async flow `while` references unknown step helper `{}`",
            prepared.step_callee
        ));
    };
    if !function.is_async {
        return Err(format!(
            "async flow `while` step helper `{}` must be `async fn`",
            prepared.step_callee
        ));
    }
    if function.params.len() != 1 {
        return Err(format!(
            "async flow `while` step helper `{}` must take exactly one parameter",
            prepared.step_callee
        ));
    }

    let Some(initial_name) = bindings.get(&prepared.binding_name).cloned() else {
        return Err(format!(
            "async flow `while` expected an existing binding for `{}` before the loop",
            prepared.binding_name
        ));
    };
    let mut carry_initial_names = Vec::with_capacity(prepared.carries.len());
    for carry in &prepared.carries {
        let Some(carry_initial_name) = bindings.get(&carry.binding_name).cloned() else {
            return Err(format!(
                "async flow `while` expected an existing binding for `{}` before the loop",
                carry.binding_name
            ));
        };
        carry_initial_names.push(carry_initial_name);
    }
    let limit_name = lower_expr(&prepared.limit, state, bindings)?;
    let (control_args, control_dep_inputs, control_effect_inputs, control_uses_cond_chain) =
        encode_loop_flow_control_args(&prepared.control, state, bindings)?;
    let has_conditional = prepared
        .carries
        .iter()
        .any(|carry| matches!(carry.kind, PreparedCarryUpdateKind::Conditional { .. }));
    let compare = render_loop_compare(prepared.compare);
    let mut args = vec![
        initial_name.clone(),
        limit_name.clone(),
        prepared.step_callee.clone(),
        compare.to_owned(),
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
    let name = next_name(
        state,
        if has_conditional || control_uses_cond_chain {
            "loop_while_scalar_async_flow_cond_chain"
        } else {
            "loop_while_scalar_async_flow_chain"
        },
    );
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: if has_conditional || control_uses_cond_chain {
                "loop_while_scalar_async_flow_cond_chain".to_owned()
            } else {
                "loop_while_scalar_async_flow_chain".to_owned()
            },
            args,
        },
    });
    for dep in [&initial_name, &limit_name] {
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
    for effect in [&initial_name, &limit_name] {
        state.yir.edges.push(Edge {
            kind: EdgeKind::Effect,
            from: effect.clone(),
            to: name.clone(),
        });
    }
    for control_effect_input in &control_effect_inputs {
        state.yir.edges.push(Edge {
            kind: EdgeKind::Effect,
            from: control_effect_input.clone(),
            to: name.clone(),
        });
    }
    for carry_initial_name in &carry_initial_names {
        state.yir.edges.push(Edge {
            kind: EdgeKind::Effect,
            from: carry_initial_name.clone(),
            to: name.clone(),
        });
    }
    for extra_effect_input in &extra_effect_inputs {
        state.yir.edges.push(Edge {
            kind: EdgeKind::Effect,
            from: extra_effect_input.clone(),
            to: name.clone(),
        });
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
        let lowered_name = next_name(state, "loop_carry");
        state.yir.nodes.push(Node {
            name: lowered_name.clone(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "field".to_owned(),
                args: vec![name.clone(), format!("carry{index}")],
            },
        });
        push_dep_edges(state, &name, &lowered_name);
        bindings.insert(carry.binding_name.clone(), lowered_name);
    }
    Ok(())
}
