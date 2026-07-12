use super::packet_helpers::scalar_to_color_key;
use super::parse_ball_packet_scene_core_fields::BallPacketSceneCoreFields;
use super::parse_ball_packet_scene_core_helpers::{field, field_with, scaled};
use yir_core::{StructValue, Value};

pub(crate) fn parse_ball_packet_scene_core(
    packet: &StructValue,
    op: &str,
    color: &Value,
    speed: &Value,
    radius_scale: f32,
) -> Result<BallPacketSceneCoreFields, String> {
    let speed_key = || scalar_to_color_key(speed, op).unwrap_or(0);
    let accent = field_with(
        packet,
        op,
        &["accent", "header_accent"],
        &["theme", "header"],
        &["accent"],
        || scalar_to_color_key(color, op).unwrap_or(0),
    )?;
    let surface = field(
        packet,
        op,
        &["theme_surface"],
        "theme",
        "surface",
        scaled(radius_scale, 8.0),
    )?;
    let panel_mode = field(
        packet,
        op,
        &["panel_mode"],
        "theme",
        "panel_mode",
        accent.rem_euclid(2),
    )?;
    let contrast = field_with(
        packet,
        op,
        &["contrast"],
        &["theme"],
        &["contrast"],
        speed_key,
    )?;
    let surface_density = field_with(
        packet,
        op,
        &["density"],
        &["surface"],
        &["density"],
        speed_key,
    )?;
    let surface_elevation = field(
        packet,
        op,
        &["elevation"],
        "surface",
        "elevation",
        scaled(radius_scale, 16.0),
    )?;
    let surface_grid = field(
        packet,
        op,
        &["grid"],
        "surface",
        "grid",
        accent.rem_euclid(3),
    )?;
    let surface_sheen = field(packet, op, &["sheen"], "surface", "sheen", accent)?;
    let viewport_x = field(
        packet,
        op,
        &["origin_x"],
        "viewport",
        "origin_x",
        accent.rem_euclid(4),
    )?;
    let viewport_y = field(
        packet,
        op,
        &["origin_y"],
        "viewport",
        "origin_y",
        contrast.rem_euclid(3),
    )?;
    let viewport_width = field(packet, op, &["width"], "viewport", "width", 48)?;
    let viewport_height = field(packet, op, &["height"], "viewport", "height", 18)?;
    let layer_order = field(packet, op, &["order"], "layer", "order", 1)?;
    let layer_blend = field(
        packet,
        op,
        &["blend"],
        "layer",
        "blend",
        contrast.rem_euclid(3),
    )?;
    let layer_visibility = field(packet, op, &["visibility"], "layer", "visibility", 1)?;
    let layer_clip = field(
        packet,
        op,
        &["clip"],
        "layer",
        "clip",
        scaled(radius_scale, 8.0),
    )?;
    let scene_root_count = field(packet, op, &["root_count"], "scene", "root_count", 7)?;
    let scene_active_camera = field(
        packet,
        op,
        &["active_camera"],
        "scene",
        "active_camera",
        accent.rem_euclid(6),
    )?;
    let scene_light_count = field(packet, op, &["light_count"], "scene", "light_count", 3)?;
    let scene_animation_phase = field(
        packet,
        op,
        &["animation_phase"],
        "scene",
        "animation_phase",
        contrast.rem_euclid(4),
    )?;
    let camera_kind = field(
        packet,
        op,
        &["kind"],
        "camera",
        "kind",
        contrast.rem_euclid(3),
    )?;
    let camera_focus = field(
        packet,
        op,
        &["camera_focus"],
        "camera",
        "focus",
        accent.rem_euclid(6),
    )?;
    let camera_zoom = field_with(packet, op, &["zoom"], &["camera"], &["zoom"], speed_key)?;
    let camera_orbit = field(
        packet,
        op,
        &["orbit"],
        "camera",
        "orbit",
        scaled(radius_scale, 12.0),
    )?;
    let material_shader_kind = field(
        packet,
        op,
        &["shader_kind"],
        "material",
        "shader_kind",
        contrast.rem_euclid(3),
    )?;
    let material_albedo = field(packet, op, &["albedo"], "material", "albedo", accent)?;
    let material_roughness = field_with(
        packet,
        op,
        &["roughness"],
        &["material"],
        &["roughness"],
        speed_key,
    )?;
    let material_emissive = field(
        packet,
        op,
        &["emissive"],
        "material",
        "emissive",
        scaled(radius_scale, 24.0),
    )?;
    let light_kind = field(
        packet,
        op,
        &["kind"],
        "light",
        "kind",
        contrast.rem_euclid(3),
    )?;
    let light_intensity = field_with(
        packet,
        op,
        &["intensity"],
        &["light"],
        &["intensity"],
        speed_key,
    )?;
    let light_range = field(
        packet,
        op,
        &["range"],
        "light",
        "range",
        scaled(radius_scale, 18.0),
    )?;
    let light_reactive = field(packet, op, &["reactive"], "light", "reactive", accent)?;
    let mesh_primitive = field(
        packet,
        op,
        &["primitive"],
        "mesh",
        "primitive",
        contrast.rem_euclid(3),
    )?;
    let mesh_vertex_count = field_with(
        packet,
        op,
        &["vertex_count"],
        &["mesh"],
        &["vertex_count"],
        || speed_key().max(3),
    )?;
    let mesh_index_count = field(
        packet,
        op,
        &["index_count"],
        "mesh",
        "index_count",
        scaled(radius_scale, 18.0),
    )?;
    let mesh_skinning = field(packet, op, &["skinning"], "mesh", "skinning", accent)?;
    let transform_translate = field_with(
        packet,
        op,
        &["translate"],
        &["transform"],
        &["translate"],
        speed_key,
    )?;
    let transform_rotate = field(
        packet,
        op,
        &["rotate"],
        "transform",
        "rotate",
        contrast.rem_euclid(4),
    )?;
    let transform_scale = field(
        packet,
        op,
        &["scale"],
        "transform",
        "scale",
        scaled(radius_scale, 16.0),
    )?;
    let transform_pivot = field(
        packet,
        op,
        &["pivot"],
        "transform",
        "pivot",
        accent.rem_euclid(6),
    )?;
    let node_id = field(
        packet,
        op,
        &["node_id"],
        "node",
        "node_id",
        accent.rem_euclid(8),
    )?;
    let node_parent_id = field(
        packet,
        op,
        &["parent_id"],
        "node",
        "parent_id",
        contrast.rem_euclid(4),
    )?;
    let node_flags = field(packet, op, &["flags"], "node", "flags", accent)?;
    let node_depth = field(packet, op, &["depth"], "node", "depth", 2)?;
    let scene_link_node_slot = field(
        packet,
        op,
        &["node_slot"],
        "scene_link",
        "node_slot",
        node_id,
    )?;
    let scene_link_transform_slot = field(
        packet,
        op,
        &["transform_slot"],
        "scene_link",
        "transform_slot",
        transform_translate,
    )?;
    let scene_link_mesh_slot = field(
        packet,
        op,
        &["mesh_slot"],
        "scene_link",
        "mesh_slot",
        mesh_vertex_count,
    )?;
    let scene_link_material_slot = field(
        packet,
        op,
        &["material_slot"],
        "scene_link",
        "material_slot",
        material_albedo,
    )?;
    let scene_link_light_slot = field(
        packet,
        op,
        &["light_slot"],
        "scene_link",
        "light_slot",
        light_kind,
    )?;
    let scene_link_layer_slot = field(
        packet,
        op,
        &["layer_slot"],
        "scene_link",
        "layer_slot",
        layer_order,
    )?;
    let instance_node_slot = field(
        packet,
        op,
        &["node_slot"],
        "instance",
        "node_slot",
        scene_link_node_slot,
    )?;
    let instance_count = field(packet, op, &["count"], "instance", "count", 3)?;
    let instance_stride = field(packet, op, &["stride"], "instance", "stride", 2)?;
    let instance_phase = field_with(packet, op, &["phase"], &["instance"], &["phase"], speed_key)?;
    let instance_material_slot = field(
        packet,
        op,
        &["material_slot"],
        "instance",
        "material_slot",
        scene_link_material_slot,
    )?;
    let instance_light_slot = field(
        packet,
        op,
        &["light_slot"],
        "instance",
        "light_slot",
        scene_link_light_slot,
    )?;
    let scene_graph_root_slot = field(
        packet,
        op,
        &["root_slot"],
        "scene_graph",
        "root_slot",
        scene_link_node_slot,
    )?;
    let scene_graph_node_count =
        field(packet, op, &["node_count"], "scene_graph", "node_count", 6)?;
    let scene_graph_link_count =
        field(packet, op, &["link_count"], "scene_graph", "link_count", 3)?;
    let scene_graph_instance_count = field(
        packet,
        op,
        &["instance_count"],
        "scene_graph",
        "instance_count",
        instance_count,
    )?;
    let scene_graph_active_layer = field(
        packet,
        op,
        &["active_layer"],
        "scene_graph",
        "active_layer",
        scene_link_layer_slot,
    )?;
    let scene_node_slot = field(
        packet,
        op,
        &["node_slot"],
        "scene_node",
        "node_slot",
        scene_graph_root_slot,
    )?;
    let scene_node_first_child_slot = field(
        packet,
        op,
        &["first_child_slot"],
        "scene_node",
        "first_child_slot",
        scene_link_transform_slot,
    )?;
    let scene_node_sibling_slot = field(
        packet,
        op,
        &["sibling_slot"],
        "scene_node",
        "sibling_slot",
        scene_link_mesh_slot,
    )?;
    let scene_node_instance_slot = field(
        packet,
        op,
        &["instance_slot"],
        "scene_node",
        "instance_slot",
        3,
    )?;
    let scene_node_visibility = field(packet, op, &["visibility"], "scene_node", "visibility", 1)?;
    let instance_group_root_slot = field(
        packet,
        op,
        &["root_instance_slot"],
        "instance_group",
        "root_instance_slot",
        scene_node_instance_slot,
    )?;
    let instance_group_count = field(
        packet,
        op,
        &["group_count"],
        "instance_group",
        "group_count",
        4,
    )?;
    let instance_group_visible_count = field(
        packet,
        op,
        &["visible_count"],
        "instance_group",
        "visible_count",
        instance_count,
    )?;
    let instance_group_phase_bias = field(
        packet,
        op,
        &["phase_bias"],
        "instance_group",
        "phase_bias",
        instance_phase,
    )?;
    let instance_group_material_slot = field(
        packet,
        op,
        &["material_slot"],
        "instance_group",
        "material_slot",
        instance_material_slot,
    )?;
    let scene_cluster_root_slot = field(
        packet,
        op,
        &["root_node_slot"],
        "scene_cluster",
        "root_node_slot",
        scene_node_slot,
    )?;
    let scene_cluster_node_budget = field(
        packet,
        op,
        &["node_budget"],
        "scene_cluster",
        "node_budget",
        scene_graph_node_count,
    )?;
    let scene_cluster_instance_group_slot = field(
        packet,
        op,
        &["instance_group_slot"],
        "scene_cluster",
        "instance_group_slot",
        instance_group_root_slot,
    )?;
    let scene_cluster_material_slot = field(
        packet,
        op,
        &["material_slot"],
        "scene_cluster",
        "material_slot",
        instance_group_material_slot,
    )?;
    let scene_cluster_layer_slot = field(
        packet,
        op,
        &["layer_slot"],
        "scene_cluster",
        "layer_slot",
        scene_graph_active_layer,
    )?;

    Ok(BallPacketSceneCoreFields {
        accent,
        surface,
        panel_mode,
        contrast,
        surface_density,
        surface_elevation,
        surface_grid,
        surface_sheen,
        viewport_x,
        viewport_y,
        viewport_width,
        viewport_height,
        layer_order,
        layer_blend,
        layer_visibility,
        layer_clip,
        scene_root_count,
        scene_active_camera,
        scene_light_count,
        scene_animation_phase,
        camera_kind,
        camera_focus,
        camera_zoom,
        camera_orbit,
        material_shader_kind,
        material_albedo,
        material_roughness,
        material_emissive,
        light_kind,
        light_intensity,
        light_range,
        light_reactive,
        mesh_primitive,
        mesh_vertex_count,
        mesh_index_count,
        mesh_skinning,
        transform_translate,
        transform_rotate,
        transform_scale,
        transform_pivot,
        node_id,
        node_parent_id,
        node_flags,
        node_depth,
        scene_link_node_slot,
        scene_link_transform_slot,
        scene_link_mesh_slot,
        scene_link_material_slot,
        scene_link_light_slot,
        scene_link_layer_slot,
        instance_node_slot,
        instance_count,
        instance_stride,
        instance_phase,
        instance_material_slot,
        instance_light_slot,
        scene_graph_root_slot,
        scene_graph_node_count,
        scene_graph_link_count,
        scene_graph_instance_count,
        scene_graph_active_layer,
        scene_node_slot,
        scene_node_first_child_slot,
        scene_node_sibling_slot,
        scene_node_instance_slot,
        scene_node_visibility,
        instance_group_root_slot,
        instance_group_count,
        instance_group_visible_count,
        instance_group_phase_bias,
        instance_group_material_slot,
        scene_cluster_root_slot,
        scene_cluster_node_budget,
        scene_cluster_instance_group_slot,
        scene_cluster_material_slot,
        scene_cluster_layer_slot,
    })
}
