// Loop and carry metadata parsing helpers for the CPU domain.

use crate::carry_payload::carry_source_payload_len;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LoopCondExpr {
    Leaf {
        kind: String,
        rhs: Option<String>,
    },
    Binary {
        op: String,
        lhs: Box<LoopCondExpr>,
        rhs: Box<LoopCondExpr>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LoopFlowExpr {
    Legacy {
        condition: LoopCondExpr,
        action: String,
    },
    Terminal {
        action: String,
        condition: LoopCondExpr,
    },
    Binary {
        op: String,
        lhs: Box<LoopFlowExpr>,
        rhs: Box<LoopFlowExpr>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ParsedConditionalCarry {
    pub(crate) initial: String,
    pub(crate) condition: LoopCondExpr,
    pub(crate) then_source: ParsedCarryBranchSource,
    pub(crate) else_source: ParsedCarryBranchSource,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ParsedCarryBranchSource {
    pub(crate) kind: String,
    pub(crate) payload: Vec<String>,
}
pub(crate) fn validate_loop_compare_kind(kind: &str, node_name: &str) -> Result<(), String> {
    match kind {
        "eq" | "ne" | "lt" | "le" | "gt" | "ge" => Ok(()),
        other => Err(format!(
            "node `{}` has invalid loop compare kind `{}`",
            node_name, other
        )),
    }
}
pub(crate) fn validate_loop_step_kind(kind: &str, node_name: &str) -> Result<(), String> {
    match kind {
        "add" | "sub" => Ok(()),
        other => Err(format!(
            "node `{}` has invalid loop step kind `{}`",
            node_name, other
        )),
    }
}
pub(crate) fn validate_indexed_compare_kind(
    kind: &str,
    node_name: &str,
    label: &str,
    prefixes: &[&str],
) -> Result<(), String> {
    let suffixes = ["_eq", "_ne", "_lt", "_le", "_gt", "_ge"];
    for prefix in prefixes {
        if let Some(rest) = kind.strip_prefix(prefix) {
            for suffix in suffixes {
                if let Some(index) = rest.strip_suffix(suffix) {
                    return index.parse::<usize>().map(|_| ()).map_err(|_| {
                        format!("node `{}` has invalid {} `{}`", node_name, label, kind)
                    });
                }
            }
        }
    }
    Err(format!(
        "node `{}` has invalid {} `{}`",
        node_name, label, kind
    ))
}
pub(crate) fn validate_flow_control_kind(kind: &str, node_name: &str) -> Result<(), String> {
    match kind {
        "current_eq" | "current_ne" | "current_lt" | "current_le" | "current_gt" | "current_ge" => {
            Ok(())
        }
        other => validate_indexed_compare_kind(other, node_name, "flow control kind", &["carry"]),
    }
}
pub(crate) fn validate_carry_condition_kind(
    kind: &str,
    node_name: &str,
    allow_prev: bool,
) -> Result<(), String> {
    match kind {
        "always" | "current_eq" | "current_ne" | "current_lt" | "current_le" | "current_gt"
        | "current_ge" => Ok(()),
        "prev_current_eq" | "prev_current_ne" | "prev_current_lt" | "prev_current_le"
        | "prev_current_gt" | "prev_current_ge"
            if allow_prev =>
        {
            Ok(())
        }
        other if allow_prev => validate_indexed_compare_kind(
            other,
            node_name,
            "conditional carry kind",
            &["prev_carry", "carry"],
        ),
        other => {
            validate_indexed_compare_kind(other, node_name, "conditional carry kind", &["carry"])
        }
    }
}

pub(crate) fn parse_carry_branch_source(
    args: &[String],
    start: usize,
    node_name: &str,
) -> Result<(ParsedCarryBranchSource, usize), String> {
    let Some(kind) = args.get(start).cloned() else {
        return Err(format!("node `{}` is missing carry kind", node_name));
    };
    let Some(payload_len) = carry_source_payload_len(&kind) else {
        return Err(format!(
            "node `{}` has invalid carry kind `{}`",
            node_name, kind
        ));
    };
    let end = start + 1 + payload_len;
    if end > args.len() {
        return Err(format!(
            "node `{}` is missing carry payload for `{}`",
            node_name, kind
        ));
    }
    Ok((
        ParsedCarryBranchSource {
            kind,
            payload: args[start + 1..end].to_vec(),
        },
        end,
    ))
}

pub(crate) fn collect_carry_branch_source_inputs(
    source: &ParsedCarryBranchSource,
    inputs: &mut Vec<String>,
) {
    inputs.extend(source.payload.iter().cloned());
}

pub(crate) fn parse_loop_control_expr<F>(
    args: &[String],
    start: usize,
    node_name: &str,
    validate_kind: &F,
) -> Result<(LoopCondExpr, usize), String>
where
    F: Fn(&str, &str) -> Result<(), String>,
{
    let Some(token) = args.get(start).map(String::as_str) else {
        return Err(format!(
            "node `{}` is missing flow control metadata",
            node_name
        ));
    };
    if token == "and" || token == "or" {
        let (lhs, after_lhs) = parse_loop_control_expr(args, start + 1, node_name, validate_kind)?;
        let (rhs, after_rhs) = parse_loop_control_expr(args, after_lhs, node_name, validate_kind)?;
        Ok((
            LoopCondExpr::Binary {
                op: token.to_owned(),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            after_rhs,
        ))
    } else {
        validate_kind(token, node_name)?;
        let Some(rhs) = args.get(start + 1) else {
            return Err(format!("node `{}` is missing flow control rhs", node_name));
        };
        Ok((
            LoopCondExpr::Leaf {
                kind: token.to_owned(),
                rhs: Some(rhs.clone()),
            },
            start + 2,
        ))
    }
}

pub(crate) fn parse_loop_flow_expr<F>(
    args: &[String],
    start: usize,
    node_name: &str,
    validate_kind: &F,
) -> Result<(LoopFlowExpr, usize), String>
where
    F: Fn(&str, &str) -> Result<(), String>,
{
    let Some(token) = args.get(start).map(String::as_str) else {
        return Err(format!(
            "node `{}` is missing flow control metadata",
            node_name
        ));
    };
    match token {
        "flow_and" | "flow_or" => {
            let (lhs, after_lhs) = parse_loop_flow_expr(args, start + 1, node_name, validate_kind)?;
            let (rhs, after_rhs) = parse_loop_flow_expr(args, after_lhs, node_name, validate_kind)?;
            Ok((
                LoopFlowExpr::Binary {
                    op: token.to_owned(),
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                after_rhs,
            ))
        }
        "flow_break" | "flow_continue" => {
            let (condition, after_condition) =
                parse_loop_control_expr(args, start + 1, node_name, validate_kind)?;
            Ok((
                LoopFlowExpr::Terminal {
                    action: token.trim_start_matches("flow_").to_owned(),
                    condition,
                },
                after_condition,
            ))
        }
        _ => {
            let (condition, action_index) =
                parse_loop_control_expr(args, start, node_name, validate_kind)?;
            let Some(action) = args.get(action_index) else {
                return Err(format!(
                    "node `{}` is missing flow control action",
                    node_name
                ));
            };
            match action.as_str() {
                "break" | "continue" => Ok((
                    LoopFlowExpr::Legacy {
                        condition,
                        action: action.clone(),
                    },
                    action_index + 1,
                )),
                other => Err(format!(
                    "node `{}` has invalid flow control action `{}`",
                    node_name, other
                )),
            }
        }
    }
}

pub(crate) fn parse_loop_carry_condition_expr(
    args: &[String],
    start: usize,
    node_name: &str,
    allow_prev: bool,
) -> Result<(LoopCondExpr, usize), String> {
    let Some(token) = args.get(start).map(String::as_str) else {
        return Err(format!(
            "node `{}` is missing conditional carry metadata",
            node_name
        ));
    };
    if token == "and" || token == "or" {
        let (lhs, after_lhs) =
            parse_loop_carry_condition_expr(args, start + 1, node_name, allow_prev)?;
        let (rhs, after_rhs) =
            parse_loop_carry_condition_expr(args, after_lhs, node_name, allow_prev)?;
        Ok((
            LoopCondExpr::Binary {
                op: token.to_owned(),
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            },
            after_rhs,
        ))
    } else {
        validate_carry_condition_kind(token, node_name, allow_prev)?;
        if token == "always" {
            Ok((
                LoopCondExpr::Leaf {
                    kind: token.to_owned(),
                    rhs: None,
                },
                start + 1,
            ))
        } else {
            let Some(rhs) = args.get(start + 1) else {
                return Err(format!(
                    "node `{}` is missing conditional carry rhs",
                    node_name
                ));
            };
            Ok((
                LoopCondExpr::Leaf {
                    kind: token.to_owned(),
                    rhs: Some(rhs.clone()),
                },
                start + 2,
            ))
        }
    }
}

pub(crate) fn parse_conditional_carries(
    args: &[String],
    start: usize,
    node_name: &str,
    allow_prev: bool,
) -> Result<Vec<ParsedConditionalCarry>, String> {
    if start >= args.len() {
        return Ok(Vec::new());
    }
    let mut index = start;
    let mut carries = Vec::new();
    while index < args.len() {
        let Some(initial) = args.get(index) else {
            return Err(format!("node `{}` is missing carry initial", node_name));
        };
        let (condition, after_condition) =
            parse_loop_carry_condition_expr(args, index + 1, node_name, allow_prev)?;
        let branch_start = match &condition {
            LoopCondExpr::Leaf { kind, rhs: None } if kind == "always" => {
                match parse_carry_branch_source(args, after_condition, node_name) {
                    Ok((_, _)) => after_condition,
                    Err(_) if after_condition + 1 < args.len() => after_condition + 1,
                    Err(error) => return Err(error),
                }
            }
            _ => after_condition,
        };
        let (then_source, after_then) = parse_carry_branch_source(args, branch_start, node_name)?;
        let (else_source, after_else) = parse_carry_branch_source(args, after_then, node_name)?;
        carries.push(ParsedConditionalCarry {
            initial: initial.clone(),
            condition,
            then_source,
            else_source,
        });
        index = after_else;
    }
    Ok(carries)
}

pub(crate) fn collect_loop_condition_rhs_inputs(expr: &LoopCondExpr, inputs: &mut Vec<String>) {
    match expr {
        LoopCondExpr::Leaf { rhs, .. } => {
            if let Some(rhs) = rhs {
                inputs.push(rhs.clone());
            }
        }
        LoopCondExpr::Binary { lhs, rhs, .. } => {
            collect_loop_condition_rhs_inputs(lhs, inputs);
            collect_loop_condition_rhs_inputs(rhs, inputs);
        }
    }
}

pub(crate) fn collect_loop_flow_rhs_inputs(expr: &LoopFlowExpr, inputs: &mut Vec<String>) {
    match expr {
        LoopFlowExpr::Legacy { condition, .. } | LoopFlowExpr::Terminal { condition, .. } => {
            collect_loop_condition_rhs_inputs(condition, inputs);
        }
        LoopFlowExpr::Binary { lhs, rhs, .. } => {
            collect_loop_flow_rhs_inputs(lhs, inputs);
            collect_loop_flow_rhs_inputs(rhs, inputs);
        }
    }
}

pub(crate) fn format_loop_condition_expr<F>(
    expr: &LoopCondExpr,
    resolve_rhs: &F,
) -> Result<String, String>
where
    F: Fn(&str) -> Result<String, String>,
{
    match expr {
        LoopCondExpr::Leaf { kind, rhs } => match rhs {
            Some(rhs) => Ok(format!("{} {}", kind, resolve_rhs(rhs)?)),
            None => Ok(kind.clone()),
        },
        LoopCondExpr::Binary { op, lhs, rhs } => Ok(format!(
            "({} {} {})",
            format_loop_condition_expr(lhs, resolve_rhs)?,
            op,
            format_loop_condition_expr(rhs, resolve_rhs)?
        )),
    }
}

pub(crate) fn format_loop_flow_expr<F>(
    expr: &LoopFlowExpr,
    resolve_rhs: &F,
) -> Result<String, String>
where
    F: Fn(&str) -> Result<String, String>,
{
    match expr {
        LoopFlowExpr::Legacy { condition, action }
        | LoopFlowExpr::Terminal { action, condition } => Ok(format!(
            "if {} then {}",
            format_loop_condition_expr(condition, resolve_rhs)?,
            action
        )),
        LoopFlowExpr::Binary { op, lhs, rhs } => Ok(format!(
            "({} {} {})",
            format_loop_flow_expr(lhs, resolve_rhs)?,
            op,
            format_loop_flow_expr(rhs, resolve_rhs)?
        )),
    }
}

pub(crate) fn format_conditional_carry<F>(
    carry: &ParsedConditionalCarry,
    resolve_value: &F,
) -> Result<String, String>
where
    F: Fn(&str) -> Result<String, String>,
{
    Ok(format!(
        "{}:{} ? {} : {}",
        resolve_value(&carry.initial)?,
        format_loop_condition_expr(&carry.condition, resolve_value)?,
        if carry.then_source.payload.is_empty() {
            carry.then_source.kind.clone()
        } else {
            format!(
                "{}({})",
                carry.then_source.kind,
                carry
                    .then_source
                    .payload
                    .iter()
                    .map(|value_name| resolve_value(value_name))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ")
            )
        },
        if carry.else_source.payload.is_empty() {
            carry.else_source.kind.clone()
        } else {
            format!(
                "{}({})",
                carry.else_source.kind,
                carry
                    .else_source
                    .payload
                    .iter()
                    .map(|value_name| resolve_value(value_name))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ")
            )
        }
    ))
}
