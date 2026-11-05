# Native State Predicate Argument Print V1

Audit date: 2026-11-04. Checkout baseline: beta-0.18.5, `bbb00651`.
This extends the uncommitted [mixed-exit checkpoint](nuis-native-state-mixed-argument-print-v1.md).
No version bump, commit or release is created here.

## Checked Exit Predicates

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/state_predicate_argument_print.ns)
replaces pure exit comparisons with independent `continue_at` and `break_at`
boolean helpers. Both first perform a checked division and then compare the
current index with their target. The child State update precedes both calls.
A selected continue steps once and skips the break call entirely. Otherwise the
break call executes before any suffix; a selected break does not step.

The helpers have separate native entry observers (603/604) and reference call
counters. Exact observed arguments distinguish reached checks, omitted checks
and their source order. A reversed native source with colliding targets skips a
poisoned continue predicate when break comes first. When both predicates would
fail, that reversed source must fail at the break helper, not at continue.

This composes existing lowering, without a new admission exception or ordinary
epoch-read permission. The
[source controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/parent_reentry_argument_print_tests.rs)
check both float widths, literal/dynamic parents and normal/reversed declarations.
Wrong predicate argument types are asserted as front-end diagnostics; malformed
steps, intervening effects, terminal-control suffixes, forged internal names,
borrowed aliases, unready owners and mutated bounds remain structural refusals.

## Independent Models And Faults

The [profile](../../tools/nuisc/tests/native_application_bridge/state_predicate_argument_print_cases.rs)
contains 31 raw sequences, twelve finite sequences and 30 fault rows per f32/f64
kind. Seventeen callback words append two predicate guards to the fifteen-word
mixed-exit prefix; older profiles retain their layouts.

The [effect model](../../tools/nuisc/tests/native_application_bridge/parent_bound_argument_print_cases.rs)
checks continue, handles its exit, then checks break and handles its exit before
the checked suffix. Predicate call expectations derive from completed child work
and reached continue indices, independently of observed event counts. Requested
work still uses the IEEE-aware State schedule, while completed updates drive
future parent bounds and accepted callback State.

The [native/model controls](../../tools/nuisc/tests/native_application_bridge/state_predicate_argument_print.rs)
cover poisoned predicates in empty loops and skipped branches, later iterations
still reaching their checks, predicates failing before their true exit, and
State-update faults before either predicate. Removing a selected continue makes
its poisoned break predicate fail. Fault rows also retain the preceding producer,
eager/lazy argument, suffix, State publication and post-break producer failures.
Reference sessions preserve the last accepted State through failure/faulted close;
direct native probes require exact flushed output prefixes, not output rollback
or inspection of failed output-buffer memory.

## Budgets And Delivery

Positive and negative schedules retain eleven/ten and twelve/eleven work
boundaries. A reached producer fault wins over reservation and predicate faults
at the same boundary. Helper-entry limits of eight/64 reject, while 192 remains
the unchanged healthy limit; no budget is raised to admit the extra checks.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs)
checks fresh/cache/two-source-free-restoration delivery for f32/f64. Source,
manifest and project cache are removed before restoration. Healthy sequences
surround all faults and preserve State, printed output and binary/container/
host-object/envelope/lifecycle/packaging identity. CLI trap output may be a
buffered prefix; native observer probes provide exact entry-order evidence.

## Validation

All 179 distinct selected tests/checks passed:

- 54 compiler library controls, including front-end type diagnostics and lowering refusals.
- 54 native/reference/oracle controls (332.09 seconds).
- One checked-predicate CLI delivery matrix (569.69 seconds), f32 and f64.
- 40 Python controls.
- Two file-budget checks and one host-path policy check.
- 27 tensor unit tests, including registration order and uniqueness.

Eight tests are new: two source controls, five native/model controls and one
CLI delivery test. Preliminary runs, matrix rows, repeated tests and historical
receipt totals are not added to the 179 count. Existing pure-comparison exit
profiles are covered by the native regression selection; their older CLI
matrices were not rerun in this checkpoint.

The ordinary CLI build, formatting, whitespace and locked offline metadata
checks pass. Static audits cover 6,048 UTF-8 files, 4,776 local documentation
links, and 29 packages across six domains with 74 normal/build edges.
All 4,263 runtime tensor guards pass, retaining the prior 4,253 IDs in order.
The six source/runtime fields and full predecessor hashes agree; coverage,
hierarchy and task lineage are clean. Historical commands remain syntax-valid
inert data, not replayed jobs.

## Scope And Next Gate

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
This is bounded host CPU evidence, not arbitrary resource/I/O/async capture,
stable ABI, Linux/GPU parity, full native-engine closure or completed self-hosting.
Next compose short-circuit boolean exit conditions and prove skipped right-hand
faults, once-only ordered checks, accepted State and reservation priority.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_predicate_argument_print state_mixed_argument_print state_continue_argument_print state_break_argument_print state_bound_argument_print fallible_bound_argument_print parent_bound_argument_print child_progress_argument_print parent_reentry_argument_print epoch_local_ child_print_epoch local_scalar_argument_print loop_bool_projections float_compare_argument_print_epoch iteration_effects_exits --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- parent_reentry_argument_print:: epoch_local_argument_print:: float_compare_argument_print_epoch:: state_loop_exits:: sibling_bounds_native_ready_updates_fit_160_but_entry_limit_100_still_rejects --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- parent_reentry_argument_print::state_predicate --test-threads=1 --nocapture
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

All six complete mixed-exit fields are retained with these hashes. Historical
commands remain quoted no-op here-document data, syntax-checked through stdin
and tested for inertness without replaying their contents.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `c3522967c93ed35f1f93e102782b2d6e40c8104f5c6538c3b8316a47c5f57f89` |
| `next_step` | `d4615686530b203b124bfe86b9a8a243ceea2f78428a1f76cdd867d1bc6fd616` |
| `blocker` | `a353a1a832d6a0542170e15b7266ad9621930536de7972439bc1b66063d4d657` |
| `next_action` | `23541450bf8d45b56d87ced9f53d0e01e10330344718265faac98414782a0571` |
| `validation_command` | `30fe08a6866b2e816a0456c452ff2f549f38d3d0c6ef3ff3085787cd3d7adc28` |
| `expected_artifact` | `4245a813c12fdd3e1b8ebd1bea649d0c391b2deca26199ed00f1c0840c44adcd` |
