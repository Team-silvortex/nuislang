use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(crate) const DEV_TENSOR_MAINLINE_DRIFT_CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "application-session-provider-finish-gate",
        path: "crates/yir-runtime-host/src/provider_application_session.rs",
        required_patterns: &[
            "nuis-yir-provider-application-session-v1",
            "ApplicationSession::preflight",
            "ApplicationProviderSource::Ipc",
            "ApplicationProviderSource::Replay",
            "application.completion_status()?",
            "finish_provider_source(&provider)?",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "application-session-provider-failure-regressions",
        path: "crates/yir-runtime-host/tests/provider_application_session.rs",
        required_patterns: &[
            "one_transport_carries_independent_events_and_finishes_only_after_close",
            "missing_close_driver_error_and_swallowed_failures_do_not_acknowledge_success",
            "wrong_module_and_close_acknowledgement_are_rejected",
            "invalid_binding_is_rejected_before_connecting",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "application-session-live-metal-evidence",
        path: "tools/nuis/src/artifact_device_sample_shader_session_tests.rs",
        required_patterns: &[
            "executes_ns_nova_persistent_image_session_through_live_provider",
            "nsdb::serve_runtime_provider_session",
            "PhysicalFence",
            "runtime_dispatch_session_worker_count",
            "compiled,hit",
            "unconsumed frame",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "application-session-boundary-and-scope",
        path: "docs/reference/nuis-yir-application-session-v1.md",
        required_patterns: &[
            "nuis-yir-application-session-v1",
            "reference provider",
            "not migrated",
            "no rollback",
            "ns_nova_application_session",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "application-session-runtime-carrier",
        path: "crates/yir-runtime-host/src/application_session.rs",
        required_patterns: &[
            "nuis-yir-application-session-v1",
            "SessionBoundary::bind",
            "FunctionSession::new",
            "ApplicationSessionPhase::Faulted",
            "ApplicationSessionPhase::Closed",
            "close_error",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "application-session-compiled-nuis-evidence",
        path: "tools/nuisc/tests/ns_nova_application_session.rs",
        required_patterns: &[
            "compile_project",
            "compiled_nuis_image_state_survives_independent_events_without_main_replay",
            "last_completion_tick",
            "frame history must be drained",
            "entry_nodes.contains",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "dev-tensor-application-led-plan",
        path: "docs/reference/nuis-development-tensor.mainline.toml",
        required_patterns: &[
            "nuis-dev-tensor-mainline-v1",
            "ns-nova-application-led",
            "interrupts =",
            "persistent-application-session",
            "lifecycle-failure-resource-safety",
            "sustained-runtime-performance",
            "compiler-component-ownership-transfer",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "dev-tensor-mainline-manifest-validation",
        path: "tools/nuis/src/dev_tensor_mainline_parse.rs",
        required_patterns: &[
            "unsupported mainline schema",
            "duplicate mainline node",
            "duplicate mainline field",
            "duplicate mainline array entry",
            "unknown mainline field",
            "unterminated mainline array",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "dev-tensor-mainline-selection-contract",
        path: "docs/reference/nuis-development-tensor-mainline.md",
        required_patterns: &[
            "mainline_dependency_path",
            "mainline-plan-invalid",
            "mainline-plan-complete",
            "not a measured",
            "baseline validation",
            "not substitutes for runtime",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "application-led-module-migration-roadmap",
        path: "docs/versioning/nuis-beta-0.11-application-led-mainline.md",
        required_patterns: &[
            "beta-0.11.3",
            "entire engine",
            "not compiler self-hosting",
            "existing explicit authorization",
            "gamma-0.5.*",
            "gamma-0.10.*",
        ],
    },
];
