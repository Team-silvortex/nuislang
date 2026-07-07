use nuis_semantics::model::NirExpr;

use super::super::{lower_expr, named_type};
use super::NovaBuiltinInput;

pub(super) fn lower_nova_packet_accessor_builtin_call(
    input: NovaBuiltinInput<'_>,
) -> Result<Option<NirExpr>, String> {
    let NovaBuiltinInput {
        callee,
        args,
        current_domain,
        bindings,
        signatures,
        struct_table,
        ..
    } = input;
    let (expected_type, field_name) = match callee {
        "nova_slider_disabled" => ("NovaSliderPacket", "disabled"),
        "nova_toggle_disabled" => ("NovaTogglePacket", "disabled"),
        "nova_text_input_dirty" => ("NovaTextInputPacket", "dirty"),
        "nova_text_input_read_only" => ("NovaTextInputPacket", "read_only"),
        "nova_select_committed" => ("NovaSelectPacket", "committed"),
        "nova_select_multiple" => ("NovaSelectPacket", "multiple"),
        "nova_checkbox_checked" => ("NovaCheckboxPacket", "checked"),
        "nova_checkbox_disabled" => ("NovaCheckboxPacket", "disabled"),
        "nova_radio_disabled" => ("NovaRadioPacket", "disabled"),
        "nova_textarea_dirty" => ("NovaTextAreaPacket", "dirty"),
        "nova_textarea_read_only" => ("NovaTextAreaPacket", "read_only"),
        "nova_tabs_compact" => ("NovaTabsPacket", "compact"),
        "nova_list_dense" => ("NovaListPacket", "dense"),
        "nova_table_zebra" => ("NovaTablePacket", "zebra"),
        "nova_tree_expanded" => ("NovaTreePacket", "expanded"),
        "nova_inspector_pinned" => ("NovaInspectorPacket", "pinned"),
        "nova_outline_collapsed" => ("NovaOutlinePacket", "collapsed"),
        "nova_selection_selected" => ("NovaSelectionPacket", "selected"),
        "nova_selection_mode" => ("NovaSelectionPacket", "mode"),
        _ => return Ok(None),
    };
    let [packet] = args else {
        return Err(format!("{callee}(...) expects 1 arg"));
    };
    let packet = lower_expr(
        packet,
        current_domain,
        bindings,
        signatures,
        struct_table,
        Some(&named_type(expected_type)),
    )?;
    Ok(Some(NirExpr::FieldAccess {
        base: Box::new(packet),
        field: field_name.to_owned(),
    }))
}
