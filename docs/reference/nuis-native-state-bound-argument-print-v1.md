# Native State Bound Argument Print V1

Audit date: 2026-11-04. Checkout baseline: `dc87ec43`; latest version-label
commit: beta-0.18.4, `ceaa2caa`. This extends the
[fallible-bound receipt](nuis-native-fallible-bound-argument-print-v1.md).
No release, version bump or commit is created here.

## State-Dependent Child Bounds

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/state_bound_argument_print.ns)
passes `total < 0.0` to the checked bound producer on every reached parent.
Completed children negate the carried total. With base three, a positive
callback starts with schedule 0/3/0/4 and accepts a negative State; the next
callback consumes that accepted State and runs 0/4/0/4 despite its positive
initial-value decoy. A stale callback-entry snapshot would instead produce
0/3/0/3 for the first callback.

The producer consumes an ordinary ready scalar predicate, not a new record
capture permission. The existing parent identity update establishes readiness
before the comparison; child record fields retain their original epoch rules.
Zero child work still evaluates a reached producer. A base-zero callback that
starts negative completes one child, becomes positive and reaches a later
zero-child producer that can itself fail.

## Lowering Fix

The first source-admission run rejected the new witness: the scoped loop
expression validator allowed integer comparisons and float arithmetic, but not
comparisons between ready float values. The
[validator](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs)
now admits all six comparisons on same-width f32/f64 operands after the existing
recursive type and readiness checks. It does not invent a new float operator
or bypass helper, scope, epoch or work-budget validation.

The [source controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/parent_reentry_argument_print_tests.rs)
cover all six operators, both widths and reversed declarations. Direct checks
retain unready sibling/record rejection, legal self-update reads, mixed-width,
integer/bool operand rejection and unsupported float remainder/XOR rejection.
Existing branch/alias/child-mutated-bound refusals remain. Established IEEE
comparison/print-epoch tests are included in the selected regression set.

## Model And Execution

Each f32/f64 kind has 27 raw sequences, eight finite sequences and 14 fault rows.
The [model](../../tools/nuisc/tests/native_application_bridge/state_bound_argument_print_cases.rs)
decodes IEEE sign and magnitude rather than evaluating the source comparison.
Negative zero and signed NaNs are not less than zero; negative subnormals and
infinities are. Modeled child updates toggle raw sign bits, while parent entry
alternates independently of source modulo arithmetic.

The [shared driver](../../tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print.rs)
now supplies the previous accepted State to helper-count expectations. Counts
must not use an event's decoy initial value. Marker 602 observes real producer
base, parent, guard and boolean arguments, including the changed predicate on
later backedges. The twelve-word callback profile is unchanged from the
fallible-bound fixture; older eleven-word profiles keep their own shape.

Faults cover all four producers, later-child initializer/lazy-use/update,
eager-false evaluation, zero-word lazy use, tail publication and a producer that
becomes zero-work after a completed child. Reference sessions check the last
accepted State through rejection and faulted close. Native fault probes check
exact flushed prefixes and absence of successful callback publication, not a
failed-buffer memory snapshot or rollback of earlier output effects.

## Exact Work And Delivery

The positive 0/3/0/4 schedule succeeds with eleven work units and rejects at ten
before its fourth-parent child body. The negative 0/4/0/4 schedule succeeds in
the twelve-unit matrix and rejects at eleven. Both reservation failures follow
actual producer evaluation. A checked producer failure at the corresponding
ten-unit boundary must occur first and emit neither budget diagnostic.
Separate eight/64-entry refusal controls remain under the 192-entry healthy
limit. No production budget is increased.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs)
checks fresh compilation, confirmed cache hits and two restores after deleting
source, manifest and project cache. Three-callback finite sequences expose
accepted-State reuse and decoys. Healthy runs surround faults, with State
reports and binary/container/host-object/envelope/lifecycle/packaging identities
checked. Trapping CLI output is allowed to be a buffered prefix; direct native
probes require the exact flushed trace.

## Validation

The final selected suite passed 148 distinct tests/checks:

| Selected Set | Passed |
| --- | ---: |
| Source admission, ready-value/epoch and existing IEEE comparison regressions | 42 |
| Native/reference/oracle profiles, IEEE comparisons and literal-print budget control | 34 |
| State-bound and fallible-bound f32/f64 delivery matrices | 2 |
| Python CI-script regressions | 40 |
| Repository file budgets | 2 |
| Host-absolute-path policy | 1 |
| Development tensor tests | 27 |

Eight controls are new: three source/type tests, four native/reference/oracle
controls and one CLI matrix. The first rejected source run exposed the missing
comparison admission and is not counted as a pass. Preliminary drift matching,
repeated runs, matrix rows and historical receipts are not added to the total.
The two CLI matrices completed in 646.35 seconds. Older parent/child profiles
were rerun in native/reference tests, not all of their CLI delivery matrices.

Structural checks passed for 6,027 UTF-8 text files, 4,737 local Markdown links,
locked offline metadata, formatting and whitespace. The architecture remains
29 packages, six domains and 74 normal/build edges. The proof index stays at
300 lines, the drift registry at 796 and the shared native test driver at 750.
No broad unselected library suite, Linux run or GPU execution is claimed.

The rebuilt ordinary CLI passes all 4,220 drift checks, retaining all 4,209
preceding IDs and adding eleven checks. All six source fields match the runtime
report and preserve complete predecessor values and SHA-256 hashes. Coverage,
hierarchy and task lineage are clean; mainline selection remains active/99.
Historical commands pass syntax and inertness checks through stdin without
executing their contents.

## Scope And Next Gate

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. These are bounded host CPU witnesses, not general record captures,
resource/I/O/async support, stable ABI, Linux/GPU parity or native-engine closure.

Next combine these state-dependent bounds with an early child-loop exit.
Check the carried value and effect order after that exit, and distinguish
reserved work from completed iterations without weakening the existing gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_bound_argument_print fallible_bound_argument_print parent_bound_argument_print child_progress_argument_print parent_reentry_argument_print epoch_local_ child_print_epoch local_scalar_argument_print loop_bool_projections float_compare_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- parent_reentry_argument_print:: epoch_local_argument_print:: float_compare_argument_print_epoch:: sibling_bounds_native_ready_updates_fit_160_but_entry_limit_100_still_rejects --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- parent_reentry_argument_print::state_bound parent_reentry_argument_print::fallible_bound --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete fallible-bound fields remain with the hashes below. Historical
validation commands are quoted no-op here-document data, checked through stdin
for syntax and inertness without replaying their contents.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `cd5727d18f16c3cd5a48aad2b43451b5d27d44c04151fd645432d675071b7e74` |
| `next_step` | `bc3f926044f4ce6e9d3c8e85e6531815e2615a0f01698cfc7a6ab12af18161ba` |
| `blocker` | `3e573ef791bf7c202ea23d16f03575e216a1d5787da11f9371e58d54b0064daa` |
| `next_action` | `2f40904bf0d20e38ccefa586a46f03cbeba7f6eae53ddcd6d0f9d80fa4671281` |
| `validation_command` | `7849be339cba0e0c877af85c25ab12374f3ff1d43919550cdcc38f71bee75e4a` |
| `expected_artifact` | `5cb577098acd210899e2d87f87f7d19f42db0cb7bc6afba487e6b0a71ee8d745` |
