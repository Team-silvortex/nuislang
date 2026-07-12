use super::packet_helpers::{find_packet_field, find_slider_packet_field, scalar_to_color_key};
use yir_core::{StructValue, Value};

pub(crate) struct BallPacketControlFields {
    pub(crate) color_min: i64,
    pub(crate) color_max: i64,
    pub(crate) color_step: i64,
    pub(crate) color_disabled: i64,
    pub(crate) speed_min: i64,
    pub(crate) speed_max: i64,
    pub(crate) speed_step: i64,
    pub(crate) speed_disabled: i64,
    pub(crate) radius_min: i64,
    pub(crate) radius_max: i64,
    pub(crate) radius_step: i64,
    pub(crate) radius_disabled: i64,
    pub(crate) toggle_state: i64,
    pub(crate) focus_index: i64,
    pub(crate) progress_value: i64,
    pub(crate) progress_max: i64,
    pub(crate) meter_value: i64,
    pub(crate) meter_max: i64,
    pub(crate) button_state: i64,
    pub(crate) button_intent: i64,
    pub(crate) header_title_mode: i64,
    pub(crate) toggle_disabled: i64,
    pub(crate) text_caret: i64,
    pub(crate) text_echo: i64,
    pub(crate) text_placeholder: i64,
    pub(crate) text_read_only: i64,
    pub(crate) text_dirty: i64,
    pub(crate) select_index: i64,
    pub(crate) select_options: i64,
    pub(crate) select_multiple: i64,
    pub(crate) select_committed: i64,
    pub(crate) checkbox_checked: i64,
    pub(crate) checkbox_disabled: i64,
    pub(crate) radio_selected: i64,
    pub(crate) radio_options: i64,
    pub(crate) radio_disabled: i64,
    pub(crate) textarea_lines: i64,
    pub(crate) textarea_scroll: i64,
    pub(crate) textarea_placeholder: i64,
    pub(crate) textarea_read_only: i64,
    pub(crate) textarea_dirty: i64,
    pub(crate) tabs_active: i64,
    pub(crate) tabs_count: i64,
    pub(crate) tabs_compact: i64,
    pub(crate) list_selected: i64,
    pub(crate) list_items: i64,
    pub(crate) list_dense: i64,
    pub(crate) table_rows: i64,
    pub(crate) table_cols: i64,
    pub(crate) table_selected_row: i64,
    pub(crate) table_zebra: i64,
    pub(crate) tree_selected: i64,
    pub(crate) tree_nodes: i64,
    pub(crate) tree_expanded: i64,
    pub(crate) inspector_selected: i64,
    pub(crate) inspector_fields: i64,
    pub(crate) inspector_pinned: i64,
    pub(crate) outline_selected: i64,
    pub(crate) outline_items: i64,
    pub(crate) outline_collapsed: i64,
}

pub(crate) fn parse_ball_packet_controls(
    packet: &StructValue,
    op: &str,
    radius_scale: f32,
    accent: i64,
    speed: &Value,
) -> Result<BallPacketControlFields, String> {
    let color_min = slider_i64(packet, op, "color", "min", 0)?;
    let color_max = slider_i64(packet, op, "color", "max", 127)?;
    let color_step = slider_i64(packet, op, "color", "step", 4)?;
    let color_disabled = slider_i64(packet, op, "color", "disabled", 0)?;
    let speed_min = slider_i64(packet, op, "speed", "min", 0)?;
    let speed_max = slider_i64(packet, op, "speed", "max", 63)?;
    let speed_step = slider_i64(packet, op, "speed", "step", 2)?;
    let speed_disabled = slider_i64(packet, op, "speed", "disabled", 0)?;
    let radius_min = slider_i64(packet, op, "radius", "min", 0)?;
    let radius_max = slider_i64(packet, op, "radius", "max", 127)?;
    let radius_step = slider_i64(packet, op, "radius", "step", 3)?;
    let radius_disabled = slider_i64(packet, op, "radius", "disabled", 0)?;
    let toggle_state = packet_i64(
        packet,
        op,
        &["toggle_state", "toggle_live"],
        "toggle",
        "live",
        1,
    )?;
    let toggle_disabled = packet_i64(packet, op, &["toggle_disabled"], "toggle", "disabled", 0)?;
    let focus_index = packet_i64(
        packet,
        op,
        &["focus_index", "focus_slot"],
        "focus",
        "slot",
        0,
    )?;
    let progress_value =
        packet_i64_with(packet, op, &["progress_value"], "progress", "value", || {
            scalar_to_color_key(speed, op).unwrap_or(0)
        })?;
    let progress_max = packet_i64(packet, op, &["progress_max"], "progress", "max", 63)?;
    let meter_value = packet_i64_with(packet, op, &["meter_value"], "meter", "value", || {
        (radius_scale * 96.0).round() as i64
    })?;
    let meter_max = packet_i64(packet, op, &["meter_max"], "meter", "max", 127)?;
    let button_state = packet_i64(
        packet,
        op,
        &["button_state"],
        "button",
        "active",
        toggle_state,
    )?;
    let button_intent = packet_i64(
        packet,
        op,
        &["button_intent"],
        "button",
        "intent",
        focus_index,
    )?;
    let header_title_mode = packet_i64(
        packet,
        op,
        &["header_title_mode"],
        "header",
        "title_mode",
        focus_index.rem_euclid(2),
    )?;
    let text_caret = packet_i64(
        packet,
        op,
        &["text_caret"],
        "text_input",
        "caret",
        focus_index,
    )?;
    let text_echo = packet_i64(packet, op, &["text_echo"], "text_input", "echo", accent)?;
    let text_placeholder = packet_i64(
        packet,
        op,
        &["text_placeholder"],
        "text_input",
        "placeholder",
        radius_scale.round() as i64,
    )?;
    let text_read_only = packet_i64(
        packet,
        op,
        &["text_read_only"],
        "text_input",
        "read_only",
        0,
    )?;
    let text_dirty = packet_i64(packet, op, &["text_dirty"], "text_input", "dirty", 0)?;
    let select_index = packet_i64(
        packet,
        op,
        &["select_index"],
        "select",
        "selected",
        focus_index,
    )?;
    let select_options = packet_i64(packet, op, &["select_options"], "select", "options", 3)?;
    let select_multiple = packet_i64(packet, op, &["select_multiple"], "select", "multiple", 0)?;
    let select_committed = packet_i64(packet, op, &["select_committed"], "select", "committed", 1)?;
    let checkbox_checked = packet_i64(
        packet,
        op,
        &["checkbox_checked"],
        "checkbox",
        "checked",
        toggle_state,
    )?;
    let checkbox_disabled = packet_i64(
        packet,
        op,
        &["checkbox_disabled"],
        "checkbox",
        "disabled",
        0,
    )?;
    let radio_selected = packet_i64(
        packet,
        op,
        &["radio_selected"],
        "radio",
        "selected",
        focus_index,
    )?;
    let radio_options = packet_i64(packet, op, &["radio_options"], "radio", "options", 4)?;
    let radio_disabled = packet_i64(packet, op, &["radio_disabled"], "radio", "disabled", 0)?;
    let textarea_lines = packet_i64(packet, op, &["textarea_lines"], "textarea", "lines", 3)?;
    let textarea_scroll = packet_i64(
        packet,
        op,
        &["textarea_scroll"],
        "textarea",
        "scroll",
        text_caret,
    )?;
    let textarea_placeholder = packet_i64(
        packet,
        op,
        &["textarea_placeholder"],
        "textarea",
        "placeholder",
        text_placeholder,
    )?;
    let textarea_read_only = packet_i64(
        packet,
        op,
        &["textarea_read_only"],
        "textarea",
        "read_only",
        text_read_only,
    )?;
    let textarea_dirty = packet_i64(
        packet,
        op,
        &["textarea_dirty"],
        "textarea",
        "dirty",
        text_dirty,
    )?;
    let tabs_active = packet_i64(packet, op, &["tabs_active"], "tabs", "active", focus_index)?;
    let tabs_count = packet_i64(packet, op, &["tabs_count"], "tabs", "count", 4)?;
    let tabs_compact = packet_i64(packet, op, &["tabs_compact"], "tabs", "compact", 0)?;
    let list_selected = packet_i64(
        packet,
        op,
        &["list_selected"],
        "list",
        "selected",
        focus_index,
    )?;
    let list_items = packet_i64(packet, op, &["list_items"], "list", "items", 5)?;
    let list_dense = packet_i64(packet, op, &["list_dense"], "list", "dense", 0)?;
    let table_rows = packet_i64(packet, op, &["table_rows"], "table", "rows", 4)?;
    let table_cols = packet_i64(packet, op, &["table_cols"], "table", "cols", 3)?;
    let table_selected_row = packet_i64(
        packet,
        op,
        &["table_selected_row"],
        "table",
        "selected_row",
        focus_index,
    )?;
    let table_zebra = packet_i64(packet, op, &["table_zebra"], "table", "zebra", 1)?;
    let tree_selected = packet_i64(
        packet,
        op,
        &["tree_selected"],
        "tree",
        "selected",
        focus_index,
    )?;
    let tree_nodes = packet_i64(packet, op, &["tree_nodes"], "tree", "nodes", 6)?;
    let tree_expanded = packet_i64(
        packet,
        op,
        &["tree_expanded"],
        "tree",
        "expanded",
        toggle_state,
    )?;
    let inspector_selected = packet_i64(
        packet,
        op,
        &["inspector_selected"],
        "inspector",
        "selected",
        focus_index,
    )?;
    let inspector_fields = packet_i64(packet, op, &["inspector_fields"], "inspector", "fields", 4)?;
    let inspector_pinned = packet_i64(
        packet,
        op,
        &["inspector_pinned"],
        "inspector",
        "pinned",
        toggle_state,
    )?;
    let outline_selected = packet_i64(
        packet,
        op,
        &["outline_selected"],
        "outline",
        "selected",
        focus_index,
    )?;
    let outline_items = packet_i64(packet, op, &["outline_items"], "outline", "items", 6)?;
    let outline_collapsed = packet_i64(
        packet,
        op,
        &["outline_collapsed"],
        "outline",
        "collapsed",
        toggle_state,
    )?;

    Ok(BallPacketControlFields {
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

fn slider_i64(
    packet: &StructValue,
    op: &str,
    slider_name: &str,
    field_name: &str,
    default: i64,
) -> Result<i64, String> {
    find_slider_packet_field(packet, slider_name, field_name)
        .map(|value| scalar_to_color_key(value, op))
        .transpose()
        .map(|value| value.unwrap_or(default))
}

fn packet_i64(
    packet: &StructValue,
    op: &str,
    flat_names: &[&str],
    nested_name: &str,
    nested_field: &str,
    default: i64,
) -> Result<i64, String> {
    packet_i64_with(packet, op, flat_names, nested_name, nested_field, || {
        default
    })
}

fn packet_i64_with(
    packet: &StructValue,
    op: &str,
    flat_names: &[&str],
    nested_name: &str,
    nested_field: &str,
    default: impl FnOnce() -> i64,
) -> Result<i64, String> {
    find_packet_field(packet, flat_names, &[nested_name], &[nested_field])
        .map(|value| scalar_to_color_key(value, op))
        .transpose()
        .map(|value| value.unwrap_or_else(default))
}
