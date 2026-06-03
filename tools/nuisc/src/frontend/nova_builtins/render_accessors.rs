use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::super::{lower_expr, named_type, FunctionSignature, ModuleConstValue};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_nova_render_accessor_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    _current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    _module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    let Some((expected_type, field_name)) = render_state_accessor_target(callee) else {
        return Ok(None);
    };
    let [state] = args else {
        return Err(format!("{callee}(...) expects 1 arg"));
    };
    let state = lower_expr(
        state,
        current_domain,
        bindings,
        signatures,
        struct_table,
        Some(&named_type(expected_type)),
    )?;
    Ok(Some(NirExpr::FieldAccess {
        base: Box::new(state),
        field: field_name.to_owned(),
    }))
}

fn render_state_accessor_target(callee: &str) -> Option<(&'static str, &'static str)> {
    Some(match callee {
        "nova_theme_state_accent" => ("NovaThemeState", "accent"),
        "nova_theme_state_surface" => ("NovaThemeState", "surface"),
        "nova_theme_state_panel_mode" => ("NovaThemeState", "panel_mode"),
        "nova_theme_state_contrast" => ("NovaThemeState", "contrast"),
        "nova_surface_state_density" => ("NovaSurfaceState", "density"),
        "nova_surface_state_elevation" => ("NovaSurfaceState", "elevation"),
        "nova_surface_state_grid" => ("NovaSurfaceState", "grid"),
        "nova_surface_state_sheen" => ("NovaSurfaceState", "sheen"),
        "nova_viewport_state_origin_x" => ("NovaViewportState", "origin_x"),
        "nova_viewport_state_origin_y" => ("NovaViewportState", "origin_y"),
        "nova_viewport_state_width" => ("NovaViewportState", "width"),
        "nova_viewport_state_height" => ("NovaViewportState", "height"),
        "nova_layer_state_order" => ("NovaLayerState", "order"),
        "nova_layer_state_blend" => ("NovaLayerState", "blend"),
        "nova_layer_state_visibility" => ("NovaLayerState", "visibility"),
        "nova_layer_state_clip" => ("NovaLayerState", "clip"),
        "nova_scene_state_root_count" => ("NovaSceneState", "root_count"),
        "nova_scene_state_active_camera" => ("NovaSceneState", "active_camera"),
        "nova_scene_state_light_count" => ("NovaSceneState", "light_count"),
        "nova_scene_state_animation_phase" => ("NovaSceneState", "animation_phase"),
        "nova_camera_state_kind" => ("NovaCameraState", "kind"),
        "nova_camera_state_focus" => ("NovaCameraState", "focus"),
        "nova_camera_state_zoom" => ("NovaCameraState", "zoom"),
        "nova_camera_state_orbit" => ("NovaCameraState", "orbit"),
        "nova_material_state_shader_kind" => ("NovaMaterialState", "shader_kind"),
        "nova_material_state_albedo" => ("NovaMaterialState", "albedo"),
        "nova_material_state_roughness" => ("NovaMaterialState", "roughness"),
        "nova_material_state_emissive" => ("NovaMaterialState", "emissive"),
        "nova_light_state_kind" => ("NovaLightState", "kind"),
        "nova_light_state_intensity" => ("NovaLightState", "intensity"),
        "nova_light_state_range" => ("NovaLightState", "range"),
        "nova_light_state_reactive" => ("NovaLightState", "reactive"),
        "nova_mesh_state_primitive" => ("NovaMeshState", "primitive"),
        "nova_mesh_state_vertex_count" => ("NovaMeshState", "vertex_count"),
        "nova_mesh_state_index_count" => ("NovaMeshState", "index_count"),
        "nova_mesh_state_skinning" => ("NovaMeshState", "skinning"),
        _ => return None,
    })
}
