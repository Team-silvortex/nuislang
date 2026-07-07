use nuis_semantics::model::NirExpr;

use super::super::{lower_expr, named_type};
use super::NovaBuiltinInput;

pub(super) fn lower_nova_view_state_builtin_call(
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
        "nova_tabs_state" => {
            let [packet] = args else {
                return Err("nova_tabs_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTabsPacket")),
            )?;
            NirExpr::StructLiteral {
                type_name: "NovaTabsState".to_owned(),
                type_args: Vec::new(),
                fields: vec![
                    ("active".to_owned(), field(packet.clone(), "active")),
                    ("count".to_owned(), field(packet.clone(), "count")),
                    ("compact".to_owned(), field(packet, "compact")),
                ],
            }
        }
        "nova_list_state" => {
            let [packet] = args else {
                return Err("nova_list_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaListPacket")),
            )?;
            NirExpr::StructLiteral {
                type_name: "NovaListState".to_owned(),
                type_args: Vec::new(),
                fields: vec![
                    ("selected".to_owned(), field(packet.clone(), "selected")),
                    ("items".to_owned(), field(packet.clone(), "items")),
                    ("dense".to_owned(), field(packet, "dense")),
                ],
            }
        }
        "nova_table_state" => {
            let [packet] = args else {
                return Err("nova_table_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTablePacket")),
            )?;
            NirExpr::StructLiteral {
                type_name: "NovaTableState".to_owned(),
                type_args: Vec::new(),
                fields: vec![
                    ("rows".to_owned(), field(packet.clone(), "rows")),
                    ("cols".to_owned(), field(packet.clone(), "cols")),
                    (
                        "selected_row".to_owned(),
                        field(packet.clone(), "selected_row"),
                    ),
                    ("zebra".to_owned(), field(packet, "zebra")),
                ],
            }
        }
        "nova_tree_state" => {
            let [packet] = args else {
                return Err("nova_tree_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaTreePacket")),
            )?;
            NirExpr::StructLiteral {
                type_name: "NovaTreeState".to_owned(),
                type_args: Vec::new(),
                fields: vec![
                    ("selected".to_owned(), field(packet.clone(), "selected")),
                    ("nodes".to_owned(), field(packet.clone(), "nodes")),
                    ("expanded".to_owned(), field(packet, "expanded")),
                ],
            }
        }
        "nova_inspector_state" => {
            let [packet] = args else {
                return Err("nova_inspector_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaInspectorPacket")),
            )?;
            NirExpr::StructLiteral {
                type_name: "NovaInspectorState".to_owned(),
                type_args: Vec::new(),
                fields: vec![
                    ("selected".to_owned(), field(packet.clone(), "selected")),
                    ("fields".to_owned(), field(packet.clone(), "fields")),
                    ("pinned".to_owned(), field(packet, "pinned")),
                ],
            }
        }
        "nova_outline_state" => {
            let [packet] = args else {
                return Err("nova_outline_state(...) expects 1 arg".to_owned());
            };
            let packet = lower_expr(
                packet,
                current_domain,
                bindings,
                signatures,
                struct_table,
                Some(&named_type("NovaOutlinePacket")),
            )?;
            NirExpr::StructLiteral {
                type_name: "NovaOutlineState".to_owned(),
                type_args: Vec::new(),
                fields: vec![
                    ("selected".to_owned(), field(packet.clone(), "selected")),
                    ("items".to_owned(), field(packet.clone(), "items")),
                    ("collapsed".to_owned(), field(packet, "collapsed")),
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
