# Native Return Storage Record-Dependent Child Bounds V1

This bounded CPU follow-up extends the
[branch-selected helper witnesses](nuis-native-return-effect-storage-branch-calls-v1.md).
The inspected Git identity remains beta-0.17.4 at bef7a23d. No release is made.

## Completed Records Determine Work

The eighth explicit source mode, `BranchBoundCalls`, consumes the completed
record returned by the selected outer helper before entering a selected child
loop. Its [source adapter](../../tools/nuisc/tests/native_application_bridge/state_loop_return_shared_bound_calls.rs)
inserts real [Nuis helpers](../../tools/nuisc/tests/native_application_bridge/state_loop_return_shared_bound_helpers.ns).
`bound_seed` reads the completed record's integer parity, choosing scale 1 for
even values and 2 for odd values. It returns a record containing `trips / scale`.
`bound_extent` consumes that completed record and calls the original `extent`
division. Thus the reached loop actually uses fewer iterations for an odd
completed State; this is not a canceled arithmetic placeholder or a Rust
replacement for Nuis computation. Negative parity also produces scale 1 or 2.

The new computation remains inside the existing source-valid, catalog-admitted
route. No production call, purity, ownership, GLM, read-authority or reservation
permission is expanded. The production module change only registers tests.

## Reached-Only Evaluation And Reservations

For a successfully reached bound, source effects are ordered:

```text
selected outer record completes
6, 930, 931, 932, 933, 16, 7
full computed child bound is reserved
10, ... child work, selected return/break/continue
```

Native probes observe the actual arguments of the original `extent`, without
replacing its computation. The reached bound is computed once, even though the
child condition is checked repeatedly. An inactive child never executes these
helpers, divides its poisoned divisor, or reserves its enormous bound.
The original branch mode's unselected raw poisoned arguments remain in source.

A separate reservation witness sets input trips to 7 and consumes an odd
completed State, producing width 3. With two outer units already reserved,
limit 4 rejects after bound completion/print 7 and before any child effect;
limit 5 succeeds. This distinguishes reservation of the complete computed
bound from charging only the iterations that happen before a local exit.
Existing huge, inactive and exact-bound witnesses remain regression coverage.

## Independent State And Failure Boundaries

The source oracle receives the actual State at a semantic bound hook after
the selected record update. It does not infer sites from numeric print markers
or generated guards. Earlier source modes keep their original division hook.
Multi-callback rows thread results from this mode, not from the older,
differently bounded program, while preserving each row's intended selection.
A control proves that changing completed-State parity changes both the number
of child iterations and the final State, with input trips held constant.

Failure controls separate aggregate argument completion, bound callee
completion and final bound arithmetic:

```text
bound aggregate argument: [..., 6, 930]
bound callee:             [..., 6, 930, 931, 932]
division/overflow:        [..., 6, 930, 931, 932, 933, 16]
```

No child effect or callback result may follow those failures. The MIN / -1
case deliberately establishes an even completed State after its accepted
prefix; an odd State would halve MIN first, making the division valid and the
subsequent enormous reservation a different failure. Reference sessions retain
the last accepted actual State bits after three callbacks, enter Faulted,
reject later work and refuse successful completion after close. Flushed native
trap probes validate the exact pre-failure effects, not memory after termination.

[Certificate controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_bound_call_tests.rs)
rename helpers/storage, deny bound helper and extent grants, retain immutable
source and pending identity, and reject type-valid aggregate bypasses,
argument swaps, changed guards and effect reordering. Candidate fallback and
original-invalid inner reads remain protected.
Grant denial is checked at its direct caller: removing `extent` invalidates
`bound_extent`, while removing `bound_seed` or `bound_extent` invalidates `work`.
An already-certified catalog entry is not silently treated as untrusted or
recursively reinterpreted by the storage certificate.
[Independent/shared execution](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_execution_tests.rs)
uses normal/reversed YIR and retains the four-word logical scoped carry
reduction. This is not measured physical memory or runtime speedup.

## Native And Source-Free Delivery

The new fixture explicitly declares 160 inclusive YIR function entries per
callback. The preceding branch mode keeps 128 and older modes keep 100;
production defaults/guards are unchanged. This bounded test configuration
counts generated helper entries, not only source functions.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/state_loop_return_shared_bound_tests.rs)
cover actual raw State words, local exits, computed reservations and selected
failures. [CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_shared.rs)
exercises fresh build, cache hit and two restorations after removing source,
manifest, project cache and original output. Completed-State-dependent work and
the four selected bound failures run through each image. Binary/container,
host-object, envelope, lifecycle and packaging identities remain unchanged.
Fatal CLI output is not substituted for the flushed native prefix proof.

## Remaining Frontier

The [sibling-bound follow-up](nuis-native-return-effect-storage-sibling-bounds-v1.md)
now owns executable checks for the boundary below and advances its own frontier
to a completed snapshot dominating the parent loop across backedges.

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next validate one completed selected bound record reused by sibling reached
child loops: each reached loop must independently reserve its full bound,
without recomputing record effects or reserving an unreachable sibling after
return/break/continue. Keep the current catalog, original/shared reads,
bounded candidates, fresh pending identity and exact independent fallback.

Resource/I/O/async capture, wider changing backedges, stable ABI, device and
cross-platform parity, complete heterogeneous closure and completed
self-hosting remain separate gates. These are local CPU witnesses only.

## Validation

Focused acceptance passes 336 distinct tests, including 11 new tests:
261 compiler units (359.42 seconds), 17 native/reference tests (379.69 seconds),
two CLI delivery tests (445.70 seconds), three host-fault tests,
24 repository-maintenance tests, two line-budget tests and 27 tensor tests.
The calibration probes are included in these suites, not counted twice.

Native acceptance covers the new bound mode, the preceding branch and loop
modes, and original parent-break compatibility, not all eight native modes.
Delivery covers eight fresh/cache/restored images across the new bound and
preceding branch representatives, not all eight CLI modes or all eight source
variants. Normal/reversed native and reference checks retain their stated
variant and failure matrices; source-free delivery remains representative.

The ordinary CLI rebuild and live tensor audit pass. All six source/current
CLI fields match, all six complete predecessor hashes verify, the selected
task card is ready, and coverage, hierarchy, lineage and inert history are
clean. Static drift validation passes 3,497 checks covering 23,985 required
patterns. Locked metadata contains 29 workspace packages. Formatting, UTF-8,
local documentation links, source/test line budgets and whitespace checks pass.
The source coordinate remains active/99; no release or version bump is made.

Initial probes corrected three test-construction assumptions, not production
lowering: transitive grant denial belongs at the direct caller; an inactive
child witness must first have enough outer reservation budget; and minimum
`i64` is expressed as `(-9223372036854775807 - 1)` in generated entry source
because the parser does not accept the overflowing positive literal beneath
unary negation. Native callback inputs still carry the actual minimum raw word.

The focused executable routes are:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- iteration_effects control_loops scalar_helpers scalar_control state_loop --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_loop_return_shared::bound_tests:: state_loop_return_shared::shared_return_storage_branch_helper_calls state_loop_return_shared::shared_return_storage_loop_helper_calls state_loop_return_parent_mixed_keeps_preceding_parent_break_native_profiles_unchanged --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop_return_shared::branch_bound_calls effect_calls::state_loop_return_shared::shared_return_storage_branch_helper_calls --test-threads=1 --nocapture
```

Each of the six current tensor fields retains its complete predecessor and
SHA-256. Historical commands remain inert inside a quoted here-document.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `a571966e71fc8505ef6ca6a7280cfd5e62f55328d76f439ccf475c47ec6ba221` |
| `next_step` | `6ddff017ecd83cb9af68b117457abb20daf06251698ced0aea361c33eea8b6c0` |
| `blocker` | `59cf29c361da98e81f1a1559ce8f6ea1d15fd973db352993806332325d870b15` |
| `next_action` | `9c90ec733c1a5709876eb0e9f167de8d8c30a3911811be68f6f98d26a8e4714f` |
| `validation_command` | `889d4700e051eb83e26ba83a4d2ed9073b8ef033f23fb79d19d439bb060dd6f9` |
| `expected_artifact` | `7a0b6c0b550aea60d06c720551f1f86be8170855d21ad0288fe126e44508fbc8` |
