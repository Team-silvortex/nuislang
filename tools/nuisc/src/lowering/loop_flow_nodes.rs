use super::*;

pub(super) fn lower_flow_while(
    prepared: PreparedFlowWhile,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let Some(initial_name) = bindings.get(&prepared.binding_name).cloned() else {
        return Err(format!(
            "flow `while` expected an existing binding for `{}` before the loop",
            prepared.binding_name
        ));
    };
    let mut carry_initial_names = Vec::with_capacity(prepared.carries.len());
    for carry in &prepared.carries {
        let Some(carry_initial_name) = bindings.get(&carry.binding_name).cloned() else {
            return Err(format!(
                "flow `while` expected an existing binding for `{}` before the loop",
                carry.binding_name
            ));
        };
        carry_initial_names.push(carry_initial_name);
    }
    let limit_name = lower_expr(&prepared.limit, state, bindings)?;
    let step_name = lower_expr(&prepared.step, state, bindings)?;
    let (control_args, control_dep_inputs, control_effect_inputs) =
        match &prepared.control.condition {
            PreparedLoopFlowCondition::Simple(condition) => {
                let control_rhs_name = lower_expr(&condition.rhs, state, bindings)?;
                (
                    vec![
                        render_loop_cond_kind(&condition.lhs, condition.compare),
                        control_rhs_name.clone(),
                        match prepared.control.action {
                            PreparedLoopFlowAction::Break => "break".to_owned(),
                            PreparedLoopFlowAction::Continue => "continue".to_owned(),
                        },
                    ],
                    vec![control_rhs_name.clone()],
                    vec![control_rhs_name],
                )
            }
            PreparedLoopFlowCondition::Compound { op, lhs, rhs } => {
                let lhs_rhs_name = lower_expr(&lhs.rhs, state, bindings)?;
                let rhs_rhs_name = lower_expr(&rhs.rhs, state, bindings)?;
                (
                    vec![
                        render_loop_logic_op(*op).to_owned(),
                        render_loop_cond_kind(&lhs.lhs, lhs.compare),
                        lhs_rhs_name.clone(),
                        render_loop_cond_kind(&rhs.lhs, rhs.compare),
                        rhs_rhs_name.clone(),
                        match prepared.control.action {
                            PreparedLoopFlowAction::Break => "break".to_owned(),
                            PreparedLoopFlowAction::Continue => "continue".to_owned(),
                        },
                    ],
                    vec![lhs_rhs_name.clone(), rhs_rhs_name.clone()],
                    vec![lhs_rhs_name, rhs_rhs_name],
                )
            }
        };
    let has_conditional = prepared
        .carries
        .iter()
        .any(|carry| matches!(carry.kind, PreparedCarryUpdateKind::Conditional { .. }));
    let name = next_name(
        state,
        if has_conditional {
            "loop_while_i64_flow_cond_chain"
        } else {
            "loop_while_i64_flow_chain"
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
                    let carry_kind = render_loop_carry_kind(*op, *source);
                    args.push(carry_kind.clone());
                    args.push(carry_kind);
                    extra_dep_inputs.push(initial_name.clone());
                    extra_effect_inputs.push(initial_name.clone());
                } else {
                    args.push(render_loop_carry_kind(*op, *source));
                }
            }
            PreparedCarryUpdateKind::Conditional {
                condition,
                then_source,
                else_source,
            } => {
                let condition_tag = render_loop_cond_kind(&condition.lhs, condition.compare);
                let rhs_name = lower_expr(&condition.rhs, state, bindings)?;
                args.push(condition_tag);
                args.push(rhs_name.clone());
                let encode_branch_source = |source: &PreparedCarryBranchSource| match source {
                    PreparedCarryBranchSource::Keep => "keep".to_owned(),
                    PreparedCarryBranchSource::Source { op, source } => {
                        render_loop_carry_kind(*op, *source)
                    }
                };
                args.push(encode_branch_source(then_source));
                args.push(encode_branch_source(else_source));
                extra_dep_inputs.push(rhs_name.clone());
                extra_effect_inputs.push(rhs_name);
            }
        }
    }
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: if has_conditional {
                "loop_while_i64_flow_cond_chain".to_owned()
            } else {
                "loop_while_i64_flow_chain".to_owned()
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
    for effect in [&initial_name, &limit_name, &step_name] {
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

pub(super) fn lower_post_flow_while(
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
    let limit_name = lower_expr(&prepared.limit, state, bindings)?;
    let step_name = lower_expr(&prepared.step, state, bindings)?;
    let (control_args, control_dep_inputs, control_effect_inputs) =
        match &prepared.control.condition {
            PreparedLoopFlowCondition::Simple(condition) => {
                let control_rhs_name = lower_expr(&condition.rhs, state, bindings)?;
                (
                    vec![
                        render_loop_cond_kind(&condition.lhs, condition.compare),
                        control_rhs_name.clone(),
                        match prepared.control.action {
                            PreparedLoopFlowAction::Break => "break".to_owned(),
                            PreparedLoopFlowAction::Continue => "continue".to_owned(),
                        },
                    ],
                    vec![control_rhs_name.clone()],
                    vec![control_rhs_name],
                )
            }
            PreparedLoopFlowCondition::Compound { op, lhs, rhs } => {
                let lhs_rhs_name = lower_expr(&lhs.rhs, state, bindings)?;
                let rhs_rhs_name = lower_expr(&rhs.rhs, state, bindings)?;
                (
                    vec![
                        render_loop_logic_op(*op).to_owned(),
                        render_loop_cond_kind(&lhs.lhs, lhs.compare),
                        lhs_rhs_name.clone(),
                        render_loop_cond_kind(&rhs.lhs, rhs.compare),
                        rhs_rhs_name.clone(),
                        match prepared.control.action {
                            PreparedLoopFlowAction::Break => "break".to_owned(),
                            PreparedLoopFlowAction::Continue => "continue".to_owned(),
                        },
                    ],
                    vec![lhs_rhs_name.clone(), rhs_rhs_name.clone()],
                    vec![lhs_rhs_name, rhs_rhs_name],
                )
            }
        };
    let has_conditional = prepared
        .carries
        .iter()
        .any(|carry| matches!(carry.kind, PreparedCarryUpdateKind::Conditional { .. }));
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
    args.extend(control_args);
    let mut extra_dep_inputs: Vec<String> = Vec::new();
    let mut extra_effect_inputs: Vec<String> = Vec::new();
    for (index, carry_initial_name) in carry_initial_names.iter().enumerate() {
        args.push(carry_initial_name.clone());
        match &prepared.carries[index].kind {
            PreparedCarryUpdateKind::Linear { op, source } => {
                let carry_kind = render_loop_carry_kind(*op, *source);
                if has_conditional {
                    args.push("always".to_owned());
                    args.push(initial_name.clone());
                    args.push(carry_kind.clone());
                    args.push(carry_kind);
                    extra_dep_inputs.push(initial_name.clone());
                    extra_effect_inputs.push(initial_name.clone());
                } else {
                    args.push(carry_kind);
                }
            }
            PreparedCarryUpdateKind::Conditional {
                condition,
                then_source,
                else_source,
            } => {
                let condition_tag = render_loop_cond_kind(&condition.lhs, condition.compare);
                let rhs_name = lower_expr(&condition.rhs, state, bindings)?;
                args.push(condition_tag);
                args.push(rhs_name.clone());
                let encode_branch_source = |source: &PreparedCarryBranchSource| match source {
                    PreparedCarryBranchSource::Keep => "keep".to_owned(),
                    PreparedCarryBranchSource::Source { op, source } => {
                        render_loop_carry_kind(*op, *source)
                    }
                };
                args.push(encode_branch_source(then_source));
                args.push(encode_branch_source(else_source));
                extra_dep_inputs.push(rhs_name.clone());
                extra_effect_inputs.push(rhs_name);
            }
        }
    }
    let name = next_name(
        state,
        if has_conditional {
            "loop_while_i64_post_flow_cond_chain"
        } else {
            "loop_while_i64_post_flow_chain"
        },
    );
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: if has_conditional {
                "loop_while_i64_post_flow_cond_chain".to_owned()
            } else {
                "loop_while_i64_post_flow_chain".to_owned()
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
