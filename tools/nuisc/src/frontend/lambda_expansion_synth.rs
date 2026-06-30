use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{
    AstExpr, AstFunction, AstGenericParam, AstImplDef, AstParam, AstStmt, AstStructDef, AstTypeRef,
    AstVisibility,
};

use super::super::lambda_validation::collect_lambda_block_captures;
use super::expand_lambda_block;
use super::lambda_expansion_types::{
    callable_type_arity, infer_generic_call_substitutions, infer_impl_method_substitutions,
    infer_local_binding_type, specialize_type_with_substitutions, LambdaBinding,
};

pub(super) fn synthesize_lambda_function(
    params: &[AstParam],
    return_type: &Option<AstTypeRef>,
    body: &[AstStmt],
    inherited_generic_params: &[AstGenericParam],
    lambda_aliases: &BTreeMap<String, LambdaBinding>,
    outer_locals: &BTreeSet<String>,
    outer_local_types: &BTreeMap<String, AstTypeRef>,
    module_impls: &[AstImplDef],
    visible_structs: &BTreeMap<String, AstStructDef>,
    module_const_names: &BTreeSet<String>,
    module_function_table: &BTreeMap<String, AstFunction>,
    owning_function_name: &str,
    counter: &mut usize,
    synthesized: &mut Vec<AstFunction>,
) -> Result<LambdaBinding, String> {
    let Some(lambda_return_type) = return_type.clone() else {
        return Err("inline lambda currently requires an explicit return type".to_owned());
    };
    synthesize_lambda_function_with_known_return_type(
        params,
        lambda_return_type,
        body,
        inherited_generic_params,
        lambda_aliases,
        outer_locals,
        outer_local_types,
        module_impls,
        visible_structs,
        module_const_names,
        module_function_table,
        owning_function_name,
        counter,
        synthesized,
    )
}

pub(super) fn synthesize_lambda_function_with_known_return_type(
    params: &[AstParam],
    lambda_return_type: AstTypeRef,
    body: &[AstStmt],
    inherited_generic_params: &[AstGenericParam],
    lambda_aliases: &BTreeMap<String, LambdaBinding>,
    outer_locals: &BTreeSet<String>,
    outer_local_types: &BTreeMap<String, AstTypeRef>,
    module_impls: &[AstImplDef],
    visible_structs: &BTreeMap<String, AstStructDef>,
    module_const_names: &BTreeSet<String>,
    module_function_table: &BTreeMap<String, AstFunction>,
    owning_function_name: &str,
    counter: &mut usize,
    synthesized: &mut Vec<AstFunction>,
) -> Result<LambdaBinding, String> {
    let mut lambda_locals = params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let mut captures = BTreeSet::new();
    collect_lambda_block_captures(body, &mut lambda_locals, outer_locals, &mut captures)?;
    let capture_params = captures
        .iter()
        .map(|capture| {
            let capture_ty = outer_local_types.get(capture).cloned().ok_or_else(|| {
                format!(
                    "captured local `{capture}` currently requires an explicit type annotation before it can be used in a lambda"
                )
            })?;
            Ok(AstParam {
                name: capture.clone(),
                ty: capture_ty,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let synthesized_name = format!("__lambda_{}_{}", owning_function_name, *counter);
    *counter += 1;

    let mut lambda_visible_locals = params
        .iter()
        .map(|param| param.name.clone())
        .collect::<BTreeSet<_>>();
    let mut lambda_visible_local_types = params
        .iter()
        .map(|param| (param.name.clone(), param.ty.clone()))
        .collect::<BTreeMap<_, _>>();
    for capture in &capture_params {
        lambda_visible_locals.insert(capture.name.clone());
        lambda_visible_local_types.insert(capture.name.clone(), capture.ty.clone());
    }

    let lambda_body = expand_lambda_block(
        body,
        Some(&lambda_return_type),
        inherited_generic_params,
        lambda_aliases,
        &lambda_visible_locals,
        &lambda_visible_local_types,
        module_impls,
        visible_structs,
        module_const_names,
        module_function_table,
        owning_function_name,
        counter,
        synthesized,
    )?;
    let mut synthesized_params = params.to_vec();
    synthesized_params.extend(capture_params.clone());
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
        benchmark_name: None,
        benchmark_warmup_iters: None,
        benchmark_measure_iters: None,
        benchmark_timeout_ms: None,
        benchmark_clock_domain: None,
        benchmark_clock_policy: None,
        is_async: false,
        generic_params: inherited_generic_params.to_vec(),
        where_bounds: Vec::new(),
        params: synthesized_params,
        return_type: Some(lambda_return_type),
        body: lambda_body,
    });
    Ok(LambdaBinding {
        symbol: synthesized_name,
        captured_locals: capture_params.into_iter().map(|param| param.name).collect(),
    })
}

pub(super) fn inline_lambda_return_type_from_callable(
    params: &[AstParam],
    explicit_return_type: &Option<AstTypeRef>,
    expected_callable_type: Option<&AstTypeRef>,
) -> Result<Option<AstTypeRef>, String> {
    let Some(expected_callable_type) = expected_callable_type else {
        return Ok(explicit_return_type.clone());
    };
    let Some(arity) = callable_type_arity(expected_callable_type) else {
        return Ok(explicit_return_type.clone());
    };
    if params.len() != arity || expected_callable_type.generic_args.len() != arity + 1 {
        return Ok(explicit_return_type.clone());
    }
    for (param, expected) in params
        .iter()
        .zip(expected_callable_type.generic_args[..arity].iter())
    {
        if param.ty != *expected {
            return Ok(explicit_return_type.clone());
        }
    }
    let inferred_return_type = expected_callable_type.generic_args[arity].clone();
    if let Some(explicit_return_type) = explicit_return_type {
        if *explicit_return_type != inferred_return_type {
            return Err(format!(
                "inline lambda return type `{}` does not match expected callable return type `{}`",
                explicit_return_type.name, inferred_return_type.name
            ));
        }
    }
    Ok(Some(inferred_return_type))
}

pub(super) fn expected_callable_type_for_call_arg(
    callee: &str,
    index: usize,
    generic_args: &[AstTypeRef],
    args: &[AstExpr],
    expected_result_type: Option<&AstTypeRef>,
    visible_local_types: &BTreeMap<String, AstTypeRef>,
    module_function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
) -> Option<AstTypeRef> {
    let function = module_function_table.get(callee)?;
    let param = function.params.get(index)?;
    let specialized = if function.generic_params.is_empty() && generic_args.is_empty() {
        param.ty.clone()
    } else {
        let substitutions = infer_generic_call_substitutions(
            function,
            generic_args,
            args,
            expected_result_type,
            visible_local_types,
            module_function_table,
            module_impls,
        );
        specialize_type_with_substitutions(&param.ty, &substitutions)
    };
    callable_type_arity(&specialized).map(|_| specialized)
}

pub(super) fn expected_callable_type_for_method_arg(
    receiver: &AstExpr,
    method: &str,
    index: usize,
    args: &[AstExpr],
    expected_result_type: Option<&AstTypeRef>,
    visible_local_types: &BTreeMap<String, AstTypeRef>,
    module_function_table: &BTreeMap<String, AstFunction>,
    module_impls: &[AstImplDef],
) -> Option<AstTypeRef> {
    let receiver_ty = infer_local_binding_type(
        receiver,
        visible_local_types,
        module_function_table,
        module_impls,
    )?;
    for definition in module_impls {
        let Some(method_index) = definition
            .methods
            .iter()
            .position(|item| item.name == method)
        else {
            continue;
        };
        let substitutions = infer_impl_method_substitutions(
            definition,
            method_index,
            &receiver_ty,
            args,
            expected_result_type,
            visible_local_types,
            module_function_table,
            module_impls,
        );
        let specialized_for_type =
            specialize_type_with_substitutions(&definition.for_type, &substitutions);
        if specialized_for_type != receiver_ty {
            continue;
        }
        let method_def = &definition.methods[method_index];
        let Some(param) = method_def.params.get(index + 1) else {
            continue;
        };
        let specialized = specialize_type_with_substitutions(&param.ty, &substitutions);
        if callable_type_arity(&specialized).is_some() {
            return Some(specialized);
        }
    }
    None
}
