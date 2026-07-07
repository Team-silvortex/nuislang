use std::collections::BTreeMap;

use super::match_lowering::{lower_match_stmt_with_async, MatchStmtLoweringInput};
use super::metadata::ModuleConstValue;
use super::stmt_lowering_control::{
    expand_nested_control_expr_stmt, lower_if_expr_stmt_with_context,
    lower_match_expr_stmt_with_context, ControlExprKind, ControlExprLoweringContext,
    IfExprStmtLoweringInput, MatchExprStmtLoweringInput,
};
use super::stmt_lowering_destructure::{
    lower_destructure_let_stmt_with_async, DestructureLetLoweringInput,
};
use super::stmt_lowering_sequence::{
    lower_expanded_stmt_sequence_with_async, ExpandedStmtSequenceLoweringInput,
};
pub(super) use super::stmt_lowering_sequence::{
    lower_stmt_block_with_async, StmtBlockLoweringInput,
};
use super::stmt_lowering_try::{expand_try_stmt, TryStmtExpansionContext, TryStmtExpansionInput};
use super::stmt_lowering_try_nested::{
    expand_nested_try_stmt, NestedTryStmtExpansionInput, TryExpansionContext,
};
use super::validation_helpers::validate_type_ref;
use super::{
    bool_type, infer_nir_expr_type, lower_expr_with_async, lower_type_ref_with_aliases,
    resolve_declared_or_inferred, AstStmt, AstTypeAlias, AstTypeRef, ExprWithAsyncInput,
    FunctionSignature, NirStmt, NirStructDef, NirTypeRef,
};

pub(super) struct StmtLoweringInput<'a> {
    pub(super) stmt: &'a AstStmt,
    pub(super) current_domain: &'a str,
    pub(super) current_function_is_async: bool,
    pub(super) bindings: &'a mut BTreeMap<String, NirTypeRef>,
    pub(super) module_consts: &'a BTreeMap<String, ModuleConstValue>,
    pub(super) return_type: Option<&'a AstTypeRef>,
    pub(super) type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) struct_table: &'a BTreeMap<String, NirStructDef>,
}

pub(super) fn lower_stmt_with_async(input: StmtLoweringInput<'_>) -> Result<NirStmt, String> {
    let StmtLoweringInput {
        stmt,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
    } = input;
    macro_rules! control_context {
        ($wrap_terminal:expr) => {
            ControlExprLoweringContext {
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                return_type,
                type_aliases,
                signatures,
                struct_table,
                wrap_terminal: $wrap_terminal,
            }
        };
    }
    macro_rules! lower_if_expr {
        ($condition:expr, $then_body:expr, $else_body:expr, $wrap_terminal:expr) => {
            lower_if_expr_stmt_with_context(IfExprStmtLoweringInput {
                condition: $condition,
                then_body: $then_body,
                else_body: $else_body,
                context: control_context!($wrap_terminal),
            })
        };
    }
    macro_rules! lower_match_expr {
        ($value:expr, $arms:expr, $wrap_terminal:expr) => {
            lower_match_expr_stmt_with_context(MatchExprStmtLoweringInput {
                value: $value,
                arms: $arms,
                context: control_context!($wrap_terminal),
            })
        };
    }

    Ok(match stmt {
        AstStmt::Let {
            name, ty, value, ..
        } => {
            if let super::AstExpr::If {
                condition,
                then_body,
                else_body,
            } = value
            {
                let expected = ty
                    .as_ref()
                    .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
                    .transpose()?;
                if let Some(expected_ty) = expected.as_ref() {
                    validate_type_ref(expected_ty)?;
                }
                let final_type = expected.clone().ok_or_else(|| {
                    format!(
                        "`if` expression let binding `{name}` currently requires an explicit type annotation"
                    )
                })?;
                bindings.insert(name.clone(), final_type.clone());
                return lower_if_expr!(condition, then_body, else_body, &|value| AstStmt::Let {
                    mutable: false,
                    name: name.clone(),
                    ty: Some(ty.clone().unwrap_or_else(|| AstTypeRef {
                        name: final_type.name.clone(),
                        generic_args: Vec::new(),
                        is_ref: final_type.is_ref,
                        is_optional: final_type.is_optional,
                    })),
                    value,
                });
            }
            if let super::AstExpr::Match { value, arms } = value {
                let expected = ty
                    .as_ref()
                    .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
                    .transpose()?;
                if let Some(expected_ty) = expected.as_ref() {
                    validate_type_ref(expected_ty)?;
                }
                let final_type = expected.clone().ok_or_else(|| {
                    format!(
                        "`match` expression let binding `{name}` currently requires an explicit type annotation"
                    )
                })?;
                bindings.insert(name.clone(), final_type.clone());
                return lower_match_expr!(value, arms, &|value| AstStmt::Let {
                    mutable: false,
                    name: name.clone(),
                    ty: Some(ty.clone().unwrap_or_else(|| AstTypeRef {
                        name: final_type.name.clone(),
                        generic_args: Vec::new(),
                        is_ref: final_type.is_ref,
                        is_optional: final_type.is_optional,
                    })),
                    value,
                });
            }
            let expected = ty
                .as_ref()
                .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
                .transpose()?;
            if let Some(expected_ty) = expected.as_ref() {
                validate_type_ref(expected_ty)?;
            }
            let lowered = lower_expr_with_async(ExprWithAsyncInput {
                expr: value,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                expected: expected.as_ref(),
                allow_async_calls: false,
            })?;
            let inferred = infer_nir_expr_type(&lowered, bindings, signatures, struct_table);
            let final_type = resolve_declared_or_inferred(name, expected, inferred)?;
            bindings.insert(name.clone(), final_type.clone());
            NirStmt::Let {
                name: name.clone(),
                ty: Some(final_type),
                value: lowered,
            }
        }
        AstStmt::AssignLocal { name, value } => {
            let current_ty = bindings
                .get(name)
                .cloned()
                .ok_or_else(|| format!("cannot assign to unknown local `{name}`"))?;
            let lowered = lower_expr_with_async(ExprWithAsyncInput {
                expr: value,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                expected: Some(&current_ty),
                allow_async_calls: false,
            })?;
            let inferred = infer_nir_expr_type(&lowered, bindings, signatures, struct_table);
            let final_type =
                resolve_declared_or_inferred(name, Some(current_ty.clone()), inferred)?;
            bindings.insert(name.clone(), final_type.clone());
            NirStmt::Let {
                name: name.clone(),
                ty: Some(final_type),
                value: lowered,
            }
        }
        AstStmt::Const { name, ty, value } => {
            if let super::AstExpr::If {
                condition,
                then_body,
                else_body,
            } = value
            {
                if ty.is_none() {
                    return Err(format!(
                        "`if` expression const binding `{name}` currently requires an explicit type annotation"
                    ));
                }
                return lower_if_expr!(condition, then_body, else_body, &|value| AstStmt::Const {
                    name: name.clone(),
                    ty: ty.clone(),
                    value,
                });
            }
            if let super::AstExpr::Match { value, arms } = value {
                if ty.is_none() {
                    return Err(format!(
                        "`match` expression const binding `{name}` currently requires an explicit type annotation"
                    ));
                }
                return lower_match_expr!(value, arms, &|value| AstStmt::Const {
                    name: name.clone(),
                    ty: ty.clone(),
                    value,
                });
            }
            let expected = ty
                .as_ref()
                .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
                .transpose()?;
            if let Some(expected_ty) = expected.as_ref() {
                validate_type_ref(expected_ty)?;
            }
            let lowered = lower_expr_with_async(ExprWithAsyncInput {
                expr: value,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                expected: expected.as_ref(),
                allow_async_calls: false,
            })?;
            let inferred = infer_nir_expr_type(&lowered, bindings, signatures, struct_table);
            let final_type = resolve_declared_or_inferred(name, expected, inferred)?;
            bindings.insert(name.clone(), final_type.clone());
            NirStmt::Const {
                name: name.clone(),
                ty: final_type,
                value: lowered,
            }
        }
        AstStmt::Print(value) => {
            if let super::AstExpr::If {
                condition,
                then_body,
                else_body,
            } = value
            {
                return lower_if_expr!(condition, then_body, else_body, &AstStmt::Print);
            }
            if let super::AstExpr::Match { value, arms } = value {
                return lower_match_expr!(value, arms, &AstStmt::Print);
            }
            NirStmt::Print(lower_expr_with_async(ExprWithAsyncInput {
                expr: value,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                expected: None,
                allow_async_calls: false,
            })?)
        }
        AstStmt::Await(value) => {
            if !current_function_is_async {
                return Err("`await` is only allowed inside `async fn`".to_owned());
            }
            NirStmt::Await(lower_expr_with_async(ExprWithAsyncInput {
                expr: value,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                expected: None,
                allow_async_calls: true,
            })?)
        }
        AstStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            let mut then_bindings = bindings.clone();
            let mut else_bindings = bindings.clone();
            NirStmt::If {
                condition: lower_expr_with_async(ExprWithAsyncInput {
                    expr: condition,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    signatures,
                    struct_table,
                    expected: Some(&bool_type()),
                    allow_async_calls: false,
                })?,
                then_body: lower_stmt_block_with_async(StmtBlockLoweringInput {
                    stmts: then_body,
                    current_domain,
                    current_function_is_async,
                    bindings: &mut then_bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                })?,
                else_body: lower_stmt_block_with_async(StmtBlockLoweringInput {
                    stmts: else_body,
                    current_domain,
                    current_function_is_async,
                    bindings: &mut else_bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                })?,
            }
        }
        AstStmt::Match { value, arms } => lower_match_stmt_with_async(MatchStmtLoweringInput {
            value,
            arms,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        })?,
        AstStmt::While { condition, body } => {
            let mut loop_bindings = bindings.clone();
            NirStmt::While {
                condition: lower_expr_with_async(ExprWithAsyncInput {
                    expr: condition,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    signatures,
                    struct_table,
                    expected: Some(&bool_type()),
                    allow_async_calls: false,
                })?,
                body: lower_stmt_block_with_async(StmtBlockLoweringInput {
                    stmts: body,
                    current_domain,
                    current_function_is_async,
                    bindings: &mut loop_bindings,
                    module_consts,
                    return_type,
                    type_aliases,
                    signatures,
                    struct_table,
                })?,
            }
        }
        AstStmt::Break => NirStmt::Break,
        AstStmt::Continue => NirStmt::Continue,
        AstStmt::Expr(expr) => NirStmt::Expr(lower_expr_with_async(ExprWithAsyncInput {
            expr,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            expected: None,
            allow_async_calls: false,
        })?),
        AstStmt::Return(value) => {
            if let Some(super::AstExpr::If {
                condition,
                then_body,
                else_body,
            }) = value
            {
                return lower_if_expr!(condition, then_body, else_body, &|value| AstStmt::Return(
                    Some(value)
                ));
            }
            if let Some(super::AstExpr::Match { value, arms }) = value {
                return lower_match_expr!(value, arms, &|value| AstStmt::Return(Some(value)));
            }
            let expected = return_type
                .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
                .transpose()?;
            if let Some(expected_ty) = expected.as_ref() {
                validate_type_ref(expected_ty)?;
            }
            NirStmt::Return(match value {
                Some(value) => Some(lower_expr_with_async(ExprWithAsyncInput {
                    expr: value,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    signatures,
                    struct_table,
                    expected: expected.as_ref(),
                    allow_async_calls: false,
                })?),
                None => None,
            })
        }
        AstStmt::DestructureLet { .. } => return Err(
            "internal error: destructuring let must be lowered through statement-sequence lowering"
                .to_owned(),
        ),
    })
}

pub(super) struct StmtSequenceLoweringInput<'a> {
    pub(super) stmt: &'a AstStmt,
    pub(super) current_domain: &'a str,
    pub(super) current_function_is_async: bool,
    pub(super) bindings: &'a mut BTreeMap<String, NirTypeRef>,
    pub(super) module_consts: &'a BTreeMap<String, ModuleConstValue>,
    pub(super) return_type: Option<&'a AstTypeRef>,
    pub(super) type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) struct_table: &'a BTreeMap<String, NirStructDef>,
}

pub(super) fn lower_stmt_sequence_with_async(
    input: StmtSequenceLoweringInput<'_>,
) -> Result<Vec<NirStmt>, String> {
    let StmtSequenceLoweringInput {
        stmt,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
    } = input;
    for kind in [ControlExprKind::If, ControlExprKind::Match] {
        if let Some(expanded) = expand_nested_control_expr_stmt(stmt, kind)? {
            return lower_expanded_stmt_sequence_with_async(ExpandedStmtSequenceLoweringInput {
                original_stmt: stmt,
                expanded,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                return_type,
                type_aliases,
                signatures,
                struct_table,
            });
        }
    }
    if let Some(expanded) = expand_try_stmt(TryStmtExpansionInput {
        stmt,
        context: TryStmtExpansionContext {
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        },
    })? {
        return lower_expanded_stmt_sequence_with_async(ExpandedStmtSequenceLoweringInput {
            original_stmt: stmt,
            expanded,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        });
    }
    if let Some(expanded) = expand_nested_try_stmt(NestedTryStmtExpansionInput {
        stmt,
        context: TryExpansionContext {
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        },
    })? {
        return lower_expanded_stmt_sequence_with_async(ExpandedStmtSequenceLoweringInput {
            original_stmt: stmt,
            expanded,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            return_type,
            type_aliases,
            signatures,
            struct_table,
        });
    }
    if matches!(stmt, AstStmt::DestructureLet { .. }) {
        return lower_destructure_let_stmt_with_async(DestructureLetLoweringInput {
            stmt,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            type_aliases,
            signatures,
            struct_table,
        });
    }
    Ok(vec![lower_stmt_with_async(StmtLoweringInput {
        stmt,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
    })?])
}
