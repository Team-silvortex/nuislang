use crate::{dev_tensor::DevTensorSummary, json_field, json_string_array_field, json_usize_field};

pub(crate) fn dev_tensor_task_card_lineage_json_fields(summary: &DevTensorSummary) -> Vec<String> {
    let lineage = &summary.weakest_bootstrap_task_card_lineage;
    vec![
        json_field(
            "weakest_bootstrap_task_card_lineage_protocol",
            lineage.protocol,
        ),
        json_field("weakest_bootstrap_task_card_lineage_status", lineage.status),
        json_usize_field(
            "weakest_bootstrap_task_card_lineage_error_count",
            lineage.error_count,
        ),
        json_field(
            "weakest_bootstrap_task_card_lineage_first_error",
            lineage.first_error.as_deref().unwrap_or("<none>"),
        ),
        json_string_array_field(
            "weakest_bootstrap_task_card_lineage_errors",
            &lineage.errors,
        ),
        json_string_array_field(
            "weakest_bootstrap_task_card_task_ancestry",
            &lineage.task_ancestry,
        ),
        json_string_array_field(
            "weakest_bootstrap_task_card_handoff_ancestry",
            &lineage.handoff_ancestry,
        ),
        json_field(
            "weakest_bootstrap_task_card_common_ancestor_path",
            &lineage.common_ancestor_path,
        ),
        json_usize_field(
            "weakest_bootstrap_task_card_transition_depth",
            lineage.transition_depth,
        ),
    ]
}

pub(crate) fn dev_tensor_task_card_lineage_text_lines(summary: &DevTensorSummary) -> Vec<String> {
    let lineage = &summary.weakest_bootstrap_task_card_lineage;
    let mut lines = vec![
        format!(
            "  weakest_bootstrap_task_card_lineage_protocol: {}",
            lineage.protocol
        ),
        format!(
            "  weakest_bootstrap_task_card_lineage_status: {}",
            lineage.status
        ),
        format!(
            "  weakest_bootstrap_task_card_lineage_error_count: {}",
            lineage.error_count
        ),
        format!(
            "  weakest_bootstrap_task_card_lineage_first_error: {}",
            lineage.first_error.as_deref().unwrap_or("<none>")
        ),
        format!(
            "  weakest_bootstrap_task_card_common_ancestor_path: {}",
            lineage.common_ancestor_path
        ),
        format!(
            "  weakest_bootstrap_task_card_transition_depth: {}",
            lineage.transition_depth
        ),
    ];
    lines.extend(
        lineage
            .task_ancestry
            .iter()
            .map(|path| format!("  weakest_bootstrap_task_card_task_ancestor: {path}")),
    );
    lines.extend(
        lineage
            .handoff_ancestry
            .iter()
            .map(|path| format!("  weakest_bootstrap_task_card_handoff_ancestor: {path}")),
    );
    lines
}
