use super::*;

#[test]
fn lowers_nova_panel_packet_without_shader_unit_literal() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let packet: NovaPanelPacket = nova_panel_packet(1, 2, 3, 4, 5, 6);
            return 1;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.first(),
        Some(NirStmt::Let {
            ty: Some(ty),
            value:
                NirExpr::ShaderProfilePacket {
                    unit,
                    packet_type_name,
                    accent: Some(_),
                    toggle_state: Some(_),
                    focus_index: Some(_),
                    ..
                },
            ..
        }) if ty.render() == "NovaPanelPacket"
            && unit == "__nova__"
            && packet_type_name.as_deref() == Some("NovaPanelPacket")
    ));
}

#[test]
fn lowers_nova_control_packet_builders() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let slider: NovaSliderPacket = nova_slider_packet(7, 0, 10, 2, 1);
            let progress: NovaProgressPacket = nova_progress_packet(4, 10);
            let toggle: NovaTogglePacket = nova_toggle_packet(1, 1);
            let button: NovaButtonPacket = nova_button_packet(1, 9, 2);
            let text_input: NovaTextInputPacket =
              nova_text_input_packet(8, 1, 4, 1, 1);
            let select: NovaSelectPacket = nova_select_packet(2, 5, 4, 1, 0);
            let checkbox: NovaCheckboxPacket = nova_checkbox_packet(1, 5, 0);
            let radio: NovaRadioPacket = nova_radio_packet(2, 4, 5, 1);
            let textarea: NovaTextAreaPacket = nova_textarea_packet(3, 1, 7, 0, 1);
            let tabs: NovaTabsPacket = nova_tabs_packet(1, 4, 5, 0);
            let list: NovaListPacket = nova_list_packet(1, 5, 7, 1);
            let table: NovaTablePacket = nova_table_packet(4, 3, 1, 1);
            let tree: NovaTreePacket = nova_tree_packet(1, 6, 1, 7);
            let inspector: NovaInspectorPacket = nova_inspector_packet(1, 4, 1, 7);
            let outline: NovaOutlinePacket = nova_outline_packet(1, 6, 1, 7);
            let theme: NovaThemePacket = nova_theme_packet(7, 3, 1, 2);
            let selection: NovaSelectionPacket = nova_selection_packet(1, 6, 1, 4);
            return 1;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.first(),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaSliderPacket" && type_name == "NovaSliderPacket"
    ));
    assert!(matches!(
        function.body.get(1),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaProgressPacket" && type_name == "NovaProgressPacket"
    ));
    assert!(matches!(
        function.body.get(2),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaTogglePacket" && type_name == "NovaTogglePacket"
    ));
    assert!(matches!(
        function.body.get(3),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaButtonPacket" && type_name == "NovaButtonPacket"
    ));
    assert!(matches!(
        function.body.get(4),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaTextInputPacket" && type_name == "NovaTextInputPacket"
    ));
    assert!(matches!(
        function.body.get(5),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaSelectPacket" && type_name == "NovaSelectPacket"
    ));
    assert!(matches!(
        function.body.get(6),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaCheckboxPacket" && type_name == "NovaCheckboxPacket"
    ));
    assert!(matches!(
        function.body.get(7),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaRadioPacket" && type_name == "NovaRadioPacket"
    ));
    assert!(matches!(
        function.body.get(8),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaTextAreaPacket" && type_name == "NovaTextAreaPacket"
    ));
    assert!(matches!(
        function.body.get(9),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaTabsPacket" && type_name == "NovaTabsPacket"
    ));
    assert!(matches!(
        function.body.get(10),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaListPacket" && type_name == "NovaListPacket"
    ));
    assert!(matches!(
        function.body.get(11),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaTablePacket" && type_name == "NovaTablePacket"
    ));
    assert!(matches!(
        function.body.get(12),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaTreePacket" && type_name == "NovaTreePacket"
    ));
    assert!(matches!(
        function.body.get(13),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaInspectorPacket" && type_name == "NovaInspectorPacket"
    ));
    assert!(matches!(
        function.body.get(14),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaOutlinePacket" && type_name == "NovaOutlinePacket"
    ));
    assert!(matches!(
        function.body.get(15),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaThemePacket" && type_name == "NovaThemePacket"
    ));
    assert!(matches!(
        function.body.get(16),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaSelectionPacket" && type_name == "NovaSelectionPacket"
    ));
}

#[test]
fn lowers_nova_control_state_observers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let slider: NovaSliderPacket = nova_slider_packet(7, 0, 10, 2, 1);
            let text_input: NovaTextInputPacket =
              nova_text_input_packet(8, 1, 4, 1, 1);
            let select: NovaSelectPacket = nova_select_packet(2, 5, 4, 1, 0);
            let slider_disabled: i64 = nova_slider_disabled(slider);
            let dirty: i64 = nova_text_input_dirty(text_input);
            let committed: i64 = nova_select_committed(select);
            return committed;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.get(3),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "disabled"
    ));
    assert!(matches!(
        function.body.get(4),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "dirty"
    ));
    assert!(matches!(
        function.body.get(5),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "committed"
    ));
}

#[test]
fn lowers_extended_nova_control_state_observers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let checkbox: NovaCheckboxPacket = nova_checkbox_packet(1, 5, 1);
            let radio: NovaRadioPacket = nova_radio_packet(2, 4, 5, 0);
            let textarea: NovaTextAreaPacket = nova_textarea_packet(3, 1, 7, 1, 1);
            let tabs: NovaTabsPacket = nova_tabs_packet(1, 4, 5, 1);
            let checkbox_state: NovaCheckboxState = nova_checkbox_state(checkbox);
            let radio_state: NovaRadioState = nova_radio_state(radio);
            let textarea_state: NovaTextAreaState = nova_textarea_state(textarea);
            let tabs_state: NovaTabsState = nova_tabs_state(tabs);
            let checked: i64 = nova_checkbox_state_checked(checkbox_state);
            let radio_disabled: i64 = nova_radio_state_disabled(radio_state);
            let dirty: i64 = nova_textarea_state_dirty(textarea_state);
            let compact: i64 = nova_tabs_state_compact(tabs_state);
            return checked + radio_disabled + dirty + compact;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.get(4),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaCheckboxState" && type_name == "NovaCheckboxState"
    ));
    assert!(matches!(
        function.body.get(5),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaRadioState" && type_name == "NovaRadioState"
    ));
    assert!(matches!(
        function.body.get(6),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaTextAreaState" && type_name == "NovaTextAreaState"
    ));
    assert!(matches!(
        function.body.get(7),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaTabsState" && type_name == "NovaTabsState"
    ));
    assert!(matches!(
        function.body.get(8),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "checked"
    ));
    assert!(matches!(
        function.body.get(9),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "disabled"
    ));
    assert!(matches!(
        function.body.get(10),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "dirty"
    ));
    assert!(matches!(
        function.body.get(11),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "compact"
    ));
}

#[test]
fn lowers_complex_nova_control_state_observers() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let list: NovaListPacket = nova_list_packet(1, 5, 7, 1);
            let table: NovaTablePacket = nova_table_packet(4, 3, 1, 1);
            let list_state: NovaListState = nova_list_state(list);
            let table_state: NovaTableState = nova_table_state(table);
            let tree: NovaTreePacket = nova_tree_packet(1, 6, 1, 7);
            let tree_state: NovaTreeState = nova_tree_state(tree);
            let inspector: NovaInspectorPacket = nova_inspector_packet(1, 4, 1, 7);
            let inspector_state: NovaInspectorState = nova_inspector_state(inspector);
            let outline: NovaOutlinePacket = nova_outline_packet(1, 6, 1, 7);
            let outline_state: NovaOutlineState = nova_outline_state(outline);
            let dense: i64 = nova_list_state_dense(list_state);
            let selected: i64 = nova_list_state_selected(list_state);
            let zebra: i64 = nova_table_state_zebra(table_state);
            let selected_row: i64 = nova_table_state_selected_row(table_state);
            let expanded: i64 = nova_tree_state_expanded(tree_state);
            let tree_selected: i64 = nova_tree_state_selected(tree_state);
            let pinned: i64 = nova_inspector_state_pinned(inspector_state);
            let inspected: i64 = nova_inspector_state_selected(inspector_state);
            let collapsed: i64 = nova_outline_state_collapsed(outline_state);
            let outlined: i64 = nova_outline_state_selected(outline_state);
            return dense + selected + zebra + selected_row + expanded + tree_selected + pinned + inspected + collapsed + outlined;
          }
        }
        "#,
    )
    .unwrap();

    let function = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .unwrap();
    assert!(matches!(
        function.body.get(2),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaListState" && type_name == "NovaListState"
    ));
    assert!(matches!(
        function.body.get(3),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaTableState" && type_name == "NovaTableState"
    ));
    assert!(matches!(
        function.body.get(5),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaTreeState" && type_name == "NovaTreeState"
    ));
    assert!(matches!(
        function.body.get(7),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaInspectorState" && type_name == "NovaInspectorState"
    ));
    assert!(matches!(
        function.body.get(9),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaOutlineState" && type_name == "NovaOutlineState"
    ));
    assert!(matches!(
        function.body.get(10),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "dense"
    ));
    assert!(matches!(
        function.body.get(11),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "selected"
    ));
    assert!(matches!(
        function.body.get(12),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "zebra"
    ));
    assert!(matches!(
        function.body.get(13),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "selected_row"
    ));
    assert!(matches!(
        function.body.get(14),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "expanded"
    ));
    assert!(matches!(
        function.body.get(15),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "selected"
    ));
    assert!(matches!(
        function.body.get(16),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "pinned"
    ));
    assert!(matches!(
        function.body.get(17),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "selected"
    ));
    assert!(matches!(
        function.body.get(18),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "collapsed"
    ));
    assert!(matches!(
        function.body.get(19),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "selected"
    ));
}
