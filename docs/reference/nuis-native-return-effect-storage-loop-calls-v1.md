# Native Return Storage Reached-Loop Helper Assignments V1

This bounded CPU follow-up extends the
[prefix helper-call witnesses](nuis-native-return-effect-storage-helper-calls-v1.md).
The inspected Git identity remains beta-0.17.4 at bef7a23d. No release is made.

## Real Reached-Loop Updates

The sixth fixture mode, `LoopHelperCalls`, composes the preceding prefix calls
with five actual source record updates: outer iteration entry, inner iteration
entry, the unselected child branch, and the parent-local break/continue paths.
These replace existing `carry.total + delta` assignments rather than inserting
identity-only work. Existing bool/float updates, guards, local exits, steps and
return construction remain in their original order.

The [source adapter](../../tools/nuisc/tests/native_application_bridge/state_loop_return_shared_loop_calls.rs)
inserts real [Nuis helper declarations](../../tools/nuisc/tests/native_application_bridge/state_loop_return_shared_loop_helpers.ns)
into the generated source module. This `.ns` file is a declaration fragment,
not a standalone application. Its four `@noinline` helpers capture the aggregate,
evaluate ordered scalar arguments and return the updated bool/i64/f32/f64 record.
Successful call effects are `[810, 811, 814, 812, 813]` at each reached site.
No C/Rust application wrapper implements these calculations; Rust remains the
test oracle and harness, not the Nuis program's helper implementation.

Existing typing, effectful-call admission and independent/shared storage
certificates already admit the program. No production call, purity, ownership,
GLM, loop-budget or storage authority is widened. The production module edit
only registers tests. The declaration-order candidate cap and independent
fallback remain unchanged.

[Certificate controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_loop_call_tests.rs)
cover eight step/guard variants and three storage names with renamed helpers.
They retain immutable source and independent pending identity, deny missing
callee grants and reject type-valid deleted, duplicated, reordered, argument-
swapped or bypassed loop calls. Failed-first/later-successful candidate selection
still composes with the calls; an original inadmissible inner read cannot gain
permission through sharing.

[Execution comparisons](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_execution_tests.rs)
compare independent/shared actual entry values, effect order and selected
failures in normal/reversed YIR storage. The four-word maximum scoped carry
reduction remains logical YIR evidence, not measured physical memory or runtime speedup.

## Semantic Oracle Events

The independent source oracle now offers a test-only assignment-site hook at
outer/inner/unselected-child/parent-exit updates. The preceding entrypoint
delegates with a no-op hook, retaining all previous modes. The hook runs at the
actual source semantic point, before the corresponding record update, without
altering dataflow, reservation order, local exit precedence or return handling.

This matters because native argument observers can print raw words `8`, `9`
or `10`, which are also source print markers. Scanning a flat numeric trace and
expanding matching numbers would fabricate helper calls. The new mode instead
uses semantic hooks; calling its old flat-trace adapter is explicitly rejected.
An oracle control and additional native/reference sequences use observer raw
words that collide with these markers. Observed and unobserved traces retain
the same number of real helper invocations, without synthetic work.

## Later Iteration Failures

The inner assignment passes `root + 11 - index` to the second argument and
`after + 12 - index` to the callee. With `root = -9`, the second argument fails
at index 2. With `after = -10` and an ordinary root, all arguments complete but
the callee fails at index 2. Both leading/trailing step representatives complete
at least one inner assignment before that failure. The same program retains
healthy inactive-path and original selected failure controls.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/state_loop_return_shared.rs)
check these exact terminal suffixes in flushed native traces:

```text
later argument failure: [..., 10, 810, 811]
later callee failure:   [..., 10, 810, 811, 814, 812]
```

The argument failure skips the last argument and callee; the callee failure
skips completion, local exit handling and normal callback publication. After
three accepted callbacks, normal/reversed reference sessions retain their last
actual State, enter Faulted, reject further events and refuse successful
completion after close. Native process-trap checks observe flushed prefixes
and absent normal publication, not memory after process termination.
Full raw-word, selected-exit and full reached-bound reservation matrices also
cover the new source mode without weakening inactive-bound behavior.

## Source-Free Delivery

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_shared.rs)
adds one loop-helper representative to the five preceding modes. Each mode
runs fresh/cache/two restored images, with healthy runs before and after
selected failures and unchanged binary/container identity. The loop-helper
source additionally selects both later-iteration failures and the preceding
prefix-helper failures in each delivery image. It publishes no failed State
or successful completion marker.

Original source, manifest, project cache and original build output are removed
before restoration. Each program retains its own binary/container, host
objects, envelope, lifecycle and packaging mode; different programs need not
share equal binaries. Exact fatal stdout belongs to the flushed native probe,
not buffered CLI output. Source-free coverage is one new representative, not
the entire eight-variant matrix.

## Remaining Frontier

The later [branch-selected follow-up](nuis-native-return-effect-storage-branch-calls-v1.md)
provides witnesses for the task below; this receipt retains its historical
scope and counts.

The selected tensor coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next validate branch-selected record-helper updates inside nested loops where
different selected helpers modify bool/float/integer fields before local exits.
Require only the reached branch's effects, complete record handoff, actual
previous-State bits, selected failure and native/source-free replay. Diagnose
the existing route first, preserving original-read authority, independent
fallback and the current call catalog rather than adding blanket grants.

Resource/I/O/async capture, wider changing backedges, stable ABI,
device/cross-platform parity, complete heterogeneous closure and finished
self-hosting remain separate gates.

## Validation

This follow-up passes 338 distinct focused tests, eleven new: 251 compiler
units, 25 native/reference/reservation/failure/preceding-break compatibility
tests, six CLI delivery tests, three host-fault tests, 24 repository-maintenance
tests, two line-budget tests and 27 tensor tests. Standalone execution/mutation
probes are included in the final compiler suite, not added again. Earlier
receipts retain their own historical scope and acceptance counts.

The final compiler suite passes in 215.42 seconds, native suite in 564.97 seconds
and CLI delivery suite in 565.47 seconds on this M2. These are validation
durations, not runtime performance comparisons. All six modes retain their
values, ordered work, local exits, reservations and failure boundaries.
The six CLI representatives cover 24 fresh/cache/restored images total, four
for the new source; this is not eight-variant source-free coverage. The same
new program selects both later-iteration failures in every delivery image,
alongside the prior prefix-helper and selected loop/fallback failures.
The dedicated tail-failure cases belong to the full native/reference matrix;
the CLI image matrix selects `root` and `finish`, not a separate `tail = 0` case.

All 3,476 registered drift checks and 23,842 required patterns pass. The rebuilt
ordinary CLI exactly matches the six current source tensor values and their
complete predecessor hashes. Coverage, hierarchy and task-card lineage are
clean, with a ready active/99 task card for branch-selected multi-field helper
updates. Shell syntax and inert-history probes confirm that previous command
chains are retained as data. Locked metadata for 29 packages, formatting, UTF-8,
relative document links and line budgets pass, without enlarged exceptions.

Two fixture diagnostics were corrected without weakening production admission:
the source adapter initially assumed only one outer update, but the real source
also has parent break/continue updates, so all three are now replaced and
observed. The first structural negative control removed the first loop write,
which also invalidated a subsequent original read. The final control mutates a
later parent-exit write: deletion/duplication/reordering/argument changes still
pass ordinary admission, then fail the structural certificate. The original
read-admission assertion is retained, and the complete 251-unit suite was rerun
after the correction. No lowering authority or failed production case is hidden
by a relaxed gate.

No commit, push, tag or version bump is performed. Existing work is preserved.
No new Linux, Windows, GPU, full-workspace semantic, stable-ABI, physical-memory/
speedup or completed self-hosting acceptance is claimed.

The focused executable routes are:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- iteration_effects control_loops scalar_helpers scalar_control state_loop --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_loop_return_shared:: state_loop_return_parent_mixed_keeps_preceding_parent_break_native_profiles_unchanged --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop_return_shared:: --test-threads=1 --nocapture
```

Each of the six tensor values retains its entire preceding value with a
separate SHA-256 and explicit checkpoint marker. The previous command chain
is data in an inert here-document, not a repeated executable validation chain.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `4ae2b80ab1178d556a07086a920230153ea10a0b4093914270ac545252da65ab` |
| `next_step` | `04838480129d394c55fcfacf1d82573b26598a879e25e0cc5c136af52485d5f9` |
| `blocker` | `b9de78145572a4ca69d9dbac0dafe88685853d77e69954940350de6d68d1c158` |
| `next_action` | `c71b3c4f711a5545019115818e8d85ad893cc4139dd18219facbe8e09ab6f5ee` |
| `validation_command` | `4e534d4e48843fc1bf629ce1e2388d2fad5e680f3700f3d861fec34304900e7e` |
| `expected_artifact` | `0ea1bcc1ee789b61a89a4e9b45724895e7e93d2a15585135ffc57b0fd6052cba` |
