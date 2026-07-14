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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DevTensorExpectedCoordinate {
    architecture: &'static str,
    module: &'static str,
    function: &'static str,
    milestone: &'static str,
    required: bool,
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
        progress: 70,
        bootstrap_critical: true,
        closure_role: "executable-output-boundary",
        evidence: "final-executable-output reports normalized boundary status, materialization status, recommended next action, path presence, ownership, runnable candidate, blockers, and Nuis/Nsld mirrors",
        next_step: "complete host-shell and OS-native materialization beyond the self-contained Nsld-owned image path",
    },
    DevTensorCell {
        architecture: "heterogeneous-runtime",
        module: "nustar",
        function: "registered-domain-contracts",
        status: "active",
        progress: 71,
        bootstrap_critical: true,
        closure_role: "heterogeneous-domain-registration",
        evidence: "domain units, lowering targets, backend families, contract completeness status, contract drift checks, and heterogeneous domain readiness are visible in build/link reports",
        next_step: "connect shader/kernel/network execution-specific readiness and dispatch bridge materialization to registered Nustar domain contracts without hardcoding nsld domain logic",
    },
    DevTensorCell {
        architecture: "standard-library",
        module: "std",
        function: "host-io-filesystem-text",
        status: "usable",
        progress: 70,
        bootstrap_critical: true,
        closure_role: "bootstrap-std-foundation",
        evidence: "std IO/filesystem/text examples, std_filesystem_smoke, tooling docs, and std lane docs anchor the current host-backed CLI foundation",
        next_step: "promote the light std smoke chain into milestone-owned CLI closure evidence",
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
        progress: 71,
        bootstrap_critical: true,
        closure_role: "self-owned-native-binary",
        evidence: "Nsld container, object/image dry-runs, final executable pipeline, self-contained NSB image emission, launcher dry-run checks, host entrypoint handoff stub emission, Nsld pipeline self-owned image status, entrypoint materialization status, and Nuis frontdoor consumption are visible",
        next_step: "materialize the host-shell or OS-native entrypoint from the ready self-contained NSB image route",
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

const DEV_TENSOR_EXPECTED_COORDINATES: &[DevTensorExpectedCoordinate] = &[
    DevTensorExpectedCoordinate {
        architecture: "compiler-frontdoor",
        module: "nuis-cli",
        function: "workflow-and-project-orientation",
        milestone: "alpha-frontdoor",
        required: true,
    },
    DevTensorExpectedCoordinate {
        architecture: "compiler-frontdoor",
        module: "nuis-cli",
        function: "artifact-runtime-closure",
        milestone: "alpha-frontdoor",
        required: true,
    },
    DevTensorExpectedCoordinate {
        architecture: "linker-toolchain",
        module: "nsld",
        function: "artifact-chain-drive",
        milestone: "alpha-linker",
        required: true,
    },
    DevTensorExpectedCoordinate {
        architecture: "linker-toolchain",
        module: "nsld",
        function: "final-output-boundary",
        milestone: "alpha-linker",
        required: true,
    },
    DevTensorExpectedCoordinate {
        architecture: "heterogeneous-runtime",
        module: "nustar",
        function: "registered-domain-contracts",
        milestone: "alpha-heterogeneous",
        required: true,
    },
    DevTensorExpectedCoordinate {
        architecture: "standard-library",
        module: "std",
        function: "host-io-filesystem-text",
        milestone: "alpha-stdlib",
        required: true,
    },
    DevTensorExpectedCoordinate {
        architecture: "standard-library",
        module: "pixelmagic",
        function: "image-processing-lane",
        milestone: "alpha-official-galaxy",
        required: false,
    },
    DevTensorExpectedCoordinate {
        architecture: "standard-library",
        module: "witsage",
        function: "classical-ml-lane",
        milestone: "alpha-official-galaxy",
        required: false,
    },
    DevTensorExpectedCoordinate {
        architecture: "language-core",
        module: "nuisc",
        function: "type-control-flow-generics",
        milestone: "alpha-language-core",
        required: true,
    },
    DevTensorExpectedCoordinate {
        architecture: "native-binary-system",
        module: "nsb-nsld",
        function: "self-owned-binary-assembly",
        milestone: "alpha-native-binary",
        required: true,
    },
    DevTensorExpectedCoordinate {
        architecture: "developer-system",
        module: "dev-tensor",
        function: "architecture-module-function-progress-model",
        milestone: "alpha-governance",
        required: true,
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
            "nsld_final_executable_output_materialization_status",
            "nsld_final_executable_output_execution_handoff_contract",
            "nsld_final_executable_output_execution_handoff_ready",
            "nsld_final_executable_output_execution_handoff_status",
            "nsld_final_executable_output_execution_handoff_target",
            "nsld_final_executable_output_execution_handoff_evidence_status",
            "nsld_final_executable_output_execution_handoff_first_blocker",
            "nsld_final_executable_output_execution_handoff_decision_code",
            "nsld_final_executable_output_recommended_next_action",
            "nsld_final_executable_output_ready",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-final-output-materialization-status",
        path: "tools/nsld/src/final_executable_output.rs",
        required_patterns: &[
            "final_executable_output_materialization_status",
            "final_executable_output_execution_handoff_contract",
            "final_executable_output_execution_handoff_ready",
            "final_executable_output_execution_handoff_status",
            "final_executable_output_execution_handoff_target",
            "final_executable_output_execution_handoff_evidence_status",
            "final_executable_output_execution_handoff_first_blocker",
            "final_executable_output_execution_handoff_decision_code",
            "final_executable_output_recommended_next_action",
            "self-contained-image-ready",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-final-output-materialization-regression",
        path: "tools/nsld/src/main_final_executable_output_tests.rs",
        required_patterns: &[
            "materialization_status",
            "execution_handoff_contract",
            "execution_handoff_ready",
            "execution_handoff_status",
            "execution_handoff_target",
            "execution_handoff_evidence_status",
            "execution_handoff_first_blocker",
            "execution_handoff_decision_code",
            "recommended_next_action",
            "self-contained-image-ready",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-host-finalizer-gate-surface",
        path: "tools/nsld/src/json_final_executable.rs",
        required_patterns: &[
            "host_finalizer_gate_status",
            "host_finalizer_gate_action",
            "policy-blocked",
            "explicit-allow-missing",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-host-finalizer-gate-artifact",
        path: "tools/nsld/src/final_executable_render.rs",
        required_patterns: &[
            "host_finalizer_gate_status",
            "host_finalizer_gate_action",
            "set-env:NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-host-finalizer-gate-regression",
        path: "tools/nsld/src/main_final_executable_emit_tests.rs",
        required_patterns: &[
            "host_finalizer_gate_status",
            "policy-blocked",
            "set-env:NUIS_NSLD_HOST_FINALIZER_POLICY=allow-host-invoke",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-check-host-finalizer-gate-surface",
        path: "tools/nsld/src/json_check_final.rs",
        required_patterns: &[
            "final_executable_host_finalizer_gate_status",
            "final_executable_host_finalizer_gate_action",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-nsb-relocation-application-layout",
        path: "tools/nsld/src/final_executable_layout_stage.rs",
        required_patterns: &[
            "relocation_application_strategy",
            "nsb-loader-relocation-table",
            "relocation_application_table_hash",
            "relocation_applications",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-nsb-relocation-application-image",
        path: "tools/nsld/src/final_executable_image_stage.rs",
        required_patterns: &[
            "relocation_application_strategy",
            "relocation_application_count",
            "relocation_application_table_hash",
            "relocation_application_audit_status",
            "relocation_application_audit_blockers",
            "relocation_patch_preview_status",
            "relocation_patch_preview_table_hash",
            "relocation_patch_preview_record_table_hash",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-nsb-relocation-application-regression",
        path: "tools/nsld/src/main_final_executable_layout_tests.rs",
        required_patterns: &[
            "relocation_application_strategy",
            "relocation_application_count",
            "relocation_application_table_hash",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-check-next-action-final-output-boundary",
        path: "tools/nsld/src/check.rs",
        required_patterns: &[
            "artifact_chain_final_output_boundary_ready",
            "final-output-boundary",
            "artifact_chain_final_output_boundary_command_resolved",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-check-next-action-final-output-boundary-regression",
        path: "tools/nsld/src/drive_tests.rs",
        required_patterns: &[
            "drive_until_clean_command_reaches_host_assisted_pipeline_block",
            "final-output-boundary",
            "final-executable-output",
            "blocked-boundary",
            "read-only-boundary:final-executable-output",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-driver-final-output-boundary-doc",
        path: "docs/reference/nsld-driver-frontdoor.md",
        required_patterns: &[
            "final-output-boundary",
            "blocked-boundary",
            "nsld final-executable-output <input>",
            "Host-assisted final executable emission",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "frontdoor-self-owned-image-status",
        path: "tools/nuis/src/workflow/link_plan.rs",
        required_patterns: &[
            "nsld_self_owned_image_status",
            "nsld_entrypoint_materialization_status",
            "pipeline_self_owned_image_status",
            "nsld_self_owned_image_header_valid",
            "nsld_final_executable_pipeline_execution_handoff_contract",
            "nsld_final_executable_pipeline_execution_handoff_ready",
            "nsld_final_executable_pipeline_execution_handoff_status",
            "nsld_final_executable_pipeline_execution_handoff_target",
            "nsld_final_executable_pipeline_execution_handoff_evidence_status",
            "nsld_final_executable_pipeline_execution_handoff_first_blocker",
            "nsld_final_executable_pipeline_execution_handoff_decision_code",
            "nsld_final_executable_pipeline_entrypoint_materialization_kind",
            "nsld_final_executable_pipeline_entrypoint_materialization_path",
            "nsld_final_executable_pipeline_entrypoint_materialization_ready",
            "nsld_final_executable_pipeline_entrypoint_materialization_first_blocker",
            "nsld_final_executable_pipeline_entrypoint_materialization_present",
            "nsld_final_executable_pipeline_entrypoint_materialization_hash",
            "nsld_final_executable_pipeline_entrypoint_materialization_runner_command",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-pipeline-self-owned-image-entrypoint-status",
        path: "tools/nsld/src/final_executable_pipeline.rs",
        required_patterns: &[
            "self_owned_image_status",
            "entrypoint_materialization_status",
            "execution_handoff_contract",
            "execution_handoff_ready",
            "execution_handoff_status",
            "execution_handoff_target",
            "execution_handoff_evidence_status",
            "execution_handoff_first_blocker",
            "execution_handoff_decision_code",
            "nsld_pipeline_self_owned_image_status",
            "nsld_pipeline_entrypoint_materialization_status",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-check-pipeline-handoff-status",
        path: "tools/nsld/src/reports_check.rs",
        required_patterns: &[
            "final_executable_pipeline_execution_handoff_contract",
            "final_executable_pipeline_execution_handoff_ready",
            "final_executable_pipeline_execution_handoff_status",
            "final_executable_pipeline_execution_handoff_target",
            "final_executable_pipeline_execution_handoff_evidence_status",
            "final_executable_pipeline_execution_handoff_first_blocker",
            "final_executable_pipeline_execution_handoff_decision_code",
            "final_executable_pipeline_entrypoint_materialization_kind",
            "final_executable_pipeline_entrypoint_materialization_path",
            "final_executable_pipeline_entrypoint_materialization_ready",
            "final_executable_pipeline_entrypoint_materialization_first_blocker",
            "final_executable_pipeline_entrypoint_materialization_present",
            "final_executable_pipeline_entrypoint_materialization_hash",
            "final_executable_pipeline_entrypoint_materialization_runner_command",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-pipeline-self-owned-image-regression",
        path: "tools/nsld/src/main_final_executable_pipeline_tests.rs",
        required_patterns: &[
            "actual_self_owned_image_status",
            "actual_entrypoint_materialization_status",
            "actual_entrypoint_materialization_kind",
            "actual_entrypoint_materialization_ready",
            "actual_entrypoint_materialization_first_blocker",
            "actual_entrypoint_materialization_present",
            "actual_entrypoint_materialization_hash",
            "actual_entrypoint_materialization_runner_command",
            "actual_execution_handoff_contract",
            "actual_execution_handoff_ready",
            "actual_execution_handoff_status",
            "actual_execution_handoff_target",
            "actual_execution_handoff_evidence_status",
            "actual_execution_handoff_first_blocker",
            "actual_execution_handoff_decision_code",
            "self_owned_image_status",
            "entrypoint_materialization_status",
            "host-shell-entrypoint-plan",
            "NUIS_HOST_RUNNER",
            "nuis-host-runner --manifest",
            "required_stage_path_count, 10",
            "verify_final_executable_pipeline_reports_missing_entrypoint_materialization",
            "verify_final_executable_pipeline_reports_tampered_entrypoint_materialization",
            "entrypoint_materialization_hash mismatch",
            "nsld-final-output-handoff-v1",
            "handoff-entrypoint-materializer",
            "Some(\"ready\")",
            "Some(\"host-launcher-ready\")",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nustar-domain-contract-completeness-status",
        path: "tools/nuisc/src/registry_contract.rs",
        required_patterns: &[
            "contract_status",
            "required_domain_contract_groups",
            "missing_domain_contract_groups",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nustar-domain-contract-completeness-json",
        path: "tools/nuisc/src/registry_domain_json.rs",
        required_patterns: &[
            "contract_complete",
            "required_contract_groups",
            "missing_contract_groups",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "workflow-surface-json-regression",
        path: "tools/nuis/src/main_tests/workflow_surface.rs",
        required_patterns: &[
            "nsld_final_executable_output_boundary_status",
            "nsld_final_executable_output_materialization_status",
            "nsld_final_executable_output_execution_handoff_contract",
            "nsld_final_executable_output_execution_handoff_ready",
            "nsld_final_executable_output_execution_handoff_status",
            "nsld_final_executable_output_execution_handoff_target",
            "nsld_final_executable_output_execution_handoff_evidence_status",
            "nsld_final_executable_output_execution_handoff_first_blocker",
            "nsld_final_executable_output_execution_handoff_decision_code",
            "nsld_final_executable_pipeline_execution_handoff_contract",
            "nsld_final_executable_pipeline_execution_handoff_ready",
            "nsld_final_executable_pipeline_execution_handoff_status",
            "nsld_final_executable_pipeline_execution_handoff_target",
            "nsld_final_executable_pipeline_execution_handoff_evidence_status",
            "nsld_final_executable_pipeline_execution_handoff_first_blocker",
            "nsld_final_executable_pipeline_execution_handoff_decision_code",
            "nsld_final_executable_output_recommended_next_action",
            "nsld_self_owned_image_status",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "artifact-runtime-json-regression",
        path: "tools/nuis/src/main_tests/artifact_runtime.rs",
        required_patterns: &[
            "nsld_final_executable_output_boundary_status",
            "nsld_final_executable_output_materialization_status",
            "nsld_final_executable_output_execution_handoff_contract",
            "nsld_final_executable_output_execution_handoff_ready",
            "nsld_final_executable_output_execution_handoff_status",
            "nsld_final_executable_output_execution_handoff_target",
            "nsld_final_executable_output_execution_handoff_evidence_status",
            "nsld_final_executable_output_execution_handoff_first_blocker",
            "nsld_final_executable_output_execution_handoff_decision_code",
            "nsld_final_executable_pipeline_execution_handoff_contract",
            "nsld_final_executable_pipeline_execution_handoff_ready",
            "nsld_final_executable_pipeline_execution_handoff_status",
            "nsld_final_executable_pipeline_execution_handoff_target",
            "nsld_final_executable_pipeline_execution_handoff_evidence_status",
            "nsld_final_executable_pipeline_execution_handoff_first_blocker",
            "nsld_final_executable_pipeline_execution_handoff_decision_code",
            "nsld_final_executable_output_recommended_next_action",
            "nsld_self_owned_image_status",
            "run_artifact_prelaunch_kind",
            "run_artifact_prelaunch_status",
            "nsld-host-entrypoint",
            "artifact_closure_kind",
            "artifact_closure_status",
            "run_artifact_json_blocks_nsld_prelaunch_when_entrypoint_stub_is_missing",
            "host entrypoint stub is missing on disk",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "frontdoor-reference-doc",
        path: "docs/reference/nuis-frontdoor-surface-reference.md",
        required_patterns: &[
            "nsld_final_executable_output_boundary_status",
            "nsld_final_executable_output_materialization_status",
            "nsld_final_executable_output_execution_handoff_contract",
            "nsld_final_executable_output_execution_handoff_ready",
            "nsld_final_executable_output_execution_handoff_status",
            "nsld_final_executable_output_execution_handoff_target",
            "nsld_final_executable_output_execution_handoff_evidence_status",
            "nsld_final_executable_output_execution_handoff_first_blocker",
            "nsld_final_executable_output_execution_handoff_decision_code",
            "nsld_final_executable_pipeline_execution_handoff_contract",
            "nsld_final_executable_pipeline_execution_handoff_ready",
            "nsld_final_executable_pipeline_execution_handoff_status",
            "nsld_final_executable_pipeline_execution_handoff_target",
            "nsld_final_executable_pipeline_execution_handoff_evidence_status",
            "nsld_final_executable_pipeline_execution_handoff_first_blocker",
            "nsld_final_executable_pipeline_execution_handoff_decision_code",
            "nsld_final_executable_output_recommended_next_action",
            "nsld_self_owned_image_status",
            "run_artifact_prelaunch_kind",
            "run_artifact_prelaunch_status",
            "run_artifact_prelaunch_command",
            "artifact_closure_kind",
            "artifact_closure_status",
            "artifact_closure_command",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-binary-assembly-doc",
        path: "docs/reference/nsld-binary-assembly-gap-map.md",
        required_patterns: &["self_owned_image_status", "internal binary assembly layer"],
    },
    DevTensorDriftCheckSpec {
        id: "native-artifact-workflow-doc",
        path: "docs/reference/nuis-native-artifact-workflow.md",
        required_patterns: &[
            "nsld_final_executable_output_boundary_status",
            "nsld_final_executable_output_materialization_status",
            "nsld_final_executable_output_execution_handoff_contract",
            "nsld_final_executable_output_execution_handoff_ready",
            "nsld_final_executable_output_execution_handoff_status",
            "nsld_final_executable_output_execution_handoff_target",
            "nsld_final_executable_output_execution_handoff_evidence_status",
            "nsld_final_executable_output_execution_handoff_first_blocker",
            "nsld_final_executable_output_execution_handoff_decision_code",
            "nsld_final_executable_pipeline_execution_handoff_contract",
            "nsld_final_executable_pipeline_execution_handoff_ready",
            "nsld_final_executable_pipeline_execution_handoff_status",
            "nsld_final_executable_pipeline_execution_handoff_target",
            "nsld_final_executable_pipeline_execution_handoff_evidence_status",
            "nsld_final_executable_pipeline_execution_handoff_first_blocker",
            "nsld_final_executable_pipeline_execution_handoff_decision_code",
            "nsld_final_executable_output_recommended_next_action",
            "nsld_self_owned_image_status",
            "run_artifact_prelaunch_kind",
            "run_artifact_prelaunch_status",
            "run_artifact_prelaunch_command",
            "artifact_closure_kind",
            "artifact_closure_status",
            "artifact_closure_command",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "std-filesystem-light-smoke",
        path: "tools/nuis/tests/std_filesystem_smoke.rs",
        required_patterns: &[
            "STD_TOOLING_LIGHT_SMOKE_PROJECTS",
            "std_tooling_light_project_smokes_build_doctor_and_run",
            "text_pipeline_demo",
            "io_runtime_demo",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "std-tooling-doc-smoke-chain",
        path: "examples/projects/tooling/README.md",
        required_patterns: &[
            "cargo test -q -p nuis --test std_filesystem_smoke",
            "filesystem_io_report_demo",
            "text_report_json_demo",
            "terminal/stdin/TTY",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "std-readme-host-io-text-lane",
        path: "stdlib/std/README.md",
        required_patterns: &[
            "host I/O and text",
            "filesystem/path/location",
            "lib/io_contracts.ns",
            "lib/text_contracts.ns",
            "lib/fs_contracts.ns",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "dev-tensor-coverage-doc",
        path: "docs/reference/nuis-development-tensor.md",
        required_patterns: &[
            "Coverage Manifest",
            "coverage_status",
            "coverage_missing_coordinates",
            "coverage_orphaned_coordinates",
            "coverage_stale_coordinates",
            "coverage_first_gap",
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DevTensorCoverageSummary {
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
        coverage_status: coverage.status,
        coverage_expected_count: coverage.expected_count,
        coverage_covered_count: coverage.covered_count,
        coverage_missing_count: coverage.missing_count,
        coverage_orphaned_count: coverage.orphaned_count,
        coverage_stale_count: coverage.stale_count,
    }
}

pub(crate) fn dev_tensor_coverage_summary() -> DevTensorCoverageSummary {
    let cell_coordinates = DEV_TENSOR_CELLS
        .iter()
        .map(|cell| dev_tensor_coordinate_key(cell.architecture, cell.module, cell.function))
        .collect::<BTreeSet<_>>();
    let expected_coordinates = DEV_TENSOR_EXPECTED_COORDINATES
        .iter()
        .map(|coordinate| {
            dev_tensor_coordinate_key(
                coordinate.architecture,
                coordinate.module,
                coordinate.function,
            )
        })
        .collect::<BTreeSet<_>>();
    let missing_coordinates = DEV_TENSOR_EXPECTED_COORDINATES
        .iter()
        .filter_map(|coordinate| {
            let key = dev_tensor_coordinate_key(
                coordinate.architecture,
                coordinate.module,
                coordinate.function,
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
                || cell.closure_role.is_empty()
                || cell.evidence.is_empty()
                || cell.next_step.is_empty()
                || cell.progress > 100;
            stale.then(|| dev_tensor_coordinate_key(cell.architecture, cell.module, cell.function))
        })
        .collect::<Vec<_>>();
    let covered_count = DEV_TENSOR_EXPECTED_COORDINATES
        .len()
        .saturating_sub(missing_coordinates.len());
    let status = if required_missing_count == 0
        && orphaned_coordinates.is_empty()
        && stale_coordinates.is_empty()
    {
        "clean"
    } else {
        "gap"
    };
    let first_gap = missing_coordinates
        .first()
        .or_else(|| orphaned_coordinates.first())
        .or_else(|| stale_coordinates.first())
        .cloned();
    DevTensorCoverageSummary {
        expected_count: DEV_TENSOR_EXPECTED_COORDINATES.len(),
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
    }
}

fn dev_tensor_coordinate_key(architecture: &str, module: &str, function: &str) -> String {
    format!("{architecture}/{module}/{function}")
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
    let coverage = dev_tensor_coverage_summary();
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
            json_field("coverage_status", coverage.status),
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
    let coverage = dev_tensor_coverage_summary();
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
        format!("  coverage_status: {}", coverage.status),
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
        assert_eq!(summary.coverage_status, "clean");
        assert_eq!(
            summary.coverage_expected_count,
            DEV_TENSOR_EXPECTED_COORDINATES.len()
        );
        assert_eq!(summary.coverage_missing_count, 0);
        assert_eq!(summary.coverage_orphaned_count, 0);
        assert_eq!(summary.coverage_stale_count, 0);
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
        assert!(json.contains("\"coverage_status\":\"clean\""));
        assert!(json.contains("\"coverage_expected_count\":"));
        assert!(json.contains("\"coverage_missing_count\":0"));
        assert!(json.contains("\"coverage_orphaned_count\":0"));
        assert!(json.contains("\"coverage_stale_count\":0"));
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
        assert!(text.contains("coverage_missing_count: 0"));
        assert!(text.contains("coverage_orphaned_count: 0"));
        assert!(text.contains("coverage_stale_count: 0"));
        assert!(text.contains("drift_status: clean"));
        assert!(text.contains("drift_check: id=frontdoor-final-output-boundary-status"));
        assert!(text.contains("drift_check: id=std-filesystem-light-smoke"));
        assert!(text.contains("drift_first_failed_check: <none>"));
    }

    #[test]
    fn dev_tensor_coverage_manifest_matches_current_cells() {
        let coverage = dev_tensor_coverage_summary();
        assert_eq!(coverage.status, "clean");
        assert_eq!(
            coverage.expected_count,
            DEV_TENSOR_EXPECTED_COORDINATES.len()
        );
        assert_eq!(coverage.covered_count, DEV_TENSOR_CELLS.len());
        assert_eq!(coverage.missing_count, 0);
        assert_eq!(coverage.required_missing_count, 0);
        assert_eq!(coverage.orphaned_count, 0);
        assert_eq!(coverage.stale_count, 0);
        assert!(coverage.first_gap.is_none());
        assert!(coverage.missing_coordinates.is_empty());
        assert!(coverage.orphaned_coordinates.is_empty());
        assert!(coverage.stale_coordinates.is_empty());
    }
}
