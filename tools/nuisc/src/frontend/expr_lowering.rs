use std::collections::{BTreeMap, BTreeSet};

use super::binary_lowering::lower_binary_expr_with_async;
use super::call_helpers::{ensure_call_arg_matches_param, lower_extern_call_arg_for_param};
use super::metadata::{hidden_private_field_count, ModuleConstValue};
use super::name_suggestions::suggest_similar_name;
use super::unary_lowering::lower_unary_expr_with_async;
use super::validation_helpers::render_type_name;
use super::{
    infer_nir_expr_type, instantiate_struct_field_type, lower_call_expr_with_async, named_type,
    resolve_declared_or_inferred, struct_field_type, AstExpr, FunctionSignature,
    NirExpr, NirStructDef, NirTypeRef,
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
            } else if expected.is_some_and(|ty| {
                !ty.is_optional
                    && !ty.is_ref
                    && matches!(
                        (ty.name.as_str(), ty.generic_args.len()),
                        ("Fn1", 2) | ("Fn2", 3) | ("Fn3", 4)
                    )
            }) && signatures.contains_key(name)
            {
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
        AstExpr::Try(_) => {
            return Err(
                "`?` is currently only supported as the direct value of `let`, `const`, `print`, `return`, or expression statements"
                    .to_owned(),
            )
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
            generic_args,
            args,
        } => {
            let receiver_expected =
                super::receiver_expected::explicit_receiver_expected_nir_type(receiver, generic_args);
            if let Some(receiver_name) = super::render_field_access_path(receiver) {
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
                if !is_shadowed_simple_local && !args.is_empty() {
                    let Some(first_arg_expr) = args.first() else {
                        return Err(format!("trait method `{signature_key}` expects at least 1 arg"));
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
                        if !generic_args.is_empty()
                            && receiver_ty.generic_args.len() != generic_args.len()
                        {
                            return Err(format!(
                                "trait method `{signature_key}` for `{}` expects {} explicit receiver generic argument(s), found {}",
                                receiver_ty.render(),
                                receiver_ty.generic_args.len(),
                                generic_args.len()
                            ));
                        }
                        for symbol_name in
                            super::impl_method_symbol_names(&receiver_name, &receiver_ty, method)
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
                        .map(super::lower_type_ref)
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
                for signature_key in super::impl_method_lookup_keys(&receiver_ty, method) {
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
                let expected_matches_parent = expected
                    .name
                    .eq(type_name.rsplit_once('.').map(|(parent, _)| parent).unwrap_or_default());
                if expected.name != *type_name && !expected_matches_parent {
                    return Err(format!(
                        "cannot infer generic arguments for struct literal `{}` from expected type `{}`",
                        type_name,
                        expected.render()
                    ));
                }
                NirTypeRef {
                    name: type_name.clone(),
                    generic_args: expected.generic_args.clone(),
                    is_optional: false,
                    is_ref: false,
                }
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
            if let Some(base_path) = super::render_field_access_path(base) {
                let qualified_name = format!("{base_path}.{field}");
                if let Some(definition) = struct_table.get(&qualified_name) {
                    if definition.fields.is_empty() {
                        return Ok(NirExpr::StructLiteral {
                            type_name: qualified_name,
                            type_args: if let Some(expected) = expected {
                                if expected.generic_args.len() == definition.generic_params.len() {
                                    expected.generic_args.clone()
                                } else {
                                    Vec::new()
                                }
                            } else {
                                Vec::new()
                            },
                            fields: Vec::new(),
                        });
                    }
                }
            }
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
    let generic_names = definition
        .generic_params
        .iter()
        .map(|param| param.name.as_str())
        .collect::<Vec<_>>();
    let generic_name_set = definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    infer_generic_struct_literal_type_from_fields_seeded(
        type_name,
        definition,
        fields,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        &generic_names,
        &generic_name_set,
        BTreeMap::new(),
    )
}

#[allow(clippy::too_many_arguments)]
fn infer_generic_struct_literal_type_from_fields_seeded(
    type_name: &str,
    definition: &NirStructDef,
    fields: &[(String, AstExpr)],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    generic_names: &[&str],
    generic_name_set: &BTreeSet<String>,
    mut substitutions: BTreeMap<String, NirTypeRef>,
) -> Result<NirTypeRef, String> {
    let mut pending = fields
        .iter()
        .map(|(name, value)| (name.as_str(), value))
        .collect::<Vec<_>>();
    while !pending.is_empty() {
        let mut progress = false;
        let mut next_pending = Vec::new();
        for (name, value) in pending {
            let field = definition
                .fields
                .iter()
                .find(|field| field.name == name)
                .ok_or_else(|| format!("struct `{}` has no field `{}`", type_name, name))?;
            let field_pattern =
                specialize_nir_type_pattern_with_known_substitutions(&field.ty, &substitutions);
            let inferred = infer_field_expr_type_for_generic_pattern(
                value,
                &field_pattern,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                generic_name_set,
            )?;
            let Some(inferred) = inferred else {
                next_pending.push((name, value));
                continue;
            };
            unify_generic_struct_field_type_pattern(
                &field.ty,
                &inferred,
                generic_names,
                &mut substitutions,
                type_name,
            )?;
            progress = true;
        }
        if !progress {
            return Err(format!(
                "cannot infer generic arguments for struct literal `{}` in the current frontend; add an explicit expected type",
                type_name
            ));
        }
        pending = next_pending;
    }
    for (name, value) in fields {
        let _ = (name, value);
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

#[allow(clippy::too_many_arguments)]
fn infer_field_expr_type_for_generic_pattern(
    value: &AstExpr,
    expected_pattern: &NirTypeRef,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    placeholder_names: &BTreeSet<String>,
) -> Result<Option<NirTypeRef>, String> {
    match value {
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } if type_args.is_empty() && expected_pattern.name == *type_name => {
            let Some(definition) = struct_table.get(type_name) else {
                return Ok(None);
            };
            let generic_names = definition
                .generic_params
                .iter()
                .map(|param| param.name.as_str())
                .collect::<Vec<_>>();
            let generic_name_set = definition
                .generic_params
                .iter()
                .map(|param| param.name.clone())
                .collect::<BTreeSet<_>>();
            let seed = seed_generic_substitutions_from_expected_pattern(
                definition,
                expected_pattern,
                placeholder_names,
            );
            return match infer_generic_struct_literal_type_from_fields_seeded(
                type_name,
                definition,
                fields,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                &generic_names,
                &generic_name_set,
                seed,
            ) {
                Ok(inferred) => Ok(Some(inferred)),
                Err(error)
                    if error.contains("cannot infer generic arguments for struct literal") =>
                {
                    Ok(None)
                }
                Err(error) => Err(error),
            };
        }
        AstExpr::Call {
            callee,
            generic_args,
            args,
        } if generic_args.is_empty() && expected_pattern.name == *callee => {
            let Some(definition) = struct_table.get(callee) else {
                return Ok(None);
            };
            if definition.fields.len() != 1 || args.len() != 1 {
                return Ok(None);
            }
            return match infer_generic_payload_constructor_type_from_arg_seeded(
                callee,
                definition,
                &definition.fields[0].ty,
                &args[0],
                expected_pattern,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
                placeholder_names,
            ) {
                Ok(inferred) => Ok(Some(inferred)),
                Err(error)
                    if error
                        .contains("cannot infer generic arguments for payload-style struct constructor") =>
                {
                    Ok(None)
                }
                Err(error) => Err(error),
            };
        }
        _ => {}
    }
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
    Ok(infer_nir_expr_type(
        &lowered,
        bindings,
        signatures,
        struct_table,
    ))
}

#[allow(clippy::too_many_arguments)]
fn infer_generic_payload_constructor_type_from_arg_seeded(
    callee: &str,
    definition: &NirStructDef,
    field_ty: &NirTypeRef,
    arg: &AstExpr,
    expected_pattern: &NirTypeRef,
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    placeholder_names: &BTreeSet<String>,
) -> Result<NirTypeRef, String> {
    let generic_names = definition
        .generic_params
        .iter()
        .map(|param| param.name.as_str())
        .collect::<Vec<_>>();
    let mut substitutions =
        seed_generic_substitutions_from_expected_pattern(definition, expected_pattern, placeholder_names);
    let arg_pattern = specialize_nir_type_pattern_with_known_substitutions(field_ty, &substitutions);
    let Some(arg_ty) = infer_field_expr_type_for_generic_pattern(
        arg,
        &arg_pattern,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        &definition
            .generic_params
            .iter()
            .map(|param| param.name.clone())
            .collect::<BTreeSet<_>>(),
    )?
    else {
        return Err(format!(
            "cannot infer generic arguments for payload-style struct constructor `{callee}(...)`; add an explicit expected type"
        ));
    };
    unify_payload_constructor_type_pattern(
        field_ty,
        &arg_ty,
        &generic_names,
        &mut substitutions,
        callee,
    )?;
    let generic_args = definition
        .generic_params
        .iter()
        .map(|param| {
            substitutions.get(&param.name).cloned().ok_or_else(|| {
                format!(
                    "cannot infer generic arguments for payload-style struct constructor `{callee}(...)`; add an explicit expected type"
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(NirTypeRef {
        name: callee.to_owned(),
        generic_args,
        is_optional: false,
        is_ref: false,
    })
}

fn seed_generic_substitutions_from_expected_pattern(
    definition: &NirStructDef,
    expected_pattern: &NirTypeRef,
    placeholder_names: &BTreeSet<String>,
) -> BTreeMap<String, NirTypeRef> {
    if expected_pattern.name != definition.name
        || expected_pattern.generic_args.len() != definition.generic_params.len()
    {
        return BTreeMap::new();
    }
    definition
        .generic_params
        .iter()
        .zip(&expected_pattern.generic_args)
        .filter_map(|(param, arg)| {
            (!contains_placeholder_generic_name(arg, placeholder_names))
                .then_some((param.name.clone(), arg.clone()))
        })
        .collect()
}

fn specialize_nir_type_pattern_with_known_substitutions(
    pattern: &NirTypeRef,
    substitutions: &BTreeMap<String, NirTypeRef>,
) -> NirTypeRef {
    if pattern.generic_args.is_empty()
        && !pattern.is_optional
        && !pattern.is_ref
        && substitutions.contains_key(&pattern.name)
    {
        return substitutions.get(&pattern.name).cloned().unwrap_or_else(|| pattern.clone());
    }
    NirTypeRef {
        name: pattern.name.clone(),
        generic_args: pattern
            .generic_args
            .iter()
            .map(|arg| specialize_nir_type_pattern_with_known_substitutions(arg, substitutions))
            .collect(),
        is_optional: pattern.is_optional,
        is_ref: pattern.is_ref,
    }
}

fn contains_placeholder_generic_name(ty: &NirTypeRef, placeholder_names: &BTreeSet<String>) -> bool {
    (ty.generic_args.is_empty()
        && !ty.is_optional
        && !ty.is_ref
        && placeholder_names.contains(&ty.name))
        || ty
            .generic_args
            .iter()
            .any(|arg| contains_placeholder_generic_name(arg, placeholder_names))
}

fn unify_payload_constructor_type_pattern(
    pattern: &NirTypeRef,
    concrete: &NirTypeRef,
    generic_names: &[&str],
    substitutions: &mut BTreeMap<String, NirTypeRef>,
    callee: &str,
) -> Result<(), String> {
    if pattern.generic_args.is_empty()
        && !pattern.is_optional
        && !pattern.is_ref
        && generic_names.contains(&pattern.name.as_str())
    {
        if let Some(existing) = substitutions.get(&pattern.name) {
            if existing.render() != concrete.render() {
                return Err(format!(
                    "payload-style struct constructor `{callee}(...)` inferred conflicting types `{}` and `{}` for generic parameter `{}`",
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
            "cannot infer generic arguments for payload-style struct constructor `{callee}(...)`; add an explicit expected type"
        ));
    }
    for (pattern_arg, concrete_arg) in pattern.generic_args.iter().zip(&concrete.generic_args) {
        unify_payload_constructor_type_pattern(
            pattern_arg,
            concrete_arg,
            generic_names,
            substitutions,
            callee,
        )?;
    }
    Ok(())
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
