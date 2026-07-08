use super::*;

#[test]
fn lowers_shared_nova_selection_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let selection: NovaSelectionPacket = nova_selection_packet(2, 6, 1, 4);
            let list: NovaListPacket = nova_list_packet(2, 6, 7, 1);
            let table: NovaTablePacket = nova_table_packet(4, 3, 2, 1);
            let tree: NovaTreePacket = nova_tree_packet(2, 6, 1, 7);
            let inspector: NovaInspectorPacket = nova_inspector_packet(2, 4, 1, 7);
            let outline: NovaOutlinePacket = nova_outline_packet(2, 6, 1, 7);
            let state: NovaSelectionState = nova_selection_state(selection);
            let list_selection: NovaSelectionState = nova_list_selection(list);
            let table_selection: NovaSelectionState = nova_table_selection(table);
            let tree_selection: NovaSelectionState = nova_tree_selection(tree);
            let inspector_selection: NovaSelectionState = nova_inspector_selection(inspector);
            let outline_selection: NovaSelectionState = nova_outline_selection(outline);
            let selected: i64 = nova_selection_state_selected(state);
            let span: i64 = nova_selection_state_span(list_selection);
            let mode: i64 = nova_selection_state_mode(table_selection);
            let origin: i64 = nova_selection_state_origin(tree_selection);
            let inspector_origin: i64 = nova_selection_state_origin(inspector_selection);
            let outline_origin: i64 = nova_selection_state_origin(outline_selection);
            return selected + span + mode + origin + inspector_origin + outline_origin;
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
        function.body.get(6),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaSelectionState" && type_name == "NovaSelectionState"
    ));
    assert!(matches!(
        function.body.get(7),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaSelectionState" && type_name == "NovaSelectionState"
    ));
    assert!(matches!(
        function.body.get(12),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "selected"
    ));
    assert!(matches!(
        function.body.get(13),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "span"
    ));
    assert!(matches!(
        function.body.get(14),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "mode"
    ));
    assert!(matches!(
        function.body.get(15),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "origin"
    ));
}

#[test]
fn lowers_nova_theme_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let theme: NovaThemePacket = nova_theme_packet(7, 3, 1, 2);
            let state: NovaThemeState = nova_theme_state(theme);
            let accent: i64 = nova_theme_state_accent(state);
            let surface: i64 = nova_theme_state_surface(state);
            let panel_mode: i64 = nova_theme_state_panel_mode(state);
            let contrast: i64 = nova_theme_state_contrast(state);
            return accent + surface + panel_mode + contrast;
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
        function.body.get(1),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        }) if ty.render() == "NovaThemeState" && type_name == "NovaThemeState"
    ));
    assert!(matches!(
        function.body.get(2),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "accent"
    ));
    assert!(matches!(
        function.body.get(5),
        Some(NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::FieldAccess { field, .. },
            ..
        }) if ty.render() == "i64" && field == "contrast"
    ));
}

#[test]
fn lowers_nova_render_state_contracts() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let surface: NovaSurfacePacket = nova_surface_packet(3, 2, 1, 4);
            let viewport: NovaViewportPacket = nova_viewport_packet(2, 1, 48, 18);
            let layer: NovaLayerPacket = nova_layer_packet(1, 2, 1, 0);
            let surface_state: NovaSurfaceState = nova_surface_state(surface);
            let viewport_state: NovaViewportState = nova_viewport_state(viewport);
            let layer_state: NovaLayerState = nova_layer_state(layer);
            let density: i64 = nova_surface_state_density(surface_state);
            let width: i64 = nova_viewport_state_width(viewport_state);
            let visibility: i64 = nova_layer_state_visibility(layer_state);
            return density + width + visibility;
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
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaSurfaceState" && type_name == "NovaSurfaceState",
        _ => false,
    }));
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaViewportState" && type_name == "NovaViewportState",
        _ => false,
    }));
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaLayerState" && type_name == "NovaLayerState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_scene_state_contracts() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let scene: NovaScenePacket = nova_scene_packet(7, 2, 3, 1);
            let camera: NovaCameraPacket = nova_camera_packet(1, 2, 12, 9);
            let material: NovaMaterialPacket = nova_material_packet(1, 8, 3, 2);
            let scene_state: NovaSceneState = nova_scene_state(scene);
            let camera_state: NovaCameraState = nova_camera_state(camera);
            let material_state: NovaMaterialState = nova_material_state(material);
            let lights: i64 = nova_scene_state_light_count(scene_state);
            let zoom: i64 = nova_camera_state_zoom(camera_state);
            let emissive: i64 = nova_material_state_emissive(material_state);
            return lights + zoom + emissive;
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
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaSceneState" && type_name == "NovaSceneState",
        _ => false,
    }));
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaCameraState" && type_name == "NovaCameraState",
        _ => false,
    }));
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaMaterialState" && type_name == "NovaMaterialState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_light_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let light: NovaLightPacket = nova_light_packet(1, 12, 9, 8);
            let state: NovaLightState = nova_light_state(light);
            let intensity: i64 = nova_light_state_intensity(state);
            return intensity;
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
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaLightState" && type_name == "NovaLightState",
        _ => false,
    }));
}

#[test]
fn lowers_nova_mesh_state_contract() {
    let module = parse_nuis_module(
        r#"
        mod cpu Main {
          fn main() -> i64 {
            let mesh: NovaMeshPacket = nova_mesh_packet(1, 12, 9, 8);
            let state: NovaMeshState = nova_mesh_state(mesh);
            let vertices: i64 = nova_mesh_state_vertex_count(state);
            return vertices;
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
    assert!(function.body.iter().any(|stmt| match stmt {
        NirStmt::Let {
            ty: Some(ty),
            value: NirExpr::StructLiteral { type_name, .. },
            ..
        } => ty.render() == "NovaMeshState" && type_name == "NovaMeshState",
        _ => false,
    }));
}
