use std::collections::BTreeMap;

use nuis_semantics::model::{
    AstExpr, NirAddressClass, NirExpr, NirResultFamily, NirResultStage, NirStructDef, NirTypeRef,
};

use super::{
    async_parameter_violation_detail, compatible_types, ensure_result_like, expr_type,
    infer_nir_expr_address_class, infer_nir_expr_type, infer_result_stage,
    lower_nested_expr_with_async_and_consts, render_type_name, FunctionSignature, ModuleConstValue,
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

pub(super) fn ensure_thread_like(
    name: &str,
    expr: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<(), String> {
    match infer_nir_expr_type(expr, bindings, signatures, struct_table) {
        Some(ty) if ty.is_thread_family() => Ok(()),
        Some(ty) => Err(format!(
            "{name}(...) expects `Thread<...>`, found `{}`",
            render_type_name(&ty)
        )),
        None => Err(format!(
            "{name}(...) requires a typed thread handle in the current frontend"
        )),
    }
}

pub(super) fn ensure_mutex_like(
    name: &str,
    expr: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<(), String> {
    match infer_nir_expr_type(expr, bindings, signatures, struct_table) {
        Some(ty) if ty.is_mutex_family() => Ok(()),
        Some(ty) => Err(format!(
            "{name}(...) expects `Mutex<...>`, found `{}`",
            render_type_name(&ty)
        )),
        None => Err(format!(
            "{name}(...) requires a typed mutex handle in the current frontend"
        )),
    }
}

pub(super) fn ensure_mutex_guard_like(
    name: &str,
    expr: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<(), String> {
    match infer_nir_expr_type(expr, bindings, signatures, struct_table) {
        Some(ty) if ty.is_mutex_guard_family() => Ok(()),
        Some(ty) => Err(format!(
            "{name}(...) expects `MutexGuard<...>`, found `{}`",
            render_type_name(&ty)
        )),
        None => Err(format!(
            "{name}(...) requires a typed mutex guard in the current frontend"
        )),
    }
}

pub(super) fn ensure_call_arg_matches_param(
    callee: &str,
    arg_index: usize,
    arg: &NirExpr,
    expected_param: &NirTypeRef,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    is_extern: bool,
) -> Result<(), String> {
    let Some(actual_ty) = infer_nir_expr_type(arg, bindings, signatures, struct_table) else {
        return Ok(());
    };
    if compatible_types(expected_param, &actual_ty)
        || (is_extern && supports_host_buffer_handle_bridge(expected_param, &actual_ty))
    {
        return Ok(());
    }
    let param_position = arg_index + 1;
    let bridge_detail = if is_extern && expected_param == &i64_host_value_type() {
        "; only `ref Buffer -> i64` is currently allowed as the narrow host buffer-handle bridge"
    } else {
        ""
    };
    Err(format!(
        "function `{callee}` argument {param_position} expects `{}`, found `{}`{bridge_detail}",
        render_type_name(expected_param),
        render_type_name(&actual_ty)
    ))
}

pub(super) fn lower_extern_call_arg_for_param(
    arg: NirExpr,
    expected_param: &NirTypeRef,
) -> NirExpr {
    if expected_param == &buffer_ref_type() {
        return NirExpr::HostBufferHandle(Box::new(arg));
    }
    arg
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
        Some(ty) if ty.is_ref => {
            let detail = match infer_boundary_address_class(expr, bindings, signatures, struct_table)
            {
                Some(NirAddressClass::Borrowed) => {
                    "borrowed `ref` task inputs, including traversal-derived borrowed refs"
                }
                Some(NirAddressClass::Owned) => {
                    "owned `ref` task inputs yet; pointer ownership transfer across async boundaries is not stabilized"
                }
                None => "`ref` task inputs",
            };
            Err(format!(
                "{name}(...) does not currently allow {detail}, found `{}`",
                render_type_name(&ty)
            ))
        }
        Some(ty) if async_parameter_violation_detail(&ty, struct_table).is_some() => Err(format!(
            "{name}(...) does not currently allow task inputs whose nested payloads cross the async boundary, found `{}`",
            render_type_name(&ty)
        )),
        _ => Ok(()),
    }
}

fn infer_boundary_address_class(
    expr: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Option<NirAddressClass> {
    infer_nir_expr_address_class(expr, bindings, &BTreeMap::new(), signatures, struct_table)
}

fn supports_host_buffer_handle_bridge(expected: &NirTypeRef, actual: &NirTypeRef) -> bool {
    (expected == &i64_host_value_type() || expected == &buffer_ref_type())
        && (actual == &buffer_ref_type() || actual == &i64_host_value_type())
}

fn i64_host_value_type() -> NirTypeRef {
    NirTypeRef {
        name: "i64".to_owned(),
        generic_args: vec![],
        is_optional: false,
        is_ref: false,
    }
}

fn buffer_ref_type() -> NirTypeRef {
    NirTypeRef {
        name: "Buffer".to_owned(),
        generic_args: vec![],
        is_optional: false,
        is_ref: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ref_node() -> NirTypeRef {
        NirTypeRef {
            name: "Node".to_owned(),
            generic_args: vec![],
            is_optional: false,
            is_ref: true,
        }
    }

    fn thread_i64() -> NirTypeRef {
        NirTypeRef {
            name: "Thread".to_owned(),
            generic_args: vec![NirTypeRef {
                name: "i64".to_owned(),
                generic_args: vec![],
                is_optional: false,
                is_ref: false,
            }],
            is_optional: false,
            is_ref: false,
        }
    }

    #[test]
    fn spawn_input_error_mentions_owned_ref_boundary_for_owned_pointer_expr() {
        let expr = NirExpr::AllocNode {
            value: Box::new(NirExpr::Int(1)),
            next: Box::new(NirExpr::Null),
        };
        let error = ensure_spawn_input_safe(
            "spawn",
            &expr,
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .unwrap_err();
        assert!(error.contains("owned `ref` task inputs yet"));
        assert!(
            error.contains("pointer ownership transfer across async boundaries is not stabilized")
        );
    }

    #[test]
    fn spawn_input_error_mentions_borrowed_ref_boundary_for_traversal_pointer_expr() {
        let mut bindings = BTreeMap::new();
        bindings.insert("head".to_owned(), ref_node());
        let expr = NirExpr::LoadNext(Box::new(NirExpr::Borrow(Box::new(NirExpr::Var(
            "head".to_owned(),
        )))));
        let error = ensure_spawn_input_safe(
            "spawn",
            &expr,
            &bindings,
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .unwrap_err();
        assert!(error.contains("borrowed `ref` task inputs"));
        assert!(error.contains("traversal-derived borrowed refs"));
    }

    #[test]
    fn accepts_host_buffer_handle_bridge_for_extern_i64_slot() {
        let expr = NirExpr::AllocBuffer {
            len: Box::new(NirExpr::Int(8)),
            fill: Box::new(NirExpr::Int(0)),
        };
        ensure_call_arg_matches_param(
            "host_stdin_read",
            0,
            &expr,
            &i64_host_value_type(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            true,
        )
        .unwrap();
    }

    #[test]
    fn rejects_non_buffer_ref_for_extern_i64_slot() {
        let expr = NirExpr::AllocNode {
            value: Box::new(NirExpr::Int(1)),
            next: Box::new(NirExpr::Null),
        };
        let error = ensure_call_arg_matches_param(
            "host_stdin_read",
            0,
            &expr,
            &i64_host_value_type(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            true,
        )
        .unwrap_err();
        assert!(error.contains("expects `i64`, found `ref Node`"));
        assert!(error.contains("`ref Buffer -> i64`"));
    }

    #[test]
    fn spawn_input_rejects_staged_thread_handles() {
        let mut bindings = BTreeMap::new();
        bindings.insert("worker".to_owned(), thread_i64());
        let error = ensure_spawn_input_safe(
            "spawn",
            &NirExpr::Var("worker".to_owned()),
            &bindings,
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .unwrap_err();
        assert!(error.contains("nested payloads cross the async boundary"));
        assert!(error.contains("Thread<i64>"));
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
