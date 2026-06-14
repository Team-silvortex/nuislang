use std::collections::BTreeSet;

use nuis_semantics::model::{AstDestructureBinding, AstDestructureField, AstExpr, AstStmt};

pub(super) fn collect_lambda_block_captures(
    body: &[AstStmt],
    visible_locals: &mut BTreeSet<String>,
    outer_locals: &BTreeSet<String>,
    captures: &mut BTreeSet<String>,
) -> Result<(), String> {
    for stmt in body {
        match stmt {
            AstStmt::Let { name, value, .. } => {
                collect_lambda_expr_captures(value, visible_locals, outer_locals, captures)?;
                visible_locals.insert(name.clone());
            }
            AstStmt::DestructureLet { fields, value, .. } => {
                collect_lambda_expr_captures(value, visible_locals, outer_locals, captures)?;
                collect_destructure_binding_names(fields, visible_locals);
            }
            AstStmt::Const { name, value, .. } => {
                collect_lambda_expr_captures(value, visible_locals, outer_locals, captures)?;
                visible_locals.insert(name.clone());
            }
            AstStmt::AssignLocal { value, .. } => {
                collect_lambda_expr_captures(value, visible_locals, outer_locals, captures)?;
            }
            AstStmt::Print(value) | AstStmt::Await(value) | AstStmt::Expr(value) => {
                collect_lambda_expr_captures(value, visible_locals, outer_locals, captures)?;
            }
            AstStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                collect_lambda_expr_captures(condition, visible_locals, outer_locals, captures)?;
                let mut then_locals = visible_locals.clone();
                let mut else_locals = visible_locals.clone();
                collect_lambda_block_captures(then_body, &mut then_locals, outer_locals, captures)?;
                collect_lambda_block_captures(else_body, &mut else_locals, outer_locals, captures)?;
            }
            AstStmt::Match { value, arms } => {
                collect_lambda_expr_captures(value, visible_locals, outer_locals, captures)?;
                for arm in arms {
                    let mut arm_locals = visible_locals.clone();
                    collect_lambda_block_captures(
                        &arm.body,
                        &mut arm_locals,
                        outer_locals,
                        captures,
                    )?;
                }
            }
            AstStmt::While { condition, body } => {
                collect_lambda_expr_captures(condition, visible_locals, outer_locals, captures)?;
                let mut loop_locals = visible_locals.clone();
                collect_lambda_block_captures(body, &mut loop_locals, outer_locals, captures)?;
            }
            AstStmt::Return(Some(value)) => {
                collect_lambda_expr_captures(value, visible_locals, outer_locals, captures)?;
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

pub(super) fn collect_lambda_expr_captures(
    expr: &AstExpr,
    visible_locals: &BTreeSet<String>,
    outer_locals: &BTreeSet<String>,
    captures: &mut BTreeSet<String>,
) -> Result<(), String> {
    match expr {
        AstExpr::Var(name) if outer_locals.contains(name) && !visible_locals.contains(name) => {
            captures.insert(name.clone());
            Ok(())
        }
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => {
            collect_lambda_expr_captures(condition, visible_locals, outer_locals, captures)?;
            let mut then_locals = visible_locals.clone();
            let mut else_locals = visible_locals.clone();
            collect_lambda_block_captures(then_body, &mut then_locals, outer_locals, captures)?;
            collect_lambda_block_captures(else_body, &mut else_locals, outer_locals, captures)?;
            Ok(())
        }
        AstExpr::Match { value, arms } => {
            collect_lambda_expr_captures(value, visible_locals, outer_locals, captures)?;
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_lambda_expr_captures(guard, visible_locals, outer_locals, captures)?;
                }
                let mut arm_locals = visible_locals.clone();
                collect_lambda_block_captures(&arm.body, &mut arm_locals, outer_locals, captures)?;
            }
            Ok(())
        }
        AstExpr::Lambda { .. } => Err(
            "nested or inline lambdas are not supported in the current MVP; bind lambdas with `let name = |...| -> ... { ... };` only"
                .to_owned(),
        ),
        AstExpr::Await(value) => {
            collect_lambda_expr_captures(value, visible_locals, outer_locals, captures)
        }
        AstExpr::Invoke { callee, args } => {
            collect_lambda_expr_captures(callee, visible_locals, outer_locals, captures)?;
            for arg in args {
                collect_lambda_expr_captures(arg, visible_locals, outer_locals, captures)?;
            }
            Ok(())
        }
        AstExpr::Call { args, .. } => {
            for arg in args {
                collect_lambda_expr_captures(arg, visible_locals, outer_locals, captures)?;
            }
            Ok(())
        }
        AstExpr::MethodCall { receiver, args, .. } => {
            collect_lambda_expr_captures(receiver, visible_locals, outer_locals, captures)?;
            for arg in args {
                collect_lambda_expr_captures(arg, visible_locals, outer_locals, captures)?;
            }
            Ok(())
        }
        AstExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                collect_lambda_expr_captures(value, visible_locals, outer_locals, captures)?;
            }
            Ok(())
        }
        AstExpr::FieldAccess { base, .. } => {
            collect_lambda_expr_captures(base, visible_locals, outer_locals, captures)
        }
        AstExpr::Unary { operand, .. } => {
            collect_lambda_expr_captures(operand, visible_locals, outer_locals, captures)
        }
        AstExpr::Binary { lhs, rhs, .. } => {
            collect_lambda_expr_captures(lhs, visible_locals, outer_locals, captures)?;
            collect_lambda_expr_captures(rhs, visible_locals, outer_locals, captures)
        }
        AstExpr::Bool(_)
        | AstExpr::Text(_)
        | AstExpr::Int(_)
        | AstExpr::Float(_)
        | AstExpr::Var(_)
        | AstExpr::Instantiate { .. } => Ok(()),
    }
}
