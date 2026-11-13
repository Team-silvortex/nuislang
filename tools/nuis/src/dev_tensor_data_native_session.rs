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
    status: "active",
    progress: 95,
    bootstrap_critical: false,
    closure_role: "native-resource-callback-boundary",
    evidence: "The Bytes-only and mixed profiles retain 80 events, complete-state publication and ordinary-error cleanup evidence. Mixed State admits 1..=16 i64 fields plus one owned Bytes resource. The checked Result<State, i64> contract retains Nuis-owned validation and explicit errors. transfer_bytes_v1 and cleanup_bytes_v1 partition live owners exactly. Earlier three nested conditions retain selected-arm evidence. Empty-else and nonempty else joins reach one shared root continuation through tail_call_owned_struct_return without copying the suffix or repacking its Result. 120 alternating events across six paired-arm paths preserve snapshots and once-only suffix effects. The explicit i64 let/const scalar shadowing baseline now extends to inferred and aliased i64: both forms run 120 events each, preserving local early returns, outer continuation values and zero tracked live blobs. Registered roots retain declaration identities before NIR merges declarations and assignments; shared semantic lowering resolves types, then canonical i64 admission checks both sides before optimization. Imported/generic aliases, inferred outer locals, scalar helpers, Buffer reads, real-write rejection and failure cleanup have scoped regressions. Shared bounds retain root-inclusive depth 8 and 32 functions. Only successfully closed RGBA8 State is exported. The inferred/alias paired canvas retains cache, source-free NSB relocation and binary-only launch with empty PATH. A record-returning helper field resolves semantically but native admission still rejects call_owned_struct; that is the next concrete boundary. See docs/reference/nuis-native-resource-inferred-shadowing-v1.md; local host evidence only, no fresh Linux/GPU parity.",
    next_step: "Admit a bounded scalar/value-record helper result inside resource callbacks, with independent call_owned_struct checks and original Bytes ownership preserved across the call. Retain inferred scalar shadowing, the root continuation and source-free delivery evidence.",
    blocker: "Mixed State is limited to flat i64 fields and one Bytes owner. Inferred/aliased i64 shadows are admitted, but native resource callbacks still reject ordinary call_owned_struct helper results. Non-i64 shadows and additional lexical regions, observable writes rebinding captured values, resource-changing phi joins, general resource loops/helpers, resize/cancellation and provider/device completion remain outside the synchronous fixed-extent profile. Static host finalization still uses clang and the existing allocation compatibility ABI, not pure Nsld or an all-Nuis runtime.",
    next_action: "Use the reproduced measure(width).value case: semantic inference succeeds but emit_registered rejects call_owned_struct. Reuse the existing scalar/value-record callee contract without granting arbitrary resource helpers; verify selected execution, held Bytes cleanup, allocation/error paths and source-free packaging.",
    validation_command: "env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_resource_session_workflow --locked -j1 -- --test-threads=1 && env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_resource_session --locked -j1 -- --test-threads=1 && env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib --locked -j1 -- native_session_resource_boundary --test-threads=1 && env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_application_session --test-threads=1",
    expected_artifact: "Packaged Bytes-only, mixed-State and checked Result<State, i64> resource sessions with Nuis-owned dimensions/control and explicit configuration errors, exact final PPM export and source-free execution. This does not prove native frame integration for ns-nova windows, arbitrary mixed layouts, recoverable traps, provider/device retirement or whole-application closure.",
};
