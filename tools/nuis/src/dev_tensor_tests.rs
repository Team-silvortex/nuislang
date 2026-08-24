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
    assert_eq!(
        expected,
        "developer-system/bootstrap/differential-reproducibility-gate"
    );
    assert_eq!(selected.status, "early");
}

#[test]
fn handoff_advances_to_current_weakest_self_hosting_gate() {
    let mut cells = DEV_TENSOR_CELLS.to_vec();
    let data_model = cells
        .iter_mut()
        .find(|cell| cell.function == "compiler-data-model")
        .expect("compiler data model cell");
    data_model.status = "stable";
    data_model.progress = 100;
    let selected = select_dev_tensor_handoff_bootstrap_cell(&cells).expect("select handoff cell");
    assert_eq!(
        dev_tensor_coordinate_key(selected.architecture, selected.module, selected.function),
        "developer-system/bootstrap/differential-reproducibility-gate"
    );
}

#[test]
fn task_selection_reports_none_after_every_registered_cell_closes() {
    let mut cells = DEV_TENSOR_CELLS.to_vec();
    for cell in &mut cells {
        cell.status = "stable";
        cell.progress = 100;
    }
    assert!(select_dev_tensor_task_cell(&cells).is_none());
}

#[test]
fn task_selection_advances_to_linux_cuda_after_previous_cells_close() {
    let mut cells = DEV_TENSOR_CELLS.to_vec();
    for cell in &mut cells {
        cell.status = "stable";
        cell.progress = 100;
    }
    let cuda = cells
        .iter_mut()
        .find(|cell| cell.function == "cuda-provider-bringup")
        .expect("CUDA cell");
    cuda.status = "active";
    cuda.progress = 99;
    let selected = select_dev_tensor_task_cell(&cells).expect("select CUDA task");
    assert_eq!(
        dev_tensor_coordinate_key(selected.architecture, selected.module, selected.function),
        "heterogeneous-runtime/linux-cuda/cuda-provider-bringup"
    );
    assert_eq!(selected.status, "active");
    assert_eq!(selected.progress, 99);
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
        "weakest-bootstrap-status-progress-path"
    );
    assert_eq!(summary.weakest_bootstrap_task_card_status, "ready");
    assert!(summary.weakest_bootstrap_task_card_ready);
    assert_eq!(
        summary.weakest_bootstrap_task_card_coordinate,
        "developer-system/bootstrap/differential-reproducibility-gate"
    );
    assert!(summary
        .weakest_bootstrap_task_card_priority_reason
        .contains("weakest bootstrap-critical status/progress ordering"));
    assert_eq!(
        summary.weakest_bootstrap_task_card_handoff_coordinate,
        "developer-system/bootstrap/differential-reproducibility-gate"
    );
    assert_eq!(summary.weakest_bootstrap_task_card_handoff_mode, "direct");
    assert!(summary
        .weakest_bootstrap_task_card_handoff_reason
        .contains("weakest task card is directly actionable"));
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
            .first()
            .map(String::as_str),
        Some("nuislang")
    );
    assert_eq!(
        summary.weakest_bootstrap_task_card_lineage.task_ancestry,
        summary.weakest_bootstrap_task_card_lineage.handoff_ancestry
    );
    assert_eq!(
        summary
            .weakest_bootstrap_task_card_lineage
            .common_ancestor_path,
        "developer-system/bootstrap/differential-reproducibility-gate"
    );
    assert_eq!(
        summary.weakest_bootstrap_task_card_lineage.transition_depth,
        0
    );
    assert!(summary.bootstrap_critical_average_progress < 100);
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
fn self_hosting_phase_roadmap_is_protocolized_without_claiming_completion() {
    let cell = DEV_TENSOR_CELLS
        .iter()
        .find(|cell| {
            dev_tensor_coordinate_key(cell.architecture, cell.module, cell.function)
                == "developer-system/dev-tensor/self-hosting-phase-roadmap"
        })
        .expect("self-hosting phase roadmap cell");

    assert_eq!(cell.status, "stable");
    assert_eq!(cell.progress, 100);
    assert!(cell.bootstrap_critical);
    assert_eq!(cell.closure_role, "self-hosting-phase-governance");
    assert!(cell.evidence.contains("nuis-self-hosting-phase-roadmap-v1"));
    assert!(cell.evidence.contains("beta-0.9.*"));
    assert!(cell.evidence.contains("beta-0.10.*"));
    assert!(cell.evidence.contains("gamma-0.5.*"));
    assert!(cell.evidence.contains("gamma-0.10.*"));
    assert!(cell
        .blocker
        .contains("self-hosting implementation intentionally remains future work"));
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
        "\"weakest_bootstrap_task_card_source\":\"weakest-bootstrap-status-progress-path\""
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
    assert!(json.contains("\"weakest_bootstrap_task_card_task_ancestry\":["));
    assert!(json.contains("\"weakest_bootstrap_task_card_handoff_ancestry\":["));
    assert!(json.contains("standard-library/std/concurrency-task-thread-lock"));
    assert!(json.contains("\"weakest_bootstrap_task_card_common_ancestor_path\""));
    assert!(json.contains("\"weakest_bootstrap_task_card_transition_depth\":"));
    assert!(json.contains("weakest bootstrap-critical status/progress ordering"));
    assert!(json.contains("\"module\":\"nuis-runtime\""));
    assert!(json.contains("\"function\":\"lifecycle-loader-bootstrap\""));
    assert!(json.contains("\"function\":\"lifecycle-context-dispatch\""));
    assert!(json.contains("nuis-runtime-lifecycle-bootstrap-plan-v1"));
    assert!(json.contains("\"module\":\"linux-cuda\""));
    assert!(json.contains("\"function\":\"cuda-provider-bringup\""));
    assert!(json.contains("nuis-linux-cuda-host-probe-v1"));
    assert!(json.contains("nuis-cuda-device-inventory-v1"));
    assert!(json.contains("capability-ranked-lowest-ordinal"));
    assert!(json.contains("\"module\":\"linux-vulkan\""));
    assert!(json.contains("\"function\":\"vulkan-provider-bringup\""));
    assert!(json.contains("nuis-vulkan-host-probe-v1"));
    assert!(json.contains("spirv.vulkan-gpu.bundle.v1"));
    assert!(json.contains("\"blocker\""));
    assert!(json.contains("\"next_action\""));
    assert!(json.contains("\"validation_command\""));
    assert!(json.contains("\"expected_artifact\""));
    assert!(json.contains("\"module\":\"nsld\""));
    assert!(json.contains("\"function\":\"final-output-boundary\""));
    assert!(json.contains("\"function\":\"self-hosting-phase-roadmap\""));
    assert!(json.contains("nuis-self-hosting-phase-roadmap-v1"));
    assert!(json.contains("beta-0.10.*"));
    assert!(json.contains("gamma-0.5.*"));
    assert!(json.contains("gamma-0.10.*"));
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
    assert!(json.contains("\"id\":\"cffi-memory-capability-canonical-hash\""));
    assert!(json.contains("\"id\":\"cffi-memory-capability-project-nsld-roundtrip\""));
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
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "cffi-memory-capability-project-nsld-roundtrip"));
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "cffi-owned-buffer-yir-escape-gate"));
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "cffi-owned-buffer-llvm-native-lowering"));
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "cffi-owned-buffer-nested-helper-native"));
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "nsld-macho-placement-binding-contract"));
    assert!(drift
        .checks
        .iter()
        .any(|check| { check.id == "nsld-macho-placement-binding-three-surface-evidence" }));
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "nsld-elf-amd64-relocation-application-contract"));
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "nsld-elf-amd64-relocation-application-regression"));
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "nsld-elf-amd64-materialization-preview-contract"));
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "nsld-elf-amd64-materialization-preview-regression"));
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "nsld-elf-amd64-patch-application-contract"));
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "nsld-elf-amd64-patch-application-regression"));
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "nsld-elf-amd64-platform-structure-plan-contract"));
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "nsld-elf-amd64-platform-structure-plan-regression"));
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "nsld-elf-amd64-platform-patch-application-contract"));
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "nsld-elf-amd64-platform-patch-application-regression"));
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "scheduler-mutex-yir-contract"));
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "scheduler-mutex-runtime-visibility"));
    assert!(drift
        .checks
        .iter()
        .any(|check| check.id == "scheduler-shared-mutex-llvm-native-lowering"));
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
    assert!(text.contains(
        "milestone_coordinate: self-hosting-roadmap:required:developer-system/dev-tensor/self-hosting-phase-roadmap"
    ));
    assert!(text.contains(
        "cell: architecture=developer-system module=dev-tensor function=self-hosting-phase-roadmap"
    ));
    assert!(text.contains("nuis-self-hosting-phase-roadmap-v1"));
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
    assert!(
        text.contains("weakest_bootstrap_task_card_source: weakest-bootstrap-status-progress-path")
    );
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
    assert!(text.contains(
        "weakest_bootstrap_task_card_common_ancestor_path: developer-system/bootstrap/differential-reproducibility-gate"
    ));
    assert!(text.contains("weakest_bootstrap_task_card_transition_depth: 0"));
    assert!(text.contains("weakest bootstrap-critical status/progress ordering"));
    assert!(text.contains(
        "cell: architecture=standard-library module=std function=concurrency-task-thread-lock"
    ));
    assert!(text.contains(
        "cell: architecture=native-binary-system module=nuis-runtime function=lifecycle-loader-bootstrap"
    ));
    assert!(text.contains("nuis-runtime-lifecycle-bootstrap-plan-v1"));
    assert!(text.contains(
        "cell: architecture=heterogeneous-runtime module=linux-vulkan function=vulkan-provider-bringup"
    ));
    assert!(text.contains("nuis-vulkan-host-probe-v1"));
    assert!(text.contains("    blocker:"));
    assert!(text.contains("    next_action:"));
    assert!(text.contains("    validation_command:"));
    assert!(text.contains("    expected_artifact:"));
    assert!(text.contains("status_protocol: status=stable rank=4"));
    assert!(text.contains("hierarchy_node: level=root path=nuislang"));
    assert!(text.contains("drift_check: id=frontdoor-final-output-boundary-status"));
    assert!(text.contains("drift_check: id=std-filesystem-light-smoke"));
    assert!(text.contains("drift_check: id=cffi-memory-capability-canonical-hash"));
    assert!(text.contains("drift_check: id=cffi-memory-capability-project-nsld-roundtrip"));
    assert!(text.contains("drift_check: id=cffi-owned-buffer-yir-escape-gate"));
    assert!(text.contains("drift_check: id=cffi-owned-buffer-llvm-native-lowering"));
    assert!(text.contains("drift_check: id=cffi-owned-buffer-nested-helper-native"));
    assert!(text.contains("drift_check: id=nsld-macho-placement-binding-contract"));
    assert!(text.contains("drift_check: id=nsld-macho-placement-binding-three-surface-evidence"));
    assert!(text.contains("drift_check: id=nsld-elf-amd64-placement-binding-contract"));
    assert!(text.contains("drift_check: id=nsld-elf-amd64-placement-binding-regression"));
    assert!(text.contains("drift_check: id=nsld-elf-amd64-relocation-application-contract"));
    assert!(text.contains("drift_check: id=nsld-elf-amd64-relocation-application-regression"));
    assert!(text.contains("drift_check: id=nsld-elf-amd64-materialization-preview-contract"));
    assert!(text.contains("drift_check: id=nsld-elf-amd64-materialization-preview-regression"));
    assert!(text.contains("drift_check: id=nsld-elf-amd64-patch-application-contract"));
    assert!(text.contains("drift_check: id=nsld-elf-amd64-patch-application-regression"));
    assert!(text.contains("drift_check: id=nsld-elf-amd64-platform-structure-plan-contract"));
    assert!(text.contains("drift_check: id=nsld-elf-amd64-platform-structure-plan-regression"));
    assert!(text.contains("drift_check: id=nsld-elf-amd64-platform-patch-application-contract"));
    assert!(text.contains("drift_check: id=nsld-elf-amd64-platform-patch-application-regression"));
    assert!(text.contains("drift_check: id=scheduler-mutex-yir-contract"));
    assert!(text.contains("drift_check: id=scheduler-mutex-runtime-visibility"));
    assert!(text.contains("drift_check: id=scheduler-shared-mutex-llvm-native-lowering"));
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
