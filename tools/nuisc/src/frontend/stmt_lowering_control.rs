use std::collections::BTreeMap;

use nuis_semantics::model::AstMatchArm;

use super::match_lowering::lower_match_stmt_with_async;
use super::metadata::ModuleConstValue;
use super::stmt_lowering::lower_stmt_block_with_async;
use super::{
    bool_type, lower_expr_with_async, AstStmt, AstTypeAlias, AstTypeRef, FunctionSignature,
    NirStmt, NirStructDef, NirTypeRef,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_if_expr_stmt_with_async(
    condition: &super::AstExpr,
    then_body: &[AstStmt],
    else_body: &[AstStmt],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    wrap_terminal: &dyn Fn(super::AstExpr) -> AstStmt,
) -> Result<NirStmt, String> {
    let condition = lower_expr_with_async(
        condition,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        Some(&bool_type()),
        false,
    )?;
    let mut then_bindings = bindings.clone();
    let mut else_bindings = bindings.clone();
    Ok(NirStmt::If {
        condition,
        then_body: lower_if_expr_branch_with_async(
            then_body,
            current_domain,
            current_function_is_async,
            &mut then_bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
            wrap_terminal,
        )?,
        else_body: lower_if_expr_branch_with_async(
            else_body,
            current_domain,
            current_function_is_async,
            &mut else_bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
            wrap_terminal,
        )?,
    })
}

#[allow(clippy::too_many_arguments)]
fn lower_if_expr_branch_with_async(
    body: &[AstStmt],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    wrap_terminal: &dyn Fn(super::AstExpr) -> AstStmt,
) -> Result<Vec<NirStmt>, String> {
    let rewritten_body =
        rewrite_control_expr_terminal_branch(body, wrap_terminal, ControlExprKind::If)?;
    lower_stmt_block_with_async(
        &rewritten_body,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_match_expr_stmt_with_async(
    value: &super::AstExpr,
    arms: &[AstMatchArm],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &mut BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    wrap_terminal: &dyn Fn(super::AstExpr) -> AstStmt,
) -> Result<NirStmt, String> {
    let rewritten_arms = arms
        .iter()
        .map(|arm| {
            Ok(AstMatchArm {
                pattern: arm.pattern.clone(),
                guard: arm.guard.clone(),
                body: rewrite_control_expr_terminal_branch(
                    &arm.body,
                    wrap_terminal,
                    ControlExprKind::Match,
                )?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    lower_match_stmt_with_async(
        value,
        &rewritten_arms,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
    )
}

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
    expr: &super::AstExpr,
    wrap: &dyn Fn(super::AstExpr) -> AstStmt,
    kind: ControlExprKind,
    allow_root_control: bool,
) -> Result<Option<Vec<AstStmt>>, String> {
    if allow_root_control {
        match (kind, expr) {
            (
                ControlExprKind::If,
                super::AstExpr::If {
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
            (ControlExprKind::Match, super::AstExpr::Match { value, arms }) => {
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
        super::AstExpr::If { .. } if kind == ControlExprKind::If => Ok(None),
        super::AstExpr::Match { .. } if kind == ControlExprKind::Match => Ok(None),
        super::AstExpr::Await(value) => expand_nested_control_expr_as_stmt(
            value,
            &|rewritten| wrap(super::AstExpr::Await(Box::new(rewritten))),
            kind,
            true,
        ),
        super::AstExpr::Try(value) => expand_nested_control_expr_as_stmt(
            value,
            &|rewritten| wrap(super::AstExpr::Try(Box::new(rewritten))),
            kind,
            true,
        ),
        super::AstExpr::Unary { op, operand } => expand_nested_control_expr_as_stmt(
            operand,
            &|rewritten| {
                wrap(super::AstExpr::Unary {
                    op: *op,
                    operand: Box::new(rewritten),
                })
            },
            kind,
            true,
        ),
        super::AstExpr::Invoke { callee, args } => {
            if let Some(expanded) = expand_nested_control_expr_as_stmt(
                callee,
                &|rewritten| {
                    wrap(super::AstExpr::Invoke {
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
                        wrap(super::AstExpr::Invoke {
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
        super::AstExpr::Call {
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
                        wrap(super::AstExpr::Call {
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
        super::AstExpr::MethodCall {
            receiver,
            method,
            generic_args,
            args,
        } => {
            if let Some(expanded) = expand_nested_control_expr_as_stmt(
                receiver,
                &|rewritten| {
                    wrap(super::AstExpr::MethodCall {
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
                        wrap(super::AstExpr::MethodCall {
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
        super::AstExpr::StructLiteral {
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
                        wrap(super::AstExpr::StructLiteral {
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
        super::AstExpr::FieldAccess { base, field } => expand_nested_control_expr_as_stmt(
            base,
            &|rewritten| {
                wrap(super::AstExpr::FieldAccess {
                    base: Box::new(rewritten),
                    field: field.clone(),
                })
            },
            kind,
            true,
        ),
        super::AstExpr::Binary { op, lhs, rhs } => {
            if let Some(expanded) = expand_nested_control_expr_as_stmt(
                lhs,
                &|rewritten| {
                    wrap(super::AstExpr::Binary {
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
                    wrap(super::AstExpr::Binary {
                        op: *op,
                        lhs: lhs.clone(),
                        rhs: Box::new(rewritten),
                    })
                },
                kind,
                true,
            )
        }
        super::AstExpr::Bool(_)
        | super::AstExpr::Text(_)
        | super::AstExpr::Int(_)
        | super::AstExpr::Float(_)
        | super::AstExpr::Var(_)
        | super::AstExpr::Lambda { .. }
        | super::AstExpr::Instantiate { .. } => Ok(None),
        super::AstExpr::If { .. } | super::AstExpr::Match { .. } => Ok(None),
    }
}

fn rewrite_control_expr_terminal_branch(
    body: &[AstStmt],
    wrap: &dyn Fn(super::AstExpr) -> AstStmt,
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
