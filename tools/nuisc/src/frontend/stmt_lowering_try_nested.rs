use std::collections::BTreeMap;

use super::metadata::ModuleConstValue;
use super::stmt_lowering_try::{
    current_function_result_type, split_result_type, synthesize_try_statements,
};
use super::{
    compatible_types, infer_nir_expr_type, lower_expr_with_async, AstStmt, AstTypeAlias,
    AstTypeRef, FunctionSignature, NirStructDef, NirTypeRef,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn expand_nested_try_stmt(
    stmt: &AstStmt,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<Vec<AstStmt>>, String> {
    match stmt {
        AstStmt::Let {
            mutable,
            name,
            ty,
            value,
        } => expand_nested_try_expr_as_stmt(
            value,
            &|value| AstStmt::Let {
                mutable: *mutable,
                name: name.clone(),
                ty: ty.clone(),
                value,
            },
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        ),
        AstStmt::AssignLocal { name, value } => expand_nested_try_expr_as_stmt(
            value,
            &|value| AstStmt::AssignLocal {
                name: name.clone(),
                value,
            },
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        ),
        AstStmt::Const { name, ty, value } => expand_nested_try_expr_as_stmt(
            value,
            &|value| AstStmt::Const {
                name: name.clone(),
                ty: ty.clone(),
                value,
            },
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        ),
        AstStmt::Print(value) => expand_nested_try_expr_as_stmt(
            value,
            &AstStmt::Print,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        ),
        AstStmt::Expr(value) => expand_nested_try_expr_as_stmt(
            value,
            &AstStmt::Expr,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        ),
        AstStmt::Return(Some(value)) => expand_nested_try_expr_as_stmt(
            value,
            &|value| AstStmt::Return(Some(value)),
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        ),
        _ => Ok(None),
    }
}

#[allow(clippy::too_many_arguments)]
fn synthesize_try_wrapped_statements(
    inner: &super::AstExpr,
    wrap: &dyn Fn(super::AstExpr) -> AstStmt,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Vec<AstStmt>, String> {
    let function_result_ty = current_function_result_type(return_type, type_aliases)?;
    let lowered_inner = lower_expr_with_async(
        inner,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        None,
        false,
    )?;
    let inner_ty = infer_nir_expr_type(&lowered_inner, bindings, signatures, struct_table)
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

#[allow(clippy::too_many_arguments)]
fn expand_nested_try_expr_as_stmt(
    expr: &super::AstExpr,
    wrap: &dyn Fn(super::AstExpr) -> AstStmt,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    return_type: Option<&AstTypeRef>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<Vec<AstStmt>>, String> {
    match expr {
        super::AstExpr::Try(inner) => Ok(Some(synthesize_try_wrapped_statements(
            inner,
            wrap,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        )?)),
        super::AstExpr::Await(value) => expand_nested_try_expr_as_stmt(
            value,
            &|rewritten| wrap(super::AstExpr::Await(Box::new(rewritten))),
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        ),
        super::AstExpr::Unary { op, operand } => expand_nested_try_expr_as_stmt(
            operand,
            &|rewritten| {
                wrap(super::AstExpr::Unary {
                    op: *op,
                    operand: Box::new(rewritten),
                })
            },
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        ),
        super::AstExpr::Invoke { callee, args } => {
            if let Some(expanded) = expand_nested_try_expr_as_stmt(
                callee,
                &|rewritten| {
                    wrap(super::AstExpr::Invoke {
                        callee: Box::new(rewritten),
                        args: args.clone(),
                    })
                },
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                return_type,
                type_aliases,
                signatures,
                struct_table,
            )? {
                return Ok(Some(expanded));
            }
            for (index, arg) in args.iter().enumerate() {
                if let Some(expanded) = expand_nested_try_expr_as_stmt(
                    arg,
                    &|rewritten| {
                        let mut rewritten_args = args.clone();
                        rewritten_args[index] = rewritten;
                        wrap(super::AstExpr::Invoke {
                            callee: callee.clone(),
                            args: rewritten_args,
                        })
                    },
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
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
                if let Some(expanded) = expand_nested_try_expr_as_stmt(
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
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
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
            if let Some(expanded) = expand_nested_try_expr_as_stmt(
                receiver,
                &|rewritten| {
                    wrap(super::AstExpr::MethodCall {
                        receiver: Box::new(rewritten),
                        method: method.clone(),
                        generic_args: generic_args.clone(),
                        args: args.clone(),
                    })
                },
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                return_type,
                type_aliases,
                signatures,
                struct_table,
            )? {
                return Ok(Some(expanded));
            }
            for (index, arg) in args.iter().enumerate() {
                if let Some(expanded) = expand_nested_try_expr_as_stmt(
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
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
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
                if let Some(expanded) = expand_nested_try_expr_as_stmt(
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
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                )? {
                    return Ok(Some(expanded));
                }
            }
            Ok(None)
        }
        super::AstExpr::FieldAccess { base, field } => expand_nested_try_expr_as_stmt(
            base,
            &|rewritten| {
                wrap(super::AstExpr::FieldAccess {
                    base: Box::new(rewritten),
                    field: field.clone(),
                })
            },
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        ),
        super::AstExpr::Binary { op, lhs, rhs } => {
            if let Some(expanded) = expand_nested_try_expr_as_stmt(
                lhs,
                &|rewritten| {
                    wrap(super::AstExpr::Binary {
                        op: *op,
                        lhs: Box::new(rewritten),
                        rhs: rhs.clone(),
                    })
                },
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                return_type,
                type_aliases,
                signatures,
                struct_table,
            )? {
                return Ok(Some(expanded));
            }
            expand_nested_try_expr_as_stmt(
                rhs,
                &|rewritten| {
                    wrap(super::AstExpr::Binary {
                        op: *op,
                        lhs: lhs.clone(),
                        rhs: Box::new(rewritten),
                    })
                },
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
