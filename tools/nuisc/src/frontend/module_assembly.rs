use std::collections::BTreeMap;

use nuis_semantics::model::{
    AstEnumDef, AstEnumVariantKind, AstImplDef, AstModule, AstStructDef, AstStructField,
    AstTypeAlias, AstVisibility, NirEnumDef, NirEnumVariant, NirEnumVariantKind, NirExternFunction,
    NirExternInterface, NirGenericParam, NirStructDef, NirStructField, NirTypeAlias,
    NirWherePredicate,
};

use super::{
    helper_visible_struct_annotations, is_public_visibility, lower_ast_attributes,
    lower_param_with_aliases, lower_type_ref_with_aliases, lower_visibility,
};

pub(super) fn build_visible_struct_defs(
    module: &AstModule,
    local_cpu_helpers: &[&AstModule],
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<Vec<NirStructDef>, String> {
    module
        .structs
        .iter()
        .map(|definition| {
            Ok(NirStructDef {
                annotations: lower_ast_attributes(&definition.attributes),
                visibility: lower_visibility(definition.visibility),
                name: definition.name.clone(),
                generic_params: definition
                    .generic_params
                    .iter()
                    .map(|param| {
                        Ok(NirGenericParam {
                            name: param.name.clone(),
                            bounds: param
                                .bounds
                                .iter()
                                .map(|ty| lower_type_ref_with_aliases(ty, visible_type_aliases))
                                .collect::<Result<Vec<_>, _>>()?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
                where_bounds: definition
                    .where_bounds
                    .iter()
                    .map(|predicate| {
                        Ok(NirWherePredicate {
                            param_name: predicate.param_name.clone(),
                            bounds: predicate
                                .bounds
                                .iter()
                                .map(|ty| lower_type_ref_with_aliases(ty, visible_type_aliases))
                                .collect::<Result<Vec<_>, _>>()?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
                fields: definition
                    .fields
                    .iter()
                    .map(|field| {
                        Ok(NirStructField {
                            annotations: lower_ast_attributes(&field.attributes),
                            visibility: lower_visibility(field.visibility),
                            name: field.name.clone(),
                            ty: lower_type_ref_with_aliases(&field.ty, visible_type_aliases)?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            })
        })
        .chain(local_cpu_helpers.iter().flat_map(|helper| {
            helper
                .structs
                .iter()
                .filter(|definition| is_public_visibility(definition.visibility))
                .map(|definition| {
                    Ok(NirStructDef {
                        annotations: helper_visible_struct_annotations(definition),
                        visibility: lower_visibility(definition.visibility),
                        name: definition.name.clone(),
                        generic_params: definition
                            .generic_params
                            .iter()
                            .map(|param| {
                                Ok(NirGenericParam {
                                    name: param.name.clone(),
                                    bounds: param
                                        .bounds
                                        .iter()
                                        .map(|ty| {
                                            lower_type_ref_with_aliases(ty, visible_type_aliases)
                                        })
                                        .collect::<Result<Vec<_>, _>>()?,
                                })
                            })
                            .collect::<Result<Vec<_>, String>>()?,
                        where_bounds: definition
                            .where_bounds
                            .iter()
                            .map(|predicate| {
                                Ok(NirWherePredicate {
                                    param_name: predicate.param_name.clone(),
                                    bounds: predicate
                                        .bounds
                                        .iter()
                                        .map(|ty| {
                                            lower_type_ref_with_aliases(ty, visible_type_aliases)
                                        })
                                        .collect::<Result<Vec<_>, _>>()?,
                                })
                            })
                            .collect::<Result<Vec<_>, String>>()?,
                        fields: definition
                            .fields
                            .iter()
                            .filter(|field| is_public_visibility(field.visibility))
                            .map(|field| {
                                Ok(NirStructField {
                                    annotations: lower_ast_attributes(&field.attributes),
                                    visibility: lower_visibility(field.visibility),
                                    name: field.name.clone(),
                                    ty: lower_type_ref_with_aliases(
                                        &field.ty,
                                        visible_type_aliases,
                                    )?,
                                })
                            })
                            .collect::<Result<Vec<_>, String>>()?,
                    })
                })
        }))
        .chain(module.enums.iter().flat_map(|definition| {
            synthesize_enum_variant_structs(definition)
                .into_iter()
                .map(|definition| lower_ast_struct_def(&definition, visible_type_aliases, false))
        }))
        .chain(local_cpu_helpers.iter().flat_map(|helper| {
            helper
                .enums
                .iter()
                .filter(|definition| is_public_visibility(definition.visibility))
                .flat_map(|definition| synthesize_enum_variant_structs(definition).into_iter())
                .map(|definition| lower_ast_struct_def(&definition, visible_type_aliases, true))
        }))
        .collect::<Result<Vec<_>, String>>()
}

pub(super) fn build_module_struct_table(module: &AstModule) -> BTreeMap<String, AstStructDef> {
    module
        .structs
        .iter()
        .cloned()
        .chain(
            module
                .enums
                .iter()
                .flat_map(synthesize_enum_variant_structs),
        )
        .map(|definition| (definition.name.clone(), definition))
        .collect::<BTreeMap<_, _>>()
}

pub(super) fn build_visible_enum_defs(
    module: &AstModule,
    local_cpu_helpers: &[&AstModule],
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<Vec<NirEnumDef>, String> {
    module
        .enums
        .iter()
        .map(|definition| lower_enum_def(definition, visible_type_aliases, false))
        .chain(local_cpu_helpers.iter().flat_map(|helper| {
            helper
                .enums
                .iter()
                .filter(|definition| is_public_visibility(definition.visibility))
                .map(|definition| lower_enum_def(definition, visible_type_aliases, true))
        }))
        .collect::<Result<Vec<_>, String>>()
}

pub(super) fn build_impl_lookup(
    module: &AstModule,
    local_cpu_helpers: &[&AstModule],
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<BTreeMap<(String, String), AstImplDef>, String> {
    let mut lookup = module
        .impls
        .iter()
        .map(|definition| {
            Ok((
                (
                    definition.trait_name.clone(),
                    lower_type_ref_with_aliases(&definition.for_type, visible_type_aliases)?
                        .render(),
                ),
                definition.clone(),
            ))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    for helper in local_cpu_helpers {
        for definition in &helper.impls {
            let rendered_for_type =
                lower_type_ref_with_aliases(&definition.for_type, visible_type_aliases)?.render();
            lookup.insert(
                (definition.trait_name.clone(), rendered_for_type.clone()),
                definition.clone(),
            );
            lookup.insert(
                (
                    format!("{}.{}", helper.unit, definition.trait_name),
                    rendered_for_type,
                ),
                definition.clone(),
            );
        }
    }
    Ok(lookup)
}

pub(super) fn lower_type_alias_items(
    module: &AstModule,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<Vec<NirTypeAlias>, String> {
    module
        .type_aliases
        .iter()
        .map(|alias| {
            Ok(NirTypeAlias {
                visibility: lower_visibility(alias.visibility),
                name: alias.name.clone(),
                generic_params: alias
                    .generic_params
                    .iter()
                    .map(|param| {
                        Ok(NirGenericParam {
                            name: param.name.clone(),
                            bounds: param
                                .bounds
                                .iter()
                                .map(|ty| lower_type_ref_with_aliases(ty, visible_type_aliases))
                                .collect::<Result<Vec<_>, _>>()?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
                where_bounds: alias
                    .where_bounds
                    .iter()
                    .map(|predicate| {
                        Ok(NirWherePredicate {
                            param_name: predicate.param_name.clone(),
                            bounds: predicate
                                .bounds
                                .iter()
                                .map(|ty| lower_type_ref_with_aliases(ty, visible_type_aliases))
                                .collect::<Result<Vec<_>, _>>()?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
                target: lower_type_ref_with_aliases(&alias.target, visible_type_aliases)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()
}

pub(super) fn lower_extern_items(
    module: &AstModule,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<(Vec<NirExternFunction>, Vec<NirExternInterface>), String> {
    let externs = module
        .externs
        .iter()
        .map(|function| {
            Ok(NirExternFunction {
                visibility: lower_visibility(function.visibility),
                abi: function.abi.clone(),
                interface: None,
                name: function.name.clone(),
                host_symbol: function.host_symbol.clone(),
                params: function
                    .params
                    .iter()
                    .map(|param| lower_param_with_aliases(param, visible_type_aliases))
                    .collect::<Result<Vec<_>, _>>()?,
                return_type: lower_type_ref_with_aliases(
                    &function.return_type,
                    visible_type_aliases,
                )?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let extern_interfaces = module
        .extern_interfaces
        .iter()
        .map(|interface| {
            Ok(NirExternInterface {
                visibility: lower_visibility(interface.visibility),
                abi: interface.abi.clone(),
                name: interface.name.clone(),
                methods: interface
                    .methods
                    .iter()
                    .map(|function| {
                        Ok(NirExternFunction {
                            visibility: lower_visibility(function.visibility),
                            abi: function.abi.clone(),
                            interface: Some(interface.name.clone()),
                            name: function.name.clone(),
                            host_symbol: function.host_symbol.clone(),
                            params: function
                                .params
                                .iter()
                                .map(|param| lower_param_with_aliases(param, visible_type_aliases))
                                .collect::<Result<Vec<_>, _>>()?,
                            return_type: lower_type_ref_with_aliases(
                                &function.return_type,
                                visible_type_aliases,
                            )?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok((externs, extern_interfaces))
}

fn lower_enum_def(
    definition: &AstEnumDef,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    helper_visible: bool,
) -> Result<NirEnumDef, String> {
    Ok(NirEnumDef {
        annotations: if helper_visible {
            helper_visible_struct_annotations(&AstStructDef {
                visibility: definition.visibility,
                attributes: definition.attributes.clone(),
                name: definition.name.clone(),
                generic_params: definition.generic_params.clone(),
                where_bounds: definition.where_bounds.clone(),
                fields: Vec::new(),
            })
        } else {
            lower_ast_attributes(&definition.attributes)
        },
        visibility: lower_visibility(definition.visibility),
        name: definition.name.clone(),
        generic_params: definition
            .generic_params
            .iter()
            .map(|param| {
                Ok(NirGenericParam {
                    name: param.name.clone(),
                    bounds: param
                        .bounds
                        .iter()
                        .map(|ty| lower_type_ref_with_aliases(ty, visible_type_aliases))
                        .collect::<Result<Vec<_>, _>>()?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        where_bounds: definition
            .where_bounds
            .iter()
            .map(|predicate| {
                Ok(NirWherePredicate {
                    param_name: predicate.param_name.clone(),
                    bounds: predicate
                        .bounds
                        .iter()
                        .map(|ty| lower_type_ref_with_aliases(ty, visible_type_aliases))
                        .collect::<Result<Vec<_>, _>>()?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        variants: definition
            .variants
            .iter()
            .map(|variant| {
                Ok(NirEnumVariant {
                    name: variant.name.clone(),
                    kind: match &variant.kind {
                        AstEnumVariantKind::Unit => NirEnumVariantKind::Unit,
                        AstEnumVariantKind::Tuple(fields) => NirEnumVariantKind::Tuple(
                            fields
                                .iter()
                                .map(|ty| lower_type_ref_with_aliases(ty, visible_type_aliases))
                                .collect::<Result<Vec<_>, _>>()?,
                        ),
                        AstEnumVariantKind::Struct(fields) => NirEnumVariantKind::Struct(
                            fields
                                .iter()
                                .filter(|field| {
                                    !helper_visible || is_public_visibility(field.visibility)
                                })
                                .map(|field| {
                                    Ok(NirStructField {
                                        annotations: lower_ast_attributes(&field.attributes),
                                        visibility: lower_visibility(field.visibility),
                                        name: field.name.clone(),
                                        ty: lower_type_ref_with_aliases(
                                            &field.ty,
                                            visible_type_aliases,
                                        )?,
                                    })
                                })
                                .collect::<Result<Vec<_>, String>>()?,
                        ),
                    },
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
    })
}

fn lower_ast_struct_def(
    definition: &AstStructDef,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    helper_visible: bool,
) -> Result<NirStructDef, String> {
    Ok(NirStructDef {
        annotations: if helper_visible {
            helper_visible_struct_annotations(definition)
        } else {
            lower_ast_attributes(&definition.attributes)
        },
        visibility: lower_visibility(definition.visibility),
        name: definition.name.clone(),
        generic_params: definition
            .generic_params
            .iter()
            .map(|param| {
                Ok(NirGenericParam {
                    name: param.name.clone(),
                    bounds: param
                        .bounds
                        .iter()
                        .map(|ty| lower_type_ref_with_aliases(ty, visible_type_aliases))
                        .collect::<Result<Vec<_>, _>>()?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        where_bounds: definition
            .where_bounds
            .iter()
            .map(|predicate| {
                Ok(NirWherePredicate {
                    param_name: predicate.param_name.clone(),
                    bounds: predicate
                        .bounds
                        .iter()
                        .map(|ty| lower_type_ref_with_aliases(ty, visible_type_aliases))
                        .collect::<Result<Vec<_>, _>>()?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        fields: definition
            .fields
            .iter()
            .filter(|field| !helper_visible || is_public_visibility(field.visibility))
            .map(|field| {
                Ok(NirStructField {
                    annotations: lower_ast_attributes(&field.attributes),
                    visibility: lower_visibility(field.visibility),
                    name: field.name.clone(),
                    ty: lower_type_ref_with_aliases(&field.ty, visible_type_aliases)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
    })
}

fn synthesize_enum_variant_structs(definition: &AstEnumDef) -> Vec<AstStructDef> {
    definition
        .variants
        .iter()
        .map(|variant| AstStructDef {
            visibility: definition.visibility,
            attributes: Vec::new(),
            name: format!("{}.{}", definition.name, variant.name),
            generic_params: definition.generic_params.clone(),
            where_bounds: definition.where_bounds.clone(),
            fields: match &variant.kind {
                AstEnumVariantKind::Unit => Vec::new(),
                AstEnumVariantKind::Tuple(fields) => fields
                    .iter()
                    .enumerate()
                    .map(|(index, ty)| AstStructField {
                        visibility: AstVisibility::Public,
                        attributes: Vec::new(),
                        name: if fields.len() == 1 {
                            "value".to_owned()
                        } else {
                            format!("_{index}")
                        },
                        ty: ty.clone(),
                    })
                    .collect(),
                AstEnumVariantKind::Struct(fields) => fields.clone(),
            },
        })
        .collect()
}
