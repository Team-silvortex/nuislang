use nuis_semantics::model::NirExpr;

use super::super::{lower_expr, named_type};
use super::NovaBuiltinInput;

pub(super) fn lower_nova_control_accessor_builtin_call(
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
    let Some((expected_type, field_name)) = control_state_accessor_target(callee) else {
        return Ok(None);
    };
    let [state] = args else {
        return Err(format!("{callee}(...) expects 1 arg"));
    };
    let state = lower_expr(
        state,
        current_domain,
        bindings,
        signatures,
        struct_table,
        Some(&named_type(expected_type)),
    )?;
    Ok(Some(NirExpr::FieldAccess {
        base: Box::new(state),
        field: field_name.to_owned(),
    }))
}

fn control_state_accessor_target(callee: &str) -> Option<(&'static str, &'static str)> {
    Some(match callee {
        "nova_slider_state_disabled" => ("NovaSliderState", "disabled"),
        "nova_toggle_state_disabled" => ("NovaToggleState", "disabled"),
        "nova_text_input_state_dirty" => ("NovaTextInputState", "dirty"),
        "nova_text_input_state_read_only" => ("NovaTextInputState", "read_only"),
        "nova_select_state_committed" => ("NovaSelectState", "committed"),
        "nova_select_state_multiple" => ("NovaSelectState", "multiple"),
        "nova_checkbox_state_checked" => ("NovaCheckboxState", "checked"),
        "nova_checkbox_state_disabled" => ("NovaCheckboxState", "disabled"),
        "nova_radio_state_selected" => ("NovaRadioState", "selected"),
        "nova_radio_state_disabled" => ("NovaRadioState", "disabled"),
        "nova_textarea_state_dirty" => ("NovaTextAreaState", "dirty"),
        "nova_textarea_state_read_only" => ("NovaTextAreaState", "read_only"),
        "nova_selection_state_origin" => ("NovaSelectionState", "origin"),
        _ => return None,
    })
}
