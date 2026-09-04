use crate::{
    dev_tensor_data::DEV_TENSOR_CELLS,
    dev_tensor_drift::{
        dev_tensor_drift_summary as build_dev_tensor_drift_summary, DevTensorDriftSummary,
    },
    dev_tensor_drift_data::dev_tensor_drift_checks,
    dev_tensor_hierarchy::dev_tensor_hierarchy_summary,
    dev_tensor_manifest::{dev_tensor_manifest_coverage, DevTensorManifestCoverage},
    dev_tensor_milestones::{
        dev_tensor_milestone_coverage, expected_coordinates_from_milestones,
        DevTensorMilestoneCoverage,
    },
    dev_tensor_status::dev_tensor_status_rank,
    dev_tensor_task_card_lineage::{
        validate_dev_tensor_task_card_lineage, DevTensorTaskCardLineage,
    },
};
use std::collections::BTreeSet;

pub(crate) const DEV_TENSOR_TASK_CARD_PROTOCOL: &str = "nuis-dev-tensor-task-card-v1";

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
    pub(crate) hierarchy_protocol_version: &'static str,
    pub(crate) hierarchy_validation_status: &'static str,
    pub(crate) hierarchy_validation_node_count: usize,
    pub(crate) hierarchy_validation_max_depth: usize,
    pub(crate) hierarchy_validation_error_count: usize,
    pub(crate) hierarchy_validation_first_error: String,
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
    pub(crate) weakest_bootstrap_task_card_protocol: &'static str,
    pub(crate) weakest_bootstrap_task_card_source: &'static str,
    pub(crate) weakest_bootstrap_task_card_status: &'static str,
    pub(crate) weakest_bootstrap_task_card_ready: bool,
    pub(crate) weakest_bootstrap_task_card_coordinate: String,
    pub(crate) weakest_bootstrap_task_card_priority_reason: String,
    pub(crate) weakest_bootstrap_task_card_action: &'static str,
    pub(crate) weakest_bootstrap_task_card_command: &'static str,
    pub(crate) weakest_bootstrap_task_card_expected_artifact: &'static str,
    pub(crate) weakest_bootstrap_task_card_handoff_mode: &'static str,
    pub(crate) weakest_bootstrap_task_card_handoff_coordinate: String,
    pub(crate) weakest_bootstrap_task_card_handoff_reason: String,
    pub(crate) weakest_bootstrap_task_card_handoff_action: &'static str,
    pub(crate) weakest_bootstrap_task_card_handoff_command: &'static str,
    pub(crate) weakest_bootstrap_task_card_handoff_expected_artifact: &'static str,
    pub(crate) weakest_bootstrap_task_card_lineage: DevTensorTaskCardLineage,
    pub(crate) coverage_status: &'static str,
    pub(crate) coverage_expected_count: usize,
    pub(crate) coverage_covered_count: usize,
    pub(crate) coverage_missing_count: usize,
    pub(crate) coverage_orphaned_count: usize,
    pub(crate) coverage_stale_count: usize,
}

pub(crate) fn dev_tensor_summary() -> DevTensorSummary {
    let coverage = dev_tensor_coverage_summary();
    let hierarchy = dev_tensor_hierarchy_summary();
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
                .map(|weakest| {
                    dev_tensor_cell_weakness_key(cell) < dev_tensor_cell_weakness_key(weakest)
                })
                .unwrap_or(true)
            {
                weakest_bootstrap = Some(cell);
            }
        }
    }
    let cell_count = DEV_TENSOR_CELLS.len();
    let bootstrap_closed = dev_tensor_bootstrap_cells_closed(DEV_TENSOR_CELLS);
    let task_cell = select_dev_tensor_task_cell(DEV_TENSOR_CELLS);
    let task_card_coordinate = task_cell
        .map(|cell| dev_tensor_coordinate_key(cell.architecture, cell.module, cell.function))
        .unwrap_or_else(|| "<none>".to_owned());
    let task_card_source = if bootstrap_closed && task_cell.is_none() {
        "all-cells-complete"
    } else if bootstrap_closed {
        "weakest-global-incomplete-status-progress-path"
    } else {
        "weakest-bootstrap-status-progress-path"
    };
    let task_card_priority_reason = task_cell
        .map(|cell| {
            if bootstrap_closed {
                format!(
                    "all bootstrap-critical cells are stable at 100/100; weakest global incomplete status/progress ordering: status `{}` rank {}, progress {}/100 at {}",
                    cell.status,
                    dev_tensor_status_rank(cell.status),
                    cell.progress,
                    task_card_coordinate
                )
            } else {
                format!(
                    "weakest bootstrap-critical status/progress ordering: status `{}` rank {}, progress {}/100 at {}",
                    cell.status,
                    dev_tensor_status_rank(cell.status),
                    cell.progress,
                    task_card_coordinate
                )
            }
        })
        .unwrap_or_else(|| {
            if bootstrap_closed {
                "all registered tensor cells are stable at 100/100".to_owned()
            } else {
                "no bootstrap-critical tensor cell is currently registered".to_owned()
            }
        });
    let handoff_bootstrap = task_cell.and_then(dev_tensor_handoff_bootstrap_cell);
    let handoff_coordinate = handoff_bootstrap
        .map(|cell| dev_tensor_coordinate_key(cell.architecture, cell.module, cell.function))
        .unwrap_or_else(|| task_card_coordinate.clone());
    let handoff_mode = if handoff_bootstrap.is_some() {
        "self-maintenance-handoff"
    } else {
        "direct"
    };
    let task_card_lineage = validate_dev_tensor_task_card_lineage(
        &hierarchy.root,
        hierarchy.validation.status,
        &task_card_coordinate,
        &handoff_coordinate,
        handoff_mode,
    );
    let task_card_ready = task_cell.is_some()
        && coverage.status == "clean"
        && hierarchy.validation.status == "clean"
        && task_card_lineage.status == "clean";
    let handoff_reason = handoff_bootstrap
        .map(|cell| {
            format!(
                "weakest coordinate is the dev tensor itself; after refreshing the tensor, continue at {} with status `{}` rank {} and {}/100 progress",
                handoff_coordinate,
                cell.status,
                dev_tensor_status_rank(cell.status),
                cell.progress
            )
        })
        .unwrap_or_else(|| match task_cell {
            Some(_) => format!(
                "weakest task card is directly actionable at {}",
                task_card_coordinate
            ),
            None => "all registered tensor cells are complete; no handoff is required".to_owned(),
        });
    DevTensorSummary {
        hierarchy_protocol_version: hierarchy.hierarchy_protocol_version,
        hierarchy_validation_status: hierarchy.validation.status,
        hierarchy_validation_node_count: hierarchy.validation.node_count,
        hierarchy_validation_max_depth: hierarchy.validation.max_depth,
        hierarchy_validation_error_count: hierarchy.validation.error_count,
        hierarchy_validation_first_error: hierarchy
            .validation
            .first_error
            .unwrap_or_else(|| "<none>".to_owned()),
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
        weakest_bootstrap_task_card_protocol: DEV_TENSOR_TASK_CARD_PROTOCOL,
        weakest_bootstrap_task_card_source: task_card_source,
        weakest_bootstrap_task_card_status: if task_card_ready {
            "ready"
        } else if bootstrap_closed && task_cell.is_none() {
            "complete"
        } else {
            "blocked"
        },
        weakest_bootstrap_task_card_ready: task_card_ready,
        weakest_bootstrap_task_card_coordinate: task_card_coordinate,
        weakest_bootstrap_task_card_priority_reason: task_card_priority_reason,
        weakest_bootstrap_task_card_action: task_cell
            .map(|cell| cell.next_action)
            .unwrap_or("<none>"),
        weakest_bootstrap_task_card_command: task_cell
            .map(|cell| cell.validation_command)
            .unwrap_or("<none>"),
        weakest_bootstrap_task_card_expected_artifact: task_cell
            .map(|cell| cell.expected_artifact)
            .unwrap_or("<none>"),
        weakest_bootstrap_task_card_handoff_mode: handoff_mode,
        weakest_bootstrap_task_card_handoff_coordinate: handoff_coordinate,
        weakest_bootstrap_task_card_handoff_reason: handoff_reason,
        weakest_bootstrap_task_card_handoff_action: handoff_bootstrap
            .or(task_cell)
            .map(|cell| cell.next_action)
            .unwrap_or("<none>"),
        weakest_bootstrap_task_card_handoff_command: handoff_bootstrap
            .or(task_cell)
            .map(|cell| cell.validation_command)
            .unwrap_or("<none>"),
        weakest_bootstrap_task_card_handoff_expected_artifact: handoff_bootstrap
            .or(task_cell)
            .map(|cell| cell.expected_artifact)
            .unwrap_or("<none>"),
        weakest_bootstrap_task_card_lineage: task_card_lineage,
        coverage_status: coverage.status,
        coverage_expected_count: coverage.expected_count,
        coverage_covered_count: coverage.covered_count,
        coverage_missing_count: coverage.missing_count,
        coverage_orphaned_count: coverage.orphaned_count,
        coverage_stale_count: coverage.stale_count,
    }
}

fn dev_tensor_handoff_bootstrap_cell(weakest: &DevTensorCell) -> Option<&'static DevTensorCell> {
    if dev_tensor_coordinate_key(weakest.architecture, weakest.module, weakest.function)
        != "developer-system/dev-tensor/architecture-module-function-progress-model"
    {
        return None;
    }
    select_dev_tensor_handoff_bootstrap_cell(DEV_TENSOR_CELLS)
}

fn dev_tensor_bootstrap_cells_closed(cells: &[DevTensorCell]) -> bool {
    let mut critical = cells.iter().filter(|cell| cell.bootstrap_critical);
    let critical_count = critical.clone().count();
    critical_count > 0 && critical.all(|cell| cell.status == "stable" && cell.progress == 100)
}

fn select_dev_tensor_task_cell(cells: &[DevTensorCell]) -> Option<&DevTensorCell> {
    if dev_tensor_bootstrap_cells_closed(cells) {
        return cells
            .iter()
            .filter(|cell| cell.status != "stable" || cell.progress < 100)
            .min_by_key(|cell| dev_tensor_cell_weakness_key(cell));
    }
    cells
        .iter()
        .filter(|cell| cell.bootstrap_critical)
        .min_by_key(|cell| dev_tensor_cell_weakness_key(cell))
}

fn select_dev_tensor_handoff_bootstrap_cell(cells: &[DevTensorCell]) -> Option<&DevTensorCell> {
    cells
        .iter()
        .filter(|cell| cell.bootstrap_critical)
        .filter(|cell| cell.status != "stable" || cell.progress < 100)
        .filter(|cell| {
            dev_tensor_coordinate_key(cell.architecture, cell.module, cell.function)
                != "developer-system/dev-tensor/architecture-module-function-progress-model"
        })
        .min_by_key(|cell| dev_tensor_cell_weakness_key(cell))
}

fn dev_tensor_cell_weakness_key(cell: &DevTensorCell) -> (usize, usize, String) {
    (
        dev_tensor_status_rank(cell.status),
        cell.progress,
        dev_tensor_coordinate_key(cell.architecture, cell.module, cell.function),
    )
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
        .or(milestone.first_gap.as_ref())
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
#[path = "dev_tensor_tests.rs"]
mod tests;
