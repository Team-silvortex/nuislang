use crate::{
    json_bool_field, json_field, json_string_array_field, json_usize_field,
    surface_render::append_json_field_strings,
};
use std::{collections::BTreeSet, fs, path::PathBuf};

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
}

const DEV_TENSOR_CELLS: &[DevTensorCell] = &[
    DevTensorCell {
        architecture: "compiler-frontdoor",
        module: "nuis-cli",
        function: "workflow-and-project-orientation",
        status: "stable",
        progress: 86,
        bootstrap_critical: true,
        closure_role: "developer-frontdoor",
        evidence: "workflow/project-status/project-doctor JSON and text frontdoors are regression-backed",
        next_step: "compress repeated status fields into a higher-level closure summary",
    },
    DevTensorCell {
        architecture: "compiler-frontdoor",
        module: "nuis-cli",
        function: "artifact-runtime-closure",
        status: "active",
        progress: 74,
        bootstrap_critical: true,
        closure_role: "artifact-execution-frontdoor",
        evidence: "run-artifact, artifact-doctor, release-check, and nsld handoff share final-output boundary fields",
        next_step: "add one consolidated host-runnable vs nsld-owned closure status",
    },
    DevTensorCell {
        architecture: "linker-toolchain",
        module: "nsld",
        function: "artifact-chain-drive",
        status: "stable",
        progress: 82,
        bootstrap_critical: true,
        closure_role: "linker-artifact-chain",
        evidence: "nsld drive can apply whitelisted artifact-chain steps and stop cleanly on host-assisted final output boundaries",
        next_step: "keep drive mutating actions separate from read-only final-output inspection",
    },
    DevTensorCell {
        architecture: "linker-toolchain",
        module: "nsld",
        function: "final-output-boundary",
        status: "active",
        progress: 68,
        bootstrap_critical: true,
        closure_role: "executable-output-boundary",
        evidence: "final-executable-output reports normalized boundary status, path presence, ownership, runnable candidate, blockers, and Nuis/Nsld mirrors",
        next_step: "complete host-shell and OS-native materialization beyond the self-contained Nsld-owned image path",
    },
    DevTensorCell {
        architecture: "heterogeneous-runtime",
        module: "nustar",
        function: "registered-domain-contracts",
        status: "active",
        progress: 69,
        bootstrap_critical: true,
        closure_role: "heterogeneous-domain-registration",
        evidence: "domain units, lowering targets, backend families, contract drift checks, and heterogeneous domain readiness are visible in build/link reports",
        next_step: "connect shader/kernel/network execution-specific readiness to registered Nustar domain contracts without hardcoding nsld domain logic",
    },
    DevTensorCell {
        architecture: "standard-library",
        module: "std",
        function: "host-io-filesystem-text",
        status: "usable",
        progress: 67,
        bootstrap_critical: true,
        closure_role: "bootstrap-std-foundation",
        evidence: "std IO/filesystem/text examples and smoke tests can build and run through the current host path",
        next_step: "separate minimal stable std surface from experimental hetero/runtime layers",
    },
    DevTensorCell {
        architecture: "standard-library",
        module: "pixelmagic",
        function: "image-processing-lane",
        status: "early",
        progress: 34,
        bootstrap_critical: false,
        closure_role: "official-compute-package",
        evidence: "PixelMagic exists as an official package lane but still depends on stronger shader/kernel lowering",
        next_step: "add shader-backed image kernels once heterogeneous final-output closure is clearer",
    },
    DevTensorCell {
        architecture: "standard-library",
        module: "witsage",
        function: "classical-ml-lane",
        status: "early",
        progress: 28,
        bootstrap_critical: false,
        closure_role: "official-compute-package",
        evidence: "WitSage is named and staged as an official package lane but not yet a mature compute workload",
        next_step: "anchor first matrix/vector workloads to kernel/shader domain tests",
    },
    DevTensorCell {
        architecture: "language-core",
        module: "nuisc",
        function: "type-control-flow-generics",
        status: "active",
        progress: 72,
        bootstrap_critical: true,
        closure_role: "language-self-hosting-foundation",
        evidence: "control flow, generics, traits, pointers, floats, enums, error handling, and inference have staged coverage",
        next_step: "bind language feature maturity to bootstrap-critical examples rather than isolated syntax tests",
    },
    DevTensorCell {
        architecture: "native-binary-system",
        module: "nsb-nsld",
        function: "self-owned-binary-assembly",
        status: "active",
        progress: 67,
        bootstrap_critical: true,
        closure_role: "self-owned-native-binary",
        evidence: "Nsld container, object/image dry-runs, final executable pipeline, self-contained NSB image emission, launcher dry-run checks, and Nuis self-owned image status are visible",
        next_step: "bridge self-contained NSB image output toward host-shell and OS-native entrypoint materialization",
    },
    DevTensorCell {
        architecture: "developer-system",
        module: "dev-tensor",
        function: "architecture-module-function-progress-model",
        status: "active",
        progress: 70,
        bootstrap_critical: true,
        closure_role: "bootstrap-progress-model",
        evidence: "tensor frontdoor maps progress by architecture, module, and function, summarizes through nuis status, and runs drift checks over key docs/tests/frontdoor fields",
        next_step: "expand drift checks from field anchors to milestone-owned test and example evidence",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DevTensorDriftCheckSpec {
    id: &'static str,
    path: &'static str,
    required_patterns: &'static [&'static str],
}

const DEV_TENSOR_DRIFT_CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "frontdoor-final-output-boundary-status",
        path: "tools/nuis/src/workflow/link_plan.rs",
        required_patterns: &[
            "nsld_final_executable_output_boundary_status",
            "nsld_final_executable_output_ready",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "frontdoor-self-owned-image-status",
        path: "tools/nuis/src/workflow/link_plan.rs",
        required_patterns: &[
            "nsld_self_owned_image_status",
            "nsld_self_owned_image_header_valid",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "workflow-surface-json-regression",
        path: "tools/nuis/src/main_tests/workflow_surface.rs",
        required_patterns: &[
            "nsld_final_executable_output_boundary_status",
            "nsld_self_owned_image_status",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "artifact-runtime-json-regression",
        path: "tools/nuis/src/main_tests/artifact_runtime.rs",
        required_patterns: &[
            "nsld_final_executable_output_boundary_status",
            "nsld_self_owned_image_status",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "frontdoor-reference-doc",
        path: "docs/reference/nuis-frontdoor-surface-reference.md",
        required_patterns: &[
            "nsld_final_executable_output_boundary_status",
            "nsld_self_owned_image_status",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "native-artifact-workflow-doc",
        path: "docs/reference/nuis-native-artifact-workflow.md",
        required_patterns: &[
            "nsld_final_executable_output_boundary_status",
            "nsld_self_owned_image_status",
        ],
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DevTensorDriftCheck {
    pub(crate) id: &'static str,
    pub(crate) path: &'static str,
    pub(crate) passed: bool,
    pub(crate) missing_patterns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DevTensorDriftSummary {
    pub(crate) check_count: usize,
    pub(crate) passed_count: usize,
    pub(crate) failed_count: usize,
    pub(crate) status: &'static str,
    pub(crate) first_failed_check: Option<&'static str>,
    pub(crate) checks: Vec<DevTensorDriftCheck>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub(crate) weakest_bootstrap_progress: usize,
}

pub(crate) fn dev_tensor_summary() -> DevTensorSummary {
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
        weakest_bootstrap_progress: weakest_bootstrap.map_or(0, |cell| cell.progress),
    }
}

pub(crate) fn dev_tensor_drift_summary() -> DevTensorDriftSummary {
    let checks = DEV_TENSOR_DRIFT_CHECKS
        .iter()
        .map(run_dev_tensor_drift_check)
        .collect::<Vec<_>>();
    let check_count = checks.len();
    let passed_count = checks.iter().filter(|check| check.passed).count();
    let failed_count = check_count.saturating_sub(passed_count);
    let first_failed_check = checks
        .iter()
        .find(|check| !check.passed)
        .map(|check| check.id);
    DevTensorDriftSummary {
        check_count,
        passed_count,
        failed_count,
        status: if failed_count == 0 { "clean" } else { "drift" },
        first_failed_check,
        checks,
    }
}

fn run_dev_tensor_drift_check(spec: &DevTensorDriftCheckSpec) -> DevTensorDriftCheck {
    let path = repo_root().join(spec.path);
    let source = fs::read_to_string(path).unwrap_or_default();
    let missing_patterns = spec
        .required_patterns
        .iter()
        .filter(|pattern| !source.contains(**pattern))
        .map(|pattern| (*pattern).to_owned())
        .collect::<Vec<_>>();
    DevTensorDriftCheck {
        id: spec.id,
        path: spec.path,
        passed: missing_patterns.is_empty(),
        missing_patterns,
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub(crate) fn render_dev_tensor_json() -> String {
    let summary = dev_tensor_summary();
    let drift = dev_tensor_drift_summary();
    let cells = DEV_TENSOR_CELLS
        .iter()
        .map(dev_tensor_cell_json)
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
            json_usize_field("axis_count", 3),
            json_field("axis_0", "architecture"),
            json_field("axis_1", "module"),
            json_field("axis_2", "function"),
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
            json_usize_field(
                "weakest_bootstrap_progress",
                summary.weakest_bootstrap_progress,
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
            format!("\"cells\":[{}]", cells.join(",")),
        ],
    );
    out.push('}');
    out
}

pub(crate) fn render_dev_tensor_text() -> Vec<String> {
    let summary = dev_tensor_summary();
    let drift = dev_tensor_drift_summary();
    let mut lines = vec![
        "nuis development tensor".to_owned(),
        "  model: architecture-module-function-progress-tensor".to_owned(),
        "  version: dev-tensor-v1".to_owned(),
        "  axes: architecture, module, function".to_owned(),
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
            "  weakest_bootstrap_progress: {}",
            summary.weakest_bootstrap_progress
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
        ]
        .join(",")
    )
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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(summary.weakest_bootstrap_progress > 0);
        assert!(summary.weakest_bootstrap_progress <= summary.bootstrap_critical_average_progress);
    }

    #[test]
    fn dev_tensor_json_exposes_coordinate_cells() {
        let json = render_dev_tensor_json();
        assert!(json.contains("\"kind\":\"nuis_dev_tensor\""));
        assert!(json.contains("\"axis_0\":\"architecture\""));
        assert!(json.contains("\"axis_1\":\"module\""));
        assert!(json.contains("\"axis_2\":\"function\""));
        assert!(json.contains("\"coordinates\":["));
        assert!(json.contains("\"bootstrap_critical\":true"));
        assert!(json.contains("\"closure_role\":\"self-owned-native-binary\""));
        assert!(json.contains("\"weakest_bootstrap_architecture\""));
        assert!(json.contains("\"weakest_bootstrap_module\""));
        assert!(json.contains("\"weakest_bootstrap_function\""));
        assert!(json.contains("\"module\":\"nsld\""));
        assert!(json.contains("\"function\":\"final-output-boundary\""));
        assert!(json.contains("\"drift_status\":\"clean\""));
        assert!(json.contains("\"drift_checks\":["));
        assert!(json.contains("\"id\":\"frontdoor-self-owned-image-status\""));
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
    }

    #[test]
    fn dev_tensor_text_exposes_drift_status() {
        let text = render_dev_tensor_text().join("\n");
        assert!(text.contains("drift_status: clean"));
        assert!(text.contains("drift_check: id=frontdoor-final-output-boundary-status"));
        assert!(text.contains("drift_first_failed_check: <none>"));
    }
}
