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
mod packet_accessors;
mod packet_helpers;
mod panel_parts;
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

use super::FunctionSignature;

#[derive(Clone, Copy)]
pub(super) struct NovaBuiltinInput<'a> {
    pub(super) callee: &'a str,
    pub(super) args: &'a [AstExpr],
    pub(super) current_domain: &'a str,
    pub(super) bindings: &'a BTreeMap<String, NirTypeRef>,
    pub(super) signatures: &'a BTreeMap<String, FunctionSignature>,
    pub(super) struct_table: &'a BTreeMap<String, NirStructDef>,
}

pub(super) fn lower_nova_builtin_call(
    input: NovaBuiltinInput<'_>,
) -> Result<Option<NirExpr>, String> {
    macro_rules! try_nova_input_builtin {
        ($lower:expr) => {
            if let Some(expr) = $lower(input)? {
                return Ok(Some(expr));
            }
        };
    }

    try_nova_input_builtin!(controls::lower_nova_control_builtin_call);
    try_nova_input_builtin!(control_states::lower_nova_control_state_builtin_call);
    try_nova_input_builtin!(control_accessors::lower_nova_control_accessor_builtin_call);
    try_nova_input_builtin!(meta_packets::lower_nova_meta_packet_builtin_call);
    try_nova_input_builtin!(meta_states::lower_nova_meta_state_builtin_call);
    try_nova_input_builtin!(meta_accessors::lower_nova_meta_accessor_builtin_call);
    try_nova_input_builtin!(panel_parts::lower_nova_panel_parts_builtin_call);
    try_nova_input_builtin!(packet_accessors::lower_nova_packet_accessor_builtin_call);
    try_nova_input_builtin!(render_packets::lower_nova_render_packet_builtin_call);
    try_nova_input_builtin!(views::lower_nova_view_builtin_call);
    try_nova_input_builtin!(view_states::lower_nova_view_state_builtin_call);
    try_nova_input_builtin!(view_accessors::lower_nova_view_accessor_builtin_call);
    try_nova_input_builtin!(render_states::lower_nova_render_state_builtin_call);
    try_nova_input_builtin!(render_accessors::lower_nova_render_accessor_builtin_call);
    try_nova_input_builtin!(graph_packets::lower_nova_graph_packet_builtin_call);
    try_nova_input_builtin!(graph_states::lower_nova_graph_state_builtin_call);
    try_nova_input_builtin!(graph_accessors::lower_nova_graph_accessor_builtin_call);
    try_nova_input_builtin!(resource_packets::lower_nova_resource_packet_builtin_call);
    try_nova_input_builtin!(resource_states::lower_nova_resource_state_builtin_call);
    try_nova_input_builtin!(resource_accessors::lower_nova_resource_accessor_builtin_call);
    try_nova_input_builtin!(execution_packets::lower_nova_execution_packet_builtin_call);
    try_nova_input_builtin!(execution_accessors::lower_nova_execution_accessor_builtin_call);
    try_nova_input_builtin!(execution_states::lower_nova_execution_state_builtin_call);

    Ok(None)
}
