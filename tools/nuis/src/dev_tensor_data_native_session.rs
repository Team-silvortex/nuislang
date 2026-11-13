use crate::dev_tensor::DevTensorCell;

pub(super) const SCALAR_RECORD_PROFILE: DevTensorCell = DevTensorCell {
    architecture: "compiler-toolchain",
    module: "nuisc",
    function: "native-scalar-record-session-profile",
    status: "stable",
    progress: 100,
    bootstrap_critical: false,
    closure_role: "bounded-native-session-profile",
    evidence: "Closed bounded profile, not all CPU lowering: explicit scalar/value-record slot transport, guarded finite control/effects, exact print-site grants and independent loop/helper budgets have native/reference and source-free delivery receipts. The direct-return checkpoint records 224 compiler tests, 15 native/reference tests and 120 CLI launches on the local host; no fresh Linux/GPU parity is implied.",
    next_step: "Maintain the admitted profile and its rejection boundaries. Extend it only for a reproduced caller requirement; prioritize native resource callbacks for the application path.",
    blocker: "No blocker for this bounded slice. Buffer/Bytes ownership, provider calls and arbitrary control remain outside its admission and have separate coordinates.",
    next_action: "Retain direct/loop record regression coverage and exact bit/State publication while generic resource admission is developed separately.",
    validation_command: "env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_return_direct_eager_pair --test-threads=1",
    expected_artifact: "Stable scalar/record callback baseline with no reference fallback, no synthetic loop requirement and explicit rejection of resources; not an interactive image application.",
};

pub(super) const RESOURCE_CALLBACKS: DevTensorCell = DevTensorCell {
    architecture: "compiler-toolchain",
    module: "nuisc",
    function: "native-resource-session-callbacks",
    status: "early",
    progress: 0,
    bootstrap_critical: false,
    closure_role: "native-resource-callback-boundary",
    evidence: "The scalar bridge admits value records but excludes Buffer/Bytes/provider instructions; runtime tests reject resource-bearing native State. Buffer-writing native function tests and embedded-YIR image execution are adjacent evidence, not registered native resource-session acceptance.",
    next_step: "Establish the smallest generic owned Buffer/Bytes callback boundary before extending the complete native frame path.",
    blocker: "No registered native session contract yet carries owned resources through admission, invocation, completion and cleanup; scalar slots must not smuggle raw host pointers or borrow provider ownership.",
    next_action: "Reproduce one Buffer-writing callback rejected by the scalar bridge, then introduce a separately registered resource capability/descriptor path. Verify ownership transfer, invalid-capability nonentry, trap/failure cleanup and complete State publication without engine-specific compiler logic or interpreter fallback.",
    validation_command: "env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib --locked -j1 -- native_session_resource_boundary --test-threads=1 && env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_application_session --test-threads=1",
    expected_artifact: "A native-vs-reference Buffer/Bytes session witness with exact bytes, explicit ownership/GLM and completion, bounded lifetime, rejected drift and once-only cleanup. The current rejection tests are baseline gates, not implementation success.",
};
