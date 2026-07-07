use std::collections::BTreeMap;

use super::metadata::ModuleConstValue;
use super::stmt_lowering_try::{
    current_function_result_type, split_result_type, synthesize_try_statements,
};
use super::{
    compatible_types, infer_nir_expr_type, lower_expr_with_async, AstStmt, AstTypeAlias,
    AstTypeRef, ExprWithAsyncInput, FunctionSignature, NirStructDef, NirTypeRef,
};

#[derive(Clone, Copy)]
pub(super) struct TryExpansionContext<'a> {
    pub(super) current_domain: &'a str,
    pub(super) current_function_is_async: bool,
    pub(super) bindings: &'a BTreeMap<String, NirTypeRef>,
    pub(super) module_consts: &'a BTreeMap<String, ModuleConstValue>,
    pub(super) return_type: Option<&'a AstTypeRef>,
    pub(super) type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) struct_table: &'a BTreeMap<String, NirStructDef>,
}

pub(super) struct NestedTryStmtExpansionInput<'a> {
    pub(super) stmt: &'a AstStmt,
    pub(super) context: TryExpansionContext<'a>,
}

struct TryWrappedStatementsInput<'a> {
    inner: &'a super::AstExpr,
    wrap: &'a dyn Fn(super::AstExpr) -> AstStmt,
    context: TryExpansionContext<'a>,
}

struct NestedTryExprExpansionInput<'a> {
    expr: &'a super::AstExpr,
    wrap: &'a dyn Fn(super::AstExpr) -> AstStmt,
    context: TryExpansionContext<'a>,
}

pub(super) fn expand_nested_try_stmt(
    input: NestedTryStmtExpansionInput<'_>,
) -> Result<Option<Vec<AstStmt>>, String> {
    let NestedTryStmtExpansionInput { stmt, context } = input;
    match stmt {
        AstStmt::Let {
            mutable,
            name,
            ty,
            value,
        } => expand_nested_try_expr_as_stmt(NestedTryExprExpansionInput {
            expr: value,
            wrap: &|value| AstStmt::Let {
                mutable: *mutable,
                name: name.clone(),
                ty: ty.clone(),
                value,
            },
            context,
        }),
        AstStmt::AssignLocal { name, value } => {
            expand_nested_try_expr_as_stmt(NestedTryExprExpansionInput {
                expr: value,
                wrap: &|value| AstStmt::AssignLocal {
                    name: name.clone(),
                    value,
                },
                context,
            })
        }
        AstStmt::Const { name, ty, value } => {
            expand_nested_try_expr_as_stmt(NestedTryExprExpansionInput {
                expr: value,
                wrap: &|value| AstStmt::Const {
                    name: name.clone(),
                    ty: ty.clone(),
                    value,
                },
                context,
            })
        }
        AstStmt::Print(value) => expand_nested_try_expr_as_stmt(NestedTryExprExpansionInput {
            expr: value,
            wrap: &AstStmt::Print,
            context,
        }),
        AstStmt::Expr(value) => expand_nested_try_expr_as_stmt(NestedTryExprExpansionInput {
            expr: value,
            wrap: &AstStmt::Expr,
            context,
        }),
        AstStmt::Return(Some(value)) => {
            expand_nested_try_expr_as_stmt(NestedTryExprExpansionInput {
                expr: value,
                wrap: &|value| AstStmt::Return(Some(value)),
                context,
            })
        }
        _ => Ok(None),
    }
}

fn synthesize_try_wrapped_statements(
    input: TryWrappedStatementsInput<'_>,
) -> Result<Vec<AstStmt>, String> {
    let TryWrappedStatementsInput {
        inner,
        wrap,
        context,
    } = input;
    let function_result_ty =
        current_function_result_type(context.return_type, context.type_aliases)?;
    let lowered_inner = lower_expr_with_async(ExprWithAsyncInput {
        expr: inner,
        current_domain: context.current_domain,
        current_function_is_async: context.current_function_is_async,
        bindings: context.bindings,
        module_consts: context.module_consts,
        signatures: context.signatures,
        struct_table: context.struct_table,
        expected: None,
        allow_async_calls: false,
    })?;
    let inner_ty = infer_nir_expr_type(
        &lowered_inner,
        context.bindings,
        context.signatures,
        context.struct_table,
    )
    .ok_or_else(|| "could not infer operand type for `?`".to_owned())?;
    let (_payload_ty, error_ty) = split_result_type(&inner_ty)?;
    if !compatible_types(&function_result_ty.1, &error_ty) {
        return Err(format!(
            "`?` error type `{}` does not match enclosing function error type `{}`",
            error_ty.render(),
            function_result_ty.1.render()
        ));
    }
    synthesize_try_statements(
        lowered_inner,
        inner_ty,
        wrap(super::AstExpr::Var("__nuis_try_payload".to_owned())),
    )
}

fn expand_nested_try_expr_as_stmt(
    input: NestedTryExprExpansionInput<'_>,
) -> Result<Option<Vec<AstStmt>>, String> {
    let NestedTryExprExpansionInput {
        expr,
        wrap,
        context,
    } = input;
    match expr {
        super::AstExpr::Try(inner) => Ok(Some(synthesize_try_wrapped_statements(
            TryWrappedStatementsInput {
                inner,
                wrap,
                context,
            },
        )?)),
        super::AstExpr::Await(value) => {
            expand_nested_try_expr_as_stmt(NestedTryExprExpansionInput {
                expr: value,
                wrap: &|rewritten| wrap(super::AstExpr::Await(Box::new(rewritten))),
                context,
            })
        }
        super::AstExpr::Unary { op, operand } => {
            expand_nested_try_expr_as_stmt(NestedTryExprExpansionInput {
                expr: operand,
                wrap: &|rewritten| {
                    wrap(super::AstExpr::Unary {
                        op: *op,
                        operand: Box::new(rewritten),
                    })
                },
                context,
            })
        }
        super::AstExpr::Invoke { callee, args } => {
            if let Some(expanded) = expand_nested_try_expr_as_stmt(NestedTryExprExpansionInput {
                expr: callee,
                wrap: &|rewritten| {
                    wrap(super::AstExpr::Invoke {
                        callee: Box::new(rewritten),
                        args: args.clone(),
                    })
                },
                context,
            })? {
                return Ok(Some(expanded));
            }
            for (index, arg) in args.iter().enumerate() {
                if let Some(expanded) =
                    expand_nested_try_expr_as_stmt(NestedTryExprExpansionInput {
                        expr: arg,
                        wrap: &|rewritten| {
                            let mut rewritten_args = args.clone();
                            rewritten_args[index] = rewritten;
                            wrap(super::AstExpr::Invoke {
                                callee: callee.clone(),
                                args: rewritten_args,
                            })
                        },
                        context,
                    })?
                {
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
                if let Some(expanded) =
                    expand_nested_try_expr_as_stmt(NestedTryExprExpansionInput {
                        expr: arg,
                        wrap: &|rewritten| {
                            let mut rewritten_args = args.clone();
                            rewritten_args[index] = rewritten;
                            wrap(super::AstExpr::Call {
                                callee: callee.clone(),
                                generic_args: generic_args.clone(),
                                args: rewritten_args,
                            })
                        },
                        context,
                    })?
                {
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
            if let Some(expanded) = expand_nested_try_expr_as_stmt(NestedTryExprExpansionInput {
                expr: receiver,
                wrap: &|rewritten| {
                    wrap(super::AstExpr::MethodCall {
                        receiver: Box::new(rewritten),
                        method: method.clone(),
                        generic_args: generic_args.clone(),
                        args: args.clone(),
                    })
                },
                context,
            })? {
                return Ok(Some(expanded));
            }
            for (index, arg) in args.iter().enumerate() {
                if let Some(expanded) =
                    expand_nested_try_expr_as_stmt(NestedTryExprExpansionInput {
                        expr: arg,
                        wrap: &|rewritten| {
                            let mut rewritten_args = args.clone();
                            rewritten_args[index] = rewritten;
                            wrap(super::AstExpr::MethodCall {
                                receiver: receiver.clone(),
                                method: method.clone(),
                                generic_args: generic_args.clone(),
                                args: rewritten_args,
                            })
                        },
                        context,
                    })?
                {
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
                if let Some(expanded) =
                    expand_nested_try_expr_as_stmt(NestedTryExprExpansionInput {
                        expr: value,
                        wrap: &|rewritten| {
                            let mut rewritten_fields = fields.clone();
                            rewritten_fields[index].1 = rewritten;
                            wrap(super::AstExpr::StructLiteral {
                                type_name: type_name.clone(),
                                type_args: type_args.clone(),
                                fields: rewritten_fields,
                            })
                        },
                        context,
                    })?
                {
                    return Ok(Some(expanded));
                }
            }
            Ok(None)
        }
        super::AstExpr::FieldAccess { base, field } => {
            expand_nested_try_expr_as_stmt(NestedTryExprExpansionInput {
                expr: base,
                wrap: &|rewritten| {
                    wrap(super::AstExpr::FieldAccess {
                        base: Box::new(rewritten),
                        field: field.clone(),
                    })
                },
                context,
            })
        }
        super::AstExpr::Binary { op, lhs, rhs } => {
            if let Some(expanded) = expand_nested_try_expr_as_stmt(NestedTryExprExpansionInput {
                expr: lhs,
                wrap: &|rewritten| {
                    wrap(super::AstExpr::Binary {
                        op: *op,
                        lhs: Box::new(rewritten),
                        rhs: rhs.clone(),
                    })
                },
                context,
            })? {
                return Ok(Some(expanded));
            }
            expand_nested_try_expr_as_stmt(NestedTryExprExpansionInput {
                expr: rhs,
                wrap: &|rewritten| {
                    wrap(super::AstExpr::Binary {
                        op: *op,
                        lhs: lhs.clone(),
                        rhs: Box::new(rewritten),
                    })
                },
                context,
            })
        }
        super::AstExpr::Bool(_)
        | super::AstExpr::Text(_)
        | super::AstExpr::Int(_)
        | super::AstExpr::Float(_)
        | super::AstExpr::Var(_)
        | super::AstExpr::Lambda { .. }
        | super::AstExpr::Instantiate { .. }
        | super::AstExpr::If { .. }
        | super::AstExpr::Match { .. } => Ok(None),
    }
}
