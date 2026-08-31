use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    NirEnumDef, NirEnumVariantKind, NirFunction, NirModule, NirStructDef, NirTypeRef,
};

use super::LoweringState;

pub(super) const OWNED_VARIANT_UNION_PREFIX: &str = yir_core::OWNED_VARIANT_UNION_LAYOUT_PREFIX;

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
        if let Some(definition) = structs.get(ty.name.as_str()).copied() {
            let mut visiting = BTreeSet::new();
            return encode_definition(definition, structs, enums, &mut visiting);
        }
    }
    let mut visiting = BTreeSet::new();
    encode_variant_union(ty, structs, enums, &mut visiting)
}

fn encode_variant_union(
    ty: &NirTypeRef,
    structs: &BTreeMap<&str, &NirStructDef>,
    enums: &BTreeMap<&str, &NirEnumDef>,
    visiting: &mut BTreeSet<String>,
) -> Option<String> {
    let definition = enums.get(ty.name.as_str()).copied()?;
    if definition.generic_params.len() != ty.generic_args.len() || definition.variants.is_empty() {
        return None;
    }
    let visiting_key = format!("enum:{}", definition.name);
    if !visiting.insert(visiting_key.clone()) {
        return None;
    }
    let substitutions = definition
        .generic_params
        .iter()
        .zip(&ty.generic_args)
        .map(|(parameter, argument)| (parameter.name.as_str(), argument))
        .collect::<BTreeMap<_, _>>();
    let encoded = (|| {
        let variants = definition
            .variants
            .iter()
            .map(|variant| {
                let fields = match &variant.kind {
                    NirEnumVariantKind::Unit => Vec::new(),
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
                                    enums,
                                    visiting,
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
                                    enums,
                                    visiting,
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
    })();
    visiting.remove(&visiting_key);
    encoded
}

fn encode_substituted_type(
    ty: &NirTypeRef,
    substitutions: &BTreeMap<&str, &NirTypeRef>,
    structs: &BTreeMap<&str, &NirStructDef>,
    enums: &BTreeMap<&str, &NirEnumDef>,
    visiting: &mut BTreeSet<String>,
) -> Option<String> {
    let resolved = substitutions.get(ty.name.as_str()).copied().unwrap_or(ty);
    if resolved.is_ref || resolved.is_optional {
        return None;
    }
    if resolved.generic_args.is_empty() && is_scheduler_scalar(&resolved.name) {
        return Some(resolved.name.clone());
    }
    if resolved.generic_args.is_empty() {
        if let Some(nested) = structs.get(resolved.name.as_str()).copied() {
            return encode_definition(nested, structs, enums, visiting);
        }
    }
    encode_variant_union(resolved, structs, enums, visiting)
}

fn encode_definition(
    definition: &NirStructDef,
    structs: &BTreeMap<&str, &NirStructDef>,
    enums: &BTreeMap<&str, &NirEnumDef>,
    visiting: &mut BTreeSet<String>,
) -> Option<String> {
    let visiting_key = format!("struct:{}", definition.name);
    if definition.fields.is_empty()
        || !definition.generic_params.is_empty()
        || !visiting.insert(visiting_key.clone())
    {
        return None;
    }
    let substitutions = BTreeMap::new();
    let fields = definition
        .fields
        .iter()
        .map(|field| {
            let encoded_type =
                encode_substituted_type(&field.ty, &substitutions, structs, enums, visiting)?;
            Some(format!("{}:{encoded_type}", field.name))
        })
        .collect::<Option<Vec<_>>>();
    visiting.remove(&visiting_key);
    Some(format!("{}{{{}}}", definition.name, fields?.join(";")))
}

fn is_scheduler_scalar(name: &str) -> bool {
    matches!(
        name,
        "bool" | "i32" | "i64" | "f32" | "f64" | "String" | "Bytes"
    )
}
