use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(crate) const DEV_TENSOR_MAINLINE_DRIFT_CHECKS: &[DevTensorDriftCheckSpec] = &[
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
