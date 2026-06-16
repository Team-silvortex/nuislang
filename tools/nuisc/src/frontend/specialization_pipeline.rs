use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstFunction, AstImplDef, AstModule, AstTypeAlias, AstTypeRef, NirFunction, NirStructDef,
};

use super::higher_order::{is_callable_type_with_aliases, rewrite_higher_order_calls_in_function};
use super::stmt_lowering::lower_stmt_sequence_with_async;
use super::{
    build_default_impl_method, build_default_impl_method_function, build_function_return_type_table,
    build_impl_method_function, impl_method_lookup_key, impl_method_symbol_name,
    infer_missing_function_return_type, is_public_visibility, lower_function,
    lower_param_with_aliases, lower_type_ref_with_aliases, lower_visibility,
    rewrite_generic_calls_in_function, FunctionSignature, GenericImplMethodTemplate,
    ModuleConstValue,
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
    let mut trait_defs = module
        .traits
        .iter()
        .map(|definition| (definition.name.clone(), definition))
        .collect::<BTreeMap<_, _>>();
    for helper in local_cpu_helpers {
        for definition in helper
            .traits
            .iter()
            .filter(|definition| is_public_visibility(definition.visibility))
        {
            trait_defs.insert(definition.name.clone(), definition);
            trait_defs.insert(format!("{}.{}", helper.unit, definition.name), definition);
        }
    }
    let generic_impl_method_templates = module
        .impls
        .iter()
        .filter(|definition| {
            !definition.generic_params.is_empty() || !definition.where_bounds.is_empty()
        })
        .flat_map(|definition| {
            let Some(trait_def) = trait_defs.get(&definition.trait_name) else {
                return Vec::new().into_iter();
            };
            let lowered_for_type =
                lower_type_ref_with_aliases(&definition.for_type, visible_type_aliases).ok();
            let mut impl_methods = definition.methods.clone();
            for trait_method in &trait_def.methods {
                if trait_method.default_body.is_none()
                    || impl_methods
                        .iter()
                        .any(|method| method.name == trait_method.name)
                {
                    continue;
                }
                impl_methods.push(build_default_impl_method(definition, trait_method));
            }
            impl_methods
                .into_iter()
                .filter_map(move |method| {
                    lowered_for_type
                        .as_ref()
                        .map(|lowered_for_type| GenericImplMethodTemplate {
                            trait_name: definition.trait_name.clone(),
                            method_name: method.name.clone(),
                            function: build_impl_method_function(
                                definition,
                                &method,
                                &impl_method_symbol_name(
                                    &definition.trait_name,
                                    lowered_for_type,
                                    &method.name,
                                ),
                            ),
                        })
                })
                .collect::<Vec<_>>()
                .into_iter()
        })
        .collect::<Vec<_>>();
    let higher_order_templates = module
        .functions
        .iter()
        .filter(|function| {
            function.params.iter().any(|param| {
                is_callable_type_with_aliases(&param.ty, visible_type_aliases).unwrap_or(false)
            })
        })
        .map(|function| (function.name.clone(), function.clone()))
        .collect::<BTreeMap<_, _>>();
    let higher_order_function_table = module
        .functions
        .iter()
        .map(|function| (function.name.clone(), function.clone()))
        .collect::<BTreeMap<_, _>>();
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
                &generic_impl_method_templates,
                &higher_order_templates,
                &higher_order_function_table,
                signatures,
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

    let original_specialized_functions = std::mem::take(&mut specialized_functions);
    let mut postprocessed_specialized_functions = Vec::new();
    let mut postprocessed_specialized_signatures = Vec::new();
    let mut higher_order_specialization_cache = BTreeSet::new();
    for function in original_specialized_functions {
        let mut higher_order_specialized_templates = Vec::new();
        let higher_order_rewritten = rewrite_higher_order_calls_in_function(
            &function,
            &higher_order_templates,
            &higher_order_function_table,
            &[],
            &BTreeMap::new(),
            &BTreeMap::new(),
            visible_type_aliases,
            &mut higher_order_specialization_cache,
            &mut higher_order_specialized_templates,
        )?;
        let mut extended_generic_templates = generic_templates.clone();
        for template in higher_order_specialized_templates {
            if !template.generic_params.is_empty() {
                extended_generic_templates.insert(template.name.clone(), template);
            }
        }
        let rewritten = rewrite_generic_calls_in_function(
            &higher_order_rewritten,
            &BTreeMap::new(),
            visible_type_aliases,
            &extended_generic_templates,
            &generic_impl_method_templates,
            &higher_order_templates,
            &higher_order_function_table,
            signatures,
            impl_lookup,
            module_struct_table,
            &inferred_function_return_types,
            &mut specialization_cache,
            &mut postprocessed_specialized_functions,
            &mut postprocessed_specialized_signatures,
        )?;
        postprocessed_specialized_functions.push(rewritten);
    }
    specialized_functions = postprocessed_specialized_functions;
    for (name, signature) in postprocessed_specialized_signatures {
        signatures.insert(name, signature);
    }

    for definition in &module.impls {
        if !definition.generic_params.is_empty() || !definition.where_bounds.is_empty() {
            continue;
        }
        let Some(trait_def) = trait_defs.get(&definition.trait_name) else {
            continue;
        };
        let lowered_for_type =
            lower_type_ref_with_aliases(&definition.for_type, visible_type_aliases)?;
        let mut impl_methods = definition
            .methods
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        for trait_method in &trait_def.methods {
            if trait_method.default_body.is_none()
                || impl_methods
                    .iter()
                    .any(|method| method.name == trait_method.name)
            {
                continue;
            }
            impl_methods.push(build_default_impl_method(definition, trait_method));
        }
        for method in &impl_methods {
            if method.params.iter().any(|param| {
                is_callable_type_with_aliases(&param.ty, visible_type_aliases).unwrap_or(false)
            }) {
                continue;
            }
            let symbol_name =
                impl_method_symbol_name(&definition.trait_name, &lowered_for_type, &method.name);
            let signature = FunctionSignature {
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
            };
            signatures.insert(
                impl_method_lookup_key(&lowered_for_type, &method.name),
                signature.clone(),
            );
            signatures.insert(symbol_name.clone(), signature);
            if definition.methods.iter().any(|candidate| candidate.name == method.name) {
                rewritten_module_functions.push(build_impl_method_function(
                    definition,
                    method,
                    &symbol_name,
                ));
            } else if let Some(trait_method) = trait_def.methods.iter().find(|candidate| candidate.name == method.name) {
                rewritten_module_functions.push(build_default_impl_method_function(
                    definition,
                    trait_method,
                    &symbol_name,
                ));
            }
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
        if !definition.generic_params.is_empty() || !definition.where_bounds.is_empty() {
            continue;
        }
        let Some(trait_def) = trait_defs.get(&definition.trait_name) else {
            continue;
        };
        let mut all_methods = definition.methods.clone();
        for trait_method in &trait_def.methods {
            if trait_method.default_body.is_none()
                || all_methods
                    .iter()
                    .any(|method| method.name == trait_method.name)
            {
                continue;
            }
            all_methods.push(build_default_impl_method(definition, trait_method));
        }
        let mut methods = Vec::new();
        for method in &all_methods {
            if method.params.iter().any(|param| {
                is_callable_type_with_aliases(&param.ty, visible_type_aliases).unwrap_or(false)
            }) {
                continue;
            }
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
                    lower_stmt_sequence_with_async(
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
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
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
