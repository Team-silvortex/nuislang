mod aliases;
mod annotations;
mod binary_lowering;
mod call_helpers;
mod call_routing;
mod data_builtins;
mod data_profile_builtins;
mod direct_calls;
mod expr_lowering;
mod generic_rewrite;
mod generics;
mod higher_order;
mod kernel_builtins;
mod lexer;
mod match_lowering;
mod metadata;
mod network_builtins;
mod nova_builtins;
mod parser;
mod return_inference;
mod shader_builtins;
mod stmt_lowering;
mod task_builtins;
#[cfg(test)]
mod tests_control_flow;
#[cfg(test)]
mod tests_generics;
#[cfg(test)]
mod tests_higher_order;
#[cfg(test)]
mod tests_lambda_higher_order;
#[cfg(test)]
mod tests_parse_annotations;
#[cfg(test)]
mod tests_return_inference;
mod types;
mod validation_helpers;

use std::collections::{BTreeMap, BTreeSet};

use self::annotations::{
    extern_function_symbol_name, function_host_symbol_name, validate_const_item,
    validate_export_annotations, validate_extern_host_symbols, validate_function_annotations,
    validate_host_symbol_bridge_annotations, validate_struct_annotations,
};
use self::binary_lowering::lower_binary_expr_with_async;
use self::call_helpers::{
    ensure_ref_like, ensure_spawn_input_safe, ensure_task_like,
    lower_result_observer_call_with_consts, lower_result_wrapper_call_with_consts,
};
use self::call_routing::lower_routed_call_or_core_builtin;
use self::direct_calls::lower_direct_call_builtin_or_named_call;
use self::expr_lowering::{
    lower_expr, lower_expr_with_async, lower_nested_expr_with_async,
    lower_nested_expr_with_async_and_consts,
};
use self::generic_rewrite::rewrite_generic_calls_in_function;
use self::higher_order::expand_higher_order_functions;
use self::metadata::{helper_visible_struct_annotations, lower_ast_attributes, ModuleConstValue};
use self::return_inference::infer_missing_function_return_type;
use self::stmt_lowering::lower_stmt_with_async;
use self::validation_helpers::{
    async_boundary_violation_detail, async_parameter_violation_detail, render_type_name,
    select_expected_semantic_token_type, validate_test_function_signature, validate_type_ref,
};
use aliases::*;
use nuis_semantics::model::{
    AstBinaryOp, AstExpr, AstFunction, AstImplDef, AstMatchArm, AstModule, AstParam, AstStmt,
    AstTypeAlias, AstTypeRef, AstVisibility, NirExpr, NirExternFunction, NirExternInterface,
    NirFunction, NirGenericParam, NirImplDef, NirImplMethod, NirModule, NirStmt, NirStructDef,
    NirStructField, NirTraitDef, NirTraitMethodSig, NirTypeAlias, NirTypeRef, NirUse,
    NirVisibility,
};
use types::*;

pub fn frontend_name() -> &'static str {
    "nuisc-parser-minimal"
}

fn lower_visibility(visibility: AstVisibility) -> NirVisibility {
    match visibility {
        AstVisibility::Private => NirVisibility::Private,
        AstVisibility::Public => NirVisibility::Public,
    }
}

fn is_public_visibility(visibility: AstVisibility) -> bool {
    matches!(visibility, AstVisibility::Public)
}

pub fn parse_nuis_ast(input: &str) -> Result<AstModule, String> {
    let tokens = lexer::tokenize(input)?;
    let mut parser = parser::Parser::new(tokens);
    parser.parse_module()
}

pub fn lower_ast_to_nir(module: &AstModule) -> Result<NirModule, String> {
    lower_project_ast_to_nir(module, &[])
}

pub fn lower_project_ast_to_nir(
    module: &AstModule,
    local_modules: &[AstModule],
) -> Result<NirModule, String> {
    let expanded_module = expand_module_lambdas(module)?;
    let local_cpu_helpers = expanded_module
        .uses
        .iter()
        .filter(|item| item.domain == expanded_module.domain)
        .filter_map(|item| {
            local_modules
                .iter()
                .find(|candidate| candidate.domain == item.domain && candidate.unit == item.unit)
        })
        .collect::<Vec<_>>();
    let visible_type_aliases = build_visible_type_alias_map(&expanded_module, &local_cpu_helpers)?;
    let expanded_module = expand_higher_order_functions(&expanded_module, &visible_type_aliases)?;
    let expanded_module = expand_effectful_match_scrutinees(&expanded_module);
    let module = &expanded_module;
    validate_export_annotations(module)?;
    validate_extern_host_symbols(module)?;
    validate_host_symbol_bridge_annotations(module)?;
    for definition in &module.structs {
        validate_struct_annotations(definition)?;
    }
    for constant in &module.consts {
        validate_const_item(constant)?;
    }
    for function in &module.functions {
        validate_function_annotations(function)?;
    }

    let struct_defs = module
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
                            ty: lower_type_ref_with_aliases(&field.ty, &visible_type_aliases)?,
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
                                        &visible_type_aliases,
                                    )?,
                                })
                            })
                            .collect::<Result<Vec<_>, String>>()?,
                    })
                })
        }))
        .collect::<Result<Vec<_>, String>>()?;
    let struct_table = struct_defs
        .iter()
        .map(|definition| (definition.name.clone(), definition.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut signatures = BTreeMap::<String, FunctionSignature>::new();
    for function in &module.externs {
        let symbol_name = extern_function_symbol_name(function)?;
        signatures.insert(
            function.name.clone(),
            FunctionSignature {
                abi: function.abi.clone(),
                interface: None,
                symbol_name,
                params: function
                    .params
                    .iter()
                    .map(|param| lower_type_ref_with_aliases(&param.ty, &visible_type_aliases))
                    .collect::<Result<Vec<_>, _>>()?,
                return_type: Some(lower_type_ref_with_aliases(
                    &function.return_type,
                    &visible_type_aliases,
                )?),
                is_extern: true,
                is_async: false,
            },
        );
    }
    for interface in &module.extern_interfaces {
        for function in &interface.methods {
            let symbol_name = extern_function_symbol_name(function)?;
            signatures.insert(
                format!("{}.{}", interface.name, function.name),
                FunctionSignature {
                    abi: function.abi.clone(),
                    interface: Some(interface.name.clone()),
                    symbol_name,
                    params: function
                        .params
                        .iter()
                        .map(|param| lower_type_ref_with_aliases(&param.ty, &visible_type_aliases))
                        .collect::<Result<Vec<_>, _>>()?,
                    return_type: Some(lower_type_ref_with_aliases(
                        &function.return_type,
                        &visible_type_aliases,
                    )?),
                    is_extern: true,
                    is_async: false,
                },
            );
        }
    }
    for helper in &local_cpu_helpers {
        for function in helper
            .externs
            .iter()
            .filter(|function| is_public_visibility(function.visibility))
        {
            let symbol_name = extern_function_symbol_name(function)?;
            let signature = FunctionSignature {
                abi: function.abi.clone(),
                interface: None,
                symbol_name,
                params: function
                    .params
                    .iter()
                    .map(|param| lower_type_ref_with_aliases(&param.ty, &visible_type_aliases))
                    .collect::<Result<Vec<_>, _>>()?,
                return_type: Some(lower_type_ref_with_aliases(
                    &function.return_type,
                    &visible_type_aliases,
                )?),
                is_extern: true,
                is_async: false,
            };
            signatures.insert(
                format!("{}.{}", helper.unit, function.name),
                signature.clone(),
            );
            signatures.entry(function.name.clone()).or_insert(signature);
        }
        for interface in helper
            .extern_interfaces
            .iter()
            .filter(|interface| is_public_visibility(interface.visibility))
        {
            for function in interface
                .methods
                .iter()
                .filter(|function| is_public_visibility(function.visibility))
            {
                let symbol_name = extern_function_symbol_name(function)?;
                let signature = FunctionSignature {
                    abi: function.abi.clone(),
                    interface: Some(interface.name.clone()),
                    symbol_name,
                    params: function
                        .params
                        .iter()
                        .map(|param| lower_type_ref_with_aliases(&param.ty, &visible_type_aliases))
                        .collect::<Result<Vec<_>, _>>()?,
                    return_type: Some(lower_type_ref_with_aliases(
                        &function.return_type,
                        &visible_type_aliases,
                    )?),
                    is_extern: true,
                    is_async: false,
                };
                signatures.insert(
                    format!("{}.{}.{}", helper.unit, interface.name, function.name),
                    signature.clone(),
                );
                signatures
                    .entry(format!("{}.{}", interface.name, function.name))
                    .or_insert_with(|| signature.clone());
            }
        }
    }
    let module_struct_table = module
        .structs
        .iter()
        .map(|definition| (definition.name.clone(), definition.clone()))
        .collect::<BTreeMap<_, _>>();
    let impl_lookup = module
        .impls
        .iter()
        .map(|definition| {
            Ok((
                (
                    definition.trait_name.clone(),
                    lower_type_ref_with_aliases(&definition.for_type, &visible_type_aliases)?
                        .render(),
                ),
                definition.clone(),
            ))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    let mut generic_templates = BTreeMap::<String, AstFunction>::new();
    let mut concrete_module_functions = Vec::new();
    for function in &module.functions {
        let host_symbol = function_host_symbol_name(function)?;
        let is_host_bridge = host_symbol.is_some();
        let signature = FunctionSignature {
            abi: if is_host_bridge {
                "c".to_owned()
            } else {
                "nuis".to_owned()
            },
            interface: None,
            symbol_name: host_symbol.unwrap_or_else(|| function.name.clone()),
            params: function
                .params
                .iter()
                .map(|param| lower_type_ref_with_aliases(&param.ty, &visible_type_aliases))
                .collect::<Result<Vec<_>, _>>()?,
            return_type: function
                .return_type
                .as_ref()
                .map(|ty| lower_type_ref_with_aliases(ty, &visible_type_aliases))
                .transpose()?,
            is_extern: is_host_bridge,
            is_async: function.is_async,
        };
        signatures.insert(function.name.clone(), signature);
        if is_host_bridge {
            continue;
        }
        if function.generic_params.is_empty() {
            concrete_module_functions.push(function.clone());
        } else {
            generic_templates.insert(function.name.clone(), function.clone());
        }
    }
    for helper in &local_cpu_helpers {
        for function in helper
            .functions
            .iter()
            .filter(|function| is_public_visibility(function.visibility))
        {
            let signature = FunctionSignature {
                abi: "nuis".to_owned(),
                interface: None,
                symbol_name: format!("{}.{}", helper.unit, function.name),
                params: function
                    .params
                    .iter()
                    .map(|param| lower_type_ref_with_aliases(&param.ty, &visible_type_aliases))
                    .collect::<Result<Vec<_>, _>>()?,
                return_type: function
                    .return_type
                    .as_ref()
                    .map(|ty| lower_type_ref_with_aliases(ty, &visible_type_aliases))
                    .transpose()?,
                is_extern: false,
                is_async: function.is_async,
            };
            signatures.insert(
                format!("{}.{}", helper.unit, function.name),
                signature.clone(),
            );
            signatures.entry(function.name.clone()).or_insert(signature);
        }
    }

    let mut helper_const_maps = BTreeMap::<String, BTreeMap<String, ModuleConstValue>>::new();
    let mut visible_helper_consts = BTreeMap::<String, ModuleConstValue>::new();
    for helper in &local_cpu_helpers {
        let helper_aliases = build_visible_type_alias_map(helper, &[])?;
        let (_, helper_consts) = lower_module_const_items(
            helper,
            &BTreeMap::new(),
            &helper_aliases,
            &signatures,
            &struct_table,
        )?;
        for (name, constant) in &helper_consts {
            if matches!(constant.visibility, NirVisibility::Public) {
                visible_helper_consts.insert(format!("{}.{}", helper.unit, name), constant.clone());
                visible_helper_consts
                    .entry(name.clone())
                    .or_insert_with(|| constant.clone());
            }
        }
        helper_const_maps.insert(helper.unit.clone(), helper_consts);
    }
    let (lowered_consts, module_local_consts) = lower_module_const_items(
        module,
        &visible_helper_consts,
        &visible_type_aliases,
        &signatures,
        &struct_table,
    )?;
    let mut module_const_values = visible_helper_consts.clone();
    module_const_values.extend(module_local_consts.clone());
    let module_const_env = ast_const_type_env(&module_const_values);

    let function_return_types = build_function_return_type_table(
        module,
        &concrete_module_functions,
        &generic_templates,
        &local_cpu_helpers,
        &visible_type_aliases,
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
                &module_const_env,
                &visible_type_aliases,
                &generic_templates,
                &impl_lookup,
                &module_struct_table,
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
                    &module_const_env,
                    &impl_lookup,
                    &module_struct_table,
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
                        &visible_type_aliases,
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
            lower_type_ref_with_aliases(&definition.for_type, &visible_type_aliases)?;
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
                        .map(|param| lower_type_ref_with_aliases(&param.ty, &visible_type_aliases))
                        .collect::<Result<Vec<_>, _>>()?,
                    return_type: method
                        .return_type
                        .as_ref()
                        .map(|ty| lower_type_ref_with_aliases(ty, &visible_type_aliases))
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
                &module_const_values,
                &visible_type_aliases,
                &signatures,
                &struct_table,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    for function in &mut specialized_functions {
        if function.return_type.is_none() {
            if let Some(inferred_return_type) = infer_missing_function_return_type(
                function,
                &module_const_env,
                &impl_lookup,
                &module_struct_table,
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
                    &visible_type_aliases,
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
                    &module_const_values,
                    &visible_type_aliases,
                    &signatures,
                    &struct_table,
                )
            })
            .collect::<Result<Vec<_>, _>>()?,
    );
    for helper in &local_cpu_helpers {
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
                &visible_type_aliases,
                &signatures,
                &struct_table,
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
                                .map(|param| lower_param_with_aliases(param, &visible_type_aliases))
                                .collect::<Result<Vec<_>, _>>()?,
                            return_type: method
                                .return_type
                                .as_ref()
                                .map(|ty| lower_type_ref_with_aliases(ty, &visible_type_aliases))
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
                    lower_type_ref_with_aliases(&param.ty, &visible_type_aliases)?,
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
                        &module_const_values,
                        method.return_type.as_ref(),
                        &visible_type_aliases,
                        &signatures,
                        &struct_table,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            methods.push(NirImplMethod {
                name: method.name.clone(),
                params: method
                    .params
                    .iter()
                    .map(|param| lower_param_with_aliases(param, &visible_type_aliases))
                    .collect::<Result<Vec<_>, _>>()?,
                return_type: method
                    .return_type
                    .as_ref()
                    .map(|ty| lower_type_ref_with_aliases(ty, &visible_type_aliases))
                    .transpose()?,
                body,
            });
        }
        lowered_impls.push(NirImplDef {
            trait_name: definition.trait_name.clone(),
            for_type: lower_type_ref_with_aliases(&definition.for_type, &visible_type_aliases)?,
            methods,
        });
    }

    let nir = NirModule {
        uses: module
            .uses
            .iter()
            .map(|item| NirUse {
                domain: item.domain.clone(),
                unit: item.unit.clone(),
            })
            .collect(),
        domain: module.domain.clone(),
        unit: module.unit.clone(),
        type_aliases: module
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
                                    .map(|ty| {
                                        lower_type_ref_with_aliases(ty, &visible_type_aliases)
                                    })
                                    .transpose()?,
                            })
                        })
                        .collect::<Result<Vec<_>, String>>()?,
                    target: lower_type_ref_with_aliases(&alias.target, &visible_type_aliases)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        externs: module
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
                        .map(|param| lower_param_with_aliases(param, &visible_type_aliases))
                        .collect::<Result<Vec<_>, _>>()?,
                    return_type: lower_type_ref_with_aliases(
                        &function.return_type,
                        &visible_type_aliases,
                    )?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        extern_interfaces: module
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
                                    .map(|param| {
                                        lower_param_with_aliases(param, &visible_type_aliases)
                                    })
                                    .collect::<Result<Vec<_>, _>>()?,
                                return_type: lower_type_ref_with_aliases(
                                    &function.return_type,
                                    &visible_type_aliases,
                                )?,
                            })
                        })
                        .collect::<Result<Vec<_>, String>>()?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        consts: lowered_consts,
        structs: struct_defs,
        traits: lowered_traits,
        impls: lowered_impls,
        functions: lowered_functions,
    };
    validate_declared_nir_types(&nir)?;
    Ok(nir)
}

fn ast_expr_requires_match_hoist(expr: &AstExpr) -> bool {
    match expr {
        AstExpr::Call { .. }
        | AstExpr::Invoke { .. }
        | AstExpr::MethodCall { .. }
        | AstExpr::Await(_)
        | AstExpr::Instantiate { .. } => true,
        AstExpr::FieldAccess { base, .. } => ast_expr_requires_match_hoist(base),
        AstExpr::Binary { lhs, rhs, .. } => {
            ast_expr_requires_match_hoist(lhs) || ast_expr_requires_match_hoist(rhs)
        }
        AstExpr::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| ast_expr_requires_match_hoist(value)),
        AstExpr::Bool(_)
        | AstExpr::Text(_)
        | AstExpr::Int(_)
        | AstExpr::Var(_)
        | AstExpr::Lambda { .. } => false,
    }
}

fn expand_effectful_match_scrutinees(module: &AstModule) -> AstModule {
    let mut expanded = module.clone();
    expanded.functions = module
        .functions
        .iter()
        .map(rewrite_effectful_match_scrutinees_in_function)
        .collect();
    expanded
}

fn rewrite_effectful_match_scrutinees_in_function(function: &AstFunction) -> AstFunction {
    let mut counter = 0usize;
    let mut rewritten = function.clone();
    rewritten.body = rewrite_effectful_match_scrutinees_in_block(&function.body, &mut counter);
    rewritten
}

fn rewrite_effectful_match_scrutinees_in_block(
    body: &[AstStmt],
    counter: &mut usize,
) -> Vec<AstStmt> {
    let mut rewritten = Vec::new();
    for stmt in body {
        match stmt {
            AstStmt::If {
                condition,
                then_body,
                else_body,
            } => rewritten.push(AstStmt::If {
                condition: condition.clone(),
                then_body: rewrite_effectful_match_scrutinees_in_block(then_body, counter),
                else_body: rewrite_effectful_match_scrutinees_in_block(else_body, counter),
            }),
            AstStmt::Match { value, arms } if ast_expr_requires_match_hoist(value) => {
                let temp_name = format!("__match_scrutinee_{counter}");
                *counter += 1;
                rewritten.push(AstStmt::Let {
                    name: temp_name.clone(),
                    ty: None,
                    value: value.clone(),
                });
                rewritten.push(AstStmt::Match {
                    value: AstExpr::Var(temp_name),
                    arms: arms
                        .iter()
                        .map(|arm| AstMatchArm {
                            pattern: arm.pattern.clone(),
                            guard: arm.guard.clone(),
                            body: rewrite_effectful_match_scrutinees_in_block(&arm.body, counter),
                        })
                        .collect(),
                });
            }
            AstStmt::Match { value, arms } => rewritten.push(AstStmt::Match {
                value: value.clone(),
                arms: arms
                    .iter()
                    .map(|arm| AstMatchArm {
                        pattern: arm.pattern.clone(),
                        guard: arm.guard.clone(),
                        body: rewrite_effectful_match_scrutinees_in_block(&arm.body, counter),
                    })
                    .collect(),
            }),
            AstStmt::While { condition, body } => rewritten.push(AstStmt::While {
                condition: condition.clone(),
                body: rewrite_effectful_match_scrutinees_in_block(body, counter),
            }),
            other => rewritten.push(other.clone()),
        }
    }
    rewritten
}

fn expand_module_lambdas(module: &AstModule) -> Result<AstModule, String> {
    let module_const_names = module
        .consts
        .iter()
        .map(|constant| constant.name.clone())
        .collect::<BTreeSet<_>>();
    let mut expanded = module.clone();
    expanded.functions.clear();
    for function in &module.functions {
        let (rewritten, synthesized) = expand_function_lambdas(function, &module_const_names)?;
        expanded.functions.extend(synthesized);
        expanded.functions.push(rewritten);
    }
    Ok(expanded)
}

fn expand_function_lambdas(
    function: &AstFunction,
    module_const_names: &BTreeSet<String>,
) -> Result<(AstFunction, Vec<AstFunction>), String> {
    let mut counter = 0usize;
    let mut synthesized = Vec::new();
    let visible_locals = function
        .params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let body = expand_lambda_block(
        &function.body,
        &BTreeMap::new(),
        &visible_locals,
        module_const_names,
        &function.name,
        &mut counter,
        &mut synthesized,
    )?;
    let mut rewritten = function.clone();
    rewritten.body = body;
    Ok((rewritten, synthesized))
}

fn synthesize_lambda_function(
    params: &[AstParam],
    return_type: &Option<AstTypeRef>,
    body: &[AstStmt],
    lambda_aliases: &BTreeMap<String, String>,
    outer_locals: &BTreeSet<String>,
    module_const_names: &BTreeSet<String>,
    owning_function_name: &str,
    counter: &mut usize,
    synthesized: &mut Vec<AstFunction>,
) -> Result<String, String> {
    let Some(lambda_return_type) = return_type.clone() else {
        return Err("inline lambda currently requires an explicit return type".to_owned());
    };
    let mut lambda_locals = params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    validate_lambda_block_no_capture(body, &mut lambda_locals, outer_locals)?;
    let synthesized_name = format!("__lambda_{}_{}", owning_function_name, *counter);
    *counter += 1;
    let lambda_body = expand_lambda_block(
        body,
        lambda_aliases,
        &params
            .iter()
            .map(|param| param.name.clone())
            .collect::<BTreeSet<_>>(),
        module_const_names,
        owning_function_name,
        counter,
        synthesized,
    )?;
    synthesized.push(AstFunction {
        visibility: AstVisibility::Private,
        name: synthesized_name.clone(),
        attributes: Vec::new(),
        test_name: None,
        test_ignored: false,
        test_should_fail: false,
        test_reason: None,
        test_timeout_ms: None,
        test_clock_domain: None,
        test_clock_policy: None,
        is_async: false,
        generic_params: Vec::new(),
        params: params.to_vec(),
        return_type: Some(lambda_return_type),
        body: lambda_body,
    });
    Ok(synthesized_name)
}

#[allow(clippy::too_many_arguments)]
fn expand_lambda_block(
    body: &[AstStmt],
    lambda_aliases: &BTreeMap<String, String>,
    visible_locals: &BTreeSet<String>,
    module_const_names: &BTreeSet<String>,
    owning_function_name: &str,
    counter: &mut usize,
    synthesized: &mut Vec<AstFunction>,
) -> Result<Vec<AstStmt>, String> {
    let mut aliases = lambda_aliases.clone();
    let mut locals = visible_locals.clone();
    let mut rewritten = Vec::new();
    for stmt in body {
        match stmt {
            AstStmt::Let {
                name,
                ty,
                value:
                    AstExpr::Lambda {
                        params,
                        return_type,
                        body,
                    },
            } => {
                if ty.is_some() {
                    return Err(format!(
                        "lambda binding `{name}` currently does not support an explicit type annotation"
                    ));
                }
                let synthesized_name = synthesize_lambda_function(
                    params,
                    return_type,
                    body,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )
                .map_err(|error| {
                    if error == "inline lambda currently requires an explicit return type" {
                        format!(
                            "lambda binding `{name}` currently requires an explicit return type"
                        )
                    } else {
                        error
                    }
                })?;
                aliases.insert(name.clone(), synthesized_name);
                locals.insert(name.clone());
            }
            AstStmt::Let { name, ty, value } => {
                let rewritten_value = rewrite_lambda_expr(
                    value,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?;
                rewritten.push(AstStmt::Let {
                    name: name.clone(),
                    ty: ty.clone(),
                    value: rewritten_value,
                });
                aliases.remove(name);
                locals.insert(name.clone());
            }
            AstStmt::Const { name, ty, value } => {
                let rewritten_value = rewrite_lambda_expr(
                    value,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?;
                rewritten.push(AstStmt::Const {
                    name: name.clone(),
                    ty: ty.clone(),
                    value: rewritten_value,
                });
                aliases.remove(name);
                locals.insert(name.clone());
            }
            AstStmt::Print(value) => rewritten.push(AstStmt::Print(rewrite_lambda_expr(
                value,
                &aliases,
                &locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?)),
            AstStmt::Await(value) => rewritten.push(AstStmt::Await(rewrite_lambda_expr(
                value,
                &aliases,
                &locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?)),
            AstStmt::If {
                condition,
                then_body,
                else_body,
            } => rewritten.push(AstStmt::If {
                condition: rewrite_lambda_expr(
                    condition,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?,
                then_body: expand_lambda_block(
                    then_body,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?,
                else_body: expand_lambda_block(
                    else_body,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?,
            }),
            AstStmt::Match { value, arms } => rewritten.push(AstStmt::Match {
                value: rewrite_lambda_expr(
                    value,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?,
                arms: arms
                    .iter()
                    .map(|arm| {
                        Ok(AstMatchArm {
                            pattern: arm.pattern.clone(),
                            guard: arm
                                .guard
                                .clone()
                                .map(|guard| {
                                    rewrite_lambda_expr(
                                        &guard,
                                        &aliases,
                                        &locals,
                                        module_const_names,
                                        owning_function_name,
                                        counter,
                                        synthesized,
                                    )
                                })
                                .transpose()?,
                            body: expand_lambda_block(
                                &arm.body,
                                &aliases,
                                &locals,
                                module_const_names,
                                owning_function_name,
                                counter,
                                synthesized,
                            )?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            }),
            AstStmt::While { condition, body } => rewritten.push(AstStmt::While {
                condition: rewrite_lambda_expr(
                    condition,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?,
                body: expand_lambda_block(
                    body,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?,
            }),
            AstStmt::Expr(expr) => rewritten.push(AstStmt::Expr(rewrite_lambda_expr(
                expr,
                &aliases,
                &locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?)),
            AstStmt::Return(value) => rewritten.push(AstStmt::Return(match value {
                Some(value) => Some(rewrite_lambda_expr(
                    value,
                    &aliases,
                    &locals,
                    module_const_names,
                    owning_function_name,
                    counter,
                    synthesized,
                )?),
                None => None,
            })),
            AstStmt::Break => rewritten.push(AstStmt::Break),
            AstStmt::Continue => rewritten.push(AstStmt::Continue),
        }
    }
    Ok(rewritten)
}

fn validate_lambda_block_no_capture(
    body: &[AstStmt],
    visible_locals: &mut BTreeSet<String>,
    outer_locals: &BTreeSet<String>,
) -> Result<(), String> {
    for stmt in body {
        match stmt {
            AstStmt::Let { name, value, .. } => {
                validate_lambda_expr_no_capture(value, visible_locals, outer_locals)?;
                visible_locals.insert(name.clone());
            }
            AstStmt::Const { name, value, .. } => {
                validate_lambda_expr_no_capture(value, visible_locals, outer_locals)?;
                visible_locals.insert(name.clone());
            }
            AstStmt::Print(value) | AstStmt::Await(value) | AstStmt::Expr(value) => {
                validate_lambda_expr_no_capture(value, visible_locals, outer_locals)?;
            }
            AstStmt::If {
                condition,
                then_body,
                else_body,
            } => {
                validate_lambda_expr_no_capture(condition, visible_locals, outer_locals)?;
                let mut then_locals = visible_locals.clone();
                let mut else_locals = visible_locals.clone();
                validate_lambda_block_no_capture(then_body, &mut then_locals, outer_locals)?;
                validate_lambda_block_no_capture(else_body, &mut else_locals, outer_locals)?;
            }
            AstStmt::Match { value, arms } => {
                validate_lambda_expr_no_capture(value, visible_locals, outer_locals)?;
                for arm in arms {
                    let mut arm_locals = visible_locals.clone();
                    validate_lambda_block_no_capture(&arm.body, &mut arm_locals, outer_locals)?;
                }
            }
            AstStmt::While { condition, body } => {
                validate_lambda_expr_no_capture(condition, visible_locals, outer_locals)?;
                let mut loop_locals = visible_locals.clone();
                validate_lambda_block_no_capture(body, &mut loop_locals, outer_locals)?;
            }
            AstStmt::Return(Some(value)) => {
                validate_lambda_expr_no_capture(value, visible_locals, outer_locals)?;
            }
            AstStmt::Return(None) | AstStmt::Break | AstStmt::Continue => {}
        }
    }
    Ok(())
}

fn validate_lambda_expr_no_capture(
    expr: &AstExpr,
    visible_locals: &BTreeSet<String>,
    outer_locals: &BTreeSet<String>,
) -> Result<(), String> {
    match expr {
        AstExpr::Var(name) if outer_locals.contains(name) && !visible_locals.contains(name) => {
            Err(format!(
                "lambda currently does not support capturing outer local `{name}`"
            ))
        }
        AstExpr::Lambda { .. } => Err(
            "nested or inline lambdas are not supported in the current MVP; bind lambdas with `let name = |...| -> ... { ... };` only"
                .to_owned(),
        ),
        AstExpr::Await(value) => {
            validate_lambda_expr_no_capture(value, visible_locals, outer_locals)
        }
        AstExpr::Invoke { callee, args } => {
            validate_lambda_expr_no_capture(callee, visible_locals, outer_locals)?;
            for arg in args {
                validate_lambda_expr_no_capture(arg, visible_locals, outer_locals)?;
            }
            Ok(())
        }
        AstExpr::Call { args, .. } => {
            for arg in args {
                validate_lambda_expr_no_capture(arg, visible_locals, outer_locals)?;
            }
            Ok(())
        }
        AstExpr::MethodCall { receiver, args, .. } => {
            validate_lambda_expr_no_capture(receiver, visible_locals, outer_locals)?;
            for arg in args {
                validate_lambda_expr_no_capture(arg, visible_locals, outer_locals)?;
            }
            Ok(())
        }
        AstExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                validate_lambda_expr_no_capture(value, visible_locals, outer_locals)?;
            }
            Ok(())
        }
        AstExpr::FieldAccess { base, .. } => {
            validate_lambda_expr_no_capture(base, visible_locals, outer_locals)
        }
        AstExpr::Binary { lhs, rhs, .. } => {
            validate_lambda_expr_no_capture(lhs, visible_locals, outer_locals)?;
            validate_lambda_expr_no_capture(rhs, visible_locals, outer_locals)
        }
        AstExpr::Bool(_)
        | AstExpr::Text(_)
        | AstExpr::Int(_)
        | AstExpr::Var(_)
        | AstExpr::Instantiate { .. } => Ok(()),
    }
}

fn rewrite_lambda_expr(
    expr: &AstExpr,
    lambda_aliases: &BTreeMap<String, String>,
    visible_locals: &BTreeSet<String>,
    module_const_names: &BTreeSet<String>,
    owning_function_name: &str,
    counter: &mut usize,
    synthesized: &mut Vec<AstFunction>,
) -> Result<AstExpr, String> {
    Ok(match expr {
        AstExpr::Var(name)
            if lambda_aliases.contains_key(name) && !module_const_names.contains(name) =>
        {
            AstExpr::Var(
                lambda_aliases
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| name.clone()),
            )
        }
        AstExpr::Lambda { .. } => {
            let AstExpr::Lambda {
                params,
                return_type,
                body,
            } = expr
            else {
                unreachable!();
            };
            let synthesized_name = synthesize_lambda_function(
                params,
                return_type,
                body,
                lambda_aliases,
                visible_locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?;
            AstExpr::Var(synthesized_name)
        }
        AstExpr::Await(value) => AstExpr::Await(Box::new(rewrite_lambda_expr(
            value,
            lambda_aliases,
            visible_locals,
            module_const_names,
            owning_function_name,
            counter,
            synthesized,
        )?)),
        AstExpr::Invoke { callee, args } => {
            let rewritten_args = args
                .iter()
                .map(|arg| {
                    rewrite_lambda_expr(
                        arg,
                        lambda_aliases,
                        visible_locals,
                        module_const_names,
                        owning_function_name,
                        counter,
                        synthesized,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            match callee.as_ref() {
                AstExpr::Lambda {
                    params,
                    return_type,
                    body,
                } => {
                    let synthesized_name = synthesize_lambda_function(
                        params,
                        return_type,
                        body,
                        lambda_aliases,
                        visible_locals,
                        module_const_names,
                        owning_function_name,
                        counter,
                        synthesized,
                    )?;
                    AstExpr::Call {
                        callee: synthesized_name,
                        args: rewritten_args,
                    }
                }
                AstExpr::Var(name) => AstExpr::Call {
                    callee: lambda_aliases
                        .get(name)
                        .cloned()
                        .unwrap_or_else(|| name.clone()),
                    args: rewritten_args,
                },
                _ => {
                    return Err(
                        "only immediate no-capture lambda invocation and named function invocation are supported in the current MVP"
                            .to_owned(),
                    )
                }
            }
        }
        AstExpr::Call { callee, args } => AstExpr::Call {
            callee: lambda_aliases
                .get(callee)
                .cloned()
                .unwrap_or_else(|| callee.clone()),
            args: args
                .iter()
                .map(|arg| {
                    rewrite_lambda_expr(
                        arg,
                        lambda_aliases,
                        visible_locals,
                        module_const_names,
                        owning_function_name,
                        counter,
                        synthesized,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstExpr::MethodCall {
            receiver,
            method,
            args,
        } => AstExpr::MethodCall {
            receiver: Box::new(rewrite_lambda_expr(
                receiver,
                lambda_aliases,
                visible_locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?),
            method: method.clone(),
            args: args
                .iter()
                .map(|arg| {
                    rewrite_lambda_expr(
                        arg,
                        lambda_aliases,
                        visible_locals,
                        module_const_names,
                        owning_function_name,
                        counter,
                        synthesized,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        },
        AstExpr::StructLiteral { type_name, fields } => AstExpr::StructLiteral {
            type_name: type_name.clone(),
            fields: fields
                .iter()
                .map(|(name, value)| {
                    Ok((
                        name.clone(),
                        rewrite_lambda_expr(
                            value,
                            lambda_aliases,
                            visible_locals,
                            module_const_names,
                            owning_function_name,
                            counter,
                            synthesized,
                        )?,
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?,
        },
        AstExpr::FieldAccess { base, field } => AstExpr::FieldAccess {
            base: Box::new(rewrite_lambda_expr(
                base,
                lambda_aliases,
                visible_locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?),
            field: field.clone(),
        },
        AstExpr::Binary { op, lhs, rhs } => AstExpr::Binary {
            op: *op,
            lhs: Box::new(rewrite_lambda_expr(
                lhs,
                lambda_aliases,
                visible_locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?),
            rhs: Box::new(rewrite_lambda_expr(
                rhs,
                lambda_aliases,
                visible_locals,
                module_const_names,
                owning_function_name,
                counter,
                synthesized,
            )?),
        },
        AstExpr::Bool(_)
        | AstExpr::Text(_)
        | AstExpr::Int(_)
        | AstExpr::Var(_)
        | AstExpr::Instantiate { .. } => expr.clone(),
    })
}
pub fn parse_nuis_module(input: &str) -> Result<NirModule, String> {
    let ast = parse_nuis_ast(input)?;
    lower_ast_to_nir(&ast)
}

pub fn collect_nir_tests<'a>(module: &'a NirModule) -> Vec<&'a NirFunction> {
    module
        .functions
        .iter()
        .filter(|function| function.test_name.is_some())
        .collect()
}

#[derive(Clone)]
struct FunctionSignature {
    abi: String,
    interface: Option<String>,
    symbol_name: String,
    params: Vec<NirTypeRef>,
    return_type: Option<NirTypeRef>,
    is_extern: bool,
    is_async: bool,
}

fn impl_method_lookup_key(for_type: &NirTypeRef, method: &str) -> String {
    format!("{}.{}", for_type.render(), method)
}

fn impl_method_symbol_name(trait_name: &str, for_type: &NirTypeRef, method: &str) -> String {
    let rendered = for_type
        .render()
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => ch,
            _ => '_',
        })
        .collect::<String>();
    format!("impl.{}.for.{}.{}", trait_name, rendered, method)
}

fn build_impl_method_function(
    definition: &AstImplDef,
    method: &nuis_semantics::model::AstImplMethod,
    symbol_name: &str,
) -> AstFunction {
    AstFunction {
        name: symbol_name.to_owned(),
        visibility: AstVisibility::Private,
        attributes: vec![],
        test_name: None,
        test_ignored: false,
        test_should_fail: false,
        test_reason: None,
        test_timeout_ms: None,
        test_clock_domain: None,
        test_clock_policy: None,
        is_async: false,
        generic_params: vec![],
        params: method.params.clone(),
        return_type: method.return_type.clone().or_else(|| {
            Some(AstTypeRef {
                name: definition.for_type.name.clone(),
                generic_args: definition.for_type.generic_args.clone(),
                is_optional: definition.for_type.is_optional,
                is_ref: definition.for_type.is_ref,
            })
        }),
        body: method.body.clone(),
    }
}

fn lower_function(
    function: &AstFunction,
    current_domain: &str,
    _current_unit: &str,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    type_aliases: &BTreeMap<String, AstTypeAlias>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirFunction, String> {
    let mut bindings = BTreeMap::<String, NirTypeRef>::new();
    for param in &function.params {
        bindings.insert(
            param.name.clone(),
            lower_type_ref_with_aliases(&param.ty, type_aliases)?,
        );
    }

    Ok(NirFunction {
        name: function.name.clone(),
        annotations: lower_ast_attributes(&function.attributes),
        visibility: lower_visibility(function.visibility),
        test_name: function.test_name.clone(),
        test_ignored: function.test_ignored,
        test_should_fail: function.test_should_fail,
        test_reason: function.test_reason.clone(),
        test_timeout_ms: function.test_timeout_ms,
        test_clock_domain: function.test_clock_domain.clone(),
        test_clock_policy: function.test_clock_policy,
        is_async: function.is_async,
        generic_params: function
            .generic_params
            .iter()
            .map(|param| {
                Ok(NirGenericParam {
                    name: param.name.clone(),
                    bound: param
                        .bound
                        .as_ref()
                        .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
                        .transpose()?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        params: function
            .params
            .iter()
            .map(|param| lower_param_with_aliases(param, type_aliases))
            .collect::<Result<Vec<_>, _>>()?,
        return_type: function
            .return_type
            .as_ref()
            .map(|ty| lower_type_ref_with_aliases(ty, type_aliases))
            .transpose()?,
        body: function
            .body
            .iter()
            .map(|stmt| {
                lower_stmt_with_async(
                    stmt,
                    current_domain,
                    function.is_async,
                    &mut bindings,
                    module_consts,
                    function.return_type.as_ref(),
                    type_aliases,
                    signatures,
                    struct_table,
                )
            })
            .collect::<Result<Vec<_>, _>>()?,
    })
}

#[allow(dead_code)]
#[allow(dead_code)]
fn lower_binary_expr(
    op: &AstBinaryOp,
    lhs: &AstExpr,
    rhs: &AstExpr,
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirExpr, String> {
    lower_binary_expr_with_async(
        op,
        lhs,
        rhs,
        current_domain,
        false,
        bindings,
        &BTreeMap::new(),
        signatures,
        struct_table,
    )
}

#[allow(dead_code)]
fn lower_call_expr(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
) -> Result<NirExpr, String> {
    lower_call_expr_with_async(
        callee,
        args,
        current_domain,
        false,
        bindings,
        &BTreeMap::new(),
        signatures,
        struct_table,
        expected,
        false,
    )
}

fn lower_call_expr_with_async(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
    allow_async_calls: bool,
) -> Result<NirExpr, String> {
    if let Some(routed_or_core) = lower_routed_call_or_core_builtin(
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        expected,
    )? {
        return Ok(routed_or_core);
    }
    match callee {
        "nova_header_packet" => {
            let (accent, title_mode) = match args {
                [accent] => (accent, None),
                [accent, title_mode] => (accent, Some(title_mode)),
                _ => return Err("nova_header_packet(...) expects 1 or 2 args".to_owned()),
            };
            let accent = lower_expr(
                accent,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let title_mode = title_mode
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| accent.clone());
            Ok(NirExpr::StructLiteral {
                type_name: "NovaHeaderPacket".to_owned(),
                fields: vec![
                    ("accent".to_owned(), accent),
                    ("title_mode".to_owned(), title_mode),
                ],
            })
        }
        "nova_theme_packet" => {
            let (accent, surface, panel_mode, contrast) = match args {
                [accent, surface, panel_mode, contrast] => (accent, surface, panel_mode, contrast),
                _ => return Err("nova_theme_packet(...) expects 4 args".to_owned()),
            };
            let accent = lower_expr(
                accent,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let surface = lower_expr(
                surface,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let panel_mode = lower_expr(
                panel_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let contrast = lower_expr(
                contrast,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaThemePacket".to_owned(),
                fields: vec![
                    ("accent".to_owned(), accent),
                    ("surface".to_owned(), surface),
                    ("panel_mode".to_owned(), panel_mode),
                    ("contrast".to_owned(), contrast),
                ],
            })
        }
        "nova_surface_packet" => {
            let (density, elevation, grid, sheen) = match args {
                [density, elevation, grid, sheen] => (density, elevation, grid, sheen),
                _ => return Err("nova_surface_packet(...) expects 4 args".to_owned()),
            };
            let density = lower_expr(
                density,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let elevation = lower_expr(
                elevation,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let grid = lower_expr(
                grid,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let sheen = lower_expr(
                sheen,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaSurfacePacket".to_owned(),
                fields: vec![
                    ("density".to_owned(), density),
                    ("elevation".to_owned(), elevation),
                    ("grid".to_owned(), grid),
                    ("sheen".to_owned(), sheen),
                ],
            })
        }
        "nova_viewport_packet" => {
            let (origin_x, origin_y, width, height) = match args {
                [origin_x, origin_y, width, height] => (origin_x, origin_y, width, height),
                _ => return Err("nova_viewport_packet(...) expects 4 args".to_owned()),
            };
            let origin_x = lower_expr(
                origin_x,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let origin_y = lower_expr(
                origin_y,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let width = lower_expr(
                width,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let height = lower_expr(
                height,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaViewportPacket".to_owned(),
                fields: vec![
                    ("origin_x".to_owned(), origin_x),
                    ("origin_y".to_owned(), origin_y),
                    ("width".to_owned(), width),
                    ("height".to_owned(), height),
                ],
            })
        }
        "nova_layer_packet" => {
            let (order, blend, visibility, clip) = match args {
                [order, blend, visibility, clip] => (order, blend, visibility, clip),
                _ => return Err("nova_layer_packet(...) expects 4 args".to_owned()),
            };
            let order = lower_expr(
                order,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let blend = lower_expr(
                blend,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let visibility = lower_expr(
                visibility,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let clip = lower_expr(
                clip,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaLayerPacket".to_owned(),
                fields: vec![
                    ("order".to_owned(), order),
                    ("blend".to_owned(), blend),
                    ("visibility".to_owned(), visibility),
                    ("clip".to_owned(), clip),
                ],
            })
        }
        "nova_scene_packet" => {
            let (root_count, active_camera, light_count, animation_phase) = match args {
                [root_count, active_camera, light_count, animation_phase] => {
                    (root_count, active_camera, light_count, animation_phase)
                }
                _ => return Err("nova_scene_packet(...) expects 4 args".to_owned()),
            };
            let root_count = lower_expr(
                root_count,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let active_camera = lower_expr(
                active_camera,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let light_count = lower_expr(
                light_count,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let animation_phase = lower_expr(
                animation_phase,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaScenePacket".to_owned(),
                fields: vec![
                    ("root_count".to_owned(), root_count),
                    ("active_camera".to_owned(), active_camera),
                    ("light_count".to_owned(), light_count),
                    ("animation_phase".to_owned(), animation_phase),
                ],
            })
        }
        "nova_camera_packet" => {
            let (kind, focus, zoom, orbit) = match args {
                [kind, focus, zoom, orbit] => (kind, focus, zoom, orbit),
                _ => return Err("nova_camera_packet(...) expects 4 args".to_owned()),
            };
            let kind = lower_expr(
                kind,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let focus = lower_expr(
                focus,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let zoom = lower_expr(
                zoom,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let orbit = lower_expr(
                orbit,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaCameraPacket".to_owned(),
                fields: vec![
                    ("kind".to_owned(), kind),
                    ("focus".to_owned(), focus),
                    ("zoom".to_owned(), zoom),
                    ("orbit".to_owned(), orbit),
                ],
            })
        }
        "nova_material_packet" => {
            let (shader_kind, albedo, roughness, emissive) = match args {
                [shader_kind, albedo, roughness, emissive] => {
                    (shader_kind, albedo, roughness, emissive)
                }
                _ => return Err("nova_material_packet(...) expects 4 args".to_owned()),
            };
            let shader_kind = lower_expr(
                shader_kind,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let albedo = lower_expr(
                albedo,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let roughness = lower_expr(
                roughness,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let emissive = lower_expr(
                emissive,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaMaterialPacket".to_owned(),
                fields: vec![
                    ("shader_kind".to_owned(), shader_kind),
                    ("albedo".to_owned(), albedo),
                    ("roughness".to_owned(), roughness),
                    ("emissive".to_owned(), emissive),
                ],
            })
        }
        "nova_light_packet" => {
            let (kind, intensity, range, reactive) = match args {
                [kind, intensity, range, reactive] => (kind, intensity, range, reactive),
                _ => return Err("nova_light_packet(...) expects 4 args".to_owned()),
            };
            let kind = lower_expr(
                kind,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let intensity = lower_expr(
                intensity,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let range = lower_expr(
                range,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let reactive = lower_expr(
                reactive,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaLightPacket".to_owned(),
                fields: vec![
                    ("kind".to_owned(), kind),
                    ("intensity".to_owned(), intensity),
                    ("range".to_owned(), range),
                    ("reactive".to_owned(), reactive),
                ],
            })
        }
        "nova_mesh_packet" => {
            let (primitive, vertex_count, index_count, skinning) = match args {
                [primitive, vertex_count, index_count, skinning] => {
                    (primitive, vertex_count, index_count, skinning)
                }
                _ => return Err("nova_mesh_packet(...) expects 4 args".to_owned()),
            };
            let primitive = lower_expr(
                primitive,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let vertex_count = lower_expr(
                vertex_count,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let index_count = lower_expr(
                index_count,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let skinning = lower_expr(
                skinning,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaMeshPacket".to_owned(),
                fields: vec![
                    ("primitive".to_owned(), primitive),
                    ("vertex_count".to_owned(), vertex_count),
                    ("index_count".to_owned(), index_count),
                    ("skinning".to_owned(), skinning),
                ],
            })
        }
        "nova_transform_packet" => {
            let (translate, rotate, scale, pivot) = match args {
                [translate, rotate, scale, pivot] => (translate, rotate, scale, pivot),
                _ => return Err("nova_transform_packet(...) expects 4 args".to_owned()),
            };
            let translate = lower_expr(
                translate,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let rotate = lower_expr(
                rotate,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let scale = lower_expr(
                scale,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let pivot = lower_expr(
                pivot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaTransformPacket".to_owned(),
                fields: vec![
                    ("translate".to_owned(), translate),
                    ("rotate".to_owned(), rotate),
                    ("scale".to_owned(), scale),
                    ("pivot".to_owned(), pivot),
                ],
            })
        }
        "nova_node_packet" => {
            let (node_id, parent_id, flags, depth) = match args {
                [node_id, parent_id, flags, depth] => (node_id, parent_id, flags, depth),
                _ => return Err("nova_node_packet(...) expects 4 args".to_owned()),
            };
            let node_id = lower_expr(
                node_id,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let parent_id = lower_expr(
                parent_id,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let flags = lower_expr(
                flags,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let depth = lower_expr(
                depth,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaNodePacket".to_owned(),
                fields: vec![
                    ("node_id".to_owned(), node_id),
                    ("parent_id".to_owned(), parent_id),
                    ("flags".to_owned(), flags),
                    ("depth".to_owned(), depth),
                ],
            })
        }
        "nova_scene_link_packet" => {
            let (node_slot, transform_slot, mesh_slot, material_slot, light_slot, layer_slot) =
                match args {
                    [node_slot, transform_slot, mesh_slot, material_slot, light_slot, layer_slot] => {
                        (
                            node_slot,
                            transform_slot,
                            mesh_slot,
                            material_slot,
                            light_slot,
                            layer_slot,
                        )
                    }
                    _ => return Err("nova_scene_link_packet(...) expects 6 args".to_owned()),
                };
            let node_slot = lower_expr(
                node_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let transform_slot = lower_expr(
                transform_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let mesh_slot = lower_expr(
                mesh_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let material_slot = lower_expr(
                material_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let light_slot = lower_expr(
                light_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let layer_slot = lower_expr(
                layer_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaSceneLinkPacket".to_owned(),
                fields: vec![
                    ("node_slot".to_owned(), node_slot),
                    ("transform_slot".to_owned(), transform_slot),
                    ("mesh_slot".to_owned(), mesh_slot),
                    ("material_slot".to_owned(), material_slot),
                    ("light_slot".to_owned(), light_slot),
                    ("layer_slot".to_owned(), layer_slot),
                ],
            })
        }
        "nova_instance_packet" => {
            let (node_slot, count, stride, phase, material_slot, light_slot) = match args {
                [node_slot, count, stride, phase, material_slot, light_slot] => {
                    (node_slot, count, stride, phase, material_slot, light_slot)
                }
                _ => return Err("nova_instance_packet(...) expects 6 args".to_owned()),
            };
            let node_slot = lower_expr(
                node_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let count = lower_expr(
                count,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let stride = lower_expr(
                stride,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let phase = lower_expr(
                phase,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let material_slot = lower_expr(
                material_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let light_slot = lower_expr(
                light_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaInstancePacket".to_owned(),
                fields: vec![
                    ("node_slot".to_owned(), node_slot),
                    ("count".to_owned(), count),
                    ("stride".to_owned(), stride),
                    ("phase".to_owned(), phase),
                    ("material_slot".to_owned(), material_slot),
                    ("light_slot".to_owned(), light_slot),
                ],
            })
        }
        "nova_scene_graph_packet" => {
            let (root_slot, node_count, link_count, instance_count, active_layer) = match args {
                [root_slot, node_count, link_count, instance_count, active_layer] => (
                    root_slot,
                    node_count,
                    link_count,
                    instance_count,
                    active_layer,
                ),
                _ => return Err("nova_scene_graph_packet(...) expects 5 args".to_owned()),
            };
            let root_slot = lower_expr(
                root_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let node_count = lower_expr(
                node_count,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let link_count = lower_expr(
                link_count,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let instance_count = lower_expr(
                instance_count,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let active_layer = lower_expr(
                active_layer,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaSceneGraphPacket".to_owned(),
                fields: vec![
                    ("root_slot".to_owned(), root_slot),
                    ("node_count".to_owned(), node_count),
                    ("link_count".to_owned(), link_count),
                    ("instance_count".to_owned(), instance_count),
                    ("active_layer".to_owned(), active_layer),
                ],
            })
        }
        "nova_scene_node_packet" => {
            let (node_slot, first_child_slot, sibling_slot, instance_slot, visibility) = match args
            {
                [node_slot, first_child_slot, sibling_slot, instance_slot, visibility] => (
                    node_slot,
                    first_child_slot,
                    sibling_slot,
                    instance_slot,
                    visibility,
                ),
                _ => return Err("nova_scene_node_packet(...) expects 5 args".to_owned()),
            };
            let node_slot = lower_expr(
                node_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let first_child_slot = lower_expr(
                first_child_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let sibling_slot = lower_expr(
                sibling_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let instance_slot = lower_expr(
                instance_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let visibility = lower_expr(
                visibility,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaSceneNodePacket".to_owned(),
                fields: vec![
                    ("node_slot".to_owned(), node_slot),
                    ("first_child_slot".to_owned(), first_child_slot),
                    ("sibling_slot".to_owned(), sibling_slot),
                    ("instance_slot".to_owned(), instance_slot),
                    ("visibility".to_owned(), visibility),
                ],
            })
        }
        "nova_instance_group_packet" => {
            let (root_instance_slot, group_count, visible_count, phase_bias, material_slot) =
                match args {
                    [root_instance_slot, group_count, visible_count, phase_bias, material_slot] => {
                        (
                            root_instance_slot,
                            group_count,
                            visible_count,
                            phase_bias,
                            material_slot,
                        )
                    }
                    _ => return Err("nova_instance_group_packet(...) expects 5 args".to_owned()),
                };
            let root_instance_slot = lower_expr(
                root_instance_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let group_count = lower_expr(
                group_count,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let visible_count = lower_expr(
                visible_count,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let phase_bias = lower_expr(
                phase_bias,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let material_slot = lower_expr(
                material_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaInstanceGroupPacket".to_owned(),
                fields: vec![
                    ("root_instance_slot".to_owned(), root_instance_slot),
                    ("group_count".to_owned(), group_count),
                    ("visible_count".to_owned(), visible_count),
                    ("phase_bias".to_owned(), phase_bias),
                    ("material_slot".to_owned(), material_slot),
                ],
            })
        }
        "nova_scene_cluster_packet" => {
            let (root_node_slot, node_budget, instance_group_slot, material_slot, layer_slot) =
                match args {
                    [root_node_slot, node_budget, instance_group_slot, material_slot, layer_slot] => {
                        (
                            root_node_slot,
                            node_budget,
                            instance_group_slot,
                            material_slot,
                            layer_slot,
                        )
                    }
                    _ => return Err("nova_scene_cluster_packet(...) expects 5 args".to_owned()),
                };
            let root_node_slot = lower_expr(
                root_node_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let node_budget = lower_expr(
                node_budget,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let instance_group_slot = lower_expr(
                instance_group_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let material_slot = lower_expr(
                material_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let layer_slot = lower_expr(
                layer_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaSceneClusterPacket".to_owned(),
                fields: vec![
                    ("root_node_slot".to_owned(), root_node_slot),
                    ("node_budget".to_owned(), node_budget),
                    ("instance_group_slot".to_owned(), instance_group_slot),
                    ("material_slot".to_owned(), material_slot),
                    ("layer_slot".to_owned(), layer_slot),
                ],
            })
        }
        "nova_visibility_packet" => {
            let (cluster_slot, visible_nodes, occlusion_mode, distance_band, mask) = match args {
                [cluster_slot, visible_nodes, occlusion_mode, distance_band, mask] => (
                    cluster_slot,
                    visible_nodes,
                    occlusion_mode,
                    distance_band,
                    mask,
                ),
                _ => return Err("nova_visibility_packet(...) expects 5 args".to_owned()),
            };
            let cluster_slot = lower_expr(
                cluster_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let visible_nodes = lower_expr(
                visible_nodes,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let occlusion_mode = lower_expr(
                occlusion_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let distance_band = lower_expr(
                distance_band,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let mask = lower_expr(
                mask,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaVisibilityPacket".to_owned(),
                fields: vec![
                    ("cluster_slot".to_owned(), cluster_slot),
                    ("visible_nodes".to_owned(), visible_nodes),
                    ("occlusion_mode".to_owned(), occlusion_mode),
                    ("distance_band".to_owned(), distance_band),
                    ("mask".to_owned(), mask),
                ],
            })
        }
        "nova_cull_packet" => {
            let (cluster_slot, kept_nodes, cull_mode, lod_band, mask) = match args {
                [cluster_slot, kept_nodes, cull_mode, lod_band, mask] => {
                    (cluster_slot, kept_nodes, cull_mode, lod_band, mask)
                }
                _ => return Err("nova_cull_packet(...) expects 5 args".to_owned()),
            };
            let cluster_slot = lower_expr(
                cluster_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let kept_nodes = lower_expr(
                kept_nodes,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let cull_mode = lower_expr(
                cull_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let lod_band = lower_expr(
                lod_band,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let mask = lower_expr(
                mask,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaCullPacket".to_owned(),
                fields: vec![
                    ("cluster_slot".to_owned(), cluster_slot),
                    ("kept_nodes".to_owned(), kept_nodes),
                    ("cull_mode".to_owned(), cull_mode),
                    ("lod_band".to_owned(), lod_band),
                    ("mask".to_owned(), mask),
                ],
            })
        }
        "nova_lod_packet" => {
            let (cluster_slot, level_count, active_level, switch_distance, bias) = match args {
                [cluster_slot, level_count, active_level, switch_distance, bias] => (
                    cluster_slot,
                    level_count,
                    active_level,
                    switch_distance,
                    bias,
                ),
                _ => return Err("nova_lod_packet(...) expects 5 args".to_owned()),
            };
            let cluster_slot = lower_expr(
                cluster_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let level_count = lower_expr(
                level_count,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let active_level = lower_expr(
                active_level,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let switch_distance = lower_expr(
                switch_distance,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let bias = lower_expr(
                bias,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaLodPacket".to_owned(),
                fields: vec![
                    ("cluster_slot".to_owned(), cluster_slot),
                    ("level_count".to_owned(), level_count),
                    ("active_level".to_owned(), active_level),
                    ("switch_distance".to_owned(), switch_distance),
                    ("bias".to_owned(), bias),
                ],
            })
        }
        "nova_streaming_packet" => {
            let (cluster_slot, resident_levels, prefetch_mode, evict_budget, channel) = match args {
                [cluster_slot, resident_levels, prefetch_mode, evict_budget, channel] => (
                    cluster_slot,
                    resident_levels,
                    prefetch_mode,
                    evict_budget,
                    channel,
                ),
                _ => return Err("nova_streaming_packet(...) expects 5 args".to_owned()),
            };
            let cluster_slot = lower_expr(
                cluster_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let resident_levels = lower_expr(
                resident_levels,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let prefetch_mode = lower_expr(
                prefetch_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let evict_budget = lower_expr(
                evict_budget,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let channel = lower_expr(
                channel,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaStreamingPacket".to_owned(),
                fields: vec![
                    ("cluster_slot".to_owned(), cluster_slot),
                    ("resident_levels".to_owned(), resident_levels),
                    ("prefetch_mode".to_owned(), prefetch_mode),
                    ("evict_budget".to_owned(), evict_budget),
                    ("channel".to_owned(), channel),
                ],
            })
        }
        "nova_residency_packet" => {
            let (cluster_slot, committed_levels, residency_mode, spill_budget, residency_mask) =
                match args {
                    [cluster_slot, committed_levels, residency_mode, spill_budget, residency_mask] => {
                        (
                            cluster_slot,
                            committed_levels,
                            residency_mode,
                            spill_budget,
                            residency_mask,
                        )
                    }
                    _ => return Err("nova_residency_packet(...) expects 5 args".to_owned()),
                };
            let cluster_slot = lower_expr(
                cluster_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let committed_levels = lower_expr(
                committed_levels,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let residency_mode = lower_expr(
                residency_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let spill_budget = lower_expr(
                spill_budget,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let residency_mask = lower_expr(
                residency_mask,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaResidencyPacket".to_owned(),
                fields: vec![
                    ("cluster_slot".to_owned(), cluster_slot),
                    ("committed_levels".to_owned(), committed_levels),
                    ("residency_mode".to_owned(), residency_mode),
                    ("spill_budget".to_owned(), spill_budget),
                    ("residency_mask".to_owned(), residency_mask),
                ],
            })
        }
        "nova_eviction_packet" => {
            let (cluster_slot, evicted_levels, eviction_mode, reclaim_budget, eviction_mask) =
                match args {
                    [cluster_slot, evicted_levels, eviction_mode, reclaim_budget, eviction_mask] => {
                        (
                            cluster_slot,
                            evicted_levels,
                            eviction_mode,
                            reclaim_budget,
                            eviction_mask,
                        )
                    }
                    _ => return Err("nova_eviction_packet(...) expects 5 args".to_owned()),
                };
            let cluster_slot = lower_expr(
                cluster_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let evicted_levels = lower_expr(
                evicted_levels,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let eviction_mode = lower_expr(
                eviction_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let reclaim_budget = lower_expr(
                reclaim_budget,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let eviction_mask = lower_expr(
                eviction_mask,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaEvictionPacket".to_owned(),
                fields: vec![
                    ("cluster_slot".to_owned(), cluster_slot),
                    ("evicted_levels".to_owned(), evicted_levels),
                    ("eviction_mode".to_owned(), eviction_mode),
                    ("reclaim_budget".to_owned(), reclaim_budget),
                    ("eviction_mask".to_owned(), eviction_mask),
                ],
            })
        }
        "nova_prefetch_packet" => {
            let (cluster_slot, requested_levels, prefetch_window, warm_budget, prefetch_mask) =
                match args {
                    [cluster_slot, requested_levels, prefetch_window, warm_budget, prefetch_mask] => {
                        (
                            cluster_slot,
                            requested_levels,
                            prefetch_window,
                            warm_budget,
                            prefetch_mask,
                        )
                    }
                    _ => return Err("nova_prefetch_packet(...) expects 5 args".to_owned()),
                };
            let cluster_slot = lower_expr(
                cluster_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let requested_levels = lower_expr(
                requested_levels,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let prefetch_window = lower_expr(
                prefetch_window,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let warm_budget = lower_expr(
                warm_budget,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let prefetch_mask = lower_expr(
                prefetch_mask,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaPrefetchPacket".to_owned(),
                fields: vec![
                    ("cluster_slot".to_owned(), cluster_slot),
                    ("requested_levels".to_owned(), requested_levels),
                    ("prefetch_window".to_owned(), prefetch_window),
                    ("warm_budget".to_owned(), warm_budget),
                    ("prefetch_mask".to_owned(), prefetch_mask),
                ],
            })
        }
        "nova_budget_packet" => {
            let (cluster_slot, total_budget, used_budget, headroom, budget_policy) = match args {
                [cluster_slot, total_budget, used_budget, headroom, budget_policy] => (
                    cluster_slot,
                    total_budget,
                    used_budget,
                    headroom,
                    budget_policy,
                ),
                _ => return Err("nova_budget_packet(...) expects 5 args".to_owned()),
            };
            let cluster_slot = lower_expr(
                cluster_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let total_budget = lower_expr(
                total_budget,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let used_budget = lower_expr(
                used_budget,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let headroom = lower_expr(
                headroom,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let budget_policy = lower_expr(
                budget_policy,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaBudgetPacket".to_owned(),
                fields: vec![
                    ("cluster_slot".to_owned(), cluster_slot),
                    ("total_budget".to_owned(), total_budget),
                    ("used_budget".to_owned(), used_budget),
                    ("headroom".to_owned(), headroom),
                    ("budget_policy".to_owned(), budget_policy),
                ],
            })
        }
        "nova_pressure_packet" => {
            let (cluster_slot, pressure_level, saturation, throttled, pressure_mask) = match args {
                [cluster_slot, pressure_level, saturation, throttled, pressure_mask] => (
                    cluster_slot,
                    pressure_level,
                    saturation,
                    throttled,
                    pressure_mask,
                ),
                _ => return Err("nova_pressure_packet(...) expects 5 args".to_owned()),
            };
            let cluster_slot = lower_expr(
                cluster_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let pressure_level = lower_expr(
                pressure_level,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let saturation = lower_expr(
                saturation,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let throttled = lower_expr(
                throttled,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let pressure_mask = lower_expr(
                pressure_mask,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaPressurePacket".to_owned(),
                fields: vec![
                    ("cluster_slot".to_owned(), cluster_slot),
                    ("pressure_level".to_owned(), pressure_level),
                    ("saturation".to_owned(), saturation),
                    ("throttled".to_owned(), throttled),
                    ("pressure_mask".to_owned(), pressure_mask),
                ],
            })
        }
        "nova_thermal_packet" => {
            let (cluster_slot, thermal_level, cooling_mode, throttled, thermal_mask) = match args {
                [cluster_slot, thermal_level, cooling_mode, throttled, thermal_mask] => (
                    cluster_slot,
                    thermal_level,
                    cooling_mode,
                    throttled,
                    thermal_mask,
                ),
                _ => return Err("nova_thermal_packet(...) expects 5 args".to_owned()),
            };
            let cluster_slot = lower_expr(
                cluster_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let thermal_level = lower_expr(
                thermal_level,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let cooling_mode = lower_expr(
                cooling_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let throttled = lower_expr(
                throttled,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let thermal_mask = lower_expr(
                thermal_mask,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaThermalPacket".to_owned(),
                fields: vec![
                    ("cluster_slot".to_owned(), cluster_slot),
                    ("thermal_level".to_owned(), thermal_level),
                    ("cooling_mode".to_owned(), cooling_mode),
                    ("throttled".to_owned(), throttled),
                    ("thermal_mask".to_owned(), thermal_mask),
                ],
            })
        }
        "nova_power_packet" => {
            let (cluster_slot, power_level, source_mode, capped, power_mask) = match args {
                [cluster_slot, power_level, source_mode, capped, power_mask] => {
                    (cluster_slot, power_level, source_mode, capped, power_mask)
                }
                _ => return Err("nova_power_packet(...) expects 5 args".to_owned()),
            };
            let cluster_slot = lower_expr(
                cluster_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let power_level = lower_expr(
                power_level,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let source_mode = lower_expr(
                source_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let capped = lower_expr(
                capped,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let power_mask = lower_expr(
                power_mask,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaPowerPacket".to_owned(),
                fields: vec![
                    ("cluster_slot".to_owned(), cluster_slot),
                    ("power_level".to_owned(), power_level),
                    ("source_mode".to_owned(), source_mode),
                    ("capped".to_owned(), capped),
                    ("power_mask".to_owned(), power_mask),
                ],
            })
        }
        "nova_latency_packet" => {
            let (cluster_slot, frame_latency, input_latency, jitter, latency_mask) = match args {
                [cluster_slot, frame_latency, input_latency, jitter, latency_mask] => (
                    cluster_slot,
                    frame_latency,
                    input_latency,
                    jitter,
                    latency_mask,
                ),
                _ => return Err("nova_latency_packet(...) expects 5 args".to_owned()),
            };
            let cluster_slot = lower_expr(
                cluster_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let frame_latency = lower_expr(
                frame_latency,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let input_latency = lower_expr(
                input_latency,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let jitter = lower_expr(
                jitter,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let latency_mask = lower_expr(
                latency_mask,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaLatencyPacket".to_owned(),
                fields: vec![
                    ("cluster_slot".to_owned(), cluster_slot),
                    ("frame_latency".to_owned(), frame_latency),
                    ("input_latency".to_owned(), input_latency),
                    ("jitter".to_owned(), jitter),
                    ("latency_mask".to_owned(), latency_mask),
                ],
            })
        }
        "nova_frame_pacing_packet" => {
            let (cluster_slot, cadence, variance, vsync_mode, pacing_mask) = match args {
                [cluster_slot, cadence, variance, vsync_mode, pacing_mask] => {
                    (cluster_slot, cadence, variance, vsync_mode, pacing_mask)
                }
                _ => return Err("nova_frame_pacing_packet(...) expects 5 args".to_owned()),
            };
            let cluster_slot = lower_expr(
                cluster_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let cadence = lower_expr(
                cadence,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let variance = lower_expr(
                variance,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let vsync_mode = lower_expr(
                vsync_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let pacing_mask = lower_expr(
                pacing_mask,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaFramePacingPacket".to_owned(),
                fields: vec![
                    ("cluster_slot".to_owned(), cluster_slot),
                    ("cadence".to_owned(), cadence),
                    ("variance".to_owned(), variance),
                    ("vsync_mode".to_owned(), vsync_mode),
                    ("pacing_mask".to_owned(), pacing_mask),
                ],
            })
        }
        "nova_jank_packet" => {
            let (cluster_slot, spikes, severity, recovery, jank_mask) = match args {
                [cluster_slot, spikes, severity, recovery, jank_mask] => {
                    (cluster_slot, spikes, severity, recovery, jank_mask)
                }
                _ => return Err("nova_jank_packet(...) expects 5 args".to_owned()),
            };
            let cluster_slot = lower_expr(
                cluster_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let spikes = lower_expr(
                spikes,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let severity = lower_expr(
                severity,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let recovery = lower_expr(
                recovery,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let jank_mask = lower_expr(
                jank_mask,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaJankPacket".to_owned(),
                fields: vec![
                    ("cluster_slot".to_owned(), cluster_slot),
                    ("spikes".to_owned(), spikes),
                    ("severity".to_owned(), severity),
                    ("recovery".to_owned(), recovery),
                    ("jank_mask".to_owned(), jank_mask),
                ],
            })
        }
        "nova_frame_variance_packet" => {
            let (cluster_slot, frame_variance, input_variance, burst_mode, variance_mask) =
                match args {
                    [cluster_slot, frame_variance, input_variance, burst_mode, variance_mask] => (
                        cluster_slot,
                        frame_variance,
                        input_variance,
                        burst_mode,
                        variance_mask,
                    ),
                    _ => {
                        return Err("nova_frame_variance_packet(...) expects 5 args".to_owned());
                    }
                };
            let cluster_slot = lower_expr(
                cluster_slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let frame_variance = lower_expr(
                frame_variance,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let input_variance = lower_expr(
                input_variance,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let burst_mode = lower_expr(
                burst_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let variance_mask = lower_expr(
                variance_mask,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaFrameVariancePacket".to_owned(),
                fields: vec![
                    ("cluster_slot".to_owned(), cluster_slot),
                    ("frame_variance".to_owned(), frame_variance),
                    ("input_variance".to_owned(), input_variance),
                    ("burst_mode".to_owned(), burst_mode),
                    ("variance_mask".to_owned(), variance_mask),
                ],
            })
        }
        "nova_pass_packet" => {
            let (stage, clear_mode, sample_count, debug_view) = match args {
                [stage, clear_mode, sample_count, debug_view] => {
                    (stage, clear_mode, sample_count, debug_view)
                }
                _ => return Err("nova_pass_packet(...) expects 4 args".to_owned()),
            };
            let stage = lower_expr(
                stage,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let clear_mode = lower_expr(
                clear_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let sample_count = lower_expr(
                sample_count,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let debug_view = lower_expr(
                debug_view,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaPassPacket".to_owned(),
                fields: vec![
                    ("stage".to_owned(), stage),
                    ("clear_mode".to_owned(), clear_mode),
                    ("sample_count".to_owned(), sample_count),
                    ("debug_view".to_owned(), debug_view),
                ],
            })
        }
        "nova_frame_packet" => {
            let (frame_index, present_mode, sync_interval, exposure) = match args {
                [frame_index, present_mode, sync_interval, exposure] => {
                    (frame_index, present_mode, sync_interval, exposure)
                }
                _ => return Err("nova_frame_packet(...) expects 4 args".to_owned()),
            };
            let frame_index = lower_expr(
                frame_index,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let present_mode = lower_expr(
                present_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let sync_interval = lower_expr(
                sync_interval,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let exposure = lower_expr(
                exposure,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaFramePacket".to_owned(),
                fields: vec![
                    ("frame_index".to_owned(), frame_index),
                    ("present_mode".to_owned(), present_mode),
                    ("sync_interval".to_owned(), sync_interval),
                    ("exposure".to_owned(), exposure),
                ],
            })
        }
        "nova_target_packet" => {
            let (kind, width, height, multisample) = match args {
                [kind, width, height, multisample] => (kind, width, height, multisample),
                _ => return Err("nova_target_packet(...) expects 4 args".to_owned()),
            };
            let kind = lower_expr(
                kind,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let width = lower_expr(
                width,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let height = lower_expr(
                height,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let multisample = lower_expr(
                multisample,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaTargetPacket".to_owned(),
                fields: vec![
                    ("kind".to_owned(), kind),
                    ("width".to_owned(), width),
                    ("height".to_owned(), height),
                    ("multisample".to_owned(), multisample),
                ],
            })
        }
        "nova_frame_graph_packet" => {
            let (passes, targets, present_stage, debug_overlay) = match args {
                [passes, targets, present_stage, debug_overlay] => {
                    (passes, targets, present_stage, debug_overlay)
                }
                _ => return Err("nova_frame_graph_packet(...) expects 4 args".to_owned()),
            };
            let passes = lower_expr(
                passes,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let targets = lower_expr(
                targets,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let present_stage = lower_expr(
                present_stage,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let debug_overlay = lower_expr(
                debug_overlay,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaFrameGraphPacket".to_owned(),
                fields: vec![
                    ("passes".to_owned(), passes),
                    ("targets".to_owned(), targets),
                    ("present_stage".to_owned(), present_stage),
                    ("debug_overlay".to_owned(), debug_overlay),
                ],
            })
        }
        "nova_attachment_packet" => {
            let (slot, format_kind, load_op, store_op) = match args {
                [slot, format_kind, load_op, store_op] => (slot, format_kind, load_op, store_op),
                _ => return Err("nova_attachment_packet(...) expects 4 args".to_owned()),
            };
            let slot = lower_expr(
                slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let format_kind = lower_expr(
                format_kind,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let load_op = lower_expr(
                load_op,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let store_op = lower_expr(
                store_op,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaAttachmentPacket".to_owned(),
                fields: vec![
                    ("slot".to_owned(), slot),
                    ("format_kind".to_owned(), format_kind),
                    ("load_op".to_owned(), load_op),
                    ("store_op".to_owned(), store_op),
                ],
            })
        }
        "nova_pass_chain_packet" => {
            let (stages, fanout, resolve_stage, barrier_mode) = match args {
                [stages, fanout, resolve_stage, barrier_mode] => {
                    (stages, fanout, resolve_stage, barrier_mode)
                }
                _ => return Err("nova_pass_chain_packet(...) expects 4 args".to_owned()),
            };
            let stages = lower_expr(
                stages,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let fanout = lower_expr(
                fanout,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let resolve_stage = lower_expr(
                resolve_stage,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let barrier_mode = lower_expr(
                barrier_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaPassChainPacket".to_owned(),
                fields: vec![
                    ("stages".to_owned(), stages),
                    ("fanout".to_owned(), fanout),
                    ("resolve_stage".to_owned(), resolve_stage),
                    ("barrier_mode".to_owned(), barrier_mode),
                ],
            })
        }
        "nova_barrier_packet" => {
            let (scope, source_stage, target_stage, flush_mode) = match args {
                [scope, source_stage, target_stage, flush_mode] => {
                    (scope, source_stage, target_stage, flush_mode)
                }
                _ => return Err("nova_barrier_packet(...) expects 4 args".to_owned()),
            };
            let scope = lower_expr(
                scope,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let source_stage = lower_expr(
                source_stage,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let target_stage = lower_expr(
                target_stage,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let flush_mode = lower_expr(
                flush_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaBarrierPacket".to_owned(),
                fields: vec![
                    ("scope".to_owned(), scope),
                    ("source_stage".to_owned(), source_stage),
                    ("target_stage".to_owned(), target_stage),
                    ("flush_mode".to_owned(), flush_mode),
                ],
            })
        }
        "nova_resource_set_packet" => {
            let (buffers, textures, samplers, residency) = match args {
                [buffers, textures, samplers, residency] => {
                    (buffers, textures, samplers, residency)
                }
                _ => return Err("nova_resource_set_packet(...) expects 4 args".to_owned()),
            };
            let buffers = lower_expr(
                buffers,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let textures = lower_expr(
                textures,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let samplers = lower_expr(
                samplers,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let residency = lower_expr(
                residency,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaResourceSetPacket".to_owned(),
                fields: vec![
                    ("buffers".to_owned(), buffers),
                    ("textures".to_owned(), textures),
                    ("samplers".to_owned(), samplers),
                    ("residency".to_owned(), residency),
                ],
            })
        }
        "nova_schedule_packet" => {
            let (lanes, queue_depth, async_budget, tick_mode) = match args {
                [lanes, queue_depth, async_budget, tick_mode] => {
                    (lanes, queue_depth, async_budget, tick_mode)
                }
                _ => return Err("nova_schedule_packet(...) expects 4 args".to_owned()),
            };
            let lanes = lower_expr(
                lanes,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let queue_depth = lower_expr(
                queue_depth,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let async_budget = lower_expr(
                async_budget,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let tick_mode = lower_expr(
                tick_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaSchedulePacket".to_owned(),
                fields: vec![
                    ("lanes".to_owned(), lanes),
                    ("queue_depth".to_owned(), queue_depth),
                    ("async_budget".to_owned(), async_budget),
                    ("tick_mode".to_owned(), tick_mode),
                ],
            })
        }
        "nova_submission_packet" => {
            let (batches, fences, signal_mode, present_hint) = match args {
                [batches, fences, signal_mode, present_hint] => {
                    (batches, fences, signal_mode, present_hint)
                }
                _ => return Err("nova_submission_packet(...) expects 4 args".to_owned()),
            };
            let batches = lower_expr(
                batches,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let fences = lower_expr(
                fences,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let signal_mode = lower_expr(
                signal_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let present_hint = lower_expr(
                present_hint,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaSubmissionPacket".to_owned(),
                fields: vec![
                    ("batches".to_owned(), batches),
                    ("fences".to_owned(), fences),
                    ("signal_mode".to_owned(), signal_mode),
                    ("present_hint".to_owned(), present_hint),
                ],
            })
        }
        "nova_queue_packet" => {
            let (kind, priority, budget, ownership) = match args {
                [kind, priority, budget, ownership] => (kind, priority, budget, ownership),
                _ => return Err("nova_queue_packet(...) expects 4 args".to_owned()),
            };
            let kind = lower_expr(
                kind,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let priority = lower_expr(
                priority,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let budget = lower_expr(
                budget,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let ownership = lower_expr(
                ownership,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaQueuePacket".to_owned(),
                fields: vec![
                    ("kind".to_owned(), kind),
                    ("priority".to_owned(), priority),
                    ("budget".to_owned(), budget),
                    ("ownership".to_owned(), ownership),
                ],
            })
        }
        "nova_semaphore_packet" => {
            let (wait_count, signal_count, timeline_mode, scope) = match args {
                [wait_count, signal_count, timeline_mode, scope] => {
                    (wait_count, signal_count, timeline_mode, scope)
                }
                _ => return Err("nova_semaphore_packet(...) expects 4 args".to_owned()),
            };
            let wait_count = lower_expr(
                wait_count,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let signal_count = lower_expr(
                signal_count,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let timeline_mode = lower_expr(
                timeline_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let scope = lower_expr(
                scope,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaSemaphorePacket".to_owned(),
                fields: vec![
                    ("wait_count".to_owned(), wait_count),
                    ("signal_count".to_owned(), signal_count),
                    ("timeline_mode".to_owned(), timeline_mode),
                    ("scope".to_owned(), scope),
                ],
            })
        }
        "nova_timeline_packet" => {
            let (value, step, epoch, domain) = match args {
                [value, step, epoch, domain] => (value, step, epoch, domain),
                _ => return Err("nova_timeline_packet(...) expects 4 args".to_owned()),
            };
            let value = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let step = lower_expr(
                step,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let epoch = lower_expr(
                epoch,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let domain = lower_expr(
                domain,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaTimelinePacket".to_owned(),
                fields: vec![
                    ("value".to_owned(), value),
                    ("step".to_owned(), step),
                    ("epoch".to_owned(), epoch),
                    ("domain".to_owned(), domain),
                ],
            })
        }
        "nova_fence_packet" => {
            let (signaled, epoch, scope, recycle_mode) = match args {
                [signaled, epoch, scope, recycle_mode] => (signaled, epoch, scope, recycle_mode),
                _ => return Err("nova_fence_packet(...) expects 4 args".to_owned()),
            };
            let signaled = lower_expr(
                signaled,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let epoch = lower_expr(
                epoch,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let scope = lower_expr(
                scope,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let recycle_mode = lower_expr(
                recycle_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaFencePacket".to_owned(),
                fields: vec![
                    ("signaled".to_owned(), signaled),
                    ("epoch".to_owned(), epoch),
                    ("scope".to_owned(), scope),
                    ("recycle_mode".to_owned(), recycle_mode),
                ],
            })
        }
        "nova_signal_packet" => {
            let (kind, phase, fanout, ack_mode) = match args {
                [kind, phase, fanout, ack_mode] => (kind, phase, fanout, ack_mode),
                _ => return Err("nova_signal_packet(...) expects 4 args".to_owned()),
            };
            let kind = lower_expr(
                kind,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let phase = lower_expr(
                phase,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let fanout = lower_expr(
                fanout,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let ack_mode = lower_expr(
                ack_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaSignalPacket".to_owned(),
                fields: vec![
                    ("kind".to_owned(), kind),
                    ("phase".to_owned(), phase),
                    ("fanout".to_owned(), fanout),
                    ("ack_mode".to_owned(), ack_mode),
                ],
            })
        }
        "nova_event_packet" => {
            let (kind, route, priority, payload_mode) = match args {
                [kind, route, priority, payload_mode] => (kind, route, priority, payload_mode),
                _ => return Err("nova_event_packet(...) expects 4 args".to_owned()),
            };
            let kind = lower_expr(
                kind,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let route = lower_expr(
                route,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let priority = lower_expr(
                priority,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let payload_mode = lower_expr(
                payload_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaEventPacket".to_owned(),
                fields: vec![
                    ("kind".to_owned(), kind),
                    ("route".to_owned(), route),
                    ("priority".to_owned(), priority),
                    ("payload_mode".to_owned(), payload_mode),
                ],
            })
        }
        "nova_dispatch_packet" => {
            let (queue_kind, lane, batch, completion_mode) = match args {
                [queue_kind, lane, batch, completion_mode] => {
                    (queue_kind, lane, batch, completion_mode)
                }
                _ => return Err("nova_dispatch_packet(...) expects 4 args".to_owned()),
            };
            let queue_kind = lower_expr(
                queue_kind,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let lane = lower_expr(
                lane,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let batch = lower_expr(
                batch,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let completion_mode = lower_expr(
                completion_mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaDispatchPacket".to_owned(),
                fields: vec![
                    ("queue_kind".to_owned(), queue_kind),
                    ("lane".to_owned(), lane),
                    ("batch".to_owned(), batch),
                    ("completion_mode".to_owned(), completion_mode),
                ],
            })
        }
        "nova_panel_from_parts" => {
            let [header, sliders, toggle, progress, meter, button, text_input, select, checkbox, radio, textarea, tabs, list, table, tree, inspector, outline, theme, surface, viewport, layer, scene, camera, material, light, mesh, transform, node, scene_link, instance, scene_graph, scene_node, instance_group, scene_cluster, visibility, cull, lod, streaming, residency, eviction, prefetch, budget, pressure, thermal, power, latency, frame_pacing, frame_variance, jank, pass, frame, target, frame_graph, attachment, pass_chain, barrier, resource_set, schedule, submission, queue, semaphore, timeline, fence, signal, event, dispatch, feedback, intent, reaction, outcome, resolution, commit, snapshot, checkpoint, focus] =
                args
            else {
                return Err("nova_panel_from_parts(...) expects 75 args".to_owned());
            };
            let header = lower_expr(
                header,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaHeaderPacket")),
            )?;
            let sliders = lower_expr(
                sliders,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSliderGroupPacket")),
            )?;
            let toggle = lower_expr(
                toggle,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTogglePacket")),
            )?;
            let progress = lower_expr(
                progress,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaProgressPacket")),
            )?;
            let meter = lower_expr(
                meter,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaMeterPacket")),
            )?;
            let button = lower_expr(
                button,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaButtonPacket")),
            )?;
            let text_input = lower_expr(
                text_input,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTextInputPacket")),
            )?;
            let select = lower_expr(
                select,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSelectPacket")),
            )?;
            let checkbox = lower_expr(
                checkbox,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaCheckboxPacket")),
            )?;
            let radio = lower_expr(
                radio,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaRadioPacket")),
            )?;
            let textarea = lower_expr(
                textarea,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTextAreaPacket")),
            )?;
            let tabs = lower_expr(
                tabs,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTabsPacket")),
            )?;
            let list = lower_expr(
                list,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaListPacket")),
            )?;
            let table = lower_expr(
                table,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTablePacket")),
            )?;
            let tree = lower_expr(
                tree,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTreePacket")),
            )?;
            let inspector = lower_expr(
                inspector,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaInspectorPacket")),
            )?;
            let outline = lower_expr(
                outline,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaOutlinePacket")),
            )?;
            let theme = lower_expr(
                theme,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaThemePacket")),
            )?;
            let surface = lower_expr(
                surface,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSurfacePacket")),
            )?;
            let viewport = lower_expr(
                viewport,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaViewportPacket")),
            )?;
            let layer = lower_expr(
                layer,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaLayerPacket")),
            )?;
            let scene = lower_expr(
                scene,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaScenePacket")),
            )?;
            let camera = lower_expr(
                camera,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaCameraPacket")),
            )?;
            let material = lower_expr(
                material,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaMaterialPacket")),
            )?;
            let light = lower_expr(
                light,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaLightPacket")),
            )?;
            let mesh = lower_expr(
                mesh,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaMeshPacket")),
            )?;
            let transform = lower_expr(
                transform,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTransformPacket")),
            )?;
            let node = lower_expr(
                node,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaNodePacket")),
            )?;
            let scene_link = lower_expr(
                scene_link,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSceneLinkPacket")),
            )?;
            let instance = lower_expr(
                instance,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaInstancePacket")),
            )?;
            let scene_graph = lower_expr(
                scene_graph,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSceneGraphPacket")),
            )?;
            let scene_node = lower_expr(
                scene_node,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSceneNodePacket")),
            )?;
            let instance_group = lower_expr(
                instance_group,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaInstanceGroupPacket")),
            )?;
            let scene_cluster = lower_expr(
                scene_cluster,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSceneClusterPacket")),
            )?;
            let visibility = lower_expr(
                visibility,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaVisibilityPacket")),
            )?;
            let cull = lower_expr(
                cull,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaCullPacket")),
            )?;
            let lod = lower_expr(
                lod,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaLodPacket")),
            )?;
            let streaming = lower_expr(
                streaming,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaStreamingPacket")),
            )?;
            let residency = lower_expr(
                residency,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaResidencyPacket")),
            )?;
            let eviction = lower_expr(
                eviction,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaEvictionPacket")),
            )?;
            let prefetch = lower_expr(
                prefetch,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaPrefetchPacket")),
            )?;
            let budget = lower_expr(
                budget,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaBudgetPacket")),
            )?;
            let pressure = lower_expr(
                pressure,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaPressurePacket")),
            )?;
            let thermal = lower_expr(
                thermal,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaThermalPacket")),
            )?;
            let power = lower_expr(
                power,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaPowerPacket")),
            )?;
            let latency = lower_expr(
                latency,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaLatencyPacket")),
            )?;
            let frame_pacing = lower_expr(
                frame_pacing,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaFramePacingPacket")),
            )?;
            let frame_variance = lower_expr(
                frame_variance,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaFrameVariancePacket")),
            )?;
            let jank = lower_expr(
                jank,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaJankPacket")),
            )?;
            let pass = lower_expr(
                pass,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaPassPacket")),
            )?;
            let frame = lower_expr(
                frame,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaFramePacket")),
            )?;
            let target = lower_expr(
                target,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTargetPacket")),
            )?;
            let frame_graph = lower_expr(
                frame_graph,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaFrameGraphPacket")),
            )?;
            let attachment = lower_expr(
                attachment,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaAttachmentPacket")),
            )?;
            let pass_chain = lower_expr(
                pass_chain,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaPassChainPacket")),
            )?;
            let barrier = lower_expr(
                barrier,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaBarrierPacket")),
            )?;
            let resource_set = lower_expr(
                resource_set,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaResourceSetPacket")),
            )?;
            let schedule = lower_expr(
                schedule,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSchedulePacket")),
            )?;
            let submission = lower_expr(
                submission,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSubmissionPacket")),
            )?;
            let queue = lower_expr(
                queue,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaQueuePacket")),
            )?;
            let semaphore = lower_expr(
                semaphore,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSemaphorePacket")),
            )?;
            let timeline = lower_expr(
                timeline,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTimelinePacket")),
            )?;
            let fence = lower_expr(
                fence,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaFencePacket")),
            )?;
            let signal = lower_expr(
                signal,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSignalPacket")),
            )?;
            let event = lower_expr(
                event,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaEventPacket")),
            )?;
            let dispatch = lower_expr(
                dispatch,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaDispatchPacket")),
            )?;
            let feedback = lower_expr(
                feedback,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaFeedbackPacket")),
            )?;
            let intent = lower_expr(
                intent,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaIntentPacket")),
            )?;
            let reaction = lower_expr(
                reaction,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaReactionPacket")),
            )?;
            let outcome = lower_expr(
                outcome,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaOutcomePacket")),
            )?;
            let resolution = lower_expr(
                resolution,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaResolutionPacket")),
            )?;
            let commit = lower_expr(
                commit,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaCommitPacket")),
            )?;
            let snapshot = lower_expr(
                snapshot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSnapshotPacket")),
            )?;
            let checkpoint = lower_expr(
                checkpoint,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaCheckpointPacket")),
            )?;
            let focus = lower_expr(
                focus,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaFocusPacket")),
            )?;
            Ok(NirExpr::StructLiteral {
                type_name: "NovaPanelPacket".to_owned(),
                fields: vec![
                    ("header".to_owned(), header),
                    ("sliders".to_owned(), sliders),
                    ("toggle".to_owned(), toggle),
                    ("progress".to_owned(), progress),
                    ("meter".to_owned(), meter),
                    ("button".to_owned(), button),
                    ("text_input".to_owned(), text_input),
                    ("select".to_owned(), select),
                    ("checkbox".to_owned(), checkbox),
                    ("radio".to_owned(), radio),
                    ("textarea".to_owned(), textarea),
                    ("tabs".to_owned(), tabs),
                    ("list".to_owned(), list),
                    ("table".to_owned(), table),
                    ("tree".to_owned(), tree),
                    ("inspector".to_owned(), inspector),
                    ("outline".to_owned(), outline),
                    ("theme".to_owned(), theme),
                    ("surface".to_owned(), surface),
                    ("viewport".to_owned(), viewport),
                    ("layer".to_owned(), layer),
                    ("scene".to_owned(), scene),
                    ("camera".to_owned(), camera),
                    ("material".to_owned(), material),
                    ("light".to_owned(), light),
                    ("mesh".to_owned(), mesh),
                    ("transform".to_owned(), transform),
                    ("node".to_owned(), node),
                    ("scene_link".to_owned(), scene_link),
                    ("instance".to_owned(), instance),
                    ("scene_graph".to_owned(), scene_graph),
                    ("scene_node".to_owned(), scene_node),
                    ("instance_group".to_owned(), instance_group),
                    ("scene_cluster".to_owned(), scene_cluster),
                    ("scene_visibility".to_owned(), visibility),
                    ("scene_cull".to_owned(), cull),
                    ("scene_lod".to_owned(), lod),
                    ("scene_streaming".to_owned(), streaming),
                    ("scene_residency".to_owned(), residency),
                    ("scene_eviction".to_owned(), eviction),
                    ("scene_prefetch".to_owned(), prefetch),
                    ("scene_budget".to_owned(), budget),
                    ("scene_pressure".to_owned(), pressure),
                    ("scene_thermal".to_owned(), thermal),
                    ("scene_power".to_owned(), power),
                    ("scene_latency".to_owned(), latency),
                    ("scene_frame_pacing".to_owned(), frame_pacing),
                    ("scene_frame_variance".to_owned(), frame_variance),
                    ("scene_jank".to_owned(), jank),
                    ("pass".to_owned(), pass),
                    ("frame".to_owned(), frame),
                    ("target".to_owned(), target),
                    ("frame_graph".to_owned(), frame_graph),
                    ("attachment".to_owned(), attachment),
                    ("pass_chain".to_owned(), pass_chain),
                    ("barrier".to_owned(), barrier),
                    ("resource_set".to_owned(), resource_set),
                    ("schedule".to_owned(), schedule),
                    ("submission".to_owned(), submission),
                    ("queue".to_owned(), queue),
                    ("semaphore".to_owned(), semaphore),
                    ("timeline".to_owned(), timeline),
                    ("fence".to_owned(), fence),
                    ("signal".to_owned(), signal),
                    ("event".to_owned(), event),
                    ("dispatch".to_owned(), dispatch),
                    ("feedback".to_owned(), feedback),
                    ("intent".to_owned(), intent),
                    ("reaction".to_owned(), reaction),
                    ("outcome".to_owned(), outcome),
                    ("resolution".to_owned(), resolution),
                    ("commit".to_owned(), commit),
                    ("snapshot".to_owned(), snapshot),
                    ("checkpoint".to_owned(), checkpoint),
                    ("focus".to_owned(), focus),
                ],
            })
        }
        "nova_slider_disabled"
        | "nova_toggle_disabled"
        | "nova_text_input_dirty"
        | "nova_text_input_read_only"
        | "nova_select_committed"
        | "nova_select_multiple"
        | "nova_checkbox_checked"
        | "nova_checkbox_disabled"
        | "nova_radio_disabled"
        | "nova_textarea_dirty"
        | "nova_textarea_read_only"
        | "nova_tabs_compact"
        | "nova_list_dense"
        | "nova_table_zebra"
        | "nova_tree_expanded"
        | "nova_inspector_pinned"
        | "nova_outline_collapsed"
        | "nova_selection_selected"
        | "nova_selection_mode" => {
            let [packet] = args else {
                return Err(format!("{callee}(...) expects 1 arg"));
            };
            let (expected_type, field_name) = match callee {
                "nova_slider_disabled" => ("NovaSliderPacket", "disabled"),
                "nova_toggle_disabled" => ("NovaTogglePacket", "disabled"),
                "nova_text_input_dirty" => ("NovaTextInputPacket", "dirty"),
                "nova_text_input_read_only" => ("NovaTextInputPacket", "read_only"),
                "nova_select_committed" => ("NovaSelectPacket", "committed"),
                "nova_select_multiple" => ("NovaSelectPacket", "multiple"),
                "nova_checkbox_checked" => ("NovaCheckboxPacket", "checked"),
                "nova_checkbox_disabled" => ("NovaCheckboxPacket", "disabled"),
                "nova_radio_disabled" => ("NovaRadioPacket", "disabled"),
                "nova_textarea_dirty" => ("NovaTextAreaPacket", "dirty"),
                "nova_textarea_read_only" => ("NovaTextAreaPacket", "read_only"),
                "nova_tabs_compact" => ("NovaTabsPacket", "compact"),
                "nova_list_dense" => ("NovaListPacket", "dense"),
                "nova_table_zebra" => ("NovaTablePacket", "zebra"),
                "nova_tree_expanded" => ("NovaTreePacket", "expanded"),
                "nova_inspector_pinned" => ("NovaInspectorPacket", "pinned"),
                "nova_outline_collapsed" => ("NovaOutlinePacket", "collapsed"),
                "nova_selection_selected" => ("NovaSelectionPacket", "selected"),
                _ => ("NovaSelectionPacket", "mode"),
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type(expected_type)),
            )?;
            Ok(NirExpr::FieldAccess {
                base: Box::new(packet),
                field: field_name.to_owned(),
            })
        }
        _ => lower_direct_call_builtin_or_named_call(
            callee,
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            allow_async_calls,
        )?
        .ok_or_else(|| format!("unknown function `{callee}`")),
    }
}

fn validate_declared_nir_types(module: &NirModule) -> Result<(), String> {
    let struct_table = module
        .structs
        .iter()
        .map(|definition| (definition.name.clone(), definition.clone()))
        .collect::<BTreeMap<_, _>>();
    for function in &module.externs {
        for param in &function.params {
            validate_type_ref(&param.ty)?;
        }
        validate_type_ref(&function.return_type)?;
    }
    for interface in &module.extern_interfaces {
        for method in &interface.methods {
            for param in &method.params {
                validate_type_ref(&param.ty)?;
            }
            validate_type_ref(&method.return_type)?;
        }
    }
    for definition in &module.structs {
        for field in &definition.fields {
            validate_type_ref(&field.ty)?;
        }
    }
    for function in &module.functions {
        if function.test_name.is_some() {
            validate_test_function_signature(module, function)?;
        }
        if function.is_async && module.domain != "cpu" {
            return Err(format!(
                "mod {} {} cannot declare `async fn {}` yet; async entry is currently only supported in `mod cpu` while {} logic must stay AOT/synchronous and interact through explicit profile/data contracts",
                module.domain,
                module.unit,
                function.name,
                module.domain
            ));
        }
        if function.is_async
            && module.domain == "cpu"
            && function.name == "main"
            && !function.params.is_empty()
        {
            return Err(format!(
                "async entry `mod cpu {}::main` cannot take parameters in the current scheduler; pass data through explicit data/profile contracts or call async helpers from `main` instead",
                module.unit
            ));
        }
        for param in &function.params {
            validate_type_ref(&param.ty)?;
            if function.is_async {
                if let Some(detail) = async_parameter_violation_detail(&param.ty, &struct_table) {
                    return Err(format!(
                    "async function `{}` parameter `{}` cannot cross async boundary with type `{}`; {}; async parameters currently forbid `ref`, resource-bearing `Window<...>` / `WindowMut<...>` / `Pipe<...>`, control-plane `Marker<...>` / `HandleTable<...>`, `?`, `Instance<...>`, `Task<...>`, and `TaskResult<...>` / `DataResult<...>` families",
                    function.name,
                    param.name,
                    param.ty.render(),
                    detail,
                ));
                }
            }
        }
        if let Some(return_type) = &function.return_type {
            validate_type_ref(return_type)?;
            if function.is_async {
                if let Some(detail) = async_boundary_violation_detail(return_type, &struct_table) {
                    return Err(format!(
                    "async function `{}` cannot return `{}` across async boundary; {}; async returns currently forbid `ref`, resource-bearing `Window<...>` / `WindowMut<...>` / `Pipe<...>`, control-plane `Marker<...>` / `HandleTable<...>`, `?`, `Instance<...>`, `Task<...>`, and `*Result<...>` families",
                    function.name,
                    return_type.render()
                    ,
                    detail
                ));
                }
            }
        }
        for stmt in &function.body {
            match stmt {
                NirStmt::Let { ty, .. } => {
                    if let Some(ty) = ty {
                        validate_type_ref(ty)?;
                    }
                }
                NirStmt::Const { ty, .. } => validate_type_ref(ty)?,
                NirStmt::Print(_)
                | NirStmt::Await(_)
                | NirStmt::Expr(_)
                | NirStmt::Return(_)
                | NirStmt::If { .. }
                | NirStmt::While { .. }
                | NirStmt::Break
                | NirStmt::Continue => {}
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::lower_type_ref;
    use super::parse_nuis_ast;
    use super::parse_nuis_module;
    use nuis_semantics::model::{
        AstStmt, AstVisibility, NirBinaryOp, NirDataFlowState, NirExpr, NirKernelFlowState,
        NirShaderFlowState, NirStmt, TestClockDomain, TestClockPolicy,
    };

    #[test]
    fn rejects_unknown_function_annotation() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              @mystery
              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("unknown annotation `@mystery`"));
    }

    #[test]
    fn rejects_unknown_struct_annotation() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              @mystery
              struct Packet {
                id: i64,
              }

              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("struct `Packet` uses unknown annotation `@mystery`"));
    }

    #[test]
    fn rejects_packet_field_outside_packet_struct() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              struct Packet {
                @packet_field
                id: i64,
              }

              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains(
            "annotation `@packet_field` requires parent struct `Packet` to also declare `@packet`"
        ));
    }

    #[test]
    fn rejects_empty_packet_struct() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              @packet
              struct Packet {}

              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("annotation `@packet` requires at least one field"));
    }

    #[test]
    fn rejects_packet_struct_without_packet_fields() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              @packet
              struct Packet {
                id: i64,
              }

              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("annotation `@packet` requires at least one `@packet_field`"));
    }

    #[test]
    fn rejects_ref_fields_inside_packet_struct() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              @packet
              struct Packet {
                @packet_field
                payload: ref i64,
              }

              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains(
            "annotation `@packet_field` currently only supports payload-role fields (role=unsupported-shape)"
        ));
    }

    #[test]
    fn rejects_optional_fields_inside_packet_struct() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              @packet
              struct Packet {
                @packet_field
                payload: i64?,
              }

              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains(
            "annotation `@packet_field` currently only supports payload-role fields (role=unsupported-shape)"
        ));
    }

    #[test]
    fn rejects_marker_fields_inside_packet_struct() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              @packet
              struct Packet {
                @packet_field
                payload: Marker<Tag>,
              }

              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("annotation `@packet_field` currently only supports payload-role fields (role=control-plane)"));
    }

    #[test]
    fn accepts_packet_control_field_for_marker_field() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              @packet
              struct Packet {
                @packet_field
                payload: i64,
                @packet_control_field
                tag: Marker<Tag>,
              }

              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap();

        assert_eq!(module.structs.len(), 1);
        assert_eq!(module.structs[0].fields.len(), 2);
        assert_eq!(
            module.structs[0].fields[1].annotations[0].name,
            "packet_control_field"
        );
    }

    #[test]
    fn rejects_result_fields_inside_packet_struct() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              @packet
              struct Packet {
                @packet_field
                payload: DataResult<i64>,
              }

              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("annotation `@packet_field` currently only supports payload-role fields (role=async-carrier)"));
    }

    #[test]
    fn rejects_task_fields_inside_packet_struct() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              @packet
              struct Packet {
                @packet_field
                payload: Task<i64>,
              }

              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("annotation `@packet_field` currently only supports payload-role fields (role=async-carrier)"));
    }

    #[test]
    fn rejects_handle_table_fields_inside_packet_struct() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              @packet
              struct Packet {
                @packet_field
                payload: HandleTable<Bindings>,
              }

              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("annotation `@packet_field` currently only supports payload-role fields (role=control-plane)"));
    }

    #[test]
    fn rejects_packet_control_field_on_payload_role_field() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              @packet
              struct Packet {
                @packet_field
                payload: i64,
                @packet_control_field
                extra: bool,
              }

              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains(
            "annotation `@packet_control_field` currently only supports control-plane-role fields (role=payload)"
        ));
    }

    #[test]
    fn rejects_field_with_both_packet_slots() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              @packet
              struct Packet {
                @packet_field
                @packet_control_field
                payload: Marker<Tag>,
              }

              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("cannot use both `@packet_field` and `@packet_control_field`"));
    }

    #[test]
    fn rejects_conflicting_inline_annotations() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              @inline
              @noinline
              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("cannot use both `@inline` and `@noinline`"));
    }

    #[test]
    fn rejects_malformed_export_annotation() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              @export("main")
              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("annotation `@export` expects `name = \"...\"`"));
    }

    #[test]
    fn rejects_export_annotation_on_non_main_function() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              @export(name = "entry_main")
              fn helper() -> i64 {
                return 0;
              }

              fn main() -> i64 {
                return helper();
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("only `fn main()` can be exported"));
    }

    #[test]
    fn rejects_export_annotation_with_non_c_symbol_name() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              @export(name = "entry.main")
              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("requires a C-style symbol name"));
    }

    #[test]
    fn rejects_malformed_host_symbol_annotation() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              @host_symbol(name = "network.open_tcp")
              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("annotation `@host_symbol` expects `@host_symbol(\"...\")`"));
    }

    #[test]
    fn rejects_unknown_std_host_symbol_annotation() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              @host_symbol("network.future_magic")
              fn open_magic(value: i64) -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("is not a recognized std-owned host symbol"));
    }

    #[test]
    fn rejects_non_c_extern_host_symbol_bridge() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              extern "nurs" @host_symbol("network.open_tcp") fn open_tcp(local_port: i64, remote_port: i64) -> i64;
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("require `extern \"c\"`"));
    }

    #[test]
    fn lowers_test_function_modifiers_into_nir() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              test(ignored=true, should_fail=true) fn smoke_add() -> i64 {
                return 0;
              }

              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(
            module.contains("cannot be both `ignored` and `should_fail`"),
            "unexpected error: {module}"
        );
    }

    #[test]
    fn lowers_test_function_call_syntax_into_nir() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              test("smoke_add", reason="kept for docs") fn smoke_add() -> i64 {
                return 1;
              }

              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(module.contains("can only use `reason=\"...\"` together with `should_fail=true`"));
    }

    #[test]
    fn lowers_test_function_reason_into_nir() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              test("smoke_add", should_fail=true, reason="must reject zero", timeout_ms=25, clock_domain="monotonic") fn smoke_add() -> i64 {
                return 0;
              }

              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "smoke_add")
            .unwrap();
        assert_eq!(function.test_name.as_deref(), Some("smoke_add"));
        assert!(!function.test_ignored);
        assert!(function.test_should_fail);
        assert_eq!(function.test_reason.as_deref(), Some("must reject zero"));
        assert_eq!(function.test_timeout_ms, Some(25));
        assert_eq!(function.test_clock_domain, Some(TestClockDomain::Monotonic));
        assert_eq!(function.test_clock_policy, None);
    }

    #[test]
    fn parses_test_clock_policy_into_ast() {
        let ast = parse_nuis_ast(
            r#"
            mod cpu Main {
              test("slow_global", timeout_ms=25, clock_domain="global", clock_policy="bridge") async fn slow_global() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = ast
            .functions
            .iter()
            .find(|function| function.name == "slow_global")
            .unwrap();
        assert_eq!(function.test_clock_domain, Some(TestClockDomain::Global));
        assert_eq!(function.test_clock_policy, Some(TestClockPolicy::Bridge));
    }

    #[test]
    fn lowers_test_clock_policy_into_nir() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              test("slow_global", timeout_ms=25, clock_domain="global", clock_policy="bridge") async fn slow_global() -> i64 {
                return 1;
              }

              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "slow_global")
            .unwrap();
        assert_eq!(function.test_clock_domain, Some(TestClockDomain::Global));
        assert_eq!(function.test_clock_policy, Some(TestClockPolicy::Bridge));
        assert!(function
            .annotations
            .iter()
            .any(|annotation| annotation.name == "test"));
    }

    #[test]
    fn lowers_at_test_function_into_nir() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              @test("smoke_add", should_fail=true, reason="must reject zero", timeout_ms=25, clock_domain="monotonic")
              fn smoke_add() -> i64 {
                return 0;
              }

              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "smoke_add")
            .unwrap();
        assert_eq!(function.test_name.as_deref(), Some("smoke_add"));
        assert!(function.test_should_fail);
        assert_eq!(function.test_reason.as_deref(), Some("must reject zero"));
        assert_eq!(function.test_timeout_ms, Some(25));
        assert_eq!(function.test_clock_domain, Some(TestClockDomain::Monotonic));
        assert!(function
            .annotations
            .iter()
            .any(|annotation| annotation.name == "test"));
    }

    #[test]
    fn rejects_mixing_test_declaration_styles() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              @test
              test() fn smoke_add() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("cannot use both `test(...)` and `@test(...)`"));
    }

    #[test]
    fn accepts_bool_and_i64_test_functions() {
        parse_nuis_module(
            r#"
            mod cpu Main {
              test() fn smoke_bool() -> bool {
                return true;
              }

              test() async fn smoke_i64() -> i64 {
                return 1;
              }

              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap();
    }

    #[test]
    fn rejects_test_function_with_parameters() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              test() fn smoke(value: i64) -> i64 {
                return value;
              }

              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("cannot take parameters"));
    }

    #[test]
    fn rejects_test_function_with_unsupported_return_type() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              test() fn smoke() -> String {
                return "nope";
              }

              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("must return `bool` or integer scalar"));
    }

    #[test]
    fn rejects_test_function_with_conflicting_modifiers() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              test(ignored=true, should_fail=true) fn smoke() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("cannot be both `ignored` and `should_fail`"));
    }

    #[test]
    fn rejects_test_reason_without_should_fail() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              test("smoke", reason="must reject zero") fn smoke() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("can only use `reason=\"...\"` together with `should_fail=true`"));
    }

    #[test]
    fn rejects_non_positive_test_timeout() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              test("smoke", timeout_ms=0) fn smoke() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("must use `timeout_ms` > 0"));
    }

    #[test]
    fn rejects_test_clock_domain_without_timeout() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              test("smoke", clock_domain="wall") fn smoke() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(
            error.contains("can only use `clock_domain=\"...\"` together with `timeout_ms=...`")
        );
    }

    #[test]
    fn rejects_unknown_test_clock_domain() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              test("smoke", timeout_ms=25, clock_domain="gpu_global") fn smoke() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("unsupported `clock_domain=\"gpu_global\"`"));
    }

    #[test]
    fn rejects_wall_clock_domain_on_async_tests() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              test("slow_async", timeout_ms=25, clock_domain="wall") async fn slow_async() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("cannot use `clock_domain=\"wall\"` on `async fn`"));
    }

    #[test]
    fn rejects_test_clock_policy_without_timeout() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              test("slow_global", clock_policy="bridge") async fn slow_global() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(
            error.contains("can only use `clock_policy=\"...\"` together with `timeout_ms=...`")
        );
    }

    #[test]
    fn rejects_test_clock_policy_without_global_domain() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              test("slow_mono", timeout_ms=25, clock_domain="monotonic", clock_policy="bridge") async fn slow_mono() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains(
            "can only use `clock_policy=\"bridge\"` together with `clock_domain=\"global\"`"
        ));
    }

    #[test]
    fn rejects_test_function_outside_cpu_domain() {
        let error = parse_nuis_module(
            r#"
            mod shader SurfaceShader {
              test() fn smoke() -> i64 {
                return 1;
              }

              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("only supported in `mod cpu`"));
    }

    #[test]
    fn rejects_legacy_test_prefix_syntax() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              test fn smoke() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("test declarations now require `test(...) fn ...`"));
    }

    #[test]
    fn infers_struct_field_type_from_shared_type_helper() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              struct Packet {
                count: i32,
                label: String,
              }

              fn pick(packet: Packet) -> i32 {
                return packet.count;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "pick")
            .unwrap();
        let return_type = function.return_type.as_ref().unwrap();
        assert_eq!(return_type.render(), "i32");
    }

    #[test]
    fn infers_binary_result_from_operand_scalar_type() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn add(lhs: i32, rhs: i32) -> i32 {
                let sum: i32 = lhs + rhs;
                return sum;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "add")
            .unwrap();
        let sum_stmt = function
            .body
            .iter()
            .find_map(|stmt| match stmt {
                NirStmt::Let { name, ty, .. } if name == "sum" => ty.as_ref(),
                _ => None,
            })
            .unwrap();
        assert_eq!(sum_stmt.render(), "i32");
    }

    #[test]
    fn rejects_non_numeric_binary_operands() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn join(lhs: String, rhs: String) -> String {
                let out: String = lhs + rhs;
                return out;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("numeric scalar operands"));
    }

    #[test]
    fn rejects_bare_window_type_without_payload() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let packet: Window = data_profile_send_uplink("FabricPlane", 7);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("Window"));
        assert!(error.contains("payload type argument"));
    }

    #[test]
    fn rejects_nested_pipe_payload_type() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let pipe: Pipe<Pipe<i64>> = data_output_pipe(7);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("Pipe<Pipe"));
    }

    #[test]
    fn accepts_window_mut_type_annotation() {
        parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let copy: WindowMut<i64> = data_copy_window(7, 0, 1);
              }
            }
            "#,
        )
        .unwrap();
    }

    #[test]
    fn keeps_window_annotation_compatible_with_copy_window_for_now() {
        parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let copy: Window<i64> = data_copy_window(7, 0, 1);
              }
            }
            "#,
        )
        .unwrap();
    }

    #[test]
    fn infers_frozen_window_as_immutable_window_type() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let frozen: Window<i64> = data_freeze_window(data_copy_window(7, 0, 1));
              }
            }
            "#,
        )
        .unwrap();

        let NirStmt::Let { ty: Some(ty), .. } = &module.functions[0].body[0] else {
            panic!("expected typed let binding");
        };
        assert_eq!(ty.render(), "Window<i64>");
    }

    #[test]
    fn infers_written_window_as_mutable_window_type() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let copy: WindowMut<i64> = data_copy_window(7, 0, 1);
                let updated: WindowMut<i64> = data_write_window(copy, 0, 9);
              }
            }
            "#,
        )
        .unwrap();

        let NirStmt::Let { ty: Some(ty), .. } = &module.functions[0].body[1] else {
            panic!("expected typed let binding");
        };
        assert_eq!(ty.render(), "WindowMut<i64>");
    }

    #[test]
    fn infers_buffer_backed_window_payload_as_i64() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let backing: ref Buffer = alloc_buffer(4, 0);
                let copy: WindowMut<i64> = data_copy_window(backing, 1, 2);
              }
            }
            "#,
        )
        .unwrap();

        let NirStmt::Let { ty: Some(ty), .. } = &module.functions[0].body[1] else {
            panic!("expected typed let binding");
        };
        assert_eq!(ty.render(), "WindowMut<i64>");
    }

    #[test]
    fn infers_read_window_payload_type() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let copy: WindowMut<i64> = data_copy_window(7, 0, 1);
                let value: i64 = data_read_window(copy, 0);
              }
            }
            "#,
        )
        .unwrap();

        let NirStmt::Let { ty: Some(ty), .. } = &module.functions[0].body[1] else {
            panic!("expected typed let binding");
        };
        assert_eq!(ty.render(), "i64");
    }

    #[test]
    fn rejects_instance_of_scalar_type() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let wrong: Instance<i64> = instantiate shader SurfaceShader;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("nominal unit type"));
    }

    #[test]
    fn accepts_typed_marker_and_handle_table_annotations() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let handles: HandleTable<FabricBindings> =
                  data_profile_handle_table("FabricPlane");
                let ready: Marker<CpuToShader> =
                  data_profile_marker("FabricPlane", "cpu_to_shader");
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        let declared_types = function
            .body
            .iter()
            .filter_map(|stmt| match stmt {
                NirStmt::Let { ty: Some(ty), .. } => Some(ty.render()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(declared_types.contains(&"HandleTable<FabricBindings>".to_owned()));
        assert!(declared_types.contains(&"Marker<CpuToShader>".to_owned()));
    }

    #[test]
    fn rejects_marker_with_non_nominal_tag_type() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                let ready: Marker<i64> = data_marker("cpu_to_shader");
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("nominal tag type"));
    }

    #[test]
    fn lowers_async_fn_and_await_stmt_into_nir() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              async fn main() {
                await ping();
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.is_async);
        assert!(matches!(function.body.first(), Some(NirStmt::Await(_))));
    }

    #[test]
    fn lowers_await_expression_in_let_and_return() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              async fn main() -> i64 {
                let value: i64 = await ping();
                return await ping();
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                value: NirExpr::Await(_),
                ..
            })
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Return(Some(NirExpr::Await(_))))
        ));
    }

    #[test]
    fn lowers_await_expression_inside_call_args_and_binary_expr() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn add_one(value: i64) -> i64 {
                return value + 1;
              }

              async fn main() -> i64 {
                let value: i64 = add_one(await ping());
                return await ping() + value;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                value: NirExpr::Call { args, .. },
                ..
            }) if matches!(args.first(), Some(NirExpr::Await(_)))
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Return(Some(NirExpr::Binary { lhs, .. })))
                if matches!(lhs.as_ref(), NirExpr::Await(_))
        ));
    }

    #[test]
    fn lowers_while_into_nir_statement() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let value: i64 = 0;
                while value < 3 {
                  print(value);
                  continue;
                }
                return 0;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::While { condition, body })
                if matches!(condition, NirExpr::Binary { .. })
                    && matches!(body.as_slice(), [NirStmt::Print(_), NirStmt::Continue])
        ));
    }

    #[test]
    fn lowers_break_into_nir_statement() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                while true {
                  break;
                }
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::While { body, .. }) if matches!(body.as_slice(), [NirStmt::Break])
        ));
    }

    #[test]
    fn lowers_continue_into_nir_statement() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() {
                while true {
                  continue;
                }
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::While { body, .. }) if matches!(body.as_slice(), [NirStmt::Continue])
        ));
    }

    #[test]
    fn lowers_explicit_spawn_join_and_cancel() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let task: Task<i64> = spawn(ping());
                cancel(task);
                return join(task);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::CpuSpawn { .. },
                ..
            }) if ty.render() == "Task<i64>"
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Expr(NirExpr::CpuCancel(_)))
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Return(Some(NirExpr::CpuJoin(_))))
        ));
    }

    #[test]
    fn lowers_project_local_cpu_helper_calls_with_qualified_callees() {
        let entry = parse_nuis_ast(
            r#"
            use cpu TaskHelpers;

            mod cpu Main {
              fn main() -> i64 {
                return task_policy_completed(7);
              }
            }
            "#,
        )
        .unwrap();
        let helper = parse_nuis_ast(
            r#"
            mod cpu TaskHelpers {
              pub fn encode_completed(value: i64) -> i64 {
                return value + 1;
              }

              pub fn task_policy_completed(value: i64) -> i64 {
                return encode_completed(value);
              }
            }
            "#,
        )
        .unwrap();

        let module = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap();
        let helper_function = module
            .functions
            .iter()
            .find(|function| function.name == "TaskHelpers.task_policy_completed")
            .unwrap();
        assert!(matches!(
            helper_function.body.first(),
            Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
                if callee == "TaskHelpers.encode_completed"
        ));

        let main_function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            main_function.body.first(),
            Some(NirStmt::Return(Some(NirExpr::Call { callee, .. })))
                if callee == "TaskHelpers.task_policy_completed"
        ));
    }

    #[test]
    fn rejects_private_local_cpu_helper_calls_across_modules() {
        let entry = parse_nuis_ast(
            r#"
            use cpu TaskHelpers;

            mod cpu Main {
              fn main() -> i64 {
                return task_policy_completed(7);
              }
            }
            "#,
        )
        .unwrap();
        let helper = parse_nuis_ast(
            r#"
            mod cpu TaskHelpers {
              fn task_policy_completed(value: i64) -> i64 {
                return value + 1;
              }
            }
            "#,
        )
        .unwrap();

        let error = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap_err();
        assert!(
            error.contains("unknown function `task_policy_completed`"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn rejects_private_helper_field_access_across_modules() {
        let entry = parse_nuis_ast(
            r#"
            use cpu Shapes;

            mod cpu Main {
              fn main() -> i64 {
                let cfg: Config = Shapes.make();
                return cfg.secret;
              }
            }
            "#,
        )
        .unwrap();
        let helper = parse_nuis_ast(
            r#"
            mod cpu Shapes {
              pub struct Config {
                pub visible: i64,
                secret: i64
              }

              pub fn make() -> Config {
                return Config {
                  visible: 1,
                  secret: 2
                };
              }
            }
            "#,
        )
        .unwrap();

        let error = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap_err();
        assert!(
            error.contains("type `Config` has no field `secret`"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn rejects_struct_literals_for_imported_structs_with_hidden_private_fields() {
        let entry = parse_nuis_ast(
            r#"
            use cpu Shapes;

            mod cpu Main {
              fn main() -> i64 {
                let cfg: Config = Config {
                  visible: 1
                };
                return cfg.visible;
              }
            }
            "#,
        )
        .unwrap();
        let helper = parse_nuis_ast(
            r#"
            mod cpu Shapes {
              pub struct Config {
                pub visible: i64,
                secret: i64
              }
            }
            "#,
        )
        .unwrap();

        let error = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap_err();
        assert!(
            error.contains("struct literal `Config` cannot be constructed outside its defining module because it hides 1 private field"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn parses_pub_const_items_into_ast() {
        let ast = parse_nuis_ast(
            r#"
            mod cpu Main {
              pub const LIMIT: i64 = 7;

              fn main() -> i64 {
                return LIMIT;
              }
            }
            "#,
        )
        .unwrap();
        assert_eq!(ast.consts.len(), 1);
        assert!(matches!(ast.consts[0].visibility, AstVisibility::Public));
        assert_eq!(ast.consts[0].name, "LIMIT");
        assert_eq!(
            ast.consts[0]
                .ty
                .as_ref()
                .map(|ty| lower_type_ref(ty).render())
                .as_deref(),
            Some("i64")
        );
    }

    #[test]
    fn parses_top_level_const_items_without_explicit_type() {
        let ast = parse_nuis_ast(
            r#"
            mod cpu Main {
              const LIMIT = 7;
            }
            "#,
        )
        .unwrap();
        assert_eq!(ast.consts.len(), 1);
        assert!(ast.consts[0].ty.is_none());
    }

    #[test]
    fn lowers_top_level_const_reads_by_inlining_values() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              const LIMIT: i64 = 7;

              fn main() -> i64 {
                return LIMIT;
              }
            }
            "#,
        )
        .unwrap();
        assert_eq!(module.consts.len(), 1);
        assert!(matches!(
            module.functions[0].body.first(),
            Some(NirStmt::Return(Some(NirExpr::Int(7))))
        ));
    }

    #[test]
    fn infers_top_level_const_item_types_from_values() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              const LIMIT = 7;

              fn main() -> i64 {
                return LIMIT;
              }
            }
            "#,
        )
        .unwrap();
        assert_eq!(module.consts.len(), 1);
        assert_eq!(module.consts[0].ty.render(), "i64");
        assert!(matches!(
            module.functions[0].body.first(),
            Some(NirStmt::Return(Some(NirExpr::Int(7))))
        ));
    }

    #[test]
    fn parses_local_const_without_explicit_type() {
        let ast = parse_nuis_ast(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                const LIMIT = 7;
                return LIMIT;
              }
            }
            "#,
        )
        .unwrap();
        match &ast.functions[0].body[0] {
            AstStmt::Const { ty, .. } => assert!(ty.is_none()),
            other => panic!("expected local const statement, found {other:?}"),
        }
    }

    #[test]
    fn infers_local_const_item_types_inside_branches() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                if true {
                  const LIMIT = 7;
                  return LIMIT;
                } else {
                  match 1 {
                    1 => {
                      const LIMIT = 8;
                      return LIMIT;
                    }
                    _ => {
                      return 9;
                    }
                  }
                }
              }
            }
            "#,
        )
        .unwrap();
        match &module.functions[0].body[0] {
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                match &then_body[0] {
                    NirStmt::Const { ty, .. } => assert_eq!(ty.render(), "i64"),
                    other => panic!("expected inferred const in then branch, found {other:?}"),
                }
                match &else_body[0] {
                    NirStmt::If { then_body, .. } => match &then_body[0] {
                        NirStmt::Const { ty, .. } => assert_eq!(ty.render(), "i64"),
                        other => {
                            panic!("expected inferred const in match arm branch, found {other:?}")
                        }
                    },
                    other => panic!("expected lowered match branch if, found {other:?}"),
                }
            }
            other => panic!("expected if statement, found {other:?}"),
        }
    }

    #[test]
    fn lowers_multi_pattern_match_arms_inside_while() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                while 1 == 1 {
                  match 2 {
                    1 | 2 => {
                      return 7;
                    }
                    _ => {
                      return 9;
                    }
                  }
                }
                return 0;
              }
            }
            "#,
        )
        .unwrap();

        match &module.functions[0].body[0] {
            NirStmt::While { body, .. } => match &body[0] {
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    match condition {
                        NirExpr::Binary {
                            op: NirBinaryOp::Or,
                            lhs,
                            rhs,
                        } => {
                            for side in [lhs.as_ref(), rhs.as_ref()] {
                                match side {
                                    NirExpr::Binary { op, rhs, .. } => {
                                        assert_eq!(*op, NirBinaryOp::Eq);
                                        assert!(matches!(rhs.as_ref(), NirExpr::Int(1) | NirExpr::Int(2)));
                                    }
                                    other => panic!(
                                        "expected equality term in multi-pattern condition, found {other:?}"
                                    ),
                                }
                            }
                        }
                        other => panic!(
                            "expected `or` condition for multi-pattern match arm, found {other:?}"
                        ),
                    }
                    assert!(matches!(
                        then_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Int(7)))]
                    ));
                    assert!(matches!(
                        else_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Int(9)))]
                    ));
                }
                other => panic!("expected lowered match if in while body, found {other:?}"),
            },
            other => panic!("expected while statement, found {other:?}"),
        }
    }

    #[test]
    fn lowers_inclusive_range_match_arms_inside_while() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                while 1 == 1 {
                  match 2 {
                    1..=3 => {
                      return 7;
                    }
                    _ => {
                      return 9;
                    }
                  }
                }
                return 0;
              }
            }
            "#,
        )
        .unwrap();

        match &module.functions[0].body[0] {
            NirStmt::While { body, .. } => match &body[0] {
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    match condition {
                        NirExpr::Binary {
                            op: NirBinaryOp::And,
                            lhs,
                            rhs,
                        } => {
                            match lhs.as_ref() {
                                NirExpr::Binary { op, rhs, .. } => {
                                    assert_eq!(*op, NirBinaryOp::Ge);
                                    assert!(matches!(rhs.as_ref(), NirExpr::Int(1)));
                                }
                                other => panic!(
                                    "expected lower-bound comparison in range match, found {other:?}"
                                ),
                            }
                            match rhs.as_ref() {
                                NirExpr::Binary { op, rhs, .. } => {
                                    assert_eq!(*op, NirBinaryOp::Le);
                                    assert!(matches!(rhs.as_ref(), NirExpr::Int(3)));
                                }
                                other => panic!(
                                    "expected upper-bound comparison in range match, found {other:?}"
                                ),
                            }
                        }
                        other => {
                            panic!("expected `and` condition for range match arm, found {other:?}")
                        }
                    }
                    assert!(matches!(
                        then_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Int(7)))]
                    ));
                    assert!(matches!(
                        else_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Int(9)))]
                    ));
                }
                other => panic!("expected lowered match if in while body, found {other:?}"),
            },
            other => panic!("expected while statement, found {other:?}"),
        }
    }

    #[test]
    fn lowers_match_guard_arms_inside_while() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let ready: bool = true;
                while 1 == 1 {
                  match 2 {
                    2 if ready => {
                      return 7;
                    }
                    _ => {
                      return 9;
                    }
                  }
                }
                return 0;
              }
            }
            "#,
        )
        .unwrap();

        match &module.functions[0].body[1] {
            NirStmt::While { body, .. } => match &body[0] {
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    match condition {
                        NirExpr::Binary {
                            op: NirBinaryOp::And,
                            lhs,
                            rhs,
                        } => {
                            match lhs.as_ref() {
                                NirExpr::Binary { op, rhs, .. } => {
                                    assert_eq!(*op, NirBinaryOp::Eq);
                                    assert!(matches!(rhs.as_ref(), NirExpr::Int(2)));
                                }
                                other => panic!(
                                    "expected equality term in guarded match condition, found {other:?}"
                                ),
                            }
                            assert!(matches!(
                                rhs.as_ref(),
                                NirExpr::Bool(true) | NirExpr::Var(_)
                            ));
                        }
                        other => panic!(
                            "expected `and` condition for guarded match arm, found {other:?}"
                        ),
                    }
                    assert!(matches!(
                        then_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Int(7)))]
                    ));
                    assert!(matches!(
                        else_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Int(9)))]
                    ));
                }
                other => panic!("expected lowered match if in while body, found {other:?}"),
            },
            other => panic!("expected while statement after let binding, found {other:?}"),
        }
    }

    #[test]
    fn lowers_multiple_guarded_match_arms_inside_while() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let ready: bool = true;
                let armed: bool = false;
                while 1 == 1 {
                  match 2 {
                    1 if armed => {
                      return 5;
                    }
                    2 if ready => {
                      return 7;
                    }
                    _ => {
                      return 9;
                    }
                  }
                }
                return 0;
              }
            }
            "#,
        )
        .unwrap();

        match &module.functions[0].body[2] {
            NirStmt::While { body, .. } => match &body[0] {
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    match condition {
                        NirExpr::Binary {
                            op: NirBinaryOp::And,
                            lhs,
                            rhs,
                        } => {
                            match lhs.as_ref() {
                                NirExpr::Binary { op, rhs, .. } => {
                                    assert_eq!(*op, NirBinaryOp::Eq);
                                    assert!(matches!(rhs.as_ref(), NirExpr::Int(1)));
                                }
                                other => panic!(
                                    "expected equality term in first guarded match condition, found {other:?}"
                                ),
                            }
                            assert!(matches!(
                                rhs.as_ref(),
                                NirExpr::Bool(false) | NirExpr::Var(_)
                            ));
                        }
                        other => panic!(
                            "expected `and` condition for first guarded match arm, found {other:?}"
                        ),
                    }
                    assert!(matches!(
                        then_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Int(5)))]
                    ));
                    match else_body.as_slice() {
                        [NirStmt::If {
                            condition,
                            then_body,
                            else_body,
                        }] => {
                            match condition {
                                NirExpr::Binary {
                                    op: NirBinaryOp::And,
                                    lhs,
                                    rhs,
                                } => {
                                    match lhs.as_ref() {
                                        NirExpr::Binary { op, rhs, .. } => {
                                            assert_eq!(*op, NirBinaryOp::Eq);
                                            assert!(matches!(rhs.as_ref(), NirExpr::Int(2)));
                                        }
                                        other => panic!(
                                            "expected equality term in second guarded match condition, found {other:?}"
                                        ),
                                    }
                                    assert!(matches!(
                                        rhs.as_ref(),
                                        NirExpr::Bool(true) | NirExpr::Var(_)
                                    ));
                                }
                                other => panic!(
                                    "expected `and` condition for second guarded match arm, found {other:?}"
                                ),
                            }
                            assert!(matches!(
                                then_body.as_slice(),
                                [NirStmt::Return(Some(NirExpr::Int(7)))]
                            ));
                            assert!(matches!(
                                else_body.as_slice(),
                                [NirStmt::Return(Some(NirExpr::Int(9)))]
                            ));
                        }
                        other => panic!(
                            "expected nested if chain for multiple guarded match arms, found {other:?}"
                        ),
                    }
                }
                other => panic!("expected lowered match if in while body, found {other:?}"),
            },
            other => panic!("expected while statement after let bindings, found {other:?}"),
        }
    }

    #[test]
    fn lowers_or_pattern_guard_match_arms_inside_while() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let ready: bool = true;
                while 1 == 1 {
                  match 2 {
                    1 | 2 if ready => {
                      return 7;
                    }
                    _ => {
                      return 9;
                    }
                  }
                }
                return 0;
              }
            }
            "#,
        )
        .unwrap();

        match &module.functions[0].body[1] {
            NirStmt::While { body, .. } => match &body[0] {
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    match condition {
                        NirExpr::Binary {
                            op: NirBinaryOp::And,
                            lhs,
                            rhs,
                        } => {
                            match lhs.as_ref() {
                                NirExpr::Binary {
                                    op: NirBinaryOp::Or,
                                    ..
                                } => {}
                                other => panic!(
                                    "expected `or` term in guarded multi-pattern match condition, found {other:?}"
                                ),
                            }
                            assert!(matches!(
                                rhs.as_ref(),
                                NirExpr::Bool(true) | NirExpr::Var(_)
                            ));
                        }
                        other => panic!(
                            "expected `and` condition for guarded multi-pattern match arm, found {other:?}"
                        ),
                    }
                    assert!(matches!(
                        then_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Int(7)))]
                    ));
                    assert!(matches!(
                        else_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Int(9)))]
                    ));
                }
                other => panic!("expected lowered guarded match if in while body, found {other:?}"),
            },
            other => panic!("expected while statement after let binding, found {other:?}"),
        }
    }

    #[test]
    fn lowers_range_guard_match_arms_inside_while() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let ready: bool = true;
                while 1 == 1 {
                  match 2 {
                    1..=3 if ready => {
                      return 7;
                    }
                    _ => {
                      return 9;
                    }
                  }
                }
                return 0;
              }
            }
            "#,
        )
        .unwrap();

        match &module.functions[0].body[1] {
            NirStmt::While { body, .. } => match &body[0] {
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    match condition {
                        NirExpr::Binary {
                            op: NirBinaryOp::And,
                            lhs,
                            rhs,
                        } => {
                            match lhs.as_ref() {
                                NirExpr::Binary {
                                    op: NirBinaryOp::And,
                                    ..
                                } => {}
                                other => panic!(
                                    "expected range conjunction term in guarded range match condition, found {other:?}"
                                ),
                            }
                            assert!(matches!(
                                rhs.as_ref(),
                                NirExpr::Bool(true) | NirExpr::Var(_)
                            ));
                        }
                        other => panic!(
                            "expected `and` condition for guarded range match arm, found {other:?}"
                        ),
                    }
                    assert!(matches!(
                        then_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Int(7)))]
                    ));
                    assert!(matches!(
                        else_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Int(9)))]
                    ));
                }
                other => panic!("expected lowered guarded match if in while body, found {other:?}"),
            },
            other => panic!("expected while statement after let binding, found {other:?}"),
        }
    }

    #[test]
    fn lowers_struct_field_match_arms_inside_while() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              struct Packet {
                kind: i64,
                ready: bool,
              }

              fn main() -> i64 {
                let armed: bool = true;
                let packet: Packet = Packet { kind: 2, ready: true };
                while 1 == 1 {
                  match packet {
                    Packet { kind: 1 | 2, ready: true } if armed => {
                      return 7;
                    }
                    _ => {
                      return 9;
                    }
                  }
                }
                return 0;
              }
            }
            "#,
        )
        .unwrap();

        match &module.functions[0].body[2] {
            NirStmt::While { body, .. } => match &body[0] {
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    match condition {
                        NirExpr::Binary {
                            op: NirBinaryOp::And,
                            lhs,
                            rhs,
                        } => {
                            match lhs.as_ref() {
                                NirExpr::Binary {
                                    op: NirBinaryOp::And,
                                    ..
                                } => {}
                                other => panic!(
                                    "expected field conjunction term in struct match condition, found {other:?}"
                                ),
                            }
                            assert!(matches!(
                                rhs.as_ref(),
                                NirExpr::Bool(true) | NirExpr::Var(_)
                            ));
                        }
                        other => panic!(
                            "expected `and` condition for guarded struct match arm, found {other:?}"
                        ),
                    }
                    assert!(matches!(
                        then_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Int(7)))]
                    ));
                    assert!(matches!(
                        else_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Int(9)))]
                    ));
                }
                other => panic!("expected lowered struct match if in while body, found {other:?}"),
            },
            other => panic!("expected while statement after bindings, found {other:?}"),
        }
    }

    #[test]
    fn lowers_nested_struct_match_arms_inside_while() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              struct Header {
                kind: i64,
                ready: bool,
              }

              struct Packet {
                header: Header,
                code: i64,
              }

              fn main() -> i64 {
                let armed: bool = true;
                let packet: Packet = Packet {
                  header: Header { kind: 2, ready: true },
                  code: 5,
                };
                while 1 == 1 {
                  match packet {
                    Packet { header: Header { kind: 1 | 2, ready: true }, code: 5 } if armed => {
                      return 7;
                    }
                    _ => {
                      return 9;
                    }
                  }
                }
                return 0;
              }
            }
            "#,
        )
        .unwrap();

        match &module.functions[0].body[2] {
            NirStmt::While { body, .. } => match &body[0] {
                NirStmt::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    match condition {
                        NirExpr::Binary {
                            op: NirBinaryOp::And,
                            lhs,
                            rhs,
                        } => {
                            match lhs.as_ref() {
                                NirExpr::Binary {
                                    op: NirBinaryOp::And,
                                    ..
                                } => {}
                                other => panic!(
                                    "expected nested field conjunction term in struct match condition, found {other:?}"
                                ),
                            }
                            assert!(matches!(
                                rhs.as_ref(),
                                NirExpr::Bool(true) | NirExpr::Var(_)
                            ));
                        }
                        other => panic!(
                            "expected `and` condition for guarded nested struct match arm, found {other:?}"
                        ),
                    }
                    assert!(matches!(
                        then_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Int(7)))]
                    ));
                    assert!(matches!(
                        else_body.as_slice(),
                        [NirStmt::Return(Some(NirExpr::Int(9)))]
                    ));
                }
                other => {
                    panic!("expected lowered nested struct match if in while body, found {other:?}")
                }
            },
            other => panic!("expected while statement after bindings, found {other:?}"),
        }
    }

    #[test]
    fn helper_pub_consts_can_cross_module_but_private_ones_cannot() {
        let entry = parse_nuis_ast(
            r#"
            use cpu Limits;

            mod cpu Main {
              fn main() -> i64 {
                return LIMIT;
              }
            }
            "#,
        )
        .unwrap();
        let helper = parse_nuis_ast(
            r#"
            mod cpu Limits {
              pub const LIMIT: i64 = 9;
              const SECRET: i64 = 5;
            }
            "#,
        )
        .unwrap();
        let module = super::lower_project_ast_to_nir(&entry, &[helper.clone()]).unwrap();
        let main_function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            main_function.body.first(),
            Some(NirStmt::Return(Some(NirExpr::Int(9))))
        ));

        let hidden_entry = parse_nuis_ast(
            r#"
            use cpu Limits;

            mod cpu Main {
              fn main() -> i64 {
                return SECRET;
              }
            }
            "#,
        )
        .unwrap();
        let error = super::lower_project_ast_to_nir(&hidden_entry, &[helper]).unwrap_err();
        assert!(
            error.contains("unknown value `SECRET`"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn parses_pub_type_alias_items_into_ast() {
        let ast = parse_nuis_ast(
            r#"
            mod cpu Main {
              pub type Count = i64;

              fn main() -> Count {
                return 7;
              }
            }
            "#,
        )
        .unwrap();
        assert_eq!(ast.type_aliases.len(), 1);
        assert!(matches!(
            ast.type_aliases[0].visibility,
            AstVisibility::Public
        ));
        assert_eq!(ast.type_aliases[0].name, "Count");
        assert_eq!(ast.type_aliases[0].target.name, "i64");
    }

    #[test]
    fn lowers_type_aliases_into_nir_and_resolves_declared_types() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              type Count = i64;

              fn main() -> Count {
                let value: Count = 7;
                return value;
              }
            }
            "#,
        )
        .unwrap();
        assert_eq!(module.type_aliases.len(), 1);
        assert_eq!(module.type_aliases[0].target.name, "i64");
        assert_eq!(
            module.functions[0]
                .return_type
                .as_ref()
                .map(|ty| ty.render()),
            Some("i64".to_owned())
        );
        assert!(matches!(
            module.functions[0].body.first(),
            Some(NirStmt::Let { ty: Some(ty), .. }) if ty.render() == "i64"
        ));
    }

    #[test]
    fn helper_pub_type_aliases_can_cross_module() {
        let entry = parse_nuis_ast(
            r#"
            use cpu Types;

            mod cpu Main {
              fn main() -> i64 {
                let value: Count = 7;
                return value;
              }
            }
            "#,
        )
        .unwrap();
        let helper = parse_nuis_ast(
            r#"
            mod cpu Types {
              pub type Count = i64;
            }
            "#,
        )
        .unwrap();
        let module = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap();
        let main_function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            main_function.body.first(),
            Some(NirStmt::Let { ty: Some(ty), .. }) if ty.render() == "i64"
        ));
    }

    #[test]
    fn rejects_cyclic_type_aliases() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              type A = B;
              type B = A;

              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap_err();
        assert!(
            error.contains("type alias `A` is cyclic")
                || error.contains("type alias `B` is cyclic"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn lowers_generic_type_aliases_into_nir() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              pub type PipeOf<T> = Pipe<T>;

              fn use_pipe(pipe: PipeOf<i64>) -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap();
        assert_eq!(module.type_aliases.len(), 1);
        assert_eq!(module.type_aliases[0].generic_params.len(), 1);
        assert_eq!(module.type_aliases[0].target.render(), "Pipe<T>");
        let use_pipe = module
            .functions
            .iter()
            .find(|function| function.name == "use_pipe")
            .unwrap();
        assert_eq!(use_pipe.params[0].ty.render(), "Pipe<i64>");
    }

    #[test]
    fn helper_pub_generic_type_aliases_can_cross_module() {
        let entry = parse_nuis_ast(
            r#"
            use cpu Types;

            mod cpu Main {
              fn use_pipe(pipe: PipeOf<i64>) -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap();
        let helper = parse_nuis_ast(
            r#"
            mod cpu Types {
              pub type PipeOf<T> = Pipe<T>;
            }
            "#,
        )
        .unwrap();
        let module = super::lower_project_ast_to_nir(&entry, &[helper]).unwrap();
        let use_pipe = module
            .functions
            .iter()
            .find(|function| function.name == "use_pipe")
            .unwrap();
        assert_eq!(use_pipe.params[0].ty.render(), "Pipe<i64>");
    }

    #[test]
    fn rejects_spawn_of_sync_function() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn ping() -> i64 {
                return 7;
              }

              fn main() {
                let task: Task<i64> = spawn(ping());
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("spawn(...) expects async function call"));
    }

    #[test]
    fn rejects_join_of_non_task_value() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                return join(7);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("expects `Task<...>`"));
    }

    #[test]
    fn rejects_spawn_of_borrowed_input() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping(head_ref: ref Node) -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let head: ref Node = alloc_node(1, null());
                let task: Task<i64> = spawn(ping(borrow(head)));
                return join(task);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("does not currently allow borrowed task inputs"));
    }

    #[test]
    fn rejects_spawn_of_ref_typed_input() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping(head: ref Node) -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let head: ref Node = alloc_node(1, null());
                let task: Task<i64> = spawn(ping(head));
                return join(task);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("does not currently allow `ref` task inputs"));
    }

    #[test]
    fn rejects_async_function_ref_parameter_boundary() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping(head: ref Node) -> i64 {
                return 7;
              }

              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("cannot cross async boundary"));
        assert!(error.contains("`Task<...>`"));
    }

    #[test]
    fn rejects_async_function_result_family_return_boundary() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> TaskResult<i64> {
                return join_result(timeout(spawn(pong()), 16));
              }

              async fn pong() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                return 1;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("cannot return `TaskResult<i64>` across async boundary"));
        assert!(error.contains("*Result<...>"));
    }

    #[test]
    fn rejects_task_completed_on_raw_task_input() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> bool {
                let task: Task<i64> = spawn(ping());
                return task_completed(task);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("task_completed(...) expects `TaskResult<...>`"));
        assert!(error.contains("found `Task<i64>`"));
    }

    #[test]
    fn rejects_task_value_on_join_payload_input() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let task: Task<i64> = spawn(ping());
                let value: i64 = join(task);
                return task_value(value);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("task_value(...) expects `TaskResult<...>`"));
        assert!(error.contains("found `i64`"));
    }

    #[test]
    fn lowers_explicit_data_result_helpers() {
        let module = parse_nuis_module(
            r#"
            mod data FabricPlane {
              fn main() -> i64 {
                let pipe_result: DataResult<Pipe<i64>> = data_result(data_output_pipe(7));
                let moved: bool = data_moved(pipe_result);
                let intake: DataResult<i64> = data_result(data_input_pipe(data_output_pipe(9)));
                let ready: bool = data_ready(intake);
                let value: i64 = data_value(intake);
                return value;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::DataResult { state, .. },
                ..
            }) if ty.render() == "DataResult<Pipe<i64>>"
                && matches!(state, NirDataFlowState::Moved)
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::DataMoved(_),
                ..
            }) if ty.render() == "bool"
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::DataResult { state, .. },
                ..
            }) if ty.render() == "DataResult<i64>"
                && matches!(state, NirDataFlowState::Ready)
        ));
        assert!(matches!(
            function.body.get(4),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::DataValue(_),
                ..
            }) if ty.render() == "i64"
        ));
    }

    #[test]
    fn rejects_data_result_of_non_data_operation() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let result: DataResult<i64> = data_result(7);
                return data_value(result);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("data_result(...) expects a direct data operation"));
    }

    #[test]
    fn lowers_explicit_shader_result_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let pass_result: ShaderResult<Pass> = shader_result(shader_begin_pass(
                  shader_target("rgba8", 16, 16),
                  shader_pipeline("flat", "triangle"),
                  shader_viewport(16, 16)
                ));
                let frame_result: ShaderResult<Frame> = shader_result(shader_profile_render(
                  "SurfaceShader",
                  shader_profile_packet("SurfaceShader", 1, 2, 3)
                ));
                let ready: bool = shader_frame_ready(frame_result);
                let frame: Frame = shader_value(frame_result);
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderResult { state, .. },
                ..
            }) if ty.render() == "ShaderResult<Pass>"
                && matches!(state, NirShaderFlowState::PassReady)
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderResult { state, .. },
                ..
            }) if ty.render() == "ShaderResult<Frame>"
                && matches!(state, NirShaderFlowState::FrameReady)
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderFrameReady(_),
                ..
            }) if ty.render() == "bool"
        ));
        assert!(matches!(
            function.body.get(3),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::ShaderValue(_),
                ..
            }) if ty.render() == "Frame"
        ));
    }

    #[test]
    fn lowers_nova_panel_packet_without_shader_unit_literal() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let packet: NovaPanelPacket = nova_panel_packet(1, 2, 3, 4, 5, 6);
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value:
                    NirExpr::ShaderProfilePacket {
                        unit,
                        packet_type_name,
                        accent: Some(_),
                        toggle_state: Some(_),
                        focus_index: Some(_),
                        ..
                    },
                ..
            }) if ty.render() == "NovaPanelPacket"
                && unit == "__nova__"
                && packet_type_name.as_deref() == Some("NovaPanelPacket")
        ));
    }

    #[test]
    fn lowers_nova_control_packet_builders() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let slider: NovaSliderPacket = nova_slider_packet(7, 0, 10, 2, 1);
                let progress: NovaProgressPacket = nova_progress_packet(4, 10);
                let toggle: NovaTogglePacket = nova_toggle_packet(1, 1);
                let button: NovaButtonPacket = nova_button_packet(1, 9, 2);
                let text_input: NovaTextInputPacket =
                  nova_text_input_packet(8, 1, 4, 1, 1);
                let select: NovaSelectPacket = nova_select_packet(2, 5, 4, 1, 0);
                let checkbox: NovaCheckboxPacket = nova_checkbox_packet(1, 5, 0);
                let radio: NovaRadioPacket = nova_radio_packet(2, 4, 5, 1);
                let textarea: NovaTextAreaPacket = nova_textarea_packet(3, 1, 7, 0, 1);
                let tabs: NovaTabsPacket = nova_tabs_packet(1, 4, 5, 0);
                let list: NovaListPacket = nova_list_packet(1, 5, 7, 1);
                let table: NovaTablePacket = nova_table_packet(4, 3, 1, 1);
                let tree: NovaTreePacket = nova_tree_packet(1, 6, 1, 7);
                let inspector: NovaInspectorPacket = nova_inspector_packet(1, 4, 1, 7);
                let outline: NovaOutlinePacket = nova_outline_packet(1, 6, 1, 7);
                let theme: NovaThemePacket = nova_theme_packet(7, 3, 1, 2);
                let selection: NovaSelectionPacket = nova_selection_packet(1, 6, 1, 4);
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaSliderPacket" && type_name == "NovaSliderPacket"
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaProgressPacket" && type_name == "NovaProgressPacket"
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTogglePacket" && type_name == "NovaTogglePacket"
        ));
        assert!(matches!(
            function.body.get(3),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaButtonPacket" && type_name == "NovaButtonPacket"
        ));
        assert!(matches!(
            function.body.get(4),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTextInputPacket" && type_name == "NovaTextInputPacket"
        ));
        assert!(matches!(
            function.body.get(5),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaSelectPacket" && type_name == "NovaSelectPacket"
        ));
        assert!(matches!(
            function.body.get(6),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaCheckboxPacket" && type_name == "NovaCheckboxPacket"
        ));
        assert!(matches!(
            function.body.get(7),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaRadioPacket" && type_name == "NovaRadioPacket"
        ));
        assert!(matches!(
            function.body.get(8),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTextAreaPacket" && type_name == "NovaTextAreaPacket"
        ));
        assert!(matches!(
            function.body.get(9),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTabsPacket" && type_name == "NovaTabsPacket"
        ));
        assert!(matches!(
            function.body.get(10),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaListPacket" && type_name == "NovaListPacket"
        ));
        assert!(matches!(
            function.body.get(11),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTablePacket" && type_name == "NovaTablePacket"
        ));
        assert!(matches!(
            function.body.get(12),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTreePacket" && type_name == "NovaTreePacket"
        ));
        assert!(matches!(
            function.body.get(13),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaInspectorPacket" && type_name == "NovaInspectorPacket"
        ));
        assert!(matches!(
            function.body.get(14),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaOutlinePacket" && type_name == "NovaOutlinePacket"
        ));
        assert!(matches!(
            function.body.get(15),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaThemePacket" && type_name == "NovaThemePacket"
        ));
        assert!(matches!(
            function.body.get(16),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaSelectionPacket" && type_name == "NovaSelectionPacket"
        ));
    }

    #[test]
    fn lowers_nova_control_state_observers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let slider: NovaSliderPacket = nova_slider_packet(7, 0, 10, 2, 1);
                let text_input: NovaTextInputPacket =
                  nova_text_input_packet(8, 1, 4, 1, 1);
                let select: NovaSelectPacket = nova_select_packet(2, 5, 4, 1, 0);
                let slider_disabled: i64 = nova_slider_disabled(slider);
                let dirty: i64 = nova_text_input_dirty(text_input);
                let committed: i64 = nova_select_committed(select);
                return committed;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(3),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "disabled"
        ));
        assert!(matches!(
            function.body.get(4),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "dirty"
        ));
        assert!(matches!(
            function.body.get(5),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "committed"
        ));
    }

    #[test]
    fn lowers_extended_nova_control_state_observers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let checkbox: NovaCheckboxPacket = nova_checkbox_packet(1, 5, 1);
                let radio: NovaRadioPacket = nova_radio_packet(2, 4, 5, 0);
                let textarea: NovaTextAreaPacket = nova_textarea_packet(3, 1, 7, 1, 1);
                let tabs: NovaTabsPacket = nova_tabs_packet(1, 4, 5, 1);
                let checkbox_state: NovaCheckboxState = nova_checkbox_state(checkbox);
                let radio_state: NovaRadioState = nova_radio_state(radio);
                let textarea_state: NovaTextAreaState = nova_textarea_state(textarea);
                let tabs_state: NovaTabsState = nova_tabs_state(tabs);
                let checked: i64 = nova_checkbox_state_checked(checkbox_state);
                let radio_disabled: i64 = nova_radio_state_disabled(radio_state);
                let dirty: i64 = nova_textarea_state_dirty(textarea_state);
                let compact: i64 = nova_tabs_state_compact(tabs_state);
                return checked + radio_disabled + dirty + compact;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(4),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaCheckboxState" && type_name == "NovaCheckboxState"
        ));
        assert!(matches!(
            function.body.get(5),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaRadioState" && type_name == "NovaRadioState"
        ));
        assert!(matches!(
            function.body.get(6),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTextAreaState" && type_name == "NovaTextAreaState"
        ));
        assert!(matches!(
            function.body.get(7),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTabsState" && type_name == "NovaTabsState"
        ));
        assert!(matches!(
            function.body.get(8),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "checked"
        ));
        assert!(matches!(
            function.body.get(9),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "disabled"
        ));
        assert!(matches!(
            function.body.get(10),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "dirty"
        ));
        assert!(matches!(
            function.body.get(11),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "compact"
        ));
    }

    #[test]
    fn lowers_complex_nova_control_state_observers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let list: NovaListPacket = nova_list_packet(1, 5, 7, 1);
                let table: NovaTablePacket = nova_table_packet(4, 3, 1, 1);
                let list_state: NovaListState = nova_list_state(list);
                let table_state: NovaTableState = nova_table_state(table);
                let tree: NovaTreePacket = nova_tree_packet(1, 6, 1, 7);
                let tree_state: NovaTreeState = nova_tree_state(tree);
                let inspector: NovaInspectorPacket = nova_inspector_packet(1, 4, 1, 7);
                let inspector_state: NovaInspectorState = nova_inspector_state(inspector);
                let outline: NovaOutlinePacket = nova_outline_packet(1, 6, 1, 7);
                let outline_state: NovaOutlineState = nova_outline_state(outline);
                let dense: i64 = nova_list_state_dense(list_state);
                let selected: i64 = nova_list_state_selected(list_state);
                let zebra: i64 = nova_table_state_zebra(table_state);
                let selected_row: i64 = nova_table_state_selected_row(table_state);
                let expanded: i64 = nova_tree_state_expanded(tree_state);
                let tree_selected: i64 = nova_tree_state_selected(tree_state);
                let pinned: i64 = nova_inspector_state_pinned(inspector_state);
                let inspected: i64 = nova_inspector_state_selected(inspector_state);
                let collapsed: i64 = nova_outline_state_collapsed(outline_state);
                let outlined: i64 = nova_outline_state_selected(outline_state);
                return dense + selected + zebra + selected_row + expanded + tree_selected + pinned + inspected + collapsed + outlined;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaListState" && type_name == "NovaListState"
        ));
        assert!(matches!(
            function.body.get(3),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTableState" && type_name == "NovaTableState"
        ));
        assert!(matches!(
            function.body.get(5),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaTreeState" && type_name == "NovaTreeState"
        ));
        assert!(matches!(
            function.body.get(7),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaInspectorState" && type_name == "NovaInspectorState"
        ));
        assert!(matches!(
            function.body.get(9),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaOutlineState" && type_name == "NovaOutlineState"
        ));
        assert!(matches!(
            function.body.get(10),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "dense"
        ));
        assert!(matches!(
            function.body.get(11),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "selected"
        ));
        assert!(matches!(
            function.body.get(12),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "zebra"
        ));
        assert!(matches!(
            function.body.get(13),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "selected_row"
        ));
        assert!(matches!(
            function.body.get(14),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "expanded"
        ));
        assert!(matches!(
            function.body.get(15),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "selected"
        ));
        assert!(matches!(
            function.body.get(16),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "pinned"
        ));
        assert!(matches!(
            function.body.get(17),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "selected"
        ));
        assert!(matches!(
            function.body.get(18),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "collapsed"
        ));
        assert!(matches!(
            function.body.get(19),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "selected"
        ));
    }

    #[test]
    fn lowers_shared_nova_selection_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let selection: NovaSelectionPacket = nova_selection_packet(2, 6, 1, 4);
                let list: NovaListPacket = nova_list_packet(2, 6, 7, 1);
                let table: NovaTablePacket = nova_table_packet(4, 3, 2, 1);
                let tree: NovaTreePacket = nova_tree_packet(2, 6, 1, 7);
                let inspector: NovaInspectorPacket = nova_inspector_packet(2, 4, 1, 7);
                let outline: NovaOutlinePacket = nova_outline_packet(2, 6, 1, 7);
                let state: NovaSelectionState = nova_selection_state(selection);
                let list_selection: NovaSelectionState = nova_list_selection(list);
                let table_selection: NovaSelectionState = nova_table_selection(table);
                let tree_selection: NovaSelectionState = nova_tree_selection(tree);
                let inspector_selection: NovaSelectionState = nova_inspector_selection(inspector);
                let outline_selection: NovaSelectionState = nova_outline_selection(outline);
                let selected: i64 = nova_selection_state_selected(state);
                let span: i64 = nova_selection_state_span(list_selection);
                let mode: i64 = nova_selection_state_mode(table_selection);
                let origin: i64 = nova_selection_state_origin(tree_selection);
                let inspector_origin: i64 = nova_selection_state_origin(inspector_selection);
                let outline_origin: i64 = nova_selection_state_origin(outline_selection);
                return selected + span + mode + origin + inspector_origin + outline_origin;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(6),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaSelectionState" && type_name == "NovaSelectionState"
        ));
        assert!(matches!(
            function.body.get(7),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaSelectionState" && type_name == "NovaSelectionState"
        ));
        assert!(matches!(
            function.body.get(12),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "selected"
        ));
        assert!(matches!(
            function.body.get(13),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "span"
        ));
        assert!(matches!(
            function.body.get(14),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "mode"
        ));
        assert!(matches!(
            function.body.get(15),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "origin"
        ));
    }

    #[test]
    fn lowers_nova_theme_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let theme: NovaThemePacket = nova_theme_packet(7, 3, 1, 2);
                let state: NovaThemeState = nova_theme_state(theme);
                let accent: i64 = nova_theme_state_accent(state);
                let surface: i64 = nova_theme_state_surface(state);
                let panel_mode: i64 = nova_theme_state_panel_mode(state);
                let contrast: i64 = nova_theme_state_contrast(state);
                return accent + surface + panel_mode + contrast;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            }) if ty.render() == "NovaThemeState" && type_name == "NovaThemeState"
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "accent"
        ));
        assert!(matches!(
            function.body.get(5),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::FieldAccess { field, .. },
                ..
            }) if ty.render() == "i64" && field == "contrast"
        ));
    }

    #[test]
    fn lowers_nova_render_state_contracts() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let surface: NovaSurfacePacket = nova_surface_packet(3, 2, 1, 4);
                let viewport: NovaViewportPacket = nova_viewport_packet(2, 1, 48, 18);
                let layer: NovaLayerPacket = nova_layer_packet(1, 2, 1, 0);
                let surface_state: NovaSurfaceState = nova_surface_state(surface);
                let viewport_state: NovaViewportState = nova_viewport_state(viewport);
                let layer_state: NovaLayerState = nova_layer_state(layer);
                let density: i64 = nova_surface_state_density(surface_state);
                let width: i64 = nova_viewport_state_width(viewport_state);
                let visibility: i64 = nova_layer_state_visibility(layer_state);
                return density + width + visibility;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSurfaceState" && type_name == "NovaSurfaceState",
            _ => false,
        }));
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaViewportState" && type_name == "NovaViewportState",
            _ => false,
        }));
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaLayerState" && type_name == "NovaLayerState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_scene_state_contracts() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let scene: NovaScenePacket = nova_scene_packet(7, 2, 3, 1);
                let camera: NovaCameraPacket = nova_camera_packet(1, 2, 12, 9);
                let material: NovaMaterialPacket = nova_material_packet(1, 8, 3, 2);
                let scene_state: NovaSceneState = nova_scene_state(scene);
                let camera_state: NovaCameraState = nova_camera_state(camera);
                let material_state: NovaMaterialState = nova_material_state(material);
                let lights: i64 = nova_scene_state_light_count(scene_state);
                let zoom: i64 = nova_camera_state_zoom(camera_state);
                let emissive: i64 = nova_material_state_emissive(material_state);
                return lights + zoom + emissive;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSceneState" && type_name == "NovaSceneState",
            _ => false,
        }));
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaCameraState" && type_name == "NovaCameraState",
            _ => false,
        }));
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaMaterialState" && type_name == "NovaMaterialState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_light_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let light: NovaLightPacket = nova_light_packet(1, 12, 9, 8);
                let state: NovaLightState = nova_light_state(light);
                let intensity: i64 = nova_light_state_intensity(state);
                return intensity;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaLightState" && type_name == "NovaLightState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_mesh_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let mesh: NovaMeshPacket = nova_mesh_packet(1, 12, 9, 8);
                let state: NovaMeshState = nova_mesh_state(mesh);
                let vertices: i64 = nova_mesh_state_vertex_count(state);
                return vertices;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaMeshState" && type_name == "NovaMeshState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_transform_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let transform: NovaTransformPacket = nova_transform_packet(12, 1, 9, 2);
                let state: NovaTransformState = nova_transform_state(transform);
                let scale: i64 = nova_transform_state_scale(state);
                return scale;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaTransformState" && type_name == "NovaTransformState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_node_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let node: NovaNodePacket = nova_node_packet(2, 1, 8, 2);
                let state: NovaNodeState = nova_node_state(node);
                let depth: i64 = nova_node_state_depth(state);
                return depth;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaNodeState" && type_name == "NovaNodeState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_scene_link_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let link: NovaSceneLinkPacket = nova_scene_link_packet(1, 2, 3, 4, 5, 6);
                let state: NovaSceneLinkState = nova_scene_link_state(link);
                let mesh_slot: i64 = nova_scene_link_state_mesh(state);
                return mesh_slot;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSceneLinkState" && type_name == "NovaSceneLinkState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_instance_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let instance: NovaInstancePacket = nova_instance_packet(1, 2, 3, 4, 5, 6);
                let state: NovaInstanceState = nova_instance_state(instance);
                let count: i64 = nova_instance_state_count(state);
                return count;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaInstanceState" && type_name == "NovaInstanceState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_scene_graph_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let graph: NovaSceneGraphPacket = nova_scene_graph_packet(1, 6, 3, 2, 1);
                let state: NovaSceneGraphState = nova_scene_graph_state(graph);
                let roots: i64 = nova_scene_graph_state_root(state);
                return roots;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSceneGraphState" && type_name == "NovaSceneGraphState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_scene_node_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let node: NovaSceneNodePacket = nova_scene_node_packet(1, 2, 3, 4, 1);
                let state: NovaSceneNodeState = nova_scene_node_state(node);
                let child: i64 = nova_scene_node_state_first_child(state);
                return child;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSceneNodeState" && type_name == "NovaSceneNodeState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_instance_group_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let group: NovaInstanceGroupPacket = nova_instance_group_packet(1, 4, 3, 2, 8);
                let state: NovaInstanceGroupState = nova_instance_group_state(group);
                let visible: i64 = nova_instance_group_state_visible(state);
                return visible;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaInstanceGroupState" && type_name == "NovaInstanceGroupState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_scene_cluster_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let cluster: NovaSceneClusterPacket = nova_scene_cluster_packet(1, 6, 3, 8, 1);
                let state: NovaSceneClusterState = nova_scene_cluster_state(cluster);
                let budget: i64 = nova_scene_cluster_state_budget(state);
                return budget;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSceneClusterState" && type_name == "NovaSceneClusterState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_visibility_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let visibility: NovaVisibilityPacket = nova_visibility_packet(3, 5, 1, 2, 7);
                let state: NovaVisibilityState = nova_visibility_state(visibility);
                let visible: i64 = nova_visibility_state_visible(state);
                return visible;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaVisibilityState" && type_name == "NovaVisibilityState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_cull_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let cull: NovaCullPacket = nova_cull_packet(3, 4, 1, 2, 7);
                let state: NovaCullState = nova_cull_state(cull);
                let kept: i64 = nova_cull_state_kept(state);
                return kept;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaCullState" && type_name == "NovaCullState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_lod_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let lod: NovaLodPacket = nova_lod_packet(3, 4, 1, 9, 2);
                let state: NovaLodState = nova_lod_state(lod);
                let active: i64 = nova_lod_state_active(state);
                return active;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaLodState" && type_name == "NovaLodState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_streaming_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let streaming: NovaStreamingPacket = nova_streaming_packet(3, 2, 1, 6, 2);
                let state: NovaStreamingState = nova_streaming_state(streaming);
                let resident: i64 = nova_streaming_state_resident(state);
                return resident;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaStreamingState" && type_name == "NovaStreamingState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_residency_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let residency: NovaResidencyPacket = nova_residency_packet(3, 2, 1, 6, 7);
                let state: NovaResidencyState = nova_residency_state(residency);
                let committed: i64 = nova_residency_state_committed(state);
                return committed;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaResidencyState" && type_name == "NovaResidencyState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_eviction_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let eviction: NovaEvictionPacket = nova_eviction_packet(3, 1, 1, 5, 6);
                let state: NovaEvictionState = nova_eviction_state(eviction);
                let evicted: i64 = nova_eviction_state_evicted(state);
                return evicted;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaEvictionState" && type_name == "NovaEvictionState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_prefetch_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let prefetch: NovaPrefetchPacket = nova_prefetch_packet(3, 2, 1, 5, 5);
                let state: NovaPrefetchState = nova_prefetch_state(prefetch);
                let requested: i64 = nova_prefetch_state_requested(state);
                return requested;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaPrefetchState" && type_name == "NovaPrefetchState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_budget_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let budget: NovaBudgetPacket = nova_budget_packet(3, 12, 7, 5, 1);
                let state: NovaBudgetState = nova_budget_state(budget);
                let total: i64 = nova_budget_state_total(state);
                return total;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaBudgetState" && type_name == "NovaBudgetState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_pressure_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let pressure: NovaPressurePacket = nova_pressure_packet(3, 2, 7, 1, 6);
                let state: NovaPressureState = nova_pressure_state(pressure);
                let level: i64 = nova_pressure_state_level(state);
                return level;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaPressureState" && type_name == "NovaPressureState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_thermal_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let thermal: NovaThermalPacket = nova_thermal_packet(3, 2, 1, 1, 6);
                let state: NovaThermalState = nova_thermal_state(thermal);
                let level: i64 = nova_thermal_state_level(state);
                return level;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaThermalState" && type_name == "NovaThermalState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_power_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let power: NovaPowerPacket = nova_power_packet(3, 2, 1, 1, 6);
                let state: NovaPowerState = nova_power_state(power);
                let level: i64 = nova_power_state_level(state);
                return level;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaPowerState" && type_name == "NovaPowerState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_latency_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let latency: NovaLatencyPacket = nova_latency_packet(3, 4, 2, 1, 7);
                let state: NovaLatencyState = nova_latency_state(latency);
                let frame: i64 = nova_latency_state_frame(state);
                return frame;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaLatencyState" && type_name == "NovaLatencyState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_frame_pacing_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let pacing: NovaFramePacingPacket = nova_frame_pacing_packet(3, 4, 1, 1, 7);
                let state: NovaFramePacingState = nova_frame_pacing_state(pacing);
                let cadence: i64 = nova_frame_pacing_state_cadence(state);
                return cadence;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaFramePacingState" && type_name == "NovaFramePacingState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_jank_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let jank: NovaJankPacket = nova_jank_packet(3, 2, 1, 4, 7);
                let state: NovaJankState = nova_jank_state(jank);
                let spikes: i64 = nova_jank_state_spikes(state);
                return spikes;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaJankState" && type_name == "NovaJankState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_frame_variance_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let variance: NovaFrameVariancePacket = nova_frame_variance_packet(3, 2, 1, 4, 7);
                let state: NovaFrameVarianceState = nova_frame_variance_state(variance);
                let frame: i64 = nova_frame_variance_state_frame(state);
                return frame;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaFrameVarianceState" && type_name == "NovaFrameVarianceState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_pass_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let pass: NovaPassPacket = nova_pass_packet(1, 8, 4, 2);
                let state: NovaPassState = nova_pass_state(pass);
                let samples: i64 = nova_pass_state_sample_count(state);
                return samples;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaPassState" && type_name == "NovaPassState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_frame_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let frame: NovaFramePacket = nova_frame_packet(7, 1, 1, 9);
                let state: NovaFrameState = nova_frame_state(frame);
                let exposure: i64 = nova_frame_state_exposure(state);
                return exposure;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaFrameState" && type_name == "NovaFrameState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_target_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let target: NovaTargetPacket = nova_target_packet(1, 48, 18, 8);
                let state: NovaTargetState = nova_target_state(target);
                let msaa: i64 = nova_target_state_multisample(state);
                return msaa;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaTargetState" && type_name == "NovaTargetState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_frame_graph_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let frame_graph: NovaFrameGraphPacket = nova_frame_graph_packet(2, 1, 1, 2);
                let state: NovaFrameGraphState = nova_frame_graph_state(frame_graph);
                let passes: i64 = nova_frame_graph_state_passes(state);
                return passes;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaFrameGraphState" && type_name == "NovaFrameGraphState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_attachment_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let attachment: NovaAttachmentPacket = nova_attachment_packet(0, 8, 1, 1);
                let state: NovaAttachmentState = nova_attachment_state(attachment);
                let format_kind: i64 = nova_attachment_state_format_kind(state);
                return format_kind;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaAttachmentState" && type_name == "NovaAttachmentState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_pass_chain_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let pass_chain: NovaPassChainPacket = nova_pass_chain_packet(2, 1, 1, 8);
                let state: NovaPassChainState = nova_pass_chain_state(pass_chain);
                let stages: i64 = nova_pass_chain_state_stages(state);
                return stages;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaPassChainState" && type_name == "NovaPassChainState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_barrier_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let barrier: NovaBarrierPacket = nova_barrier_packet(1, 1, 2, 8);
                let state: NovaBarrierState = nova_barrier_state(barrier);
                let flush_mode: i64 = nova_barrier_state_flush_mode(state);
                return flush_mode;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaBarrierState" && type_name == "NovaBarrierState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_resource_set_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let resource_set: NovaResourceSetPacket = nova_resource_set_packet(2, 1, 1, 8);
                let state: NovaResourceSetState = nova_resource_set_state(resource_set);
                let residency: i64 = nova_resource_set_state_residency(state);
                return residency;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaResourceSetState" && type_name == "NovaResourceSetState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_schedule_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let schedule: NovaSchedulePacket = nova_schedule_packet(2, 4, 9, 1);
                let state: NovaScheduleState = nova_schedule_state(schedule);
                let budget: i64 = nova_schedule_state_async_budget(state);
                return budget;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaScheduleState" && type_name == "NovaScheduleState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_submission_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let submission: NovaSubmissionPacket = nova_submission_packet(2, 1, 1, 8);
                let state: NovaSubmissionState = nova_submission_state(submission);
                let batches: i64 = nova_submission_state_batches(state);
                return batches;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSubmissionState" && type_name == "NovaSubmissionState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_queue_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let queue: NovaQueuePacket = nova_queue_packet(1, 2, 9, 1);
                let state: NovaQueueState = nova_queue_state(queue);
                let budget: i64 = nova_queue_state_budget(state);
                return budget;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaQueueState" && type_name == "NovaQueueState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_semaphore_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let semaphore: NovaSemaphorePacket = nova_semaphore_packet(1, 2, 1, 3);
                let state: NovaSemaphoreState = nova_semaphore_state(semaphore);
                let scope: i64 = nova_semaphore_state_scope(state);
                return scope;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSemaphoreState" && type_name == "NovaSemaphoreState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_timeline_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let timeline: NovaTimelinePacket = nova_timeline_packet(9, 1, 0, 3);
                let state: NovaTimelineState = nova_timeline_state(timeline);
                let epoch: i64 = nova_timeline_state_epoch(state);
                return epoch;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaTimelineState" && type_name == "NovaTimelineState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_fence_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let fence: NovaFencePacket = nova_fence_packet(1, 0, 3, 1);
                let state: NovaFenceState = nova_fence_state(fence);
                let scope: i64 = nova_fence_state_scope(state);
                return scope;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaFenceState" && type_name == "NovaFenceState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_signal_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let signal: NovaSignalPacket = nova_signal_packet(1, 2, 3, 4);
                let state: NovaSignalState = nova_signal_state(signal);
                let phase: i64 = nova_signal_state_phase(state);
                return phase;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSignalState" && type_name == "NovaSignalState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_event_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let event: NovaEventPacket = nova_event_packet(1, 2, 3, 4);
                let state: NovaEventState = nova_event_state(event);
                let route: i64 = nova_event_state_route(state);
                return route;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaEventState" && type_name == "NovaEventState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_dispatch_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let dispatch: NovaDispatchPacket = nova_dispatch_packet(1, 2, 3, 4);
                let state: NovaDispatchState = nova_dispatch_state(dispatch);
                let queue_kind: i64 = nova_dispatch_state_queue_kind(state);
                return queue_kind;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaDispatchState" && type_name == "NovaDispatchState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_feedback_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let feedback: NovaFeedbackPacket = nova_feedback_packet(1, 2, 3, 4);
                let state: NovaFeedbackState = nova_feedback_state(feedback);
                let status: i64 = nova_feedback_state_status(state);
                return status;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaFeedbackState" && type_name == "NovaFeedbackState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_intent_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let intent: NovaIntentPacket = nova_intent_packet(1, 2, 3, 4);
                let state: NovaIntentState = nova_intent_state(intent);
                let target_slot: i64 = nova_intent_state_target(state);
                return target_slot;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaIntentState" && type_name == "NovaIntentState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_reaction_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let reaction: NovaReactionPacket = nova_reaction_packet(1, 2, 3, 4);
                let state: NovaReactionState = nova_reaction_state(reaction);
                let result_slot: i64 = nova_reaction_state_result(state);
                return result_slot;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaReactionState" && type_name == "NovaReactionState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_outcome_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let outcome: NovaOutcomePacket = nova_outcome_packet(1, 2, 3, 4);
                let state: NovaOutcomeState = nova_outcome_state(outcome);
                let final_slot: i64 = nova_outcome_state_final(state);
                return final_slot;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaOutcomeState" && type_name == "NovaOutcomeState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_resolution_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let resolution: NovaResolutionPacket = nova_resolution_packet(1, 2, 3, 4);
                let state: NovaResolutionState = nova_resolution_state(resolution);
                let commit_slot: i64 = nova_resolution_state_commit(state);
                return commit_slot;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaResolutionState" && type_name == "NovaResolutionState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_commit_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let commit: NovaCommitPacket = nova_commit_packet(1, 2, 3, 4);
                let state: NovaCommitState = nova_commit_state(commit);
                let applied_slot: i64 = nova_commit_state_applied(state);
                return applied_slot;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaCommitState" && type_name == "NovaCommitState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_snapshot_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let snapshot: NovaSnapshotPacket = nova_snapshot_packet(1, 2, 3, 4);
                let state: NovaSnapshotState = nova_snapshot_state(snapshot);
                let source_slot: i64 = nova_snapshot_state_source(state);
                return source_slot;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaSnapshotState" && type_name == "NovaSnapshotState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_checkpoint_state_contract() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let checkpoint: NovaCheckpointPacket = nova_checkpoint_packet(1, 2, 3, 4);
                let state: NovaCheckpointState = nova_checkpoint_state(checkpoint);
                let anchor_slot: i64 = nova_checkpoint_state_anchor(state);
                return anchor_slot;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaCheckpointState" && type_name == "NovaCheckpointState",
            _ => false,
        }));
    }

    #[test]
    fn lowers_nova_panel_from_parts_builder() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let header: NovaHeaderPacket = nova_header_packet(8);
                let slider_color: NovaSliderPacket = nova_slider_packet(1);
                let slider_speed: NovaSliderPacket = nova_slider_packet(2);
                let slider_radius: NovaSliderPacket = nova_slider_packet(3);
                let sliders: NovaSliderGroupPacket =
                  nova_slider_group_packet(slider_color, slider_speed, slider_radius);
                let toggle: NovaTogglePacket = nova_toggle_packet(1);
                let progress: NovaProgressPacket = nova_progress_packet(2);
                let meter: NovaMeterPacket = nova_meter_packet(3);
                let button: NovaButtonPacket = nova_button_packet(1, 8);
                let text_input: NovaTextInputPacket = nova_text_input_packet(4, 1);
                let select: NovaSelectPacket = nova_select_packet(0, 8);
                let checkbox: NovaCheckboxPacket = nova_checkbox_packet(1, 8);
                let radio: NovaRadioPacket = nova_radio_packet(1, 4, 8);
                let textarea: NovaTextAreaPacket = nova_textarea_packet(3, 1);
                let tabs: NovaTabsPacket = nova_tabs_packet(0, 4, 8);
                let list: NovaListPacket = nova_list_packet(1, 5, 8);
                let table: NovaTablePacket = nova_table_packet(4, 3, 1);
                let tree: NovaTreePacket = nova_tree_packet(1, 6, 1, 8);
                let inspector: NovaInspectorPacket = nova_inspector_packet(1, 4, 1, 8);
                let outline: NovaOutlinePacket = nova_outline_packet(1, 6, 1, 8);
                let theme: NovaThemePacket = nova_theme_packet(8, 3, 1, 2);
                let surface: NovaSurfacePacket = nova_surface_packet(3, 2, 1, 4);
                let viewport: NovaViewportPacket = nova_viewport_packet(2, 1, 48, 18);
                let layer: NovaLayerPacket = nova_layer_packet(1, 2, 1, 0);
                let scene: NovaScenePacket = nova_scene_packet(7, 2, 3, 1);
                let camera: NovaCameraPacket = nova_camera_packet(1, 2, 12, 9);
                let material: NovaMaterialPacket = nova_material_packet(1, 8, 3, 2);
                let light: NovaLightPacket = nova_light_packet(1, 12, 9, 8);
                let mesh: NovaMeshPacket = nova_mesh_packet(1, 12, 9, 8);
                let transform: NovaTransformPacket = nova_transform_packet(12, 1, 9, 2);
                let node: NovaNodePacket = nova_node_packet(2, 1, 8, 2);
                let scene_link: NovaSceneLinkPacket = nova_scene_link_packet(2, 12, 9, 8, 1, 1);
                let instance: NovaInstancePacket = nova_instance_packet(2, 3, 2, 1, 8, 1);
                let scene_graph: NovaSceneGraphPacket = nova_scene_graph_packet(2, 6, 3, 3, 1);
                let scene_node: NovaSceneNodePacket = nova_scene_node_packet(2, 4, 5, 3, 1);
                let instance_group: NovaInstanceGroupPacket = nova_instance_group_packet(3, 4, 3, 1, 8);
                let scene_cluster: NovaSceneClusterPacket = nova_scene_cluster_packet(2, 6, 3, 8, 1);
                let visibility: NovaVisibilityPacket = nova_visibility_packet(3, 5, 1, 2, 7);
                let cull: NovaCullPacket = nova_cull_packet(3, 4, 1, 2, 7);
                let lod: NovaLodPacket = nova_lod_packet(3, 4, 1, 9, 2);
                let streaming: NovaStreamingPacket = nova_streaming_packet(3, 2, 1, 6, 2);
                let residency: NovaResidencyPacket = nova_residency_packet(3, 2, 1, 6, 7);
                let eviction: NovaEvictionPacket = nova_eviction_packet(3, 1, 1, 5, 6);
                let prefetch: NovaPrefetchPacket = nova_prefetch_packet(3, 2, 1, 5, 5);
                let budget: NovaBudgetPacket = nova_budget_packet(3, 12, 7, 5, 1);
                let pressure: NovaPressurePacket = nova_pressure_packet(3, 2, 7, 1, 6);
                let thermal: NovaThermalPacket = nova_thermal_packet(3, 2, 1, 1, 6);
                let power: NovaPowerPacket = nova_power_packet(3, 2, 1, 1, 6);
                let latency: NovaLatencyPacket = nova_latency_packet(3, 4, 2, 1, 7);
                let frame_pacing: NovaFramePacingPacket = nova_frame_pacing_packet(3, 4, 1, 1, 7);
                let frame_variance: NovaFrameVariancePacket = nova_frame_variance_packet(3, 2, 1, 4, 7);
                let jank: NovaJankPacket = nova_jank_packet(3, 2, 1, 4, 7);
                let pass: NovaPassPacket = nova_pass_packet(1, 8, 4, 2);
                let frame: NovaFramePacket = nova_frame_packet(7, 1, 1, 9);
                let target: NovaTargetPacket = nova_target_packet(1, 48, 18, 8);
                let frame_graph: NovaFrameGraphPacket = nova_frame_graph_packet(2, 1, 1, 2);
                let attachment: NovaAttachmentPacket = nova_attachment_packet(0, 8, 1, 1);
                let pass_chain: NovaPassChainPacket = nova_pass_chain_packet(2, 1, 1, 8);
                let barrier: NovaBarrierPacket = nova_barrier_packet(1, 1, 2, 8);
                let resource_set: NovaResourceSetPacket = nova_resource_set_packet(2, 1, 1, 8);
                let schedule: NovaSchedulePacket = nova_schedule_packet(2, 4, 9, 1);
                let submission: NovaSubmissionPacket = nova_submission_packet(2, 1, 1, 8);
                let queue: NovaQueuePacket = nova_queue_packet(1, 2, 9, 1);
                let semaphore: NovaSemaphorePacket = nova_semaphore_packet(1, 2, 1, 3);
                let timeline: NovaTimelinePacket = nova_timeline_packet(9, 1, 0, 3);
                let fence: NovaFencePacket = nova_fence_packet(1, 0, 3, 1);
                let signal: NovaSignalPacket = nova_signal_packet(1, 2, 3, 1);
                let event: NovaEventPacket = nova_event_packet(1, 2, 3, 1);
                let dispatch: NovaDispatchPacket = nova_dispatch_packet(1, 2, 3, 1);
                let feedback: NovaFeedbackPacket = nova_feedback_packet(1, 2, 3, 1);
                let intent: NovaIntentPacket = nova_intent_packet(1, 2, 3, 1);
                let reaction: NovaReactionPacket = nova_reaction_packet(1, 2, 3, 1);
                let outcome: NovaOutcomePacket = nova_outcome_packet(1, 2, 3, 1);
                let resolution: NovaResolutionPacket = nova_resolution_packet(1, 2, 3, 1);
                let commit: NovaCommitPacket = nova_commit_packet(1, 2, 3, 1);
                let snapshot: NovaSnapshotPacket = nova_snapshot_packet(1, 2, 3, 1);
                let checkpoint: NovaCheckpointPacket = nova_checkpoint_packet(1, 2, 3, 1);
                let focus: NovaFocusPacket = nova_focus_packet(2);
                let panel: NovaPanelPacket = nova_panel_from_parts(
                  header,
                  sliders,
                  toggle,
                  progress,
                  meter,
                  button,
                  text_input,
                  select,
                  checkbox,
                  radio,
                  textarea,
                  tabs,
                  list,
                  table,
                  tree,
                  inspector,
                  outline,
                  theme,
                  surface,
                  viewport,
                  layer,
                  scene,
                  camera,
                  material,
                  light,
                  mesh,
                  transform,
                  node,
                  scene_link,
                  instance,
                  scene_graph,
                      scene_node,
                      instance_group,
                      scene_cluster,
                      visibility,
                  cull,
                        lod,
                  streaming,
                  residency,
                  eviction,
                  prefetch,
                  budget,
                  pressure,
                  thermal,
                  power,
                  latency,
                  frame_pacing,
                  frame_variance,
                  jank,
                  pass,
                  frame,
                  target,
                  frame_graph,
                  attachment,
                  pass_chain,
                  barrier,
                  resource_set,
                  schedule,
                  submission,
                  queue,
                  semaphore,
                  timeline,
                  fence,
                  signal,
                  event,
                  dispatch,
                  feedback,
                  intent,
                  reaction,
                  outcome,
                  resolution,
                  commit,
                  snapshot,
                  checkpoint,
                  focus
                );
                return 1;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::StructLiteral { type_name, .. },
                ..
            } => ty.render() == "NovaPanelPacket" && type_name == "NovaPanelPacket",
            _ => false,
        }));
    }

    #[test]
    fn lowers_explicit_kernel_result_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let lanes: KernelResult<i64> = kernel_result(kernel_profile_batch_lanes("KernelUnit"));
                let ready: bool = kernel_config_ready(lanes);
                let value: i64 = kernel_value(lanes);
                return value;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::KernelResult { state, .. },
                ..
            }) if ty.render() == "KernelResult<i64>"
                && matches!(state, NirKernelFlowState::ConfigReady)
        ));
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::KernelConfigReady(_),
                ..
            }) if ty.render() == "bool"
        ));
        assert!(matches!(
            function.body.get(2),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::KernelValue(_),
                ..
            }) if ty.render() == "i64"
        ));
    }

    #[test]
    fn lowers_explicit_kernel_result_helpers_from_tensor_reductions() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let total: KernelResult<i64> = kernel_result(kernel_reduce_sum(input));
                let peak: KernelResult<i64> = kernel_result(kernel_reduce_max(input));
                let avg: KernelResult<i64> = kernel_result(kernel_reduce_mean(input));
                return kernel_value(total) + kernel_value(peak) + kernel_value(avg);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::KernelResult { value, state },
                ..
            } => {
                ty.render() == "KernelResult<i64>"
                    && matches!(state, NirKernelFlowState::ConfigReady)
                    && matches!(value.as_ref(), NirExpr::KernelReduceSum(_))
            }
            _ => false,
        }));
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                value: NirExpr::KernelResult { value, .. },
                ..
            } => matches!(value.as_ref(), NirExpr::KernelReduceMax(_)),
            _ => false,
        }));
        assert!(function.body.iter().any(|stmt| match stmt {
            NirStmt::Let {
                value: NirExpr::KernelResult { value, .. },
                ..
            } => matches!(value.as_ref(), NirExpr::KernelReduceMean(_)),
            _ => false,
        }));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(1, 3, "2,4,6");
                let weights = kernel_tensor(3, 2, "1,-2,3,0,2,1");
                let bias = kernel_tensor(1, 2, "-4,3");
                let projected = kernel_matmul(input, weights);
                let shifted = kernel_add_bias(projected, bias);
                let activated = kernel_relu(shifted);
                return kernel_reduce_sum(activated);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelTensor { .. },
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelMatmul { .. },
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelAddBias { .. },
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelRelu(_),
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::KernelReduceSum(_))))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_inspect_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(1, 3, "2,4,6");
                let layout = kernel_shape(input);
                let rows: i64 = kernel_rows(input);
                let cols: i64 = kernel_cols(input);
                let first_row = kernel_row(input);
                let first_col = kernel_col(input);
                return kernel_element_at(first_row, 0, 1) + rows + cols + kernel_element_at(first_col, 0, 0);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelShape(_),
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelRows(_),
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelCols(_),
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelRow(_),
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelCol(_),
                ..
            }
        )));
        assert!(function
            .body
            .iter()
            .any(|stmt| matches!(stmt, NirStmt::Return(Some(NirExpr::Binary { .. })))));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_map_zip_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(1, 3, "2,4,6");
                let lifted = kernel_map(input, "add_scalar", 3);
                let scaled = kernel_map(lifted, "mul_scalar", 2);
                let activated = kernel_map(scaled, "relu");
                let mask = kernel_tensor(1, 3, "1,0,1");
                let mixed = kernel_zip(activated, mask, "mul");
                return kernel_reduce_sum(mixed);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelMap { .. },
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelZip { .. },
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::KernelReduceSum(_))))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_reshape_helper() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let reshaped = kernel_reshape(input, 3, 2);
                return kernel_element_at(reshaped, 2, 1);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelReshape { .. },
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::KernelElementAt { .. })))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_broadcast_helper() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(1, 3, "2,4,6");
                let widened = kernel_broadcast(input, 2, 3);
                return kernel_element_at(widened, 1, 2);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelBroadcast { .. },
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::KernelElementAt { .. })))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_reduction_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let maxed: i64 = kernel_reduce_max(input);
                return maxed + kernel_reduce_mean(input);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelReduceMax(_),
                ..
            }
        )));
        assert!(function
            .body
            .iter()
            .any(|stmt| matches!(stmt, NirStmt::Return(Some(NirExpr::Binary { .. })))));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_selection_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let hi: i64 = kernel_argmax(input);
                return hi + kernel_argmin(input);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelArgmax(_),
                ..
            }
        )));
        assert!(function
            .body
            .iter()
            .any(|stmt| matches!(stmt, NirStmt::Return(Some(NirExpr::Binary { .. })))));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_reduce_axis_helper() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let row_sums = kernel_reduce_sum_axis(input, "rows");
                return kernel_element_at(row_sums, 0, 1);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelReduceSumAxis { .. },
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::KernelElementAt { .. })))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_reduce_axis_family_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let row_max = kernel_reduce_max_axis(input, "rows");
                let col_mean = kernel_reduce_mean_axis(input, "cols");
                return kernel_element_at(row_max, 0, 0) + kernel_element_at(col_mean, 0, 1);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelReduceMaxAxis { .. },
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelReduceMeanAxis { .. },
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::Binary { .. })))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_select_axis_family_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let row_hi = kernel_argmax_axis(input, "rows");
                let col_lo = kernel_argmin_axis(input, "cols");
                return kernel_element_at(row_hi, 0, 1) + kernel_element_at(col_lo, 0, 2);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelArgmaxAxis { .. },
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelArgminAxis { .. },
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::Binary { .. })))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_topk_axis_helper() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let top2_rows = kernel_topk_axis(input, "rows", 2);
                return kernel_element_at(top2_rows, 0, 1);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelTopkAxis { .. },
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::KernelElementAt { .. })))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_map_axis_helper() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "-2,4,-6,1,-3,5");
                let activated = kernel_map_axis(input, "rows", "relu");
                let lifted = kernel_map_axis(activated, "cols", "add_scalar", 2);
                return kernel_element_at(lifted, 0, 0);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelMapAxis { .. },
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::KernelElementAt { .. })))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_sort_axis_helper() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let sorted_rows = kernel_sort_axis(input, "rows");
                return kernel_element_at(sorted_rows, 0, 1);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelSortAxis { .. },
                ..
            }
        )));
        assert!(matches!(
            function.body.last(),
            Some(NirStmt::Return(Some(NirExpr::KernelElementAt { .. })))
        ));
    }

    #[test]
    fn lowers_explicit_kernel_tensor_order_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              fn main() -> i64 {
                let input = kernel_tensor(2, 3, "2,4,6,1,3,5");
                let sorted = kernel_sort(input);
                let top2 = kernel_topk(input, 2);
                return kernel_element_at(sorted, 0, 0) + kernel_element_at(top2, 0, 1);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelSort(_),
                ..
            }
        )));
        assert!(function.body.iter().any(|stmt| matches!(
            stmt,
            NirStmt::Let {
                value: NirExpr::KernelTopk { .. },
                ..
            }
        )));
        assert!(function
            .body
            .iter()
            .any(|stmt| matches!(stmt, NirStmt::Return(Some(NirExpr::Binary { .. })))));
    }

    #[test]
    fn lowers_explicit_timeout_on_task_handle() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let task: Task<i64> = timeout(spawn(ping()), 16);
                return join(task);
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.first(),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::CpuTimeout { .. },
                ..
            }) if ty.render() == "Task<i64>"
        ));
    }

    #[test]
    fn lowers_explicit_join_result_and_task_state_helpers() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let task: Task<i64> = timeout(spawn(ping()), 16);
                let result: TaskResult<i64> = join_result(task);
                if task_completed(result) {
                  return task_value(result);
                }
                return 0;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "main")
            .unwrap();
        assert!(matches!(
            function.body.get(1),
            Some(NirStmt::Let {
                ty: Some(ty),
                value: NirExpr::CpuJoinResult(_),
                ..
            }) if ty.render() == "TaskResult<i64>"
        ));
    }

    #[test]
    fn rejects_timeout_with_non_integer_limit() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              fn main() -> i64 {
                let task: Task<i64> = timeout(spawn(ping()), "slow");
                return join(task);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("expects integer limit"));
    }

    #[test]
    fn rejects_await_inside_sync_function() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              fn ping() -> i64 {
                return 7;
              }

              fn main() {
                await ping();
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("`await`"));
        assert!(error.contains("async fn"));
    }

    #[test]
    fn rejects_async_function_returning_ref_type() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn head() -> ref Node {
                return null();
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("cannot return"));
        assert!(error.contains("ref Node"));
        assert!(error.contains("async boundary"));
    }

    #[test]
    fn rejects_async_function_returning_result_family() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn main() -> DataResult<i64> {
                return data_result(data_input_pipe(data_output_pipe(7)));
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("DataResult<i64>"));
        assert!(error.contains("async boundary"));
    }

    #[test]
    fn rejects_async_function_taking_instance_param() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn render(shader: Instance<SurfaceShader>) {
                return;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("parameter `shader`"));
        assert!(error.contains("Instance<SurfaceShader>"));
        assert!(error.contains("async boundary"));
    }

    #[test]
    fn accepts_async_function_taking_shader_result_family_param() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn consume(result: ShaderResult<Frame>) -> i64 {
                if shader_frame_ready(result) {
                  return 1;
                }
                return 0;
              }

              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "consume")
            .unwrap();
        assert_eq!(function.params[0].ty.render(), "ShaderResult<Frame>");
    }

    #[test]
    fn accepts_async_function_taking_kernel_result_family_param() {
        let module = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn consume(result: KernelResult<i64>) -> i64 {
                if kernel_config_ready(result) {
                  return kernel_value(result);
                }
                return 0;
              }

              fn main() -> i64 {
                return 0;
              }
            }
            "#,
        )
        .unwrap();

        let function = module
            .functions
            .iter()
            .find(|function| function.name == "consume")
            .unwrap();
        assert_eq!(function.params[0].ty.render(), "KernelResult<i64>");
    }

    #[test]
    fn rejects_async_function_taking_struct_with_nested_ref_field() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              struct RefPacket {
                head: ref Node
              }

              async fn consume(packet: RefPacket) -> i64 {
                return 7;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("parameter `packet`"));
        assert!(error.contains("RefPacket"));
        assert!(error.contains("nested field `RefPacket.head`"));
        assert!(error.contains("ref Node"));
        assert!(error.contains("async boundary"));
    }

    #[test]
    fn rejects_async_function_returning_struct_with_nested_ref_field() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              struct RefPacket {
                head: ref Node
              }

              async fn emit() -> RefPacket {
                return RefPacket { head: null() };
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("cannot return `RefPacket` across async boundary"));
        assert!(error.contains("nested field `RefPacket.head`"));
        assert!(error.contains("ref Node"));
    }

    #[test]
    fn rejects_async_function_taking_struct_with_nested_optional_field() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              struct OptionalPacket {
                value: i64?
              }

              async fn consume(packet: OptionalPacket) -> i64 {
                return 7;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("parameter `packet`"));
        assert!(error.contains("OptionalPacket"));
        assert!(error.contains("nested field `OptionalPacket.value`"));
        assert!(error.contains("i64?"));
    }

    #[test]
    fn rejects_async_function_taking_struct_with_nested_instance_field() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              struct ShaderPacket {
                shader: Instance<SurfaceShader>
              }

              async fn consume(packet: ShaderPacket) -> i64 {
                return 7;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("parameter `packet`"));
        assert!(error.contains("ShaderPacket"));
        assert!(error.contains("nested field `ShaderPacket.shader`"));
        assert!(error.contains("Instance<SurfaceShader>"));
    }

    #[test]
    fn rejects_async_function_taking_struct_with_nested_result_field() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              struct ResultPacket {
                result: TaskResult<i64>
              }

              async fn consume(packet: ResultPacket) -> i64 {
                return 7;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("parameter `packet`"));
        assert!(error.contains("ResultPacket"));
        assert!(error.contains("nested field `ResultPacket.result`"));
        assert!(error.contains("TaskResult<i64>"));
    }

    #[test]
    fn rejects_async_function_taking_window_param() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn consume(window: Window<i64>) -> i64 {
                return 7;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("parameter `window`"));
        assert!(error.contains("Window<i64>"));
        assert!(error.contains("resource-bearing"));
    }

    #[test]
    fn rejects_async_function_taking_struct_with_nested_marker_field() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              struct MarkerPacket {
                ready: Marker<CpuToShader>
              }

              async fn consume(packet: MarkerPacket) -> i64 {
                return 7;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("parameter `packet`"));
        assert!(error.contains("MarkerPacket"));
        assert!(error.contains("nested field `MarkerPacket.ready`"));
        assert!(error.contains("resource-bearing `Marker<CpuToShader>`"));
    }

    #[test]
    fn allows_async_function_taking_nested_scalar_struct_payload() {
        parse_nuis_module(
            r#"
            mod cpu Main {
              struct ScalarPair {
                lhs: i64,
                rhs: i64
              }

              struct NestedPacket {
                pair: ScalarPair,
                bias: i64
              }

              async fn add(packet: NestedPacket) -> i64 {
                return packet.pair.lhs + packet.pair.rhs + packet.bias;
              }
            }
            "#,
        )
        .unwrap();
    }

    #[test]
    fn allows_async_function_taking_nested_text_struct_payload() {
        parse_nuis_module(
            r#"
            mod cpu Main {
              struct MessagePacket {
                message: String
              }

              struct LabeledMessage {
                packet: MessagePacket,
                label: String
              }

              async fn show(input: LabeledMessage) -> i64 {
                return 5;
              }
            }
            "#,
        )
        .unwrap();
    }

    #[test]
    fn rejects_async_shader_function_for_now() {
        let error = parse_nuis_module(
            r#"
            mod shader SurfaceShader {
              async fn profile() {
                return;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("mod shader SurfaceShader"));
        assert!(error.contains("async fn profile"));
        assert!(error.contains("only supported in `mod cpu`"));
    }

    #[test]
    fn rejects_async_data_function_for_now() {
        let error = parse_nuis_module(
            r#"
            mod data FabricPlane {
              async fn profile() {
                return;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("mod data FabricPlane"));
        assert!(error.contains("only supported in `mod cpu`"));
    }

    #[test]
    fn rejects_async_kernel_function_for_now() {
        let error = parse_nuis_module(
            r#"
            mod kernel KernelUnit {
              async fn profile() {
                return;
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("mod kernel KernelUnit"));
        assert!(error.contains("only supported in `mod cpu`"));
    }

    #[test]
    fn rejects_async_main_with_parameters() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn main(seed: i64) {
                print(seed);
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("async entry"));
        assert!(error.contains("Main::main"));
        assert!(error.contains("cannot take parameters"));
    }

    #[test]
    fn rejects_async_call_without_await() {
        let error = parse_nuis_module(
            r#"
            mod cpu Main {
              async fn ping() -> i64 {
                return 7;
              }

              async fn main() -> i64 {
                return ping();
              }
            }
            "#,
        )
        .unwrap_err();

        assert!(error.contains("must be used under `await`"));
    }
}
