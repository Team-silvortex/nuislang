use nuis_semantics::model::{
    AstAttribute, AstAttributeArg, AstAttributeValue, AstExpr, AstFunction, AstModule, AstStmt,
};

#[derive(Default, Clone, Copy)]
struct RewriteStats {
    helper_rewrites: usize,
    local_rewrites: usize,
}

pub(super) fn rewrite_text_handle_helpers(module: &AstModule) -> AstModule {
    let mut rewritten = module.clone();
    if rewritten.domain != "cpu" {
        return rewritten;
    }
    rewritten.functions = rewritten
        .functions
        .iter()
        .map(rewrite_text_handle_helper_function)
        .collect();
    rewritten
}

fn rewrite_text_handle_helper_function(function: &AstFunction) -> AstFunction {
    let Some(text_value) = match_text_handle_helper_body(&function.body) else {
        let mut rewritten = function.clone();
        let (body, stats) = rewrite_stmt_block(&function.body);
        rewritten.body = body;
        attach_rewrite_annotation(&mut rewritten, stats);
        return rewritten;
    };
    let mut rewritten = function.clone();
    rewritten.body = vec![AstStmt::Return(Some(AstExpr::Call {
        callee: "text_handle".to_owned(),
        generic_args: vec![],
        args: vec![text_value],
    }))];
    attach_rewrite_annotation(
        &mut rewritten,
        RewriteStats {
            helper_rewrites: 1,
            local_rewrites: 0,
        },
    );
    rewritten
}

fn match_text_handle_helper_body(body: &[AstStmt]) -> Option<AstExpr> {
    let [AstStmt::Let {
        mutable: false,
        name: buffer_name,
        ty: _,
        value: buffer_value,
    }, AstStmt::Let {
        mutable: false,
        name: len_name,
        ty: _,
        value: len_value,
    }, AstStmt::Return(Some(return_value))] = body
    else {
        return None;
    };

    if !is_alloc_buffer_call(buffer_value) {
        return None;
    }

    let serialized_text =
        match_serialize_text_into_call(len_value, buffer_name).filter(|_| !len_name.is_empty())?;
    if !matches_deserialize_text_from_call(return_value, buffer_name, len_name) {
        return None;
    }
    Some(serialized_text)
}

fn attach_rewrite_annotation(function: &mut AstFunction, stats: RewriteStats) {
    if stats.helper_rewrites == 0 && stats.local_rewrites == 0 {
        return;
    }
    function.attributes.push(AstAttribute {
        name: "__nuisc_text_handle_rewrite".to_owned(),
        args: vec![
            AstAttributeArg {
                name: Some("helper".to_owned()),
                value: AstAttributeValue::Int(stats.helper_rewrites as i64),
            },
            AstAttributeArg {
                name: Some("local".to_owned()),
                value: AstAttributeValue::Int(stats.local_rewrites as i64),
            },
        ],
    });
}

fn rewrite_stmt_block(body: &[AstStmt]) -> (Vec<AstStmt>, RewriteStats) {
    let mut rewritten = Vec::with_capacity(body.len());
    let mut stats = RewriteStats::default();
    let mut index = 0;
    while index < body.len() {
        if let Some((replacement, consumed)) = match_local_text_handle_pattern(&body[index..]) {
            rewritten.push(replacement);
            stats.local_rewrites += 1;
            index += consumed;
            continue;
        }
        let (stmt, stmt_stats) = rewrite_stmt(&body[index]);
        rewritten.push(stmt);
        stats.helper_rewrites += stmt_stats.helper_rewrites;
        stats.local_rewrites += stmt_stats.local_rewrites;
        index += 1;
    }
    (rewritten, stats)
}

fn rewrite_stmt(stmt: &AstStmt) -> (AstStmt, RewriteStats) {
    match stmt {
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            let (then_body, then_stats) = rewrite_stmt_block(then_body);
            let (else_body, else_stats) = rewrite_stmt_block(else_body);
            (
                AstStmt::If {
                    condition: rewrite_expr(condition),
                    then_body,
                    else_body,
                },
                RewriteStats {
                    helper_rewrites: then_stats.helper_rewrites + else_stats.helper_rewrites,
                    local_rewrites: then_stats.local_rewrites + else_stats.local_rewrites,
                },
            )
        }
        AstStmt::Match { value, arms } => {
            let mut stats = RewriteStats::default();
            let rewritten_arms = arms
                .iter()
                .map(|arm| {
                    let (body, body_stats) = rewrite_stmt_block(&arm.body);
                    stats.helper_rewrites += body_stats.helper_rewrites;
                    stats.local_rewrites += body_stats.local_rewrites;
                    nuis_semantics::model::AstMatchArm {
                        pattern: arm.pattern.clone(),
                        guard: arm.guard.as_ref().map(rewrite_expr),
                        body,
                    }
                })
                .collect();
            (
                AstStmt::Match {
                    value: rewrite_expr(value),
                    arms: rewritten_arms,
                },
                stats,
            )
        }
        AstStmt::While { condition, body } => {
            let (body, stats) = rewrite_stmt_block(body);
            (
                AstStmt::While {
                    condition: rewrite_expr(condition),
                    body,
                },
                stats,
            )
        }
        AstStmt::Let {
            mutable,
            name,
            ty,
            value,
        } => (
            AstStmt::Let {
                mutable: *mutable,
                name: name.clone(),
                ty: ty.clone(),
                value: rewrite_expr(value),
            },
            RewriteStats::default(),
        ),
        AstStmt::AssignLocal { name, value } => (
            AstStmt::AssignLocal {
                name: name.clone(),
                value: rewrite_expr(value),
            },
            RewriteStats::default(),
        ),
        AstStmt::DestructureLet {
            type_ref,
            fields,
            value,
        } => (
            AstStmt::DestructureLet {
                type_ref: type_ref.clone(),
                fields: fields.clone(),
                value: rewrite_expr(value),
            },
            RewriteStats::default(),
        ),
        AstStmt::Const { name, ty, value } => (
            AstStmt::Const {
                name: name.clone(),
                ty: ty.clone(),
                value: rewrite_expr(value),
            },
            RewriteStats::default(),
        ),
        AstStmt::Print(value) => (AstStmt::Print(rewrite_expr(value)), RewriteStats::default()),
        AstStmt::Await(value) => (AstStmt::Await(rewrite_expr(value)), RewriteStats::default()),
        AstStmt::Expr(value) => (AstStmt::Expr(rewrite_expr(value)), RewriteStats::default()),
        AstStmt::Return(Some(value)) => (
            AstStmt::Return(Some(rewrite_expr(value))),
            RewriteStats::default(),
        ),
        _ => (stmt.clone(), RewriteStats::default()),
    }
}

fn rewrite_expr(expr: &AstExpr) -> AstExpr {
    match expr {
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => AstExpr::If {
            condition: Box::new(rewrite_expr(condition)),
            then_body: rewrite_stmt_block(then_body).0,
            else_body: rewrite_stmt_block(else_body).0,
        },
        AstExpr::Match { value, arms } => AstExpr::Match {
            value: Box::new(rewrite_expr(value)),
            arms: arms
                .iter()
                .map(|arm| nuis_semantics::model::AstMatchArm {
                    pattern: arm.pattern.clone(),
                    guard: arm.guard.as_ref().map(rewrite_expr),
                    body: rewrite_stmt_block(&arm.body).0,
                })
                .collect(),
        },
        AstExpr::Await(value) => AstExpr::Await(Box::new(rewrite_expr(value))),
        AstExpr::Try(value) => AstExpr::Try(Box::new(rewrite_expr(value))),
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => AstExpr::Call {
            callee: callee.clone(),
            generic_args: generic_args.clone(),
            args: args.iter().map(rewrite_expr).collect(),
        },
        AstExpr::Invoke { callee, args } => AstExpr::Invoke {
            callee: Box::new(rewrite_expr(callee)),
            args: args.iter().map(rewrite_expr).collect(),
        },
        AstExpr::MethodCall {
            receiver,
            method,
            generic_args,
            args,
        } => AstExpr::MethodCall {
            receiver: Box::new(rewrite_expr(receiver)),
            method: method.clone(),
            generic_args: generic_args.clone(),
            args: args.iter().map(rewrite_expr).collect(),
        },
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => AstExpr::StructLiteral {
            type_name: type_name.clone(),
            type_args: type_args.clone(),
            fields: fields
                .iter()
                .map(|(field, value)| (field.clone(), rewrite_expr(value)))
                .collect(),
        },
        AstExpr::FieldAccess { base, field } => AstExpr::FieldAccess {
            base: Box::new(rewrite_expr(base)),
            field: field.clone(),
        },
        AstExpr::Unary { op, operand } => AstExpr::Unary {
            op: *op,
            operand: Box::new(rewrite_expr(operand)),
        },
        AstExpr::Binary { op, lhs, rhs } => AstExpr::Binary {
            op: *op,
            lhs: Box::new(rewrite_expr(lhs)),
            rhs: Box::new(rewrite_expr(rhs)),
        },
        _ => expr.clone(),
    }
}

fn match_local_text_handle_pattern(stmts: &[AstStmt]) -> Option<(AstStmt, usize)> {
    let [AstStmt::Let {
        mutable: false,
        name: buffer_name,
        ty: _,
        value: buffer_value,
    }, AstStmt::Let {
        mutable: false,
        name: len_name,
        ty: _,
        value: len_value,
    }, third, ..] = stmts
    else {
        return None;
    };

    if !is_alloc_buffer_call(buffer_value) || len_name.is_empty() {
        return None;
    }
    let serialized_text = match_serialize_text_into_call(len_value, buffer_name)?;
    let replacement = match third {
        AstStmt::Let {
            mutable,
            name,
            ty,
            value,
        } if matches_deserialize_text_from_call(value, buffer_name, len_name)
            && matches!(ty, Some(ty) if ty.name == "i64" && !ty.is_ref && !ty.is_optional)
            && !stmts[3..]
                .iter()
                .any(|stmt| stmt_references_any_name(stmt, &[buffer_name, len_name])) =>
        {
            AstStmt::Let {
                mutable: *mutable,
                name: name.clone(),
                ty: ty.clone(),
                value: AstExpr::Call {
                    callee: "text_handle".to_owned(),
                    generic_args: vec![],
                    args: vec![serialized_text],
                },
            }
        }
        AstStmt::Return(Some(value))
            if matches_deserialize_text_from_call(value, buffer_name, len_name) =>
        {
            AstStmt::Return(Some(AstExpr::Call {
                callee: "text_handle".to_owned(),
                generic_args: vec![],
                args: vec![serialized_text],
            }))
        }
        _ => return None,
    };
    Some((replacement, 3))
}

fn stmt_references_any_name(stmt: &AstStmt, names: &[&str]) -> bool {
    match stmt {
        AstStmt::Let { value, .. }
        | AstStmt::AssignLocal { value, .. }
        | AstStmt::DestructureLet { value, .. }
        | AstStmt::Const { value, .. }
        | AstStmt::Print(value)
        | AstStmt::Await(value)
        | AstStmt::Expr(value) => expr_references_any_name(value, names),
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_references_any_name(condition, names)
                || then_body
                    .iter()
                    .any(|stmt| stmt_references_any_name(stmt, names))
                || else_body
                    .iter()
                    .any(|stmt| stmt_references_any_name(stmt, names))
        }
        AstStmt::Match { value, arms } => {
            expr_references_any_name(value, names)
                || arms.iter().any(|arm| {
                    arm.guard
                        .as_ref()
                        .is_some_and(|guard| expr_references_any_name(guard, names))
                        || arm
                            .body
                            .iter()
                            .any(|stmt| stmt_references_any_name(stmt, names))
                })
        }
        AstStmt::While { condition, body } => {
            expr_references_any_name(condition, names)
                || body
                    .iter()
                    .any(|stmt| stmt_references_any_name(stmt, names))
        }
        AstStmt::Return(Some(value)) => expr_references_any_name(value, names),
        _ => false,
    }
}

fn expr_references_any_name(expr: &AstExpr, names: &[&str]) -> bool {
    match expr {
        AstExpr::Var(name) => names.iter().any(|candidate| name == *candidate),
        AstExpr::If {
            condition,
            then_body,
            else_body,
        } => {
            expr_references_any_name(condition, names)
                || then_body
                    .iter()
                    .any(|stmt| stmt_references_any_name(stmt, names))
                || else_body
                    .iter()
                    .any(|stmt| stmt_references_any_name(stmt, names))
        }
        AstExpr::Match { value, arms } => {
            expr_references_any_name(value, names)
                || arms.iter().any(|arm| {
                    arm.guard
                        .as_ref()
                        .is_some_and(|guard| expr_references_any_name(guard, names))
                        || arm
                            .body
                            .iter()
                            .any(|stmt| stmt_references_any_name(stmt, names))
                })
        }
        AstExpr::Lambda { body, .. } => body
            .iter()
            .any(|stmt| stmt_references_any_name(stmt, names)),
        AstExpr::Await(value) | AstExpr::Try(value) => expr_references_any_name(value, names),
        AstExpr::Call { args, .. } => args.iter().any(|arg| expr_references_any_name(arg, names)),
        AstExpr::Invoke { callee, args } => {
            expr_references_any_name(callee, names)
                || args.iter().any(|arg| expr_references_any_name(arg, names))
        }
        AstExpr::MethodCall { receiver, args, .. } => {
            expr_references_any_name(receiver, names)
                || args.iter().any(|arg| expr_references_any_name(arg, names))
        }
        AstExpr::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expr_references_any_name(value, names)),
        AstExpr::FieldAccess { base, .. } => expr_references_any_name(base, names),
        AstExpr::Unary { operand, .. } => expr_references_any_name(operand, names),
        AstExpr::Binary { lhs, rhs, .. } => {
            expr_references_any_name(lhs, names) || expr_references_any_name(rhs, names)
        }
        _ => false,
    }
}

fn is_alloc_buffer_call(expr: &AstExpr) -> bool {
    matches!(
        expr,
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } if callee == "alloc_buffer" && generic_args.is_empty() && args.len() == 2
    )
}

fn match_serialize_text_into_call(expr: &AstExpr, buffer_name: &str) -> Option<AstExpr> {
    let AstExpr::Call {
        callee,
        generic_args,
        args,
    } = expr
    else {
        return None;
    };
    if callee != "serialize_text_into" || !generic_args.is_empty() || args.len() != 3 {
        return None;
    }
    if !matches!(args.get(1), Some(AstExpr::Var(name)) if name == buffer_name) {
        return None;
    }
    if !matches!(args.get(2), Some(AstExpr::Int(0))) {
        return None;
    }
    args.first().cloned()
}

fn matches_deserialize_text_from_call(expr: &AstExpr, buffer_name: &str, len_name: &str) -> bool {
    matches!(
        expr,
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } if callee == "deserialize_text_from"
            && generic_args.is_empty()
            && args.len() == 3
            && matches!(args.first(), Some(AstExpr::Var(name)) if name == buffer_name)
            && matches!(args.get(1), Some(AstExpr::Int(0)))
            && matches!(args.get(2), Some(AstExpr::Var(name)) if name == len_name)
    )
}
