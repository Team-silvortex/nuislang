use std::collections::{BTreeMap, BTreeSet};

use super::binary_lowering::lower_binary_expr_with_async;
use super::metadata::{hidden_private_field_count, ModuleConstValue};
use super::validation_helpers::render_type_name;
use super::{
    infer_nir_expr_type, lower_call_expr_with_async, resolve_declared_or_inferred,
    struct_field_type, AstExpr, FunctionSignature, NirExpr, NirStructDef, NirTypeRef,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_expr(
    expr: &AstExpr,
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
) -> Result<NirExpr, String> {
    lower_expr_with_async(
        expr,
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

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_nested_expr_with_async(
    expr: &AstExpr,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
) -> Result<NirExpr, String> {
    lower_expr_with_async(
        expr,
        current_domain,
        current_function_is_async,
        bindings,
        &BTreeMap::new(),
        signatures,
        struct_table,
        expected,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_nested_expr_with_async_and_consts(
    expr: &AstExpr,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
) -> Result<NirExpr, String> {
    lower_expr_with_async(
        expr,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        expected,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_expr_with_async(
    expr: &AstExpr,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
    allow_async_calls: bool,
) -> Result<NirExpr, String> {
    Ok(match expr {
        AstExpr::Bool(value) => NirExpr::Bool(*value),
        AstExpr::Text(text) => NirExpr::Text(text.clone()),
        AstExpr::Int(value) => NirExpr::Int(*value),
        AstExpr::Lambda { .. } => {
            return Err(
                "internal frontend error: lambda expression should have been expanded before NIR lowering"
                    .to_owned(),
            )
        }
        AstExpr::Invoke { .. } => {
            return Err(
                "internal frontend error: invoke expression should have been rewritten before NIR lowering"
                    .to_owned(),
            )
        }
        AstExpr::Var(name) => {
            if let Some(constant) = module_consts.get(name) {
                constant.value.clone()
            } else if bindings.contains_key(name) {
                NirExpr::Var(name.clone())
            } else if signatures.contains_key(name) {
                return Err(format!(
                    "function symbol `{name}` cannot currently be used as a first-class value; pass it only to `Fn1<...>`/`Fn2<...>` higher-order parameters or invoke it directly"
                ));
            } else {
                return Err(format!("unknown value `{name}`"));
            }
        }
        AstExpr::Await(value) => {
            if !current_function_is_async {
                return Err("`await` is only allowed inside `async fn`".to_owned());
            }
            NirExpr::Await(Box::new(lower_expr_with_async(
                value,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                expected,
                true,
            )?))
        }
        AstExpr::Instantiate { domain, unit } => {
            if current_domain != "cpu" {
                return Err(format!(
                    "instantiate {} {} is only allowed inside `mod cpu <unit>` in the current frontend",
                    domain, unit
                ));
            }
            NirExpr::Instantiate {
                domain: domain.clone(),
                unit: unit.clone(),
            }
        }
        AstExpr::Call { callee, args } => lower_call_expr_with_async(
            callee,
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            expected,
            allow_async_calls,
        )?,
        AstExpr::MethodCall {
            receiver,
            method,
            args,
        } => {
            if let AstExpr::Var(receiver_name) = receiver.as_ref() {
                let signature_key = format!("{receiver_name}.{method}");
                if let Some(signature) = signatures.get(&signature_key) {
                    let lowered_args = args
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
                        .collect::<Result<Vec<_>, _>>()?;
                    if signature.params.len() != lowered_args.len() {
                        return Err(format!(
                            "method `{signature_key}` expects {} args, found {}",
                            signature.params.len(),
                            lowered_args.len()
                        ));
                    }
                    if signature.is_extern {
                        if current_domain != "cpu" {
                            return Err(format!(
                                "extern method `{signature_key}` is currently only allowed inside `mod cpu <unit>`"
                            ));
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
            }
            let lowered_receiver = lower_nested_expr_with_async_and_consts(
                receiver,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                None,
            )?;
            if let Some(receiver_ty) =
                infer_nir_expr_type(&lowered_receiver, bindings, signatures, struct_table)
            {
                let signature_key = super::impl_method_lookup_key(&receiver_ty, method);
                if let Some(signature) = signatures.get(&signature_key) {
                    let mut lowered_args = vec![lowered_receiver.clone()];
                    lowered_args.extend(
                        args.iter()
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
            NirExpr::MethodCall {
                receiver: Box::new(lowered_receiver),
                method: method.clone(),
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
            }
        }
        AstExpr::StructLiteral { type_name, fields } => {
            let definition = struct_table
                .get(type_name)
                .ok_or_else(|| format!("unknown struct type `{}`", type_name))?;
            let hidden_private_fields = hidden_private_field_count(definition);
            if hidden_private_fields > 0 {
                return Err(format!(
                    "struct literal `{}` cannot be constructed outside its defining module because it hides {} private field(s)",
                    type_name, hidden_private_fields
                ));
            }
            let mut seen = BTreeSet::new();
            let mut lowered_fields = Vec::new();
            for (name, value) in fields {
                let field = definition
                    .fields
                    .iter()
                    .find(|field| field.name == *name)
                    .ok_or_else(|| format!("struct `{}` has no field `{}`", type_name, name))?;
                if !seen.insert(name.clone()) {
                    return Err(format!(
                        "struct literal `{}` duplicates field `{}`",
                        type_name, name
                    ));
                }
                let lowered = lower_nested_expr_with_async_and_consts(
                    value,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    signatures,
                    struct_table,
                    Some(&field.ty),
                )?;
                let inferred = infer_nir_expr_type(&lowered, bindings, signatures, struct_table);
                let _ = resolve_declared_or_inferred(name, Some(field.ty.clone()), inferred)?;
                lowered_fields.push((name.clone(), lowered));
            }
            if definition.fields.len() != lowered_fields.len() {
                return Err(format!(
                    "struct literal `{}` must initialize all {} field(s)",
                    type_name,
                    definition.fields.len()
                ));
            }
            NirExpr::StructLiteral {
                type_name: type_name.clone(),
                fields: lowered_fields,
            }
        }
        AstExpr::FieldAccess { base, field } => {
            let lowered_base = lower_nested_expr_with_async_and_consts(
                base,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                None,
            )?;
            let base_ty = infer_nir_expr_type(&lowered_base, bindings, signatures, struct_table)
                .ok_or_else(|| format!("cannot infer base type for field access `.{} `", field))?;
            if struct_field_type(&base_ty, field, struct_table).is_none() {
                return Err(format!(
                    "type `{}` has no field `{}`",
                    render_type_name(&base_ty),
                    field
                ));
            }
            NirExpr::FieldAccess {
                base: Box::new(lowered_base),
                field: field.clone(),
            }
        }
        AstExpr::Binary { op, lhs, rhs } => lower_binary_expr_with_async(
            op,
            lhs,
            rhs,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
        )?,
    })
}
