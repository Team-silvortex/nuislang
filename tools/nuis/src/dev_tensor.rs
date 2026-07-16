use crate::{
    dev_tensor_data::DEV_TENSOR_CELLS,
    dev_tensor_drift::{
        dev_tensor_drift_summary as build_dev_tensor_drift_summary, DevTensorDriftSummary,
    },
    dev_tensor_drift_data::dev_tensor_drift_checks,
    dev_tensor_manifest::{dev_tensor_manifest_coverage, DevTensorManifestCoverage},
    dev_tensor_milestones::{
        dev_tensor_milestone_coverage, expected_coordinates_from_milestones,
        DevTensorMilestoneCoverage,
    },
    dev_tensor_status::dev_tensor_status_rank,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DevTensorCell {
    pub(crate) architecture: &'static str,
    pub(crate) module: &'static str,
    pub(crate) function: &'static str,
    pub(crate) status: &'static str,
    pub(crate) progress: usize,
    pub(crate) bootstrap_critical: bool,
    pub(crate) closure_role: &'static str,
    pub(crate) evidence: &'static str,
    pub(crate) next_step: &'static str,
    pub(crate) blocker: &'static str,
    pub(crate) next_action: &'static str,
    pub(crate) validation_command: &'static str,
    pub(crate) expected_artifact: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DevTensorExpectedCoordinate {
    pub(crate) architecture: &'static str,
    pub(crate) module: &'static str,
    pub(crate) function: &'static str,
    pub(crate) milestone: &'static str,
    pub(crate) required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DevTensorCoverageSummary {
    pub(crate) expected_source: &'static str,
    pub(crate) expected_fallback_used: bool,
    pub(crate) expected_source_error: Option<String>,
    pub(crate) expected_count: usize,
    pub(crate) covered_count: usize,
    pub(crate) missing_count: usize,
    pub(crate) required_missing_count: usize,
    pub(crate) orphaned_count: usize,
    pub(crate) stale_count: usize,
    pub(crate) status: &'static str,
    pub(crate) first_gap: Option<String>,
    pub(crate) missing_coordinates: Vec<String>,
    pub(crate) orphaned_coordinates: Vec<String>,
    pub(crate) stale_coordinates: Vec<String>,
    pub(crate) manifest: DevTensorManifestCoverage,
    pub(crate) milestone: DevTensorMilestoneCoverage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DevTensorSummary {
    pub(crate) architecture_count: usize,
    pub(crate) module_count: usize,
    pub(crate) function_count: usize,
    pub(crate) cell_count: usize,
    pub(crate) average_progress: usize,
    pub(crate) bootstrap_critical_count: usize,
    pub(crate) bootstrap_critical_average_progress: usize,
    pub(crate) weakest_bootstrap_architecture: &'static str,
    pub(crate) weakest_bootstrap_module: &'static str,
    pub(crate) weakest_bootstrap_function: &'static str,
    pub(crate) weakest_bootstrap_status: &'static str,
    pub(crate) weakest_bootstrap_progress: usize,
    pub(crate) weakest_bootstrap_closure_role: &'static str,
    pub(crate) weakest_bootstrap_evidence: &'static str,
    pub(crate) weakest_bootstrap_next_step: &'static str,
    pub(crate) weakest_bootstrap_blocker: &'static str,
    pub(crate) weakest_bootstrap_next_action: &'static str,
    pub(crate) weakest_bootstrap_validation_command: &'static str,
    pub(crate) weakest_bootstrap_expected_artifact: &'static str,
    pub(crate) weakest_bootstrap_task_card_coordinate: String,
    pub(crate) weakest_bootstrap_task_card_priority_reason: String,
    pub(crate) weakest_bootstrap_task_card_action: &'static str,
    pub(crate) weakest_bootstrap_task_card_command: &'static str,
    pub(crate) weakest_bootstrap_task_card_expected_artifact: &'static str,
    pub(crate) coverage_status: &'static str,
    pub(crate) coverage_expected_count: usize,
    pub(crate) coverage_covered_count: usize,
    pub(crate) coverage_missing_count: usize,
    pub(crate) coverage_orphaned_count: usize,
    pub(crate) coverage_stale_count: usize,
}

pub(crate) fn dev_tensor_summary() -> DevTensorSummary {
    let coverage = dev_tensor_coverage_summary();
    let mut architectures = BTreeSet::new();
    let mut modules = BTreeSet::new();
    let mut functions = BTreeSet::new();
    let mut total_progress = 0usize;
    let mut critical_progress = 0usize;
    let mut critical_count = 0usize;
    let mut weakest_bootstrap = None::<&DevTensorCell>;
    for cell in DEV_TENSOR_CELLS {
        architectures.insert(cell.architecture);
        modules.insert(cell.module);
        functions.insert(cell.function);
        total_progress += cell.progress;
        if cell.bootstrap_critical {
            critical_count += 1;
            critical_progress += cell.progress;
            if weakest_bootstrap
                .map(|weakest| cell.progress < weakest.progress)
                .unwrap_or(true)
            {
                weakest_bootstrap = Some(cell);
            }
        }
    }
    let cell_count = DEV_TENSOR_CELLS.len();
    let task_card_coordinate = weakest_bootstrap
        .map(|cell| dev_tensor_coordinate_key(cell.architecture, cell.module, cell.function))
        .unwrap_or_else(|| "<none>".to_owned());
    let task_card_priority_reason = weakest_bootstrap
        .map(|cell| {
            format!(
                "lowest bootstrap-critical progress: {}/100 at {}",
                cell.progress, task_card_coordinate
            )
        })
        .unwrap_or_else(|| "no bootstrap-critical tensor cell is currently registered".to_owned());
    DevTensorSummary {
        architecture_count: architectures.len(),
        module_count: modules.len(),
        function_count: functions.len(),
        cell_count,
        average_progress: if cell_count == 0 {
            0
        } else {
            total_progress / cell_count
        },
        bootstrap_critical_count: critical_count,
        bootstrap_critical_average_progress: if critical_count == 0 {
            0
        } else {
            critical_progress / critical_count
        },
        weakest_bootstrap_architecture: weakest_bootstrap
            .map(|cell| cell.architecture)
            .unwrap_or("<none>"),
        weakest_bootstrap_module: weakest_bootstrap
            .map(|cell| cell.module)
            .unwrap_or("<none>"),
        weakest_bootstrap_function: weakest_bootstrap
            .map(|cell| cell.function)
            .unwrap_or("<none>"),
        weakest_bootstrap_status: weakest_bootstrap
            .map(|cell| cell.status)
            .unwrap_or("<none>"),
        weakest_bootstrap_progress: weakest_bootstrap.map_or(0, |cell| cell.progress),
        weakest_bootstrap_closure_role: weakest_bootstrap
            .map(|cell| cell.closure_role)
            .unwrap_or("<none>"),
        weakest_bootstrap_evidence: weakest_bootstrap
            .map(|cell| cell.evidence)
            .unwrap_or("<none>"),
        weakest_bootstrap_next_step: weakest_bootstrap
            .map(|cell| cell.next_step)
            .unwrap_or("<none>"),
        weakest_bootstrap_blocker: weakest_bootstrap
            .map(|cell| cell.blocker)
            .unwrap_or("<none>"),
        weakest_bootstrap_next_action: weakest_bootstrap
            .map(|cell| cell.next_action)
            .unwrap_or("<none>"),
        weakest_bootstrap_validation_command: weakest_bootstrap
            .map(|cell| cell.validation_command)
            .unwrap_or("<none>"),
        weakest_bootstrap_expected_artifact: weakest_bootstrap
            .map(|cell| cell.expected_artifact)
            .unwrap_or("<none>"),
        weakest_bootstrap_task_card_coordinate: task_card_coordinate,
        weakest_bootstrap_task_card_priority_reason: task_card_priority_reason,
        weakest_bootstrap_task_card_action: weakest_bootstrap
            .map(|cell| cell.next_action)
            .unwrap_or("<none>"),
        weakest_bootstrap_task_card_command: weakest_bootstrap
            .map(|cell| cell.validation_command)
            .unwrap_or("<none>"),
        weakest_bootstrap_task_card_expected_artifact: weakest_bootstrap
            .map(|cell| cell.expected_artifact)
            .unwrap_or("<none>"),
        coverage_status: coverage.status,
        coverage_expected_count: coverage.expected_count,
        coverage_covered_count: coverage.covered_count,
        coverage_missing_count: coverage.missing_count,
        coverage_orphaned_count: coverage.orphaned_count,
        coverage_stale_count: coverage.stale_count,
    }
}

pub(crate) fn dev_tensor_coverage_summary() -> DevTensorCoverageSummary {
    let manifest = dev_tensor_manifest_coverage();
    let milestone = dev_tensor_milestone_coverage();
    let expected = expected_coordinates_from_milestones();
    let cell_coordinates = DEV_TENSOR_CELLS
        .iter()
        .map(|cell| dev_tensor_coordinate_key(cell.architecture, cell.module, cell.function))
        .collect::<BTreeSet<_>>();
    let expected_coordinates = expected
        .coordinates
        .iter()
        .map(|coordinate| {
            dev_tensor_coordinate_key(
                &coordinate.architecture,
                &coordinate.module,
                &coordinate.function,
            )
        })
        .collect::<BTreeSet<_>>();
    let missing_coordinates = expected
        .coordinates
        .iter()
        .filter_map(|coordinate| {
            let key = dev_tensor_coordinate_key(
                &coordinate.architecture,
                &coordinate.module,
                &coordinate.function,
            );
            (!cell_coordinates.contains(&key)).then(|| {
                format!(
                    "{}{}",
                    key,
                    if coordinate.required {
                        ":required"
                    } else {
                        ":optional"
                    }
                )
            })
        })
        .collect::<Vec<_>>();
    let required_missing_count = missing_coordinates
        .iter()
        .filter(|coordinate| coordinate.ends_with(":required"))
        .count();
    let orphaned_coordinates = DEV_TENSOR_CELLS
        .iter()
        .filter_map(|cell| {
            let key = dev_tensor_coordinate_key(cell.architecture, cell.module, cell.function);
            (!expected_coordinates.contains(&key)).then_some(key)
        })
        .collect::<Vec<_>>();
    let stale_coordinates = DEV_TENSOR_CELLS
        .iter()
        .filter_map(|cell| {
            let stale = cell.status.is_empty()
                || dev_tensor_status_rank(cell.status) == 0
                || cell.closure_role.is_empty()
                || cell.evidence.is_empty()
                || cell.next_step.is_empty()
                || cell.blocker.is_empty()
                || cell.next_action.is_empty()
                || cell.validation_command.is_empty()
                || cell.expected_artifact.is_empty()
                || cell.progress > 100;
            stale.then(|| dev_tensor_coordinate_key(cell.architecture, cell.module, cell.function))
        })
        .collect::<Vec<_>>();
    let covered_count = expected
        .coordinates
        .len()
        .saturating_sub(missing_coordinates.len());
    let status = if required_missing_count == 0
        && orphaned_coordinates.is_empty()
        && stale_coordinates.is_empty()
        && milestone.status == "clean"
    {
        "clean"
    } else {
        "gap"
    };
    let first_gap = missing_coordinates
        .first()
        .or_else(|| orphaned_coordinates.first())
        .or_else(|| stale_coordinates.first())
        .or_else(|| milestone.first_gap.as_ref())
        .cloned();
    DevTensorCoverageSummary {
        expected_source: expected.source,
        expected_fallback_used: expected.fallback_used,
        expected_source_error: expected.error,
        expected_count: expected.coordinates.len(),
        covered_count,
        missing_count: missing_coordinates.len(),
        required_missing_count,
        orphaned_count: orphaned_coordinates.len(),
        stale_count: stale_coordinates.len(),
        status,
        first_gap,
        missing_coordinates,
        orphaned_coordinates,
        stale_coordinates,
        manifest,
        milestone,
    }
}

pub(crate) fn dev_tensor_coordinate_key(
    architecture: &str,
    module: &str,
    function: &str,
) -> String {
    format!("{architecture}/{module}/{function}")
}

pub(crate) fn dev_tensor_drift_summary() -> DevTensorDriftSummary {
    build_dev_tensor_drift_summary(dev_tensor_drift_checks())
}

pub(crate) fn render_dev_tensor_json() -> String {
    super::dev_tensor_render::render_dev_tensor_json_impl()
}

pub(crate) fn render_dev_tensor_text() -> Vec<String> {
    super::dev_tensor_render::render_dev_tensor_text_impl()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dev_tensor_data::DEV_TENSOR_EXPECTED_COORDINATES;

    #[test]
    fn dev_tensor_summary_reports_three_axes_and_cells() {
        let summary = dev_tensor_summary();
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
        assert_ne!(summary.weakest_bootstrap_task_card_coordinate, "<none>");
        assert!(summary.weakest_bootstrap_task_card_coordinate.contains('/'));
        assert!(summary
            .weakest_bootstrap_task_card_priority_reason
            .contains("lowest bootstrap-critical progress"));
        assert_eq!(
            summary.weakest_bootstrap_task_card_action,
            summary.weakest_bootstrap_next_action
        );
        assert_eq!(
            summary.weakest_bootstrap_task_card_command,
            summary.weakest_bootstrap_validation_command
        );
        assert_eq!(
            summary.weakest_bootstrap_task_card_expected_artifact,
            summary.weakest_bootstrap_expected_artifact
        );
        assert!(summary.weakest_bootstrap_progress <= summary.bootstrap_critical_average_progress);
        let hierarchy = crate::dev_tensor_hierarchy::dev_tensor_hierarchy_summary();
        assert_eq!(hierarchy.protocol_version, "dev-tensor-status-v1");
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
        assert!(json.contains("\"weakest_bootstrap_task_card_coordinate\""));
        assert!(json.contains("\"weakest_bootstrap_task_card_priority_reason\""));
        assert!(json.contains("\"weakest_bootstrap_task_card_action\""));
        assert!(json.contains("\"weakest_bootstrap_task_card_command\""));
        assert!(json.contains("\"weakest_bootstrap_task_card_expected_artifact\""));
        assert!(json.contains("lowest bootstrap-critical progress"));
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
        assert!(text
            .contains("manifest_backed_coordinate: standard-library/std/host-io-filesystem-text"));
        assert!(text.contains("manifest_untracked_module: core"));
        assert!(text.contains("milestone_coverage_status: clean"));
        assert!(text.contains(
            "milestone_coverage_source: docs/reference/nuis-development-tensor.milestones.toml"
        ));
        assert!(text.contains(
            "milestone_derived_cache_protocol: nuis-dev-tensor-derived-coordinate-cache-v1"
        ));
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
        assert!(text.contains("weakest_bootstrap_next_step:"));
        assert!(text.contains("weakest_bootstrap_evidence:"));
        assert!(text.contains("weakest_bootstrap_blocker:"));
        assert!(text.contains("weakest_bootstrap_next_action:"));
        assert!(text.contains("weakest_bootstrap_validation_command:"));
        assert!(text.contains("weakest_bootstrap_expected_artifact:"));
        assert!(text.contains("weakest_bootstrap_task_card_coordinate:"));
        assert!(text.contains("weakest_bootstrap_task_card_priority_reason:"));
        assert!(text.contains("weakest_bootstrap_task_card_action:"));
        assert!(text.contains("weakest_bootstrap_task_card_command:"));
        assert!(text.contains("weakest_bootstrap_task_card_expected_artifact:"));
        assert!(text.contains("lowest bootstrap-critical progress"));
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
}
