use crate::{
    dev_tensor::{
        dev_tensor_coverage_summary, dev_tensor_drift_summary, dev_tensor_summary, DevTensorCell,
    },
    dev_tensor_data::DEV_TENSOR_CELLS,
    dev_tensor_drift::DevTensorDriftCheck,
    dev_tensor_hierarchy::{dev_tensor_hierarchy_summary, DevTensorHierarchyNode},
    dev_tensor_status::{DevTensorStatusProtocolEntry, DEV_TENSOR_STATUS_PROTOCOL},
    json_bool_field, json_field, json_string_array_field, json_usize_field,
    surface_render::append_json_field_strings,
};

pub(crate) fn render_dev_tensor_json_impl() -> String {
    let summary = dev_tensor_summary();
    let coverage = dev_tensor_coverage_summary();
    let drift = dev_tensor_drift_summary();
    let hierarchy = dev_tensor_hierarchy_summary();
    let cells = DEV_TENSOR_CELLS
        .iter()
        .map(dev_tensor_cell_json)
        .collect::<Vec<_>>();
    let status_protocol = DEV_TENSOR_STATUS_PROTOCOL
        .iter()
        .map(dev_tensor_status_protocol_json)
        .collect::<Vec<_>>();
    let drift_checks = drift
        .checks
        .iter()
        .map(dev_tensor_drift_check_json)
        .collect::<Vec<_>>();
    let mut out = String::from("{");
    append_json_field_strings(
        &mut out,
        vec![
            json_field("kind", "nuis_dev_tensor"),
            json_field("model", "architecture-module-function-progress-tensor"),
            json_field("version", "dev-tensor-v1"),
            json_field("status_protocol_version", hierarchy.protocol_version),
            json_usize_field("axis_count", 3),
            json_field("axis_0", "architecture"),
            json_field("axis_1", "module"),
            json_field("axis_2", "function"),
            json_field("hierarchy_root_status", hierarchy.root.status),
            json_usize_field("hierarchy_root_progress", hierarchy.root.progress),
            json_field(
                "hierarchy_root_weakest_child_path",
                hierarchy
                    .root
                    .weakest_child_path
                    .as_deref()
                    .unwrap_or("<none>"),
            ),
            json_usize_field("architecture_count", summary.architecture_count),
            json_usize_field("module_count", summary.module_count),
            json_usize_field("function_count", summary.function_count),
            json_usize_field("cell_count", summary.cell_count),
            json_usize_field("average_progress", summary.average_progress),
            json_usize_field("bootstrap_critical_count", summary.bootstrap_critical_count),
            json_usize_field(
                "bootstrap_critical_average_progress",
                summary.bootstrap_critical_average_progress,
            ),
            json_field(
                "weakest_bootstrap_architecture",
                summary.weakest_bootstrap_architecture,
            ),
            json_field("weakest_bootstrap_module", summary.weakest_bootstrap_module),
            json_field(
                "weakest_bootstrap_function",
                summary.weakest_bootstrap_function,
            ),
            json_field("weakest_bootstrap_status", summary.weakest_bootstrap_status),
            json_usize_field(
                "weakest_bootstrap_progress",
                summary.weakest_bootstrap_progress,
            ),
            json_field(
                "weakest_bootstrap_closure_role",
                summary.weakest_bootstrap_closure_role,
            ),
            json_field(
                "weakest_bootstrap_evidence",
                summary.weakest_bootstrap_evidence,
            ),
            json_field(
                "weakest_bootstrap_next_step",
                summary.weakest_bootstrap_next_step,
            ),
            json_field(
                "weakest_bootstrap_blocker",
                summary.weakest_bootstrap_blocker,
            ),
            json_field(
                "weakest_bootstrap_next_action",
                summary.weakest_bootstrap_next_action,
            ),
            json_field(
                "weakest_bootstrap_validation_command",
                summary.weakest_bootstrap_validation_command,
            ),
            json_field(
                "weakest_bootstrap_expected_artifact",
                summary.weakest_bootstrap_expected_artifact,
            ),
            json_field("coverage_status", coverage.status),
            json_field("coverage_expected_source", coverage.expected_source),
            json_bool_field(
                "coverage_expected_fallback_used",
                coverage.expected_fallback_used,
            ),
            json_field(
                "coverage_expected_source_error",
                coverage
                    .expected_source_error
                    .as_deref()
                    .unwrap_or("<none>"),
            ),
            json_usize_field("coverage_expected_count", coverage.expected_count),
            json_usize_field("coverage_covered_count", coverage.covered_count),
            json_usize_field("coverage_missing_count", coverage.missing_count),
            json_usize_field(
                "coverage_required_missing_count",
                coverage.required_missing_count,
            ),
            json_usize_field("coverage_orphaned_count", coverage.orphaned_count),
            json_usize_field("coverage_stale_count", coverage.stale_count),
            json_field(
                "coverage_first_gap",
                coverage.first_gap.as_deref().unwrap_or("<none>"),
            ),
            json_string_array_field(
                "coverage_missing_coordinates",
                &coverage.missing_coordinates,
            ),
            json_string_array_field(
                "coverage_orphaned_coordinates",
                &coverage.orphaned_coordinates,
            ),
            json_string_array_field("coverage_stale_coordinates", &coverage.stale_coordinates),
            json_field("manifest_coverage_status", coverage.manifest.status),
            json_field("manifest_coverage_source", coverage.manifest.source),
            json_usize_field(
                "manifest_module_count",
                coverage.manifest.manifest_module_count,
            ),
            json_usize_field(
                "manifest_tracked_module_count",
                coverage.manifest.tracked_manifest_module_count,
            ),
            json_usize_field(
                "manifest_backed_coordinate_count",
                coverage.manifest.manifest_backed_coordinate_count,
            ),
            json_usize_field(
                "manifest_missing_module_count",
                coverage.manifest.manifest_missing_module_count,
            ),
            json_usize_field(
                "manifest_untracked_module_count",
                coverage.manifest.manifest_untracked_module_count,
            ),
            json_field(
                "manifest_first_gap",
                coverage.manifest.first_gap.as_deref().unwrap_or("<none>"),
            ),
            json_string_array_field(
                "manifest_backed_coordinates",
                &coverage.manifest.manifest_backed_coordinates,
            ),
            json_string_array_field(
                "manifest_missing_modules",
                &coverage.manifest.manifest_missing_modules,
            ),
            json_string_array_field(
                "manifest_untracked_modules",
                &coverage.manifest.manifest_untracked_modules,
            ),
            json_field("milestone_coverage_status", coverage.milestone.status),
            json_field("milestone_coverage_source", coverage.milestone.source),
            json_field("milestone_schema", &coverage.milestone.schema),
            json_usize_field("milestone_count", coverage.milestone.milestone_count),
            json_usize_field(
                "milestone_coordinate_count",
                coverage.milestone.milestone_coordinate_count,
            ),
            json_usize_field(
                "milestone_required_coordinate_count",
                coverage.milestone.milestone_required_coordinate_count,
            ),
            json_usize_field(
                "milestone_missing_coordinate_count",
                coverage.milestone.milestone_missing_coordinate_count,
            ),
            json_usize_field(
                "milestone_untracked_coordinate_count",
                coverage.milestone.milestone_untracked_coordinate_count,
            ),
            json_usize_field(
                "milestone_constant_drift_count",
                coverage.milestone.milestone_constant_drift_count,
            ),
            json_field(
                "milestone_first_gap",
                coverage.milestone.first_gap.as_deref().unwrap_or("<none>"),
            ),
            json_string_array_field(
                "milestone_coordinates",
                &coverage.milestone.milestone_coordinates,
            ),
            json_string_array_field(
                "milestone_missing_coordinates",
                &coverage.milestone.milestone_missing_coordinates,
            ),
            json_string_array_field(
                "milestone_untracked_coordinates",
                &coverage.milestone.milestone_untracked_coordinates,
            ),
            json_string_array_field(
                "milestone_constant_drift_coordinates",
                &coverage.milestone.milestone_constant_drift_coordinates,
            ),
            json_usize_field("drift_check_count", drift.check_count),
            json_usize_field("drift_check_passed_count", drift.passed_count),
            json_usize_field("drift_check_failed_count", drift.failed_count),
            json_field("drift_status", drift.status),
            json_field(
                "drift_first_failed_check",
                drift.first_failed_check.unwrap_or("<none>"),
            ),
            format!("\"drift_checks\":[{}]", drift_checks.join(",")),
            format!("\"status_protocol\":[{}]", status_protocol.join(",")),
            format!(
                "\"hierarchy\":{}",
                dev_tensor_hierarchy_node_json(&hierarchy.root)
            ),
            format!("\"cells\":[{}]", cells.join(",")),
        ],
    );
    out.push('}');
    out
}

pub(crate) fn render_dev_tensor_text_impl() -> Vec<String> {
    let summary = dev_tensor_summary();
    let coverage = dev_tensor_coverage_summary();
    let drift = dev_tensor_drift_summary();
    let hierarchy = dev_tensor_hierarchy_summary();
    let mut lines = vec![
        "nuis development tensor".to_owned(),
        "  model: architecture-module-function-progress-tensor".to_owned(),
        "  version: dev-tensor-v1".to_owned(),
        format!("  status_protocol_version: {}", hierarchy.protocol_version),
        "  axes: architecture, module, function".to_owned(),
        format!("  hierarchy_root_status: {}", hierarchy.root.status),
        format!("  hierarchy_root_progress: {}", hierarchy.root.progress),
        format!(
            "  hierarchy_root_weakest_child_path: {}",
            hierarchy
                .root
                .weakest_child_path
                .as_deref()
                .unwrap_or("<none>")
        ),
        format!("  architecture_count: {}", summary.architecture_count),
        format!("  module_count: {}", summary.module_count),
        format!("  function_count: {}", summary.function_count),
        format!("  cell_count: {}", summary.cell_count),
        format!("  average_progress: {}", summary.average_progress),
        format!(
            "  bootstrap_critical_count: {}",
            summary.bootstrap_critical_count
        ),
        format!(
            "  bootstrap_critical_average_progress: {}",
            summary.bootstrap_critical_average_progress
        ),
        format!(
            "  weakest_bootstrap_architecture: {}",
            summary.weakest_bootstrap_architecture
        ),
        format!(
            "  weakest_bootstrap_module: {}",
            summary.weakest_bootstrap_module
        ),
        format!(
            "  weakest_bootstrap_function: {}",
            summary.weakest_bootstrap_function
        ),
        format!(
            "  weakest_bootstrap_status: {}",
            summary.weakest_bootstrap_status
        ),
        format!(
            "  weakest_bootstrap_progress: {}",
            summary.weakest_bootstrap_progress
        ),
        format!(
            "  weakest_bootstrap_closure_role: {}",
            summary.weakest_bootstrap_closure_role
        ),
        format!(
            "  weakest_bootstrap_evidence: {}",
            summary.weakest_bootstrap_evidence
        ),
        format!(
            "  weakest_bootstrap_next_step: {}",
            summary.weakest_bootstrap_next_step
        ),
        format!(
            "  weakest_bootstrap_blocker: {}",
            summary.weakest_bootstrap_blocker
        ),
        format!(
            "  weakest_bootstrap_next_action: {}",
            summary.weakest_bootstrap_next_action
        ),
        format!(
            "  weakest_bootstrap_validation_command: {}",
            summary.weakest_bootstrap_validation_command
        ),
        format!(
            "  weakest_bootstrap_expected_artifact: {}",
            summary.weakest_bootstrap_expected_artifact
        ),
        format!("  coverage_status: {}", coverage.status),
        format!("  coverage_expected_source: {}", coverage.expected_source),
        format!(
            "  coverage_expected_fallback_used: {}",
            coverage.expected_fallback_used
        ),
        format!(
            "  coverage_expected_source_error: {}",
            coverage
                .expected_source_error
                .as_deref()
                .unwrap_or("<none>")
        ),
        format!("  coverage_expected_count: {}", coverage.expected_count),
        format!("  coverage_covered_count: {}", coverage.covered_count),
        format!("  coverage_missing_count: {}", coverage.missing_count),
        format!(
            "  coverage_required_missing_count: {}",
            coverage.required_missing_count
        ),
        format!("  coverage_orphaned_count: {}", coverage.orphaned_count),
        format!("  coverage_stale_count: {}", coverage.stale_count),
        format!(
            "  coverage_first_gap: {}",
            coverage.first_gap.as_deref().unwrap_or("<none>")
        ),
        format!("  manifest_coverage_status: {}", coverage.manifest.status),
        format!("  manifest_coverage_source: {}", coverage.manifest.source),
        format!(
            "  manifest_module_count: {}",
            coverage.manifest.manifest_module_count
        ),
        format!(
            "  manifest_tracked_module_count: {}",
            coverage.manifest.tracked_manifest_module_count
        ),
        format!(
            "  manifest_backed_coordinate_count: {}",
            coverage.manifest.manifest_backed_coordinate_count
        ),
        format!(
            "  manifest_missing_module_count: {}",
            coverage.manifest.manifest_missing_module_count
        ),
        format!(
            "  manifest_untracked_module_count: {}",
            coverage.manifest.manifest_untracked_module_count
        ),
        format!(
            "  manifest_first_gap: {}",
            coverage.manifest.first_gap.as_deref().unwrap_or("<none>")
        ),
        format!("  milestone_coverage_status: {}", coverage.milestone.status),
        format!("  milestone_coverage_source: {}", coverage.milestone.source),
        format!("  milestone_schema: {}", coverage.milestone.schema),
        format!("  milestone_count: {}", coverage.milestone.milestone_count),
        format!(
            "  milestone_coordinate_count: {}",
            coverage.milestone.milestone_coordinate_count
        ),
        format!(
            "  milestone_required_coordinate_count: {}",
            coverage.milestone.milestone_required_coordinate_count
        ),
        format!(
            "  milestone_missing_coordinate_count: {}",
            coverage.milestone.milestone_missing_coordinate_count
        ),
        format!(
            "  milestone_untracked_coordinate_count: {}",
            coverage.milestone.milestone_untracked_coordinate_count
        ),
        format!(
            "  milestone_constant_drift_count: {}",
            coverage.milestone.milestone_constant_drift_count
        ),
        format!(
            "  milestone_first_gap: {}",
            coverage.milestone.first_gap.as_deref().unwrap_or("<none>")
        ),
        format!("  drift_status: {}", drift.status),
        format!("  drift_check_count: {}", drift.check_count),
        format!("  drift_check_passed_count: {}", drift.passed_count),
        format!("  drift_check_failed_count: {}", drift.failed_count),
        format!(
            "  drift_first_failed_check: {}",
            drift.first_failed_check.unwrap_or("<none>")
        ),
    ];
    for coordinate in &coverage.missing_coordinates {
        lines.push(format!("  coverage_missing_coordinate: {coordinate}"));
    }
    for coordinate in &coverage.orphaned_coordinates {
        lines.push(format!("  coverage_orphaned_coordinate: {coordinate}"));
    }
    for coordinate in &coverage.stale_coordinates {
        lines.push(format!("  coverage_stale_coordinate: {coordinate}"));
    }
    for coordinate in &coverage.manifest.manifest_backed_coordinates {
        lines.push(format!("  manifest_backed_coordinate: {coordinate}"));
    }
    for module in &coverage.manifest.manifest_missing_modules {
        lines.push(format!("  manifest_missing_module: {module}"));
    }
    for module in &coverage.manifest.manifest_untracked_modules {
        lines.push(format!("  manifest_untracked_module: {module}"));
    }
    for coordinate in &coverage.milestone.milestone_coordinates {
        lines.push(format!("  milestone_coordinate: {coordinate}"));
    }
    for coordinate in &coverage.milestone.milestone_missing_coordinates {
        lines.push(format!("  milestone_missing_coordinate: {coordinate}"));
    }
    for coordinate in &coverage.milestone.milestone_untracked_coordinates {
        lines.push(format!("  milestone_untracked_coordinate: {coordinate}"));
    }
    for coordinate in &coverage.milestone.milestone_constant_drift_coordinates {
        lines.push(format!(
            "  milestone_constant_drift_coordinate: {coordinate}"
        ));
    }
    for check in &drift.checks {
        lines.push(format!(
            "  drift_check: id={} path={} passed={} missing={}",
            check.id,
            check.path,
            check.passed,
            if check.missing_patterns.is_empty() {
                "<none>".to_owned()
            } else {
                check.missing_patterns.join("|")
            }
        ));
    }
    for entry in DEV_TENSOR_STATUS_PROTOCOL {
        lines.push(format!(
            "  status_protocol: status={} rank={} phase={} terminal={} blocks_bootstrap={}",
            entry.status, entry.rank, entry.phase, entry.terminal, entry.blocks_bootstrap
        ));
    }
    push_dev_tensor_hierarchy_text(&mut lines, &hierarchy.root, 1);
    for cell in DEV_TENSOR_CELLS {
        lines.push(format!(
            "  cell: architecture={} module={} function={} status={} progress={} bootstrap_critical={} closure_role={}",
            cell.architecture,
            cell.module,
            cell.function,
            cell.status,
            cell.progress,
            cell.bootstrap_critical,
            cell.closure_role
        ));
        lines.push(format!("    evidence: {}", cell.evidence));
        lines.push(format!("    next_step: {}", cell.next_step));
        lines.push(format!("    blocker: {}", cell.blocker));
        lines.push(format!("    next_action: {}", cell.next_action));
        lines.push(format!(
            "    validation_command: {}",
            cell.validation_command
        ));
        lines.push(format!("    expected_artifact: {}", cell.expected_artifact));
    }
    lines
}

fn dev_tensor_cell_json(cell: &DevTensorCell) -> String {
    let coordinates = vec![
        cell.architecture.to_owned(),
        cell.module.to_owned(),
        cell.function.to_owned(),
    ];
    format!(
        "{{{}}}",
        [
            json_field("architecture", cell.architecture),
            json_field("module", cell.module),
            json_field("function", cell.function),
            json_string_array_field("coordinates", &coordinates),
            json_field("status", cell.status),
            json_usize_field("progress", cell.progress),
            json_bool_field("bootstrap_critical", cell.bootstrap_critical),
            json_field("closure_role", cell.closure_role),
            json_field("evidence", cell.evidence),
            json_field("next_step", cell.next_step),
            json_field("blocker", cell.blocker),
            json_field("next_action", cell.next_action),
            json_field("validation_command", cell.validation_command),
            json_field("expected_artifact", cell.expected_artifact),
        ]
        .join(",")
    )
}

fn dev_tensor_status_protocol_json(entry: &DevTensorStatusProtocolEntry) -> String {
    format!(
        "{{{}}}",
        [
            json_field("status", entry.status),
            json_usize_field("rank", entry.rank),
            json_field("phase", entry.phase),
            json_bool_field("terminal", entry.terminal),
            json_bool_field("blocks_bootstrap", entry.blocks_bootstrap),
        ]
        .join(",")
    )
}

fn dev_tensor_hierarchy_node_json(node: &DevTensorHierarchyNode) -> String {
    let children = node
        .children
        .iter()
        .map(dev_tensor_hierarchy_node_json)
        .collect::<Vec<_>>();
    format!(
        "{{{}}}",
        [
            json_field("level", node.level),
            json_field("name", &node.name),
            json_field("path", &node.path),
            json_field("status", node.status),
            json_usize_field("status_rank", node.status_rank),
            json_usize_field("progress", node.progress),
            json_usize_field("cell_count", node.cell_count),
            json_usize_field("bootstrap_critical_count", node.bootstrap_critical_count),
            json_field(
                "weakest_child_path",
                node.weakest_child_path.as_deref().unwrap_or("<none>"),
            ),
            format!("\"children\":[{}]", children.join(",")),
        ]
        .join(",")
    )
}

fn push_dev_tensor_hierarchy_text(
    lines: &mut Vec<String>,
    node: &DevTensorHierarchyNode,
    depth: usize,
) {
    let indent = "  ".repeat(depth + 1);
    lines.push(format!(
        "{indent}hierarchy_node: level={} path={} status={} progress={} cells={} weakest={}",
        node.level,
        node.path,
        node.status,
        node.progress,
        node.cell_count,
        node.weakest_child_path.as_deref().unwrap_or("<none>")
    ));
    for child in &node.children {
        push_dev_tensor_hierarchy_text(lines, child, depth + 1);
    }
}

fn dev_tensor_drift_check_json(check: &DevTensorDriftCheck) -> String {
    format!(
        "{{{}}}",
        [
            json_field("id", check.id),
            json_field("path", check.path),
            json_bool_field("passed", check.passed),
            json_string_array_field("missing_patterns", &check.missing_patterns),
        ]
        .join(",")
    )
}
