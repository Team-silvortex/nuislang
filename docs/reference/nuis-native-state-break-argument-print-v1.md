# Native State Break Argument Print V1

Audit date: 2026-11-04. Checkout baseline: `dc87ec43`; latest version-label
commit: beta-0.18.4, `ceaa2caa`. This extends the
[state-bound receipt](nuis-native-state-bound-argument-print-v1.md).
No release, version bump or commit is created here.

## State-Bounded Early Breaks

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/state_break_argument_print.ns)
adds a cut index to the state-dependent child loop. A reached child completes
its checked initializer, observer and State update before testing the cut.
A taken break emits marker 78 and skips marker 79 and the induction step.
Every parent observes the actual final child index, including zero-work parents.
Later bound producers consume the actual carried State, not the reserved count
or a callback-entry snapshot.

For a positive initial State, cut zero reserves 0/3/0/4 children but completes
0/1/0/1. Cut one instead reserves 0/3/0/3 and completes 0/2/0/2. These witnesses
separate changed State from merely reserved work. Negative and out-of-range
cuts retain full traversal. A cut on the last iteration still skips that
iteration's suffix and leaves its index unadvanced.

## Lowering Fix

The initial source-admission test rejected the witness. Control-flow
normalization prepends generated flag initializers, while the epoch-local
initializer plan permits only literal prints before its definition/observer
pair. Treating the generated flags as source declarations hid a legal pair.

The [normalizer](../../tools/nuisc/src/lowering/buffer_loop_outline/control_flow.rs)
now records its exact generated entry prologue. The
[entry validator](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/epoch_block_owner.rs)
requires the original normalized block identity and unchanged prologue,
validates that prologue through ordinary statement rules, then validates the
remaining entry body with its fresh epoch owner. Root and child routes share
this handling; actual lowering retains all generated statements.

This is not a name-prefix exemption or general permission for source-local
initializers. Copied/sliced blocks, changed or missing generated initializers,
source declarations resembling generated flags and branch-moved pairs remain
rejected. Mixed break/continue normalization also checks both generated flags.
Ordinary branch recursion receives no owner, and borrowed record-field reads
do not become ordinary availability.

## Model And Execution

Each f32/f64 kind has 27 raw sequences, eight finite sequences and 15 fault rows.
The [schedule model](../../tools/nuisc/tests/native_application_bridge/state_break_argument_print_cases.rs)
records requested and completed children separately and toggles the raw carried
sign only for completed updates. The existing IEEE model distinguishes signed
zero, NaNs, infinities and subnormals. The thirteenth callback input is the cut;
the earlier twelve inputs and older profiles retain their shape.

The [oracle controls](../../tools/nuisc/tests/native_application_bridge/state_break_argument_print.rs)
pin indices and markers for first, middle, last, absent and out-of-range cuts.
The shared helper-count model counts completed work, while producer counts
include every reached parent, even if its child count is zero. Three-callback
sequences consume accepted State rather than an initial-value decoy.

Poisoned later-child checks must remain inert after a break. Reached producer,
initializer, lazy-use, update and tail faults retain their exact effect prefixes;
a reached failing update occurs before its break. Reference sessions retain
last accepted State through rejection and faulted close. Direct native probes
check exact flushed output and no successful publication, not a failed-buffer
memory snapshot or rollback of already emitted effects.

The expanded legacy exit regressions exposed a separate probe mismatch: literal
print suites already mixed source faults and expected reservation rejection.
Their exact-prefix contract is preserved, while selected old reservation tests
now require the exact per-row rejection kind. Counts above 65,536 must fail
induction admission before reservation; smaller over-budget counts must emit
the work-limit diagnostic. New computed-print source faults reject both of
those diagnostics and helper-entry exhaustion. A trap alone cannot stand in
for the intended strict failure. Runtime budgets are unchanged.

## Reservations And Delivery

Early exits do not refund reserved work. The positive cut-zero schedule succeeds
with eleven work units and rejects at ten before entering the final parent's
child body, even though only two children would complete. Negative cut-one
retains the twelve/eleven boundary. Producer failure at that boundary precedes
reservation failure and emits neither budget diagnostic. Separate eight/64
helper-entry refusal controls use a full-traversal row under the unchanged
192-entry healthy limit.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs)
checks fresh compilation, confirmed cache hits and two source-free restores
after deleting the source, manifest and project cache. Healthy sequences run
before and after fault probes. State reports and binary/container/host-object/
envelope/lifecycle/packaging identities must agree across delivery routes.
Trapping CLI output may be a buffered prefix; direct native probes require
exact flushed prefixes.

## Validation

The final selected suite passed 164 distinct tests/checks:

| Selected Set | Passed |
| --- | ---: |
| Source admission, normalization provenance, epochs and existing exit/comparison regressions | 48 |
| Native/reference/oracle profiles, old child/exits and budget controls | 44 |
| State-break and state-bound f32/f64 delivery matrices | 2 |
| Python CI-script regressions | 40 |
| Repository file budgets | 2 |
| Host-absolute-path policy | 1 |
| Development tensor tests | 27 |

Eight controls are new: two source tests, one normalization-origin test, four
native/reference/oracle controls and one CLI matrix. Existing root-source
admission and two legacy reservation tests were strengthened. Initial failed
source and probe-classification runs are not counted as passes; repeated runs,
matrix rows and historical receipts are not added to the total. The final
44-test native/reference run completed in 364.08 seconds; the two delivery
matrices completed in 567.27 seconds. Older profiles were rerun through direct
native/reference controls, not every older CLI delivery matrix.

Structural checks passed for 6,032 UTF-8 text files, 4,747 local Markdown links,
locked offline metadata, formatting and whitespace. The architecture remains
29 packages, six domains and 74 normal/build edges. The proof index stays at
300 lines, the drift registry at 799 and the shared native test driver at 760.
No full-workspace suite, Linux run or GPU execution is claimed.

The rebuilt ordinary CLI passes all 4,233 drift checks, retaining all 4,220
preceding IDs and adding thirteen checks. All six source fields match the runtime
report and preserve complete predecessor values and SHA-256 hashes. Coverage,
hierarchy and task lineage are clean; mainline selection remains active/99.
Historical commands pass syntax and inertness checks through stdin without
executing their contents.

## Scope And Next Gate

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. These bounded host CPU witnesses do not establish general record
captures, resource/I/O/async support, stable ABI, Linux/GPU parity or a complete
native engine.

Next combine state-dependent child bounds with `continue`: prove the required
single induction step, skipped suffix, later State and reservation behavior
without granting branch-local epoch ownership.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_break_argument_print state_bound_argument_print fallible_bound_argument_print parent_bound_argument_print child_progress_argument_print parent_reentry_argument_print epoch_local_ child_print_epoch local_scalar_argument_print loop_bool_projections float_compare_argument_print_epoch iteration_effects_exits --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- parent_reentry_argument_print:: epoch_local_argument_print:: float_compare_argument_print_epoch:: state_loop_children:: state_loop_exits:: sibling_bounds_native_ready_updates_fit_160_but_entry_limit_100_still_rejects --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- parent_reentry_argument_print::state_break parent_reentry_argument_print::state_bound --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete state-bound fields remain with the hashes below. Historical
validation commands are quoted no-op here-document data, checked through stdin
for syntax and inertness without replaying their contents.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `6d4a0eb747c0453457621b561964e88e0e9946f2e4fd3c37eb744003b9f663dd` |
| `next_step` | `594251c5050565275095326606e180181d24e1a98c4edb513b8efbc911882374` |
| `blocker` | `8d0eb55504bd9d916061d00de00a8c88a6d8c8ecaa514dea8cc55347c891ccf3` |
| `next_action` | `c2eff05aa298f8834ac63c11adde11721a7b0123e030f7495166d92a26afd7ac` |
| `validation_command` | `02981c9c3e65241648dfb04f6510c50f4ab709df30ff4c8aa6f5b94fc42e8657` |
| `expected_artifact` | `7f0654dc8ae82f3ebad367f8da42ac58eabe274cb2f8d3322ac95aa13350646f` |
