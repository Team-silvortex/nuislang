# Native State Parameter Exit Argument Print V1

Audit date: 2026-11-07. Checkout baseline: beta-0.18.6, `696d2bd7`.
This continues the uncommitted [helper-exit checkpoint](nuis-native-state-helper-exit-argument-print-v1.md)
without replacing its evidence or changing committed release membership.
No version bump, commit, tag or release is created here.

## Ordered Later Confirmation Arguments

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/state_parameter_exit_argument_print.ns)
adds independently checked i64 producers to the second and third arguments of
the grouped continue/break confirmation helpers. `continue_mode_value` and
`continue_guard_value` follow the continue quotient; the corresponding break
helpers follow its quotient. Each producer checks its own fault input before
returning its value. The guard value and the guard producer's fault input remain
separate: returning zero can succeed before the confirmation itself rejects it.

The selected expression now distinguishes six failure stages: left float
producer, right float producer, division, mode producer, guard producer and
confirmation check. A failure skips every later stage. The surrounding grouped
AND/OR and continue/break control still govern whether any arguments are reached.
No new source, epoch-read or native effect permission is introduced.

[Source controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_parameter_exit_argument_print_tests.rs)
cover exact i64 producer results and inputs, bool/i32/float mistakes, both
premature exits and inherited owner/alias/step refusals. Both float widths and
normal/reversed declarations are exercised. An earlier print-only read grant
cannot authorize a confirmation argument before the record update.

## Value, Call And Failure Models

The [parameter profile](../../tools/nuisc/tests/native_application_bridge/state_parameter_exit_argument_print_cases.rs)
reuses the established division/State schedule for successful value-preserving
producers. Its separate lazy trace model completes division before recording
mode entry, then records guard entry before confirmation. Healthy source calls
are counted independently for each producer. Predecessor profiles retain zero
calls to the four new parameter helpers.

Each width covers 31 raw sequences, eight finite sequences and 34 fault rows.
The raw matrix retains the predecessor IEEE operand classes and adds modes
`4294967296`, `i64::MIN` and `i64::MAX` with NaN inputs. Arithmetic NaN quotient
observations alone are normalized; actual producer input/return values and
State payloads are not rewritten by instrumentation.

All 24 helper-profile fault rows are retained. Later parameter faults are also
poisoned where an earlier producer or division already fails, establishing that
they remain unentered. Ten new rows cover mode/guard failures for both exits in
first and later parents, plus fallback-selected breaks. Mode and guard faults
can collide with the confirmation check. Later-parent rows retain seven completed
updates; reference sessions preserve the last accepted positive State rather
than negative callback decoys, including after faulted close.

The input layout appends four i64 producer-fault controls to the unchanged
31-word helper prefix, for 35 words. This stays within the existing 64-slot
transport bound. Earlier profiles retain their layouts; there is no stable ABI
claim or expanded scalar-slot limit.

## Independent Native Controls

[Native probes](../../tools/nuisc/tests/native_application_bridge/state_parameter_exit_argument_print.rs)
record quotient producer entries, complete 64-bit mode/guard inputs, and
confirmation entry separately. They check flushed prefixes without reading
pending output memory after a trapping callback.

A parameter-order control swaps the second/third call arguments together with
the confirmation's parameter declarations. Healthy semantics and State stay the
same, but the entry order and raw confirmation argument positions change. When
both producers fail, the newly earlier guard producer wins. This isolates source
parameter order rather than merely changing a target value.

Return-value controls XOR bit 32 into each producer's source input and separately
XOR it back in the producer's return. The entry word changes, but the final
confirmation value must remain exact. Separate mode and guard runs include
finite/zero/NaN states, high and signed modes, and zero guard returns that must
still fail at confirmation. Substituting a producer input for its returned value
or truncating its i64 word cannot satisfy these observations.

An effect-admission control adds print to the four new producers. Explicit grants
for every selected site permit native emission; omitting any one site rejects it
while other grants remain. Existing iteration-print shape admission is not an
inherited grant. This is an admission check, not execution evidence for a new
effectful profile.

## Update Checks And Unchanged Quotas

Initial validation found that retaining all wrapper calls exceeded the existing
192-entry limit. Pure child-count rewrites did not close that budget, and a fused
print classifier did not fit the original read-admission route. Those exploratory
forms were discarded; neither was used to relax admission or accounting.

The final witness instead spells out the record-update wrapper's existing
`0 / ready_guard` check in the loop immediately before the record assignment.
The field's sign negation remains total and helper-free. The check still occurs
before any updated record or exit argument can be observed, and its failure
retains the original printed prefix. No potentially failing operation is removed
or moved past the update. Parent and final `finish` calls remain in place;
reference call counts distinguish them from the now-inlined update check.

A negative native control restores the original update wrapper and verifies its
entry exhaustion at 192. The new healthy profile retains the same 192-entry
limit; limits eight and 64 remain separate rejections. Work reservations stay at
most 12 with exact 11/10, 12/11 and 10/9 accept/reject controls. A later poisoned
parameter and zero divisor still lose to the earlier parent reservation failure.
No exit refunds work, and no runtime accounting or production compiler rule is
changed by the source-level wrapper expansion.

## Artifact Delivery

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs)
runs eight healthy sequences before and after 34 faults on fresh, cached and
two independently restored artifacts in each width: 400 artifact runs. Source,
manifest and project cache are removed before restoration. State, stdout,
completion status and binary/container/host-object/envelope/lifecycle/packaging
identity are checked. Exact trapping prefixes belong to direct native probes;
CLI stdout may be buffered. Non-finite payload evidence remains in word probes,
not finite CLI serialization.

## Validation

Final selected runs in this checkout:

| Selected Suite | Result |
| --- | --- |
| Compiler source/refusal regression | 66 passed |
| Shared parent-reentry native/reference regression | 72 passed |
| Later-parameter CLI delivery matrix | 1 passed |
| Tensor and complete-predecessor lineage tests | 31 passed |
| Script unit tests | 40 passed |
| Repository file budgets | 2 passed |
| Host absolute-path policy | 1 passed |

Total: 213 distinct selected tests, including eleven parameter-exit/lineage tests
(two source, seven native/model, one CLI and one tensor). Preliminary failures,
focused subsets and reruns are not added again. Native/reference execution took
543.28 seconds; delivery took 840.46 seconds with all 400 artifact runs. The
reference call-count assertion retained its predecessor expression explicitly;
both the old and new reference paths were then rechecked. These are functional
results, not performance measurements, a full-workspace run or another-platform
acceptance claim.

The rebuilt ordinary CLI reports 4328 passing drift checks, zero failures and
all 4316 predecessor IDs retained in order alongside twelve new checks. Coverage,
hierarchy and task lineage are clean. All six source/runtime fields agree and
retain their complete predecessor values/hashes. Historical command syntax and
only the quoted inert suffix were checked without replaying old validation chains.

UTF-8 validation covers 6082 tracked files and local-link validation checks 4836
links. Formatting and whitespace checks pass. Dependencies, production admission,
work/entry/transport limits and the active/99 scope marker are unchanged.

## Scope And Next Gate

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next derive mode/guard values from updated record State instead of invariant
parameter inputs. Preserve producer order, returned-value consumption,
parameter/confirmation fault priority and State-dependent selected paths across
parent backedges and accepted callbacks, with the same quotas and authority.

This is bounded host CPU evidence, not arbitrary resource/I/O/async capture,
pre-update ordinary reads, Linux/GPU parity, a stable ABI, arithmetic NaN payload
guarantees, full native-engine closure or completed self-hosting.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_parameter_exit_argument_print state_helper_exit_argument_print state_division_exit_argument_print state_field_exit_argument_print state_grouped_argument_print state_short_circuit_argument_print state_predicate_argument_print state_mixed_argument_print state_continue_argument_print state_break_argument_print state_bound_argument_print fallible_bound_argument_print parent_bound_argument_print child_progress_argument_print parent_reentry_argument_print epoch_local_ child_print_epoch local_scalar_argument_print loop_bool_projections float_compare_argument_print_epoch iteration_effects_exits --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- parent_reentry_argument_print:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- parent_reentry_argument_print::state_parameter_exit --test-threads=1 --nocapture
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

All six complete helper-exit values are retained, not summaries. The
[persistent lineage regression](../../tools/nuis/src/dev_tensor_native_argument_lineage_tests.rs)
checks these full SHA-256 values while older checkpoints keep their own guards.
Historical validation commands remain quoted no-op here-document data. Check
syntax through stdin and execute only the inert suffix, never old command chains.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `2d873db1233bf9d11e1c4dd382324f202bcc7d311fc0885c6721de7f846df1c3` |
| `next_step` | `7d387bb7117daf185d5b4627b9b57e4b3decc71ef777a8db27f3c58571552de6` |
| `blocker` | `38797a0a7b107bb5a38a5eac8a388d26faad84c7c0b5893ee73c7fcca0ad56d0` |
| `next_action` | `18680433a47202e69b559d368fa950dc1fab4835f1e30a478acbbf3b4394da1f` |
| `validation_command` | `6cd903b22989f87f96421f9481c8388bdd85d181b4cec1ab29b58df93630dfe4` |
| `expected_artifact` | `b0b5fb87120e78604158c7aaf4b849195db4aafcf659e31af55ada211c2d07c3` |
