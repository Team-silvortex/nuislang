use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::super::{i64_type, lower_expr, named_type, FunctionSignature, ModuleConstValue};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_nova_view_builtin_call(
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
        "nova_tabs_packet" => {
            let (active, count, accent, compact) = match args {
                [active, count, accent] => (active, count, accent, None),
                [active, count, accent, compact] => (active, count, accent, Some(compact)),
                _ => return Err("nova_tabs_packet(...) expects 3 or 4 args".to_owned()),
            };
            let active = lower_expr(
                active,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let count = lower_expr(
                count,
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
            let compact = compact
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
                type_name: "NovaTabsPacket".to_owned(),
                fields: vec![
                    ("active".to_owned(), active),
                    ("count".to_owned(), count),
                    ("accent".to_owned(), accent),
                    ("compact".to_owned(), compact),
                ],
            }
        }
        "nova_list_packet" => {
            let (selected, items, accent, dense) = match args {
                [selected, items, accent] => (selected, items, accent, None),
                [selected, items, accent, dense] => (selected, items, accent, Some(dense)),
                _ => return Err("nova_list_packet(...) expects 3 or 4 args".to_owned()),
            };
            let selected = lower_expr(
                selected,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let items = lower_expr(
                items,
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
            let dense = dense
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
                type_name: "NovaListPacket".to_owned(),
                fields: vec![
                    ("selected".to_owned(), selected),
                    ("items".to_owned(), items),
                    ("accent".to_owned(), accent),
                    ("dense".to_owned(), dense),
                ],
            }
        }
        "nova_table_packet" => {
            let (rows, cols, selected_row, zebra) = match args {
                [rows, cols, selected_row] => (rows, cols, selected_row, None),
                [rows, cols, selected_row, zebra] => (rows, cols, selected_row, Some(zebra)),
                _ => return Err("nova_table_packet(...) expects 3 or 4 args".to_owned()),
            };
            let rows = lower_expr(
                rows,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let cols = lower_expr(
                cols,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let selected_row = lower_expr(
                selected_row,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let zebra = zebra
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
                type_name: "NovaTablePacket".to_owned(),
                fields: vec![
                    ("rows".to_owned(), rows),
                    ("cols".to_owned(), cols),
                    ("selected_row".to_owned(), selected_row),
                    ("zebra".to_owned(), zebra),
                ],
            }
        }
        "nova_tree_packet" => {
            let [selected, nodes, expanded, accent] = args else {
                return Err("nova_tree_packet(...) expects 4 args".to_owned());
            };
            let selected = lower_expr(
                selected,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let nodes = lower_expr(
                nodes,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let expanded = lower_expr(
                expanded,
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
            NirExpr::StructLiteral {
                type_name: "NovaTreePacket".to_owned(),
                fields: vec![
                    ("selected".to_owned(), selected),
                    ("nodes".to_owned(), nodes),
                    ("expanded".to_owned(), expanded),
                    ("accent".to_owned(), accent),
                ],
            }
        }
        "nova_inspector_packet" => {
            let [selected, fields, pinned, accent] = args else {
                return Err("nova_inspector_packet(...) expects 4 args".to_owned());
            };
            let selected = lower_expr(
                selected,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let fields = lower_expr(
                fields,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let pinned = lower_expr(
                pinned,
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
            NirExpr::StructLiteral {
                type_name: "NovaInspectorPacket".to_owned(),
                fields: vec![
                    ("selected".to_owned(), selected),
                    ("fields".to_owned(), fields),
                    ("pinned".to_owned(), pinned),
                    ("accent".to_owned(), accent),
                ],
            }
        }
        "nova_outline_packet" => {
            let [selected, items, collapsed, accent] = args else {
                return Err("nova_outline_packet(...) expects 4 args".to_owned());
            };
            let selected = lower_expr(
                selected,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let items = lower_expr(
                items,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let collapsed = lower_expr(
                collapsed,
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
            NirExpr::StructLiteral {
                type_name: "NovaOutlinePacket".to_owned(),
                fields: vec![
                    ("selected".to_owned(), selected),
                    ("items".to_owned(), items),
                    ("collapsed".to_owned(), collapsed),
                    ("accent".to_owned(), accent),
                ],
            }
        }
        "nova_selection_packet" => {
            let [selected, span, mode, origin] = args else {
                return Err("nova_selection_packet(...) expects 4 args".to_owned());
            };
            let selected = lower_expr(
                selected,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let span = lower_expr(
                span,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let mode = lower_expr(
                mode,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            let origin = lower_expr(
                origin,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            NirExpr::StructLiteral {
                type_name: "NovaSelectionPacket".to_owned(),
                fields: vec![
                    ("selected".to_owned(), selected),
                    ("span".to_owned(), span),
                    ("mode".to_owned(), mode),
                    ("origin".to_owned(), origin),
                ],
            }
        }
        "nova_focus_packet" => {
            let [slot] = args else {
                return Err("nova_focus_packet(...) expects 1 arg".to_owned());
            };
            let slot = lower_expr(
                slot,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&i64_type()),
            )?;
            NirExpr::StructLiteral {
                type_name: "NovaFocusPacket".to_owned(),
                fields: vec![("slot".to_owned(), slot)],
            }
        }
        "nova_slider_group_packet" => {
            let [color, speed, radius] = args else {
                return Err("nova_slider_group_packet(...) expects 3 args".to_owned());
            };
            let color = lower_expr(
                color,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSliderPacket")),
            )?;
            let speed = lower_expr(
                speed,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSliderPacket")),
            )?;
            let radius = lower_expr(
                radius,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaSliderPacket")),
            )?;
            NirExpr::StructLiteral {
                type_name: "NovaSliderGroupPacket".to_owned(),
                fields: vec![
                    ("color".to_owned(), color),
                    ("speed".to_owned(), speed),
                    ("radius".to_owned(), radius),
                ],
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(expr))
}
