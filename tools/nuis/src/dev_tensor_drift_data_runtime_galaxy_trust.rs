use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

pub(crate) const DEV_TENSOR_RUNTIME_GALAXY_TRUST_DRIFT_CHECKS: &[DevTensorDriftCheckSpec] = &[
    DevTensorDriftCheckSpec {
        id: "galaxy-provider-persistent-trust-state",
        path: "tools/nuisc/src/stdlib_registry_provider_trust_state.rs",
        required_patterns: &[
            "nuis-galaxy-provider-trust-registry-v1",
            "nuis-galaxy-provider-trust-state-v1",
            "enforce_candidate_set_trust",
            "authorize_candidate_signers",
            "candidate-set rollback",
            "candidate-set same-generation fork",
            "trust registry rollback",
            "trust registry same-generation fork",
            "must not be group or other writable",
            "verified-persistent-trust",
            "state_identity",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "galaxy-provider-atomic-trust-state-io",
        path: "tools/nuisc/src/stdlib_registry_provider_trust_state_io.rs",
        required_patterns: &[
            "nuis-galaxy-provider-trust-state-lock-v1",
            "create_new(true)",
            "LOCK_STALE_AFTER_MS",
            "sync_all",
            "fs::rename",
            "validate_state_target",
            "owner_token",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "galaxy-provider-trust-fail-closed-regression",
        path: "tools/nuisc/src/stdlib_registry_provider_trust_state_tests.rs",
        required_patterns: &[
            "revoked_and_unknown_signers_fail_before_state_creation",
            "candidate_generation_rejects_rollback_and_same_generation_fork",
            "registry_generation_rejects_rollback_and_same_generation_fork",
            "policy_files_inside_provider_root_fail_closed",
            "writable_registry_or_control_directory_fails_closed",
            "concurrent_replay_serializes_to_one_canonical_state",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "galaxy-provider-trust-lock-cache-isolation",
        path: "tools/nuis/src/galaxy/deps_provider_tests.rs",
        required_patterns: &[
            "persistent_provider_trust_does_not_enter_lock_or_addressed_cache",
            "verified-trusted-candidate-set",
            "verified-persistent-trust",
            "snapshot_tree(&unsigned.synced.root)",
            "provider-trust-state",
        ],
    },
    DevTensorDriftCheckSpec {
        id: "galaxy-provider-trust-cli-and-protocol",
        path: "docs/reference/galaxy-provider-trust-state-v1.toml",
        required_patterns: &[
            "nuis-galaxy-provider-trust-state-v1",
            "--trust-registry",
            "--trust-state",
            "range_requires_trusted_status = true",
            "trust_fields_in_resolution_lock = false",
            "trust_fields_in_addressed_cache = false",
        ],
    },
];
