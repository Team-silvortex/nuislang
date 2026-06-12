use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::call_helpers::ensure_call_arg_matches_param;
use super::{
    ensure_ref_like, lower_expr, lower_nested_expr_with_async, FunctionSignature, ModuleConstValue,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_direct_call_builtin_or_named_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    _module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    allow_async_calls: bool,
) -> Result<Option<NirExpr>, String> {
    match callee {
        "free" => {
            let [value] = args else {
                return Err("free(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            ensure_ref_like("free", &lowered, bindings, signatures, struct_table)?;
            Ok(Some(NirExpr::Free(Box::new(lowered))))
        }
        "is_null" => {
            let [value] = args else {
                return Err("is_null(...) expects 1 arg".to_owned());
            };
            let lowered = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                None,
            )?;
            ensure_ref_like("is_null", &lowered, bindings, signatures, struct_table)?;
            Ok(Some(NirExpr::IsNull(Box::new(lowered))))
        }
        _ => lower_named_call(
            callee,
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            allow_async_calls,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn lower_named_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    allow_async_calls: bool,
) -> Result<Option<NirExpr>, String> {
    let Some(signature) = signatures.get(callee) else {
        return Ok(None);
    };
    let lowered_args = args
        .iter()
        .zip(signature.params.iter())
        .map(|(arg, expected_param)| {
            lower_nested_expr_with_async(
                arg,
                current_domain,
                current_function_is_async,
                bindings,
                signatures,
                struct_table,
                Some(expected_param),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    if signature.params.len() != lowered_args.len() {
        return Err(format!(
            "function `{callee}` expects {} args, found {}",
            signature.params.len(),
            lowered_args.len()
        ));
    }
    for (index, (arg, expected_param)) in
        lowered_args.iter().zip(signature.params.iter()).enumerate()
    {
        ensure_call_arg_matches_param(
            callee,
            index,
            arg,
            expected_param,
            bindings,
            signatures,
            struct_table,
            signature.is_extern,
        )?;
    }
    if signature.is_async {
        if !current_function_is_async {
            return Err(format!(
                "async function `{callee}` can only be called inside `async fn`"
            ));
        }
        if !allow_async_calls {
            return Err(format!(
                "async function `{callee}` must be used under `await`"
            ));
        }
    }
    if signature.is_extern {
        if current_domain != "cpu" {
            return Err(format!(
                "extern call `{callee}` is currently only allowed inside `mod cpu <unit>`"
            ));
        }
        return Ok(Some(NirExpr::CpuExternCall {
            abi: signature.abi.clone(),
            interface: None,
            callee: signature.symbol_name.clone(),
            args: lowered_args,
        }));
    }
    Ok(Some(NirExpr::Call {
        callee: signature.symbol_name.clone(),
        args: lowered_args,
    }))
}
