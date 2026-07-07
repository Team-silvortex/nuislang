use nuis_semantics::model::AstMatchArm;

use super::{AstExpr, AstStmt};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ControlExprKind {
    If,
    Match,
}

impl ControlExprKind {
    fn keyword(self) -> &'static str {
        match self {
            Self::If => "if",
            Self::Match => "match",
        }
    }

    fn branch_name(self) -> &'static str {
        match self {
            Self::If => "branch",
            Self::Match => "arm",
        }
    }
}

pub(super) fn expand_nested_control_expr_stmt(
    stmt: &AstStmt,
    kind: ControlExprKind,
) -> Result<Option<Vec<AstStmt>>, String> {
    match stmt {
        AstStmt::Let {
            name,
            ty,
            value,
            mutable,
        } => expand_nested_control_expr_as_stmt(
            value,
            &|value| AstStmt::Let {
                mutable: *mutable,
                name: name.clone(),
                ty: ty.clone(),
                value,
            },
            kind,
            false,
        ),
        AstStmt::AssignLocal { name, value } => expand_nested_control_expr_as_stmt(
            value,
            &|value| AstStmt::AssignLocal {
                name: name.clone(),
                value,
            },
            kind,
            false,
        ),
        AstStmt::Const { name, ty, value } => expand_nested_control_expr_as_stmt(
            value,
            &|value| AstStmt::Const {
                name: name.clone(),
                ty: ty.clone(),
                value,
            },
            kind,
            false,
        ),
        AstStmt::Print(value) => {
            expand_nested_control_expr_as_stmt(value, &AstStmt::Print, kind, false)
        }
        AstStmt::Expr(value) => {
            expand_nested_control_expr_as_stmt(value, &AstStmt::Expr, kind, false)
        }
        AstStmt::Return(Some(value)) => expand_nested_control_expr_as_stmt(
            value,
            &|value| AstStmt::Return(Some(value)),
            kind,
            false,
        ),
        _ => Ok(None),
    }
}

fn expand_nested_control_expr_as_stmt(
    expr: &AstExpr,
    wrap: &dyn Fn(AstExpr) -> AstStmt,
    kind: ControlExprKind,
    allow_root_control: bool,
) -> Result<Option<Vec<AstStmt>>, String> {
    if allow_root_control {
        match (kind, expr) {
            (
                ControlExprKind::If,
                AstExpr::If {
                    condition,
                    then_body,
                    else_body,
                },
            ) => {
                return Ok(Some(vec![AstStmt::If {
                    condition: *condition.clone(),
                    then_body: rewrite_control_expr_terminal_branch(then_body, wrap, kind)?,
                    else_body: rewrite_control_expr_terminal_branch(else_body, wrap, kind)?,
                }]));
            }
            (ControlExprKind::Match, AstExpr::Match { value, arms }) => {
                return Ok(Some(vec![AstStmt::Match {
                    value: *value.clone(),
                    arms: arms
                        .iter()
                        .map(|arm| {
                            Ok(AstMatchArm {
                                pattern: arm.pattern.clone(),
                                guard: arm.guard.clone(),
                                body: rewrite_control_expr_terminal_branch(&arm.body, wrap, kind)?,
                            })
                        })
                        .collect::<Result<Vec<_>, String>>()?,
                }]));
            }
            _ => {}
        }
    }

    match expr {
        AstExpr::If { .. } if kind == ControlExprKind::If => Ok(None),
        AstExpr::Match { .. } if kind == ControlExprKind::Match => Ok(None),
        AstExpr::Await(value) => expand_nested_control_expr_as_stmt(
            value,
            &|rewritten| wrap(AstExpr::Await(Box::new(rewritten))),
            kind,
            true,
        ),
        AstExpr::Try(value) => expand_nested_control_expr_as_stmt(
            value,
            &|rewritten| wrap(AstExpr::Try(Box::new(rewritten))),
            kind,
            true,
        ),
        AstExpr::Unary { op, operand } => expand_nested_control_expr_as_stmt(
            operand,
            &|rewritten| {
                wrap(AstExpr::Unary {
                    op: *op,
                    operand: Box::new(rewritten),
                })
            },
            kind,
            true,
        ),
        AstExpr::Invoke { callee, args } => {
            if let Some(expanded) = expand_nested_control_expr_as_stmt(
                callee,
                &|rewritten| {
                    wrap(AstExpr::Invoke {
                        callee: Box::new(rewritten),
                        args: args.clone(),
                    })
                },
                kind,
                true,
            )? {
                return Ok(Some(expanded));
            }
            for (index, arg) in args.iter().enumerate() {
                if let Some(expanded) = expand_nested_control_expr_as_stmt(
                    arg,
                    &|rewritten| {
                        let mut rewritten_args = args.clone();
                        rewritten_args[index] = rewritten;
                        wrap(AstExpr::Invoke {
                            callee: callee.clone(),
                            args: rewritten_args,
                        })
                    },
                    kind,
                    true,
                )? {
                    return Ok(Some(expanded));
                }
            }
            Ok(None)
        }
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => {
            for (index, arg) in args.iter().enumerate() {
                if let Some(expanded) = expand_nested_control_expr_as_stmt(
                    arg,
                    &|rewritten| {
                        let mut rewritten_args = args.clone();
                        rewritten_args[index] = rewritten;
                        wrap(AstExpr::Call {
                            callee: callee.clone(),
                            generic_args: generic_args.clone(),
                            args: rewritten_args,
                        })
                    },
                    kind,
                    true,
                )? {
                    return Ok(Some(expanded));
                }
            }
            Ok(None)
        }
        AstExpr::MethodCall {
            receiver,
            method,
            generic_args,
            args,
        } => {
            if let Some(expanded) = expand_nested_control_expr_as_stmt(
                receiver,
                &|rewritten| {
                    wrap(AstExpr::MethodCall {
                        receiver: Box::new(rewritten),
                        method: method.clone(),
                        generic_args: generic_args.clone(),
                        args: args.clone(),
                    })
                },
                kind,
                true,
            )? {
                return Ok(Some(expanded));
            }
            for (index, arg) in args.iter().enumerate() {
                if let Some(expanded) = expand_nested_control_expr_as_stmt(
                    arg,
                    &|rewritten| {
                        let mut rewritten_args = args.clone();
                        rewritten_args[index] = rewritten;
                        wrap(AstExpr::MethodCall {
                            receiver: receiver.clone(),
                            method: method.clone(),
                            generic_args: generic_args.clone(),
                            args: rewritten_args,
                        })
                    },
                    kind,
                    true,
                )? {
                    return Ok(Some(expanded));
                }
            }
            Ok(None)
        }
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => {
            for (index, (_, value)) in fields.iter().enumerate() {
                if let Some(expanded) = expand_nested_control_expr_as_stmt(
                    value,
                    &|rewritten| {
                        let mut rewritten_fields = fields.clone();
                        rewritten_fields[index].1 = rewritten;
                        wrap(AstExpr::StructLiteral {
                            type_name: type_name.clone(),
                            type_args: type_args.clone(),
                            fields: rewritten_fields,
                        })
                    },
                    kind,
                    true,
                )? {
                    return Ok(Some(expanded));
                }
            }
            Ok(None)
        }
        AstExpr::FieldAccess { base, field } => expand_nested_control_expr_as_stmt(
            base,
            &|rewritten| {
                wrap(AstExpr::FieldAccess {
                    base: Box::new(rewritten),
                    field: field.clone(),
                })
            },
            kind,
            true,
        ),
        AstExpr::Binary { op, lhs, rhs } => {
            if let Some(expanded) = expand_nested_control_expr_as_stmt(
                lhs,
                &|rewritten| {
                    wrap(AstExpr::Binary {
                        op: *op,
                        lhs: Box::new(rewritten),
                        rhs: rhs.clone(),
                    })
                },
                kind,
                true,
            )? {
                return Ok(Some(expanded));
            }
            expand_nested_control_expr_as_stmt(
                rhs,
                &|rewritten| {
                    wrap(AstExpr::Binary {
                        op: *op,
                        lhs: lhs.clone(),
                        rhs: Box::new(rewritten),
                    })
                },
                kind,
                true,
            )
        }
        AstExpr::Bool(_)
        | AstExpr::Text(_)
        | AstExpr::Int(_)
        | AstExpr::Float(_)
        | AstExpr::Var(_)
        | AstExpr::Lambda { .. }
        | AstExpr::Instantiate { .. } => Ok(None),
        AstExpr::If { .. } | AstExpr::Match { .. } => Ok(None),
    }
}

pub(super) fn rewrite_control_expr_terminal_branch(
    body: &[AstStmt],
    wrap: &dyn Fn(AstExpr) -> AstStmt,
    kind: ControlExprKind,
) -> Result<Vec<AstStmt>, String> {
    let Some((last, prefix)) = body.split_last() else {
        return Err(format!(
            "`{}` expression {} cannot be empty",
            kind.keyword(),
            kind.branch_name()
        ));
    };
    let mut rewritten = prefix.to_vec();
    match last {
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            rewritten.push(AstStmt::If {
                condition: condition.clone(),
                then_body: rewrite_control_expr_terminal_branch(
                    then_body,
                    wrap,
                    ControlExprKind::If,
                )?,
                else_body: rewrite_control_expr_terminal_branch(
                    else_body,
                    wrap,
                    ControlExprKind::If,
                )?,
            });
            Ok(rewritten)
        }
        AstStmt::Match { value, arms } => {
            rewritten.push(AstStmt::Match {
                value: value.clone(),
                arms: arms
                    .iter()
                    .map(|arm| {
                        Ok(AstMatchArm {
                            pattern: arm.pattern.clone(),
                            guard: arm.guard.clone(),
                            body: rewrite_control_expr_terminal_branch(
                                &arm.body,
                                wrap,
                                ControlExprKind::Match,
                            )?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            });
            Ok(rewritten)
        }
        AstStmt::Return(Some(value)) | AstStmt::Expr(value) => {
            for root_kind in [ControlExprKind::If, ControlExprKind::Match] {
                if let Some(expanded) =
                    expand_nested_control_expr_as_stmt(value, wrap, root_kind, true)?
                {
                    rewritten.extend(expanded);
                    return Ok(rewritten);
                }
            }
            rewritten.push(wrap(value.clone()));
            Ok(rewritten)
        }
        AstStmt::Break | AstStmt::Continue => {
            rewritten.push(last.clone());
            Ok(rewritten)
        }
        _ => Err(format!(
            "`{}` expression {} currently requires either a tail expression result or terminal loop control (`break`/`continue`) in each {}",
            kind.keyword(),
            kind.branch_name(),
            kind.branch_name()
        )),
    }
}
