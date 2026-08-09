pub const CPU_MUTEX_RUNTIME_METADATA: [&str; 4] = [
    "mutex_contract=scheduler-handle-v1",
    "visibility=acquire-release-epoch-v1",
    "authority=linear-guard-v1",
    "payload_policy=i64-native-staged-fallback",
];

pub const CPU_SHARED_MUTEX_RUNTIME_METADATA: [&str; 6] = [
    "mutex_contract=scheduler-handle-v1",
    "visibility=acquire-release-epoch-v1",
    "authority=linear-permit-lease-v1",
    "permit_scope=fixed-two-lane-v1",
    "permit_policy=one-shot-generation-bound-v1",
    "payload_policy=i64-native-staged-fallback",
];
