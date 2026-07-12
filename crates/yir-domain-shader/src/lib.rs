mod control_panel_extended_summary;
mod control_panel_layout;
mod control_panel_summary;
mod describe;
mod execute_core;
mod execute_effects;
mod flow_state;
mod frame_surface;
mod geometry_overlay;
mod packet_helpers;
mod parse_ball_packet_tuple;
mod render_pass;
mod sphere_render;
mod surface_primitives;
mod texture_sampling;

use control_panel_extended_summary::draw_control_panel_extended_summary;
use control_panel_layout::resolve_control_panel_layout;
use control_panel_summary::draw_control_panel_summary;
use describe::describe_shader_node;
use execute_core::execute_shader_core_node;
use execute_effects::execute_shader_effect_node;
use flow_state::parse_shader_flow_state;
use geometry_overlay::stamp_line;
use packet_helpers::{
    find_flat_packet_field, find_packet_field, find_slider_packet_field, find_slider_packet_value,
    normalize_control_value, scalar_to_color_key, scalar_to_f32,
};
use parse_ball_packet_tuple::parse_ball_packet_tuple;
use render_pass::draw_render_pass_surface;
use surface_primitives::{
    control_panel_accent, draw_box, draw_card, draw_knob, draw_slider, fill_panel_background,
    fill_rect, put_text, BoxGlyphs,
};
use yir_core::{
    ExecutionState, FrameSurface, InstructionSemantics, Node, RegisteredMod, Resource, StructValue,
    Value,
};

pub struct ShaderMod;

impl RegisteredMod for ShaderMod {
    fn module_name(&self) -> &'static str {
        "shader"
    }

    fn describe(&self, node: &Node, resource: &Resource) -> Result<InstructionSemantics, String> {
        describe_shader_node(node, resource)
    }

    fn execute(
        &self,
        node: &Node,
        resource: &Resource,
        state: &mut ExecutionState,
    ) -> Result<Value, String> {
        if let Some(value) = execute_shader_core_node(node, resource, state)? {
            return Ok(value);
        }

        if let Some(value) = execute_shader_effect_node(node, resource, state)? {
            return Ok(value);
        }

        Err(format!(
            "unknown shader instruction `{}`",
            node.op.instruction
        ))
    }
}

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

    let slider_left = panel_left + 16;
    let slider_right = panel_right.saturating_sub(12);
    let slider_width = slider_right.saturating_sub(slider_left + 1).max(8);
    let slider_specs = [
        (
            "COLOR",
            color_value,
            panel_top + 6,
            packet.color_disabled != 0,
            packet.color_min,
            packet.color_max,
            packet.color_step,
        ),
        (
            "SPEED",
            speed_value,
            panel_top + 10,
            packet.speed_disabled != 0,
            packet.speed_min,
            packet.speed_max,
            packet.speed_step,
        ),
        (
            "RADIUS",
            radius_value,
            panel_top + 14,
            packet.radius_disabled != 0,
            packet.radius_min,
            packet.radius_max,
            packet.radius_step,
        ),
    ];
    for (label, value, y, disabled, min_value, max_value, step_value) in slider_specs {
        put_text(&mut rows, panel_left + 3, y, label);
        draw_slider(
            &mut rows,
            slider_left,
            y,
            slider_width,
            value.min(127),
            if disabled { ':' } else { accent },
        );
        put_text(
            &mut rows,
            slider_right + 2,
            y,
            &format!("{:>3}", value.min(127)),
        );
        let meta = format!("{min_value}..{max_value} /{step_value}");
        put_text(&mut rows, slider_left, y.saturating_sub(1), &meta);
        if disabled {
            put_text(&mut rows, slider_right + 7, y, "off");
        }
    }

    put_text(
        &mut rows,
        panel_left + 4,
        panel_top + 12,
        &format!(
            "camera orbit {:>3}  albedo {:>3}",
            packet.camera_orbit, packet.material_albedo
        ),
    );
    put_text(
        &mut rows,
        panel_left + 4,
        panel_top + 13,
        &format!(
            "scene roots {:>2}  active {:>2}",
            packet.scene_root_count, packet.scene_active_camera
        ),
    );
    put_text(
        &mut rows,
        panel_left + 4,
        panel_top + 14,
        &format!("light reactive {:>3}", packet.light_reactive),
    );

    let progress_y = panel_top + 17;
    put_text(
        &mut rows,
        panel_left + 3,
        progress_y.saturating_sub(1),
        "frame",
    );
    put_text(&mut rows, panel_left + 3, progress_y, "PROGRESS");
    draw_slider(
        &mut rows,
        slider_left,
        progress_y,
        slider_width,
        progress_value.min(127),
        accent,
    );
    put_text(
        &mut rows,
        slider_right + 2,
        progress_y,
        &format!("{:>3}", progress_value.min(127)),
    );

    let meter_y = panel_top + 19;
    put_text(
        &mut rows,
        panel_left + 3,
        meter_y.saturating_sub(1),
        "energy",
    );
    put_text(&mut rows, panel_left + 3, meter_y, "METER");
    draw_slider(
        &mut rows,
        slider_left,
        meter_y,
        slider_width,
        meter_value.min(127),
        accent,
    );
    put_text(
        &mut rows,
        slider_right + 2,
        meter_y,
        &format!("{:>3}", meter_value.min(127)),
    );

    let button_left = panel_right.saturating_sub(15);
    let button_right = panel_right.saturating_sub(4);
    let button_top = panel_top + 3;
    let button_bottom = button_top + 3;
    draw_box(
        &mut rows,
        button_left,
        button_top,
        button_right,
        button_bottom,
        BoxGlyphs::new('[', ']', ']', '[', '=', '|'),
    );
    fill_rect(
        &mut rows,
        button_left + 1,
        button_top + 1,
        button_right.saturating_sub(1),
        button_bottom.saturating_sub(1),
        if toggle_disabled {
            '.'
        } else if button_on {
            accent
        } else {
            ':'
        },
    );
    put_text(
        &mut rows,
        button_left + 2,
        button_top + 1,
        match packet.button_intent.rem_euclid(3) {
            _ if toggle_disabled => "LOCK ",
            0 if button_on => "APPLY",
            0 => "READY",
            1 if button_on => "LIVE ",
            1 => "ARM  ",
            _ if button_on => "SYNC ",
            _ => "HOLD ",
        },
    );
    put_text(
        &mut rows,
        button_left + 2,
        button_bottom,
        if button_on { "pulse" } else { "standby" },
    );

    let knob_center_x = panel_left + 8;
    let knob_center_y = panel_bottom.saturating_sub(5);
    draw_knob(
        &mut rows,
        knob_center_x,
        knob_center_y,
        4,
        radius_value.min(127),
        accent,
    );
    put_text(
        &mut rows,
        panel_left + 3,
        panel_bottom.saturating_sub(1),
        "gain",
    );

    let text_left = panel_left + 4;
    let text_right = panel_left + 24;
    let text_top = panel_bottom.saturating_sub(5);
    let text_bottom = text_top + 2;
    draw_box(
        &mut rows,
        text_left,
        text_top,
        text_right,
        text_bottom,
        BoxGlyphs::new('[', ']', ']', '[', '-', '|'),
    );
    let text_value = format!("nova-{:03}", packet.text_echo.abs() % 1000);
    put_text(&mut rows, text_left + 2, text_top + 1, &text_value);
    if packet.text_placeholder.rem_euclid(2) != 0 {
        put_text(&mut rows, text_left + 2, text_top, "query");
    }
    if packet.text_read_only != 0 {
        put_text(&mut rows, text_right.saturating_sub(6), text_top, "ro");
    }
    if packet.text_dirty != 0 {
        put_text(&mut rows, text_right.saturating_sub(12), text_top, "dirty");
    }
    let caret_x =
        text_left + 2 + (packet.text_caret.rem_euclid(text_value.len() as i64 + 1) as usize);
    if caret_x < text_right {
        rows[text_bottom][caret_x] = accent;
    }

    let select_left = panel_right.saturating_sub(28);
    let select_y = panel_bottom.saturating_sub(3);
    let option_count = packet.select_options.clamp(2, 4) as usize;
    let labels = match option_count {
        2 => ["AUTO", "MAN ", "", ""],
        3 => ["AUTO", "MAN ", "GPU ", ""],
        _ => ["AUTO", "MAN ", "GPU ", "CPU "],
    };
    put_text(&mut rows, select_left, select_y, labels[0]);
    put_text(&mut rows, select_left + 7, select_y, labels[1]);
    if option_count >= 3 {
        put_text(&mut rows, select_left + 13, select_y, labels[2]);
    }
    if option_count >= 4 {
        put_text(&mut rows, select_left + 19, select_y, labels[3]);
    }
    if packet.select_multiple != 0 {
        put_text(&mut rows, select_left, select_y.saturating_sub(1), "multi");
    }
    put_text(
        &mut rows,
        select_left + 22,
        select_y,
        if packet.select_committed != 0 {
            "ok"
        } else {
            "pending"
        },
    );
    let selected_x = match packet.select_index.rem_euclid(option_count as i64) {
        0 => select_left.saturating_sub(2),
        1 => select_left + 5,
        2 => select_left + 11,
        _ => select_left + 17,
    };
    put_text(&mut rows, selected_x, select_y, ">");

    let checkbox_y = panel_top + 6;
    let checkbox_left = button_left;
    put_text(&mut rows, checkbox_left, checkbox_y, "CHECK");
    put_text(
        &mut rows,
        checkbox_left,
        checkbox_y + 1,
        if packet.checkbox_disabled != 0 {
            "[~] disabled"
        } else if packet.checkbox_checked != 0 {
            "[x] enabled "
        } else {
            "[ ] enabled "
        },
    );

    let radio_y = panel_top + 10;
    let radio_left = button_left;
    put_text(&mut rows, radio_left, radio_y, "RADIO");
    let radio_count = packet.radio_options.clamp(2, 4) as usize;
    for idx in 0..radio_count {
        let label = match idx {
            0 => "fast",
            1 => "safe",
            2 => "gpu ",
            _ => "cpu ",
        };
        let mark = if packet.radio_selected.rem_euclid(radio_count as i64) as usize == idx {
            "(*)"
        } else {
            "( )"
        };
        put_text(
            &mut rows,
            radio_left,
            radio_y + 1 + idx,
            &format!("{mark} {label}"),
        );
    }
    if packet.radio_disabled != 0 {
        put_text(&mut rows, radio_left + 8, radio_y, "off");
    }

    let tabs_y = panel_top + 4;
    let tabs_count = packet.tabs_count.clamp(2, 4) as usize;
    for idx in 0..tabs_count {
        let label = match idx {
            0 => "scene",
            1 => "logic",
            2 => "perf ",
            _ => "gpu  ",
        };
        let active = packet.tabs_active.rem_euclid(tabs_count as i64) as usize == idx;
        let compact = packet.tabs_compact != 0;
        let text = if active {
            if compact {
                "[*]"
            } else {
                "[tab]"
            }
        } else if compact {
            "[ ]"
        } else {
            "[---]"
        };
        put_text(
            &mut rows,
            panel_left + 3 + idx * 10,
            tabs_y,
            &format!("{text} {}", &label[..label.len().min(5)]),
        );
    }

    let textarea_left = panel_left + 27;
    let textarea_right = panel_right.saturating_sub(30);
    let textarea_top = panel_bottom.saturating_sub(8);
    let textarea_bottom = textarea_top + 4;
    draw_box(
        &mut rows,
        textarea_left,
        textarea_top,
        textarea_right,
        textarea_bottom,
        BoxGlyphs::new('[', ']', ']', '[', '-', '|'),
    );
    put_text(&mut rows, textarea_left + 2, textarea_top, "notes");
    let visible_lines = packet.textarea_lines.clamp(2, 3) as usize;
    for line in 0..visible_lines {
        let scroll = packet.textarea_scroll.rem_euclid(9) as usize;
        let text = format!(
            "line {} :: {}",
            line + 1 + scroll,
            packet.textarea_placeholder
        );
        put_text(&mut rows, textarea_left + 2, textarea_top + 1 + line, &text);
    }
    if packet.textarea_read_only != 0 {
        put_text(
            &mut rows,
            textarea_right.saturating_sub(6),
            textarea_top,
            "ro",
        );
    }
    if packet.textarea_dirty != 0 {
        put_text(
            &mut rows,
            textarea_right.saturating_sub(13),
            textarea_top,
            "dirty",
        );
    }

    let list_left = panel_left + 3;
    let list_top = panel_bottom.saturating_sub(8);
    put_text(&mut rows, list_left, list_top, "list");
    let list_items = packet.list_items.clamp(3, 5) as usize;
    for idx in 0..list_items {
        let marker = if packet.list_selected.rem_euclid(list_items as i64) as usize == idx {
            ">"
        } else {
            " "
        };
        let row = if packet.list_dense != 0 {
            format!("{marker} item-{}", idx + 1)
        } else {
            format!(
                "{marker} row {}  accent {}",
                idx + 1,
                packet.accent.rem_euclid(9)
            )
        };
        put_text(&mut rows, list_left, list_top + 1 + idx, &row);
    }

    let table_left = panel_left + 50;
    let table_top = panel_bottom.saturating_sub(8);
    put_text(&mut rows, table_left, table_top, "table");
    let rows_count = packet.table_rows.clamp(2, 4) as usize;
    let cols_count = packet.table_cols.clamp(2, 4) as usize;
    let mut header = String::from("+");
    for _ in 0..cols_count {
        header.push_str("---+");
    }
    put_text(&mut rows, table_left, table_top + 1, &header);
    for row_idx in 0..rows_count {
        let mut body = String::from("|");
        for col_idx in 0..cols_count {
            let glyph = if packet.table_zebra != 0 && row_idx % 2 == 1 {
                ':'
            } else {
                '.'
            };
            let active =
                packet.table_selected_row.rem_euclid(rows_count as i64) as usize == row_idx;
            let cell = if active && col_idx == 0 {
                format!(">{glyph}{glyph}")
            } else {
                format!("{glyph}{glyph}{glyph}")
            };
            body.push_str(&cell);
            body.push('|');
        }
        put_text(&mut rows, table_left, table_top + 2 + row_idx, &body);
    }

    let tree_left = panel_right.saturating_sub(26);
    let tree_top = panel_top + 15;
    put_text(&mut rows, tree_left, tree_top, "tree");
    let node_count = packet.tree_nodes.clamp(3, 6) as usize;
    for idx in 0..node_count {
        let selected = packet.tree_selected.rem_euclid(node_count as i64) as usize == idx;
        let expanded = packet.tree_expanded != 0;
        let prefix = match idx {
            0 => {
                if expanded {
                    "v root"
                } else {
                    "> root"
                }
            }
            1 | 2 => "  |- child",
            _ => "  `- leaf ",
        };
        let line = if selected {
            format!("> {prefix}{}", idx + 1)
        } else {
            format!("  {prefix}{}", idx + 1)
        };
        put_text(&mut rows, tree_left, tree_top + 1 + idx, &line);
    }

    let inspector_left = panel_right.saturating_sub(26);
    let inspector_top = panel_top + 4;
    put_text(&mut rows, inspector_left, inspector_top, "inspector");
    put_text(
        &mut rows,
        inspector_left,
        inspector_top + 1,
        if packet.inspector_pinned != 0 {
            "[pin] locked"
        } else {
            "[pin] float "
        },
    );
    let inspector_fields = packet.inspector_fields.clamp(2, 4) as usize;
    for idx in 0..inspector_fields {
        let selected = packet
            .inspector_selected
            .rem_euclid(inspector_fields as i64) as usize
            == idx;
        let line = if selected {
            format!(
                "> field_{} = {}",
                idx + 1,
                packet.accent.rem_euclid(9) + idx as i64
            )
        } else {
            format!(
                "  field_{} = {}",
                idx + 1,
                packet.accent.rem_euclid(9) + idx as i64
            )
        };
        put_text(&mut rows, inspector_left, inspector_top + 2 + idx, &line);
    }

    let outline_left = panel_left + 3;
    let outline_top = panel_top + 17;
    put_text(&mut rows, outline_left, outline_top, "outline");
    let outline_items = packet.outline_items.clamp(3, 6) as usize;
    for idx in 0..outline_items {
        let selected = packet.outline_selected.rem_euclid(outline_items as i64) as usize == idx;
        let collapsed = packet.outline_collapsed != 0;
        let line = if idx == 0 {
            if selected {
                if collapsed {
                    "> > section".to_owned()
                } else {
                    "> v section".to_owned()
                }
            } else if collapsed {
                "  > section".to_owned()
            } else {
                "  v section".to_owned()
            }
        } else if collapsed {
            if selected {
                format!("> hidden {}", idx + 1)
            } else {
                format!("  hidden {}", idx + 1)
            }
        } else if selected {
            format!("> item {}", idx + 1)
        } else {
            format!("  item {}", idx + 1)
        };
        put_text(&mut rows, outline_left, outline_top + 1 + idx, &line);
    }

    let focus_target = packet.focus_index.rem_euclid(6) as usize;
    let focus_marker = match focus_target {
        0 => (slider_left.saturating_sub(3), panel_top + 6),
        1 => (slider_left.saturating_sub(3), panel_top + 10),
        2 => (slider_left.saturating_sub(3), panel_top + 14),
        3 => (button_left.saturating_sub(2), button_top + 1),
        4 => (text_left.saturating_sub(2), text_top + 1),
        _ => (select_left.saturating_sub(2), select_y),
    };
    if focus_marker.1 < rows.len() && focus_marker.0 < rows[focus_marker.1].len() {
        rows[focus_marker.1][focus_marker.0] = '>';
    }

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

fn draw_scene_preview(
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

    let visibility_span = packet.visibility_visible_nodes.clamp(1, 5) as usize;
    let vis_root_x = preview_left
        .saturating_add(24 + packet.visibility_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let vis_root_y = root_y.saturating_add(2).min(ground_y.saturating_sub(1));
    for idx in 0..visibility_span {
        let vx = (vis_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let vy = (vis_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.visibility_mask + idx as i64).rem_euclid(4) {
            0 => 'v',
            1 => 'V',
            2 => '^',
            _ => '/',
        };
        if let Some(row) = rows.get_mut(vy) {
            let slot = vx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.visibility_occlusion_mode != 0 {
            '!'
        } else {
            '.'
        };
        stamp_line(rows, cluster_root_x, cluster_root_y, vx, vy, connector);
    }

    let cull_span = packet.cull_kept_nodes.clamp(1, 4) as usize;
    let cull_root_x = preview_left
        .saturating_add(30 + packet.cull_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let cull_root_y = root_y.saturating_add(1).min(ground_y.saturating_sub(1));
    for idx in 0..cull_span {
        let cx = (cull_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let cy = (cull_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.cull_mask + idx as i64).rem_euclid(4) {
            0 => 'c',
            1 => 'C',
            2 => '<',
            _ => '>',
        };
        if let Some(row) = rows.get_mut(cy) {
            let slot = cx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.cull_mode != 0 { '-' } else { '_' };
        stamp_line(rows, vis_root_x, vis_root_y, cx, cy, connector);
    }

    let lod_span = packet.lod_level_count.clamp(1, 4) as usize;
    let lod_root_x = preview_left
        .saturating_add(36 + packet.lod_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let lod_root_y = root_y.min(ground_y.saturating_sub(1));
    for idx in 0..lod_span {
        let lx = (lod_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let ly = (lod_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = if idx as i64 == packet.lod_active_level.rem_euclid(lod_span as i64) {
            match packet.lod_bias.rem_euclid(3) {
                0 => 'L',
                1 => 'M',
                _ => 'H',
            }
        } else {
            '.'
        };
        if let Some(row) = rows.get_mut(ly) {
            let slot = lx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        stamp_line(rows, cull_root_x, cull_root_y, lx, ly, '=');
    }

    let streaming_span = packet.streaming_resident_levels.clamp(1, 4) as usize;
    let streaming_root_x = preview_left
        .saturating_add(42 + packet.streaming_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let streaming_root_y = root_y.saturating_add(1).min(ground_y.saturating_sub(1));
    for idx in 0..streaming_span {
        let sx = (streaming_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let sy = (streaming_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.streaming_channel + idx as i64).rem_euclid(4) {
            0 => 's',
            1 => '$',
            2 => '~',
            _ => '+',
        };
        if let Some(row) = rows.get_mut(sy) {
            let slot = sx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.streaming_prefetch_mode != 0 {
            ':'
        } else {
            '.'
        };
        stamp_line(rows, lod_root_x, lod_root_y, sx, sy, connector);
    }

    let residency_span = packet.residency_committed_levels.clamp(1, 4) as usize;
    let residency_root_x = preview_left
        .saturating_add(48 + packet.residency_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let residency_root_y = root_y.min(ground_y.saturating_sub(1));
    for idx in 0..residency_span {
        let rx = (residency_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let ry = (residency_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.residency_mask + idx as i64).rem_euclid(4) {
            0 => 'r',
            1 => 'R',
            2 => '#',
            _ => '%',
        };
        if let Some(row) = rows.get_mut(ry) {
            let slot = rx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.residency_mode != 0 { ';' } else { ',' };
        stamp_line(rows, streaming_root_x, streaming_root_y, rx, ry, connector);
    }

    let eviction_span = packet.eviction_levels.clamp(1, 4) as usize;
    let eviction_root_x = preview_left
        .saturating_add(54 + packet.eviction_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let eviction_root_y = root_y.saturating_add(1).min(ground_y.saturating_sub(1));
    for idx in 0..eviction_span {
        let ex = (eviction_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let ey = (eviction_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.eviction_mask + idx as i64).rem_euclid(4) {
            0 => 'e',
            1 => 'E',
            2 => 'x',
            _ => 'X',
        };
        if let Some(row) = rows.get_mut(ey) {
            let slot = ex.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.eviction_mode != 0 { '!' } else { ':' };
        stamp_line(rows, residency_root_x, residency_root_y, ex, ey, connector);
    }

    let prefetch_span = packet.prefetch_requested_levels.clamp(1, 4) as usize;
    let prefetch_root_x = preview_left
        .saturating_add(60 + packet.prefetch_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let prefetch_root_y = root_y.min(ground_y.saturating_sub(1));
    for idx in 0..prefetch_span {
        let px = (prefetch_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let py = (prefetch_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.prefetch_mask + idx as i64).rem_euclid(4) {
            0 => 'p',
            1 => 'P',
            2 => '?',
            _ => '*',
        };
        if let Some(row) = rows.get_mut(py) {
            let slot = px.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.prefetch_window != 0 {
            '/'
        } else {
            '.'
        };
        stamp_line(rows, eviction_root_x, eviction_root_y, px, py, connector);
    }

    let budget_span = packet.budget_total.clamp(1, 4) as usize;
    let budget_root_x = preview_left
        .saturating_add(66 + packet.budget_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let budget_root_y = root_y.saturating_add(1).min(ground_y.saturating_sub(1));
    for idx in 0..budget_span {
        let bx = (budget_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let by = (budget_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.budget_policy + idx as i64).rem_euclid(4) {
            0 => 'b',
            1 => 'B',
            2 => '=',
            _ => '+',
        };
        if let Some(row) = rows.get_mut(by) {
            let slot = bx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.budget_used > packet.budget_headroom {
            '!'
        } else {
            '-'
        };
        stamp_line(rows, prefetch_root_x, prefetch_root_y, bx, by, connector);
    }

    let pressure_span = packet.pressure_level.clamp(1, 4) as usize;
    let pressure_root_x = preview_left
        .saturating_add(72 + packet.pressure_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let pressure_root_y = root_y.min(ground_y.saturating_sub(1));
    for idx in 0..pressure_span {
        let px = (pressure_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let py = (pressure_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.pressure_mask + idx as i64).rem_euclid(4) {
            0 => 'p',
            1 => '!',
            2 => '^',
            _ => 'P',
        };
        if let Some(row) = rows.get_mut(py) {
            let slot = px.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.pressure_throttled != 0 {
            '!'
        } else {
            '~'
        };
        stamp_line(rows, budget_root_x, budget_root_y, px, py, connector);
    }

    let thermal_span = packet.thermal_level.clamp(1, 4) as usize;
    let thermal_root_x = preview_left
        .saturating_add(78 + packet.thermal_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let thermal_root_y = root_y.saturating_add(1).min(ground_y.saturating_sub(1));
    for idx in 0..thermal_span {
        let tx = (thermal_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let ty = (thermal_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.thermal_mask + idx as i64).rem_euclid(4) {
            0 => 't',
            1 => 'T',
            2 => '*',
            _ => '!',
        };
        if let Some(row) = rows.get_mut(ty) {
            let slot = tx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.thermal_throttled != 0 {
            '#'
        } else {
            '~'
        };
        stamp_line(rows, pressure_root_x, pressure_root_y, tx, ty, connector);
    }

    let power_span = packet.power_level.clamp(1, 4) as usize;
    let power_root_x = preview_left
        .saturating_add(84 + packet.power_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let power_root_y = root_y.min(ground_y.saturating_sub(1));
    for idx in 0..power_span {
        let px = (power_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let py = (power_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.power_mask + idx as i64).rem_euclid(4) {
            0 => 'w',
            1 => 'W',
            2 => '=',
            _ => '!',
        };
        if let Some(row) = rows.get_mut(py) {
            let slot = px.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.power_capped != 0 { '=' } else { '-' };
        stamp_line(rows, thermal_root_x, thermal_root_y, px, py, connector);
    }

    let latency_span = packet.latency_frame.clamp(1, 4) as usize;
    let latency_root_x = preview_left
        .saturating_add(90 + packet.latency_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let latency_root_y = root_y.saturating_add(1).min(ground_y.saturating_sub(1));
    for idx in 0..latency_span {
        let lx = (latency_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let ly = (latency_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.latency_mask + idx as i64).rem_euclid(4) {
            0 => 'l',
            1 => 'L',
            2 => '~',
            _ => '!',
        };
        if let Some(row) = rows.get_mut(ly) {
            let slot = lx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.latency_jitter != 0 { '~' } else { '.' };
        stamp_line(rows, power_root_x, power_root_y, lx, ly, connector);
    }

    let frame_pacing_span = packet.frame_pacing_cadence.clamp(1, 4) as usize;
    let frame_pacing_root_x = preview_left
        .saturating_add(96 + packet.frame_pacing_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let frame_pacing_root_y = root_y.min(ground_y.saturating_sub(1));
    for idx in 0..frame_pacing_span {
        let fx = (frame_pacing_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let fy = (frame_pacing_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.frame_pacing_mask + idx as i64).rem_euclid(4) {
            0 => 'f',
            1 => 'F',
            2 => '|',
            _ => '!',
        };
        if let Some(row) = rows.get_mut(fy) {
            let slot = fx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.frame_pacing_vsync_mode != 0 {
            '|'
        } else {
            ':'
        };
        stamp_line(rows, latency_root_x, latency_root_y, fx, fy, connector);
    }

    let frame_variance_span = packet.frame_variance_frame.clamp(1, 4) as usize;
    let frame_variance_root_x = preview_left
        .saturating_add(99 + packet.frame_variance_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let frame_variance_root_y = root_y.saturating_add(1).min(ground_y.saturating_sub(1));
    for idx in 0..frame_variance_span {
        let vx = (frame_variance_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let vy = (frame_variance_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.frame_variance_mask + idx as i64).rem_euclid(4) {
            0 => 'v',
            1 => 'V',
            2 => '/',
            _ => '!',
        };
        if let Some(row) = rows.get_mut(vy) {
            let slot = vx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.frame_variance_burst != 0 {
            '/'
        } else {
            ':'
        };
        stamp_line(
            rows,
            frame_pacing_root_x,
            frame_pacing_root_y,
            vx,
            vy,
            connector,
        );
    }

    let jank_span = packet.jank_spikes.clamp(1, 4) as usize;
    let jank_root_x = preview_left
        .saturating_add(105 + packet.jank_cluster_slot.rem_euclid(4) as usize)
        .min(preview_right.saturating_sub(1));
    let jank_root_y = root_y.saturating_add(2).min(ground_y.saturating_sub(1));
    for idx in 0..jank_span {
        let jx = (jank_root_x + idx * 2).min(preview_right.saturating_sub(1));
        let jy = (jank_root_y + idx.rem_euclid(2)).min(ground_y.saturating_sub(1));
        let glyph = match (packet.jank_mask + idx as i64).rem_euclid(4) {
            0 => 'j',
            1 => 'J',
            2 => '*',
            _ => '!',
        };
        if let Some(row) = rows.get_mut(jy) {
            let slot = jx.min(row.len().saturating_sub(1));
            if let Some(cell) = row.get_mut(slot) {
                *cell = glyph;
            }
        }
        let connector = if packet.jank_recovery != 0 { '^' } else { ':' };
        stamp_line(
            rows,
            frame_variance_root_x,
            frame_variance_root_y,
            jx,
            jy,
            connector,
        );
    }
}

#[derive(Debug, Clone, Copy)]
struct BallPacket {
    color_key: i64,
    speed: f32,
    radius_scale: f32,
    color_min: i64,
    color_max: i64,
    color_step: i64,
    color_disabled: i64,
    speed_min: i64,
    speed_max: i64,
    speed_step: i64,
    speed_disabled: i64,
    radius_min: i64,
    radius_max: i64,
    radius_step: i64,
    radius_disabled: i64,
    accent: i64,
    surface: i64,
    panel_mode: i64,
    contrast: i64,
    surface_density: i64,
    surface_elevation: i64,
    surface_grid: i64,
    surface_sheen: i64,
    viewport_x: i64,
    viewport_y: i64,
    viewport_width: i64,
    viewport_height: i64,
    layer_order: i64,
    layer_blend: i64,
    layer_visibility: i64,
    layer_clip: i64,
    scene_root_count: i64,
    scene_active_camera: i64,
    scene_light_count: i64,
    scene_animation_phase: i64,
    camera_kind: i64,
    camera_focus: i64,
    camera_zoom: i64,
    camera_orbit: i64,
    material_shader_kind: i64,
    material_albedo: i64,
    material_roughness: i64,
    material_emissive: i64,
    light_kind: i64,
    light_intensity: i64,
    light_range: i64,
    light_reactive: i64,
    mesh_primitive: i64,
    mesh_vertex_count: i64,
    mesh_index_count: i64,
    mesh_skinning: i64,
    transform_translate: i64,
    transform_rotate: i64,
    transform_scale: i64,
    transform_pivot: i64,
    node_id: i64,
    node_parent_id: i64,
    node_flags: i64,
    node_depth: i64,
    scene_link_node_slot: i64,
    scene_link_transform_slot: i64,
    scene_link_mesh_slot: i64,
    scene_link_material_slot: i64,
    scene_link_light_slot: i64,
    scene_link_layer_slot: i64,
    instance_node_slot: i64,
    instance_count: i64,
    instance_stride: i64,
    instance_phase: i64,
    instance_material_slot: i64,
    instance_light_slot: i64,
    scene_graph_root_slot: i64,
    scene_graph_node_count: i64,
    scene_graph_link_count: i64,
    scene_graph_instance_count: i64,
    scene_graph_active_layer: i64,
    scene_node_slot: i64,
    scene_node_first_child_slot: i64,
    scene_node_sibling_slot: i64,
    scene_node_instance_slot: i64,
    scene_node_visibility: i64,
    instance_group_root_slot: i64,
    instance_group_count: i64,
    instance_group_visible_count: i64,
    instance_group_phase_bias: i64,
    instance_group_material_slot: i64,
    scene_cluster_root_slot: i64,
    scene_cluster_node_budget: i64,
    scene_cluster_instance_group_slot: i64,
    scene_cluster_material_slot: i64,
    scene_cluster_layer_slot: i64,
    visibility_cluster_slot: i64,
    visibility_visible_nodes: i64,
    visibility_occlusion_mode: i64,
    visibility_distance_band: i64,
    visibility_mask: i64,
    cull_cluster_slot: i64,
    cull_kept_nodes: i64,
    cull_mode: i64,
    cull_lod_band: i64,
    cull_mask: i64,
    lod_cluster_slot: i64,
    lod_level_count: i64,
    lod_active_level: i64,
    lod_switch_distance: i64,
    lod_bias: i64,
    streaming_cluster_slot: i64,
    streaming_resident_levels: i64,
    streaming_prefetch_mode: i64,
    streaming_evict_budget: i64,
    streaming_channel: i64,
    residency_cluster_slot: i64,
    residency_committed_levels: i64,
    residency_mode: i64,
    residency_spill_budget: i64,
    residency_mask: i64,
    eviction_cluster_slot: i64,
    eviction_levels: i64,
    eviction_mode: i64,
    eviction_reclaim_budget: i64,
    eviction_mask: i64,
    prefetch_cluster_slot: i64,
    prefetch_requested_levels: i64,
    prefetch_window: i64,
    prefetch_warm_budget: i64,
    prefetch_mask: i64,
    budget_cluster_slot: i64,
    budget_total: i64,
    budget_used: i64,
    budget_headroom: i64,
    budget_policy: i64,
    pressure_cluster_slot: i64,
    pressure_level: i64,
    pressure_saturation: i64,
    pressure_throttled: i64,
    pressure_mask: i64,
    thermal_cluster_slot: i64,
    thermal_level: i64,
    thermal_cooling_mode: i64,
    thermal_throttled: i64,
    thermal_mask: i64,
    power_cluster_slot: i64,
    power_level: i64,
    power_source_mode: i64,
    power_capped: i64,
    power_mask: i64,
    latency_cluster_slot: i64,
    latency_frame: i64,
    latency_input: i64,
    latency_jitter: i64,
    latency_mask: i64,
    frame_pacing_cluster_slot: i64,
    frame_pacing_cadence: i64,
    frame_pacing_variance: i64,
    frame_pacing_vsync_mode: i64,
    frame_pacing_mask: i64,
    frame_variance_cluster_slot: i64,
    frame_variance_frame: i64,
    frame_variance_input: i64,
    frame_variance_burst: i64,
    frame_variance_mask: i64,
    jank_cluster_slot: i64,
    jank_spikes: i64,
    jank_severity: i64,
    jank_recovery: i64,
    jank_mask: i64,
    pass_stage: i64,
    pass_clear_mode: i64,
    pass_sample_count: i64,
    pass_debug_view: i64,
    frame_index: i64,
    frame_present_mode: i64,
    frame_sync_interval: i64,
    frame_exposure: i64,
    target_kind: i64,
    target_width: i64,
    target_height: i64,
    target_multisample: i64,
    frame_graph_passes: i64,
    frame_graph_targets: i64,
    frame_graph_present_stage: i64,
    frame_graph_debug_overlay: i64,
    attachment_slot: i64,
    attachment_format_kind: i64,
    attachment_load_op: i64,
    attachment_store_op: i64,
    pass_chain_stages: i64,
    pass_chain_fanout: i64,
    pass_chain_resolve_stage: i64,
    pass_chain_barrier_mode: i64,
    barrier_scope: i64,
    barrier_source_stage: i64,
    barrier_target_stage: i64,
    barrier_flush_mode: i64,
    resource_buffers: i64,
    resource_textures: i64,
    resource_samplers: i64,
    resource_residency: i64,
    schedule_lanes: i64,
    schedule_queue_depth: i64,
    schedule_async_budget: i64,
    schedule_tick_mode: i64,
    submission_batches: i64,
    submission_fences: i64,
    submission_signal_mode: i64,
    submission_present_hint: i64,
    queue_kind: i64,
    queue_priority: i64,
    queue_budget: i64,
    queue_ownership: i64,
    semaphore_wait_count: i64,
    semaphore_signal_count: i64,
    semaphore_timeline_mode: i64,
    semaphore_scope: i64,
    timeline_value: i64,
    timeline_step: i64,
    timeline_epoch: i64,
    timeline_domain: i64,
    fence_signaled: i64,
    fence_epoch: i64,
    fence_scope: i64,
    fence_recycle_mode: i64,
    signal_kind: i64,
    signal_phase: i64,
    signal_fanout: i64,
    signal_ack_mode: i64,
    event_kind: i64,
    event_route: i64,
    event_priority: i64,
    event_payload_mode: i64,
    dispatch_queue_kind: i64,
    dispatch_lane: i64,
    dispatch_batch: i64,
    dispatch_completion_mode: i64,
    feedback_status: i64,
    feedback_latency: i64,
    feedback_retries: i64,
    feedback_channel: i64,
    intent_kind: i64,
    intent_target: i64,
    intent_urgency: i64,
    intent_policy: i64,
    reaction_kind: i64,
    reaction_result_slot: i64,
    reaction_stability: i64,
    reaction_echo_mode: i64,
    outcome_kind: i64,
    outcome_final_slot: i64,
    outcome_confidence: i64,
    outcome_settle_mode: i64,
    resolution_kind: i64,
    resolution_commit_slot: i64,
    resolution_convergence: i64,
    resolution_policy_mode: i64,
    commit_kind: i64,
    commit_applied_slot: i64,
    commit_durability: i64,
    commit_commit_mode: i64,
    snapshot_kind: i64,
    snapshot_source_slot: i64,
    snapshot_retention: i64,
    snapshot_replay_mode: i64,
    checkpoint_kind: i64,
    checkpoint_anchor_slot: i64,
    checkpoint_rollback_depth: i64,
    checkpoint_resume_mode: i64,
    toggle_state: i64,
    focus_index: i64,
    progress_value: i64,
    progress_max: i64,
    meter_value: i64,
    meter_max: i64,
    button_state: i64,
    button_intent: i64,
    header_title_mode: i64,
    toggle_disabled: i64,
    text_caret: i64,
    text_echo: i64,
    text_placeholder: i64,
    text_read_only: i64,
    text_dirty: i64,
    select_index: i64,
    select_options: i64,
    select_multiple: i64,
    select_committed: i64,
    checkbox_checked: i64,
    checkbox_disabled: i64,
    radio_selected: i64,
    radio_options: i64,
    radio_disabled: i64,
    textarea_lines: i64,
    textarea_scroll: i64,
    textarea_placeholder: i64,
    textarea_read_only: i64,
    textarea_dirty: i64,
    tabs_active: i64,
    tabs_count: i64,
    tabs_compact: i64,
    list_selected: i64,
    list_items: i64,
    list_dense: i64,
    table_rows: i64,
    table_cols: i64,
    table_selected_row: i64,
    table_zebra: i64,
    tree_selected: i64,
    tree_nodes: i64,
    tree_expanded: i64,
    inspector_selected: i64,
    inspector_fields: i64,
    inspector_pinned: i64,
    outline_selected: i64,
    outline_items: i64,
    outline_collapsed: i64,
}

fn parse_ball_packet(value: &Value, op: &str) -> Result<BallPacket, String> {
    match value {
        Value::Tuple(items) if items.len() >= 2 => parse_ball_packet_tuple(items, op),
        Value::Struct(packet) => parse_ball_packet_struct(packet, op),
        _ => Err(format!(
            "{op} expects a packet tuple `(color, speed[, radius_scale])` or struct with `color` and `speed`"
        )),
    }
}

fn parse_ball_packet_struct(packet: &StructValue, op: &str) -> Result<BallPacket, String> {
    let color = find_slider_packet_value(packet, "color")
        .or_else(|| find_flat_packet_field(packet, &["color", "slider_color"]))
        .ok_or_else(|| format!("{op} struct packet is missing `color` field"))?;
    let speed = find_slider_packet_value(packet, "speed")
        .or_else(|| find_flat_packet_field(packet, &["speed", "slider_speed"]))
        .ok_or_else(|| format!("{op} struct packet is missing `speed` field"))?;
    let radius_scale = find_slider_packet_value(packet, "radius")
        .or_else(|| find_flat_packet_field(packet, &["radius_scale", "slider_radius"]))
        .map(|value| scalar_to_f32(value, op))
        .transpose()?
        .unwrap_or(1.0);
    let accent = find_packet_field(
        packet,
        &["accent", "header_accent"],
        &["theme", "header"],
        &["accent"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or_else(|| scalar_to_color_key(color, op).unwrap_or(0));
    let surface = find_packet_field(packet, &["theme_surface"], &["theme"], &["surface"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 8.0).round() as i64);
    let panel_mode = find_packet_field(packet, &["panel_mode"], &["theme"], &["panel_mode"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent.rem_euclid(2));
    let contrast = find_packet_field(packet, &["contrast"], &["theme"], &["contrast"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let surface_density = find_packet_field(packet, &["density"], &["surface"], &["density"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let surface_elevation = find_packet_field(packet, &["elevation"], &["surface"], &["elevation"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 16.0).round() as i64);
    let surface_grid = find_packet_field(packet, &["grid"], &["surface"], &["grid"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent.rem_euclid(3));
    let surface_sheen = find_packet_field(packet, &["sheen"], &["surface"], &["sheen"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let viewport_x = find_packet_field(packet, &["origin_x"], &["viewport"], &["origin_x"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent.rem_euclid(4));
    let viewport_y = find_packet_field(packet, &["origin_y"], &["viewport"], &["origin_y"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let viewport_width = find_packet_field(packet, &["width"], &["viewport"], &["width"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(48);
    let viewport_height = find_packet_field(packet, &["height"], &["viewport"], &["height"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(18);
    let layer_order = find_packet_field(packet, &["order"], &["layer"], &["order"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(1);
    let layer_blend = find_packet_field(packet, &["blend"], &["layer"], &["blend"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let layer_visibility = find_packet_field(packet, &["visibility"], &["layer"], &["visibility"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(1);
    let layer_clip = find_packet_field(packet, &["clip"], &["layer"], &["clip"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 8.0).round() as i64);
    let scene_root_count = find_packet_field(packet, &["root_count"], &["scene"], &["root_count"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(7);
    let scene_active_camera =
        find_packet_field(packet, &["active_camera"], &["scene"], &["active_camera"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent.rem_euclid(6));
    let scene_light_count =
        find_packet_field(packet, &["light_count"], &["scene"], &["light_count"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(3);
    let scene_animation_phase = find_packet_field(
        packet,
        &["animation_phase"],
        &["scene"],
        &["animation_phase"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(contrast.rem_euclid(4));
    let camera_kind = find_packet_field(packet, &["kind"], &["camera"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let camera_focus = find_packet_field(packet, &["camera_focus"], &["camera"], &["focus"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent.rem_euclid(6));
    let camera_zoom = find_packet_field(packet, &["zoom"], &["camera"], &["zoom"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let camera_orbit = find_packet_field(packet, &["orbit"], &["camera"], &["orbit"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 12.0).round() as i64);
    let material_shader_kind =
        find_packet_field(packet, &["shader_kind"], &["material"], &["shader_kind"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast.rem_euclid(3));
    let material_albedo = find_packet_field(packet, &["albedo"], &["material"], &["albedo"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let material_roughness =
        find_packet_field(packet, &["roughness"], &["material"], &["roughness"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let material_emissive = find_packet_field(packet, &["emissive"], &["material"], &["emissive"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 24.0).round() as i64);
    let light_kind = find_packet_field(packet, &["kind"], &["light"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let light_intensity = find_packet_field(packet, &["intensity"], &["light"], &["intensity"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let light_range = find_packet_field(packet, &["range"], &["light"], &["range"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 18.0).round() as i64);
    let light_reactive = find_packet_field(packet, &["reactive"], &["light"], &["reactive"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let mesh_primitive = find_packet_field(packet, &["primitive"], &["mesh"], &["primitive"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let mesh_vertex_count =
        find_packet_field(packet, &["vertex_count"], &["mesh"], &["vertex_count"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(3));
    let mesh_index_count = find_packet_field(packet, &["index_count"], &["mesh"], &["index_count"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 18.0).round() as i64);
    let mesh_skinning = find_packet_field(packet, &["skinning"], &["mesh"], &["skinning"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let transform_translate =
        find_packet_field(packet, &["translate"], &["transform"], &["translate"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let transform_rotate = find_packet_field(packet, &["rotate"], &["transform"], &["rotate"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(4));
    let transform_scale = find_packet_field(packet, &["scale"], &["transform"], &["scale"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 16.0).round() as i64);
    let transform_pivot = find_packet_field(packet, &["pivot"], &["transform"], &["pivot"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent.rem_euclid(6));
    let node_id = find_packet_field(packet, &["node_id"], &["node"], &["node_id"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent.rem_euclid(8));
    let node_parent_id = find_packet_field(packet, &["parent_id"], &["node"], &["parent_id"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(4));
    let node_flags = find_packet_field(packet, &["flags"], &["node"], &["flags"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let node_depth = find_packet_field(packet, &["depth"], &["node"], &["depth"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let scene_link_node_slot =
        find_packet_field(packet, &["node_slot"], &["scene_link"], &["node_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(node_id);
    let scene_link_transform_slot = find_packet_field(
        packet,
        &["transform_slot"],
        &["scene_link"],
        &["transform_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(transform_translate);
    let scene_link_mesh_slot =
        find_packet_field(packet, &["mesh_slot"], &["scene_link"], &["mesh_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(mesh_vertex_count);
    let scene_link_material_slot = find_packet_field(
        packet,
        &["material_slot"],
        &["scene_link"],
        &["material_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(material_albedo);
    let scene_link_light_slot =
        find_packet_field(packet, &["light_slot"], &["scene_link"], &["light_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(light_kind);
    let scene_link_layer_slot =
        find_packet_field(packet, &["layer_slot"], &["scene_link"], &["layer_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(layer_order);
    let instance_node_slot =
        find_packet_field(packet, &["node_slot"], &["instance"], &["node_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(scene_link_node_slot);
    let instance_count = find_packet_field(packet, &["count"], &["instance"], &["count"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(3);
    let instance_stride = find_packet_field(packet, &["stride"], &["instance"], &["stride"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let instance_phase = find_packet_field(packet, &["phase"], &["instance"], &["phase"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let instance_material_slot = find_packet_field(
        packet,
        &["material_slot"],
        &["instance"],
        &["material_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(scene_link_material_slot);
    let instance_light_slot =
        find_packet_field(packet, &["light_slot"], &["instance"], &["light_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(scene_link_light_slot);
    let scene_graph_root_slot =
        find_packet_field(packet, &["root_slot"], &["scene_graph"], &["root_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(scene_link_node_slot);
    let scene_graph_node_count =
        find_packet_field(packet, &["node_count"], &["scene_graph"], &["node_count"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(6);
    let scene_graph_link_count =
        find_packet_field(packet, &["link_count"], &["scene_graph"], &["link_count"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(3);
    let scene_graph_instance_count = find_packet_field(
        packet,
        &["instance_count"],
        &["scene_graph"],
        &["instance_count"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(instance_count);
    let scene_graph_active_layer = find_packet_field(
        packet,
        &["active_layer"],
        &["scene_graph"],
        &["active_layer"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(scene_link_layer_slot);
    let scene_node_slot =
        find_packet_field(packet, &["node_slot"], &["scene_node"], &["node_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(scene_graph_root_slot);
    let scene_node_first_child_slot = find_packet_field(
        packet,
        &["first_child_slot"],
        &["scene_node"],
        &["first_child_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(scene_link_transform_slot);
    let scene_node_sibling_slot = find_packet_field(
        packet,
        &["sibling_slot"],
        &["scene_node"],
        &["sibling_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(scene_link_mesh_slot);
    let scene_node_instance_slot = find_packet_field(
        packet,
        &["instance_slot"],
        &["scene_node"],
        &["instance_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(3);
    let scene_node_visibility =
        find_packet_field(packet, &["visibility"], &["scene_node"], &["visibility"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(1);
    let instance_group_root_slot = find_packet_field(
        packet,
        &["root_instance_slot"],
        &["instance_group"],
        &["root_instance_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(scene_node_instance_slot);
    let instance_group_count = find_packet_field(
        packet,
        &["group_count"],
        &["instance_group"],
        &["group_count"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(4);
    let instance_group_visible_count = find_packet_field(
        packet,
        &["visible_count"],
        &["instance_group"],
        &["visible_count"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(instance_count);
    let instance_group_phase_bias = find_packet_field(
        packet,
        &["phase_bias"],
        &["instance_group"],
        &["phase_bias"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(instance_phase);
    let instance_group_material_slot = find_packet_field(
        packet,
        &["material_slot"],
        &["instance_group"],
        &["material_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(instance_material_slot);
    let scene_cluster_root_slot = find_packet_field(
        packet,
        &["root_node_slot"],
        &["scene_cluster"],
        &["root_node_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(scene_node_slot);
    let scene_cluster_node_budget = find_packet_field(
        packet,
        &["node_budget"],
        &["scene_cluster"],
        &["node_budget"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(scene_graph_node_count);
    let scene_cluster_instance_group_slot = find_packet_field(
        packet,
        &["instance_group_slot"],
        &["scene_cluster"],
        &["instance_group_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(instance_group_root_slot);
    let scene_cluster_material_slot = find_packet_field(
        packet,
        &["material_slot"],
        &["scene_cluster"],
        &["material_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(instance_group_material_slot);
    let scene_cluster_layer_slot =
        find_packet_field(packet, &["layer_slot"], &["scene_cluster"], &["layer_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(scene_graph_active_layer);
    let visibility_cluster_slot = find_packet_field(
        packet,
        &["cluster_slot"],
        &["scene_visibility"],
        &["cluster_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(scene_cluster_instance_group_slot);
    let visibility_visible_nodes = find_packet_field(
        packet,
        &["visible_nodes"],
        &["scene_visibility"],
        &["visible_nodes"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(instance_group_visible_count);
    let visibility_occlusion_mode = find_packet_field(
        packet,
        &["occlusion_mode"],
        &["scene_visibility"],
        &["occlusion_mode"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(scene_node_visibility);
    let visibility_distance_band = find_packet_field(
        packet,
        &["distance_band"],
        &["scene_visibility"],
        &["distance_band"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(instance_group_phase_bias);
    let visibility_mask = find_packet_field(packet, &["mask"], &["scene_visibility"], &["mask"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(7);
    let cull_cluster_slot = find_packet_field(
        packet,
        &["cluster_slot"],
        &["scene_cull"],
        &["cluster_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(visibility_cluster_slot);
    let cull_kept_nodes =
        find_packet_field(packet, &["kept_nodes"], &["scene_cull"], &["kept_nodes"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(visibility_visible_nodes);
    let cull_mode = find_packet_field(packet, &["cull_mode"], &["scene_cull"], &["cull_mode"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(visibility_occlusion_mode);
    let cull_lod_band = find_packet_field(packet, &["lod_band"], &["scene_cull"], &["lod_band"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(visibility_distance_band);
    let cull_mask = find_packet_field(packet, &["mask"], &["scene_cull"], &["mask"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(visibility_mask);
    let lod_cluster_slot =
        find_packet_field(packet, &["cluster_slot"], &["scene_lod"], &["cluster_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(cull_cluster_slot);
    let lod_level_count =
        find_packet_field(packet, &["level_count"], &["scene_lod"], &["level_count"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(4);
    let lod_active_level =
        find_packet_field(packet, &["active_level"], &["scene_lod"], &["active_level"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(cull_mode);
    let lod_switch_distance = find_packet_field(
        packet,
        &["switch_distance"],
        &["scene_lod"],
        &["switch_distance"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(cull_lod_band);
    let lod_bias = find_packet_field(packet, &["bias"], &["scene_lod"], &["bias"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(cull_mask);
    let streaming_cluster_slot = find_packet_field(
        packet,
        &["cluster_slot"],
        &["scene_streaming"],
        &["cluster_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(lod_cluster_slot);
    let streaming_resident_levels = find_packet_field(
        packet,
        &["resident_levels"],
        &["scene_streaming"],
        &["resident_levels"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(2);
    let streaming_prefetch_mode = find_packet_field(
        packet,
        &["prefetch_mode"],
        &["scene_streaming"],
        &["prefetch_mode"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(lod_active_level);
    let streaming_evict_budget = find_packet_field(
        packet,
        &["evict_budget"],
        &["scene_streaming"],
        &["evict_budget"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(lod_switch_distance);
    let streaming_channel =
        find_packet_field(packet, &["channel"], &["scene_streaming"], &["channel"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(lod_bias);
    let residency_cluster_slot = find_packet_field(
        packet,
        &["cluster_slot"],
        &["scene_residency"],
        &["cluster_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(streaming_cluster_slot);
    let residency_committed_levels = find_packet_field(
        packet,
        &["committed_levels"],
        &["scene_residency"],
        &["committed_levels"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(streaming_resident_levels);
    let residency_mode = find_packet_field(
        packet,
        &["residency_mode"],
        &["scene_residency"],
        &["residency_mode"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(streaming_prefetch_mode);
    let residency_spill_budget = find_packet_field(
        packet,
        &["spill_budget"],
        &["scene_residency"],
        &["spill_budget"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(streaming_evict_budget);
    let residency_mask = find_packet_field(
        packet,
        &["residency_mask"],
        &["scene_residency"],
        &["residency_mask"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(streaming_channel);
    let eviction_cluster_slot = find_packet_field(
        packet,
        &["cluster_slot"],
        &["scene_eviction"],
        &["cluster_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(residency_cluster_slot);
    let eviction_levels = find_packet_field(
        packet,
        &["evicted_levels"],
        &["scene_eviction"],
        &["evicted_levels"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(residency_mode);
    let eviction_mode = find_packet_field(
        packet,
        &["eviction_mode"],
        &["scene_eviction"],
        &["eviction_mode"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(residency_mode);
    let eviction_reclaim_budget = find_packet_field(
        packet,
        &["reclaim_budget"],
        &["scene_eviction"],
        &["reclaim_budget"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(residency_spill_budget);
    let eviction_mask = find_packet_field(
        packet,
        &["eviction_mask"],
        &["scene_eviction"],
        &["eviction_mask"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(residency_mask);
    let prefetch_cluster_slot = find_packet_field(
        packet,
        &["cluster_slot"],
        &["scene_prefetch"],
        &["cluster_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(eviction_cluster_slot);
    let prefetch_requested_levels = find_packet_field(
        packet,
        &["requested_levels"],
        &["scene_prefetch"],
        &["requested_levels"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(streaming_resident_levels);
    let prefetch_window = find_packet_field(
        packet,
        &["prefetch_window"],
        &["scene_prefetch"],
        &["prefetch_window"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(streaming_prefetch_mode);
    let prefetch_warm_budget = find_packet_field(
        packet,
        &["warm_budget"],
        &["scene_prefetch"],
        &["warm_budget"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(eviction_reclaim_budget);
    let prefetch_mask = find_packet_field(
        packet,
        &["prefetch_mask"],
        &["scene_prefetch"],
        &["prefetch_mask"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(eviction_mask);
    let budget_cluster_slot = find_packet_field(
        packet,
        &["cluster_slot"],
        &["scene_budget"],
        &["cluster_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(prefetch_cluster_slot);
    let budget_total = find_packet_field(
        packet,
        &["total_budget"],
        &["scene_budget"],
        &["total_budget"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(12);
    let budget_used = find_packet_field(
        packet,
        &["used_budget"],
        &["scene_budget"],
        &["used_budget"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(prefetch_warm_budget);
    let budget_headroom =
        find_packet_field(packet, &["headroom"], &["scene_budget"], &["headroom"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(prefetch_requested_levels);
    let budget_policy = find_packet_field(
        packet,
        &["budget_policy"],
        &["scene_budget"],
        &["budget_policy"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(prefetch_window);
    let pressure_cluster_slot = find_packet_field(
        packet,
        &["cluster_slot"],
        &["scene_pressure"],
        &["cluster_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(budget_cluster_slot);
    let pressure_level = find_packet_field(
        packet,
        &["pressure_level"],
        &["scene_pressure"],
        &["pressure_level"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(2);
    let pressure_saturation = find_packet_field(
        packet,
        &["saturation"],
        &["scene_pressure"],
        &["saturation"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(budget_used);
    let pressure_throttled =
        find_packet_field(packet, &["throttled"], &["scene_pressure"], &["throttled"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(budget_policy);
    let pressure_mask = find_packet_field(
        packet,
        &["pressure_mask"],
        &["scene_pressure"],
        &["pressure_mask"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(budget_headroom);
    let thermal_cluster_slot = find_packet_field(
        packet,
        &["cluster_slot"],
        &["scene_thermal"],
        &["cluster_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(pressure_cluster_slot);
    let thermal_level = find_packet_field(
        packet,
        &["thermal_level"],
        &["scene_thermal"],
        &["thermal_level"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(pressure_level);
    let thermal_cooling_mode = find_packet_field(
        packet,
        &["cooling_mode"],
        &["scene_thermal"],
        &["cooling_mode"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(pressure_throttled);
    let thermal_throttled =
        find_packet_field(packet, &["throttled"], &["scene_thermal"], &["throttled"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(pressure_throttled);
    let thermal_mask = find_packet_field(
        packet,
        &["thermal_mask"],
        &["scene_thermal"],
        &["thermal_mask"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(pressure_mask);
    let power_cluster_slot = find_packet_field(
        packet,
        &["cluster_slot"],
        &["scene_power"],
        &["cluster_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(thermal_cluster_slot);
    let power_level =
        find_packet_field(packet, &["power_level"], &["scene_power"], &["power_level"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(thermal_level);
    let power_source_mode =
        find_packet_field(packet, &["source_mode"], &["scene_power"], &["source_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(thermal_cooling_mode);
    let power_capped = find_packet_field(packet, &["capped"], &["scene_power"], &["capped"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(thermal_throttled);
    let power_mask = find_packet_field(packet, &["power_mask"], &["scene_power"], &["power_mask"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(thermal_mask);
    let latency_cluster_slot = find_packet_field(
        packet,
        &["cluster_slot"],
        &["scene_latency"],
        &["cluster_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(power_cluster_slot);
    let latency_frame = find_packet_field(
        packet,
        &["frame_latency"],
        &["scene_latency"],
        &["frame_latency"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(4);
    let latency_input = find_packet_field(
        packet,
        &["input_latency"],
        &["scene_latency"],
        &["input_latency"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(2);
    let latency_jitter = find_packet_field(packet, &["jitter"], &["scene_latency"], &["jitter"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(power_capped);
    let latency_mask = find_packet_field(
        packet,
        &["latency_mask"],
        &["scene_latency"],
        &["latency_mask"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(power_mask);
    let frame_pacing_cluster_slot = find_packet_field(
        packet,
        &["cluster_slot"],
        &["scene_frame_pacing"],
        &["cluster_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(latency_cluster_slot);
    let frame_pacing_cadence =
        find_packet_field(packet, &["cadence"], &["scene_frame_pacing"], &["cadence"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(latency_frame);
    let frame_pacing_variance = find_packet_field(
        packet,
        &["variance"],
        &["scene_frame_pacing"],
        &["variance"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(latency_jitter);
    let frame_pacing_vsync_mode = find_packet_field(
        packet,
        &["vsync_mode"],
        &["scene_frame_pacing"],
        &["vsync_mode"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(latency_jitter);
    let frame_pacing_mask = find_packet_field(
        packet,
        &["pacing_mask"],
        &["scene_frame_pacing"],
        &["pacing_mask"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(latency_mask);
    let frame_variance_cluster_slot = find_packet_field(
        packet,
        &["cluster_slot"],
        &["scene_frame_variance"],
        &["cluster_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(frame_pacing_cluster_slot);
    let frame_variance_frame = find_packet_field(
        packet,
        &["frame_variance"],
        &["scene_frame_variance"],
        &["frame_variance"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(frame_pacing_variance.max(1));
    let frame_variance_input = find_packet_field(
        packet,
        &["input_variance"],
        &["scene_frame_variance"],
        &["input_variance"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(latency_input);
    let frame_variance_burst = find_packet_field(
        packet,
        &["burst_mode"],
        &["scene_frame_variance"],
        &["burst_mode"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(frame_pacing_cadence);
    let frame_variance_mask = find_packet_field(
        packet,
        &["variance_mask"],
        &["scene_frame_variance"],
        &["variance_mask"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(frame_pacing_mask);
    let jank_cluster_slot = find_packet_field(
        packet,
        &["cluster_slot"],
        &["scene_jank"],
        &["cluster_slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(frame_variance_cluster_slot);
    let jank_spikes = find_packet_field(packet, &["spikes"], &["scene_jank"], &["spikes"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(1 + frame_variance_frame.rem_euclid(2));
    let jank_severity = find_packet_field(packet, &["severity"], &["scene_jank"], &["severity"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(frame_variance_frame);
    let jank_recovery = find_packet_field(packet, &["recovery"], &["scene_jank"], &["recovery"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(frame_variance_burst);
    let jank_mask = find_packet_field(packet, &["jank_mask"], &["scene_jank"], &["jank_mask"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(frame_variance_mask);
    let pass_stage = find_packet_field(packet, &["stage"], &["pass"], &["stage"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let pass_clear_mode = find_packet_field(packet, &["clear_mode"], &["pass"], &["clear_mode"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let pass_sample_count =
        find_packet_field(packet, &["sample_count"], &["pass"], &["sample_count"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(4);
    let pass_debug_view = find_packet_field(packet, &["debug_view"], &["pass"], &["debug_view"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent.rem_euclid(6));
    let frame_index = find_packet_field(packet, &["frame_index"], &["frame"], &["frame_index"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let frame_present_mode =
        find_packet_field(packet, &["present_mode"], &["frame"], &["present_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent.rem_euclid(3));
    let frame_sync_interval =
        find_packet_field(packet, &["sync_interval"], &["frame"], &["sync_interval"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(1);
    let frame_exposure = find_packet_field(packet, &["exposure"], &["frame"], &["exposure"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 24.0).round() as i64);
    let target_kind = find_packet_field(packet, &["kind"], &["target"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent.rem_euclid(3));
    let target_width = find_packet_field(packet, &["width"], &["target"], &["width"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(48);
    let target_height = find_packet_field(packet, &["height"], &["target"], &["height"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(18);
    let target_multisample =
        find_packet_field(packet, &["multisample"], &["target"], &["multisample"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let frame_graph_passes = find_packet_field(packet, &["passes"], &["frame_graph"], &["passes"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let frame_graph_targets =
        find_packet_field(packet, &["targets"], &["frame_graph"], &["targets"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(1);
    let frame_graph_present_stage = find_packet_field(
        packet,
        &["present_stage"],
        &["frame_graph"],
        &["present_stage"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(contrast.rem_euclid(3));
    let frame_graph_debug_overlay = find_packet_field(
        packet,
        &["debug_overlay"],
        &["frame_graph"],
        &["debug_overlay"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(accent.rem_euclid(6));
    let attachment_slot = find_packet_field(packet, &["slot"], &["attachment"], &["slot"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let attachment_format_kind =
        find_packet_field(packet, &["format_kind"], &["attachment"], &["format_kind"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let attachment_load_op = find_packet_field(packet, &["load_op"], &["attachment"], &["load_op"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let attachment_store_op =
        find_packet_field(packet, &["store_op"], &["attachment"], &["store_op"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(1);
    let pass_chain_stages = find_packet_field(packet, &["stages"], &["pass_chain"], &["stages"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let pass_chain_fanout = find_packet_field(packet, &["fanout"], &["pass_chain"], &["fanout"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(1);
    let pass_chain_resolve_stage = find_packet_field(
        packet,
        &["resolve_stage"],
        &["pass_chain"],
        &["resolve_stage"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(contrast.rem_euclid(3));
    let pass_chain_barrier_mode = find_packet_field(
        packet,
        &["barrier_mode"],
        &["pass_chain"],
        &["barrier_mode"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(accent);
    let barrier_scope = find_packet_field(packet, &["scope"], &["barrier"], &["scope"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(1);
    let barrier_source_stage =
        find_packet_field(packet, &["source_stage"], &["barrier"], &["source_stage"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast.rem_euclid(3));
    let barrier_target_stage =
        find_packet_field(packet, &["target_stage"], &["barrier"], &["target_stage"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(2);
    let barrier_flush_mode =
        find_packet_field(packet, &["flush_mode"], &["barrier"], &["flush_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let resource_buffers = find_packet_field(packet, &["buffers"], &["resource_set"], &["buffers"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let resource_textures =
        find_packet_field(packet, &["textures"], &["resource_set"], &["textures"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(1);
    let resource_samplers =
        find_packet_field(packet, &["samplers"], &["resource_set"], &["samplers"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(1);
    let resource_residency =
        find_packet_field(packet, &["residency"], &["resource_set"], &["residency"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let schedule_lanes = find_packet_field(packet, &["lanes"], &["schedule"], &["lanes"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let schedule_queue_depth =
        find_packet_field(packet, &["queue_depth"], &["schedule"], &["queue_depth"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(4);
    let schedule_async_budget =
        find_packet_field(packet, &["async_budget"], &["schedule"], &["async_budget"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or((radius_scale * 24.0).round() as i64);
    let schedule_tick_mode =
        find_packet_field(packet, &["tick_mode"], &["schedule"], &["tick_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast.rem_euclid(3));
    let submission_batches = find_packet_field(packet, &["batches"], &["submission"], &["batches"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let submission_fences = find_packet_field(packet, &["fences"], &["submission"], &["fences"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(1);
    let submission_signal_mode =
        find_packet_field(packet, &["signal_mode"], &["submission"], &["signal_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast.rem_euclid(3));
    let submission_present_hint = find_packet_field(
        packet,
        &["present_hint"],
        &["submission"],
        &["present_hint"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(accent);
    let queue_kind = find_packet_field(packet, &["kind"], &["queue"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let queue_priority = find_packet_field(packet, &["priority"], &["queue"], &["priority"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let queue_budget = find_packet_field(packet, &["budget"], &["queue"], &["budget"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 24.0).round() as i64);
    let queue_ownership = find_packet_field(packet, &["ownership"], &["queue"], &["ownership"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let semaphore_wait_count =
        find_packet_field(packet, &["wait_count"], &["semaphore"], &["wait_count"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(1);
    let semaphore_signal_count =
        find_packet_field(packet, &["signal_count"], &["semaphore"], &["signal_count"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(2);
    let semaphore_timeline_mode = find_packet_field(
        packet,
        &["timeline_mode"],
        &["semaphore"],
        &["timeline_mode"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(contrast.rem_euclid(3));
    let semaphore_scope = find_packet_field(packet, &["scope"], &["semaphore"], &["scope"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let timeline_value = find_packet_field(packet, &["value"], &["timeline"], &["value"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or((radius_scale * 24.0).round() as i64);
    let timeline_step = find_packet_field(packet, &["step"], &["timeline"], &["step"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(1);
    let timeline_epoch = find_packet_field(packet, &["epoch"], &["timeline"], &["epoch"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let timeline_domain = find_packet_field(packet, &["domain"], &["timeline"], &["domain"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let fence_signaled = find_packet_field(packet, &["signaled"], &["fence"], &["signaled"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(1);
    let fence_epoch = find_packet_field(packet, &["epoch"], &["fence"], &["epoch"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let fence_scope = find_packet_field(packet, &["scope"], &["fence"], &["scope"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let fence_recycle_mode =
        find_packet_field(packet, &["recycle_mode"], &["fence"], &["recycle_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(1);
    let signal_kind = find_packet_field(packet, &["kind"], &["signal"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let signal_phase = find_packet_field(packet, &["phase"], &["signal"], &["phase"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let signal_fanout = find_packet_field(packet, &["fanout"], &["signal"], &["fanout"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(3);
    let signal_ack_mode = find_packet_field(packet, &["ack_mode"], &["signal"], &["ack_mode"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let event_kind = find_packet_field(packet, &["kind"], &["event"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let event_route = find_packet_field(packet, &["route"], &["event"], &["route"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let event_priority = find_packet_field(packet, &["priority"], &["event"], &["priority"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(3);
    let event_payload_mode =
        find_packet_field(packet, &["payload_mode"], &["event"], &["payload_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let dispatch_queue_kind =
        find_packet_field(packet, &["queue_kind"], &["dispatch"], &["queue_kind"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast.rem_euclid(3));
    let dispatch_lane = find_packet_field(packet, &["lane"], &["dispatch"], &["lane"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let dispatch_batch = find_packet_field(packet, &["batch"], &["dispatch"], &["batch"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(3);
    let dispatch_completion_mode = find_packet_field(
        packet,
        &["completion_mode"],
        &["dispatch"],
        &["completion_mode"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(accent);
    let feedback_status = find_packet_field(packet, &["status"], &["feedback"], &["status"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0).rem_euclid(2));
    let feedback_latency = find_packet_field(packet, &["latency"], &["feedback"], &["latency"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let feedback_retries = find_packet_field(packet, &["retries"], &["feedback"], &["retries"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(radius_scale.round() as i64 % 4);
    let feedback_channel = find_packet_field(packet, &["channel"], &["feedback"], &["channel"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let intent_kind = find_packet_field(packet, &["kind"], &["intent"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let intent_target = find_packet_field(packet, &["target_slot"], &["intent"], &["target_slot"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast);
    let intent_urgency = find_packet_field(packet, &["urgency"], &["intent"], &["urgency"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let intent_policy = find_packet_field(packet, &["policy"], &["intent"], &["policy"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let reaction_kind = find_packet_field(packet, &["kind"], &["reaction"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let reaction_result_slot =
        find_packet_field(packet, &["result_slot"], &["reaction"], &["result_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast);
    let reaction_stability =
        find_packet_field(packet, &["stability"], &["reaction"], &["stability"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(radius_scale.round() as i64 % 4);
    let reaction_echo_mode =
        find_packet_field(packet, &["echo_mode"], &["reaction"], &["echo_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let outcome_kind = find_packet_field(packet, &["kind"], &["outcome"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let outcome_final_slot =
        find_packet_field(packet, &["final_slot"], &["outcome"], &["final_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast);
    let outcome_confidence =
        find_packet_field(packet, &["confidence"], &["outcome"], &["confidence"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let outcome_settle_mode =
        find_packet_field(packet, &["settle_mode"], &["outcome"], &["settle_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let resolution_kind = find_packet_field(packet, &["kind"], &["resolution"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let resolution_commit_slot =
        find_packet_field(packet, &["commit_slot"], &["resolution"], &["commit_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast);
    let resolution_convergence =
        find_packet_field(packet, &["convergence"], &["resolution"], &["convergence"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(radius_scale.round() as i64 % 4);
    let resolution_policy_mode =
        find_packet_field(packet, &["policy_mode"], &["resolution"], &["policy_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let commit_kind = find_packet_field(packet, &["kind"], &["commit"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let commit_applied_slot =
        find_packet_field(packet, &["applied_slot"], &["commit"], &["applied_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast);
    let commit_durability =
        find_packet_field(packet, &["durability"], &["commit"], &["durability"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let commit_commit_mode =
        find_packet_field(packet, &["commit_mode"], &["commit"], &["commit_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let snapshot_kind = find_packet_field(packet, &["kind"], &["snapshot"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let snapshot_source_slot =
        find_packet_field(packet, &["source_slot"], &["snapshot"], &["source_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast);
    let snapshot_retention =
        find_packet_field(packet, &["retention"], &["snapshot"], &["retention"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(radius_scale.round() as i64 % 4);
    let snapshot_replay_mode =
        find_packet_field(packet, &["replay_mode"], &["snapshot"], &["replay_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let checkpoint_kind = find_packet_field(packet, &["kind"], &["checkpoint"], &["kind"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(contrast.rem_euclid(3));
    let checkpoint_anchor_slot =
        find_packet_field(packet, &["anchor_slot"], &["checkpoint"], &["anchor_slot"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(contrast);
    let checkpoint_rollback_depth = find_packet_field(
        packet,
        &["rollback_depth"],
        &["checkpoint"],
        &["rollback_depth"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let checkpoint_resume_mode =
        find_packet_field(packet, &["resume_mode"], &["checkpoint"], &["resume_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(accent);
    let color_min = find_slider_packet_field(packet, "color", "min")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let color_max = find_slider_packet_field(packet, "color", "max")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(127);
    let color_step = find_slider_packet_field(packet, "color", "step")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(4);
    let color_disabled = find_slider_packet_field(packet, "color", "disabled")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let speed_min = find_slider_packet_field(packet, "speed", "min")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let speed_max = find_slider_packet_field(packet, "speed", "max")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(63);
    let speed_step = find_slider_packet_field(packet, "speed", "step")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(2);
    let speed_disabled = find_slider_packet_field(packet, "speed", "disabled")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let radius_min = find_slider_packet_field(packet, "radius", "min")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let radius_max = find_slider_packet_field(packet, "radius", "max")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(127);
    let radius_step = find_slider_packet_field(packet, "radius", "step")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(3);
    let radius_disabled = find_slider_packet_field(packet, "radius", "disabled")
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let toggle_state = find_packet_field(
        packet,
        &["toggle_state", "toggle_live"],
        &["toggle"],
        &["live"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(1);
    let toggle_disabled =
        find_packet_field(packet, &["toggle_disabled"], &["toggle"], &["disabled"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(0);
    let focus_index = find_packet_field(
        packet,
        &["focus_index", "focus_slot"],
        &["focus"],
        &["slot"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(0);
    let progress_value = find_packet_field(packet, &["progress_value"], &["progress"], &["value"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| scalar_to_color_key(speed, op).unwrap_or(0));
    let progress_max = find_packet_field(packet, &["progress_max"], &["progress"], &["max"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(63);
    let meter_value = find_packet_field(packet, &["meter_value"], &["meter"], &["value"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or_else(|| (radius_scale * 96.0).round() as i64);
    let meter_max = find_packet_field(packet, &["meter_max"], &["meter"], &["max"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(127);
    let button_state = find_packet_field(packet, &["button_state"], &["button"], &["active"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(toggle_state);
    let button_intent = find_packet_field(packet, &["button_intent"], &["button"], &["intent"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(focus_index);
    let header_title_mode =
        find_packet_field(packet, &["header_title_mode"], &["header"], &["title_mode"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(focus_index.rem_euclid(2));
    let text_caret = find_packet_field(packet, &["text_caret"], &["text_input"], &["caret"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(focus_index);
    let text_echo = find_packet_field(packet, &["text_echo"], &["text_input"], &["echo"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(accent);
    let text_placeholder = find_packet_field(
        packet,
        &["text_placeholder"],
        &["text_input"],
        &["placeholder"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(radius_scale.round() as i64);
    let text_read_only =
        find_packet_field(packet, &["text_read_only"], &["text_input"], &["read_only"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(0);
    let text_dirty = find_packet_field(packet, &["text_dirty"], &["text_input"], &["dirty"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let select_index = find_packet_field(packet, &["select_index"], &["select"], &["selected"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(focus_index);
    let select_options = find_packet_field(packet, &["select_options"], &["select"], &["options"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(3);
    let select_multiple =
        find_packet_field(packet, &["select_multiple"], &["select"], &["multiple"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(0);
    let select_committed =
        find_packet_field(packet, &["select_committed"], &["select"], &["committed"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(1);
    let checkbox_checked =
        find_packet_field(packet, &["checkbox_checked"], &["checkbox"], &["checked"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(toggle_state);
    let checkbox_disabled =
        find_packet_field(packet, &["checkbox_disabled"], &["checkbox"], &["disabled"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(0);
    let radio_selected = find_packet_field(packet, &["radio_selected"], &["radio"], &["selected"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(focus_index);
    let radio_options = find_packet_field(packet, &["radio_options"], &["radio"], &["options"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(4);
    let radio_disabled = find_packet_field(packet, &["radio_disabled"], &["radio"], &["disabled"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let textarea_lines = find_packet_field(packet, &["textarea_lines"], &["textarea"], &["lines"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(3);
    let textarea_scroll =
        find_packet_field(packet, &["textarea_scroll"], &["textarea"], &["scroll"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(text_caret);
    let textarea_placeholder = find_packet_field(
        packet,
        &["textarea_placeholder"],
        &["textarea"],
        &["placeholder"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(text_placeholder);
    let textarea_read_only = find_packet_field(
        packet,
        &["textarea_read_only"],
        &["textarea"],
        &["read_only"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(text_read_only);
    let textarea_dirty = find_packet_field(packet, &["textarea_dirty"], &["textarea"], &["dirty"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(text_dirty);
    let tabs_active = find_packet_field(packet, &["tabs_active"], &["tabs"], &["active"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(focus_index);
    let tabs_count = find_packet_field(packet, &["tabs_count"], &["tabs"], &["count"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(4);
    let tabs_compact = find_packet_field(packet, &["tabs_compact"], &["tabs"], &["compact"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let list_selected = find_packet_field(packet, &["list_selected"], &["list"], &["selected"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(focus_index);
    let list_items = find_packet_field(packet, &["list_items"], &["list"], &["items"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(5);
    let list_dense = find_packet_field(packet, &["list_dense"], &["list"], &["dense"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(0);
    let table_rows = find_packet_field(packet, &["table_rows"], &["table"], &["rows"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(4);
    let table_cols = find_packet_field(packet, &["table_cols"], &["table"], &["cols"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(3);
    let table_selected_row = find_packet_field(
        packet,
        &["table_selected_row"],
        &["table"],
        &["selected_row"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(focus_index);
    let table_zebra = find_packet_field(packet, &["table_zebra"], &["table"], &["zebra"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(1);
    let tree_selected = find_packet_field(packet, &["tree_selected"], &["tree"], &["selected"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(focus_index);
    let tree_nodes = find_packet_field(packet, &["tree_nodes"], &["tree"], &["nodes"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(6);
    let tree_expanded = find_packet_field(packet, &["tree_expanded"], &["tree"], &["expanded"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(toggle_state);
    let inspector_selected = find_packet_field(
        packet,
        &["inspector_selected"],
        &["inspector"],
        &["selected"],
    )
    .map(|value| scalar_to_color_key(value, op))
    .transpose()?
    .unwrap_or(focus_index);
    let inspector_fields =
        find_packet_field(packet, &["inspector_fields"], &["inspector"], &["fields"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(4);
    let inspector_pinned =
        find_packet_field(packet, &["inspector_pinned"], &["inspector"], &["pinned"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(toggle_state);
    let outline_selected =
        find_packet_field(packet, &["outline_selected"], &["outline"], &["selected"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(focus_index);
    let outline_items = find_packet_field(packet, &["outline_items"], &["outline"], &["items"])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()?
        .unwrap_or(6);
    let outline_collapsed =
        find_packet_field(packet, &["outline_collapsed"], &["outline"], &["collapsed"])
            .map(|value| scalar_to_color_key(value, op))
            .transpose()?
            .unwrap_or(toggle_state);

    Ok(BallPacket {
        color_key: scalar_to_color_key(color, op)?,
        speed: scalar_to_f32(speed, op)?,
        radius_scale,
        color_min,
        color_max,
        color_step,
        color_disabled,
        speed_min,
        speed_max,
        speed_step,
        speed_disabled,
        radius_min,
        radius_max,
        radius_step,
        radius_disabled,
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
        visibility_cluster_slot,
        visibility_visible_nodes,
        visibility_occlusion_mode,
        visibility_distance_band,
        visibility_mask,
        cull_cluster_slot,
        cull_kept_nodes,
        cull_mode,
        cull_lod_band,
        cull_mask,
        lod_cluster_slot,
        lod_level_count,
        lod_active_level,
        lod_switch_distance,
        lod_bias,
        streaming_cluster_slot,
        streaming_resident_levels,
        streaming_prefetch_mode,
        streaming_evict_budget,
        streaming_channel,
        residency_cluster_slot,
        residency_committed_levels,
        residency_mode,
        residency_spill_budget,
        residency_mask,
        eviction_cluster_slot,
        eviction_levels,
        eviction_mode,
        eviction_reclaim_budget,
        eviction_mask,
        prefetch_cluster_slot,
        prefetch_requested_levels,
        prefetch_window,
        prefetch_warm_budget,
        prefetch_mask,
        budget_cluster_slot,
        budget_total,
        budget_used,
        budget_headroom,
        budget_policy,
        pressure_cluster_slot,
        pressure_level,
        pressure_saturation,
        pressure_throttled,
        pressure_mask,
        thermal_cluster_slot,
        thermal_level,
        thermal_cooling_mode,
        thermal_throttled,
        thermal_mask,
        power_cluster_slot,
        power_level,
        power_source_mode,
        power_capped,
        power_mask,
        latency_cluster_slot,
        latency_frame,
        latency_input,
        latency_jitter,
        latency_mask,
        frame_pacing_cluster_slot,
        frame_pacing_cadence,
        frame_pacing_variance,
        frame_pacing_vsync_mode,
        frame_pacing_mask,
        frame_variance_cluster_slot,
        frame_variance_frame,
        frame_variance_input,
        frame_variance_burst,
        frame_variance_mask,
        jank_cluster_slot,
        jank_spikes,
        jank_severity,
        jank_recovery,
        jank_mask,
        pass_stage,
        pass_clear_mode,
        pass_sample_count,
        pass_debug_view,
        frame_index,
        frame_present_mode,
        frame_sync_interval,
        frame_exposure,
        target_kind,
        target_width,
        target_height,
        target_multisample,
        frame_graph_passes,
        frame_graph_targets,
        frame_graph_present_stage,
        frame_graph_debug_overlay,
        attachment_slot,
        attachment_format_kind,
        attachment_load_op,
        attachment_store_op,
        pass_chain_stages,
        pass_chain_fanout,
        pass_chain_resolve_stage,
        pass_chain_barrier_mode,
        barrier_scope,
        barrier_source_stage,
        barrier_target_stage,
        barrier_flush_mode,
        resource_buffers,
        resource_textures,
        resource_samplers,
        resource_residency,
        schedule_lanes,
        schedule_queue_depth,
        schedule_async_budget,
        schedule_tick_mode,
        submission_batches,
        submission_fences,
        submission_signal_mode,
        submission_present_hint,
        queue_kind,
        queue_priority,
        queue_budget,
        queue_ownership,
        semaphore_wait_count,
        semaphore_signal_count,
        semaphore_timeline_mode,
        semaphore_scope,
        timeline_value,
        timeline_step,
        timeline_epoch,
        timeline_domain,
        fence_signaled,
        fence_epoch,
        fence_scope,
        fence_recycle_mode,
        signal_kind,
        signal_phase,
        signal_fanout,
        signal_ack_mode,
        event_kind,
        event_route,
        event_priority,
        event_payload_mode,
        dispatch_queue_kind,
        dispatch_lane,
        dispatch_batch,
        dispatch_completion_mode,
        feedback_status,
        feedback_latency,
        feedback_retries,
        feedback_channel,
        intent_kind,
        intent_target,
        intent_urgency,
        intent_policy,
        reaction_kind,
        reaction_result_slot,
        reaction_stability,
        reaction_echo_mode,
        outcome_kind,
        outcome_final_slot,
        outcome_confidence,
        outcome_settle_mode,
        resolution_kind,
        resolution_commit_slot,
        resolution_convergence,
        resolution_policy_mode,
        commit_kind,
        commit_applied_slot,
        commit_durability,
        commit_commit_mode,
        snapshot_kind,
        snapshot_source_slot,
        snapshot_retention,
        snapshot_replay_mode,
        checkpoint_kind,
        checkpoint_anchor_slot,
        checkpoint_rollback_depth,
        checkpoint_resume_mode,
        toggle_state,
        focus_index,
        progress_value,
        progress_max,
        meter_value,
        meter_max,
        button_state,
        button_intent,
        header_title_mode,
        toggle_disabled,
        text_caret,
        text_echo,
        text_placeholder,
        text_read_only,
        text_dirty,
        select_index,
        select_options,
        select_multiple,
        select_committed,
        checkbox_checked,
        checkbox_disabled,
        radio_selected,
        radio_options,
        radio_disabled,
        textarea_lines,
        textarea_scroll,
        textarea_placeholder,
        textarea_read_only,
        textarea_dirty,
        tabs_active,
        tabs_count,
        tabs_compact,
        list_selected,
        list_items,
        list_dense,
        table_rows,
        table_cols,
        table_selected_row,
        table_zebra,
        tree_selected,
        tree_nodes,
        tree_expanded,
        inspector_selected,
        inspector_fields,
        inspector_pinned,
        outline_selected,
        outline_items,
        outline_collapsed,
    })
}
