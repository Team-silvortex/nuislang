use std::collections::BTreeMap;

use nuis_semantics::model::{
    AstExpr, NirExpr, NirResultFamily, NirResultStage, NirStructDef, NirTypeRef,
};

use super::{
    async_parameter_violation_detail, ensure_result_like, expr_type, infer_nir_expr_type,
    infer_result_stage, lower_nested_expr_with_async_and_consts, render_type_name,
    FunctionSignature, ModuleConstValue,
};

pub(super) fn ensure_ref_like(
    name: &str,
    expr: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<(), String> {
    match infer_nir_expr_type(expr, bindings, signatures, struct_table) {
        Some(ty) if ty.is_ref => Ok(()),
        Some(ty) => Err(format!(
            "{name}(...) expects a `ref` value, found `{}`",
            render_type_name(&ty)
        )),
        None => Ok(()),
    }
}

pub(super) fn ensure_task_like(
    name: &str,
    expr: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<(), String> {
    match infer_nir_expr_type(expr, bindings, signatures, struct_table) {
        Some(ty) if ty.container_kind() == Some(nuis_semantics::model::NirContainerKind::Task) => {
            Ok(())
        }
        Some(ty) => Err(format!(
            "{name}(...) expects `Task<...>`, found `{}`",
            render_type_name(&ty)
        )),
        None => Err(format!(
            "{name}(...) requires a typed task handle in the current frontend"
        )),
    }
}

pub(super) fn ensure_spawn_input_safe(
    name: &str,
    expr: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<(), String> {
    if matches!(expr, NirExpr::Borrow(_) | NirExpr::BorrowEnd(_)) {
        return Err(format!(
            "{name}(...) does not currently allow borrowed task inputs; move or copy a value instead"
        ));
    }
    match infer_nir_expr_type(expr, bindings, signatures, struct_table) {
        Some(ty) if ty.is_ref => Err(format!(
            "{name}(...) does not currently allow `ref` task inputs, found `{}`",
            render_type_name(&ty)
        )),
        Some(ty) if async_parameter_violation_detail(&ty, struct_table).is_some() => Err(format!(
            "{name}(...) does not currently allow task inputs whose nested payloads cross the async boundary, found `{}`",
            render_type_name(&ty)
        )),
        _ => Ok(()),
    }
}

#[allow(dead_code)]
pub(super) fn lower_single_nested_expr(
    name: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirExpr, String> {
    lower_single_nested_expr_with_consts(
        name,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        &BTreeMap::new(),
        signatures,
        struct_table,
    )
}

pub(super) fn lower_single_nested_expr_with_consts(
    name: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirExpr, String> {
    let [value] = args else {
        return Err(format!("{name}(...) expects exactly one argument"));
    };
    lower_nested_expr_with_async_and_consts(
        value,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        None,
    )
}

#[allow(dead_code)]
pub(super) fn lower_result_wrapper_call(
    name: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    family: NirResultFamily,
    build: fn(Box<NirExpr>, NirResultStage) -> Result<NirExpr, String>,
    usage_hint: &str,
) -> Result<NirExpr, String> {
    lower_result_wrapper_call_with_consts(
        name,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        &BTreeMap::new(),
        signatures,
        struct_table,
        family,
        build,
        usage_hint,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_result_wrapper_call_with_consts(
    name: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    family: NirResultFamily,
    build: fn(Box<NirExpr>, NirResultStage) -> Result<NirExpr, String>,
    expected_shape: &str,
) -> Result<NirExpr, String> {
    let [value] = args else {
        return Err(format!("{name}(...) expects 1 arg"));
    };
    let lowered = lower_single_nested_expr_with_consts(
        name,
        &[value.clone()],
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    )?;
    let Some(stage) = infer_result_stage(&lowered) else {
        return Err(format!("{name}(...) {expected_shape}"));
    };
    if !family.supports_stage(stage) {
        return Err(format!(
            "{name}(...) inferred incompatible `{}` stage `{}`",
            family.type_name(),
            stage.render()
        ));
    }
    let payload = expr_type(&lowered, bindings, signatures, struct_table)
        .ok_or_else(|| format!("{name}(...) could not infer payload type for result wrapper"))?;
    validate_result_stage_payload(stage, &payload)
        .map_err(|error| format!("{name}(...): {error}"))?;
    build(Box::new(lowered), stage)
}

fn validate_result_stage_payload(
    stage: NirResultStage,
    payload: &NirTypeRef,
) -> Result<(), String> {
    stage.validate_payload(payload)
}

#[allow(dead_code)]
pub(super) fn lower_result_observer_call(
    name: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    family: NirResultFamily,
    build: fn(NirExpr) -> NirExpr,
) -> Result<NirExpr, String> {
    lower_result_observer_call_with_consts(
        name,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        &BTreeMap::new(),
        signatures,
        struct_table,
        family,
        build,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_result_observer_call_with_consts<FBuild>(
    name: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    family: NirResultFamily,
    build: FBuild,
) -> Result<NirExpr, String>
where
    FBuild: Fn(NirExpr) -> NirExpr,
{
    let lowered = lower_single_nested_expr_with_consts(
        name,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    )?;
    ensure_result_like(name, &lowered, family, bindings, signatures, struct_table)?;
    Ok(build(lowered))
}
