use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{NirFunction, NirModule, NirStructDef, NirTypeRef};

use super::LoweringState;

pub(super) fn module_owned_struct_layout(module: &NirModule, ty: &NirTypeRef) -> Option<String> {
    let definitions = module
        .structs
        .iter()
        .map(|definition| (definition.name.as_str(), definition))
        .collect::<BTreeMap<_, _>>();
    owned_struct_layout(ty, &definitions)
}

pub(super) fn function_owned_struct_layout(
    function: &NirFunction,
    state: &LoweringState<'_>,
) -> Option<String> {
    owned_struct_layout(function.return_type.as_ref()?, &state.struct_defs)
}

fn owned_struct_layout(
    ty: &NirTypeRef,
    definitions: &BTreeMap<&str, &NirStructDef>,
) -> Option<String> {
    if !is_plain_type(ty) {
        return None;
    }
    let definition = definitions.get(ty.name.as_str()).copied()?;
    let mut visiting = BTreeSet::new();
    encode_definition(definition, definitions, &mut visiting)
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
