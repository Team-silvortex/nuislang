use std::collections::{BTreeMap, BTreeSet};

use super::binary_lowering::lower_binary_expr_with_async;
use super::call_helpers::ensure_call_arg_matches_param;
use super::metadata::{hidden_private_field_count, ModuleConstValue};
use super::name_suggestions::suggest_similar_name;
use super::unary_lowering::lower_unary_expr_with_async;
use super::validation_helpers::render_type_name;
use super::{
    infer_nir_expr_type, instantiate_struct_field_type, lower_call_expr_with_async, named_type,
    resolve_declared_or_inferred, struct_field_type, AstExpr, FunctionSignature, NirExpr,
    NirStructDef, NirTypeRef,
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
        AstExpr::Float(value) => match expected {
            Some(expected) if expected.name == "f32" && !expected.is_ref && !expected.is_optional => {
                NirExpr::F32(value.clone())
            }
            Some(expected) if expected.name == "f64" && !expected.is_ref && !expected.is_optional => {
                NirExpr::F64(value.clone())
            }
            Some(expected) => {
                return Err(format!(
                    "float literal `{value}` cannot lower to expected type `{}`",
                    render_type_name(expected)
                ))
            }
            None => NirExpr::F64(value.clone()),
        },
        AstExpr::If { .. } => {
            return Err(
                "`if` expression is currently only supported as the direct value of `let`, `const`, `print`, or `return`"
                    .to_owned(),
            )
        }
        AstExpr::Match { .. } => {
            return Err(
                "`match` expression is currently only supported as the direct value of `let`, `const`, `print`, or `return`"
                    .to_owned(),
            )
        }
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
                    "function symbol `{name}` cannot currently be used as a first-class value; pass it only to `Fn1<...>`/`Fn2<...>`/`Fn3<...>` higher-order parameters or invoke it directly"
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
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } => lower_call_expr_with_async(
            callee,
            generic_args,
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
            if let Some(receiver_name) = super::render_field_access_path(receiver) {
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
                let is_shadowed_simple_local = matches!(
                    receiver.as_ref(),
                    AstExpr::Var(name) if bindings.contains_key(name)
                );
                if !is_shadowed_simple_local {
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
                    if let Some(first_arg) = lowered_args.first() {
                        if let Some(receiver_ty) =
                            infer_nir_expr_type(first_arg, bindings, signatures, struct_table)
                        {
                            let symbol_name =
                                super::impl_method_symbol_name(&receiver_name, &receiver_ty, method);
                            if let Some(signature) = signatures.get(&symbol_name) {
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
                            return Err(format!(
                                "trait method `{signature_key}` has no impl for `{}`",
                                receiver_ty.render()
                            ));
                        }
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
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => {
            let definition = struct_table
                .get(type_name)
                .ok_or_else(|| format!("unknown struct type `{}`", type_name))?;
            let literal_ty = if definition.generic_params.is_empty() {
                if !type_args.is_empty() {
                    return Err(format!(
                        "struct literal `{}` does not accept explicit generic arguments because struct `{}` is not generic",
                        type_name, type_name
                    ));
                }
                named_type(type_name)
            } else if !type_args.is_empty() {
                if type_args.len() != definition.generic_params.len() {
                    return Err(format!(
                        "struct literal `{}<...>` expects {} generic argument(s), found {}",
                        type_name,
                        definition.generic_params.len(),
                        type_args.len()
                    ));
                }
                NirTypeRef {
                    name: type_name.clone(),
                    generic_args: type_args.iter().map(super::lower_type_ref).collect(),
                    is_optional: false,
                    is_ref: false,
                }
            } else if let Some(expected) = expected {
                if expected.name != *type_name {
                    return Err(format!(
                        "cannot infer generic arguments for struct literal `{}` from expected type `{}`",
                        type_name,
                        expected.render()
                    ));
                }
                expected.clone()
            } else {
                infer_generic_struct_literal_type_from_fields(
                    type_name,
                    definition,
                    fields,
                    current_domain,
                    current_function_is_async,
                    bindings,
                    module_consts,
                    signatures,
                    struct_table,
                )?
            };
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
                    Some(&instantiate_struct_field_type(&literal_ty, definition, &field.ty)),
                )?;
                let inferred = infer_nir_expr_type(&lowered, bindings, signatures, struct_table);
                let expected_field_ty =
                    instantiate_struct_field_type(&literal_ty, definition, &field.ty);
                let _ = resolve_declared_or_inferred(name, Some(expected_field_ty), inferred)?;
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
                type_args: literal_ty.generic_args,
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
            if base_ty.is_ref && !base_ty.is_optional && base_ty.name == "Node" {
                return Ok(match field.as_str() {
                    "value" => NirExpr::LoadValue(Box::new(lowered_base)),
                    "next" => NirExpr::LoadNext(Box::new(lowered_base)),
                    _ => {
                        return Err(format!(
                            "type `{}` has no field `{}`; pointer field sugar currently supports only `value` and `next`",
                            render_type_name(&base_ty),
                            field
                        ))
                    }
                });
            }
            if base_ty.is_ref && !base_ty.is_optional && base_ty.name == "Buffer" {
                return Ok(match field.as_str() {
                    "len" => NirExpr::BufferLen(Box::new(lowered_base)),
                    _ => {
                        return Err(format!(
                            "type `{}` has no field `{}`; buffer field sugar currently supports only `len`",
                            render_type_name(&base_ty),
                            field
                        ))
                    }
                });
            }
            if struct_field_type(&base_ty, field, struct_table).is_none() {
                if let Some(suggested_field) =
                    suggest_struct_field_name(&base_ty, field, struct_table)
                {
                    return Err(format!(
                        "type `{}` has no field `{}`; did you mean `{}`?",
                        render_type_name(&base_ty),
                        field,
                        suggested_field
                    ));
                }
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
        AstExpr::Unary { op, operand } => lower_unary_expr_with_async(
            op,
            operand,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            expected,
        )?,
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
            expected,
        )?,
    })
}

fn suggest_struct_field_name(
    base_ty: &NirTypeRef,
    field: &str,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Option<String> {
    let definition = struct_table.get(&base_ty.name)?;
    let candidates = definition
        .fields
        .iter()
        .map(|item| item.name.clone())
        .collect::<BTreeSet<_>>();
    suggest_similar_name(field, &candidates)
}

#[allow(clippy::too_many_arguments)]
fn infer_generic_struct_literal_type_from_fields(
    type_name: &str,
    definition: &NirStructDef,
    fields: &[(String, AstExpr)],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirTypeRef, String> {
    let mut substitutions = BTreeMap::<String, NirTypeRef>::new();
    let generic_names = definition
        .generic_params
        .iter()
        .map(|param| param.name.as_str())
        .collect::<Vec<_>>();
    for (name, value) in fields {
        let field = definition
            .fields
            .iter()
            .find(|field| field.name == *name)
            .ok_or_else(|| format!("struct `{}` has no field `{}`", type_name, name))?;
        let lowered = lower_nested_expr_with_async_and_consts(
            value,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            None,
        )?;
        let Some(inferred) = infer_nir_expr_type(&lowered, bindings, signatures, struct_table)
        else {
            return Err(format!(
                "cannot infer generic arguments for struct literal `{}` in the current frontend; add an explicit expected type",
                type_name
            ));
        };
        unify_generic_struct_field_type_pattern(
            &field.ty,
            &inferred,
            &generic_names,
            &mut substitutions,
            type_name,
        )?;
    }
    let generic_args = definition
        .generic_params
        .iter()
        .map(|param| {
            substitutions.get(&param.name).cloned().ok_or_else(|| {
                format!(
                    "cannot infer generic arguments for struct literal `{}` in the current frontend; add an explicit expected type",
                    type_name
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(NirTypeRef {
        name: type_name.to_owned(),
        generic_args,
        is_optional: false,
        is_ref: false,
    })
}

fn unify_generic_struct_field_type_pattern(
    pattern: &NirTypeRef,
    concrete: &NirTypeRef,
    generic_names: &[&str],
    substitutions: &mut BTreeMap<String, NirTypeRef>,
    type_name: &str,
) -> Result<(), String> {
    if pattern.generic_args.is_empty()
        && !pattern.is_optional
        && !pattern.is_ref
        && generic_names.contains(&pattern.name.as_str())
    {
        if let Some(existing) = substitutions.get(&pattern.name) {
            if existing.render() != concrete.render() {
                return Err(format!(
                    "struct literal `{}` inferred conflicting types `{}` and `{}` for generic parameter `{}`",
                    type_name,
                    existing.render(),
                    concrete.render(),
                    pattern.name
                ));
            }
        } else {
            substitutions.insert(pattern.name.clone(), concrete.clone());
        }
        return Ok(());
    }
    if pattern.name != concrete.name
        || pattern.generic_args.len() != concrete.generic_args.len()
        || pattern.is_optional != concrete.is_optional
        || pattern.is_ref != concrete.is_ref
    {
        return Err(format!(
            "cannot infer generic arguments for struct literal `{}` in the current frontend; add an explicit expected type",
            type_name
        ));
    }
    for (pattern_arg, concrete_arg) in pattern.generic_args.iter().zip(&concrete.generic_args) {
        unify_generic_struct_field_type_pattern(
            pattern_arg,
            concrete_arg,
            generic_names,
            substitutions,
            type_name,
        )?;
    }
    Ok(())
}
