use std::collections::BTreeMap;

use nuis_semantics::model::{AstDestructureBinding, AstDestructureField};

use super::{AstExpr, AstModule, AstStmt};
use nuis_semantics::model::{AstFunction, AstImplMethod};

pub(super) fn validate_ast_assignments(module: &AstModule) -> Result<(), String> {
    for function in &module.functions {
        validate_function(function)?;
    }
    for implementation in &module.impls {
        for method in &implementation.methods {
            validate_impl_method(method)?;
        }
    }
    Ok(())
}

fn validate_function(function: &AstFunction) -> Result<(), String> {
    let mut env = BTreeMap::new();
    for param in &function.params {
        env.insert(param.name.clone(), false);
    }
    validate_stmt_block(
        &function.body,
        &mut env,
        &format!("function `{}` body", function.name),
    )
}

fn validate_impl_method(method: &AstImplMethod) -> Result<(), String> {
    let mut env = BTreeMap::new();
    for param in &method.params {
        env.insert(param.name.clone(), false);
    }
    validate_stmt_block(
        &method.body,
        &mut env,
        &format!("impl method `{}` body", method.name),
    )
}

fn validate_stmt_block(
    body: &[AstStmt],
    env: &mut BTreeMap<String, bool>,
    context: &str,
) -> Result<(), String> {
    for stmt in body {
        validate_stmt(stmt, env, context)?;
    }
    Ok(())
}

fn validate_stmt(
    stmt: &AstStmt,
    env: &mut BTreeMap<String, bool>,
    context: &str,
) -> Result<(), String> {
    match stmt {
        AstStmt::Let {
            mutable,
            name,
            value,
            ..
        } => {
            validate_expr(value, env, context)?;
            env.insert(name.clone(), *mutable);
        }
        AstStmt::AssignLocal { name, value } => {
            validate_expr(value, env, context)?;
            match env.get(name) {
                Some(true) => {}
                Some(false) => {
                    return Err(format!(
                        "{context} cannot assign to immutable local `{name}`; declare it with `let mut` first"
                    ))
                }
                None => {
                    return Err(format!(
                        "{context} cannot assign to unknown local `{name}`"
                    ))
                }
            }
        }
        AstStmt::DestructureLet { fields, value, .. } => {
            validate_expr(value, env, context)?;
            let mut names = Vec::new();
            collect_destructure_binding_names(fields, &mut names);
            for name in names {
                env.insert(name, false);
            }
        }
        AstStmt::Const { name, value, .. } => {
            validate_expr(value, env, context)?;
            env.insert(name.clone(), false);
        }
        AstStmt::Print(value) | AstStmt::Await(value) | AstStmt::Expr(value) => {
            validate_expr(value, env, context)?;
        }
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            validate_expr(condition, env, context)?;
            let mut then_env = env.clone();
            validate_stmt_block(then_body, &mut then_env, context)?;
            let mut else_env = env.clone();
            validate_stmt_block(else_body, &mut else_env, context)?;
        }
        AstStmt::Match { value, arms } => {
            validate_expr(value, env, context)?;
            for arm in arms {
                let mut arm_env = env.clone();
                if let Some(guard) = &arm.guard {
                    validate_expr(guard, &arm_env, context)?;
                }
                validate_stmt_block(&arm.body, &mut arm_env, context)?;
            }
        }
        AstStmt::While { condition, body } => {
            validate_expr(condition, env, context)?;
            let mut loop_env = env.clone();
            validate_stmt_block(body, &mut loop_env, context)?;
        }
        AstStmt::Return(Some(value)) => validate_expr(value, env, context)?,
        AstStmt::Break | AstStmt::Continue | AstStmt::Return(None) => {}
    }
    Ok(())
}

fn validate_expr(
    expr: &AstExpr,
    env: &BTreeMap<String, bool>,
    context: &str,
) -> Result<(), String> {
    match expr {
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => {
            validate_expr(condition, env, context)?;
            let mut then_env = env.clone();
            validate_stmt_block(then_body, &mut then_env, context)?;
            let mut else_env = env.clone();
            validate_stmt_block(else_body, &mut else_env, context)?;
        }
        AstExpr::Match { value, arms } => {
            validate_expr(value, env, context)?;
            for arm in arms {
                let mut arm_env = env.clone();
                if let Some(guard) = &arm.guard {
                    validate_expr(guard, &arm_env, context)?;
                }
                validate_stmt_block(&arm.body, &mut arm_env, context)?;
            }
        }
        AstExpr::Lambda { params, body, .. } => {
            let mut lambda_env = BTreeMap::new();
            for param in params {
                lambda_env.insert(param.name.clone(), false);
            }
            validate_stmt_block(body, &mut lambda_env, "lambda body")?;
        }
        AstExpr::Await(value)
        | AstExpr::Try(value)
        | AstExpr::Unary { operand: value, .. }
        | AstExpr::FieldAccess { base: value, .. } => validate_expr(value, env, context)?,
        AstExpr::Call { args, .. } | AstExpr::Invoke { args, .. } => {
            for arg in args {
                validate_expr(arg, env, context)?;
            }
            if let AstExpr::Invoke { callee, .. } = expr {
                validate_expr(callee, env, context)?;
            }
        }
        AstExpr::MethodCall { receiver, args, .. } => {
            validate_expr(receiver, env, context)?;
            for arg in args {
                validate_expr(arg, env, context)?;
            }
        }
        AstExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                validate_expr(value, env, context)?;
            }
        }
        AstExpr::Binary { lhs, rhs, .. } => {
            validate_expr(lhs, env, context)?;
            validate_expr(rhs, env, context)?;
        }
        AstExpr::Bool(_)
        | AstExpr::Text(_)
        | AstExpr::Int(_)
        | AstExpr::Float(_)
        | AstExpr::Var(_)
        | AstExpr::Instantiate { .. } => {}
    }
    Ok(())
}

fn collect_destructure_binding_names(fields: &[AstDestructureField], names: &mut Vec<String>) {
    for field in fields {
        collect_binding_names(&field.binding, names);
    }
}

fn collect_binding_names(binding: &AstDestructureBinding, names: &mut Vec<String>) {
    match binding {
        AstDestructureBinding::Bind(name) => names.push(name.clone()),
        AstDestructureBinding::Ignore => {}
        AstDestructureBinding::Nested { fields, .. } => {
            collect_destructure_binding_names(fields, names);
        }
    }
}
