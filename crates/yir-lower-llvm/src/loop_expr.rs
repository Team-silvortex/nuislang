use std::collections::BTreeMap;

use super::{value_ref::coerce_to_i64, LlvmValueRef};

#[derive(Clone)]
pub(crate) enum LoopControlExpr {
    Cond { kind: String, rhs_name: String },
    And(Box<LoopControlExpr>, Box<LoopControlExpr>),
    Or(Box<LoopControlExpr>, Box<LoopControlExpr>),
}
#[derive(Clone)]
pub(crate) enum ResolvedLoopControlExpr {
    Cond { kind: String, rhs: String },
    And(Box<ResolvedLoopControlExpr>, Box<ResolvedLoopControlExpr>),
    Or(Box<ResolvedLoopControlExpr>, Box<ResolvedLoopControlExpr>),
}

#[derive(Clone)]
pub(crate) enum LoopFlowExpr {
    Legacy {
        condition: LoopControlExpr,
        action: String,
    },
    Terminal {
        action: String,
        condition: LoopControlExpr,
    },
    Or(Box<LoopFlowExpr>, Box<LoopFlowExpr>),
}

#[derive(Clone)]
pub(crate) enum ResolvedLoopFlowExpr {
    Legacy {
        condition: ResolvedLoopControlExpr,
        action: String,
    },
    Terminal {
        action: String,
        condition: ResolvedLoopControlExpr,
    },
    Or(Box<ResolvedLoopFlowExpr>, Box<ResolvedLoopFlowExpr>),
}

pub(crate) fn parse_loop_control_expr_for_llvm(
    args: &[String],
    start: usize,
    node_name: &str,
    instruction_name: &str,
) -> Result<(LoopControlExpr, usize), String> {
    let Some(token) = args.get(start).map(String::as_str) else {
        return Err(format!(
            "cpu.{instruction_name} `{}` is missing control metadata during LLVM lowering",
            node_name
        ));
    };
    if token == "and" {
        let (lhs, after_lhs) =
            parse_loop_control_expr_for_llvm(args, start + 1, node_name, instruction_name)?;
        let (rhs, after_rhs) =
            parse_loop_control_expr_for_llvm(args, after_lhs, node_name, instruction_name)?;
        Ok((
            LoopControlExpr::And(Box::new(lhs), Box::new(rhs)),
            after_rhs,
        ))
    } else if token == "or" {
        let (lhs, after_lhs) =
            parse_loop_control_expr_for_llvm(args, start + 1, node_name, instruction_name)?;
        let (rhs, after_rhs) =
            parse_loop_control_expr_for_llvm(args, after_lhs, node_name, instruction_name)?;
        Ok((LoopControlExpr::Or(Box::new(lhs), Box::new(rhs)), after_rhs))
    } else {
        let Some(rhs_name) = args.get(start + 1) else {
            return Err(format!(
                "cpu.{instruction_name} `{}` is missing control rhs during LLVM lowering",
                node_name
            ));
        };
        Ok((
            LoopControlExpr::Cond {
                kind: token.to_owned(),
                rhs_name: rhs_name.clone(),
            },
            start + 2,
        ))
    }
}

pub(crate) fn parse_loop_flow_expr_for_llvm(
    args: &[String],
    start: usize,
    node_name: &str,
    instruction_name: &str,
) -> Result<(LoopFlowExpr, usize), String> {
    let Some(token) = args.get(start).map(String::as_str) else {
        return Err(format!(
            "cpu.{instruction_name} `{}` is missing control metadata during LLVM lowering",
            node_name
        ));
    };
    match token {
        "flow_or" => {
            let (lhs, after_lhs) =
                parse_loop_flow_expr_for_llvm(args, start + 1, node_name, instruction_name)?;
            let (rhs, after_rhs) =
                parse_loop_flow_expr_for_llvm(args, after_lhs, node_name, instruction_name)?;
            Ok((LoopFlowExpr::Or(Box::new(lhs), Box::new(rhs)), after_rhs))
        }
        "flow_break" | "flow_continue" => {
            let (condition, after_condition) =
                parse_loop_control_expr_for_llvm(args, start + 1, node_name, instruction_name)?;
            Ok((
                LoopFlowExpr::Terminal {
                    action: token.trim_start_matches("flow_").to_owned(),
                    condition,
                },
                after_condition,
            ))
        }
        "flow_and" => Err(format!(
            "cpu.{instruction_name} `{}` does not support `flow_and` during LLVM lowering yet",
            node_name
        )),
        _ => {
            let (condition, action_index) =
                parse_loop_control_expr_for_llvm(args, start, node_name, instruction_name)?;
            let Some(action) = args.get(action_index) else {
                return Err(format!(
                    "cpu.{instruction_name} `{}` is missing control action during LLVM lowering",
                    node_name
                ));
            };
            Ok((
                LoopFlowExpr::Legacy {
                    condition,
                    action: action.clone(),
                },
                action_index + 1,
            ))
        }
    }
}

pub(crate) fn resolve_loop_control_expr_for_llvm(
    expr: &LoopControlExpr,
    registers: &BTreeMap<String, LlvmValueRef>,
    body: &mut Vec<String>,
    next_reg: &mut usize,
    node_name: &str,
    instruction_name: &str,
) -> Option<ResolvedLoopControlExpr> {
    match expr {
        LoopControlExpr::Cond { kind, rhs_name } => {
            let Some(value) = registers.get(rhs_name).cloned() else {
                body.push(format!("  ; deferred lowering for cpu.{instruction_name} `{}` because one or more control rhs values are outside the current CPU LLVM slice", node_name));
                return None;
            };
            let Some(rhs) = coerce_to_i64(&value, body, next_reg) else {
                body.push(format!("  ; deferred lowering for cpu.{instruction_name} `{}` because one or more control rhs values are not coercible to i64", node_name));
                return None;
            };
            Some(ResolvedLoopControlExpr::Cond {
                kind: kind.clone(),
                rhs,
            })
        }
        LoopControlExpr::And(lhs, rhs) => Some(ResolvedLoopControlExpr::And(
            Box::new(resolve_loop_control_expr_for_llvm(
                lhs,
                registers,
                body,
                next_reg,
                node_name,
                instruction_name,
            )?),
            Box::new(resolve_loop_control_expr_for_llvm(
                rhs,
                registers,
                body,
                next_reg,
                node_name,
                instruction_name,
            )?),
        )),
        LoopControlExpr::Or(lhs, rhs) => Some(ResolvedLoopControlExpr::Or(
            Box::new(resolve_loop_control_expr_for_llvm(
                lhs,
                registers,
                body,
                next_reg,
                node_name,
                instruction_name,
            )?),
            Box::new(resolve_loop_control_expr_for_llvm(
                rhs,
                registers,
                body,
                next_reg,
                node_name,
                instruction_name,
            )?),
        )),
    }
}

pub(crate) fn resolve_loop_flow_expr_for_llvm(
    expr: &LoopFlowExpr,
    registers: &BTreeMap<String, LlvmValueRef>,
    body: &mut Vec<String>,
    next_reg: &mut usize,
    node_name: &str,
    instruction_name: &str,
) -> Option<ResolvedLoopFlowExpr> {
    match expr {
        LoopFlowExpr::Legacy { condition, action } => Some(ResolvedLoopFlowExpr::Legacy {
            condition: resolve_loop_control_expr_for_llvm(
                condition,
                registers,
                body,
                next_reg,
                node_name,
                instruction_name,
            )?,
            action: action.clone(),
        }),
        LoopFlowExpr::Terminal { action, condition } => Some(ResolvedLoopFlowExpr::Terminal {
            action: action.clone(),
            condition: resolve_loop_control_expr_for_llvm(
                condition,
                registers,
                body,
                next_reg,
                node_name,
                instruction_name,
            )?,
        }),
        LoopFlowExpr::Or(lhs, rhs) => Some(ResolvedLoopFlowExpr::Or(
            Box::new(resolve_loop_flow_expr_for_llvm(
                lhs,
                registers,
                body,
                next_reg,
                node_name,
                instruction_name,
            )?),
            Box::new(resolve_loop_flow_expr_for_llvm(
                rhs,
                registers,
                body,
                next_reg,
                node_name,
                instruction_name,
            )?),
        )),
    }
}

pub(crate) fn collect_resolved_loop_flow_leaves<'a>(
    expr: &'a ResolvedLoopFlowExpr,
    leaves: &mut Vec<(&'a ResolvedLoopControlExpr, &'a str)>,
) {
    match expr {
        ResolvedLoopFlowExpr::Legacy { condition, action }
        | ResolvedLoopFlowExpr::Terminal { condition, action } => {
            leaves.push((condition, action.as_str()));
        }
        ResolvedLoopFlowExpr::Or(lhs, rhs) => {
            collect_resolved_loop_flow_leaves(lhs, leaves);
            collect_resolved_loop_flow_leaves(rhs, leaves);
        }
    }
}

pub(crate) fn canonical_loop_instruction(instruction: &str) -> &str {
    match instruction {
        "loop_while_i64_chain" | "loop_while_scalar_chain" => "loop_while_scalar_chain",
        "loop_while_i64_async_chain" | "loop_while_scalar_async_chain" => {
            "loop_while_scalar_async_chain"
        }
        "loop_while_i64_async_cond_chain" | "loop_while_scalar_async_cond_chain" => {
            "loop_while_scalar_async_cond_chain"
        }
        "loop_while_i64_cond_chain" | "loop_while_scalar_cond_chain" => {
            "loop_while_scalar_cond_chain"
        }
        "loop_while_i64_flow_chain" | "loop_while_scalar_flow_chain" => {
            "loop_while_scalar_flow_chain"
        }
        "loop_while_i64_async_flow_chain" | "loop_while_scalar_async_flow_chain" => {
            "loop_while_scalar_async_flow_chain"
        }
        "loop_while_i64_flow_cond_chain" | "loop_while_scalar_flow_cond_chain" => {
            "loop_while_scalar_flow_cond_chain"
        }
        "loop_while_i64_async_flow_cond_chain" | "loop_while_scalar_async_flow_cond_chain" => {
            "loop_while_scalar_async_flow_cond_chain"
        }
        "loop_while_i64_post_flow_chain" | "loop_while_scalar_post_flow_chain" => {
            "loop_while_scalar_post_flow_chain"
        }
        "loop_while_i64_async_post_flow_chain" | "loop_while_scalar_async_post_flow_chain" => {
            "loop_while_scalar_async_post_flow_chain"
        }
        "loop_while_i64_post_flow_cond_chain" | "loop_while_scalar_post_flow_cond_chain" => {
            "loop_while_scalar_post_flow_cond_chain"
        }
        "loop_while_i64_async_post_flow_cond_chain"
        | "loop_while_scalar_async_post_flow_cond_chain" => {
            "loop_while_scalar_async_post_flow_cond_chain"
        }
        other => other,
    }
}

pub(crate) fn canonical_loop_block_prefix(instruction: &str) -> String {
    canonical_loop_instruction(instruction).replace('.', "_")
}
