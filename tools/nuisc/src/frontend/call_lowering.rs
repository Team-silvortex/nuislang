use std::collections::BTreeMap;

use nuis_semantics::model::{AstBinaryOp, AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::{
    lower_binary_expr_with_async, lower_direct_call_builtin_or_named_call,
    lower_routed_call_or_core_builtin, FunctionSignature, ModuleConstValue,
};

#[allow(dead_code)]
pub(super) fn lower_binary_expr(
    op: &AstBinaryOp,
    lhs: &AstExpr,
    rhs: &AstExpr,
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirExpr, String> {
    lower_binary_expr_with_async(
        op,
        lhs,
        rhs,
        current_domain,
        false,
        bindings,
        &BTreeMap::new(),
        signatures,
        struct_table,
    )
}

#[allow(dead_code)]
pub(super) fn lower_call_expr(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
) -> Result<NirExpr, String> {
    lower_call_expr_with_async(
        callee,
        args,
        current_domain,
        false,
        bindings,
        &BTreeMap::new(),
        signatures,
        struct_table,
        expected,
        false,
    )
}

pub(super) fn lower_call_expr_with_async(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
    allow_async_calls: bool,
) -> Result<NirExpr, String> {
    if let Some(routed_or_core) = lower_routed_call_or_core_builtin(
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        expected,
    )? {
        return Ok(routed_or_core);
    }
    match callee {
        _ => lower_direct_call_builtin_or_named_call(
            callee,
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            allow_async_calls,
        )?
        .ok_or_else(|| format!("unknown function `{callee}`")),
    }
}
