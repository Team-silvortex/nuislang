use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{AstFunction, AstImplDef, AstModule, AstStructDef};

#[path = "lambda_expansion_block.rs"]
mod lambda_expansion_block;
#[path = "lambda_expansion_expr.rs"]
mod lambda_expansion_expr;
#[path = "lambda_expansion_synth.rs"]
mod lambda_expansion_synth;
#[path = "lambda_expansion_types.rs"]
mod lambda_expansion_types;

use self::lambda_expansion_block::expand_lambda_block;
use self::lambda_expansion_types::extend_local_field_bindings_from_type;

pub(super) fn expand_module_lambdas(module: &AstModule) -> Result<AstModule, String> {
    let module_const_names = module
        .consts
        .iter()
        .map(|constant| constant.name.clone())
        .collect::<BTreeSet<_>>();
    let module_function_table = module
        .functions
        .iter()
        .map(|function| (function.name.clone(), function.clone()))
        .collect::<BTreeMap<_, _>>();
    let visible_structs = module
        .structs
        .iter()
        .map(|definition| (definition.name.clone(), definition.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut expanded = module.clone();
    expanded.functions.clear();
    for function in &module.functions {
        let (rewritten, synthesized) = expand_function_lambdas(
            function,
            &module.impls,
            &visible_structs,
            &module_const_names,
            &module_function_table,
        )?;
        expanded.functions.extend(synthesized);
        expanded.functions.push(rewritten);
    }
    Ok(expanded)
}

fn expand_function_lambdas(
    function: &AstFunction,
    module_impls: &[AstImplDef],
    visible_structs: &BTreeMap<String, AstStructDef>,
    module_const_names: &BTreeSet<String>,
    module_function_table: &BTreeMap<String, AstFunction>,
) -> Result<(AstFunction, Vec<AstFunction>), String> {
    let mut counter = 0usize;
    let mut synthesized = Vec::new();
    let visible_locals = function
        .params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let mut visible_local_types = function
        .params
        .iter()
        .map(|param| (param.name.clone(), param.ty.clone()))
        .collect::<BTreeMap<_, _>>();
    for param in &function.params {
        extend_local_field_bindings_from_type(
            &param.name,
            &param.ty,
            visible_structs,
            &mut visible_local_types,
        );
    }
    let body = expand_lambda_block(
        &function.body,
        function.return_type.as_ref(),
        &function.generic_params,
        &BTreeMap::new(),
        &visible_locals,
        &visible_local_types,
        module_impls,
        visible_structs,
        module_const_names,
        module_function_table,
        &function.name,
        &mut counter,
        &mut synthesized,
    )?;
    let mut rewritten = function.clone();
    rewritten.body = body;
    Ok((rewritten, synthesized))
}
