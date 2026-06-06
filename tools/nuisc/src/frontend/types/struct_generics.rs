use std::collections::BTreeMap;

use nuis_semantics::model::{NirStructDef, NirTypeRef};

pub(crate) fn instantiate_struct_field_type(
    base_ty: &NirTypeRef,
    definition: &NirStructDef,
    field_ty: &NirTypeRef,
) -> NirTypeRef {
    if definition.generic_params.len() != base_ty.generic_args.len() {
        return field_ty.clone();
    }
    substitute_struct_generic_type(
        field_ty,
        &definition
            .generic_params
            .iter()
            .map(|param| param.name.clone())
            .zip(base_ty.generic_args.iter().cloned())
            .collect::<BTreeMap<_, _>>(),
    )
}

fn substitute_struct_generic_type(
    ty: &NirTypeRef,
    substitutions: &BTreeMap<String, NirTypeRef>,
) -> NirTypeRef {
    if ty.generic_args.is_empty()
        && !ty.is_optional
        && !ty.is_ref
        && substitutions.contains_key(&ty.name)
    {
        return substitutions[&ty.name].clone();
    }
    NirTypeRef {
        name: ty.name.clone(),
        generic_args: ty
            .generic_args
            .iter()
            .map(|arg| substitute_struct_generic_type(arg, substitutions))
            .collect(),
        is_optional: ty.is_optional,
        is_ref: ty.is_ref,
    }
}
