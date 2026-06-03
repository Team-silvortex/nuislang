mod control_states;
mod controls;
mod execution_states;
mod graph_states;
mod render_states;
mod resource_states;
mod view_states;
mod views;

use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::{FunctionSignature, ModuleConstValue};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_nova_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    if let Some(control_builtin) = controls::lower_nova_control_builtin_call(
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    )? {
        return Ok(Some(control_builtin));
    }
    if let Some(control_state_builtin) = control_states::lower_nova_control_state_builtin_call(
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    )? {
        return Ok(Some(control_state_builtin));
    }
    views::lower_nova_view_builtin_call(
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    )?
    .map_or_else(
        || {
            view_states::lower_nova_view_state_builtin_call(
                callee,
                args,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
            )
        },
        |expr| Ok(Some(expr)),
    )?
    .map_or_else(
        || {
            render_states::lower_nova_render_state_builtin_call(
                callee,
                args,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
            )
        },
        |expr| Ok(Some(expr)),
    )?
    .map_or_else(
        || {
            graph_states::lower_nova_graph_state_builtin_call(
                callee,
                args,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
            )
        },
        |expr| Ok(Some(expr)),
    )?
    .map_or_else(
        || {
            resource_states::lower_nova_resource_state_builtin_call(
                callee,
                args,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
            )
        },
        |expr| Ok(Some(expr)),
    )?
    .map_or_else(
        || {
            execution_states::lower_nova_execution_state_builtin_call(
                callee,
                args,
                current_domain,
                current_function_is_async,
                bindings,
                module_consts,
                signatures,
                struct_table,
            )
        },
        |expr| Ok(Some(expr)),
    )
}
