use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(crate) const DEV_TENSOR_RUNTIME_BOOTSTRAP_REPRODUCIBILITY_DRIFT_CHECKS:
    &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "nuis-compiler-component-reproducibility-contract",
        path: "docs/reference/nuis-compiler-component-reproducibility-v1.toml",
        required_patterns: &[
            "nuis-compiler-component-reproducibility-v1",
            "nuis-compiler-two-clean-build-roots-v1",
            "nuis bootstrap-reproducibility",
            "compile_cache_status = \"bypass\"",
            "run_count = 2",
            "reproducible-equivalent-awaiting-authorization",
            "no-physical-paths-or-timestamps-in-aggregate",
            "replacement_authorized = false",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nuis-compiler-component-reproducibility-artifact",
        path: "crates/nuis-artifact/src/compiler_component_reproducibility.rs",
        required_patterns: &[
            "COMPILER_COMPONENT_REPRODUCIBILITY_PROTOCOL",
            "COMPILER_COMPONENT_CLEAN_BUILD_CONTRACT",
            "build_compiler_component_reproducibility_from_paths",
            "read_compiler_component_reproducibility",
            "compile-cache bypass evidence",
            "two distinct build roots",
            "drifted across clean runs",
            "replacement_authorized: false",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nuis-compiler-component-reproducibility-frontdoor",
        path: "tools/nuis/src/bootstrap_reproducibility.rs",
        required_patterns: &[
            "handle_bootstrap_reproducibility",
            "prepare_empty_output_root",
            "handle_bootstrap_clean_candidate_build",
            "clean-build-0",
            "clean-build-1",
            "build_compiler_component_reproducibility_from_paths",
            "bootstrap compiler reproducibility: verified",
            "replacement_authorized: false",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nuis-compiler-component-reproducibility-native-regression",
        path: "tools/nuis/tests/compiler_structural_projection_candidate.rs",
        required_patterns: &[
            "bootstrap-reproducibility",
            "two_uncached_clean_candidates_bind_one_reproducibility_aggregate",
            "compile_cache_status.as_deref(), Some(\"bypass\")",
            "reproducible-equivalent-awaiting-authorization",
            "bound root tampering must invalidate aggregate",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nuis-try-expansion-compilation-local-counter",
        path: "tools/nuisc/src/frontend/stmt_lowering_try.rs",
        required_patterns: &[
            "thread_local!",
            "TRY_EXPANSION_COUNTER: Cell<usize>",
            "reset_try_expansion_counter",
            "next_try_expansion_id",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nuis-try-expansion-reproducibility-regression",
        path: "tools/nuisc/src/frontend/tests_reproducibility.rs",
        required_patterns: &[
            "repeated_same_thread_lowering_resets_try_expansion_names",
            "lower_ast_to_nir(&ast)",
            "__nuis_try_result_0",
            "assert_eq!(first, second)",
        ],
    },
];
