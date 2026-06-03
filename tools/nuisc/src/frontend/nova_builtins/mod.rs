mod control_accessors;
mod control_states;
mod controls;
mod execution_accessors;
mod execution_packets;
mod execution_states;
mod graph_accessors;
mod graph_packets;
mod graph_states;
mod meta_accessors;
mod meta_packets;
mod meta_states;
mod packet_helpers;
mod render_accessors;
mod render_packets;
mod render_states;
mod resource_accessors;
mod resource_packets;
mod resource_states;
mod view_accessors;
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
    if let Some(control_accessor_builtin) =
        control_accessors::lower_nova_control_accessor_builtin_call(
            callee,
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
        )?
    {
        return Ok(Some(control_accessor_builtin));
    }
    if let Some(meta_packet_builtin) = meta_packets::lower_nova_meta_packet_builtin_call(
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    )? {
        return Ok(Some(meta_packet_builtin));
    }
    if let Some(meta_state_builtin) = meta_states::lower_nova_meta_state_builtin_call(
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    )? {
        return Ok(Some(meta_state_builtin));
    }
    if let Some(meta_accessor_builtin) = meta_accessors::lower_nova_meta_accessor_builtin_call(
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    )? {
        return Ok(Some(meta_accessor_builtin));
    }
    if let Some(render_packet_builtin) = render_packets::lower_nova_render_packet_builtin_call(
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
    )? {
        return Ok(Some(render_packet_builtin));
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
            view_accessors::lower_nova_view_accessor_builtin_call(
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
            render_accessors::lower_nova_render_accessor_builtin_call(
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
            graph_packets::lower_nova_graph_packet_builtin_call(
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
            graph_accessors::lower_nova_graph_accessor_builtin_call(
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
            resource_packets::lower_nova_resource_packet_builtin_call(
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
            resource_accessors::lower_nova_resource_accessor_builtin_call(
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
            execution_packets::lower_nova_execution_packet_builtin_call(
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
            execution_accessors::lower_nova_execution_accessor_builtin_call(
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
