use std::collections::BTreeMap;

use nuis_semantics::model::{
    AstDestructureBinding, AstDestructureField, AstExpr, AstMatchPattern, AstModule, AstStructDef,
    AstTypeAlias, AstTypeRef,
};

use super::resolve_ast_type_ref_aliases;

pub(super) fn collect_visible_structs(
    module: &AstModule,
    local_cpu_helpers: &[&AstModule],
) -> BTreeMap<String, AstStructDef> {
    let mut structs = module
        .structs
        .iter()
        .map(|definition| (definition.name.clone(), definition.clone()))
        .collect::<BTreeMap<_, _>>();
    for helper in local_cpu_helpers {
        for definition in helper
            .structs
            .iter()
            .filter(|definition| super::is_public_visibility(definition.visibility))
        {
            structs.insert(definition.name.clone(), definition.clone());
        }
    }
    structs
}

pub(super) fn simple_match_value_type(
    value: &AstExpr,
    local_type_env: &BTreeMap<String, AstTypeRef>,
) -> Option<AstTypeRef> {
    match value {
        AstExpr::Var(name) => local_type_env.get(name).cloned(),
        _ => None,
    }
}

pub(super) fn bind_destructure_fields_for_type(
    type_ref: &AstTypeRef,
    fields: &[AstDestructureField],
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    visible_structs: &BTreeMap<String, AstStructDef>,
    env: &mut BTreeMap<String, AstTypeRef>,
) -> Result<(), String> {
    let resolved = resolve_ast_type_ref_aliases(type_ref, visible_type_aliases)?;
    let Some(struct_def) = visible_structs.get(&resolved.name) else {
        return Ok(());
    };
    for field in fields {
        let Some(struct_field) = struct_def
            .fields
            .iter()
            .find(|candidate| candidate.name == field.field)
        else {
            return Err(format!(
                "type `{}` has no field `{}` for destructuring let",
                resolved.name, field.field
            ));
        };
        let field_ty = instantiate_ast_struct_field_type(&resolved, struct_def, &struct_field.ty);
        match &field.binding {
            AstDestructureBinding::Bind(name) => {
                env.insert(name.clone(), field_ty.clone());
            }
            AstDestructureBinding::Ignore => {}
            AstDestructureBinding::Nested { type_ref, fields } => {
                let nested_type = type_ref.as_ref().unwrap_or(&field_ty);
                bind_destructure_fields_for_type(
                    nested_type,
                    fields,
                    visible_type_aliases,
                    visible_structs,
                    env,
                )?;
            }
        }
    }
    Ok(())
}

pub(super) fn bind_match_pattern_for_type(
    type_ref: &AstTypeRef,
    pattern: &AstMatchPattern,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    visible_structs: &BTreeMap<String, AstStructDef>,
    env: &mut BTreeMap<String, AstTypeRef>,
) -> Result<(), String> {
    match pattern {
        AstMatchPattern::Bind(name) => {
            env.insert(name.clone(), type_ref.clone());
        }
        AstMatchPattern::PayloadStruct { type_ref, payload } => {
            let resolved = resolve_ast_type_ref_aliases(type_ref, visible_type_aliases)?;
            let Some(struct_def) = visible_structs.get(&resolved.name) else {
                return Ok(());
            };
            let Some(payload_field) = struct_def.fields.first() else {
                return Ok(());
            };
            let payload_ty =
                instantiate_ast_struct_field_type(&resolved, struct_def, &payload_field.ty);
            bind_match_pattern_for_type(
                &payload_ty,
                payload,
                visible_type_aliases,
                visible_structs,
                env,
            )?;
        }
        AstMatchPattern::StructFields {
            type_ref: explicit_type,
            fields,
        } => {
            let match_type = explicit_type.as_ref().unwrap_or(type_ref);
            let resolved = resolve_ast_type_ref_aliases(match_type, visible_type_aliases)?;
            let Some(struct_def) = visible_structs.get(&resolved.name) else {
                return Ok(());
            };
            for (field_name, field_pattern) in fields {
                let Some(struct_field) = struct_def
                    .fields
                    .iter()
                    .find(|candidate| candidate.name == *field_name)
                else {
                    continue;
                };
                let field_ty =
                    instantiate_ast_struct_field_type(&resolved, struct_def, &struct_field.ty);
                bind_match_pattern_for_type(
                    &field_ty,
                    field_pattern,
                    visible_type_aliases,
                    visible_structs,
                    env,
                )?;
            }
        }
        AstMatchPattern::Or(patterns) => {
            for nested in patterns {
                bind_match_pattern_for_type(
                    type_ref,
                    nested,
                    visible_type_aliases,
                    visible_structs,
                    env,
                )?;
            }
        }
        AstMatchPattern::Wildcard
        | AstMatchPattern::Bool(_)
        | AstMatchPattern::Int(_)
        | AstMatchPattern::IntRangeInclusive(_, _) => {}
    }
    Ok(())
}

pub(super) fn instantiate_ast_struct_field_type(
    base_ty: &AstTypeRef,
    definition: &AstStructDef,
    field_ty: &AstTypeRef,
) -> AstTypeRef {
    if definition.generic_params.len() != base_ty.generic_args.len() {
        return field_ty.clone();
    }
    let substitutions = definition
        .generic_params
        .iter()
        .map(|param| param.name.clone())
        .zip(base_ty.generic_args.iter().cloned())
        .collect::<BTreeMap<_, _>>();
    substitute_ast_struct_generic_type(field_ty, &substitutions)
}

fn substitute_ast_struct_generic_type(
    ty: &AstTypeRef,
    substitutions: &BTreeMap<String, AstTypeRef>,
) -> AstTypeRef {
    if ty.generic_args.is_empty() && !ty.is_optional && !ty.is_ref {
        if let Some(substitution) = substitutions.get(&ty.name) {
            return substitution.clone();
        }
    }
    AstTypeRef {
        name: ty.name.clone(),
        generic_args: ty
            .generic_args
            .iter()
            .map(|arg| substitute_ast_struct_generic_type(arg, substitutions))
            .collect(),
        is_optional: ty.is_optional,
        is_ref: ty.is_ref,
    }
}
