use super::surface_primitives::{draw_box, draw_knob, draw_slider, fill_rect, put_text, BoxGlyphs};
use super::BallPacket;

pub(crate) fn draw_control_panel_widgets(
    mut rows: &mut [Vec<char>],
    packet: &BallPacket,
    panel_left: usize,
    panel_top: usize,
    panel_right: usize,
    panel_bottom: usize,
    accent: char,
    color_value: usize,
    speed_value: usize,
    radius_value: usize,
    progress_value: usize,
    meter_value: usize,
    button_on: bool,
    toggle_disabled: bool,
) {
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
}
