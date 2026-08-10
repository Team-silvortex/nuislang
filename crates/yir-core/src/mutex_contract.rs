pub const CPU_MUTEX_RUNTIME_METADATA: [&str; 4] = [
    "mutex_contract=scheduler-handle-v1",
    "visibility=acquire-release-epoch-v1",
    "authority=linear-guard-v1",
    "payload_policy=i64-native-staged-fallback",
];

pub const CPU_SHARED_MUTEX_RUNTIME_METADATA: [&str; 8] = [
    "mutex_contract=scheduler-handle-v1",
    "visibility=acquire-release-epoch-v1",
    "authority=linear-permit-lease-v1",
    "permit_cardinality=share-literal-1-to-64-v1",
    "permit_policy=one-shot-generation-bound-v1",
    "payload_policy=i64-native-staged-fallback",
    "lifecycle=explicit-close-revoke-v1",
    "mutation=lease-replace-release-epoch-v1",
];
