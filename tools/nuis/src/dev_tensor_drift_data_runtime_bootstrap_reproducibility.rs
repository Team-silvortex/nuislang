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
    DevTensorDriftCheckSpec {
        id: "nuis-compiler-token-decoder-artifact",
        path: "crates/nuis-artifact/src/compiler_token_decoder.rs",
        required_patterns: &[
            "nuis-compiler-token-decoder-v1",
            "decode_compiler_token_stream",
            "COMPILER_TOKEN_DECODER_MAX_BYTES",
            "COMPILER_TOKEN_DECODER_MAX_RECORDS",
            "fold_hex_payload",
            "fold_integer_payload",
            "fold_symbol_payload",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nuis-compiler-token-decoder-stdlib",
        path: "stdlib/std/lib/compiler_tokens.ns",
        required_patterns: &[
            "mod cpu StdCompilerTokens",
            "compiler_token_decoder_step",
            "compiler_token_decoder_count_step",
            "compiler_token_decoder_semantic_step",
            "compiler_token_decoder_finish",
            "2147483629",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nuis-compiler-token-decoder-candidate-abi",
        path: "examples/projects/tooling/bootstrap_structural_projection_candidate/main.ns",
        required_patterns: &[
            "use cpu StdCompilerTokens",
            "nuis_bootstrap_candidate_token_start_v1",
            "nuis_bootstrap_candidate_token_step_v1",
            "nuis_bootstrap_candidate_token_semantic_step_v1",
            "nuis_bootstrap_candidate_token_finish_v1",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nuis-compiler-token-decoder-production-binding",
        path: "crates/nuis-artifact/src/compiler_candidate_production.rs",
        required_patterns: &[
            "nuis-compiler-candidate-production-v2",
            "token_decoder_contract",
            "token_record_count",
            "token_semantic_fold",
            "token decode summary mismatch",
        ],
    },
];
