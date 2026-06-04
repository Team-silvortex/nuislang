use std::collections::BTreeMap;

use nuis_semantics::model::{
    AstImplDef, AstModule, AstStructDef, AstTypeAlias, NirExternFunction, NirExternInterface,
    NirGenericParam, NirStructDef, NirStructField, NirTypeAlias,
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
        .collect::<Result<Vec<_>, String>>()
}

pub(super) fn build_module_struct_table(module: &AstModule) -> BTreeMap<String, AstStructDef> {
    module
        .structs
        .iter()
        .map(|definition| (definition.name.clone(), definition.clone()))
        .collect::<BTreeMap<_, _>>()
}

pub(super) fn build_impl_lookup(
    module: &AstModule,
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<BTreeMap<(String, String), AstImplDef>, String> {
    module
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
        .collect::<Result<BTreeMap<_, _>, String>>()
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
                            bound: param
                                .bound
                                .as_ref()
                                .map(|ty| lower_type_ref_with_aliases(ty, visible_type_aliases))
                                .transpose()?,
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
