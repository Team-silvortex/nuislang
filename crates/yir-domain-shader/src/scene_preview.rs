use super::geometry_overlay::stamp_line;
use super::scene_runtime_overlay::draw_scene_runtime_overlay;
use super::surface_primitives::{draw_box, fill_rect, put_text, BoxGlyphs};
use super::BallPacket;

pub(crate) fn draw_scene_preview(
    rows: &mut [Vec<char>],
    viewport_left: usize,
    viewport_top: usize,
    viewport_right: usize,
    viewport_bottom: usize,
    packet: &BallPacket,
    accent: char,
) {
    if viewport_right <= viewport_left + 6 || viewport_bottom <= viewport_top + 5 {
        return;
    }

    let preview_left = viewport_left + 2;
    let preview_top = viewport_top + 2;
    let preview_right = viewport_right.saturating_sub(2);
    let preview_bottom = viewport_bottom.saturating_sub(2);
    let width = preview_right.saturating_sub(preview_left).max(6);
    let ground_y = preview_bottom.saturating_sub(1);
    let object_y = ground_y
        .saturating_sub(2 + packet.node_depth.rem_euclid(2) as usize)
        .saturating_sub(packet.transform_pivot.rem_euclid(2) as usize);
    let scene_phase = packet.transform_translate
        + packet.camera_orbit
        + packet.scene_link_node_slot
        + packet.frame_index;
    let object_x = preview_left + scene_phase.rem_euclid(width as i64) as usize;
    let light_x = preview_left + packet.light_range.rem_euclid(width as i64) as usize;
    let light_y = preview_top + packet.scene_link_light_slot.rem_euclid(3) as usize;
    let radius =
        ((packet.transform_scale.abs() + packet.mesh_vertex_count.abs()) / 24).clamp(1, 4) as usize;
    let glyph = match (packet.mesh_primitive + packet.material_shader_kind).rem_euclid(4) {
        0 => '#',
        1 => '@',
        2 => '%',
        _ => '&',
    };
    let shadow = if packet.layer_visibility == 0 {
        ':'
    } else {
        '_'
    };

    for x in preview_left..=preview_right {
        if x < rows[ground_y].len() {
            rows[ground_y][x] = if x % 2 == 0 { '_' } else { '.' };
        }
    }

    let shadow_left = object_x.saturating_sub(radius + 1).max(preview_left);
    let shadow_right = (object_x + radius + 1).min(preview_right);
    for x in shadow_left..=shadow_right {
        if x < rows[ground_y].len() {
            rows[ground_y][x] = shadow;
        }
    }

    if packet.light_intensity > 0 {
        if let Some(row) = rows.get_mut(light_y) {
            let light_slot = light_x.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(light_slot) {
                *cell = '*';
            }
        }
        stamp_line(rows, light_x, light_y, object_x, object_y, '.');
    }

    match packet.mesh_primitive.rem_euclid(3) {
        0 => {
            let top_y = object_y.saturating_sub(radius);
            let left_x = object_x.saturating_sub(radius).max(preview_left);
            let right_x = (object_x + radius).min(preview_right);
            stamp_line(rows, object_x, top_y, left_x, object_y + radius, glyph);
            stamp_line(rows, object_x, top_y, right_x, object_y + radius, glyph);
            stamp_line(
                rows,
                left_x,
                object_y + radius,
                right_x,
                object_y + radius,
                glyph,
            );
        }
        1 => {
            let left_x = object_x.saturating_sub(radius).max(preview_left);
            let right_x = (object_x + radius).min(preview_right);
            let top_y = object_y.saturating_sub(radius).max(preview_top);
            let bottom_y = (object_y + radius).min(ground_y.saturating_sub(1));
            draw_box(
                rows,
                left_x,
                top_y,
                right_x,
                bottom_y,
                BoxGlyphs::new(glyph, glyph, glyph, glyph, glyph, glyph),
            );
            if left_x + 1 < right_x && top_y + 1 < bottom_y {
                fill_rect(
                    rows,
                    left_x + 1,
                    top_y + 1,
                    right_x - 1,
                    bottom_y - 1,
                    glyph,
                );
            }
        }
        _ => {
            let top_y = object_y.saturating_sub(radius);
            let bottom_y = (object_y + radius).min(ground_y.saturating_sub(1));
            let left_x = object_x.saturating_sub(radius).max(preview_left);
            let right_x = (object_x + radius).min(preview_right);
            stamp_line(rows, object_x, top_y, left_x, object_y, glyph);
            stamp_line(rows, object_x, top_y, right_x, object_y, glyph);
            stamp_line(rows, left_x, object_y, object_x, bottom_y, glyph);
            stamp_line(rows, right_x, object_y, object_x, bottom_y, glyph);
        }
    }

    if let Some(row) = rows.get_mut(object_y.min(rows.len().saturating_sub(1))) {
        let object_slot = object_x.min(row.len().saturating_sub(1));
        if let Some(cell) = row.get_mut(object_slot) {
            *cell = accent;
        }
    }

    let link_label = format!(
        "n{} t{} m{}",
        packet.scene_link_node_slot, packet.scene_link_transform_slot, packet.scene_link_mesh_slot
    );
    put_text(rows, preview_left, preview_bottom, &link_label);
    let material_label = format!(
        "mat{} lit{} ly{} i{}",
        packet.scene_link_material_slot,
        packet.scene_link_light_slot,
        packet.scene_link_layer_slot,
        packet.instance_node_slot
    );
    put_text(
        rows,
        preview_left,
        preview_bottom.saturating_sub(1),
        &material_label,
    );
    let instance_label = format!(
        "c{} s{} p{} l{}",
        packet.instance_count,
        packet.instance_stride,
        packet.instance_phase.rem_euclid(10),
        packet.instance_light_slot
    );
    put_text(rows, preview_left, preview_top, &instance_label);
    let graph_label = format!(
        "g{} l{} i{} a{}",
        packet.scene_graph_node_count,
        packet.scene_graph_link_count,
        packet.scene_graph_instance_count,
        packet.scene_graph_active_layer
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(1),
        &graph_label,
    );
    let scene_node_label = format!(
        "sn{} c{} s{} i{} v{}",
        packet.scene_node_slot,
        packet.scene_node_first_child_slot,
        packet.scene_node_sibling_slot,
        packet.scene_node_instance_slot,
        packet.scene_node_visibility
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(2),
        &scene_node_label,
    );
    let group_label = format!(
        "ig{} g{} v{} p{}",
        packet.instance_group_root_slot,
        packet.instance_group_count,
        packet.instance_group_visible_count,
        packet.instance_group_phase_bias.rem_euclid(10)
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(3),
        &group_label,
    );
    let cluster_label = format!(
        "cl{} n{} g{} l{}",
        packet.scene_cluster_root_slot,
        packet.scene_cluster_node_budget,
        packet.scene_cluster_instance_group_slot,
        packet.scene_cluster_layer_slot
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(4),
        &cluster_label,
    );
    let visibility_label = format!(
        "vs{} v{} o{} d{}",
        packet.visibility_cluster_slot,
        packet.visibility_visible_nodes,
        packet.visibility_occlusion_mode,
        packet.visibility_distance_band
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(5),
        &visibility_label,
    );
    let cull_label = format!(
        "cu{} k{} m{} l{}",
        packet.cull_cluster_slot, packet.cull_kept_nodes, packet.cull_mode, packet.cull_lod_band
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(6),
        &cull_label,
    );
    let lod_label = format!(
        "ld{} n{} a{} s{}",
        packet.lod_cluster_slot,
        packet.lod_level_count,
        packet.lod_active_level,
        packet.lod_switch_distance
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(7),
        &lod_label,
    );
    let streaming_label = format!(
        "st{} r{} p{} e{}",
        packet.streaming_cluster_slot,
        packet.streaming_resident_levels,
        packet.streaming_prefetch_mode,
        packet.streaming_evict_budget
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(8),
        &streaming_label,
    );
    let residency_label = format!(
        "rs{} c{} m{} s{}",
        packet.residency_cluster_slot,
        packet.residency_committed_levels,
        packet.residency_mode,
        packet.residency_spill_budget
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(9),
        &residency_label,
    );
    let eviction_label = format!(
        "ev{} n{} m{} r{}",
        packet.eviction_cluster_slot,
        packet.eviction_levels,
        packet.eviction_mode,
        packet.eviction_reclaim_budget
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(10),
        &eviction_label,
    );
    let prefetch_label = format!(
        "pf{} q{} w{} b{}",
        packet.prefetch_cluster_slot,
        packet.prefetch_requested_levels,
        packet.prefetch_window,
        packet.prefetch_warm_budget
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(11),
        &prefetch_label,
    );
    let budget_label = format!(
        "bg{} t{} u{} h{}",
        packet.budget_cluster_slot, packet.budget_total, packet.budget_used, packet.budget_headroom
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(12),
        &budget_label,
    );
    let pressure_label = format!(
        "pr{} l{} s{} t{}",
        packet.pressure_cluster_slot,
        packet.pressure_level,
        packet.pressure_saturation,
        packet.pressure_throttled
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(13),
        &pressure_label,
    );
    let thermal_label = format!(
        "th{} l{} c{} t{}",
        packet.thermal_cluster_slot,
        packet.thermal_level,
        packet.thermal_cooling_mode,
        packet.thermal_throttled
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(14),
        &thermal_label,
    );
    let power_label = format!(
        "pw{} l{} s{} c{}",
        packet.power_cluster_slot,
        packet.power_level,
        packet.power_source_mode,
        packet.power_capped
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(15),
        &power_label,
    );
    let frame_pacing_label = format!(
        "fp{} c{} v{} y{}",
        packet.frame_pacing_cluster_slot,
        packet.frame_pacing_cadence,
        packet.frame_pacing_variance,
        packet.frame_pacing_vsync_mode
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(17),
        &frame_pacing_label,
    );
    let frame_variance_label = format!(
        "fv{} f{} i{} b{}",
        packet.frame_variance_cluster_slot,
        packet.frame_variance_frame,
        packet.frame_variance_input,
        packet.frame_variance_burst
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(18),
        &frame_variance_label,
    );
    let jank_label = format!(
        "jk{} s{} v{} r{}",
        packet.jank_cluster_slot, packet.jank_spikes, packet.jank_severity, packet.jank_recovery
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(19),
        &jank_label,
    );
    let latency_label = format!(
        "lt{} f{} i{} j{}",
        packet.latency_cluster_slot,
        packet.latency_frame,
        packet.latency_input,
        packet.latency_jitter
    );
    put_text(
        rows,
        preview_left,
        preview_top.saturating_add(16),
        &latency_label,
    );

    let instance_count = packet.instance_count.clamp(1, 4) as usize;
    let instance_stride = packet.instance_stride.abs().clamp(2, 6) as usize;
    let mut last_x = object_x;
    for idx in 1..instance_count {
        let shifted_x = (object_x + idx * instance_stride)
            .min(preview_right.saturating_sub(1))
            .max(preview_left + 1);
        let shifted_y = object_y
            .saturating_add((packet.instance_phase + idx as i64).rem_euclid(2) as usize)
            .min(ground_y.saturating_sub(1));
        let ghost = match (packet.instance_material_slot + idx as i64).rem_euclid(3) {
            0 => ':',
            1 => ';',
            _ => '+',
        };
        if let Some(row) = rows.get_mut(shifted_y) {
            let shifted_slot = shifted_x.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(shifted_slot) {
                *cell = ghost;
            }
        }
        stamp_line(rows, last_x, object_y, shifted_x, shifted_y, '.');
        last_x = shifted_x;
    }

    let root_y = preview_top
        .saturating_add(packet.scene_graph_root_slot.rem_euclid(3) as usize)
        .min(ground_y.saturating_sub(2));
    let graph_span = packet.scene_graph_node_count.clamp(2, 6) as usize;
    for idx in 0..graph_span {
        let branch_x = preview_left
            .saturating_add(2 + idx * 2)
            .min(preview_right.saturating_sub(1));
        if let Some(row) = rows.get_mut(root_y) {
            let slot = branch_x.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = if idx == 0 { '@' } else { '|' };
            }
        }
        let depth_y = (root_y + 1 + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        if let Some(row) = rows.get_mut(depth_y) {
            let slot = branch_x.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = '.';
            }
        }
    }

    let node_y = root_y
        .saturating_add(1 + packet.scene_node_slot.rem_euclid(2) as usize)
        .min(ground_y.saturating_sub(1));
    let child_x = preview_left
        .saturating_add(3 + packet.scene_node_first_child_slot.rem_euclid(8) as usize)
        .min(preview_right.saturating_sub(1));
    let sibling_x = preview_left
        .saturating_add(5 + packet.scene_node_sibling_slot.rem_euclid(8) as usize)
        .min(preview_right.saturating_sub(1));
    let node_glyph = if packet.scene_node_visibility != 0 {
        '#'
    } else {
        'x'
    };
    if let Some(row) = rows.get_mut(node_y) {
        let slot = child_x.min(row.len().saturating_sub(1));
        if let Some(cell) = row.get_mut(slot) {
            *cell = node_glyph;
        }
    }
    stamp_line(rows, child_x, node_y, sibling_x, node_y, '=');

    let group_visible = packet.instance_group_visible_count.clamp(1, 4) as usize;
    let group_stride = (packet.instance_group_phase_bias.abs().clamp(2, 6)) as usize;
    for idx in 0..group_visible {
        let gx = preview_left
            .saturating_add(10 + idx * group_stride)
            .min(preview_right.saturating_sub(1));
        let gy = root_y
            .saturating_add(2 + idx.rem_euclid(2))
            .min(ground_y.saturating_sub(1));
        let glyph = match (packet.instance_group_material_slot + idx as i64).rem_euclid(3) {
            0 => '*',
            1 => '+',
            _ => '%',
        };
        if let Some(row) = rows.get_mut(gy) {
            let slot = gx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        stamp_line(rows, child_x, node_y, gx, gy, ':');
    }

    let cluster_span = packet.scene_cluster_node_budget.clamp(2, 5) as usize;
    let cluster_root_x = preview_left
        .saturating_add(18 + packet.scene_cluster_root_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let cluster_root_y = root_y.saturating_add(1).min(ground_y.saturating_sub(1));
    for idx in 0..cluster_span {
        let cx = (cluster_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let cy = (cluster_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.scene_cluster_material_slot + idx as i64).rem_euclid(3) {
            0 => 'o',
            1 => '0',
            _ => '8',
        };
        if let Some(row) = rows.get_mut(cy) {
            let slot = cx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        stamp_line(rows, cluster_root_x, cluster_root_y, cx, cy, '~');
    }

    draw_scene_runtime_overlay(
        rows,
        preview_left,
        preview_right,
        root_y,
        ground_y,
        cluster_root_x,
        cluster_root_y,
        packet,
    );
}
