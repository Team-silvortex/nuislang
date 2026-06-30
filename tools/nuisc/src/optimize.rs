#[path = "optimize_dead_bindings.rs"]
mod optimize_dead_bindings;
#[path = "optimize_expr.rs"]
mod optimize_expr;
#[path = "optimize_expr_helpers.rs"]
mod optimize_expr_helpers;

use self::optimize_dead_bindings::prune_dead_scalar_bindings;
use self::optimize_expr::simplify_expr;
use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{NirAnnotation, NirBinaryOp, NirExpr, NirFunction, NirModule, NirStmt};

pub fn simplify_nir_module(module: &mut NirModule) -> bool {
    let inline_templates = collect_inline_templates(module);
    let mut changed = false;
    for function in &mut module.functions {
        changed |= simplify_nir_function(function, &inline_templates);
    }
    changed
}

fn simplify_nir_function(
    function: &mut NirFunction,
    inline_templates: &BTreeMap<String, InlineTemplate>,
) -> bool {
    let mut env = BTreeMap::new();
    simplify_stmt_block(&mut function.body, &mut env, inline_templates)
}

fn simplify_stmt_block(
    stmts: &mut Vec<NirStmt>,
    env: &mut BTreeMap<String, NirExpr>,
    inline_templates: &BTreeMap<String, InlineTemplate>,
) -> bool {
    let mut changed = false;
    let original = std::mem::take(stmts);
    let mut rewritten = Vec::with_capacity(original.len());
    for stmt in original {
        changed |= rewrite_stmt(stmt, &mut rewritten, env, inline_templates);
    }
    changed |= prune_dead_scalar_bindings(&mut rewritten);
    *stmts = rewritten;
    changed
}

fn rewrite_stmt_block_without_prune(
    stmts: Vec<NirStmt>,
    env: &mut BTreeMap<String, NirExpr>,
    inline_templates: &BTreeMap<String, InlineTemplate>,
) -> (Vec<NirStmt>, bool) {
    let mut changed = false;
    let mut rewritten = Vec::with_capacity(stmts.len());
    for stmt in stmts {
        changed |= rewrite_stmt(stmt, &mut rewritten, env, inline_templates);
    }
    (rewritten, changed)
}

fn rewrite_stmt(
    stmt: NirStmt,
    out: &mut Vec<NirStmt>,
    env: &mut BTreeMap<String, NirExpr>,
    inline_templates: &BTreeMap<String, InlineTemplate>,
) -> bool {
    match stmt {
        NirStmt::Let { name, ty, value } => {
            let (value, changed) =
                simplify_expr(value, env, inline_templates, &mut BTreeSet::new());
            refresh_literal_binding(env, &name, &value);
            out.push(NirStmt::Let { name, ty, value });
            changed
        }
        NirStmt::Const { name, ty, value } => {
            let (value, changed) =
                simplify_expr(value, env, inline_templates, &mut BTreeSet::new());
            refresh_literal_binding(env, &name, &value);
            out.push(NirStmt::Const { name, ty, value });
            changed
        }
        NirStmt::Print(value) => {
            let (value, changed) =
                simplify_expr(value, env, inline_templates, &mut BTreeSet::new());
            out.push(NirStmt::Print(value));
            changed
        }
        NirStmt::Await(value) => {
            let (value, changed) =
                simplify_expr(value, env, inline_templates, &mut BTreeSet::new());
            out.push(NirStmt::Await(value));
            changed
        }
        NirStmt::Expr(value) => {
            let (value, changed) =
                simplify_expr(value, env, inline_templates, &mut BTreeSet::new());
            out.push(NirStmt::Expr(value));
            changed
        }
        NirStmt::Return(value) => {
            let (value, changed) = match value {
                Some(value) => {
                    let (value, changed) =
                        simplify_expr(value, env, inline_templates, &mut BTreeSet::new());
                    (Some(value), changed)
                }
                None => (None, false),
            };
            out.push(NirStmt::Return(value));
            changed
        }
        NirStmt::If {
            condition,
            mut then_body,
            mut else_body,
        } => {
            let (condition, mut changed) =
                simplify_expr(condition, env, inline_templates, &mut BTreeSet::new());
            let mut then_env = env.clone();
            let mut else_env = env.clone();
            if env.is_empty() {
                let original_then = std::mem::take(&mut then_body);
                let original_else = std::mem::take(&mut else_body);
                let (rewritten_then, then_changed) = rewrite_stmt_block_without_prune(
                    original_then,
                    &mut then_env,
                    inline_templates,
                );
                let (rewritten_else, else_changed) = rewrite_stmt_block_without_prune(
                    original_else,
                    &mut else_env,
                    inline_templates,
                );
                then_body = rewritten_then;
                else_body = rewritten_else;
                changed |= then_changed || else_changed;
            } else {
                changed |= simplify_stmt_block(&mut then_body, &mut then_env, inline_templates);
                changed |= simplify_stmt_block(&mut else_body, &mut else_env, inline_templates);
            }
            match condition {
                NirExpr::Bool(true) => {
                    out.extend(then_body);
                    true
                }
                NirExpr::Bool(false) => {
                    out.extend(else_body);
                    true
                }
                other => {
                    out.push(NirStmt::If {
                        condition: other,
                        then_body,
                        else_body,
                    });
                    changed
                }
            }
        }
        NirStmt::While {
            condition,
            mut body,
        } => {
            let loop_input_env = BTreeMap::new();
            let (condition, mut changed) = simplify_expr(
                condition,
                &loop_input_env,
                inline_templates,
                &mut BTreeSet::new(),
            );
            let mut loop_env = BTreeMap::new();
            let original_body = std::mem::take(&mut body);
            let mut rewritten_body = Vec::with_capacity(original_body.len());
            for stmt in original_body {
                changed |= rewrite_stmt(stmt, &mut rewritten_body, &mut loop_env, inline_templates);
            }
            body = rewritten_body;
            out.push(NirStmt::While { condition, body });
            env.clear();
            changed
        }
        NirStmt::Break => {
            out.push(NirStmt::Break);
            false
        }
        NirStmt::Continue => {
            out.push(NirStmt::Continue);
            false
        }
    }
}

#[derive(Clone)]
struct InlineTemplate {
    params: Vec<String>,
    value: NirExpr,
}

fn collect_inline_templates(module: &NirModule) -> BTreeMap<String, InlineTemplate> {
    module
        .functions
        .iter()
        .filter(|function| has_annotation(function, "inline"))
        .filter(|function| !has_annotation(function, "noinline"))
        .filter_map(|function| match function.body.as_slice() {
            [NirStmt::Return(Some(value))]
                if is_inline_safe_expr(
                    value,
                    &function
                        .params
                        .iter()
                        .map(|param| param.name.as_str())
                        .collect::<BTreeSet<_>>(),
                ) =>
            {
                Some((
                    function.name.clone(),
                    InlineTemplate {
                        params: function
                            .params
                            .iter()
                            .map(|param| param.name.clone())
                            .collect(),
                        value: value.clone(),
                    },
                ))
            }
            _ => None,
        })
        .collect()
}

fn has_annotation(function: &NirFunction, name: &str) -> bool {
    function
        .annotations
        .iter()
        .any(|annotation: &NirAnnotation| annotation.name == name)
}

fn is_inline_safe_expr(expr: &NirExpr, params: &BTreeSet<&str>) -> bool {
    match expr {
        NirExpr::Var(name) => params.contains(name.as_str()),
        NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::Null => true,
        NirExpr::StructLiteral { fields, .. } => fields
            .iter()
            .all(|(_, value)| is_inline_safe_expr(value, params)),
        NirExpr::FieldAccess { base, .. } => is_inline_safe_expr(base, params),
        NirExpr::Binary { lhs, rhs, .. } => {
            is_inline_safe_expr(lhs, params) && is_inline_safe_expr(rhs, params)
        }
        _ => false,
    }
}

fn substitute_inline_params(expr: &NirExpr, substitutions: &BTreeMap<String, NirExpr>) -> NirExpr {
    match expr {
        NirExpr::Var(name) => substitutions
            .get(name)
            .cloned()
            .unwrap_or_else(|| NirExpr::Var(name.clone())),
        NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => NirExpr::StructLiteral {
            type_name: type_name.clone(),
            type_args: type_args.clone(),
            fields: fields
                .iter()
                .map(|(name, value)| (name.clone(), substitute_inline_params(value, substitutions)))
                .collect(),
        },
        NirExpr::FieldAccess { base, field } => NirExpr::FieldAccess {
            base: Box::new(substitute_inline_params(base, substitutions)),
            field: field.clone(),
        },
        NirExpr::Binary { op, lhs, rhs } => NirExpr::Binary {
            op: *op,
            lhs: Box::new(substitute_inline_params(lhs, substitutions)),
            rhs: Box::new(substitute_inline_params(rhs, substitutions)),
        },
        other => other.clone(),
    }
}

fn simplify_optional_box_expr(
    value: Option<Box<NirExpr>>,
    env: &BTreeMap<String, NirExpr>,
    inline_templates: &BTreeMap<String, InlineTemplate>,
    active_inline: &mut BTreeSet<String>,
) -> (Option<Box<NirExpr>>, bool) {
    match value {
        Some(value) => {
            let (value, changed) = simplify_expr(*value, env, inline_templates, active_inline);
            (Some(Box::new(value)), changed)
        }
        None => (None, false),
    }
}

fn refresh_literal_binding(env: &mut BTreeMap<String, NirExpr>, name: &str, value: &NirExpr) {
    match literal_binding_value(value) {
        Some(value) => {
            env.insert(name.to_owned(), value);
        }
        None => {
            env.remove(name);
        }
    }
}

fn literal_binding_value(value: &NirExpr) -> Option<NirExpr> {
    match value {
        NirExpr::Bool(_)
        | NirExpr::Text(_)
        | NirExpr::Int(_)
        | NirExpr::F32(_)
        | NirExpr::F64(_)
        | NirExpr::Null => Some(value.clone()),
        _ => None,
    }
}

fn fold_int_binary(op: NirBinaryOp, lhs: i64, rhs: i64) -> Option<i64> {
    match op {
        NirBinaryOp::And => Some(((lhs != 0) && (rhs != 0)) as i64),
        NirBinaryOp::Or => Some(((lhs != 0) || (rhs != 0)) as i64),
        NirBinaryOp::Add => Some(lhs + rhs),
        NirBinaryOp::Sub => Some(lhs - rhs),
        NirBinaryOp::Mul => Some(lhs * rhs),
        NirBinaryOp::Div => (rhs != 0).then_some(lhs / rhs),
        NirBinaryOp::Rem => (rhs != 0).then_some(lhs % rhs),
        NirBinaryOp::Eq => Some((lhs == rhs) as i64),
        NirBinaryOp::Ne => Some((lhs != rhs) as i64),
        NirBinaryOp::Lt => Some((lhs < rhs) as i64),
        NirBinaryOp::Le => Some((lhs <= rhs) as i64),
        NirBinaryOp::Gt => Some((lhs > rhs) as i64),
        NirBinaryOp::Ge => Some((lhs >= rhs) as i64),
    }
}

#[cfg(test)]
#[path = "optimize_tests.rs"]
mod tests;
