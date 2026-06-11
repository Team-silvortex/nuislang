use std::collections::BTreeSet;

use nuis_semantics::model::{AstDestructureBinding, AstDestructureField, AstExpr, AstStmt};

pub(super) fn validate_lambda_block_no_capture(
    body: &[AstStmt],
    visible_locals: &mut BTreeSet<String>,
    outer_locals: &BTreeSet<String>,
) -> Result<(), String> {
    for stmt in body {
        match stmt {
            AstStmt::Let { name, value, .. } => {
                validate_lambda_expr_no_capture(value, visible_locals, outer_locals)?;
                visible_locals.insert(name.clone());
            }
            AstStmt::DestructureLet { fields, value, .. } => {
                validate_lambda_expr_no_capture(value, visible_locals, outer_locals)?;
                collect_destructure_binding_names(fields, visible_locals);
            }
            AstStmt::Const { name, value, .. } => {
                validate_lambda_expr_no_capture(value, visible_locals, outer_locals)?;
                visible_locals.insert(name.clone());
            }
            AstStmt::Print(value) | AstStmt::Await(value) | AstStmt::Expr(value) => {
                validate_lambda_expr_no_capture(value, visible_locals, outer_locals)?;
            }
            AstStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                validate_lambda_expr_no_capture(condition, visible_locals, outer_locals)?;
                let mut then_locals = visible_locals.clone();
                let mut else_locals = visible_locals.clone();
                validate_lambda_block_no_capture(then_body, &mut then_locals, outer_locals)?;
                validate_lambda_block_no_capture(else_body, &mut else_locals, outer_locals)?;
            }
            AstStmt::Match { value, arms } => {
                validate_lambda_expr_no_capture(value, visible_locals, outer_locals)?;
                for arm in arms {
                    let mut arm_locals = visible_locals.clone();
                    validate_lambda_block_no_capture(&arm.body, &mut arm_locals, outer_locals)?;
                }
            }
            AstStmt::While { condition, body } => {
                validate_lambda_expr_no_capture(condition, visible_locals, outer_locals)?;
                let mut loop_locals = visible_locals.clone();
                validate_lambda_block_no_capture(body, &mut loop_locals, outer_locals)?;
            }
            AstStmt::Return(Some(value)) => {
                validate_lambda_expr_no_capture(value, visible_locals, outer_locals)?;
            }
            AstStmt::Return(None) | AstStmt::Break | AstStmt::Continue => {}
        }
    }
    Ok(())
}

fn collect_destructure_binding_names(
    fields: &[AstDestructureField],
    visible_locals: &mut BTreeSet<String>,
) {
    for field in fields {
        match &field.binding {
            AstDestructureBinding::Bind(name) => {
                visible_locals.insert(name.clone());
            }
            AstDestructureBinding::Ignore => {}
            AstDestructureBinding::Nested { fields, .. } => {
                collect_destructure_binding_names(fields, visible_locals);
            }
        }
    }
}

pub(super) fn validate_lambda_expr_no_capture(
    expr: &AstExpr,
    visible_locals: &BTreeSet<String>,
    outer_locals: &BTreeSet<String>,
) -> Result<(), String> {
    match expr {
        AstExpr::Var(name) if outer_locals.contains(name) && !visible_locals.contains(name) => {
            Err(format!(
                "lambda currently does not support capturing outer local `{name}`"
            ))
        }
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => {
            validate_lambda_expr_no_capture(condition, visible_locals, outer_locals)?;
            let mut then_locals = visible_locals.clone();
            let mut else_locals = visible_locals.clone();
            validate_lambda_block_no_capture(then_body, &mut then_locals, outer_locals)?;
            validate_lambda_block_no_capture(else_body, &mut else_locals, outer_locals)?;
            Ok(())
        }
        AstExpr::Match { value, arms } => {
            validate_lambda_expr_no_capture(value, visible_locals, outer_locals)?;
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    validate_lambda_expr_no_capture(guard, visible_locals, outer_locals)?;
                }
                let mut arm_locals = visible_locals.clone();
                validate_lambda_block_no_capture(&arm.body, &mut arm_locals, outer_locals)?;
            }
            Ok(())
        }
        AstExpr::Lambda { .. } => Err(
            "nested or inline lambdas are not supported in the current MVP; bind lambdas with `let name = |...| -> ... { ... };` only"
                .to_owned(),
        ),
        AstExpr::Await(value) => {
            validate_lambda_expr_no_capture(value, visible_locals, outer_locals)
        }
        AstExpr::Invoke { callee, args } => {
            validate_lambda_expr_no_capture(callee, visible_locals, outer_locals)?;
            for arg in args {
                validate_lambda_expr_no_capture(arg, visible_locals, outer_locals)?;
            }
            Ok(())
        }
        AstExpr::Call { args, .. } => {
            for arg in args {
                validate_lambda_expr_no_capture(arg, visible_locals, outer_locals)?;
            }
            Ok(())
        }
        AstExpr::MethodCall { receiver, args, .. } => {
            validate_lambda_expr_no_capture(receiver, visible_locals, outer_locals)?;
            for arg in args {
                validate_lambda_expr_no_capture(arg, visible_locals, outer_locals)?;
            }
            Ok(())
        }
        AstExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                validate_lambda_expr_no_capture(value, visible_locals, outer_locals)?;
            }
            Ok(())
        }
        AstExpr::FieldAccess { base, .. } => {
            validate_lambda_expr_no_capture(base, visible_locals, outer_locals)
        }
        AstExpr::Binary { lhs, rhs, .. } => {
            validate_lambda_expr_no_capture(lhs, visible_locals, outer_locals)?;
            validate_lambda_expr_no_capture(rhs, visible_locals, outer_locals)
        }
        AstExpr::Bool(_)
        | AstExpr::Text(_)
        | AstExpr::Int(_)
        | AstExpr::Var(_)
        | AstExpr::Instantiate { .. } => Ok(()),
    }
}
