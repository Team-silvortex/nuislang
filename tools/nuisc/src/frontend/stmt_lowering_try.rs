use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use nuis_semantics::model::{AstMatchArm, NirExpr};

use super::metadata::ModuleConstValue;
use super::{
    ast_type_from_nir, compatible_types, infer_nir_expr_type, lower_expr_with_async,
    lower_type_ref_with_aliases, AstStmt, AstTypeAlias, AstTypeRef, ExprWithAsyncInput,
    FunctionSignature, NirStructDef, NirTypeRef,
};

static TRY_EXPANSION_COUNTER: AtomicUsize = AtomicUsize::new(0);

use super::stmt_lowering_try_helpers::{ast_expr_from_nir, rewrite_try_payload_placeholder};

#[derive(Clone, Copy)]
pub(super) struct TryStmtExpansionContext<'a> {
    pub(super) current_domain: &'a str,
    pub(super) current_function_is_async: bool,
    pub(super) bindings: &'a BTreeMap<String, NirTypeRef>,
    pub(super) module_consts: &'a BTreeMap<String, ModuleConstValue>,
    pub(super) return_type: Option<&'a AstTypeRef>,
    pub(super) type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) struct_table: &'a BTreeMap<String, NirStructDef>,
}

pub(super) struct TryStmtExpansionInput<'a> {
    pub(super) stmt: &'a AstStmt,
    pub(super) context: TryStmtExpansionContext<'a>,
}

pub(super) fn expand_try_stmt(
    input: TryStmtExpansionInput<'_>,
) -> Result<Option<Vec<AstStmt>>, String> {
    let TryStmtExpansionInput { stmt, context } = input;
    let (inner, expansion) = match stmt {
        AstStmt::Let {
            mutable,
            name,
            ty,
            value: super::AstExpr::Try(inner),
        } => (
            inner.as_ref(),
            TryConsumer::Let {
                mutable: *mutable,
                name: name.clone(),
                declared_ty: ty.clone(),
            },
        ),
        AstStmt::Const {
            name,
            ty,
            value: super::AstExpr::Try(inner),
        } => (
            inner.as_ref(),
            TryConsumer::Const {
                name: name.clone(),
                declared_ty: ty.clone(),
            },
        ),
        AstStmt::Print(super::AstExpr::Try(inner)) => (inner.as_ref(), TryConsumer::Print),
        AstStmt::Expr(super::AstExpr::Try(inner)) => (inner.as_ref(), TryConsumer::Expr),
        AstStmt::Return(Some(super::AstExpr::Try(inner))) => (inner.as_ref(), TryConsumer::Return),
        _ => return Ok(None),
    };

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
    let (payload_ty, error_ty) = split_result_type(&inner_ty)?;
    if !compatible_types(&function_result_ty.1, &error_ty) {
        return Err(format!(
            "`?` error type `{}` does not match enclosing function error type `{}`",
            error_ty.render(),
            function_result_ty.1.render()
        ));
    }

    let expansion = match expansion {
        TryConsumer::Let {
            mutable,
            name,
            declared_ty,
        } => {
            let final_payload_ty = match declared_ty {
                Some(declared_ty) => {
                    let lowered_declared =
                        lower_type_ref_with_aliases(&declared_ty, context.type_aliases)?;
                    if !compatible_types(&lowered_declared, &payload_ty) {
                        return Err(format!(
                            "`?` payload type `{}` does not match declared type `{}` for `{}`",
                            payload_ty.render(),
                            lowered_declared.render(),
                            name
                        ));
                    }
                    ast_type_from_nir(&lowered_declared)
                }
                None => ast_type_from_nir(&payload_ty),
            };
            synthesize_try_statements(
                lowered_inner,
                inner_ty,
                AstStmt::Let {
                    mutable,
                    name,
                    ty: Some(final_payload_ty),
                    value: super::AstExpr::Var("__nuis_try_payload".to_owned()),
                },
            )
        }
        TryConsumer::Const { name, declared_ty } => {
            let final_payload_ty = match declared_ty {
                Some(declared_ty) => {
                    let lowered_declared =
                        lower_type_ref_with_aliases(&declared_ty, context.type_aliases)?;
                    if !compatible_types(&lowered_declared, &payload_ty) {
                        return Err(format!(
                            "`?` payload type `{}` does not match declared type `{}` for `{}`",
                            payload_ty.render(),
                            lowered_declared.render(),
                            name
                        ));
                    }
                    ast_type_from_nir(&lowered_declared)
                }
                None => ast_type_from_nir(&payload_ty),
            };
            synthesize_try_statements(
                lowered_inner,
                inner_ty,
                AstStmt::Const {
                    name,
                    ty: Some(final_payload_ty),
                    value: super::AstExpr::Var("__nuis_try_payload".to_owned()),
                },
            )
        }
        TryConsumer::Print => synthesize_try_statements(
            lowered_inner,
            inner_ty,
            AstStmt::Print(super::AstExpr::Var("__nuis_try_payload".to_owned())),
        ),
        TryConsumer::Expr => synthesize_try_expr_statements(lowered_inner, inner_ty),
        TryConsumer::Return => synthesize_try_statements(
            lowered_inner,
            inner_ty,
            AstStmt::Return(Some(super::AstExpr::Var("__nuis_try_payload".to_owned()))),
        ),
    }?;

    Ok(Some(expansion))
}

enum TryConsumer {
    Let {
        mutable: bool,
        name: String,
        declared_ty: Option<AstTypeRef>,
    },
    Const {
        name: String,
        declared_ty: Option<AstTypeRef>,
    },
    Print,
    Expr,
    Return,
}

pub(super) fn current_function_result_type<'a>(
    return_type: Option<&'a AstTypeRef>,
    type_aliases: &'a BTreeMap<String, AstTypeAlias>,
) -> Result<(NirTypeRef, NirTypeRef), String> {
    let return_type = return_type.ok_or_else(|| {
        "`?` currently requires an enclosing function with explicit `Result<Payload, Error>` return type"
            .to_owned()
    })?;
    let lowered = lower_type_ref_with_aliases(return_type, type_aliases)?;
    let (payload, error) = split_result_type(&lowered)?;
    Ok((payload, error))
}

pub(super) fn split_result_type(ty: &NirTypeRef) -> Result<(NirTypeRef, NirTypeRef), String> {
    if ty.name == "Result" && ty.generic_args.len() == 2 && !ty.is_ref && !ty.is_optional {
        return Ok((ty.generic_args[0].clone(), ty.generic_args[1].clone()));
    }
    Err(format!(
        "`?` currently requires a `Result<Payload, Error>` operand, found `{}`",
        ty.render()
    ))
}

pub(super) fn synthesize_try_statements(
    lowered_inner: NirExpr,
    inner_ty: NirTypeRef,
    ok_terminal: AstStmt,
) -> Result<Vec<AstStmt>, String> {
    let id = TRY_EXPANSION_COUNTER.fetch_add(1, Ordering::Relaxed);
    let result_name = format!("__nuis_try_result_{id}");
    let payload_name = format!("__nuis_try_payload_{id}");
    let error_name = format!("__nuis_try_error_{id}");
    let result_ty = ast_type_from_nir(&inner_ty);
    let ok_stmt = rewrite_try_payload_placeholder(ok_terminal, &payload_name)?;

    Ok(vec![
        AstStmt::Let {
            mutable: false,
            name: result_name.clone(),
            ty: Some(result_ty),
            value: ast_expr_from_nir(lowered_inner),
        },
        AstStmt::Match {
            value: super::AstExpr::Var(result_name),
            arms: vec![
                AstMatchArm {
                    pattern: nuis_semantics::model::AstMatchPattern::PayloadStruct {
                        type_ref: AstTypeRef {
                            name: "Result.Err".to_owned(),
                            generic_args: Vec::new(),
                            is_optional: false,
                            is_ref: false,
                        },
                        payload: Box::new(nuis_semantics::model::AstMatchPattern::Bind(
                            error_name.clone(),
                        )),
                    },
                    guard: None,
                    body: vec![AstStmt::Return(Some(super::AstExpr::Call {
                        callee: "Result.Err".to_owned(),
                        generic_args: Vec::new(),
                        args: vec![super::AstExpr::Var(error_name)],
                    }))],
                },
                AstMatchArm {
                    pattern: nuis_semantics::model::AstMatchPattern::PayloadStruct {
                        type_ref: AstTypeRef {
                            name: "Result.Ok".to_owned(),
                            generic_args: Vec::new(),
                            is_optional: false,
                            is_ref: false,
                        },
                        payload: Box::new(nuis_semantics::model::AstMatchPattern::Bind(
                            payload_name,
                        )),
                    },
                    guard: None,
                    body: vec![ok_stmt],
                },
            ],
        },
    ])
}

fn synthesize_try_expr_statements(
    lowered_inner: NirExpr,
    inner_ty: NirTypeRef,
) -> Result<Vec<AstStmt>, String> {
    let id = TRY_EXPANSION_COUNTER.fetch_add(1, Ordering::Relaxed);
    let result_name = format!("__nuis_try_result_{id}");
    let error_name = format!("__nuis_try_error_{id}");
    Ok(vec![
        AstStmt::Let {
            mutable: false,
            name: result_name.clone(),
            ty: Some(ast_type_from_nir(&inner_ty)),
            value: ast_expr_from_nir(lowered_inner),
        },
        AstStmt::Match {
            value: super::AstExpr::Var(result_name),
            arms: vec![
                AstMatchArm {
                    pattern: nuis_semantics::model::AstMatchPattern::PayloadStruct {
                        type_ref: AstTypeRef {
                            name: "Result.Ok".to_owned(),
                            generic_args: Vec::new(),
                            is_optional: false,
                            is_ref: false,
                        },
                        payload: Box::new(nuis_semantics::model::AstMatchPattern::Wildcard),
                    },
                    guard: None,
                    body: Vec::new(),
                },
                AstMatchArm {
                    pattern: nuis_semantics::model::AstMatchPattern::PayloadStruct {
                        type_ref: AstTypeRef {
                            name: "Result.Err".to_owned(),
                            generic_args: Vec::new(),
                            is_optional: false,
                            is_ref: false,
                        },
                        payload: Box::new(nuis_semantics::model::AstMatchPattern::Bind(
                            error_name.clone(),
                        )),
                    },
                    guard: None,
                    body: vec![AstStmt::Return(Some(super::AstExpr::Call {
                        callee: "Result.Err".to_owned(),
                        generic_args: Vec::new(),
                        args: vec![super::AstExpr::Var(error_name)],
                    }))],
                },
            ],
        },
    ])
}
