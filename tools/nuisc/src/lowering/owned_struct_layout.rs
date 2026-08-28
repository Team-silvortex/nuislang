use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    NirEnumDef, NirEnumVariantKind, NirFunction, NirModule, NirStructDef, NirTypeRef,
};

use super::LoweringState;

pub(super) const OWNED_VARIANT_UNION_PREFIX: &str = "__nuis_variant_union__";

pub(super) fn module_owned_struct_layout(module: &NirModule, ty: &NirTypeRef) -> Option<String> {
    let structs = module
        .structs
        .iter()
        .map(|definition| (definition.name.as_str(), definition))
        .collect::<BTreeMap<_, _>>();
    let enums = module
        .enums
        .iter()
        .map(|definition| (definition.name.as_str(), definition))
        .collect::<BTreeMap<_, _>>();
    owned_value_layout(ty, &structs, &enums)
}

pub(super) fn function_owned_struct_layout(
    function: &NirFunction,
    state: &LoweringState<'_>,
) -> Option<String> {
    owned_value_layout(
        function.return_type.as_ref()?,
        &state.struct_defs,
        &state.enum_defs,
    )
}

pub(super) fn owned_layout_is_variant_union(layout: &str) -> bool {
    layout.starts_with(OWNED_VARIANT_UNION_PREFIX)
}

fn owned_value_layout(
    ty: &NirTypeRef,
    structs: &BTreeMap<&str, &NirStructDef>,
    enums: &BTreeMap<&str, &NirEnumDef>,
) -> Option<String> {
    if ty.is_ref || ty.is_optional {
        return None;
    }
    if ty.generic_args.is_empty() {
        let definition = structs.get(ty.name.as_str()).copied()?;
        let mut visiting = BTreeSet::new();
        return encode_definition(definition, structs, &mut visiting);
    }
    encode_variant_union(ty, structs, enums)
}

fn encode_variant_union(
    ty: &NirTypeRef,
    structs: &BTreeMap<&str, &NirStructDef>,
    enums: &BTreeMap<&str, &NirEnumDef>,
) -> Option<String> {
    let definition = enums.get(ty.name.as_str()).copied()?;
    if definition.generic_params.len() != ty.generic_args.len() || definition.variants.is_empty() {
        return None;
    }
    let substitutions = definition
        .generic_params
        .iter()
        .zip(&ty.generic_args)
        .map(|(parameter, argument)| (parameter.name.as_str(), argument))
        .collect::<BTreeMap<_, _>>();
    let mut visiting = BTreeSet::new();
    let variants = definition
        .variants
        .iter()
        .map(|variant| {
            let fields = match &variant.kind {
                NirEnumVariantKind::Unit => return None,
                NirEnumVariantKind::Tuple(types) => types
                    .iter()
                    .enumerate()
                    .map(|(index, field_ty)| {
                        let name = if types.len() == 1 {
                            "value".to_owned()
                        } else {
                            format!("_{index}")
                        };
                        Some((
                            name,
                            encode_substituted_type(
                                field_ty,
                                &substitutions,
                                structs,
                                &mut visiting,
                            )?,
                        ))
                    })
                    .collect::<Option<Vec<_>>>()?,
                NirEnumVariantKind::Struct(fields) => fields
                    .iter()
                    .map(|field| {
                        Some((
                            field.name.clone(),
                            encode_substituted_type(
                                &field.ty,
                                &substitutions,
                                structs,
                                &mut visiting,
                            )?,
                        ))
                    })
                    .collect::<Option<Vec<_>>>()?,
            };
            let variant_name = format!("{}.{}", definition.name, variant.name);
            let encoded_fields = fields
                .into_iter()
                .map(|(name, field)| format!("{name}:{field}"))
                .collect::<Vec<_>>()
                .join(";");
            Some(format!("{variant_name}:{variant_name}{{{encoded_fields}}}"))
        })
        .collect::<Option<Vec<_>>>()?;
    Some(format!(
        "{OWNED_VARIANT_UNION_PREFIX}{}{{tag:i64;{}}}",
        definition.name,
        variants.join(";")
    ))
}

fn encode_substituted_type(
    ty: &NirTypeRef,
    substitutions: &BTreeMap<&str, &NirTypeRef>,
    definitions: &BTreeMap<&str, &NirStructDef>,
    visiting: &mut BTreeSet<String>,
) -> Option<String> {
    let resolved = substitutions.get(ty.name.as_str()).copied().unwrap_or(ty);
    if !is_plain_type(resolved) {
        return None;
    }
    if is_scheduler_scalar(&resolved.name) {
        return Some(resolved.name.clone());
    }
    let nested = definitions.get(resolved.name.as_str()).copied()?;
    encode_definition(nested, definitions, visiting)
}

fn encode_definition(
    definition: &NirStructDef,
    definitions: &BTreeMap<&str, &NirStructDef>,
    visiting: &mut BTreeSet<String>,
) -> Option<String> {
    if definition.fields.is_empty()
        || !definition.generic_params.is_empty()
        || !visiting.insert(definition.name.clone())
    {
        return None;
    }
    let fields = definition
        .fields
        .iter()
        .map(|field| {
            if !is_plain_type(&field.ty) {
                return None;
            }
            let encoded_type = if is_scheduler_scalar(&field.ty.name) {
                field.ty.name.clone()
            } else {
                let nested = definitions.get(field.ty.name.as_str()).copied()?;
                encode_definition(nested, definitions, visiting)?
            };
            Some(format!("{}:{encoded_type}", field.name))
        })
        .collect::<Option<Vec<_>>>();
    visiting.remove(&definition.name);
    Some(format!("{}{{{}}}", definition.name, fields?.join(";")))
}

fn is_plain_type(ty: &NirTypeRef) -> bool {
    !ty.is_ref && !ty.is_optional && ty.generic_args.is_empty()
}

fn is_scheduler_scalar(name: &str) -> bool {
    matches!(
        name,
        "bool" | "i32" | "i64" | "f32" | "f64" | "String" | "Bytes"
    )
}
