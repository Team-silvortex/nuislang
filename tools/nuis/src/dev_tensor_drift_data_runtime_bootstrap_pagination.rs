use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(crate) const DEV_TENSOR_RUNTIME_BOOTSTRAP_PAGINATION_DRIFT_CHECKS:
    &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "nuis-compiler-structural-three-page-artifact",
        path: "crates/nuis-artifact/src/compiler_structural_projection_page.rs",
        required_patterns: &[
            "CompilerProjectionThreePageIdentity",
            "compiler_projection_three_page_identity",
            "compiler_projection_resume_page_identity",
            "first_two",
            "compiler structural continuation requires a third page",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nuis-compiler-candidate-structural-pagination-result-artifact",
        path: "crates/nuis-artifact/src/compiler_candidate_structural_pagination_result.rs",
        required_patterns: &[
            "COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_PROTOCOL",
            "nuis-bootstrap-candidate-structural-pagination-v1",
            "COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_PAGE_COUNT",
            "parse_compiler_candidate_structural_pagination_result_bytes",
            "render_compiler_candidate_structural_pagination_result",
            "not canonically encoded",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nuis-compiler-candidate-structural-pagination-artifact",
        path: "crates/nuis-artifact/src/compiler_candidate_structural_pagination.rs",
        required_patterns: &[
            "COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_PROTOCOL",
            "build_compiler_candidate_structural_pagination",
            "read_compiler_candidate_structural_pagination",
            "candidate_owned_pagination: true",
            "predecessor_unchanged: true",
            "stage0_provider_dependency: false",
            "replacement_authorized: false",
            "selection_authorized: false",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nuis-compiler-candidate-structural-pagination-adapter",
        path: "tools/nuis/src/bootstrap_candidate_structural_pagination.rs",
        required_patterns: &[
            "STRUCTURAL_PAGINATION_COMMAND",
            "structural-pagination-v1",
            "run_candidate_structural_pagination",
            ".env_clear()",
            ".stdin(Stdio::null())",
            "projection_resume_value",
            "page_count=3",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nuis-compiler-candidate-structural-pagination-frontdoor",
        path: "tools/nuis/src/bootstrap_candidate_build.rs",
        required_patterns: &[
            "COMPILER_CANDIDATE_STRUCTURAL_PAGINATION_RESULT_FILE",
            "parse_compiler_candidate_structural_pagination_result_bytes",
            "build_compiler_candidate_structural_pagination",
            "read_compiler_candidate_structural_pagination",
            "structural_pagination_sha256",
            "structural_pagination_record",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nuis-compiler-candidate-structural-pagination-contract",
        path: "docs/reference/nuis-compiler-candidate-structural-pagination-v1.toml",
        required_patterns: &[
            "nuis-compiler-candidate-structural-pagination-v1",
            "nuis-bootstrap-candidate-structural-pagination-v1",
            "result_line_count = 62",
            "page_count = 3",
            "candidate_owned_pagination = true",
            "host_recomputed = true",
            "predecessor_unchanged = true",
            "replacement_authorized = false",
            "selection_authorized = false",
        ],
    },
];
