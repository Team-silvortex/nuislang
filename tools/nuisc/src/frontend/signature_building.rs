use std::collections::BTreeMap;

use nuis_semantics::model::{AstFunction, AstModule, AstTypeAlias, NirTypeRef};

use super::{
    extern_function_symbol_name, function_host_symbol_name, is_public_visibility,
    lower_type_ref_with_aliases,
};

fn is_helper_internal_synthetic(function: &AstFunction) -> bool {
    function.name.starts_with("__hof_") || function.name.starts_with("__lambda_")
}

#[derive(Clone)]
pub(super) struct FunctionSignature {
    pub(super) abi: String,
    pub(super) interface: Option<String>,
    pub(super) symbol_name: String,
    pub(super) params: Vec<NirTypeRef>,
    pub(super) return_type: Option<NirTypeRef>,
    pub(super) is_extern: bool,
    pub(super) is_async: bool,
}

pub(super) struct InitialFunctionSignatures {
    pub(super) signatures: BTreeMap<String, FunctionSignature>,
    pub(super) generic_templates: BTreeMap<String, AstFunction>,
    pub(super) concrete_module_functions: Vec<AstFunction>,
}

pub(super) fn build_initial_function_signatures(
    module: &AstModule,
    local_cpu_helpers: &[&AstModule],
    visible_type_aliases: &BTreeMap<String, AstTypeAlias>,
) -> Result<InitialFunctionSignatures, String> {
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
                    .map(|param| lower_type_ref_with_aliases(&param.ty, visible_type_aliases))
                    .collect::<Result<Vec<_>, _>>()?,
                return_type: Some(lower_type_ref_with_aliases(
                    &function.return_type,
                    visible_type_aliases,
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
                        .map(|param| lower_type_ref_with_aliases(&param.ty, visible_type_aliases))
                        .collect::<Result<Vec<_>, _>>()?,
                    return_type: Some(lower_type_ref_with_aliases(
                        &function.return_type,
                        visible_type_aliases,
                    )?),
                    is_extern: true,
                    is_async: false,
                },
            );
        }
    }

    for helper in local_cpu_helpers {
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
                    .map(|param| lower_type_ref_with_aliases(&param.ty, visible_type_aliases))
                    .collect::<Result<Vec<_>, _>>()?,
                return_type: Some(lower_type_ref_with_aliases(
                    &function.return_type,
                    visible_type_aliases,
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
                        .map(|param| lower_type_ref_with_aliases(&param.ty, visible_type_aliases))
                        .collect::<Result<Vec<_>, _>>()?,
                    return_type: Some(lower_type_ref_with_aliases(
                        &function.return_type,
                        visible_type_aliases,
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
                .map(|param| lower_type_ref_with_aliases(&param.ty, visible_type_aliases))
                .collect::<Result<Vec<_>, _>>()?,
            return_type: function
                .return_type
                .as_ref()
                .map(|ty| lower_type_ref_with_aliases(ty, visible_type_aliases))
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

    for helper in local_cpu_helpers {
        for function in helper.functions.iter().filter(|function| {
            is_public_visibility(function.visibility) || is_helper_internal_synthetic(function)
        }) {
            let signature = FunctionSignature {
                abi: "nuis".to_owned(),
                interface: None,
                symbol_name: format!("{}.{}", helper.unit, function.name),
                params: function
                    .params
                    .iter()
                    .map(|param| lower_type_ref_with_aliases(&param.ty, visible_type_aliases))
                    .collect::<Result<Vec<_>, _>>()?,
                return_type: function
                    .return_type
                    .as_ref()
                    .map(|ty| lower_type_ref_with_aliases(ty, visible_type_aliases))
                    .transpose()?,
                is_extern: false,
                is_async: function.is_async,
            };
            signatures.insert(
                format!("{}.{}", helper.unit, function.name),
                signature.clone(),
            );
            signatures.entry(function.name.clone()).or_insert(signature);
            if !function.generic_params.is_empty() {
                generic_templates.insert(function.name.clone(), function.clone());
                let mut qualified = function.clone();
                qualified.name = format!("{}.{}", helper.unit, function.name);
                generic_templates.insert(qualified.name.clone(), qualified);
            }
        }
    }

    Ok(InitialFunctionSignatures {
        signatures,
        generic_templates,
        concrete_module_functions,
    })
}
