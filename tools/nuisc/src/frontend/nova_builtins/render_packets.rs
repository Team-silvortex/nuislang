use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::super::{i64_type, lower_expr, FunctionSignature};
use super::packet_helpers::{build_struct_literal, lower_i64_arg_list};
use super::NovaBuiltinInput;

struct NovaRenderPacketEnv<'a> {
    current_domain: &'a str,
    bindings: &'a BTreeMap<String, NirTypeRef>,
    signatures: &'a BTreeMap<String, FunctionSignature>,
    struct_table: &'a BTreeMap<String, NirStructDef>,
}

pub(super) fn lower_nova_render_packet_builtin_call(
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
    let env = NovaRenderPacketEnv {
        current_domain,
        bindings,
        signatures,
        struct_table,
    };
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
            &env,
        )?,
        "nova_surface_packet" => build_four_field_packet(
            args,
            "nova_surface_packet(...) expects 4 args",
            "NovaSurfacePacket",
            ["density", "elevation", "grid", "sheen"],
            &env,
        )?,
        "nova_viewport_packet" => build_four_field_packet(
            args,
            "nova_viewport_packet(...) expects 4 args",
            "NovaViewportPacket",
            ["origin_x", "origin_y", "width", "height"],
            &env,
        )?,
        "nova_layer_packet" => build_four_field_packet(
            args,
            "nova_layer_packet(...) expects 4 args",
            "NovaLayerPacket",
            ["order", "blend", "visibility", "clip"],
            &env,
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
            &env,
        )?,
        "nova_camera_packet" => build_four_field_packet(
            args,
            "nova_camera_packet(...) expects 4 args",
            "NovaCameraPacket",
            ["kind", "focus", "zoom", "orbit"],
            &env,
        )?,
        "nova_material_packet" => build_four_field_packet(
            args,
            "nova_material_packet(...) expects 4 args",
            "NovaMaterialPacket",
            ["shader_kind", "albedo", "roughness", "emissive"],
            &env,
        )?,
        "nova_light_packet" => build_four_field_packet(
            args,
            "nova_light_packet(...) expects 4 args",
            "NovaLightPacket",
            ["kind", "intensity", "range", "reactive"],
            &env,
        )?,
        "nova_mesh_packet" => build_four_field_packet(
            args,
            "nova_mesh_packet(...) expects 4 args",
            "NovaMeshPacket",
            ["primitive", "vertex_count", "index_count", "skinning"],
            &env,
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
    env: &NovaRenderPacketEnv<'_>,
) -> Result<NirExpr, String> {
    let values = lower_i64_arg_list(
        args,
        4,
        arg_error,
        env.current_domain,
        env.bindings,
        env.signatures,
        env.struct_table,
    )?;
    Ok(build_struct_literal(type_name, &fields, values))
}
