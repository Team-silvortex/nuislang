use nuis_semantics::model::NirExpr;

use super::super::{i64_type, lower_expr};
use super::controls_form::lower_nova_form_control_builtin_call;
use super::NovaBuiltinInput;

pub(super) fn lower_nova_control_builtin_call(
    input: NovaBuiltinInput<'_>,
) -> Result<Option<NirExpr>, String> {
    if let Some(expr) = lower_nova_form_control_builtin_call(input)? {
        return Ok(Some(expr));
    }

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
        "nova_slider_packet" => {
            let (value, min_value, max_value, step_value, disabled) = match args {
                [value] => (value, None, None, None, None),
                [value, min_value, max_value, step_value] => (
                    value,
                    Some(min_value),
                    Some(max_value),
                    Some(step_value),
                    None,
                ),
                [value, min_value, max_value, step_value, disabled] => (
                    value,
                    Some(min_value),
                    Some(max_value),
                    Some(step_value),
                    Some(disabled),
                ),
                _ => return Err("nova_slider_packet(...) expects 1, 4 or 5 args".to_owned()),
            };
            let value = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let min_expr = min_value
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or(NirExpr::Int(0));
            let max_expr = max_value
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or(NirExpr::Int(127));
            let step_expr = step_value
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or(NirExpr::Int(1));
            let disabled_expr = disabled
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or(NirExpr::Int(0));
            NirExpr::StructLiteral {
                type_name: "NovaSliderPacket".to_owned(),
                type_args: Vec::new(),
                fields: vec![
                    ("value".to_owned(), value),
                    ("min".to_owned(), min_expr),
                    ("max".to_owned(), max_expr),
                    ("step".to_owned(), step_expr),
                    ("disabled".to_owned(), disabled_expr),
                ],
            }
        }
        "nova_progress_packet" | "nova_meter_packet" => {
            let (value, max_value) = match args {
                [value] => (value, None),
                [value, max_value] => (value, Some(max_value)),
                _ => return Err(format!("{callee}(...) expects 1 or 2 args")),
            };
            let value = lower_expr(
                value,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let max_expr = max_value
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or(NirExpr::Int(127));
            let type_name = match callee {
                "nova_progress_packet" => "NovaProgressPacket",
                _ => "NovaMeterPacket",
            };
            NirExpr::StructLiteral {
                type_name: type_name.to_owned(),
                type_args: Vec::new(),
                fields: vec![("value".to_owned(), value), ("max".to_owned(), max_expr)],
            }
        }
        "nova_toggle_packet" => {
            let (live, disabled) = match args {
                [live] => (live, None),
                [live, disabled] => (live, Some(disabled)),
                _ => return Err("nova_toggle_packet(...) expects 1 or 2 args".to_owned()),
            };
            let live = lower_expr(
                live,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let disabled = disabled
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or(NirExpr::Int(0));
            NirExpr::StructLiteral {
                type_name: "NovaTogglePacket".to_owned(),
                type_args: Vec::new(),
                fields: vec![("live".to_owned(), live), ("disabled".to_owned(), disabled)],
            }
        }
        "nova_button_packet" => {
            let (active, accent, intent) = match args {
                [active, accent] => (active, accent, None),
                [active, accent, intent] => (active, accent, Some(intent)),
                _ => return Err("nova_button_packet(...) expects 2 or 3 args".to_owned()),
            };
            let active = lower_expr(
                active,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let accent = lower_expr(
                accent,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let intent = intent
                .map(|expr| {
                    lower_expr(
                        expr,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )
                })
                .transpose()?
                .unwrap_or_else(|| active.clone());
            NirExpr::StructLiteral {
                type_name: "NovaButtonPacket".to_owned(),
                type_args: Vec::new(),
                fields: vec![
                    ("active".to_owned(), active),
                    ("accent".to_owned(), accent),
                    ("intent".to_owned(), intent),
                ],
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(expr))
}
