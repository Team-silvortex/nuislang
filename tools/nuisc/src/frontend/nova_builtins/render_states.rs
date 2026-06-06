use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::super::{lower_expr, named_type, FunctionSignature, ModuleConstValue};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_nova_render_state_builtin_call(
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
        "nova_theme_state" => build_four_field_state(
            args,
            "nova_theme_state(...) expects 1 arg",
            "NovaThemePacket",
            "NovaThemeState",
            ["accent", "surface", "panel_mode", "contrast"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_surface_state" => build_four_field_state(
            args,
            "nova_surface_state(...) expects 1 arg",
            "NovaSurfacePacket",
            "NovaSurfaceState",
            ["density", "elevation", "grid", "sheen"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_viewport_state" => build_four_field_state(
            args,
            "nova_viewport_state(...) expects 1 arg",
            "NovaViewportPacket",
            "NovaViewportState",
            ["origin_x", "origin_y", "width", "height"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_layer_state" => build_four_field_state(
            args,
            "nova_layer_state(...) expects 1 arg",
            "NovaLayerPacket",
            "NovaLayerState",
            ["order", "blend", "visibility", "clip"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_scene_state" => build_four_field_state(
            args,
            "nova_scene_state(...) expects 1 arg",
            "NovaScenePacket",
            "NovaSceneState",
            [
                "root_count",
                "active_camera",
                "light_count",
                "animation_phase",
            ],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_camera_state" => build_four_field_state(
            args,
            "nova_camera_state(...) expects 1 arg",
            "NovaCameraPacket",
            "NovaCameraState",
            ["kind", "focus", "zoom", "orbit"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_material_state" => build_four_field_state(
            args,
            "nova_material_state(...) expects 1 arg",
            "NovaMaterialPacket",
            "NovaMaterialState",
            ["shader_kind", "albedo", "roughness", "emissive"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_light_state" => build_four_field_state(
            args,
            "nova_light_state(...) expects 1 arg",
            "NovaLightPacket",
            "NovaLightState",
            ["kind", "intensity", "range", "reactive"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_mesh_state" => build_four_field_state(
            args,
            "nova_mesh_state(...) expects 1 arg",
            "NovaMeshPacket",
            "NovaMeshState",
            ["primitive", "vertex_count", "index_count", "skinning"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        _ => return Ok(None),
    };
    Ok(Some(expr))
}

fn build_four_field_state(
    args: &[AstExpr],
    arg_error: &str,
    packet_type: &str,
    state_type: &str,
    fields: [&str; 4],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirExpr, String> {
    let [packet] = args else {
        return Err(arg_error.to_owned());
    };
    let packet = lower_expr(
        packet,
        current_domain,
        bindings,
        signatures,
        struct_table,
        Some(&named_type(packet_type)),
    )?;
    Ok(NirExpr::StructLiteral {
        type_name: state_type.to_owned(),
        type_args: Vec::new(),
        fields: vec![
            (fields[0].to_owned(), field(packet.clone(), fields[0])),
            (fields[1].to_owned(), field(packet.clone(), fields[1])),
            (fields[2].to_owned(), field(packet.clone(), fields[2])),
            (fields[3].to_owned(), field(packet, fields[3])),
        ],
    })
}

fn field(base: NirExpr, field: &str) -> NirExpr {
    NirExpr::FieldAccess {
        base: Box::new(base),
        field: field.to_owned(),
    }
}
