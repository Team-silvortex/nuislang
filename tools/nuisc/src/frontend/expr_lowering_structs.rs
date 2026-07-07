use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::AstTypeRef;

use crate::frontend::metadata::ModuleConstValue;
use crate::frontend::name_suggestions::suggest_similar_name;
use crate::frontend::{
    infer_nir_expr_type, lower_nested_expr_with_async_and_consts, AstExpr, FunctionSignature,
    NestedExprWithConstsInput, NirStructDef, NirTypeRef,
};

pub(super) fn suggest_struct_field_name(
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

#[derive(Clone, Copy)]
pub(super) struct StructLiteralInferenceContext<'a> {
    pub(super) current_domain: &'a str,
    pub(super) current_function_is_async: bool,
    pub(super) bindings: &'a BTreeMap<String, NirTypeRef>,
    pub(super) module_consts: &'a BTreeMap<String, ModuleConstValue>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) struct_table: &'a BTreeMap<String, NirStructDef>,
}

pub(super) struct GenericStructLiteralInferenceInput<'a> {
    pub(super) type_name: &'a str,
    pub(super) definition: &'a NirStructDef,
    pub(super) fields: &'a [(String, AstExpr)],
    pub(super) context: StructLiteralInferenceContext<'a>,
}

struct SeededStructLiteralInferenceInput<'a> {
    type_name: &'a str,
    definition: &'a NirStructDef,
    fields: &'a [(String, AstExpr)],
    context: StructLiteralInferenceContext<'a>,
    generic_names: &'a [&'a str],
    generic_name_set: &'a BTreeSet<String>,
    substitutions: BTreeMap<String, NirTypeRef>,
}

struct FieldPatternInferenceInput<'a> {
    value: &'a AstExpr,
    expected_pattern: &'a NirTypeRef,
    context: StructLiteralInferenceContext<'a>,
    placeholder_names: &'a BTreeSet<String>,
}

struct PayloadConstructorInferenceInput<'a> {
    callee: &'a str,
    definition: &'a NirStructDef,
    field_ty: &'a NirTypeRef,
    arg: &'a AstExpr,
    expected_pattern: &'a NirTypeRef,
    context: StructLiteralInferenceContext<'a>,
    placeholder_names: &'a BTreeSet<String>,
}

pub(super) fn infer_generic_struct_literal_type_from_fields(
    input: GenericStructLiteralInferenceInput<'_>,
) -> Result<NirTypeRef, String> {
    let GenericStructLiteralInferenceInput {
        type_name,
        definition,
        fields,
        context,
    } = input;
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
    infer_generic_struct_literal_type_from_fields_seeded(SeededStructLiteralInferenceInput {
        type_name,
        definition,
        fields,
        context,
        generic_names: &generic_names,
        generic_name_set: &generic_name_set,
        substitutions: BTreeMap::new(),
    })
}

fn infer_generic_struct_literal_type_from_fields_seeded(
    input: SeededStructLiteralInferenceInput<'_>,
) -> Result<NirTypeRef, String> {
    let SeededStructLiteralInferenceInput {
        type_name,
        definition,
        fields,
        context,
        generic_names,
        generic_name_set,
        mut substitutions,
    } = input;
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
            let inferred = infer_field_expr_type_for_generic_pattern(FieldPatternInferenceInput {
                value,
                expected_pattern: &field_pattern,
                context,
                placeholder_names: generic_name_set,
            })?;
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

fn infer_field_expr_type_for_generic_pattern(
    input: FieldPatternInferenceInput<'_>,
) -> Result<Option<NirTypeRef>, String> {
    let FieldPatternInferenceInput {
        value,
        expected_pattern,
        context,
        placeholder_names,
    } = input;
    let StructLiteralInferenceContext {
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    } = context;
    match value {
        AstExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } if ast_type_args_are_placeholder_generics(type_args, placeholder_names)
            && expected_pattern.name == *type_name =>
        {
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
                SeededStructLiteralInferenceInput {
                    type_name,
                    definition,
                    fields,
                    context,
                    generic_names: &generic_names,
                    generic_name_set: &generic_name_set,
                    substitutions: seed,
                },
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
        } if ast_type_args_are_placeholder_generics(generic_args, placeholder_names)
            && expected_pattern.name == *callee =>
        {
            let Some(definition) = struct_table.get(callee) else {
                return Ok(None);
            };
            if definition.fields.len() != 1 || args.len() != 1 {
                return Ok(None);
            }
            return match infer_generic_payload_constructor_type_from_arg_seeded(
                PayloadConstructorInferenceInput {
                    callee,
                    definition,
                    field_ty: &definition.fields[0].ty,
                    arg: &args[0],
                    expected_pattern,
                    context,
                    placeholder_names,
                },
            ) {
                Ok(inferred) => Ok(Some(inferred)),
                Err(error)
                    if error.contains(
                        "cannot infer generic arguments for payload-style struct constructor",
                    ) =>
                {
                    Ok(None)
                }
                Err(error) => Err(error),
            };
        }
        _ => {}
    }
    let lowered = lower_nested_expr_with_async_and_consts(NestedExprWithConstsInput {
        expr: value,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        expected: None,
    })?;
    Ok(infer_nir_expr_type(
        &lowered,
        bindings,
        signatures,
        struct_table,
    ))
}

fn infer_generic_payload_constructor_type_from_arg_seeded(
    input: PayloadConstructorInferenceInput<'_>,
) -> Result<NirTypeRef, String> {
    let PayloadConstructorInferenceInput {
        callee,
        definition,
        field_ty,
        arg,
        expected_pattern,
        context,
        placeholder_names,
    } = input;
    let generic_names = definition
        .generic_params
        .iter()
        .map(|param| param.name.as_str())
        .collect::<Vec<_>>();
    let mut substitutions = seed_generic_substitutions_from_expected_pattern(
        definition,
        expected_pattern,
        placeholder_names,
    );
    let arg_pattern =
        specialize_nir_type_pattern_with_known_substitutions(field_ty, &substitutions);
    let Some(arg_ty) = infer_field_expr_type_for_generic_pattern(FieldPatternInferenceInput {
        value: arg,
        expected_pattern: &arg_pattern,
        context,
        placeholder_names: &definition
            .generic_params
            .iter()
            .map(|param| param.name.clone())
            .collect::<BTreeSet<_>>(),
    })?
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
        return substitutions
            .get(&pattern.name)
            .cloned()
            .unwrap_or_else(|| pattern.clone());
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

fn contains_placeholder_generic_name(
    ty: &NirTypeRef,
    placeholder_names: &BTreeSet<String>,
) -> bool {
    (ty.generic_args.is_empty()
        && !ty.is_optional
        && !ty.is_ref
        && placeholder_names.contains(&ty.name))
        || ty
            .generic_args
            .iter()
            .any(|arg| contains_placeholder_generic_name(arg, placeholder_names))
}

pub(super) fn ast_type_args_are_placeholder_generics(
    type_args: &[AstTypeRef],
    placeholder_names: &BTreeSet<String>,
) -> bool {
    type_args.is_empty()
        || type_args
            .iter()
            .all(|arg| ast_type_is_placeholder_generic(arg, placeholder_names))
}

fn ast_type_is_placeholder_generic(ty: &AstTypeRef, placeholder_names: &BTreeSet<String>) -> bool {
    (ty.generic_args.is_empty()
        && !ty.is_optional
        && !ty.is_ref
        && placeholder_names.contains(&ty.name))
        || ty
            .generic_args
            .iter()
            .all(|arg| ast_type_is_placeholder_generic(arg, placeholder_names))
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
