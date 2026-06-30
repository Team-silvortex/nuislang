use std::collections::BTreeMap;

use nuis_semantics::model::AstTypeRef;

use crate::frontend::call_helpers::{
    ensure_call_arg_matches_param, lower_extern_call_arg_for_param,
};
use crate::frontend::metadata::ModuleConstValue;
use crate::frontend::{
    i32_type, infer_nir_expr_type, lower_nested_expr_with_async_and_consts, AstExpr,
    FunctionSignature, NirExpr, NirStructDef, NirTypeRef,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_method_call_with_async(
    receiver: &AstExpr,
    method: &str,
    generic_args: &[AstTypeRef],
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    _expected: Option<&NirTypeRef>,
    allow_async_calls: bool,
) -> Result<NirExpr, String> {
    let receiver_expected = crate::frontend::receiver_expected::explicit_receiver_expected_nir_type(
        receiver,
        generic_args,
    );
    if let Some(receiver_name) = crate::frontend::render_field_access_path(receiver) {
        let signature_key = format!("{receiver_name}.{method}");
        if let Some(signature) = signatures.get(&signature_key) {
            if !generic_args.is_empty() {
                return Err(format!(
                            "method `{signature_key}` does not accept explicit generic arguments in the current frontend"
                        ));
            }
            let lowered_args = args
                .iter()
                .zip(signature.params.iter())
                .map(|(arg, expected_param)| {
                    lower_nested_expr_with_async_and_consts(
                        arg,
                        current_domain,
                        current_function_is_async,
                        bindings,
                        module_consts,
                        signatures,
                        struct_table,
                        Some(expected_param),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            if signature.params.len() != lowered_args.len() {
                return Err(format!(
                    "method `{signature_key}` expects {} args, found {}",
                    signature.params.len(),
                    lowered_args.len()
                ));
            }
            for (index, (arg, expected_param)) in
                lowered_args.iter().zip(signature.params.iter()).enumerate()
            {
                ensure_call_arg_matches_param(
                    &signature_key,
                    index,
                    arg,
                    expected_param,
                    bindings,
                    signatures,
                    struct_table,
                    signature.is_extern,
                )?;
            }
            if signature.is_extern {
                if current_domain != "cpu" {
                    return Err(format!(
                                "extern method `{signature_key}` is currently only allowed inside `mod cpu <unit>`"
                            ));
                }
                let lowered_args = lowered_args
                    .into_iter()
                    .zip(signature.params.iter())
                    .map(|(arg, expected_param)| {
                        lower_extern_call_arg_for_param(arg, expected_param)
                    })
                    .collect();
                if signature.return_type.as_ref() == Some(&i32_type()) {
                    return Ok(NirExpr::CpuExternCallI32 {
                        abi: signature.abi.clone(),
                        interface: signature.interface.clone(),
                        callee: signature.symbol_name.clone(),
                        args: lowered_args,
                    });
                }
                return Ok(NirExpr::CpuExternCall {
                    abi: signature.abi.clone(),
                    interface: signature.interface.clone(),
                    callee: signature.symbol_name.clone(),
                    args: lowered_args,
                });
            }
            if signature.is_async {
                if !current_function_is_async {
                    return Err(format!(
                        "async function `{signature_key}` can only be called inside `async fn`"
                    ));
                }
                if !allow_async_calls {
                    return Err(format!(
                        "async function `{signature_key}` must be used under `await`"
                    ));
                }
            }
            return Ok(NirExpr::Call {
                callee: signature.symbol_name.clone(),
                args: lowered_args,
            });
        }
        let is_shadowed_simple_local =
            matches!(receiver, AstExpr::Var(name) if bindings.contains_key(name));
        if !is_shadowed_simple_local && !args.is_empty() {
            let Some(first_arg_expr) = args.first() else {
                return Err(format!(
                    "trait method `{signature_key}` expects at least 1 arg"
                ));
            };
            let lowered_first_arg = lower_nested_expr_with_async_and_consts(
                first_arg_expr,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                None,
            )?;
            if let Some(receiver_ty) =
                infer_nir_expr_type(&lowered_first_arg, bindings, signatures, struct_table)
            {
                if !generic_args.is_empty() && receiver_ty.generic_args.len() != generic_args.len()
                {
                    return Err(format!(
                                "trait method `{signature_key}` for `{}` expects {} explicit receiver generic argument(s), found {}",
                                receiver_ty.render(),
                                receiver_ty.generic_args.len(),
                                generic_args.len()
                            ));
                }
                for symbol_name in
                    crate::frontend::impl_method_symbol_names(&receiver_name, &receiver_ty, method)
                {
                    if let Some(signature) = signatures.get(&symbol_name) {
                        let mut lowered_args = vec![lowered_first_arg.clone()];
                        lowered_args.extend(
                            args.iter()
                                .skip(1)
                                .zip(signature.params.iter().skip(1))
                                .map(|(arg, expected_param)| {
                                    lower_nested_expr_with_async_and_consts(
                                        arg,
                                        current_domain,
                                        current_function_is_async,
                                        bindings,
                                        module_consts,
                                        signatures,
                                        struct_table,
                                        Some(expected_param),
                                    )
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                        );
                        if signature.params.len() != lowered_args.len() {
                            return Err(format!(
                                "trait method `{signature_key}` for `{}` expects {} args, found {}",
                                receiver_ty.render(),
                                signature.params.len(),
                                lowered_args.len()
                            ));
                        }
                        return Ok(NirExpr::Call {
                            callee: signature.symbol_name.clone(),
                            args: lowered_args,
                        });
                    }
                }
                return Err(format!(
                    "trait method `{signature_key}` has no impl for `{}`",
                    receiver_ty.render()
                ));
            }
        }
    }
    let lowered_receiver = lower_nested_expr_with_async_and_consts(
        receiver,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        receiver_expected.as_ref(),
    )?;
    if let Some(receiver_ty) =
        infer_nir_expr_type(&lowered_receiver, bindings, signatures, struct_table)
    {
        if !generic_args.is_empty() {
            if receiver_ty.generic_args.len() != generic_args.len() {
                return Err(format!(
                            "method `{method}` for `{}` expects {} explicit receiver generic argument(s), found {}",
                            receiver_ty.render(),
                            receiver_ty.generic_args.len(),
                            generic_args.len()
                        ));
            }
            let lowered_explicit = generic_args
                .iter()
                .map(crate::frontend::lower_type_ref)
                .collect::<Vec<_>>();
            if lowered_explicit != receiver_ty.generic_args {
                return Err(format!(
                            "method `{method}` explicit receiver generic arguments `<{}>` do not match inferred receiver type `{}`",
                            lowered_explicit
                                .iter()
                                .map(|ty| ty.render())
                                .collect::<Vec<_>>()
                                .join(", "),
                            receiver_ty.render()
                        ));
            }
        }
        for signature_key in crate::frontend::impl_method_lookup_keys(&receiver_ty, method) {
            if let Some(signature) = signatures.get(&signature_key) {
                let mut lowered_args = vec![lowered_receiver.clone()];
                lowered_args.extend(
                    args.iter()
                        .zip(signature.params.iter().skip(1))
                        .map(|(arg, expected_param)| {
                            lower_nested_expr_with_async_and_consts(
                                arg,
                                current_domain,
                                current_function_is_async,
                                bindings,
                                module_consts,
                                signatures,
                                struct_table,
                                Some(expected_param),
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                );
                if signature.params.len() != lowered_args.len() {
                    return Err(format!(
                        "method `{}` for `{}` expects {} args, found {}",
                        method,
                        receiver_ty.render(),
                        signature.params.len(),
                        lowered_args.len()
                    ));
                }
                return Ok(NirExpr::Call {
                    callee: signature.symbol_name.clone(),
                    args: lowered_args,
                });
            }
        }
    }
    Ok(NirExpr::MethodCall {
        receiver: Box::new(lowered_receiver),
        method: method.to_owned(),
        args: args
            .iter()
            .map(|arg| {
                lower_nested_expr_with_async_and_consts(
                    arg,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    signatures,
                    struct_table,
                    None,
                )
            })
            .collect::<Result<Vec<_>, _>>()?,
    })
}
