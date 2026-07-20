use crate::closure_summary::DebuggerCursorLineageClosureMirror;

pub(crate) fn lineage_repair_json_fields(
    mirror: Option<&DebuggerCursorLineageClosureMirror>,
) -> Vec<String> {
    vec![
        crate::json_optional_string_field(
            "closure_summary_debugger_cursor_lineage_repair_contract",
            mirror.map(|mirror| mirror.repair_contract.as_str()),
        ),
        crate::json_optional_string_field(
            "closure_summary_debugger_cursor_lineage_repair_path",
            mirror.map(|mirror| mirror.repair_path.as_str()),
        ),
        crate::json_optional_string_field(
            "closure_summary_debugger_cursor_lineage_repair_status",
            mirror.map(|mirror| mirror.repair_status.as_str()),
        ),
        optional_usize(
            "closure_summary_debugger_cursor_lineage_repair_entry_count",
            mirror.map(|mirror| mirror.repair_entry_count),
        ),
        crate::json_optional_bool_field(
            "closure_summary_debugger_cursor_lineage_repair_latest_mutated",
            mirror.and_then(|mirror| mirror.repair_latest_mutated),
        ),
        crate::json_optional_string_field(
            "closure_summary_debugger_cursor_lineage_repair_latest_archived_path",
            mirror.and_then(|mirror| mirror.repair_latest_archived_path.as_deref()),
        ),
        crate::json_optional_string_field(
            "closure_summary_debugger_cursor_lineage_repair_latest_archived_hash",
            mirror.and_then(|mirror| mirror.repair_latest_archived_hash.as_deref()),
        ),
        crate::json_optional_string_field(
            "closure_summary_debugger_cursor_lineage_repair_latest_rebuilt_hash",
            mirror.and_then(|mirror| mirror.repair_latest_rebuilt_hash.as_deref()),
        ),
    ]
}

fn optional_usize(name: &str, value: Option<usize>) -> String {
    value.map_or_else(
        || format!("\"{name}\":null"),
        |value| format!("\"{name}\":{value}"),
    )
}
