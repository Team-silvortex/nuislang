# Native State Mixed Argument Print V1

Audit date: 2026-11-04. Checkout baseline: beta-0.18.5, `bbb00651`.
This extends the uncommitted [continue checkpoint](nuis-native-state-continue-argument-print-v1.md).
No version bump, commit or release is created here.

## State-Bounded Mixed Exits

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/state_mixed_argument_print.ns)
combines post-update continue and break in each State-bounded child loop.
Continue prints marker 78, steps once and bypasses the break predicate and
fallible suffix. Break prints marker 81, keeps the current index and skips the
suffix without a step. Ordinary iterations print 79, call the checked suffix
and then step. Each parent prints 80 followed by the observed child index.

Distinct cuts cover continue before break, break before the continue index,
absent/out-of-range cuts and collisions. With both cuts zero, the first
continue wins and traversal completes. A separately compiled reversed-guard
source must instead break on zero, publish a different State and observe zero.
Mirrored branches and normal/reversed declarations are source admission controls.
These combinations use existing lowering; no production exception is added.

## Independent Work And Fault Models

The [case profile](../../tools/nuisc/tests/native_application_bridge/state_mixed_argument_print_cases.rs)
contains 31 raw sequences, twelve finite sequences and 22 fault rows per f32/f64
kind. Fifteen callback words append the break index to the fourteen-word continue
prefix. Existing profiles retain their input layouts.

The independent IEEE sign/magnitude schedule reserves the requested child count
but toggles State only for completed updates. The mixed schedule treats a break
at the earlier continue index as unreachable. Suffix counts subtract each
reached exit once, not twice on a collision. The trace model independently pins
side effects and observed indices. Removing a selected continue or break makes
its deliberately poisonous suffix fail. Subsequent callbacks use accepted State
rather than positive initial-value decoys, including changed later-parent bounds.

The [native/model controls](../../tools/nuisc/tests/native_application_bridge/state_mixed_argument_print.rs)
include reached bound, eager, lazy, update, suffix and publication faults, a fault
just before break, and a later producer fault after an early break. Direct native
probes require exact flushed prefixes; reference sessions retain the last accepted
callback State through failure and faulted close. No failed output-buffer memory
is inspected, and printed effects are not promised to roll back.

## Reservations And Delivery

Positive schedules accept eleven work units and reject ten even when early break
completes only two child updates. The negative four/four schedule retains the
twelve/eleven boundary. A reached producer fault precedes child reservation failure.
Helper-entry rejection at eight/64 and the healthy limit of 192 remain unchanged.

The [CLI delivery matrix](../../tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs)
checks fresh build, confirmed cache hit and two restores after source, manifest
and project cache deletion. Healthy sequences surround all fault rows; State,
output and binary/container/host-object/envelope/lifecycle/packaging identities
are compared. CLI trap output may be a buffered prefix, unlike flushed native probes.

Missing, duplicate, mismatched or moved continue steps remain refused, along with
intervening effects, generated-name lookalikes, effects after terminal break,
unready owners, borrowed aliases, branch ownership grants and mutable child bounds.

## Validation

All 172 distinct selected tests/checks passed:

- 52 compiler library controls, including final mixed-exit refusal cases.
- 49 native/reference/oracle controls (305.77 seconds).
- One mixed-exit CLI delivery matrix (468.88 seconds), f32 and f64.
- 40 Python controls.
- Two file-budget checks and one host-path policy check.
- 27 tensor unit tests, including registration order and uniqueness.

Seven tests are new: two source controls, four native/model controls and one
CLI delivery test. Repeated runs, matrix rows and historical receipt totals
are not added to the 172 count. Existing standalone break/continue profiles
are covered by the native regression selection; their older CLI delivery
matrices were not rerun in this checkpoint.

The ordinary CLI build, formatting, whitespace and locked offline metadata
checks pass. Static audits cover 6,043 UTF-8 files, 4,766 local documentation
links, and 29 packages across six domains with 74 normal/build edges.
All 4,253 runtime tensor guards pass, retaining the prior 4,243 IDs in order.
The six current source/runtime fields and full predecessor hashes agree;
coverage, hierarchy and task lineage are clean. Historical commands are
syntax-valid inert data, not replayed jobs.

## Scope And Next Gate

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
These are bounded host CPU witnesses, not arbitrary resource/I/O/async capture,
stable ABI, Linux/GPU parity, full native-engine closure or completed self-hosting.
Next add fallible exit predicates, checking reached and skipped predicate faults,
source-order priority, once-only effects and reservation accounting.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_mixed_argument_print state_continue_argument_print state_break_argument_print state_bound_argument_print fallible_bound_argument_print parent_bound_argument_print child_progress_argument_print parent_reentry_argument_print epoch_local_ child_print_epoch local_scalar_argument_print loop_bool_projections float_compare_argument_print_epoch iteration_effects_exits --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- parent_reentry_argument_print:: epoch_local_argument_print:: float_compare_argument_print_epoch:: state_loop_exits:: sibling_bounds_native_ready_updates_fit_160_but_entry_limit_100_still_rejects --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- parent_reentry_argument_print::state_mixed --test-threads=1 --nocapture
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

All six complete continue fields are retained with the hashes below. Historical
commands remain quoted no-op here-document data, syntax-checked through stdin
and checked for inertness without replaying their contents.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `a8d78c2ff1ee948b36ad44ad3507de5a8e4e3fe5966c6e53548d31a0a0757a96` |
| `next_step` | `cd1c27ce1ec195e3b6d5eec47380fd1ed43c5c14c9c0cec26465c2dc7fe3cd6c` |
| `blocker` | `2f08ccce92fcb8eac1af463eae1bcafa7ecf14db1faa0ef2348d3962f678c0b9` |
| `next_action` | `884a1023d2cfad612b1cecd61e7d7f3706f0844100f1c8f727bafc4d14c73fcc` |
| `validation_command` | `c926aeaf1349388e2c2dd821f9ac0a08f4e6048c87a3e126db7d45bcccb2a885` |
| `expected_artifact` | `93a8db3be9c6cbb27ea0cbaf158902985a379f0293553d262187a8889fde96df` |
