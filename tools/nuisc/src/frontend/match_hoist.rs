use nuis_semantics::model::{AstExpr, AstFunction, AstMatchArm, AstModule, AstStmt};

fn ast_expr_requires_match_hoist(expr: &AstExpr) -> bool {
    match expr {
        AstExpr::Call { .. }
        | AstExpr::Invoke { .. }
        | AstExpr::MethodCall { .. }
        | AstExpr::Await(_)
        | AstExpr::Instantiate { .. } => true,
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => {
            ast_expr_requires_match_hoist(condition)
                || then_body.iter().any(ast_stmt_requires_match_hoist)
                || else_body.iter().any(ast_stmt_requires_match_hoist)
        }
        AstExpr::Match { value, arms } => {
            ast_expr_requires_match_hoist(value)
                || arms.iter().any(|arm| {
                    arm.guard
                        .as_ref()
                        .is_some_and(ast_expr_requires_match_hoist)
                        || arm.body.iter().any(ast_stmt_requires_match_hoist)
                })
        }
        AstExpr::FieldAccess { base, .. } => ast_expr_requires_match_hoist(base),
        AstExpr::Binary { lhs, rhs, .. } => {
            ast_expr_requires_match_hoist(lhs) || ast_expr_requires_match_hoist(rhs)
        }
        AstExpr::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| ast_expr_requires_match_hoist(value)),
        AstExpr::Bool(_)
        | AstExpr::Text(_)
        | AstExpr::Int(_)
        | AstExpr::Float(_)
        | AstExpr::Var(_)
        | AstExpr::Lambda { .. } => false,
    }
}

fn ast_stmt_requires_match_hoist(stmt: &AstStmt) -> bool {
    match stmt {
        AstStmt::Let { value, .. }
        | AstStmt::Const { value, .. }
        | AstStmt::Print(value)
        | AstStmt::Await(value)
        | AstStmt::Expr(value)
        | AstStmt::Return(Some(value)) => ast_expr_requires_match_hoist(value),
        AstStmt::DestructureLet { value, .. } => ast_expr_requires_match_hoist(value),
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            ast_expr_requires_match_hoist(condition)
                || then_body.iter().any(ast_stmt_requires_match_hoist)
                || else_body.iter().any(ast_stmt_requires_match_hoist)
        }
        AstStmt::Match { value, arms } => {
            ast_expr_requires_match_hoist(value)
                || arms
                    .iter()
                    .any(|arm| arm.body.iter().any(ast_stmt_requires_match_hoist))
        }
        AstStmt::While { condition, body } => {
            ast_expr_requires_match_hoist(condition)
                || body.iter().any(ast_stmt_requires_match_hoist)
        }
        AstStmt::Break | AstStmt::Continue | AstStmt::Return(None) => false,
    }
}

pub(super) fn expand_effectful_match_scrutinees(module: &AstModule) -> AstModule {
    let mut expanded = module.clone();
    expanded.functions = module
        .functions
        .iter()
        .map(rewrite_effectful_match_scrutinees_in_function)
        .collect();
    expanded
}

fn rewrite_effectful_match_scrutinees_in_function(function: &AstFunction) -> AstFunction {
    let mut counter = 0usize;
    let mut rewritten = function.clone();
    rewritten.body = rewrite_effectful_match_scrutinees_in_block(&function.body, &mut counter);
    rewritten
}

fn rewrite_effectful_match_scrutinees_in_block(
    body: &[AstStmt],
    counter: &mut usize,
) -> Vec<AstStmt> {
    let mut rewritten = Vec::new();
    for stmt in body {
        match stmt {
            AstStmt::If {
                condition,
                then_body,
                else_body,
            } => rewritten.push(AstStmt::If {
                condition: condition.clone(),
                then_body: rewrite_effectful_match_scrutinees_in_block(then_body, counter),
                else_body: rewrite_effectful_match_scrutinees_in_block(else_body, counter),
            }),
            AstStmt::Match { value, arms } if ast_expr_requires_match_hoist(value) => {
                let temp_name = format!("__match_scrutinee_{counter}");
                *counter += 1;
                rewritten.push(AstStmt::Let {
                    name: temp_name.clone(),
                    ty: None,
                    value: value.clone(),
                });
                rewritten.push(AstStmt::Match {
                    value: AstExpr::Var(temp_name),
                    arms: arms
                        .iter()
                        .map(|arm| AstMatchArm {
                            pattern: arm.pattern.clone(),
                            guard: arm.guard.clone(),
                            body: rewrite_effectful_match_scrutinees_in_block(&arm.body, counter),
                        })
                        .collect(),
                });
            }
            AstStmt::Match { value, arms } => rewritten.push(AstStmt::Match {
                value: value.clone(),
                arms: arms
                    .iter()
                    .map(|arm| AstMatchArm {
                        pattern: arm.pattern.clone(),
                        guard: arm.guard.clone(),
                        body: rewrite_effectful_match_scrutinees_in_block(&arm.body, counter),
                    })
                    .collect(),
            }),
            AstStmt::While { condition, body } => rewritten.push(AstStmt::While {
                condition: condition.clone(),
                body: rewrite_effectful_match_scrutinees_in_block(body, counter),
            }),
            other => rewritten.push(other.clone()),
        }
    }
    rewritten
}
