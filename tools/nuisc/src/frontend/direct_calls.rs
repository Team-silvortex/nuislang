use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::call_helpers::{
    ensure_call_arg_matches_param, lower_extern_call_arg_for_param, CallArgParamCheck,
};
use super::{
    ensure_ref_like, i32_type, lower_expr, lower_nested_expr_with_async, FunctionSignature,
    ModuleConstValue,
};

#[path = "direct_calls_buffer.rs"]
mod direct_calls_buffer;
#[path = "direct_calls_http.rs"]
mod direct_calls_http;
#[path = "direct_calls_serialization.rs"]
mod direct_calls_serialization;
#[path = "direct_calls_text.rs"]
mod direct_calls_text;

#[derive(Clone, Copy)]
pub(super) struct DirectCallLoweringContext<'a> {
    pub(super) current_domain: &'a str,
    pub(super) bindings: &'a BTreeMap<String, NirTypeRef>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) struct_table: &'a BTreeMap<String, NirStructDef>,
}

pub(super) struct DirectCallBuiltinInput<'a> {
    pub(super) callee: &'a str,
    pub(super) args: &'a [AstExpr],
    pub(super) current_domain: &'a str,
    pub(super) current_function_is_async: bool,
    pub(super) bindings: &'a BTreeMap<String, NirTypeRef>,
    pub(super) _module_consts: &'a BTreeMap<String, ModuleConstValue>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) struct_table: &'a BTreeMap<String, NirStructDef>,
    pub(super) allow_async_calls: bool,
}

struct NamedCallLoweringInput<'a> {
    callee: &'a str,
    args: &'a [AstExpr],
    current_domain: &'a str,
    current_function_is_async: bool,
    bindings: &'a BTreeMap<String, NirTypeRef>,
    signatures: &'a BTreeMap<String, FunctionSignature>,
    struct_table: &'a BTreeMap<String, NirStructDef>,
    allow_async_calls: bool,
}

pub(super) fn lower_direct_call_builtin_or_named_call(
    input: DirectCallBuiltinInput<'_>,
) -> Result<Option<NirExpr>, String> {
    let DirectCallBuiltinInput {
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        _module_consts,
        signatures,
        struct_table,
        allow_async_calls,
    } = input;
    let context = DirectCallLoweringContext {
        current_domain,
        bindings,
        signatures,
        struct_table,
    };
    if let Some(lowered) =
        direct_calls_serialization::lower_serialization_call(callee, args, context)?
    {
        return Ok(Some(lowered));
    }
    if let Some(lowered) = direct_calls_http::lower_http_call(callee, args, context)? {
        return Ok(Some(lowered));
    }
    if let Some(lowered) = direct_calls_text::lower_text_call(callee, args, context)? {
        return Ok(Some(lowered));
    }
    if let Some(lowered) = direct_calls_buffer::lower_buffer_call(callee, args, context)? {
        return Ok(Some(lowered));
    }

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
        _ => lower_named_call(NamedCallLoweringInput {
            callee,
            args,
            current_domain,
            current_function_is_async,
            bindings,
            signatures,
            struct_table,
            allow_async_calls,
        }),
    }
}

fn lower_named_call(input: NamedCallLoweringInput<'_>) -> Result<Option<NirExpr>, String> {
    let NamedCallLoweringInput {
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        signatures,
        struct_table,
        allow_async_calls,
    } = input;
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
        ensure_call_arg_matches_param(CallArgParamCheck {
            callee,
            arg_index: index,
            arg,
            expected_param,
            bindings,
            signatures,
            struct_table,
            is_extern: signature.is_extern,
        })?;
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
        let lowered_args = lowered_args
            .into_iter()
            .zip(signature.params.iter())
            .map(|(arg, expected_param)| lower_extern_call_arg_for_param(arg, expected_param))
            .collect();
        if signature.return_type.as_ref() == Some(&i32_type()) {
            return Ok(Some(NirExpr::CpuExternCallI32 {
                abi: signature.abi.clone(),
                interface: None,
                callee: signature.symbol_name.clone(),
                args: lowered_args,
            }));
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
