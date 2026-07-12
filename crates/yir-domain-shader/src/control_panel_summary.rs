use super::surface_primitives::{fill_rect, put_text};
use super::BallPacket;

pub(crate) fn draw_control_panel_summary(
    rows: &mut [Vec<char>],
    panel_left: usize,
    panel_top: usize,
    panel_right: usize,
    viewport_width: usize,
    viewport_height: usize,
    layer_hidden: bool,
    packet: &BallPacket,
) {
    let status_bar_left = panel_left + 3;
    let status_bar_right = panel_right.saturating_sub(4);
    fill_rect(
        rows,
        status_bar_left,
        panel_top + 1,
        status_bar_right,
        panel_top + 1,
        '=',
    );
    put_text(
        rows,
        panel_left + 3,
        panel_top + 2,
        if packet.header_title_mode.rem_euclid(2) == 0 {
            if packet.panel_mode.rem_euclid(2) == 0 {
                "ns-nova control panel"
            } else {
                "ns-nova studio workspace"
            }
        } else if packet.panel_mode.rem_euclid(2) == 0 {
            "ns-nova reactive controls"
        } else {
            "ns-nova reactive cockpit"
        },
    );
    put_text(
        rows,
        panel_left + 3,
        panel_top + 3,
        "range / button / meter / text / select",
    );
    put_text(
        rows,
        panel_left + 34,
        panel_top + 3,
        "list / table / tree / inspector / outline",
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 2,
        &format!(
            "vp {}x{} @{},{}",
            viewport_width, viewport_height, packet.viewport_x, packet.viewport_y
        ),
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 4,
        &format!(
            "layer o{} b{} {}",
            packet.layer_order,
            packet.layer_blend,
            if layer_hidden { "hidden" } else { "live" }
        ),
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 5,
        &format!(
            "surf d{} e{} g{}",
            packet.surface_density, packet.surface_elevation, packet.surface_grid
        ),
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 6,
        &format!(
            "scene r{} l{} a{}",
            packet.scene_root_count, packet.scene_light_count, packet.scene_animation_phase
        ),
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 7,
        &format!(
            "cam k{} f{} z{}",
            packet.camera_kind, packet.camera_focus, packet.camera_zoom
        ),
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 8,
        &format!(
            "mat s{} r{} e{}",
            packet.material_shader_kind, packet.material_roughness, packet.material_emissive
        ),
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 9,
        &format!(
            "lit k{} i{} r{}",
            packet.light_kind, packet.light_intensity, packet.light_range
        ),
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 10,
        &format!(
            "mesh p{} v{} i{}",
            packet.mesh_primitive, packet.mesh_vertex_count, packet.mesh_index_count
        ),
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 11,
        &format!("skin {:>3}", packet.mesh_skinning),
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 12,
        &format!(
            "xform t{} r{} s{}",
            packet.transform_translate, packet.transform_rotate, packet.transform_scale
        ),
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 13,
        &format!("pivot {:>3}", packet.transform_pivot),
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 14,
        &format!(
            "node {}<-{} d{}",
            packet.node_id, packet.node_parent_id, packet.node_depth
        ),
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 15,
        &format!("flags {:>3}", packet.node_flags),
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 21,
        &format!(
            "link n{} m{}",
            packet.scene_link_node_slot, packet.scene_link_mesh_slot
        ),
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 22,
        &format!(
            "mat{} lit{}",
            packet.scene_link_material_slot, packet.scene_link_light_slot
        ),
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 16,
        &format!(
            "pass s{} c{} x{}",
            packet.pass_stage, packet.pass_clear_mode, packet.pass_sample_count
        ),
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 17,
        &format!("dbg {:>3}", packet.pass_debug_view),
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 18,
        &format!(
            "frm {:>3} pm{} v{}",
            packet.frame_index, packet.frame_present_mode, packet.frame_sync_interval
        ),
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 19,
        &format!("exp {:>3}", packet.frame_exposure),
    );
    put_text(
        rows,
        panel_right.saturating_sub(26),
        panel_top + 20,
        &format!(
            "tgt k{} {:>2}x{:>2}",
            packet.target_kind, packet.target_width, packet.target_height
        ),
    );
}
