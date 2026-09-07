use super::{MainlineSelection, MAINLINE_PROTOCOL, MAINLINE_SOURCE};
use crate::{json_field, json_string_array_field};

pub(crate) fn mainline_json_fields(selection: &MainlineSelection) -> Vec<String> {
    vec![
        json_field("mainline_protocol", MAINLINE_PROTOCOL),
        json_field("mainline_plan_source", MAINLINE_SOURCE),
        json_field("mainline_status", selection.status),
        json_field("mainline_id", &selection.id),
        json_field("mainline_target", &selection.target),
        json_field("mainline_selected", &selection.selected),
        json_field("mainline_reason", &selection.reason),
        json_string_array_field("mainline_dependency_path", &selection.dependency_path),
        json_string_array_field("mainline_pending_goals", &selection.pending_goals),
    ]
}

pub(crate) fn mainline_text_lines(selection: &MainlineSelection) -> Vec<String> {
    vec![
        format!("  mainline_protocol: {MAINLINE_PROTOCOL}"),
        format!("  mainline_plan_source: {MAINLINE_SOURCE}"),
        format!("  mainline_status: {}", selection.status),
        format!("  mainline_id: {}", selection.id),
        format!("  mainline_target: {}", selection.target),
        format!("  mainline_selected: {}", selection.selected),
        format!("  mainline_reason: {}", selection.reason),
        format!(
            "  mainline_dependency_path: {}",
            selection.dependency_path.join(" -> ")
        ),
        format!(
            "  mainline_pending_goals: {}",
            selection.pending_goals.join(", ")
        ),
    ]
}
