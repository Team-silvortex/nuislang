# Native State Record Parameter Exit Argument Print V1

Audit date: 2026-11-07. Checkout baseline: beta-0.18.6, `696d2bd7`.
This extends the uncommitted [later-parameter checkpoint](nuis-native-state-parameter-exit-argument-print-v1.md).
Earlier fixtures and dated receipts remain intact. No version bump, commit,
tag or release is created here.

## Updated Record Parameter Values

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/state_record_parameter_exit_argument_print.ns)
passes updated `current.total` to all four mode/guard producers, alongside their
i64 base values and independent fault inputs. After its checked operation, a
producer tests the field with ordered `value < 0.0`. A negative value XORs the
mode with one or subtracts one from the guard using i64 wrapping arithmetic.
Otherwise the base is returned unchanged.

Positive/negative zero and NaN do not satisfy that ordered predicate; negative
finite values and negative infinity do. The field is the original updated
record value, not the rounded quotient: underflow and negative/NaN divisors
separate these values. A derived guard can turn one into a failing zero or turn
an input zero into a valid minus one. Producer failure still precedes derivation,
and confirmation failure remains separate from producing the guard value.

Both grouped exit trees, all six ordered failure stages and accepted callback
State remain in force. [Source controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_record_parameter_exit_argument_print_tests.rs)
reject premature exits and wrong float/bool/i32 arguments or producer signatures,
in both float widths and declaration orders. Earlier print-only epochs cannot
lend readiness to these ordinary field arguments. No production admission rule
or syntax is changed.

## Independent State And Value Models

The [record-parameter model](../../tools/nuisc/tests/native_application_bridge/state_record_parameter_exit_argument_print_cases.rs)
uses raw IEEE bit classification for the field predicate, sign-bit State
transitions, full-width mode XOR and wrapping guard subtraction. Its independent
schedule accounts for State-dependent mode results when predicting completed
children, parent reservations, observed indices, six predicate counts and suffixes.
The preceding parameter/division schedule is not reused for these new decisions.
A separate checked lazy trace model records actual operand and parameter entries
before their respective failures.

Each width covers 34 raw sequences, ten finite sequences and 34 fault rows.
Raw cases include both zero signs, subnormals, normal values, overflow, infinities,
NaNs and high/signed i64 modes and guards. Underflow demonstrates why a negative field must
not be replaced by its negative-zero quotient. Negative divisors and NaN divisors
likewise separate original field selection from arithmetic classification.

Native parameter observers now record the exact f32/f64 field word, base i64
and producer-fault input. Only arithmetic NaN quotient observations retain the
predecessor normalization; field/carry payloads remain raw. Reference call counts
use the new independent State schedule for both quotient producers and parameter
producers. Older profiles retain their existing count models.

The 35-word input remains byte-for-byte the same layout as the preceding profile.
No slot is appended or repurposed. The independent record profile converts only
to its predecessor's transport representation, not its old State schedule.
The callback still publishes one float State word; an additional persistent
control scalar is a separate next gate, not claimed here.

## Failure Priority And Accepted State

The fault matrix covers three earlier bound/update/final-publication failures,
all six stages at first and later continue/break selections, both zero divisor
signs, and fallback-selected mode/guard/confirmation failures. Colliding failures
keep the order: left producer, right producer, division, mode producer, guard
producer, confirmation check. Later-parent rows observe seven completed updates
before failure; skipped stages remain unentered.

The accepted prefix first publishes negative State, then positive State. Failing
events carry negative initial-value decoys, which cannot replace the accepted
positive State or the field produced by the current update. Reference sessions
retain the accepted State after failure and faulted close. Native probes validate
flushed output prefixes without inspecting pending memory after a trap.

The [stale-parent control](../../tools/nuisc/tests/native_application_bridge/state_record_parameter_exit_argument_print.rs)
changes only the four field arguments from `current.total` to the ready parent
`total`. With identical callback inputs, updated negative fields produce mode
zero and guard zero, failing confirmation; stale positive fields instead produce
mode one and guard one, and the callback succeeds. A separate predecessor model
constructs that stale trace. The control observes both raw field arguments and
the different outcome, rather than treating any ready ancestor as equivalent.

## Returned Values, Quotas And Effects

Return-value controls XOR bit 32 into the base input and decode it inside the
producer before deriving the State-dependent result. Raw entry bases change,
but the resulting mode/guard and State must not. Separate mode and guard controls
cover finite, zero and NaN fields, wide modes and derived zero guards that still
fail at confirmation. Substituting the original input for the returned value
cannot satisfy these observations.

Additional guard rows exercise both caller-offset wrapping and a producer
receiving `i64::MIN` and returning `i64::MAX` for a negative field. Exact native
callee words and reference execution cover both confirmations, as well as
positive/negative values beyond 32 bits. These rows first exposed a non-wrapping
subtraction in the new test oracle; it now uses the language's wrapping semantics.
The source program and production arithmetic implementation were unchanged.

The normal helper computes its total integer alternative before selecting
between ready values. A negative control moves the same arithmetic into the
selected branch and verifies exhaustion at the unchanged 192-entry budget.
Thus the proof does not exempt outlined function entries or increase the budget.
Limits eight and 64 remain independent negative controls.

Loop reservations remain at most 12, with exact 11/10, 12/11 and 10/9 work
accept/reject boundaries. A later poisoned parameter and zero divisor lose to
the earlier reservation failure. Neither continue nor break refunds work.
The preceding inline record-update check is unchanged and still runs before
record publication and all exit parameters.

An effect-admission control adds a print site to each record-dependent producer.
All four explicit selected-site grants permit native emission; withholding any
one still rejects it. A changed helper signature or ready State field cannot
inherit a caller's effect grant. This checks the existing policy, not execution
acceptance for an additional effectful feature.

## Artifact Delivery

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs)
runs ten healthy sequences before and after 34 faults for fresh builds, cache
hits and two independent source-free restorations in each width: 432 artifact
runs. Source, manifest and project cache are removed before restoration. Binary,
container, host-object, envelope, lifecycle and packaging identities remain
pinned, together with State, stdout and completion status.

Exact trapping prefixes belong to direct native probes because CLI stdout can
be buffered. Non-finite payload evidence belongs to native word transport, not
finite CLI serialization. No route infers successful completion or publication
of a failed pending callback State.

## Validation

Selected runs in this checkout, without adding predecessor receipt totals:

| Selected Suite | Result |
| --- | --- |
| Compiler source/refusal regression | 68 passed |
| Shared parent-reentry native/reference regression | 79 passed |
| Record-parameter CLI delivery matrix | 1 passed |
| Tensor and complete-predecessor lineage tests | 32 passed |
| Script unit tests | 40 passed |
| Repository file budgets | 2 passed |
| Host absolute-path policy | 1 passed |

Total: 223 distinct selected tests, including eleven record-parameter/lineage
tests (two source, seven native/model, one CLI and one tensor). The full shared
native/reference run took 604.96 seconds; all seven record-profile tests were
rerun after the wide-guard additions and oracle correction, passing in 60.38
seconds. The unchanged finite CLI profiles completed all 432 artifact runs in
910.18 seconds. Subsets, initial failures and reruns are not added again. These
are functional results, not performance measurements, a full-workspace run or
another-platform acceptance claim.

The rebuilt ordinary CLI reports 4340 passing drift checks, zero failures and
all 4328 predecessor IDs retained in order alongside twelve record-parameter
checks. Coverage, hierarchy and task lineage are clean. All six source/runtime
fields agree and retain their complete predecessor values/hashes. Historical
command syntax and only the quoted inert suffix were checked, without replaying
old validation chains.

UTF-8 validation covers 6088 tracked files and 4846 local documentation links
resolve. Formatting and whitespace checks pass. Dependencies, production
admission, transport/work/entry limits and the active/99 scope marker are unchanged.

## Scope And Next Gate

The selected coordinate stays
`standard-library/ns-nova/persistent-application-session`, active/99.
Next carry a separate control scalar in published callback State alongside the
float field and use its accepted updated value in confirmation parameters.
Keep existing one-word profiles intact and verify complete multi-field publication,
failed-event/faulted-close retention, event decoys and source-free restoration
under unchanged permissions and quotas.

This checkpoint is bounded host CPU evidence, not arbitrary resource/I/O/async
capture, pre-update ordinary reads, Linux/GPU parity, a stable ABI, arithmetic
NaN payload guarantees, complete native-engine closure or self-hosting.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_record_parameter_exit_argument_print state_parameter_exit_argument_print state_helper_exit_argument_print state_division_exit_argument_print state_field_exit_argument_print state_grouped_argument_print state_short_circuit_argument_print state_predicate_argument_print state_mixed_argument_print state_continue_argument_print state_break_argument_print state_bound_argument_print fallible_bound_argument_print parent_bound_argument_print child_progress_argument_print parent_reentry_argument_print epoch_local_ child_print_epoch local_scalar_argument_print loop_bool_projections float_compare_argument_print_epoch iteration_effects_exits --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- parent_reentry_argument_print:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- parent_reentry_argument_print::state_record_parameter_exit --test-threads=1 --nocapture
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

All six complete later-parameter values are retained. The
[persistent lineage regression](../../tools/nuis/src/dev_tensor_native_argument_lineage_tests.rs)
checks the complete SHA-256 values below; older checkpoints keep their guards.
Historical validation commands remain quoted no-op here-document data. Check
syntax through stdin and execute only the inert suffix, never historical chains.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `db09b92bb1e678db5b62a924ed9d57fda653257f2e7a912e3d6bb2b47faf4998` |
| `next_step` | `5001213779cb55d5f56a89b19cff6a00e44c37df255a20c97edabf05ec1b2d39` |
| `blocker` | `e146d17c077a272b15730f280091ae11bdfa5bdbb6babc564c301d74105d0888` |
| `next_action` | `ea6bf29f4aca0115b275be2224d474265d3665cf3db646ee7b6d1cd930ae48cc` |
| `validation_command` | `187b1452289be1ca7dab7ecf5a76c04cf2e688379f810ba01209acf65836350e` |
| `expected_artifact` | `2197dc2e11d723d79e10d23cb41f531d2ada9b4cb2e6788a06aed77d5517c711` |
