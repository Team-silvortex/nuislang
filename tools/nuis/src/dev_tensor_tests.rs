use super::*;
use crate::dev_tensor_data::DEV_TENSOR_EXPECTED_COORDINATES;

#[test]
fn handoff_selection_is_status_aware_and_input_order_independent() {
    let selected =
        select_dev_tensor_handoff_bootstrap_cell(DEV_TENSOR_CELLS).expect("select handoff cell");
    let mut reversed = DEV_TENSOR_CELLS.to_vec();
    reversed.reverse();
    let reversed_selected =
        select_dev_tensor_handoff_bootstrap_cell(&reversed).expect("select reversed handoff cell");

    let expected =
        dev_tensor_coordinate_key(selected.architecture, selected.module, selected.function);
    assert_eq!(
        dev_tensor_coordinate_key(
            reversed_selected.architecture,
            reversed_selected.module,
            reversed_selected.function
        ),
        expected
    );
    assert_eq!(selected.status, "stable");
}

#[test]
fn task_selection_falls_back_to_global_incomplete_after_bootstrap_closes() {
    let selected = select_dev_tensor_task_cell(DEV_TENSOR_CELLS).expect("select global task");
    assert_eq!(
        dev_tensor_coordinate_key(selected.architecture, selected.module, selected.function),
        "standard-library/pixelmagic/image-processing-lane"
    );
    assert!(!selected.bootstrap_critical);
    assert_eq!(selected.status, "active");
    assert_eq!(selected.progress, 78);
}

#[test]
fn task_selection_keeps_bootstrap_priority_until_critical_cells_close() {
    let cells = [
        DevTensorCell {
            architecture: "optional",
            module: "lane",
            function: "weaker",
            status: "early",
            progress: 10,
            bootstrap_critical: false,
            closure_role: "optional",
            evidence: "evidence",
            next_step: "next",
            blocker: "none",
            next_action: "advance optional",
            validation_command: "validate optional",
            expected_artifact: "optional artifact",
        },
        DevTensorCell {
            architecture: "bootstrap",
            module: "lane",
            function: "required",
            status: "usable",
            progress: 90,
            bootstrap_critical: true,
            closure_role: "required",
            evidence: "evidence",
            next_step: "next",
            blocker: "none",
            next_action: "close bootstrap",
            validation_command: "validate bootstrap",
            expected_artifact: "bootstrap artifact",
        },
    ];
    let selected = select_dev_tensor_task_cell(&cells).expect("select bootstrap task");
    assert_eq!(selected.architecture, "bootstrap");
}

#[test]
fn dev_tensor_summary_reports_three_axes_and_cells() {
    let summary = dev_tensor_summary();
    assert_eq!(
        summary.hierarchy_protocol_version,
        "nuis-dev-tensor-hierarchy-v1"
    );
    assert_eq!(summary.hierarchy_validation_status, "clean");
    assert_eq!(summary.hierarchy_validation_error_count, 0);
    assert_eq!(summary.hierarchy_validation_first_error, "<none>");
    assert_eq!(summary.cell_count, DEV_TENSOR_CELLS.len());
    assert!(summary.architecture_count >= 5);
    assert!(summary.module_count >= 5);
    assert!(summary.function_count >= 5);
    assert!(summary.average_progress > 0);
    assert!(summary.bootstrap_critical_count >= 5);
    assert!(summary.bootstrap_critical_average_progress > 0);
    assert_ne!(summary.weakest_bootstrap_architecture, "<none>");
    assert_ne!(summary.weakest_bootstrap_module, "<none>");
    assert_ne!(summary.weakest_bootstrap_function, "<none>");
    assert_ne!(summary.weakest_bootstrap_status, "<none>");
    assert!(summary.weakest_bootstrap_progress > 0);
    assert_ne!(summary.weakest_bootstrap_closure_role, "<none>");
    assert_ne!(summary.weakest_bootstrap_evidence, "<none>");
    assert_ne!(summary.weakest_bootstrap_next_step, "<none>");
    assert_ne!(summary.weakest_bootstrap_blocker, "<none>");
    assert_ne!(summary.weakest_bootstrap_next_action, "<none>");
    assert_ne!(summary.weakest_bootstrap_validation_command, "<none>");
    assert_ne!(summary.weakest_bootstrap_expected_artifact, "<none>");
    assert_eq!(
        summary.weakest_bootstrap_task_card_protocol,
        DEV_TENSOR_TASK_CARD_PROTOCOL
    );
    assert_eq!(
        summary.weakest_bootstrap_task_card_source,
        "weakest-global-incomplete-status-progress-path"
    );
    assert_eq!(summary.weakest_bootstrap_task_card_status, "ready");
    assert!(summary.weakest_bootstrap_task_card_ready);
    assert_ne!(summary.weakest_bootstrap_task_card_coordinate, "<none>");
    assert!(summary.weakest_bootstrap_task_card_coordinate.contains('/'));
    assert!(summary
        .weakest_bootstrap_task_card_priority_reason
        .contains("all bootstrap-critical cells are stable at 100/100"));
    assert_eq!(
        summary.weakest_bootstrap_task_card_coordinate,
        "standard-library/pixelmagic/image-processing-lane"
    );
    assert_ne!(
        summary.weakest_bootstrap_task_card_handoff_coordinate,
        "<none>"
    );
    let selected =
        select_dev_tensor_task_cell(DEV_TENSOR_CELLS).expect("select current weakest handoff");
    assert_eq!(
        summary.weakest_bootstrap_task_card_handoff_coordinate,
        dev_tensor_coordinate_key(selected.architecture, selected.module, selected.function)
    );
    assert!(summary
        .weakest_bootstrap_task_card_handoff_coordinate
        .contains('/'));
    assert_ne!(summary.weakest_bootstrap_task_card_handoff_mode, "<none>");
    assert!(summary
        .weakest_bootstrap_task_card_handoff_reason
        .contains("weakest"));
    assert_ne!(summary.weakest_bootstrap_task_card_handoff_action, "<none>");
    assert_ne!(
        summary.weakest_bootstrap_task_card_handoff_command,
        "<none>"
    );
    assert_ne!(
        summary.weakest_bootstrap_task_card_handoff_expected_artifact,
        "<none>"
    );
    assert_eq!(
        summary.weakest_bootstrap_task_card_lineage.protocol,
        "nuis-dev-tensor-task-card-lineage-v1"
    );
    assert_eq!(summary.weakest_bootstrap_task_card_lineage.status, "clean");
    assert_eq!(summary.weakest_bootstrap_task_card_lineage.error_count, 0);
    assert!(summary
        .weakest_bootstrap_task_card_lineage
        .first_error
        .is_none());
    assert_eq!(
        summary
            .weakest_bootstrap_task_card_lineage
            .task_ancestry
            .last(),
        Some(&summary.weakest_bootstrap_task_card_coordinate)
    );
    assert_eq!(
        summary
            .weakest_bootstrap_task_card_lineage
            .handoff_ancestry
            .last(),
        Some(&summary.weakest_bootstrap_task_card_handoff_coordinate)
    );
    assert_eq!(
        summary
            .weakest_bootstrap_task_card_lineage
            .task_ancestry
            .first()
            .map(String::as_str),
        Some("nuislang")
    );
    assert_ne!(
        summary
            .weakest_bootstrap_task_card_lineage
            .common_ancestor_path,
        "<none>"
    );
    assert_eq!(summary.bootstrap_critical_average_progress, 100);
    let hierarchy = crate::dev_tensor_hierarchy::dev_tensor_hierarchy_summary();
    assert_eq!(
        hierarchy.hierarchy_protocol_version,
        "nuis-dev-tensor-hierarchy-v1"
    );
    assert_eq!(hierarchy.status_protocol_version, "dev-tensor-status-v1");
    assert_eq!(hierarchy.validation.status, "clean");
    assert_eq!(hierarchy.validation.error_count, 0);
    assert_eq!(hierarchy.validation.max_depth, 3);
    assert!(hierarchy.validation.node_count > DEV_TENSOR_CELLS.len());
    assert_eq!(hierarchy.root.level, "root");
    assert_eq!(hierarchy.root.cell_count, DEV_TENSOR_CELLS.len());
    assert!(!hierarchy.root.children.is_empty());
    assert!(hierarchy.root.weakest_child_path.is_some());
    assert!(hierarchy.root.status_rank > 0);
    assert_eq!(summary.coverage_status, "clean");
    assert_eq!(
        summary.coverage_expected_count,
        DEV_TENSOR_EXPECTED_COORDINATES.len()
    );
    assert_eq!(summary.coverage_missing_count, 0);
    assert_eq!(summary.coverage_orphaned_count, 0);
    assert_eq!(summary.coverage_stale_count, 0);
    let coverage = dev_tensor_coverage_summary();
    assert_eq!(
        coverage.expected_source,
        "docs/reference/nuis-development-tensor.milestones.toml"
    );
    assert!(!coverage.expected_fallback_used);
    assert!(coverage.expected_source_error.is_none());
    assert_eq!(coverage.manifest.status, "clean");
    assert!(coverage.manifest.manifest_backed_coordinate_count >= 3);
    assert_eq!(coverage.milestone.status, "clean");
    assert_eq!(
        coverage.milestone.derived_cache_protocol,
        "nuis-dev-tensor-derived-coordinate-cache-v1"
    );
    assert_eq!(coverage.milestone.derived_cache_status, "cacheable");
    assert!(coverage
        .milestone
        .derived_cache_key
        .starts_with("nuis-dev-tensor-derived-coordinate-cache-v1:fnv64:"));
    assert_eq!(
        coverage.milestone.derived_cache_coordinate_count,
        DEV_TENSOR_EXPECTED_COORDINATES.len()
    );
    assert_eq!(
        coverage.milestone.milestone_coordinate_count,
        DEV_TENSOR_EXPECTED_COORDINATES.len()
    );
}

#[test]
fn dev_tensor_json_exposes_coordinate_cells() {
    let json = render_dev_tensor_json();
    assert!(json.contains("\"kind\":\"nuis_dev_tensor\""));
    assert!(json.contains("\"status_protocol_version\":\"dev-tensor-status-v1\""));
    assert!(json.contains("\"axis_0\":\"architecture\""));
    assert!(json.contains("\"axis_1\":\"module\""));
    assert!(json.contains("\"axis_2\":\"function\""));
    assert!(json.contains("\"hierarchy_root_status\""));
    assert!(json.contains("\"hierarchy_root_weakest_child_path\""));
    assert!(json.contains("\"hierarchy_protocol_version\":\"nuis-dev-tensor-hierarchy-v1\""));
    assert!(json.contains("\"hierarchy_validation_status\":\"clean\""));
    assert!(json.contains("\"hierarchy_validation_error_count\":0"));
    assert!(json.contains("\"hierarchy_validation_max_depth\":3"));
    assert!(json.contains("\"status_protocol\":["));
    assert!(json.contains("\"hierarchy\":{\"level\":\"root\""));
    assert!(json.contains("\"children\":["));
    assert!(json.contains("\"rank\":4"));
    assert!(json.contains("\"phase\":\"validated\""));
    assert!(json.contains("\"coordinates\":["));
    assert!(json.contains("\"bootstrap_critical\":true"));
    assert!(json.contains("\"closure_role\":\"self-owned-native-binary\""));
    assert!(json.contains("\"weakest_bootstrap_architecture\""));
    assert!(json.contains("\"weakest_bootstrap_module\""));
    assert!(json.contains("\"weakest_bootstrap_function\""));
    assert!(json.contains("\"weakest_bootstrap_status\""));
    assert!(json.contains("\"weakest_bootstrap_closure_role\""));
    assert!(json.contains("\"weakest_bootstrap_evidence\""));
    assert!(json.contains("\"weakest_bootstrap_next_step\""));
    assert!(json.contains("\"weakest_bootstrap_blocker\""));
    assert!(json.contains("\"weakest_bootstrap_next_action\""));
    assert!(json.contains("\"weakest_bootstrap_validation_command\""));
    assert!(json.contains("\"weakest_bootstrap_expected_artifact\""));
    assert!(
        json.contains("\"weakest_bootstrap_task_card_protocol\":\"nuis-dev-tensor-task-card-v1\"")
    );
    assert!(json.contains(
        "\"weakest_bootstrap_task_card_source\":\"weakest-global-incomplete-status-progress-path\""
    ));
    assert!(json.contains("\"weakest_bootstrap_task_card_status\":\"ready\""));
    assert!(json.contains("\"weakest_bootstrap_task_card_ready\":true"));
    assert!(json.contains("\"weakest_bootstrap_task_card_coordinate\""));
    assert!(json.contains("\"weakest_bootstrap_task_card_priority_reason\""));
    assert!(json.contains("\"weakest_bootstrap_task_card_action\""));
    assert!(json.contains("\"weakest_bootstrap_task_card_command\""));
    assert!(json.contains("\"weakest_bootstrap_task_card_expected_artifact\""));
    assert!(json.contains("\"weakest_bootstrap_task_card_handoff_mode\""));
    assert!(json.contains("\"weakest_bootstrap_task_card_handoff_coordinate\""));
    assert!(json.contains("\"weakest_bootstrap_task_card_handoff_reason\""));
    assert!(json.contains("\"weakest_bootstrap_task_card_handoff_action\""));
    assert!(json.contains("\"weakest_bootstrap_task_card_handoff_command\""));
    assert!(json.contains("\"weakest_bootstrap_task_card_handoff_expected_artifact\""));
    assert!(json.contains(
        "\"weakest_bootstrap_task_card_lineage_protocol\":\"nuis-dev-tensor-task-card-lineage-v1\""
    ));
    assert!(json.contains("\"weakest_bootstrap_task_card_lineage_status\":\"clean\""));
    assert!(json.contains("\"weakest_bootstrap_task_card_lineage_error_count\":0"));
    assert!(json.contains("\"weakest_bootstrap_task_card_lineage_errors\":[]"));
    assert!(json.contains("\"weakest_bootstrap_task_card_task_ancestry\":[\"nuislang\""));
    assert!(json.contains("\"weakest_bootstrap_task_card_handoff_ancestry\":[\"nuislang\""));
    assert!(json.contains("\"weakest_bootstrap_task_card_common_ancestor_path\""));
    assert!(json.contains("\"weakest_bootstrap_task_card_transition_depth\":"));
    assert!(json.contains("all bootstrap-critical cells are stable at 100/100"));
    assert!(json.contains("\"blocker\""));
    assert!(json.contains("\"next_action\""));
    assert!(json.contains("\"validation_command\""));
    assert!(json.contains("\"expected_artifact\""));
    assert!(json.contains("\"module\":\"nsld\""));
    assert!(json.contains("\"function\":\"final-output-boundary\""));
    assert!(json.contains("\"coverage_status\":\"clean\""));
    assert!(json.contains(
        "\"coverage_expected_source\":\"docs/reference/nuis-development-tensor.milestones.toml\""
    ));
    assert!(json.contains("\"coverage_expected_fallback_used\":false"));
    assert!(json.contains("\"coverage_expected_source_error\":\"<none>\""));
    assert!(json.contains("\"coverage_expected_count\":"));
    assert!(json.contains("\"coverage_missing_count\":0"));
    assert!(json.contains("\"coverage_orphaned_count\":0"));
    assert!(json.contains("\"coverage_stale_count\":0"));
    assert!(json.contains("\"manifest_coverage_status\":\"clean\""));
    assert!(json.contains("\"manifest_coverage_source\":\"stdlib/index.toml\""));
    assert!(json.contains("\"manifest_backed_coordinates\":["));
    assert!(json.contains("\"standard-library/std/host-io-filesystem-text\""));
    assert!(json.contains("\"manifest_untracked_modules\":["));
    assert!(json.contains("\"milestone_coverage_status\":\"clean\""));
    assert!(json.contains(
        "\"milestone_coverage_source\":\"docs/reference/nuis-development-tensor.milestones.toml\""
    ));
    assert!(json.contains(
        "\"milestone_derived_cache_protocol\":\"nuis-dev-tensor-derived-coordinate-cache-v1\""
    ));
    assert!(json.contains("\"milestone_derived_cache_status\":\"cacheable\""));
    assert!(json.contains(
        "\"milestone_derived_cache_key\":\"nuis-dev-tensor-derived-coordinate-cache-v1:fnv64:"
    ));
    assert!(json.contains("\"milestone_derived_cache_coordinate_count\":"));
    assert!(json.contains("\"milestone_constant_drift_count\":0"));
    assert!(json.contains("\"milestone_coordinates\":["));
    assert!(json.contains("\"coverage_missing_coordinates\":[]"));
    assert!(json.contains("\"drift_status\":\"clean\""));
    assert!(json.contains("\"drift_checks\":["));
    assert!(json.contains("\"id\":\"frontdoor-self-owned-image-status\""));
    assert!(json.contains("\"id\":\"std-filesystem-light-smoke\""));
    assert!(json.contains("\"missing_patterns\":[]"));
}

#[test]
fn dev_tensor_drift_checks_are_currently_clean() {
    let drift = dev_tensor_drift_summary();
    assert_eq!(drift.status, "clean");
    assert_eq!(drift.failed_count, 0);
    assert_eq!(drift.passed_count, drift.check_count);
    assert!(drift.first_failed_check.is_none());
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "frontdoor-self-owned-image-status"));
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "std-filesystem-light-smoke"));
}

#[test]
fn dev_tensor_text_exposes_drift_status() {
    let text = render_dev_tensor_text().join("\n");
    assert!(text.contains("coverage_status: clean"));
    assert!(text.contains(
        "coverage_expected_source: docs/reference/nuis-development-tensor.milestones.toml"
    ));
    assert!(text.contains("coverage_expected_fallback_used: false"));
    assert!(text.contains("coverage_expected_source_error: <none>"));
    assert!(text.contains("coverage_missing_count: 0"));
    assert!(text.contains("coverage_orphaned_count: 0"));
    assert!(text.contains("coverage_stale_count: 0"));
    assert!(text.contains("manifest_coverage_status: clean"));
    assert!(text.contains("manifest_coverage_source: stdlib/index.toml"));
    assert!(
        text.contains("manifest_backed_coordinate: standard-library/std/host-io-filesystem-text")
    );
    assert!(text.contains("manifest_untracked_module: core"));
    assert!(text.contains("milestone_coverage_status: clean"));
    assert!(text.contains(
        "milestone_coverage_source: docs/reference/nuis-development-tensor.milestones.toml"
    ));
    assert!(text
        .contains("milestone_derived_cache_protocol: nuis-dev-tensor-derived-coordinate-cache-v1"));
    assert!(text.contains("milestone_derived_cache_status: cacheable"));
    assert!(text.contains(
        "milestone_derived_cache_key: nuis-dev-tensor-derived-coordinate-cache-v1:fnv64:"
    ));
    assert!(text.contains("milestone_derived_cache_coordinate_count:"));
    assert!(text.contains("milestone_constant_drift_count: 0"));
    assert!(text.contains(
            "milestone_coordinate: alpha-governance:required:developer-system/dev-tensor/architecture-module-function-progress-model"
        ));
    assert!(text.contains("drift_status: clean"));
    assert!(text.contains("status_protocol_version: dev-tensor-status-v1"));
    assert!(text.contains("hierarchy_root_status:"));
    assert!(text.contains("hierarchy_root_weakest_child_path:"));
    assert!(text.contains("hierarchy_protocol_version: nuis-dev-tensor-hierarchy-v1"));
    assert!(text.contains("hierarchy_validation_status: clean"));
    assert!(text.contains("hierarchy_validation_error_count: 0"));
    assert!(text.contains("hierarchy_validation_max_depth: 3"));
    assert!(text.contains("weakest_bootstrap_next_step:"));
    assert!(text.contains("weakest_bootstrap_evidence:"));
    assert!(text.contains("weakest_bootstrap_blocker:"));
    assert!(text.contains("weakest_bootstrap_next_action:"));
    assert!(text.contains("weakest_bootstrap_validation_command:"));
    assert!(text.contains("weakest_bootstrap_expected_artifact:"));
    assert!(text.contains("weakest_bootstrap_task_card_protocol: nuis-dev-tensor-task-card-v1"));
    assert!(text.contains(
        "weakest_bootstrap_task_card_source: weakest-global-incomplete-status-progress-path"
    ));
    assert!(text.contains("weakest_bootstrap_task_card_status: ready"));
    assert!(text.contains("weakest_bootstrap_task_card_ready: true"));
    assert!(text.contains("weakest_bootstrap_task_card_coordinate:"));
    assert!(text.contains("weakest_bootstrap_task_card_priority_reason:"));
    assert!(text.contains("weakest_bootstrap_task_card_action:"));
    assert!(text.contains("weakest_bootstrap_task_card_command:"));
    assert!(text.contains("weakest_bootstrap_task_card_expected_artifact:"));
    assert!(text.contains("weakest_bootstrap_task_card_handoff_mode:"));
    assert!(text.contains("weakest_bootstrap_task_card_handoff_coordinate:"));
    assert!(text.contains("weakest_bootstrap_task_card_handoff_reason:"));
    assert!(text.contains("weakest_bootstrap_task_card_handoff_action:"));
    assert!(text.contains("weakest_bootstrap_task_card_handoff_command:"));
    assert!(text.contains("weakest_bootstrap_task_card_handoff_expected_artifact:"));
    assert!(text.contains(
        "weakest_bootstrap_task_card_lineage_protocol: nuis-dev-tensor-task-card-lineage-v1"
    ));
    assert!(text.contains("weakest_bootstrap_task_card_lineage_status: clean"));
    assert!(text.contains("weakest_bootstrap_task_card_lineage_error_count: 0"));
    assert!(text.contains("weakest_bootstrap_task_card_task_ancestor: nuislang"));
    assert!(text.contains("weakest_bootstrap_task_card_handoff_ancestor: nuislang"));
    assert!(text.contains("weakest_bootstrap_task_card_common_ancestor_path:"));
    assert!(text.contains("weakest_bootstrap_task_card_transition_depth:"));
    assert!(text.contains("all bootstrap-critical cells are stable at 100/100"));
    assert!(text.contains("    blocker:"));
    assert!(text.contains("    next_action:"));
    assert!(text.contains("    validation_command:"));
    assert!(text.contains("    expected_artifact:"));
    assert!(text.contains("status_protocol: status=stable rank=4"));
    assert!(text.contains("hierarchy_node: level=root path=nuislang"));
    assert!(text.contains("drift_check: id=frontdoor-final-output-boundary-status"));
    assert!(text.contains("drift_check: id=std-filesystem-light-smoke"));
    assert!(text.contains("drift_first_failed_check: <none>"));
}

#[test]
fn dev_tensor_coverage_manifest_matches_current_cells() {
    let coverage = dev_tensor_coverage_summary();
    assert_eq!(coverage.status, "clean");
    assert_eq!(
        coverage.expected_source,
        "docs/reference/nuis-development-tensor.milestones.toml"
    );
    assert!(!coverage.expected_fallback_used);
    assert!(coverage.expected_source_error.is_none());
    assert_eq!(
        coverage.expected_count,
        DEV_TENSOR_EXPECTED_COORDINATES.len()
    );
    assert_eq!(coverage.covered_count, DEV_TENSOR_CELLS.len());
    assert_eq!(coverage.missing_count, 0);
    assert_eq!(coverage.required_missing_count, 0);
    assert_eq!(coverage.orphaned_count, 0);
    assert_eq!(coverage.stale_count, 0);
    assert_eq!(coverage.manifest.status, "clean");
    assert_eq!(coverage.manifest.manifest_missing_module_count, 0);
    assert!(coverage.manifest.manifest_untracked_module_count >= 1);
    assert_eq!(coverage.milestone.status, "clean");
    assert_eq!(coverage.milestone.milestone_missing_coordinate_count, 0);
    assert_eq!(coverage.milestone.milestone_untracked_coordinate_count, 0);
    assert_eq!(coverage.milestone.milestone_constant_drift_count, 0);
    assert!(coverage.first_gap.is_none());
    assert!(coverage.missing_coordinates.is_empty());
    assert!(coverage.orphaned_coordinates.is_empty());
    assert!(coverage.stale_coordinates.is_empty());
}
