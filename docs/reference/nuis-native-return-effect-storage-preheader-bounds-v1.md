# Native Return Storage Parent-Preheader Bound Records V1

This bounded local CPU follow-up extends the
[shared sibling-bound witnesses](nuis-native-return-effect-storage-sibling-bounds-v1.md).
Git remains beta-0.17.4 at bef7a23d. No release or production authority change is made.

## One Snapshot Across Backedges

The [shared Nuis fixture adapter](../../tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds_cases.rs)
adds four explicit parent-preheader variants, retaining the previous four
per-parent-iteration variants unchanged. The selected region constructs one
completed immutable `frozen` record before entering its parent loop. Both
siblings consume that same record on both parent iterations, even though the
live bool/i64/f32/f64 State changes. The outer selected guard prevents snapshot
construction and poisoned arithmetic in the inactive region.

For initial total 0 and trips 4, all four reached child invocations consume
width 4. The oracle requires one construction pair 930/931, two parent effects
941 and four extent observations. Final total is 123 with leading child steps
or 107 with trailing steps. This differs from reconstructing the snapshot
after each live-State update. The selected guard retained inside the parent
loop deliberately exercises existing mirrored branch handling; it is not a
new optimizer special case.

Each callback owns its own snapshot. Later callbacks derive it from the actual
previous State, not deliberately different unused event State inputs.
Negative zero and other raw float cases retain their existing word checks.
The old sibling fixture still constructs once per selected parent iteration.

## Independent Reservations And Failures

Normal fixture limits stay 18 loop-work units and 192 inclusive YIR helper
entries. Production defaults and guards are unchanged.

A dedicated native matrix freezes width 2 before the parent loop. Parent cost
2 plus four independently reached width-2 invocations requires 10 units.
Limit 9 fails at the second parent's second-child reservation, after print 945
but before that child's body. It also fails there when the first child breaks
after one iteration: reserving a bound is not charging actual iterations.
Limit 10 succeeds. Parent continue skips the second sibling but must reserve
the first child again after the backedge: limit 5 fails after the second 942.
Parent break at limit 4 and first-parent return at limit 6 succeed.

Function returns, parent break/continue and child-local break/continue retain
distinct targets. Unreached argument and callee poisons stay inert. Selected
faults cover snapshot construction, first extent division, second extent
argument evaluation, second extent callee and MIN/-1 overflow. Snapshot
construction precedes parent reservation only in the selected preheader
variant, exactly as in its source.

The overflow prefix is even for preheader variants and odd for the old variants:
freezing before versus after the first parent update changes the required
State parity. Both use actual accepted-prefix State, preserving the intended
scale-1 division rather than accidentally testing a huge valid loop.

Reference sessions retain the last accepted raw State after three successful
callbacks, enter Faulted, reject later events and refuse successful completion
after close. Native trap probes check exact flushed effect prefixes and no
callback publication; they do not inspect State after process termination.

## Certificates And Delivery

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_preheader_bound_tests.rs)
rename carry and snapshot, preserve immutable source, deny direct callee grants,
and reject treating the dominating but read-only snapshot as return storage.
Type-valid current-carry substitution, recomputation, effect reordering and
argument-check bypasses are rejected by the existing structural certificate.

Independent/shared normal/reversed YIR comparisons cover all ten exit modes,
inactive poison and five selected fault categories. They require an actual
four-word logical carry reduction, not measured physical memory or speedup.
No production lowering capability is widened; changes there register tests
and share test-only helpers.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs)
reuse the existing session and raw-word probe owners for all four new variants.
[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs)
uses one new representative through fresh build, cache hit and two repeated
materializations after deleting source, manifest, project cache and original
output. Every image exercises healthy exit modes, unreachable poisons and all
five selected failures. Binary/container, host-object, envelope, lifecycle and
packaging identities remain checked.

## Remaining Frontier

The [distinct-epoch follow-up](nuis-native-return-effect-storage-epoch-bounds-v1.md)
tracks the next bounded identity witness. The acceptance below remains specific
to this earlier single-preheader-record receipt.

The tensor coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next distinguish two simultaneously live completed bound records from different
State epochs: one above the parent loop and one after selected live updates.
Keep identities, construction order, independent full reservations and exit
targets distinct across backedges. Diagnose existing admission before widening
anything. Original/shared reads, bounded attempts, exact fallback,
ownership/GLM/time contracts and call authority remain gates.

This is local CPU evidence only, not device or cross-platform parity, arbitrary
resource/I/O/async capture, stable ABI, full heterogeneous closure or self-hosting.

## Validation

Focused acceptance passes 341 distinct tests, including eight new tests:
270 compiler units (411.08 seconds), 12 native/reference tests (415.87 seconds),
three CLI delivery tests (466.99 seconds), three host-fault tests,
24 repository-maintenance tests, two line-budget tests and 27 tensor tests.
The new tests comprise four compiler, three native/reference and one CLI test;
calibration and reruns are not counted again.

Native acceptance covers all four new preheader variants, the preceding four
sibling variants and the earlier record-dependent bound mode. CLI acceptance
covers 12 fresh/cache/restored images across one representative of each of those
three source families, not all variants or every earlier source mode.

The ordinary CLI rebuild and live tensor audit pass: all six fields match source,
all six complete predecessor hashes verify, the task card is ready, and drift,
coverage, hierarchy, lineage and inert history are clean. Static drift validation
passes 3,513 checks with 24,094 required patterns. Locked metadata contains 29
workspace packages. Formatting, UTF-8 (5,597 files), local links (4,056), source/test
line budgets and whitespace checks pass. The coordinate remains active/99.

The initial admission calibration used an outdated expected oracle total;
the final suite checks the corrected 123/107 values. Production lowering,
defaults and guards are unchanged. These are bounded local CPU results, not
a full-workspace, GPU, cross-platform or self-hosting acceptance claim.

Executable routes:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- iteration_effects control_loops scalar_helpers scalar_control state_loop --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_loop_sibling_bounds:: state_loop_return_shared::bound_tests:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop_sibling_bounds:: effect_calls::state_loop_return_shared::branch_bound_calls --test-threads=1 --nocapture
```

All six tensor fields retain their complete preceding values and SHA-256.
Historical commands remain inside an inert quoted here-document.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `bd0224ce4b12ff5c17c27146da36e465d77750f8072336386c22764b2117c215` |
| `next_step` | `5398901803bc91ef702d4d9528c09573a4c9a0162ae832fb1d4309a3d06fd9e1` |
| `blocker` | `31375b9f4377be74d54ed646726db47193e79ab9897827a4d05d1e47734d3cc8` |
| `next_action` | `d65d325a7403a3a8c5f33afa9186530d607007705895bc4b43339196f2ea969c` |
| `validation_command` | `c6e4880ef48473c69729dc6b89e4b35d8867e2f18878e1671192fae432595aa5` |
| `expected_artifact` | `96249fb6a134243599484d9805a0681bc2072a63fae966d4273eaaf27c8289f8` |

