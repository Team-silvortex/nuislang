# Native State Grouped Argument Print V1

Audit date: 2026-11-04. Checkout baseline: beta-0.18.5, `bbb00651`.
This extends the uncommitted [flat short-circuit checkpoint](nuis-native-state-short-circuit-argument-print-v1.md).
No version bump, commit or release is created here.

## Grouped Exit Conditions

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/state_grouped_argument_print.ns)
uses `A && (B || C)` for continue and `(D || E) && F` for break.
Each of the six boolean helpers performs a checked operation before returning.
A failed outer conjunction skips its entire inner disjunction; a true inner
left operand skips the rescue call. A false break disjunction skips its
confirmation; a successful continue skips the entire break condition and suffix.

Six native entry observers record each reached operand and its arguments.
[Regrouping probes](../../tools/nuisc/tests/native_application_bridge/state_grouped_argument_print.rs)
change only parentheses. Moving continue's rescue outside the conjunction
makes a formerly skipped check fail. Moving break's confirmation inside its
right branch makes a formerly reached failure disappear, with the same State
and other effects. These are compiled execution checks, not only AST acceptance.

[Source controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/parent_reentry_argument_print_tests.rs)
cover both float widths, dynamic/literal parents and normal/reversed declarations.
Missing, duplicate and wrong steps, effects after terminal exits, forged control
names, mutable bounds and inherited ownership/alias refusals remain rejected.
Wrong rescue/confirmation argument types remain front-end errors. No production
admission exception, syntax change or raised evaluation budget is introduced.

## Independent Models

The [grouped model](../../tools/nuisc/tests/native_application_bridge/state_grouped_argument_print_cases.rs)
retains the flat profile separately. Effective continue/break indices drive
completed work and the existing independent IEEE State schedule. Call counts
use a truth-table traversal, separately from checked lazy trace evaluation.

It covers 31 raw sequences, twelve finite sequences and 20 fault rows per f32/f64
kind. These are focused grouped profiles, not a claim to rerun every predecessor
CLI row. The 25-word input appends rescue and break-confirmation controls to the
unchanged 21-word flat prefix. State-dependent next-parent requests follow
accepted State, not fresh callback decoys.

Cases include rescue selection/veto, a rejected first break followed by a
confirmed fallback, skipped poisoned inner checks, reached checks failing
before either boolean result, later-parent failures, inner-before-confirmation
fault priority and State-update failure before all exit operands. Native probes
check flushed effect prefixes without inspecting output memory after failure.
The reference path retains the last accepted State after failure/faulted close.

## Budgets And Delivery

Positive, negative and fallback schedules retain exact 11/10, 12/11 and 10/9
accept/reject work boundaries. Producer failure precedes reservation and
operand failures at the same boundary. Helper-entry limits eight and 64 reject;
192 remains the unchanged healthy limit.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs)
checks fresh/cache/two-source-free-restoration paths in both float widths.
It deletes the temporary project source, manifest and project cache before
restoration. Healthy sequences run before and after fault batches; State,
output and binary/container/host-object/envelope/lifecycle/packaging identity
must agree. CLI trapping output can be buffered; direct native observers own
exact operand-order evidence.

## Validation

Fresh selected runs in this checkout, without adding historical totals:

| Selected Suite | Result |
| --- | --- |
| Compiler source/refusal regression | 58 passed |
| Shared parent-reentry native/reference regression | 47 passed |
| Grouped CLI delivery matrix | 1 passed |
| Script unit tests | 40 passed |
| Repository file budgets | 2 passed |
| Host absolute-path policy | 1 passed |
| Tensor tests | 27 passed |

Total: 176 distinct selected tests, including eight added tests (two source,
five native/model and one CLI). Preliminary subset runs and reruns are not
added again. The CLI matrix completed in 561.53 seconds, covering 352 artifact
runs across f32/f64 and fresh/cache/two-source-free-restoration delivery.

The rebuilt ordinary CLI reports 4283 passing drift checks, zero failures and
all 4273 predecessor IDs retained in order, with ten new checks. All six
source/runtime fields agree and retain the complete predecessor values/hashes.
Coverage, hierarchy and task lineage remain clean. Historical commands pass
syntax validation through stdin; only the quoted inert suffix was executed.

UTF-8 validation covers 6058 tracked files; local-link validation checks 4794
links. Workspace architecture passes for 29 packages, six domains and 74
normal/build edges. Offline metadata retains 29 members. Formatting and
whitespace checks pass. This is not a full workspace or multi-platform test run.

## Scope And Next Gate

The selected coordinate stays
`standard-library/ns-nova/persistent-application-session`, active/99.
This is bounded host CPU evidence, not Linux/GPU parity, general resource/I/O/
async capture, stable ABI, full native-engine closure or completed self-hosting.
Next feed updated record/scalar fields into grouped exit predicates, rather
than only index/target comparisons, preserving once-only reads, IEEE cases,
short-circuit faults, accepted State and reservation priority.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_grouped_argument_print state_short_circuit_argument_print state_predicate_argument_print state_mixed_argument_print state_continue_argument_print state_break_argument_print state_bound_argument_print fallible_bound_argument_print parent_bound_argument_print child_progress_argument_print parent_reentry_argument_print epoch_local_ child_print_epoch local_scalar_argument_print loop_bool_projections float_compare_argument_print_epoch iteration_effects_exits --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- parent_reentry_argument_print:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- parent_reentry_argument_print::state_grouped --test-threads=1 --nocapture
python3 -m unittest discover -s scripts/tests
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test file_line_limit --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-host-absolute-paths.sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
python3 scripts/check-workspace-architecture.py
cargo metadata --locked --offline --no-deps --format-version 1
cargo fmt --all -- --check
git diff --check
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --locked -j1
target/debug/nuis dev-tensor --json
```

## Predecessor Lineage

All six complete flat short-circuit fields are retained with these hashes.
Historical commands are quoted no-op here-document data. Validate syntax via
stdin and execute only the inert historical suffix, never replay that history.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `f5bd5a9a6ceeb69fd2760db441d19964dee7878f070976c1b2e805d3418f5dac` |
| `next_step` | `5c26f874ba1368e81151131486eb90273b544832f49e6e41b109880eafd64c81` |
| `blocker` | `0dd2d63fb2250ff950a51ad124be636acfd424eea50eee95078951ad4bc39b34` |
| `next_action` | `e0359c6be94edbbc3d369822c28ec62f03f7a4c840f96d1c3e3c29815ad2e517` |
| `validation_command` | `318b17f3d935b7623da6bca7bf34938032eb846d5378bddc8695d4b427d5200e` |
| `expected_artifact` | `adecbf67ef5af96fa3163a9e32e980461a85ecf8c69336c865fd9bf20d222c05` |
