use std::collections::BTreeMap;

use nuis_semantics::model::{AstBinaryOp, AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::metadata::hidden_private_field_count;
use super::{
    infer_nir_expr_type, lower_binary_expr_with_async, lower_direct_call_builtin_or_named_call,
    lower_expr_with_async, lower_routed_call_or_core_builtin, resolve_declared_or_inferred,
    FunctionSignature, ModuleConstValue,
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
    generic_args: &[nuis_semantics::model::AstTypeRef],
    args: &[AstExpr],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
) -> Result<NirExpr, String> {
    lower_call_expr_with_async(
        callee,
        generic_args,
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
    generic_args: &[nuis_semantics::model::AstTypeRef],
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
    if let Some(payload_struct_constructor) = lower_payload_struct_constructor_sugar(
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
    )? {
        return Ok(payload_struct_constructor);
    }
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

#[allow(clippy::too_many_arguments)]
fn lower_payload_struct_constructor_sugar(
    callee: &str,
    generic_args: &[nuis_semantics::model::AstTypeRef],
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
    allow_async_calls: bool,
) -> Result<Option<NirExpr>, String> {
    if signatures.contains_key(callee) {
        return Ok(None);
    }
    let Some(definition) = struct_table.get(callee) else {
        return Ok(None);
    };
    if definition.fields.len() != 1 {
        return Err(format!(
            "payload-style struct constructor `{callee}(...)` requires struct `{callee}` to have exactly one field"
        ));
    }
    let hidden_private_fields = hidden_private_field_count(definition);
    if hidden_private_fields > 0 {
        return Err(format!(
            "struct literal `{}` cannot be constructed outside its defining module because it hides {} private field(s)",
            callee, hidden_private_fields
        ));
    }
    if args.len() != 1 {
        return Err(format!(
            "payload-style struct constructor `{callee}(...)` expects exactly 1 arg"
        ));
    }
    let field = &definition.fields[0];
    let constructor_ty = if definition.generic_params.is_empty() {
        if !generic_args.is_empty() {
            return Err(format!(
                "payload-style struct constructor `{callee}(...)` does not accept explicit generic arguments because struct `{callee}` is not generic"
            ));
        }
        NirTypeRef {
            name: callee.to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }
    } else if !generic_args.is_empty() {
        if generic_args.len() != definition.generic_params.len() {
            return Err(format!(
                "payload-style struct constructor `{callee}<...>(...)` expects {} generic argument(s), found {}",
                definition.generic_params.len(),
                generic_args.len()
            ));
        }
        NirTypeRef {
            name: callee.to_owned(),
            generic_args: generic_args.iter().map(super::lower_type_ref).collect(),
            is_optional: false,
            is_ref: false,
        }
    } else if let Some(expected) = expected {
        if expected.name != callee {
            return Err(format!(
                "payload-style struct constructor `{callee}(...)` requires expected type `{callee}<...>`, found `{}`",
                expected.render()
            ));
        }
        if expected.generic_args.len() != definition.generic_params.len()
            || expected.is_optional
            || expected.is_ref
        {
            return Err(format!(
                "payload-style struct constructor `{callee}(...)` requires expected type `{callee}<...>`, found `{}`",
                expected.render()
            ));
        }
        expected.clone()
    } else {
        let lowered_arg = lower_expr_with_async(
            &args[0],
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            None,
            allow_async_calls,
        )?;
        let Some(inferred_arg_ty) =
            infer_nir_expr_type(&lowered_arg, bindings, signatures, struct_table)
        else {
            return Err(format!(
                "cannot infer generic arguments for payload-style struct constructor `{callee}(...)`; add an explicit expected type"
            ));
        };
        infer_payload_constructor_type_from_arg(callee, definition, &field.ty, &inferred_arg_ty)?
    };
    let field_ty = if definition.generic_params.is_empty() {
        field.ty.clone()
    } else {
        super::instantiate_struct_field_type(&constructor_ty, definition, &field.ty)
    };
    let lowered = lower_expr_with_async(
        &args[0],
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        Some(&field_ty),
        allow_async_calls,
    )?;
    let inferred = infer_nir_expr_type(&lowered, bindings, signatures, struct_table);
    let _ = resolve_declared_or_inferred(&field.name, Some(field_ty), inferred)?;
    Ok(Some(NirExpr::StructLiteral {
        type_name: callee.to_owned(),
        type_args: constructor_ty.generic_args,
        fields: vec![(field.name.clone(), lowered)],
    }))
}

fn infer_payload_constructor_type_from_arg(
    callee: &str,
    definition: &NirStructDef,
    field_ty: &NirTypeRef,
    arg_ty: &NirTypeRef,
) -> Result<NirTypeRef, String> {
    let mut substitutions = BTreeMap::<String, NirTypeRef>::new();
    unify_payload_constructor_type_pattern(
        field_ty,
        arg_ty,
        &definition
            .generic_params
            .iter()
            .map(|param| param.name.as_str())
            .collect::<Vec<_>>(),
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
