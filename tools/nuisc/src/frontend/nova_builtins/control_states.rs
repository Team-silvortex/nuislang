use nuis_semantics::model::NirExpr;

use super::super::{lower_expr, named_type};
use super::NovaBuiltinInput;

pub(super) fn lower_nova_control_state_builtin_call(
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
    let expr = match callee {
        "nova_slider_state" => {
            let [packet] = args else {
                return Err("nova_slider_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSliderPacket")),
            )?;
            NirExpr::StructLiteral {
                type_name: "NovaSliderState".to_owned(),
                type_args: Vec::new(),
                fields: vec![
                    ("value".to_owned(), field(packet.clone(), "value")),
                    ("min".to_owned(), field(packet.clone(), "min")),
                    ("max".to_owned(), field(packet.clone(), "max")),
                    ("step".to_owned(), field(packet.clone(), "step")),
                    ("disabled".to_owned(), field(packet, "disabled")),
                ],
            }
        }
        "nova_toggle_state" => {
            let [packet] = args else {
                return Err("nova_toggle_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTogglePacket")),
            )?;
            NirExpr::StructLiteral {
                type_name: "NovaToggleState".to_owned(),
                type_args: Vec::new(),
                fields: vec![
                    ("live".to_owned(), field(packet.clone(), "live")),
                    ("disabled".to_owned(), field(packet, "disabled")),
                ],
            }
        }
        "nova_text_input_state" => {
            let [packet] = args else {
                return Err("nova_text_input_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTextInputPacket")),
            )?;
            NirExpr::StructLiteral {
                type_name: "NovaTextInputState".to_owned(),
                type_args: Vec::new(),
                fields: vec![
                    ("dirty".to_owned(), field(packet.clone(), "dirty")),
                    ("read_only".to_owned(), field(packet.clone(), "read_only")),
                    ("caret".to_owned(), field(packet, "caret")),
                ],
            }
        }
        "nova_select_state" => {
            let [packet] = args else {
                return Err("nova_select_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSelectPacket")),
            )?;
            NirExpr::StructLiteral {
                type_name: "NovaSelectState".to_owned(),
                type_args: Vec::new(),
                fields: vec![
                    ("committed".to_owned(), field(packet.clone(), "committed")),
                    ("multiple".to_owned(), field(packet.clone(), "multiple")),
                    ("selected".to_owned(), field(packet, "selected")),
                ],
            }
        }
        "nova_checkbox_state" => {
            let [packet] = args else {
                return Err("nova_checkbox_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaCheckboxPacket")),
            )?;
            NirExpr::StructLiteral {
                type_name: "NovaCheckboxState".to_owned(),
                type_args: Vec::new(),
                fields: vec![
                    ("checked".to_owned(), field(packet.clone(), "checked")),
                    ("disabled".to_owned(), field(packet, "disabled")),
                ],
            }
        }
        "nova_radio_state" => {
            let [packet] = args else {
                return Err("nova_radio_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaRadioPacket")),
            )?;
            NirExpr::StructLiteral {
                type_name: "NovaRadioState".to_owned(),
                type_args: Vec::new(),
                fields: vec![
                    ("selected".to_owned(), field(packet.clone(), "selected")),
                    ("options".to_owned(), field(packet.clone(), "options")),
                    ("disabled".to_owned(), field(packet, "disabled")),
                ],
            }
        }
        "nova_textarea_state" => {
            let [packet] = args else {
                return Err("nova_textarea_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTextAreaPacket")),
            )?;
            NirExpr::StructLiteral {
                type_name: "NovaTextAreaState".to_owned(),
                type_args: Vec::new(),
                fields: vec![
                    ("lines".to_owned(), field(packet.clone(), "lines")),
                    ("scroll".to_owned(), field(packet.clone(), "scroll")),
                    ("read_only".to_owned(), field(packet.clone(), "read_only")),
                    ("dirty".to_owned(), field(packet, "dirty")),
                ],
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(expr))
}

fn field(base: NirExpr, field: &str) -> NirExpr {
    NirExpr::FieldAccess {
        base: Box::new(base),
        field: field.to_owned(),
    }
}
