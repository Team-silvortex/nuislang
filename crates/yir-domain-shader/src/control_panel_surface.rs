use super::control_panel_extended_summary::draw_control_panel_extended_summary;
use super::control_panel_layout::resolve_control_panel_layout;
use super::control_panel_summary::draw_control_panel_summary;
use super::control_panel_widgets::draw_control_panel_widgets;
use super::packet_helpers::normalize_control_value;
use super::parse_ball_packet;
use super::scene_preview::draw_scene_preview;
use super::surface_primitives::{
    control_panel_accent, draw_box, draw_card, fill_panel_background, fill_rect, put_text,
    BoxGlyphs,
};
use yir_core::{FrameSurface, Value};

pub(crate) fn draw_control_panel_surface(
    value: &Value,
    width: usize,
    height: usize,
) -> Result<FrameSurface, String> {
    let packet = parse_ball_packet(value, "shader.draw_instanced")?;
    let layout = resolve_control_panel_layout(
        width,
        height,
        packet.viewport_x,
        packet.viewport_y,
        packet.viewport_width,
        packet.viewport_height,
    );
    let width = layout.width;
    let height = layout.height;
    let color_value = normalize_control_value(packet.color_key, packet.color_min, packet.color_max);
    let speed_value = normalize_control_value(
        packet.speed.round() as i64,
        packet.speed_min,
        packet.speed_max,
    );
    let radius_value = normalize_control_value(
        (packet.radius_scale * 96.0).round() as i64,
        packet.radius_min,
        packet.radius_max,
    );
    let progress_value = normalize_control_value(packet.progress_value, 0, packet.progress_max);
    let meter_value = normalize_control_value(packet.meter_value, 0, packet.meter_max);
    let accent = control_panel_accent(packet.accent, packet.contrast);
    let button_on = packet.button_state != 0;
    let toggle_disabled = packet.toggle_disabled != 0;
    let viewport_width = layout.viewport_width;
    let viewport_height = layout.viewport_height;
    let layer_hidden = packet.layer_visibility == 0;
    let blend_fill = match packet.layer_blend.rem_euclid(3) {
        0 => '.',
        1 => ':',
        _ => ';',
    };

    let mut rows = vec![vec![' '; width]; height];
    fill_panel_background(
        &mut rows,
        packet.surface,
        packet.contrast + packet.surface_density + packet.surface_sheen,
    );
    let panel_left = layout.panel_left;
    let panel_top = layout.panel_top;
    let panel_right = layout.panel_right;
    let panel_bottom = layout.panel_bottom;
    let viewport_left = layout.viewport_left;
    let viewport_top = layout.viewport_top;
    let viewport_right = layout.viewport_right;
    let viewport_bottom = layout.viewport_bottom;

    draw_box(
        &mut rows,
        panel_left,
        panel_top,
        panel_right,
        panel_bottom,
        BoxGlyphs::new('/', '\\', '\\', '/', '-', '|'),
    );
    fill_rect(
        &mut rows,
        panel_left + 1,
        panel_top + 1,
        panel_right.saturating_sub(1),
        panel_bottom.saturating_sub(1),
        '.',
    );
    draw_card(
        &mut rows,
        viewport_left,
        viewport_top,
        viewport_right,
        viewport_bottom,
        accent,
        if packet.panel_mode.rem_euclid(2) == 0 {
            blend_fill
        } else {
            if packet.surface_grid.rem_euclid(2) == 0 {
                ':'
            } else {
                ';'
            }
        },
    );
    draw_card(
        &mut rows,
        panel_right.saturating_sub(29),
        panel_top + 3,
        panel_right.saturating_sub(2),
        panel_top + 13,
        accent,
        ':',
    );
    draw_card(
        &mut rows,
        panel_right.saturating_sub(29),
        panel_top + 14,
        panel_right.saturating_sub(2),
        panel_top + 20,
        accent,
        '.',
    );
    draw_card(
        &mut rows,
        panel_left + 2,
        panel_bottom.saturating_sub(9),
        panel_right.saturating_sub(30),
        panel_bottom.saturating_sub(1),
        accent,
        if packet.layer_clip.rem_euclid(2) == 0 {
            ':'
        } else {
            '.'
        },
    );
    draw_control_panel_summary(
        &mut rows,
        panel_left,
        panel_top,
        panel_right,
        viewport_width,
        viewport_height,
        layer_hidden,
        &packet,
    );
    draw_scene_preview(
        &mut rows,
        viewport_left,
        viewport_top,
        viewport_right,
        viewport_bottom,
        &packet,
        accent,
    );
    draw_control_panel_extended_summary(&mut rows, panel_top, panel_right, &packet);
    if layer_hidden {
        put_text(
            &mut rows,
            viewport_left + 3,
            viewport_top + 2,
            "layer hidden: overlay retained for debug",
        );
    }
    put_text(
        &mut rows,
        panel_right.saturating_sub(18),
        panel_top + 2,
        if toggle_disabled {
            "mode: locked"
        } else if packet.toggle_state != 0 {
            "mode: live"
        } else {
            "mode: idle"
        },
    );
    put_text(
        &mut rows,
        panel_left + 3,
        panel_top + 1,
        if packet.select_committed != 0 {
            "nova scene live graph"
        } else {
            "nova scene staging graph"
        },
    );
    put_text(
        &mut rows,
        panel_right.saturating_sub(28),
        panel_top + 1,
        &format!(
            "theme s{} m{} c{}",
            packet.surface.rem_euclid(10),
            packet.panel_mode.rem_euclid(10),
            packet.contrast.rem_euclid(10)
        ),
    );

    draw_control_panel_widgets(
        &mut rows,
        &packet,
        panel_left,
        panel_top,
        panel_right,
        panel_bottom,
        accent,
        color_value,
        speed_value,
        radius_value,
        progress_value,
        meter_value,
        button_on,
        toggle_disabled,
    );

    let rows = rows
        .into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<_>>();
    Ok(FrameSurface {
        width,
        height,
        rows,
    })
}
