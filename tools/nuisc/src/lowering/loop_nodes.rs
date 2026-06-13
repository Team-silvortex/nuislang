use super::*;
use crate::lowering::edge_helpers::push_effect_edge;
use crate::lowering::edge_helpers::push_lifetime_edge;

fn encode_carry_condition_args(
    condition: &PreparedLoopFlowCondition,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<(Vec<String>, Vec<String>, Vec<String>), String> {
    match condition {
        PreparedLoopFlowCondition::Simple(condition) => {
            let rhs_name = lower_expr(&condition.rhs, state, bindings)?;
            Ok((
                vec![
                    render_loop_cond_kind(&condition.lhs, condition.compare),
                    rhs_name.clone(),
                ],
                vec![rhs_name.clone()],
                vec![rhs_name],
            ))
        }
        PreparedLoopFlowCondition::Compound { op, lhs, rhs } => {
            let (mut lhs_args, mut lhs_dep_inputs, mut lhs_effect_inputs) =
                encode_carry_condition_args(lhs, state, bindings)?;
            let (rhs_args, rhs_dep_inputs, rhs_effect_inputs) =
                encode_carry_condition_args(rhs, state, bindings)?;
            let mut args = vec![render_loop_logic_op(*op).to_owned()];
            args.append(&mut lhs_args);
            args.extend(rhs_args);
            lhs_dep_inputs.extend(rhs_dep_inputs);
            lhs_effect_inputs.extend(rhs_effect_inputs);
            Ok((args, lhs_dep_inputs, lhs_effect_inputs))
        }
    }
}

pub(super) fn lower_counted_while(
    prepared: PreparedCountedWhile,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let Some(initial_name) = bindings.get(&prepared.binding_name).cloned() else {
        return Err(format!(
            "counted `while` expected an existing binding for `{}` before the loop",
            prepared.binding_name
        ));
    };
    let limit_name = lower_expr(&prepared.limit, state, bindings)?;
    let step_name = lower_expr(&prepared.step, state, bindings)?;
    let name = next_name(state, "loop_while_i64");
    let compare = render_loop_compare(prepared.compare);
    let step_kind = match prepared.step_kind {
        PreparedLoopStepKind::Add => "add",
        PreparedLoopStepKind::Sub => "sub",
    };
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "loop_while_i64".to_owned(),
            args: vec![
                initial_name.clone(),
                limit_name.clone(),
                step_name.clone(),
                compare.to_owned(),
                step_kind.to_owned(),
            ],
        },
    });
    push_dep_edges(state, &initial_name, &name);
    push_dep_edges(state, &limit_name, &name);
    push_dep_edges(state, &step_name, &name);
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: initial_name,
        to: name.clone(),
    });
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: limit_name,
        to: name.clone(),
    });
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: step_name,
        to: name.clone(),
    });
    bindings.insert(prepared.binding_name, name);
    Ok(())
}

pub(super) fn lower_chained_while(
    prepared: PreparedChainedWhile,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let Some(initial_name) = bindings.get(&prepared.binding_name).cloned() else {
        return Err(format!(
            "chained `while` expected an existing binding for `{}` before the loop",
            prepared.binding_name
        ));
    };
    let mut carry_initial_names = Vec::with_capacity(prepared.carries.len());
    for carry in &prepared.carries {
        let Some(carry_initial_name) = bindings.get(&carry.binding_name).cloned() else {
            return Err(format!(
                "chained `while` expected an existing binding for `{}` before the loop",
                carry.binding_name
            ));
        };
        carry_initial_names.push(carry_initial_name);
    }
    let limit_name = lower_expr(&prepared.limit, state, bindings)?;
    let step_name = lower_expr(&prepared.step, state, bindings)?;
    let has_conditional = prepared
        .carries
        .iter()
        .any(|carry| matches!(carry.kind, PreparedCarryUpdateKind::Conditional { .. }));
    let name = next_name(
        state,
        if has_conditional {
            "loop_while_scalar_cond_chain"
        } else {
            "loop_while_scalar_chain"
        },
    );
    let compare = render_loop_compare(prepared.compare);
    let step_kind = match prepared.step_kind {
        PreparedLoopStepKind::Add => "add",
        PreparedLoopStepKind::Sub => "sub",
    };
    let mut args = vec![
        initial_name.clone(),
        limit_name.clone(),
        step_name.clone(),
        compare.to_owned(),
        step_kind.to_owned(),
    ];
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
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: if has_conditional {
                "loop_while_scalar_cond_chain".to_owned()
            } else {
                "loop_while_scalar_chain".to_owned()
            },
            args,
        },
    });
    push_dep_edges(state, &initial_name, &name);
    push_dep_edges(state, &limit_name, &name);
    push_dep_edges(state, &step_name, &name);
    for carry_initial_name in &carry_initial_names {
        push_dep_edges(state, carry_initial_name, &name);
    }
    for extra_dep_input in &extra_dep_inputs {
        push_dep_edges(state, extra_dep_input, &name);
    }
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: initial_name.clone(),
        to: name.clone(),
    });
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: limit_name.clone(),
        to: name.clone(),
    });
    state.yir.edges.push(Edge {
        kind: EdgeKind::Effect,
        from: step_name.clone(),
        to: name.clone(),
    });
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
    Ok(())
}

pub(super) fn lower_async_chained_while(
    prepared: PreparedAsyncChainedWhile,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let Some(function) = state
        .function_map
        .get(prepared.step_callee.as_str())
        .copied()
    else {
        return Err(format!(
            "async chained `while` references unknown step helper `{}`",
            prepared.step_callee
        ));
    };
    if !function.is_async {
        return Err(format!(
            "async chained `while` step helper `{}` must be `async fn`",
            prepared.step_callee
        ));
    }
    if function.params.len() != 1 {
        return Err(format!(
            "async chained `while` step helper `{}` must take exactly one parameter",
            prepared.step_callee
        ));
    }

    let Some(initial_name) = bindings.get(&prepared.binding_name).cloned() else {
        return Err(format!(
            "async chained `while` expected an existing binding for `{}` before the loop",
            prepared.binding_name
        ));
    };
    let mut carry_initial_names = Vec::with_capacity(prepared.carries.len());
    for carry in &prepared.carries {
        let Some(carry_initial_name) = bindings.get(&carry.binding_name).cloned() else {
            return Err(format!(
                "async chained `while` expected an existing binding for `{}` before the loop",
                carry.binding_name
            ));
        };
        carry_initial_names.push(carry_initial_name);
    }
    let limit_name = lower_expr(&prepared.limit, state, bindings)?;
    let has_conditional = prepared
        .carries
        .iter()
        .any(|carry| matches!(carry.kind, PreparedCarryUpdateKind::Conditional { .. }));
    let name = next_name(
        state,
        if has_conditional {
            "loop_while_scalar_async_cond_chain"
        } else {
            "loop_while_scalar_async_chain"
        },
    );
    let compare = render_loop_compare(prepared.compare);
    let mut args = vec![
        initial_name.clone(),
        limit_name.clone(),
        prepared.step_callee.clone(),
        compare.to_owned(),
    ];
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
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: if has_conditional {
                "loop_while_scalar_async_cond_chain".to_owned()
            } else {
                "loop_while_scalar_async_chain".to_owned()
            },
            args,
        },
    });
    push_dep_edges(state, &initial_name, &name);
    push_dep_edges(state, &limit_name, &name);
    for carry_initial_name in &carry_initial_names {
        push_dep_edges(state, carry_initial_name, &name);
    }
    for extra_dep_input in &extra_dep_inputs {
        push_dep_edges(state, extra_dep_input, &name);
    }
    push_effect_edge(state, &initial_name, &name);
    push_effect_edge(state, &limit_name, &name);
    for carry_initial_name in &carry_initial_names {
        push_effect_edge(state, carry_initial_name, &name);
    }
    for extra_effect_input in &extra_effect_inputs {
        push_effect_edge(state, extra_effect_input, &name);
        push_lifetime_edge(state, extra_effect_input, &name);
    }

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
    Ok(())
}
