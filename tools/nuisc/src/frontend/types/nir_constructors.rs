use super::*;

pub(crate) fn named_type(name: &str) -> NirTypeRef {
    NirTypeRef {
        name: name.to_owned(),
        generic_args: Vec::new(),
        is_optional: false,
        is_ref: false,
    }
}

pub(crate) fn generic_named_type(name: &str, generic_args: Vec<NirTypeRef>) -> NirTypeRef {
    NirTypeRef {
        name: name.to_owned(),
        generic_args,
        is_optional: false,
        is_ref: false,
    }
}

pub(crate) fn ref_type(name: &str) -> NirTypeRef {
    NirTypeRef {
        name: name.to_owned(),
        generic_args: Vec::new(),
        is_optional: false,
        is_ref: true,
    }
}

pub(crate) fn i64_type() -> NirTypeRef {
    named_type("i64")
}
pub(crate) fn i32_type() -> NirTypeRef {
    named_type("i32")
}
pub(crate) fn f32_type() -> NirTypeRef {
    named_type("f32")
}
pub(crate) fn f64_type() -> NirTypeRef {
    named_type("f64")
}
pub(crate) fn bool_type() -> NirTypeRef {
    named_type("bool")
}
pub(crate) fn string_type() -> NirTypeRef {
    named_type("String")
}
pub(crate) fn unit_type() -> NirTypeRef {
    named_type("Unit")
}

pub(crate) fn struct_field_type(
    base_ty: &NirTypeRef,
    field: &str,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Option<NirTypeRef> {
    if let Some(builtin) = builtin_struct_field_type(&base_ty.name, field) {
        return Some(builtin);
    }
    let definition = struct_table.get(&base_ty.name)?;
    Some(instantiate_struct_field_type(
        base_ty,
        definition,
        &definition.field(field)?.ty,
    ))
}

pub(crate) fn instantiate_struct_field_type(
    base_ty: &NirTypeRef,
    definition: &NirStructDef,
    field_ty: &NirTypeRef,
) -> NirTypeRef {
    crate::frontend::types::struct_generics::instantiate_struct_field_type(
        base_ty, definition, field_ty,
    )
}
