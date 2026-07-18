use super::*;
use crate::lowering::edge_helpers::push_effect_edge;

fn flatten_uniform_loop_flow_control(
    control: &PreparedLoopFlowControl,
) -> Option<(PreparedLoopFlowCondition, PreparedLoopFlowAction)> {
    match control {
        PreparedLoopFlowControl::Terminal { condition, action } => {
            Some((condition.clone(), *action))
        }
        PreparedLoopFlowControl::Compound { op, lhs, rhs } => {
            let (lhs_condition, lhs_action) = flatten_uniform_loop_flow_control(lhs)?;
            let (rhs_condition, rhs_action) = flatten_uniform_loop_flow_control(rhs)?;
            if lhs_action != rhs_action {
                return None;
            }
            Some((
                PreparedLoopFlowCondition::Compound {
                    op: *op,
                    lhs: Box::new(lhs_condition),
                    rhs: Box::new(rhs_condition),
                },
                lhs_action,
            ))
        }
    }
}

fn encode_mixed_loop_flow_control_args(
    control: &PreparedLoopFlowControl,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<EncodedLoopArgs, String> {
    match control {
        PreparedLoopFlowControl::Terminal { condition, action } => {
            let (mut condition_args, dep_inputs, effect_inputs) =
                encode_carry_condition_args(condition, state, bindings)?;
            let mut args = vec![match action {
                PreparedLoopFlowAction::Break => "flow_break".to_owned(),
                PreparedLoopFlowAction::Continue => "flow_continue".to_owned(),
            }];
            args.append(&mut condition_args);
            Ok((args, dep_inputs, effect_inputs))
        }
        PreparedLoopFlowControl::Compound { op, lhs, rhs } => {
            let (mut lhs_args, mut lhs_dep_inputs, mut lhs_effect_inputs) =
                encode_mixed_loop_flow_control_args(lhs, state, bindings)?;
            let (rhs_args, rhs_dep_inputs, rhs_effect_inputs) =
                encode_mixed_loop_flow_control_args(rhs, state, bindings)?;
            let mut args = vec![match op {
                PreparedLoopLogicOp::And => "flow_and".to_owned(),
                PreparedLoopLogicOp::Or => "flow_or".to_owned(),
            }];
            args.append(&mut lhs_args);
            args.extend(rhs_args);
            lhs_dep_inputs.extend(rhs_dep_inputs);
            lhs_effect_inputs.extend(rhs_effect_inputs);
            Ok((args, lhs_dep_inputs, lhs_effect_inputs))
        }
    }
}

pub(super) fn encode_loop_flow_control_args(
    control: &PreparedLoopFlowControl,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<EncodedLoopControlArgs, String> {
    let Some((condition, action)) = flatten_uniform_loop_flow_control(control) else {
        let (args, dep_inputs, effect_inputs) =
            encode_mixed_loop_flow_control_args(control, state, bindings)?;
        return Ok((args, dep_inputs, effect_inputs, true));
    };
    match &condition {
        PreparedLoopFlowCondition::Simple(condition) => {
            let control_rhs_name = lower_expr(&condition.rhs, state, bindings)?;
            Ok((
                vec![
                    render_loop_cond_kind(&condition.lhs, condition.compare),
                    control_rhs_name.clone(),
                    match action {
                        PreparedLoopFlowAction::Break => "break".to_owned(),
                        PreparedLoopFlowAction::Continue => "continue".to_owned(),
                    },
                ],
                vec![control_rhs_name.clone()],
                vec![control_rhs_name],
                false,
            ))
        }
        PreparedLoopFlowCondition::Compound { op, lhs, rhs } => {
            let (mut condition_args, dep_inputs, effect_inputs) = encode_carry_condition_args(
                &PreparedLoopFlowCondition::Compound {
                    op: *op,
                    lhs: lhs.clone(),
                    rhs: rhs.clone(),
                },
                state,
                bindings,
            )?;
            condition_args.push(match action {
                PreparedLoopFlowAction::Break => "break".to_owned(),
                PreparedLoopFlowAction::Continue => "continue".to_owned(),
            });
            Ok((condition_args, dep_inputs, effect_inputs, false))
        }
    }
}

fn encode_carry_condition_args(
    condition: &PreparedLoopFlowCondition,
    state: &mut LoweringState<'_>,
    bindings: &mut BTreeMap<String, String>,
) -> Result<EncodedLoopArgs, String> {
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

#[path = "loop_flow_nodes_async.rs"]
mod loop_flow_nodes_async;
#[path = "loop_flow_nodes_async_post.rs"]
mod loop_flow_nodes_async_post;
#[path = "loop_flow_nodes_flow.rs"]
mod loop_flow_nodes_flow;
#[path = "loop_flow_nodes_post.rs"]
mod loop_flow_nodes_post;

pub(super) use loop_flow_nodes_async::lower_async_flow_while;
pub(super) use loop_flow_nodes_async_post::lower_async_post_flow_while;
pub(super) use loop_flow_nodes_flow::lower_flow_while;
pub(super) use loop_flow_nodes_post::lower_post_flow_while;
