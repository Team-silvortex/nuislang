# Native State Control Exit Argument Print V1

Audit date: 2026-11-07. Checkout baseline: beta-0.18.6, `696d2bd7`.
This follows the worktree [record-parameter checkpoint](nuis-native-state-record-parameter-exit-argument-print-v1.md).
It creates no version bump, commit, tag or release.

## Persistent Control State

The [independent Nuis witness](../../tools/nuisc/tests/native_application_bridge/state_control_exit_argument_print.ns)
publishes `State { total: f32/f64, control: i64 }`. Every completed child update
flips the total's sign and increments control with full-width wrapping addition.
The updated control XORs the confirmation mode base and adds to the guard base
before the existing field-dependent parameter producers run. Parent backedges
and subsequent callbacks consume both accepted fields, not fresh event inputs.

The control carry has its own parent-entry update before creating the child
record. Removing that update is a source-admission refusal, not an excuse to
inherit readiness from the adjacent float carry. Both fields remain unavailable
to ordinary exit arguments before the child record update. The
[source controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_control_exit_argument_print_tests.rs)
retain original field/owner/type and exit-step boundaries in both float widths
and normal/reversed declarations. No production compiler rule is widened.

The new callback input appends initial control and a separate tail-check guard
to the unchanged 35-word predecessor prefix. Open consumes 37 words; event
consumes the accepted two-word State plus those 37 words; close consumes two
State words. Every callback publishes exactly two output words. These fit the
existing scalar transport limits without changes. Predecessor one-word fixtures,
models and delivery drivers remain separate and unchanged by this follow-up.

## Independent Models And Complete Publication

The [model](../../tools/nuisc/tests/native_application_bridge/state_control_exit_argument_print_cases.rs)
contains a separate successful schedule and checked trace executor. The schedule
predicts State-dependent child requests, completed children, observed exit
indices, predicate counts and both State words. The executor reuses the proved
single-child word oracle and field-parameter evaluation, but propagates the
control field independently through each child, parent and callback.

There are 24 raw sequences and 30 fault rows per width. Healthy cases include
zero work, signed zero/NaN/infinity/subnormal float payloads, wide and negative
controls, and i64 boundary wrapping. The first accepted prefix changes both
fields to negative total/control minus one; the next publishes positive total
and control zero despite negative-float and high-control event decoys.

A dedicated early-break profile requests `[0, 3, 0, 4]` children but completes
`[0, 1, 0, 4]`, publishing negative total/control four from initial control minus
one. The following callback completes eight children and publishes control twelve.
The control field counts completed updates, not reserved iterations, and neither
exit refunds the reservation. This profile has its own source-free delivery check.

After the last parent, the float tail helper runs first. A literal marker then
precedes an independent control-tail division check. Failure of the float tail
ends before that marker; failure of the control tail occurs after the float
result is computed but before either returned State field is published. The
native bridge extracts both returned words before its first output store.
This is complete publication at a synchronous callback boundary, not a hardware
atomicity or concurrent-memory guarantee.

## Rejections, Failures And Close

[Native/reference probes](../../tools/nuisc/tests/native_application_bridge/state_control_exit_argument_print.rs)
check both words at every successful callback. Direct native drivers forward
the actual prior output words into the next event and use canaries around the
complete result. Exact trapping prefixes never inspect pending memory after a
native trap. Source faults cover bound production, record update, both tails,
and all six ordered exit-argument/check stages in first and later parents,
including positive/negative zero divisors.

Reference sessions reject a wrong-type event control input without leaving
Open or changing either accepted field; a valid event still succeeds afterwards.
A callback execution failure faults the session and retains both accepted words
through rejected later events and close. A separate reference close variant
computes changes to both fields and then fails its control-dependent check:
neither pending field replaces accepted State, repeated close returns the stored
error, and completion remains unsuccessful. Its healthy counterpart publishes
both changed fields. This extra close check is reference evidence, not a claim
that a process-trapping native close can be recovered.

The [raw preflight test](../../tools/nuisc/tests/native_application_bridge/state_control_exit_preflight.rs)
checks wrong input/output lengths, null pointers, noncanonical bool/i32 and f32
words on the actual new callback exports. Failure returns the expected status
without touching either sentinel State word or adjacent canaries and without
entering the effectful callback. It also asserts field order and the 37/39/2
input widths. The existing nested/overlapping raw callback transport regression
is checked alongside this new profile.

## Accepted Control Versus Event Decoys

A source contrast replaces only event control with its input decoy while keeping
the accepted float. Correct code uses accepted control zero, updates to one,
and selects continue. The altered source instead updates decoy two to three,
skips that continue, and publishes control three. Native traces and both output
words distinguish the outcomes; merely preserving the float cannot pass.

All healthy native paths retain at most 12 reserved loop units and 192 helper
entries. The work model drives exact limit/limit-minus-one rejection checks;
entry limits eight and 64 still fail before status/State publication. A control
increment is currently total. Independently fallible control production is the
next gate, not claimed by the shared record-update check or the final tail check.

## Artifact Delivery

The [independent CLI driver](../../tools/nuis/tests/native_session_workflow/state_control_exit_argument_print.rs)
checks ten healthy sequences before and after 30 faults on fresh, cached and
two independently restored artifacts in each width. That matrix contains 400
artifact executions. The early-break profile adds eight executions over the
same four delivery paths and two widths: 408 artifact runs in total.

Source, manifest and project cache are removed before restoration. Both State
fields, output, completion status and binary/container/host-object/envelope/
lifecycle/packaging identities are checked. Earlier accepted State lines remain
unchanged after a failed event. Direct native probes own exact trapping prefixes;
CLI stdout can be buffered. Non-finite payload evidence remains in raw native
word transport, not finite CLI serialization.

## Validation

Selected checks for the independent two-word profile and its direct predecessors:

| Selected Suite | Result |
| --- | --- |
| New and preceding record source/refusal controls | 4 passed |
| New profile, preceding record and raw callback transport | 15 passed |
| Main and early-break CLI delivery matrices | 2 passed |
| Tensor and complete-predecessor lineage tests | 33 passed |
| Script unit tests | 40 passed |
| Repository file budgets | 2 passed |
| Host absolute-path policy | 1 passed |

Total: 97 distinct selected tests, including twelve new controls (two source,
seven native/reference, two CLI and one tensor). The combined native run took
125.94 seconds; all seven new profile tests were rerun after adding the early-break
sequence and passed in 52.66 seconds. The unchanged main CLI matrix completed
400 artifact runs in 890.50 seconds; the separate early-break matrix completed
eight runs in 36.74 seconds. Initial failures, subsets and reruns are not added
again. The scope is narrower than the preceding shared-driver audit because the
new State profile has independent drivers. This is not a full-workspace or
multi-platform run and these timings are not performance claims.

The rebuilt ordinary CLI reports 4352 passing drift checks and zero failures.
All 4340 predecessor IDs remain in order alongside twelve new checks. Coverage,
hierarchy and task lineage are clean; six source/runtime fields agree and retain
the complete preceding values/hashes. Historical command syntax and only the
quoted inert suffix were checked without replaying old validation chains.

UTF-8 validation covers 6096 tracked files; 4857 local documentation links resolve.
Formatting and whitespace checks pass. No production admission, runtime limit,
dependency or active/99 scope change accompanies this checkpoint.

## Scope And Next Gate

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next make the in-loop control-field update independently fallible, preserving
source order after float work and before exit arguments. Verify first/later
parent failures, rejected partial record computation, accepted multi-field State,
decoys and source-free restoration under the same quotas and read/effect rules.

This is bounded host CPU evidence, not arbitrary resource/I/O/async capture,
pre-update ordinary reads, Linux/GPU parity, a stable ABI, arithmetic NaN payload
guarantees, full native-engine closure or completed self-hosting.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_control_exit_argument_print state_record_parameter_exit_argument_print --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_control_exit state_record_parameter_exit_argument_print callback_values_preserve_nested_bits_overlap_and_failure_sentinels_without_allocating --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- state_control_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test file_line_limit --locked -j1 -- --test-threads=1
python3 -m unittest discover -s scripts/tests
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-host-absolute-paths.sh
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
cargo fmt --all -- --check
git diff --check
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --locked -j1
target/debug/nuis dev-tensor --json
```

## Predecessor Lineage

All six complete record-parameter values are retained. The
[persistent lineage regression](../../tools/nuis/src/dev_tensor_native_argument_lineage_tests.rs)
checks their full SHA-256 values; earlier checkpoints retain their own guards.
Historical validation commands are quoted no-op here-document data. Validate
syntax through stdin and execute only the inert suffix, never historical chains.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `ad64c9caf53f604ba8a8dbc0c608976155bb9ec080eed2dfd18a575ece4793c8` |
| `next_step` | `d25af46d361764de5fc7af28c6845aa518e093ba9235142486ca22ada8edcac2` |
| `blocker` | `119d7ce1b42c1b708ed83461347837e026cfb1e77f5319ee92cc8c34c24c6268` |
| `next_action` | `209bb36dcde337306cfe2cc09c060f6dd6af69312bfe73c100da8cdac82ab0bf` |
| `validation_command` | `331d8edb3c6992a2e8a66d583bc403d0b64bd3d15b43c8a4806791172cfb496c` |
| `expected_artifact` | `e1b7e8714f40d6ecab0737acc3f0bb0243099c34e511cd40cb3101500747759d` |
