# Native State Short-Circuit Argument Print V1

Audit date: 2026-11-04. Checkout baseline: beta-0.18.5, `bbb00651`.
This extends the uncommitted [checked-predicate checkpoint](nuis-native-state-predicate-argument-print-v1.md).
No version bump, commit or release is created here.

## Short-Circuit Exit Conditions

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/state_short_circuit_argument_print.ns)
uses two checked boolean helpers for each exit: a conjunction confirms continue,
and a disjunction supplies a fallback break. All four helpers can fail before
returning their comparison result. False left operands skip conjunction RHS;
true left operands skip disjunction RHS. A selected continue also skips both
break operands and the suffix. Break retains its unstepped observed index.

Four native entry markers (603/605/604/606) and reference counters distinguish
which operands ran, their arguments and their order. A source with false AND
left and true OR left succeeds despite poisoned right-hand guards. Separately
compiled operand-swapped versions must fail in the previously skipped RHS.
This is an execution check, not merely a syntactic or boolean-result comparison.

The [source controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/parent_reentry_argument_print_tests.rs)
cover both float widths, literal/dynamic parents, normal/reversed declarations
and swapped operands. Predicate argument type failures remain front-end
errors; malformed steps, intervening effects, terminal suffixes, forged internal
names, mutable bounds, borrowed aliases and unready owners remain refused.
No new source syntax or production admission exception is added.

## Independent State And Call Models

The [profile and model](../../tools/nuisc/tests/native_application_bridge/state_short_circuit_argument_print_cases.rs)
contain 35 raw sequences, sixteen finite sequences and 38 fault rows per f32/f64
kind. Twenty-one callback words append confirmation/fallback controls to the
seventeen-word predicate prefix; earlier layouts are unchanged.

The reservation schedule derives an effective continue index and the earliest
non-shadowed break target. It uses the independent IEEE State model, not trace
markers. Call counts enumerate completed indices and operand truth tables;
trace evaluation separately performs lazy checked calls. A denied confirmation
can expose a break at the same index. A fallback break can shorten completed
work and change the next parent's requested bound. Subsequent callbacks retain
accepted State rather than positive initial-value decoys.

The [native/model controls](../../tools/nuisc/tests/native_application_bridge/state_short_circuit_argument_print.rs)
pin final State, indices, call counts, ABI prefixes and exact fault prefixes.
They cover RHS checks that fail before returning either true or false, later-parent
RHS faults, left-before-right fault priority and State-update failure before any
exit operand. The inherited producer, eager/lazy, predicate, suffix and publication
fault rows remain included. Reference failure/faulted-close keeps the last accepted
State; direct native probes check flushed effects without inspecting failed output
memory or promising output rollback.

## Budgets And Delivery

The fallback schedule requests 0/3/0/3 children and accepts ten work units,
rejecting nine before the last body. Ordinary positive and negative schedules
retain eleven/ten and twelve/eleven. Producer failure wins over reservation and
operand failure at the same boundary. Eight/64 helper-entry controls reject;
192 remains the unchanged healthy limit despite extra checked operands.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs)
checks fresh/cache/two-source-free-restoration delivery for both float widths.
Source, manifest and project cache are removed before restore. Healthy sequences
surround every fault batch, preserving State, output and binary/container/host-object/
envelope/lifecycle/packaging identity. CLI trap output may be buffered; native entry
observers supply exact operand-order evidence.

## Validation

Fresh runs in this checkout, without adding historical totals:

| Selected Suite | Result |
| --- | --- |
| Compiler source/refusal regression | 56 passed |
| Native/reference regression | 59 passed |
| CLI delivery matrix | 1 passed |
| Script unit tests | 40 passed |
| Repository file budgets | 2 passed |
| Host absolute-path policy | 1 passed |
| Tensor tests | 27 passed |

Total: 186 distinct selected tests, including eight added tests (two source,
five native/model and one CLI). Preliminary subset runs and reruns are not
added again. The CLI matrix completed in 813.38 seconds, covering 560 artifact
runs across f32/f64 and fresh/cache/two-source-free-restoration delivery.

The first tensor run caught one stale early-break text guard after the shared
model adopted optional short-circuit decisions. Its existing ID now checks
the break-target dispatch and its preserved index-comparison fallback; no
guard was removed. The rerun passes all 27 tensor tests.

The rebuilt ordinary CLI reports 4273 passing drift checks, zero failures,
all 4263 predecessor IDs retained in order and ten newly registered checks.
All six source/runtime fields agree and retain their complete predecessor
values and hashes.
Coverage, hierarchy and task lineage remain clean; history was syntax-checked
through stdin and only the quoted inert suffix was executed.

UTF-8 validation covers 6053 tracked files and local-link validation checks
4785 links. Workspace architecture passes for 29 packages, six domains and
74 normal/build edges; offline metadata retains 29 workspace members.
Formatting and whitespace checks pass. These checks do not imply a full
workspace test run or a new platform acceptance.

## Scope And Next Gate

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
This is bounded host CPU evidence, not arbitrary resource/I/O/async capture,
stable ABI, Linux/GPU parity, full native-engine closure or completed self-hosting.
Next nest grouped mixed logical exit trees and check grouping-sensitive calls,
skipped inner faults, once-only effects, accepted State and reservation priority.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_short_circuit_argument_print state_predicate_argument_print state_mixed_argument_print state_continue_argument_print state_break_argument_print state_bound_argument_print fallible_bound_argument_print parent_bound_argument_print child_progress_argument_print parent_reentry_argument_print epoch_local_ child_print_epoch local_scalar_argument_print loop_bool_projections float_compare_argument_print_epoch iteration_effects_exits --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- parent_reentry_argument_print:: epoch_local_argument_print:: float_compare_argument_print_epoch:: state_loop_exits:: sibling_bounds_native_ready_updates_fit_160_but_entry_limit_100_still_rejects --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- parent_reentry_argument_print::state_short_circuit --test-threads=1 --nocapture
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

All six complete checked-predicate fields are retained with these hashes.
Historical commands remain quoted no-op here-document data, syntax-checked
through stdin and checked for inertness without replaying their contents.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `d03e794640fd8f6ecf13dfa60985ef97666461a7be1f12a506f610d4251ed8c3` |
| `next_step` | `67393bc25d7d7c4bca34474e63527a6d1626a1eacec2dbece56cbbea472576ec` |
| `blocker` | `e7f404bdecbf52be697ee6921008c8f0d138b1bb1ba9df9dc656a3d99da4d98e` |
| `next_action` | `9bda383e0ab86f1a95dd255d0dbf9a5774db4b86615f6964f1f4b4d87d48d266` |
| `validation_command` | `b4be437c2025e3053db2f029f3aeba161a23a2843c954a5cf589cdf249950eb6` |
| `expected_artifact` | `5e74a0168e2d0678d7a1cd5be05752dffa39db9e8c568c33b69a71d405bbfade` |
