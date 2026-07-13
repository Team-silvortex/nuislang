use crate::{
    json_bool_field, json_field, json_string_array_field, json_usize_field,
    surface_render::append_json_field_strings,
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
        progress: 61,
        bootstrap_critical: true,
        closure_role: "executable-output-boundary",
        evidence: "final-executable-output reports path presence, ownership, runnable candidate, blockers, and check/artifact-chain mirrors",
        next_step: "complete a self-owned final executable path beyond host-assisted outputs",
    },
    DevTensorCell {
        architecture: "heterogeneous-runtime",
        module: "nustar",
        function: "registered-domain-contracts",
        status: "active",
        progress: 58,
        bootstrap_critical: true,
        closure_role: "heterogeneous-domain-registration",
        evidence: "domain units, lowering targets, backend families, and sidecar capability checks are visible in build/link reports",
        next_step: "make shader/kernel/network domain readiness comparable through the same progress cells",
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
        status: "blocked",
        progress: 43,
        bootstrap_critical: true,
        closure_role: "self-owned-native-binary",
        evidence: "Nsld container, object/image dry-runs, final executable pipeline, and binary protocol docs exist",
        next_step: "turn dry-run/image/container protocol into a real self-owned executable output path",
    },
    DevTensorCell {
        architecture: "developer-system",
        module: "dev-tensor",
        function: "architecture-module-function-progress-model",
        status: "early-usable",
        progress: 45,
        bootstrap_critical: true,
        closure_role: "bootstrap-progress-model",
        evidence: "tensor frontdoor maps progress by architecture, module, and function and is summarized by nuis status",
        next_step: "wire tensor cells to tests, docs, and frontdoor outputs for automatic drift checks",
    },
];

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

pub(crate) fn render_dev_tensor_json() -> String {
    let summary = dev_tensor_summary();
    let cells = DEV_TENSOR_CELLS
        .iter()
        .map(dev_tensor_cell_json)
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
            format!("\"cells\":[{}]", cells.join(",")),
        ],
    );
    out.push('}');
    out
}

pub(crate) fn render_dev_tensor_text() -> Vec<String> {
    let summary = dev_tensor_summary();
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
    ];
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
        assert_eq!(
            summary.weakest_bootstrap_architecture,
            "native-binary-system"
        );
        assert_eq!(summary.weakest_bootstrap_module, "nsb-nsld");
        assert_eq!(
            summary.weakest_bootstrap_function,
            "self-owned-binary-assembly"
        );
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
    }
}
