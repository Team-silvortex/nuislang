use std::collections::BTreeMap;

use nuis_semantics::model::{NirExternFunction, NirModule, NirStmt, NirStructDef, NirTypeRef};

use super::{
    async_boundary_violation_detail, async_parameter_violation_detail,
    validate_test_function_signature, validate_type_ref,
};

pub(super) fn validate_declared_nir_types(module: &NirModule) -> Result<(), String> {
    let struct_table = module
        .structs
        .iter()
        .map(|definition| (definition.name.clone(), definition.clone()))
        .collect::<BTreeMap<String, NirStructDef>>();
    for function in &module.externs {
        validate_extern_abi_surface(function, None)?;
        for param in &function.params {
            validate_type_ref(&param.ty)?;
        }
        validate_type_ref(&function.return_type)?;
    }
    for interface in &module.extern_interfaces {
        for method in &interface.methods {
            validate_extern_abi_surface(method, Some(interface.name.as_str()))?;
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
                module.domain, module.unit, function.name, module.domain
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
                        "async function `{}` parameter `{}` cannot cross async boundary with type `{}`; {}; async parameters currently forbid `ref`, resource-bearing `Window<...>` / `WindowMut<...>` / `Pipe<...>`, control-plane `Marker<...>` / `HandleTable<...>`, `?`, `Instance<...>`, `Task<...>`, `Thread<...>`, `Mutex<...>` / `MutexGuard<...>`, and `TaskResult<...>` / `DataResult<...>` families",
                        function.name, param.name, param.ty.render(), detail,
                    ));
                }
            }
        }
        if let Some(return_type) = &function.return_type {
            validate_type_ref(return_type)?;
            if function.is_async {
                if let Some(detail) = async_boundary_violation_detail(return_type, &struct_table) {
                    return Err(format!(
                        "async function `{}` cannot return `{}` across async boundary; {}; async returns currently forbid `ref`, resource-bearing `Window<...>` / `WindowMut<...>` / `Pipe<...>`, control-plane `Marker<...>` / `HandleTable<...>`, `?`, `Instance<...>`, `Task<...>`, `Thread<...>`, `Mutex<...>` / `MutexGuard<...>`, and `*Result<...>` families",
                        function.name, return_type.render(), detail
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

fn validate_extern_abi_surface(
    function: &NirExternFunction,
    interface: Option<&str>,
) -> Result<(), String> {
    let callee_label = interface
        .map(|interface_name| format!("extern method `{}.{}`", interface_name, function.name))
        .unwrap_or_else(|| format!("extern function `{}`", function.name));
    for param in &function.params {
        reject_extern_ref_abi_type(
            &param.ty,
            &format!("{callee_label} parameter `{}`", param.name),
            true,
        )?;
    }
    reject_extern_ref_abi_type(
        &function.return_type,
        &format!("{callee_label} return type"),
        false,
    )?;
    Ok(())
}

fn reject_extern_ref_abi_type(
    ty: &NirTypeRef,
    context: &str,
    allow_buffer_param_bridge: bool,
) -> Result<(), String> {
    if allow_buffer_param_bridge
        && ty.is_ref
        && ty.name == "Buffer"
        && !ty.is_optional
        && ty.generic_args.is_empty()
    {
        return Ok(());
    }
    if ty.is_ref {
        return Err(format!(
            "{context} cannot use `{}` in the current extern ABI; only non-optional `ref Buffer` parameters are currently stabilized as the narrow host buffer-handle bridge, while other host-boundary pointer parameters and all pointer returns remain unsupported",
            ty.render(),
        ));
    }
    for arg in &ty.generic_args {
        reject_extern_ref_abi_type(arg, context, false)?;
    }
    Ok(())
}
