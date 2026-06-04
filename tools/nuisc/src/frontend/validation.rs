use std::collections::BTreeMap;

use nuis_semantics::model::{NirModule, NirStmt, NirStructDef};

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
                        "async function `{}` parameter `{}` cannot cross async boundary with type `{}`; {}; async parameters currently forbid `ref`, resource-bearing `Window<...>` / `WindowMut<...>` / `Pipe<...>`, control-plane `Marker<...>` / `HandleTable<...>`, `?`, `Instance<...>`, `Task<...>`, and `TaskResult<...>` / `DataResult<...>` families",
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
                        "async function `{}` cannot return `{}` across async boundary; {}; async returns currently forbid `ref`, resource-bearing `Window<...>` / `WindowMut<...>` / `Pipe<...>`, control-plane `Marker<...>` / `HandleTable<...>`, `?`, `Instance<...>`, `Task<...>`, and `*Result<...>` families",
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
