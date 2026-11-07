# Native State Helper Exit Argument Print V1

Audit date: 2026-11-05. Checkout baseline: beta-0.18.6, `696d2bd7`.
The [division-exit checkpoint](nuis-native-state-division-exit-argument-print-v1.md)
and its predecessors are now committed; their dated worktree receipts remain
unchanged. This follow-up creates no version bump, commit, tag or release.

## Ordered Checked Producer Operands

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/state_helper_exit_argument_print.ns)
wraps each grouped confirmation's numerator and denominator in independent
same-kind f32/f64 helpers. `continue_lhs` and `continue_rhs` produce the continue
quotient; `break_lhs` and `break_rhs` produce the break quotient. Each performs
its own checked integer operation before returning the float value.

The selected path has four distinguishable failure stages: left producer,
right producer, float division, then confirmation guard. A left failure skips
the right producer; either producer failure skips division and confirmation.
Zero division skips confirmation even when its guard is poisoned. Only the
surrounding AND/OR and continue/break selection can skip the whole expression.
Original updated-field readiness and accepted callback State remain required.

Raw native entry observers identify all four producers and record exact input
words and remaining guards. Source-reversed operand probes change both the
quotient and which colliding producer fails first. These are native entry
observations, not reference completion events or reconstructed event ordering.

## Returned Values And Independent Models

The [profile](../../tools/nuisc/tests/native_application_bridge/state_helper_exit_argument_print_cases.rs)
reuses the separately tested division schedule and width-specific IEEE oracle
because healthy producers are value-preserving. Its independent lazy trace
model inserts each producer before division and before the confirmation entry,
with distinct checked guards. Healthy reference calls are counted separately
for each producer; predecessor profiles must still make zero producer calls.

The matrix has 28 raw sequences, eight finite sequences and 24 fault rows per
width. It retains raw signed-zero/subnormal/infinity/NaN operand inputs. Only
arithmetic NaN quotient observations are normalized; producer inputs and State
payloads remain exact. Skipped poisoned producer pairs include zero-child
parents, false continue gates and unselected break confirmations.

The [return-value controls](../../tools/nuisc/tests/native_application_bridge/state_helper_exit_argument_print.rs)
negate a producer's input and separately negate its actual source return. The
original quotient and State must be restored, while the entry probe must show
the changed input. Separate left/right controls cover finite values, zero and
NaN, preventing an implementation from substituting the input word for the
returned value. Native observers never modify production inputs or returns.

Four retained fault rows cover earlier bound/update failures and reached
confirmation checks. Ten division-fault rows retain positive/negative zero and
fallback-selected failures. Ten additional producer rows cover left/right
failures at first and later parents, plus fallback selection. Later-parent rows
observe seven completed updates before the first selected producer pair fails.
Both producer guards, zero division and the confirmation guard can collide;
source order chooses the first failure rather than the apparently strongest
poison. No pending callback output is inspected after a native trap.

The accepted prefix first publishes negative State, then positive State. Failed
events carry negative initial-value decoys but retain the accepted positive
State after failure and faulted reference close. Four i64 guard inputs append
to the unchanged 27-word division prefix, producing 31 input words. Earlier
profiles retain their original layouts and behavior; this is not a stable ABI.

## Quotas And Pure Local Selection

The four producers add real YIR entries. Initial native validation exhausted
the unchanged 192-entry quota. Rewriting the print classifier did not reliably
remove outlined entries inside this retained loop profile; those attempts were
not accepted as a solution.

The final witness instead binds `other = -value` in the existing `choose`
helper before selecting between two ready values. Sign-bit negation is total,
helper-free and preserves raw payloads; it is the only computation moved out of
that branch. The generated helper now selects atoms without an extra branch
function. The original choice form remains a negative native control that
exhausts 192 entries. No compiler entry accounting, runtime limit or potentially
failing expression is bypassed or hoisted by this witness change.

All healthy profiles retain 192 helper entries and at most 12 reserved loop
units. Positive, negative and zero-State work controls preserve exact 11/10,
12/11 and 10/9 accept/reject boundaries. Limits eight and 64 remain explicit
entry rejections. A late producer/zero-divisor poison loses to the earlier
parent reservation check. Neither exit refunds reserved work.

## Type, Epoch And Effect Authority

[Source controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_helper_exit_argument_print_tests.rs)
reject both premature exits, bool/i32 producer arguments and cross-width helper
signatures, together with inherited owner/alias/step refusals. Both float widths
and normal/reversed declarations are covered. There is no new source permission.

The existing iteration-effect catalog can describe a helper containing print;
shape admission alone is not a native print grant. The native effect control
adds a print to each producer, then verifies that explicit grants for every
site permit emission. Removing any one producer's site while retaining all
other grants rejects emission in both widths and declaration orders. This
preserves the existing selected-site policy rather than inventing a blanket
ban or allowing caller grants to authorize new transitive effects. This control
checks admission, not execution of an additional effectful feature profile.

## Artifact Delivery

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs)
runs eight healthy sequences before and after 24 failures on fresh builds,
cache hits and two independent source-free restorations in each width: 320
artifact runs. Source, manifest and project cache are removed before restoring.
Binary/container/host-object/envelope/lifecycle/packaging identity, State,
stdout and completion status are checked. Non-finite payload evidence belongs
to native word probes, not finite CLI serialization. Exact trapping prefixes
belong to flushed native probes; CLI output can be buffered.

## Validation

Final selected runs after resolving the witness quota and baseline-router issues:

| Selected Suite | Result |
| --- | --- |
| Compiler source/refusal regression | 64 passed |
| Shared parent-reentry native/reference regression | 65 passed |
| Helper-exit CLI delivery matrix | 1 passed |
| Tensor and complete-predecessor lineage tests | 30 passed |
| Script unit tests | 40 passed |
| Repository file budgets | 2 passed |
| Host absolute-path policy | 1 passed |

Total: 203 distinct selected tests, including eleven helper-exit/lineage tests
(two source, seven native/model, one CLI and one tensor). Preliminary failures,
focused subsets and reruns are not added again. Native/reference execution took
442.91 seconds; CLI delivery took 617.51 seconds with all 320 artifact runs.
These are functional results, not performance measurements, a full-workspace run
or proof of another platform.

The rebuilt ordinary CLI reports 4316 passing drift checks and zero failures.
All 4304 predecessor IDs remain in order alongside twelve helper-exit checks.
Coverage, hierarchy and task lineage are clean. The six source/runtime fields
agree and preserve the complete preceding values/hashes; command history was
syntax-checked and only its quoted inert suffix executed. The existing snapshot
check now tracks the actual beta-0.18.6 commit rather than a stale patch identity.

UTF-8 validation covers 6076 tracked files and local-link validation checks 4826
links. Formatting and whitespace checks pass. No dependency, production admission,
entry/work quota or active/99 scope change accompanies this checkpoint.

## Scope And Next Gate

The selected tensor coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next make the later mode/guard parameters of confirmation helpers independently
fallible. Distinguish quotient completion, subsequent parameter production and
callee entry in source order, retaining once-only evaluation, accepted State,
short-circuit skips, source-free delivery and the same quotas.

This is bounded host CPU evidence, not arbitrary resource/I/O/async capture,
pre-update ordinary-read authority, Linux/GPU parity, a stable ABI, arithmetic
NaN payload guarantees, full native-engine closure or completed self-hosting.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_helper_exit_argument_print state_division_exit_argument_print state_field_exit_argument_print state_grouped_argument_print state_short_circuit_argument_print state_predicate_argument_print state_mixed_argument_print state_continue_argument_print state_break_argument_print state_bound_argument_print fallible_bound_argument_print parent_bound_argument_print child_progress_argument_print parent_reentry_argument_print epoch_local_ child_print_epoch local_scalar_argument_print loop_bool_projections float_compare_argument_print_epoch iteration_effects_exits --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- parent_reentry_argument_print:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- parent_reentry_argument_print::state_helper_exit --test-threads=1 --nocapture
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

All six complete division-exit values are retained. The
[persistent lineage regression](../../tools/nuis/src/dev_tensor_native_argument_lineage_tests.rs)
checks their full SHA-256 values alongside the preceding grouped/field checkpoints.
Historical validation commands remain quoted no-op here-document data. Check
syntax through stdin and execute only the inert suffix; never replay old chains.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `ed49db10cdfcb14135c9bdc51e5f56a3d4eb3647a0a0fba020782f4a1cd0218f` |
| `next_step` | `1c889bcf26f65d6093d6db15d261a319bb0a05715e6bebfed0a10c6232972573` |
| `blocker` | `14c19f1dc8c0e2f5c2b4eadc35084726c70f012fe9d7c51a33b06e2dd1198e6c` |
| `next_action` | `176cfcfd297b3acf2c55bb39d88112ce7bdc39e10b3cdf7ec88c52c42b884ca8` |
| `validation_command` | `b343c2c3c5a1de1495a8142200e85e6ee83654402372dcc510369660c2e56e56` |
| `expected_artifact` | `0db81cf97516cb09a1e9855c1cc8db9eeec929dd088aab2540605ba6261cf447` |
