use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstFunction, AstImplDef, AstModule, AstTypeAlias, AstTypeRef, NirFunction, NirStructDef,
};

use super::{
    build_function_return_type_table, build_impl_method_function, impl_method_lookup_key,
    impl_method_symbol_name, infer_missing_function_return_type, is_public_visibility,
    lower_function, lower_param_with_aliases, lower_stmt_with_async, lower_type_ref_with_aliases,
    lower_visibility, rewrite_generic_calls_in_function, FunctionSignature, ModuleConstValue,
};
use nuis_semantics::model::{
    NirImplDef, NirImplMethod, NirTraitDef, NirTraitMethodSig, NirTypeRef,
};

pub(super) fn build_lowered_functions_and_impls(
    module: &AstModule,
    local_cpu_helpers: &[&AstModule],
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
    module_const_values: &BTreeMap<String, ModuleConstValue>,
    module_const_env: &BTreeMap<String, AstTypeRef>,
    helper_const_maps: &BTreeMap<String, BTreeMap<String, ModuleConstValue>>,
    signatures: &mut BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    module_struct_table: &BTreeMap<String, nuis_semantics::model::AstStructDef>,
    impl_lookup: &BTreeMap<(String, String), AstImplDef>,
    generic_templates: &BTreeMap<String, AstFunction>,
    concrete_module_functions: &[AstFunction],
) -> Result<(Vec<NirFunction>, Vec<NirTraitDef>, Vec<NirImplDef>), String> {
    let function_return_types = build_function_return_type_table(
        module,
        concrete_module_functions,
        generic_templates,
        local_cpu_helpers,
        visible_type_aliases,
    );
    let mut inferred_function_return_types = function_return_types.clone();
    let mut specialized_functions = Vec::new();
    let mut specialized_signatures = Vec::new();
    let mut specialization_cache = BTreeSet::new();
    let mut rewritten_module_functions = concrete_module_functions
        .iter()
        .map(|function| {
            rewrite_generic_calls_in_function(
                function,
                module_const_env,
                visible_type_aliases,
                generic_templates,
                impl_lookup,
                module_struct_table,
                &inferred_function_return_types,
                &mut specialization_cache,
                &mut specialized_functions,
                &mut specialized_signatures,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut changed = true;
    while changed {
        changed = false;
        for function in &mut rewritten_module_functions {
            if function.return_type.is_none() {
                if let Some(inferred_return_type) = infer_missing_function_return_type(
                    function,
                    module_const_env,
                    impl_lookup,
                    module_struct_table,
                    &inferred_function_return_types,
                )? {
                    function.return_type = Some(inferred_return_type.clone());
                    inferred_function_return_types
                        .insert(function.name.clone(), Some(inferred_return_type));
                    changed = true;
                }
            }
            if let Some(return_type) = &function.return_type {
                if let Some(signature) = signatures.get_mut(&function.name) {
                    signature.return_type = Some(lower_type_ref_with_aliases(
                        return_type,
                        visible_type_aliases,
                    )?);
                }
            }
        }
    }
    for (name, signature) in specialized_signatures {
        signatures.insert(name, signature);
    }
    for definition in &module.impls {
        let lowered_for_type =
            lower_type_ref_with_aliases(&definition.for_type, visible_type_aliases)?;
        for method in &definition.methods {
            let symbol_name =
                impl_method_symbol_name(&definition.trait_name, &lowered_for_type, &method.name);
            signatures.insert(
                impl_method_lookup_key(&lowered_for_type, &method.name),
                FunctionSignature {
                    abi: "nuis".to_owned(),
                    interface: None,
                    symbol_name: symbol_name.clone(),
                    params: method
                        .params
                        .iter()
                        .map(|param| lower_type_ref_with_aliases(&param.ty, visible_type_aliases))
                        .collect::<Result<Vec<_>, _>>()?,
                    return_type: method
                        .return_type
                        .as_ref()
                        .map(|ty| lower_type_ref_with_aliases(ty, visible_type_aliases))
                        .transpose()?,
                    is_extern: false,
                    is_async: false,
                },
            );
            rewritten_module_functions.push(build_impl_method_function(
                definition,
                method,
                &symbol_name,
            ));
        }
    }

    let mut lowered_functions = rewritten_module_functions
        .iter()
        .map(|function| {
            lower_function(
                function,
                &module.domain,
                &module.unit,
                module_const_values,
                visible_type_aliases,
                signatures,
                struct_table,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    for function in &mut specialized_functions {
        if function.return_type.is_none() {
            if let Some(inferred_return_type) = infer_missing_function_return_type(
                function,
                module_const_env,
                impl_lookup,
                module_struct_table,
                &inferred_function_return_types,
            )? {
                function.return_type = Some(inferred_return_type.clone());
                inferred_function_return_types
                    .insert(function.name.clone(), Some(inferred_return_type));
            }
        }
        if let Some(return_type) = &function.return_type {
            if let Some(signature) = signatures.get_mut(&function.name) {
                signature.return_type = Some(lower_type_ref_with_aliases(
                    return_type,
                    visible_type_aliases,
                )?);
            }
        }
    }

    lowered_functions.extend(
        specialized_functions
            .iter()
            .map(|function| {
                lower_function(
                    function,
                    &module.domain,
                    &module.unit,
                    module_const_values,
                    visible_type_aliases,
                    signatures,
                    struct_table,
                )
            })
            .collect::<Result<Vec<_>, _>>()?,
    );

    for helper in local_cpu_helpers {
        for function in helper
            .functions
            .iter()
            .filter(|function| is_public_visibility(function.visibility))
        {
            let mut renamed = function.clone();
            renamed.name = format!("{}.{}", helper.unit, function.name);
            lowered_functions.push(lower_function(
                &renamed,
                &module.domain,
                &helper.unit,
                helper_const_maps.get(&helper.unit).unwrap(),
                visible_type_aliases,
                signatures,
                struct_table,
            )?);
        }
    }

    let lowered_traits = module
        .traits
        .iter()
        .map(|definition| {
            Ok(NirTraitDef {
                visibility: lower_visibility(definition.visibility),
                name: definition.name.clone(),
                methods: definition
                    .methods
                    .iter()
                    .map(|method| {
                        Ok(NirTraitMethodSig {
                            name: method.name.clone(),
                            params: method
                                .params
                                .iter()
                                .map(|param| lower_param_with_aliases(param, visible_type_aliases))
                                .collect::<Result<Vec<_>, _>>()?,
                            return_type: method
                                .return_type
                                .as_ref()
                                .map(|ty| lower_type_ref_with_aliases(ty, visible_type_aliases))
                                .transpose()?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let mut lowered_impls = Vec::new();
    for definition in &module.impls {
        let mut methods = Vec::new();
        for method in &definition.methods {
            let mut bindings = BTreeMap::<String, NirTypeRef>::new();
            for param in &method.params {
                bindings.insert(
                    param.name.clone(),
                    lower_type_ref_with_aliases(&param.ty, visible_type_aliases)?,
                );
            }
            let body = method
                .body
                .iter()
                .map(|stmt| {
                    lower_stmt_with_async(
                        stmt,
                        &module.domain,
                        false,
                        &mut bindings,
                        module_const_values,
                        method.return_type.as_ref(),
                        visible_type_aliases,
                        signatures,
                        struct_table,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            methods.push(NirImplMethod {
                name: method.name.clone(),
                params: method
                    .params
                    .iter()
                    .map(|param| lower_param_with_aliases(param, visible_type_aliases))
                    .collect::<Result<Vec<_>, _>>()?,
                return_type: method
                    .return_type
                    .as_ref()
                    .map(|ty| lower_type_ref_with_aliases(ty, visible_type_aliases))
                    .transpose()?,
                body,
            });
        }
        lowered_impls.push(NirImplDef {
            trait_name: definition.trait_name.clone(),
            for_type: lower_type_ref_with_aliases(&definition.for_type, visible_type_aliases)?,
            methods,
        });
    }

    Ok((lowered_functions, lowered_traits, lowered_impls))
}
