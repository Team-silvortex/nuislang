use std::collections::{BTreeMap, BTreeSet};

use super::binary_lowering::{lower_binary_expr_with_async, BinaryLoweringInput};
use super::metadata::{hidden_private_field_count, ModuleConstValue};
use super::unary_lowering::{lower_unary_expr_with_async, UnaryLoweringInput};
use super::validation_helpers::render_type_name;
use super::{
    infer_nir_expr_type, instantiate_struct_field_type, lower_call_expr_with_async, named_type,
    resolve_declared_or_inferred, struct_field_type, AstExpr, CallLoweringInput, FunctionSignature,
    NirExpr, NirStructDef, NirTypeRef,
};

#[path = "expr_lowering_methods.rs"]
mod expr_lowering_methods;
#[path = "expr_lowering_structs.rs"]
mod expr_lowering_structs;

use expr_lowering_structs::{
    ast_type_args_are_placeholder_generics, infer_generic_struct_literal_type_from_fields,
    suggest_struct_field_name,
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
            } else if bindings.contains_key(name)
                || (expected.is_some_and(|ty| {
                !ty.is_optional
                    && !ty.is_ref
                    && matches!(
                        (ty.name.as_str(), ty.generic_args.len()),
                        ("Fn1", 2) | ("Fn2", 3) | ("Fn3", 4)
                    )
            }) && signatures.contains_key(name))
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
        } => lower_call_expr_with_async(CallLoweringInput {
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
        })?,
        AstExpr::MethodCall {
            receiver,
            method,
            generic_args,
            args,
        } => expr_lowering_methods::lower_method_call_with_async(
            receiver,
            method,
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
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => {
            let definition = struct_table
                .get(type_name)
                .ok_or_else(|| format!("unknown struct type `{}`", type_name))?;
            let generic_name_set = definition
                .generic_params
                .iter()
                .map(|param| param.name.clone())
                .collect::<BTreeSet<_>>();
            let has_placeholder_type_args =
                ast_type_args_are_placeholder_generics(type_args, &generic_name_set);
            let literal_ty = if definition.generic_params.is_empty() {
                if !type_args.is_empty() {
                    return Err(format!(
                        "struct literal `{}` does not accept explicit generic arguments because struct `{}` is not generic",
                        type_name, type_name
                    ));
                }
                named_type(type_name)
            } else if !type_args.is_empty() && !has_placeholder_type_args {
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
        AstExpr::Unary { op, operand } => lower_unary_expr_with_async(UnaryLoweringInput {
            op,
            operand,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            expected,
        })?,
        AstExpr::Binary { op, lhs, rhs } => lower_binary_expr_with_async(BinaryLoweringInput {
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
        })?,
    })
}
