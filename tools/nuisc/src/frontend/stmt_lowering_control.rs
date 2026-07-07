use std::collections::BTreeMap;

use nuis_semantics::model::AstMatchArm;

use super::match_lowering::{lower_match_stmt_with_async, MatchStmtLoweringInput};
use super::metadata::ModuleConstValue;
use super::stmt_lowering::{lower_stmt_block_with_async, StmtBlockLoweringInput};
use super::stmt_lowering_control_rewrite::rewrite_control_expr_terminal_branch;
pub(super) use super::stmt_lowering_control_rewrite::{
    expand_nested_control_expr_stmt, ControlExprKind,
};
use super::{
    bool_type, lower_expr_with_async, AstStmt, AstTypeAlias, AstTypeRef, ExprWithAsyncInput,
    FunctionSignature, NirStmt, NirStructDef, NirTypeRef,
};

pub(super) struct ControlExprLoweringContext<'a> {
    pub(super) current_domain: &'a str,
    pub(super) current_function_is_async: bool,
    pub(super) bindings: &'a mut BTreeMap<String, NirTypeRef>,
    pub(super) module_consts: &'a BTreeMap<String, ModuleConstValue>,
    pub(super) return_type: Option<&'a AstTypeRef>,
    pub(super) type_aliases: &'a BTreeMap<String, AstTypeAlias>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) struct_table: &'a BTreeMap<String, NirStructDef>,
    pub(super) wrap_terminal: &'a dyn Fn(super::AstExpr) -> AstStmt,
}

pub(super) struct IfExprStmtLoweringInput<'a> {
    pub(super) condition: &'a super::AstExpr,
    pub(super) then_body: &'a [AstStmt],
    pub(super) else_body: &'a [AstStmt],
    pub(super) context: ControlExprLoweringContext<'a>,
}

pub(super) struct MatchExprStmtLoweringInput<'a> {
    pub(super) value: &'a super::AstExpr,
    pub(super) arms: &'a [AstMatchArm],
    pub(super) context: ControlExprLoweringContext<'a>,
}

pub(super) fn lower_if_expr_stmt_with_context(
    input: IfExprStmtLoweringInput<'_>,
) -> Result<NirStmt, String> {
    let IfExprStmtLoweringInput {
        condition,
        then_body,
        else_body,
        context,
    } = input;
    let ControlExprLoweringContext {
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
        wrap_terminal,
    } = context;
    let condition = lower_expr_with_async(ExprWithAsyncInput {
        expr: condition,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        expected: Some(&bool_type()),
        allow_async_calls: false,
    })?;
    let mut then_bindings = bindings.clone();
    let mut else_bindings = bindings.clone();
    Ok(NirStmt::If {
        condition,
        then_body: lower_if_expr_branch_with_async(
            then_body,
            ControlExprLoweringContext {
                current_domain,
                current_function_is_async,
                bindings: &mut then_bindings,
                module_consts,
                return_type,
                type_aliases,
                signatures,
                struct_table,
                wrap_terminal,
            },
        )?,
        else_body: lower_if_expr_branch_with_async(
            else_body,
            ControlExprLoweringContext {
                current_domain,
                current_function_is_async,
                bindings: &mut else_bindings,
                module_consts,
                return_type,
                type_aliases,
                signatures,
                struct_table,
                wrap_terminal,
            },
        )?,
    })
}

fn lower_if_expr_branch_with_async(
    body: &[AstStmt],
    context: ControlExprLoweringContext<'_>,
) -> Result<Vec<NirStmt>, String> {
    let ControlExprLoweringContext {
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
        wrap_terminal,
    } = context;
    let rewritten_body =
        rewrite_control_expr_terminal_branch(body, wrap_terminal, ControlExprKind::If)?;
    lower_stmt_block_with_async(StmtBlockLoweringInput {
        stmts: &rewritten_body,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
    })
}

pub(super) fn lower_match_expr_stmt_with_context(
    input: MatchExprStmtLoweringInput<'_>,
) -> Result<NirStmt, String> {
    let MatchExprStmtLoweringInput {
        value,
        arms,
        context,
    } = input;
    let ControlExprLoweringContext {
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
        wrap_terminal,
    } = context;
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
    lower_match_stmt_with_async(MatchStmtLoweringInput {
        value,
        arms: &rewritten_arms,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        return_type,
        type_aliases,
        signatures,
        struct_table,
    })
}
