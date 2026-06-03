use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::super::{i64_type, lower_expr, FunctionSignature, ModuleConstValue};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_nova_control_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    _current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    _module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
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
                .unwrap_or_else(|| NirExpr::Int(0));
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
                .unwrap_or_else(|| NirExpr::Int(127));
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
                .unwrap_or_else(|| NirExpr::Int(1));
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
                .unwrap_or_else(|| NirExpr::Int(0));
            NirExpr::StructLiteral {
                type_name: "NovaSliderPacket".to_owned(),
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
                .unwrap_or_else(|| NirExpr::Int(127));
            let type_name = match callee {
                "nova_progress_packet" => "NovaProgressPacket",
                _ => "NovaMeterPacket",
            };
            NirExpr::StructLiteral {
                type_name: type_name.to_owned(),
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
                .unwrap_or_else(|| NirExpr::Int(0));
            NirExpr::StructLiteral {
                type_name: "NovaTogglePacket".to_owned(),
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
                fields: vec![
                    ("active".to_owned(), active),
                    ("accent".to_owned(), accent),
                    ("intent".to_owned(), intent),
                ],
            }
        }
        "nova_text_input_packet" => {
            let (echo, caret, placeholder, read_only, dirty) = match args {
                [echo, caret] => (echo, caret, None, None, None),
                [echo, caret, placeholder] => (echo, caret, Some(placeholder), None, None),
                [echo, caret, placeholder, read_only] => {
                    (echo, caret, Some(placeholder), Some(read_only), None)
                }
                [echo, caret, placeholder, read_only, dirty] => {
                    (echo, caret, Some(placeholder), Some(read_only), Some(dirty))
                }
                _ => {
                    return Err("nova_text_input_packet(...) expects 2, 3, 4 or 5 args".to_owned());
                }
            };
            let echo = lower_expr(
                echo,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let caret = lower_expr(
                caret,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let placeholder = placeholder
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
                .unwrap_or_else(|| echo.clone());
            let read_only = read_only
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
                .unwrap_or_else(|| NirExpr::Int(0));
            let dirty = dirty
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
                .unwrap_or_else(|| NirExpr::Int(0));
            NirExpr::StructLiteral {
                type_name: "NovaTextInputPacket".to_owned(),
                fields: vec![
                    ("echo".to_owned(), echo),
                    ("caret".to_owned(), caret),
                    ("placeholder".to_owned(), placeholder),
                    ("read_only".to_owned(), read_only),
                    ("dirty".to_owned(), dirty),
                ],
            }
        }
        "nova_select_packet" => {
            let (selected, accent, options, multiple, committed) = match args {
                [selected, accent] => (selected, accent, None, None, None),
                [selected, accent, options] => (selected, accent, Some(options), None, None),
                [selected, accent, options, multiple] => {
                    (selected, accent, Some(options), Some(multiple), None)
                }
                [selected, accent, options, multiple, committed] => (
                    selected,
                    accent,
                    Some(options),
                    Some(multiple),
                    Some(committed),
                ),
                _ => return Err("nova_select_packet(...) expects 2, 3, 4 or 5 args".to_owned()),
            };
            let selected = lower_expr(
                selected,
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
            let options = options
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
                .unwrap_or_else(|| NirExpr::Int(3));
            let multiple = multiple
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
                .unwrap_or_else(|| NirExpr::Int(0));
            let committed = committed
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
                .unwrap_or_else(|| NirExpr::Int(1));
            NirExpr::StructLiteral {
                type_name: "NovaSelectPacket".to_owned(),
                fields: vec![
                    ("selected".to_owned(), selected),
                    ("accent".to_owned(), accent),
                    ("options".to_owned(), options),
                    ("multiple".to_owned(), multiple),
                    ("committed".to_owned(), committed),
                ],
            }
        }
        "nova_checkbox_packet" => {
            let (checked, accent, disabled) = match args {
                [checked, accent] => (checked, accent, None),
                [checked, accent, disabled] => (checked, accent, Some(disabled)),
                _ => return Err("nova_checkbox_packet(...) expects 2 or 3 args".to_owned()),
            };
            let checked = lower_expr(
                checked,
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
                .unwrap_or_else(|| NirExpr::Int(0));
            NirExpr::StructLiteral {
                type_name: "NovaCheckboxPacket".to_owned(),
                fields: vec![
                    ("checked".to_owned(), checked),
                    ("accent".to_owned(), accent),
                    ("disabled".to_owned(), disabled),
                ],
            }
        }
        "nova_radio_packet" => {
            let (selected, options, accent, disabled) = match args {
                [selected, options, accent] => (selected, options, accent, None),
                [selected, options, accent, disabled] => {
                    (selected, options, accent, Some(disabled))
                }
                _ => return Err("nova_radio_packet(...) expects 3 or 4 args".to_owned()),
            };
            let selected = lower_expr(
                selected,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let options = lower_expr(
                options,
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
                .unwrap_or_else(|| NirExpr::Int(0));
            NirExpr::StructLiteral {
                type_name: "NovaRadioPacket".to_owned(),
                fields: vec![
                    ("selected".to_owned(), selected),
                    ("options".to_owned(), options),
                    ("accent".to_owned(), accent),
                    ("disabled".to_owned(), disabled),
                ],
            }
        }
        "nova_textarea_packet" => {
            let (lines, scroll, placeholder, read_only, dirty) = match args {
                [lines, scroll] => (lines, scroll, None, None, None),
                [lines, scroll, placeholder] => (lines, scroll, Some(placeholder), None, None),
                [lines, scroll, placeholder, read_only] => {
                    (lines, scroll, Some(placeholder), Some(read_only), None)
                }
                [lines, scroll, placeholder, read_only, dirty] => (
                    lines,
                    scroll,
                    Some(placeholder),
                    Some(read_only),
                    Some(dirty),
                ),
                _ => return Err("nova_textarea_packet(...) expects 2, 3, 4 or 5 args".to_owned()),
            };
            let lines = lower_expr(
                lines,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let scroll = lower_expr(
                scroll,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let placeholder = placeholder
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
                .unwrap_or_else(|| lines.clone());
            let read_only = read_only
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
                .unwrap_or_else(|| NirExpr::Int(0));
            let dirty = dirty
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
                .unwrap_or_else(|| NirExpr::Int(0));
            NirExpr::StructLiteral {
                type_name: "NovaTextAreaPacket".to_owned(),
                fields: vec![
                    ("lines".to_owned(), lines),
                    ("scroll".to_owned(), scroll),
                    ("placeholder".to_owned(), placeholder),
                    ("read_only".to_owned(), read_only),
                    ("dirty".to_owned(), dirty),
                ],
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(expr))
}
