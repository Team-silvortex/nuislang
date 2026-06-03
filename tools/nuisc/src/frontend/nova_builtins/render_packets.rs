use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::super::{i64_type, lower_expr, FunctionSignature, ModuleConstValue};
use super::packet_helpers::{build_struct_literal, lower_i64_arg_list};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_nova_render_packet_builtin_call(
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
        "nova_header_packet" => {
            let (accent, title_mode) = match args {
                [accent] => {
                    let accent = lower_expr(
                        accent,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )?;
                    (accent.clone(), accent)
                }
                [accent, title_mode] => (
                    lower_expr(
                        accent,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )?,
                    lower_expr(
                        title_mode,
                        current_domain,
                        bindings,
                        signatures,
                        struct_table,
                        Some(&i64_type()),
                    )?,
                ),
                _ => return Err("nova_header_packet(...) expects 1 or 2 args".to_owned()),
            };
            build_struct_literal(
                "NovaHeaderPacket",
                &["accent", "title_mode"],
                vec![accent, title_mode],
            )
        }
        "nova_theme_packet" => build_four_field_packet(
            args,
            "nova_theme_packet(...) expects 4 args",
            "NovaThemePacket",
            ["accent", "surface", "panel_mode", "contrast"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_surface_packet" => build_four_field_packet(
            args,
            "nova_surface_packet(...) expects 4 args",
            "NovaSurfacePacket",
            ["density", "elevation", "grid", "sheen"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_viewport_packet" => build_four_field_packet(
            args,
            "nova_viewport_packet(...) expects 4 args",
            "NovaViewportPacket",
            ["origin_x", "origin_y", "width", "height"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_layer_packet" => build_four_field_packet(
            args,
            "nova_layer_packet(...) expects 4 args",
            "NovaLayerPacket",
            ["order", "blend", "visibility", "clip"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_scene_packet" => build_four_field_packet(
            args,
            "nova_scene_packet(...) expects 4 args",
            "NovaScenePacket",
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
        "nova_camera_packet" => build_four_field_packet(
            args,
            "nova_camera_packet(...) expects 4 args",
            "NovaCameraPacket",
            ["kind", "focus", "zoom", "orbit"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_material_packet" => build_four_field_packet(
            args,
            "nova_material_packet(...) expects 4 args",
            "NovaMaterialPacket",
            ["shader_kind", "albedo", "roughness", "emissive"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_light_packet" => build_four_field_packet(
            args,
            "nova_light_packet(...) expects 4 args",
            "NovaLightPacket",
            ["kind", "intensity", "range", "reactive"],
            current_domain,
            bindings,
            signatures,
            struct_table,
        )?,
        "nova_mesh_packet" => build_four_field_packet(
            args,
            "nova_mesh_packet(...) expects 4 args",
            "NovaMeshPacket",
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

fn build_four_field_packet(
    args: &[AstExpr],
    arg_error: &str,
    type_name: &str,
    fields: [&str; 4],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirExpr, String> {
    let values = lower_i64_arg_list(
        args,
        4,
        arg_error,
        current_domain,
        bindings,
        signatures,
        struct_table,
    )?;
    Ok(build_struct_literal(type_name, &fields, values))
}
