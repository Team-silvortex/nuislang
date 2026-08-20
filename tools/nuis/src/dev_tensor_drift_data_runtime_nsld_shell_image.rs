use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(crate) const DEV_TENSOR_RUNTIME_NSLD_SHELL_IMAGE_DRIFT_CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "nsld-macho-arm64-shell-image-serialization-contract",
        path: "tools/nsld/src/final_executable_macho_shell_image.rs",
        required_patterns: &[
            "nuis-nsld-macho-arm64-shell-image-serialization-v1",
            "serialize_macho_arm64_shell_image",
            "private-image-serialized-with-code-signature-boundary",
            "private-not-published",
            "payload-pending",
            "encode_shell_header_and_commands",
            "encode_shell_linkedit",
            "rewrite_shell_image_addresses",
            "serialization_ledger_hash",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-macho-arm64-shell-image-final-address-evidence",
        path: "tools/nsld/src/final_executable_macho_shell_image_tests.rs",
        required_patterns: &[
            "serializes_a_deterministic_private_arm64_shell_image",
            "relocation-final-address",
            "stub-final-address",
            "serializes_internal_got_rebase_with_final_pointer_value",
            "internal-got-final-address",
            "platform image drift",
            "input hash drift",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "nsld-macho-arm64-shell-image-three-surface-evidence",
        path: "tools/nsld/tests/host_finalizer_cli.rs",
        required_patterns: &[
            "nuis-nsld-macho-arm64-shell-image-serialization-v1",
            "private-image-serialized-with-code-signature-boundary",
            "private-not-published",
            "payload-pending",
            "shell_image_serialization",
            "finalizer_input_shell_image_contract",
            "finalizer_input_shell_image_rewrite",
            "persisted_invoke_plan",
        ],
    },
];
